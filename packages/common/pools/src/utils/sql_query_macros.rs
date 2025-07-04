/// Base retry for query retry backoff.
pub const QUERY_RETRY_MS: usize = 100;
/// Maximum times a query is retried.
pub const MAX_QUERY_RETRIES: usize = 6;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("max sql retries, last error: {0}")]
	MaxSqlRetries(sqlx::Error),
}

lazy_static::lazy_static! {
	/// Rate limit used to limit creating a stampede of connections to the database.
	pub static ref CONN_ACQUIRE_RATE_LIMIT: governor::RateLimiter<
		governor::state::direct::NotKeyed,
		governor::state::InMemoryState,
		governor::clock::DefaultClock,
		governor::middleware::NoOpMiddleware
	> = governor::RateLimiter::direct(
		// Limit how fast the pool size can grow in order to encourage reusuing existing
		// connections instead of opening new ones.
		governor::Quota::per_minute(std::num::NonZeroU32::new(10 * 60).unwrap())
			.allow_burst(std::num::NonZeroU32::new(500).unwrap())
	);
}

// MARK: Metrics
#[macro_export]
macro_rules! __sql_query_metrics_acquire {
	($acquire_timer:ident) => {
		// Start timer
		let $acquire_timer = tokio::time::Instant::now();
	};
}

#[macro_export]
macro_rules! __opt_indoc {
	($lit:literal) => {
		indoc!($lit)
	};
	($other:expr) => {
		$other
	};
}

#[macro_export]
macro_rules! __sql_query_metrics_start {
	($ctx:expr, $action:expr, $acquire_timer:ident, $start_timer:ident) => {{
		let ctx = &$ctx;
		let location = concat!(file!(), ":", line!(), ":", column!());

		// Count acquire
		let acquire_duration = $acquire_timer.elapsed().as_secs_f64();
		rivet_pools::metrics::SQL_ACQUIRE_DURATION
			.with_label_values(&[stringify!($action), ctx.name(), location])
			.observe(acquire_duration);

		// Count metric
		rivet_pools::metrics::SQL_QUERY_TOTAL
			.with_label_values(&[stringify!($action), ctx.name(), location])
			.inc();
	}

	// Start timer
	let $start_timer = tokio::time::Instant::now();};
}

#[macro_export]
macro_rules! __sql_query_metrics_finish {
	($ctx:expr, $action:expr, $start_timer:ident) => {{
		let ctx = &$ctx;

		let duration = $start_timer.elapsed().as_secs_f64();

		// Log query
		let location = concat!(file!(), ":", line!(), ":", column!());
		tracing::debug!(%location, ty = %stringify!($rv), dt = ?duration, action = stringify!($action), "sql query");

		// Count metric
		rivet_pools::metrics::SQL_QUERY_DURATION
			.with_label_values(&[stringify!($action), ctx.name(), location])
			.observe(duration);
	}};
}

// MARK: Helpers
/// Acquire a connection from the pool with a rate limiting mechanism to reuse existing connecting
/// if possible.
///
/// First, attempt to acquire a connection with `.try_acquire()` if one already exists to use.
///
///	If none are available, check the rate limit to see if we can create a new connection. If so,
///	call `.acquire()` which will create a new connection in the pool.
///
///	If the rate limit is exhausted, this will sleep and try again.
///
///	This is in order to prevent connection spikes that will create a lot of connections in
///	parallel, adding strain to the database & slowing down the query.
///
///	Without this, initial queries are very very slow because it's slower to create 32 new connections
///	than make 4 RTT queries over 8 existing connections.
#[macro_export]
macro_rules! __sql_acquire {
	($ctx:expr, $driver:expr) => {{
		let location = concat!(file!(), ":", line!(), ":", column!());

		let mut tries = 0;
		let (conn, acquire_result) = loop {
			tries += 1;

			// Attempt to use an existing connection
			if let Some(conn) = $driver.try_acquire() {
				break (conn, "try_acquire");
			} else {
				// Check if we can create a new connection
				if $crate::utils::sql_query_macros::CONN_ACQUIRE_RATE_LIMIT
					.check()
					.is_ok()
				{
					// Create a new connection
					break ($driver.acquire().await?, "acquire");
				} else {
					// TODO: Backoff
					tokio::time::sleep(std::time::Duration::from_millis(1)).await;
				}
			}
		};

		rivet_pools::metrics::SQL_ACQUIRE_TOTAL
			.with_label_values(&[stringify!($action), $ctx.name(), location, acquire_result])
			.inc();
		rivet_pools::metrics::SQL_ACQUIRE_TRIES
			.with_label_values(&[stringify!($action), $ctx.name(), location, acquire_result])
			.inc_by(tries);

		conn
	}};
}

