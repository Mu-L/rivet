use chirp_client::prelude::*;
use fdb_util::prelude::*;
use global_error::GlobalResult;
use rivet_pools::prelude::*;
use std::fmt::{self, Debug};

#[derive(Clone)]
pub struct Connection {
	pub(crate) client: chirp_client::Client,
	pub(crate) pools: rivet_pools::Pools,
	pub(crate) cache: rivet_cache::Cache,
}

impl Connection {
	pub fn new(
		client: chirp_client::Client,
		pools: rivet_pools::Pools,
		cache: rivet_cache::Cache,
	) -> Self {
		Connection {
			client,
			pools,
			cache,
		}
	}

	/// Creates a new `Connection` with the appropriate context in the `Client` to make requests. Used when
	// calling another operation.
	pub fn wrap(&self, parent_req_id: Uuid, ray_id: Uuid, name: &str) -> Connection {
		// Not the same as the operation ctx's ts because this cannot be overridden by debug start ts
		let ts = rivet_util::timestamp::now();
		let trace_entry = chirp_client::TraceEntry {
			context_name: name.to_string(),
			req_id: Some(parent_req_id.into()),
			ts,
		};

		Connection::new(
			(*self.client).clone().wrap(parent_req_id, ray_id, {
				let mut x = self.client.trace().to_vec();
				x.push(trace_entry);
				x
			}),
			self.pools.clone(),
			self.cache.clone(),
		)
	}

	pub fn chirp(&self) -> &chirp_client::Client {
		&self.client
	}

	pub fn cache(&self) -> rivet_cache::RequestConfig {
		self.cache.clone().request()
	}

	pub fn cache_handle(&self) -> rivet_cache::Cache {
		self.cache.clone()
	}

	pub fn pools(&self) -> &rivet_pools::Pools {
		&self.pools
	}

	pub async fn nats(&self) -> Result<NatsPool, rivet_pools::Error> {
		self.pools.nats()
	}

	pub async fn crdb(&self) -> Result<CrdbPool, rivet_pools::Error> {
		self.pools.crdb()
	}

	pub async fn redis_chirp(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.pools.redis_chirp()
	}

	pub async fn redis_cache(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.pools.redis_cache()
	}

	pub async fn redis_cdn(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.pools.redis("persistent")
	}

	pub async fn redis_job(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.pools.redis("persistent")
	}

	pub async fn redis_mm(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.pools.redis("persistent")
	}

	pub async fn redis_chirp_ephemeral(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.pools.redis("ephemeral")
	}

	pub async fn fdb(&self) -> Result<FdbPool, rivet_pools::Error> {
		self.pools.fdb()
	}

	#[tracing::instrument(skip_all, fields(?key, read_only))]
	pub async fn sqlite(
		&self,
		key: impl TuplePack + Debug,
		read_only: bool,
	) -> Result<SqlitePool, rivet_pools::Error> {
		self.pools.sqlite(key, read_only).await
	}

	pub fn perf(&self) -> &chirp_perf::PerfCtx {
		self.client.perf()
	}

	pub async fn clickhouse(&self) -> GlobalResult<ClickHousePool> {
		self.pools.clickhouse()
	}

	pub async fn clickhouse_inserter(&self) -> GlobalResult<ClickHouseInserterHandle> {
		self.pools.clickhouse_inserter()
	}
}

impl std::ops::Deref for Connection {
	type Target = chirp_client::Client;

	fn deref(&self) -> &Self::Target {
		self.chirp()
	}
}

impl Debug for Connection {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Connection").finish()
	}
}