#[macro_export]
macro_rules! __sql_query {
	([$ctx:expr, $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		async {
			use sqlx::Acquire;

			// Acquire connection
			$crate::__sql_query_metrics_acquire!(_acquire);
			let driver = $driver;
			let mut conn = $crate::__sql_acquire!($ctx, driver);

			$crate::__sql_query_metrics_start!($ctx, execute, _acquire, _start);

			let mut backoff = $crate::__rivet_util::Backoff::new(
				4,
				None,
				$crate::utils::sql_query_macros::QUERY_RETRY_MS,
				50
			);
			let mut i = 0;

			// Retry loop
			let res = loop {
				let query = sqlx::query($crate::__opt_indoc!($sql))
				$(
					.bind($bind)
				)*;

				match query.execute(&mut *conn).await {
					Err(err) => {
						i += 1;
						if i > $crate::utils::sql_query_macros::MAX_QUERY_RETRIES {
							break Err(
								sqlx::Error::Io(
									std::io::Error::new(
										std::io::ErrorKind::Other,
										$crate::utils::sql_query_macros::Error::MaxSqlRetries(err),
									)
								)
							);
						}

						use sqlx::Error::*;
						match &err {
							// Retry other errors with a backoff
							Database(_) | Io(_) | Tls(_) | Protocol(_) | PoolTimedOut | PoolClosed
							| WorkerCrashed => {
								tracing::warn!(?err, "query retry ({i}/{})", $crate::utils::sql_query_macros::MAX_QUERY_RETRIES);
								backoff.tick().await;
							}
							// Throw error
							_ => break Err(err),
						}
					}
					x => break x,
				}
			};

			$crate::__sql_query_metrics_finish!($ctx, execute, _start);

			res.map_err(Into::<GlobalError>::into)
		}
		.instrument(tracing::info_span!("sql_query"))
	};
	([$ctx:expr, @tx $tx:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		async {
			let query = sqlx::query($crate::__opt_indoc!($sql))
			$(
				.bind($bind)
			)*;

			// Execute query
			$crate::__sql_query_metrics_acquire!(_acquire);
			$crate::__sql_query_metrics_start!($ctx, execute, _acquire, _start);
			let res = query.execute(&mut **$tx).await.map_err(Into::<GlobalError>::into);
			$crate::__sql_query_metrics_finish!($ctx, execute, _start);

			res
		}
		.instrument(tracing::info_span!("sql_query"))
	};
	([$ctx:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query!([$ctx, &$ctx.crdb().await?] $sql, $($bind),*)
	};
}

#[macro_export]
macro_rules! __sql_query_as {
	([$ctx:expr, $rv:ty, $action:ident, $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		async {
			use sqlx::Acquire;

			let query = sqlx::query_as::<_, $rv>($crate::__opt_indoc!($sql))
			$(
				.bind($bind)
			)*;

			// Acquire connection
			$crate::__sql_query_metrics_acquire!(_acquire);
			let driver = $driver;
			let mut conn = $crate::__sql_acquire!($ctx, driver);

			// Execute query
			$crate::__sql_query_metrics_start!($ctx, $action, _acquire, _start);
			let res = query.$action(&mut *conn).await.map_err(Into::<GlobalError>::into);
			$crate::__sql_query_metrics_finish!($ctx, $action, _start);

			res
		}
		.instrument(tracing::info_span!("sql_query_as"))
	};
	([$ctx:expr, $rv:ty, $action:ident, @tx $tx:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		async {
			let query = sqlx::query_as::<_, $rv>($crate::__opt_indoc!($sql))
			$(
				.bind($bind)
			)*;

			// Execute query
			$crate::__sql_query_metrics_acquire!(_acquire);
			$crate::__sql_query_metrics_start!($ctx, $action, _acquire, _start);
			let res = query.$action(&mut **$tx).await.map_err(Into::<GlobalError>::into);
			$crate::__sql_query_metrics_finish!($ctx, $action, _start);

			res
		}
		.instrument(tracing::info_span!("sql_query_as"))
	};
	([$ctx:expr, $rv:ty, $action:ident] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, $action, &$ctx.crdb().await?] $sql, $($bind),*)
	};
}

/// Returns a query without being wrapped in an async block, and therefore cannot time the query.
/// Used for the `fetch` function.
#[macro_export]
macro_rules! __sql_query_as_raw {
	([$ctx:expr, $rv:ty, $action:ident, $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		// We can't record metrics for this because we can't move the `await` in to this macro
		sqlx::query_as::<_, $rv>($crate::__opt_indoc!($sql))
		$(
			.bind($bind)
		)*
		.$action($driver)
	};
	// TODO: This doesn't work with `fetch`
	([$ctx:expr, $rv:ty, $action:ident] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as_raw!([$ctx, $rv, $action, &$ctx.crdb().await?] $sql, $($bind),*)
	};
}

// MARK: Specific actions
#[macro_export]
macro_rules! sql_execute {
	($($arg:tt)*) => {
		__sql_query!($($arg)*)
	};
}

#[macro_export]
macro_rules! sql_fetch {
	([$ctx:expr, $rv:ty, $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as_raw!([$ctx, $rv, fetch, $driver] $sql, $($bind),*)
	};
}

#[macro_export]
macro_rules! sql_fetch_all {
	([$ctx:expr, $rv:ty, $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_all, $driver] $sql, $($bind),*)
	};
	([$ctx:expr, $rv:ty, @tx $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_all, @tx $driver] $sql, $($bind),*)
	};
	([$ctx:expr, $rv:ty] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_all] $sql, $($bind),*)
	};
}

#[macro_export]
macro_rules! sql_fetch_many {
	([$ctx:expr, $rv:ty, $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_many, $driver] $sql, $($bind),*)
	};
	([$ctx:expr, $rv:ty, @tx $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_many, @tx $driver] $sql, $($bind),*)
	};
	([$ctx:expr, $rv:ty] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_many] $sql, $($bind),*)
	};
}

#[macro_export]
macro_rules! sql_fetch_one {
	([$ctx:expr, $rv:ty, $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_one, $driver] $sql, $($bind),*)
	};
	([$ctx:expr, $rv:ty, @tx $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_one, @tx $driver] $sql, $($bind),*)
	};
	([$ctx:expr, $rv:ty] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_one] $sql, $($bind),*)
	};
}

#[macro_export]
macro_rules! sql_fetch_optional {
	([$ctx:expr, $rv:ty, $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_optional, $driver] $sql, $($bind),*)
	};
	([$ctx:expr, $rv:ty, @tx $driver:expr] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_optional, @tx $driver] $sql, $($bind),*)
	};
	([$ctx:expr, $rv:ty] $sql:expr, $($bind:expr),* $(,)?) => {
		__sql_query_as!([$ctx, $rv, fetch_optional] $sql, $($bind),*)
	};
}
