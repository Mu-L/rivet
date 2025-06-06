use std::{
	sync::Arc,
	time::{Duration, Instant},
};

use futures_util::StreamExt;
use global_error::{GlobalError, GlobalResult};
use opentelemetry::trace::SpanContext;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::watch;
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

use crate::{
	activity::{Activity, ActivityInput},
	builder::{workflow as builder, WorkflowRepr},
	ctx::{ActivityCtx, ListenCtx, MessageCtx, VersionedWorkflowCtx},
	db::{DatabaseHandle, PulledWorkflowData},
	error::{WorkflowError, WorkflowResult},
	executable::{AsyncResult, Executable},
	history::{
		cursor::{Cursor, HistoryResult},
		event::{EventId, SleepState},
		location::{Coordinate, Location},
		removed::Removed,
		History,
	},
	listen::{CustomListener, Listen},
	message::Message,
	metrics,
	registry::RegistryHandle,
	signal::Signal,
	utils::{
		time::{DurationToMillis, TsToMillis},
		GlobalErrorExt,
	},
	workflow::{Workflow, WorkflowInput},
};

/// Retry interval for failed db actions
const DB_ACTION_RETRY: Duration = Duration::from_millis(150);
/// Most db action retries
const MAX_DB_ACTION_RETRIES: usize = 5;

// NOTE: Clonable because of inner arcs
#[derive(Clone)]
pub struct WorkflowCtx {
	workflow_id: Uuid,
	/// Name of the workflow to run in the registry.
	name: String,
	create_ts: i64,
	ts: i64,
	ray_id: Uuid,
	version: usize,
	// Used for activity retry backoff
	wake_deadline_ts: Option<i64>,

	registry: RegistryHandle,
	db: DatabaseHandle,

	config: rivet_config::Config,
	conn: rivet_connection::Connection,

	/// Input data passed to this workflow.
	input: Arc<Box<serde_json::value::RawValue>>,
	/// All events that have ever been recorded on this workflow.
	event_history: History,
	cursor: Cursor,

	/// If this context is currently in a loop, this is the location of the where the loop started.
	loop_location: Option<Location>,

	msg_ctx: MessageCtx,
	/// Used to stop workflow execution by the worker.
	stop: watch::Receiver<()>,
}

impl WorkflowCtx {
	#[tracing::instrument(skip_all, fields(workflow_id=%data.workflow_id, workflow_name=%data.workflow_name, ray_id=%data.ray_id))]
	pub async fn new(
		registry: RegistryHandle,
		db: DatabaseHandle,
		config: rivet_config::Config,
		conn: rivet_connection::Connection,
		data: PulledWorkflowData,
		stop: watch::Receiver<()>,
	) -> GlobalResult<Self> {
		let msg_ctx = MessageCtx::new(&conn, data.ray_id).await?;
		let event_history = Arc::new(data.events);

		Ok(WorkflowCtx {
			workflow_id: data.workflow_id,
			name: data.workflow_name,
			create_ts: data.create_ts,
			ts: rivet_util::timestamp::now(),
			ray_id: data.ray_id,
			version: 1,
			wake_deadline_ts: data.wake_deadline_ts,

			registry,
			db,

			config,
			conn,

			input: Arc::new(data.input),

			event_history: event_history.clone(),
			cursor: Cursor::new(event_history, Location::empty()),
			loop_location: None,

			msg_ctx,
			stop,
		})
	}

	/// Creates a workflow ctx reference with a given version.
	pub fn v(&mut self, version: usize) -> VersionedWorkflowCtx<'_> {
		VersionedWorkflowCtx::new(self, version)
	}

	/// Errors if the given version is less than the current version.
	pub(crate) fn compare_version(
		&self,
		step: impl std::fmt::Display,
		version: usize,
	) -> WorkflowResult<()> {
		if version < self.version {
			Err(WorkflowError::HistoryDiverged(format!(
				"version of {step} at {} is less than that of the current context (v{} < v{})",
				version,
				self.cursor.current_location(),
				self.version,
			)))
		} else {
			Ok(())
		}
	}

	#[tracing::instrument(name="workflow", skip_all, fields(workflow_id=%self.workflow_id, workflow_name=%self.name, ray_id=%self.ray_id))]
	pub(crate) async fn run(mut self, parent_span_ctx: SpanContext) -> WorkflowResult<()> {
		tracing::Span::current().add_link(parent_span_ctx);

		tracing::debug!("running workflow");

		// Check for stop before running
		self.check_stop()?;

		// Lookup workflow
		let workflow = self.registry.get_workflow(&self.name)?;

		// Run workflow
		let mut res = (workflow.run)(&mut self).await;

		// Validate no leftover events
		if res.is_ok() {
			if let Err(err) = self.cursor().check_clear() {
				res = Err(err);
			}
		}

		match res {
			Ok(output) => {
				tracing::debug!("workflow completed");

				let mut retries = 0;
				let mut interval = tokio::time::interval(DB_ACTION_RETRY);
				interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

				// Retry loop
				loop {
					interval.tick().await;

					// Write output
					if let Err(err) = self
						.db
						.complete_workflow(self.workflow_id, &self.name, &output)
						.await
					{
						if retries > MAX_DB_ACTION_RETRIES {
							return Err(err);
						}
						retries += 1;
					} else {
						break;
					}
				}
			}
			Err(err) => {
				let wake_immediate = err.wake_immediate();

				// Retry the workflow if its recoverable
				let wake_deadline_ts = if let Some(deadline_ts) = err.deadline_ts() {
					Some(deadline_ts)
				} else {
					None
				};

				// These signals come from a `listen` call that did not receive any signals. The workflow will
				// be retried when a signal is published
				let wake_signals = err.signals();

				// This sub workflow comes from a `wait_for_workflow` call on a workflow that did not
				// finish. This workflow will be retried when the sub workflow completes
				let wake_sub_workflow = err.sub_workflow();

				if err.is_recoverable() && !err.is_retryable() {
					tracing::debug!(?err, "workflow sleeping");
				} else {
					tracing::error!(?err, "workflow error");

					metrics::WORKFLOW_ERRORS
						.with_label_values(&[&self.name, err.to_string().as_str()])
						.inc();
				}

				let err_str = err.to_string();

				let mut retries = 0;
				let mut interval = tokio::time::interval(DB_ACTION_RETRY);
				interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

				// Retry loop
				loop {
					interval.tick().await;

					// Write output
					let res = self
						.db
						.commit_workflow(
							self.workflow_id,
							&self.name,
							wake_immediate,
							wake_deadline_ts,
							wake_signals,
							wake_sub_workflow,
							&err_str,
						)
						.await;

					if let Err(err) = res {
						if retries > MAX_DB_ACTION_RETRIES {
							return Err(err);
						}
						retries += 1;
					} else {
						break;
					}
				}
			}
		}

		Ok(())
	}

	/// Run then handle the result of an activity.
	#[tracing::instrument(skip_all, fields(activity_name=%A::NAME, %location))]
	async fn run_activity<A: Activity>(
		&mut self,
		input: &A::Input,
		event_id: &EventId,
		location: &Location,
		create_ts: i64,
	) -> WorkflowResult<A::Output> {
		tracing::debug!("running activity");

		let ctx = ActivityCtx::new(
			self.workflow_id,
			self.name.clone(),
			self.db.clone(),
			&self.config,
			&self.conn,
			self.create_ts,
			self.ray_id,
			A::NAME,
		)
		.await?;

		let start_instant = Instant::now();

		let res = tokio::time::timeout(A::TIMEOUT, A::run(&ctx, input).in_current_span())
			.await
			.map_err(|_| WorkflowError::ActivityTimeout(0));

		let dt = start_instant.elapsed().as_secs_f64();

		match res {
			Ok(Ok(output)) => {
				tracing::debug!("activity success");

				// Write output
				let input_val = serde_json::value::to_raw_value(input)
					.map_err(WorkflowError::SerializeActivityInput)?;
				let output_val = serde_json::value::to_raw_value(&output)
					.map_err(WorkflowError::SerializeActivityOutput)?;
				self.db
					.commit_workflow_activity_event(
						self.workflow_id,
						location,
						self.version,
						event_id,
						create_ts,
						&input_val,
						Ok(&output_val),
						self.loop_location(),
					)
					.await?;

				metrics::ACTIVITY_DURATION
					.with_label_values(&[&self.name, A::NAME, ""])
					.observe(dt);

				Ok(output)
			}
			Ok(Err(err)) => {
				tracing::debug!(?err, "activity error");

				let err_str = err.to_string();
				let input_val = serde_json::value::to_raw_value(input)
					.map_err(WorkflowError::SerializeActivityInput)?;

				// Write error (failed state)
				self.db
					.commit_workflow_activity_event(
						self.workflow_id,
						location,
						self.version,
						event_id,
						create_ts,
						&input_val,
						Err(&err_str),
						self.loop_location(),
					)
					.await?;

				if !err.is_workflow_recoverable() {
					metrics::ACTIVITY_ERRORS
						.with_label_values(&[&self.name, A::NAME, &err_str])
						.inc();
				}
				metrics::ACTIVITY_DURATION
					.with_label_values(&[&self.name, A::NAME, &err_str])
					.observe(dt);

				Err(WorkflowError::ActivityFailure(err, 0))
			}
			Err(err) => {
				tracing::debug!("activity timeout");

				let err_str = err.to_string();
				let input_val = serde_json::value::to_raw_value(input)
					.map_err(WorkflowError::SerializeActivityInput)?;

				self.db
					.commit_workflow_activity_event(
						self.workflow_id,
						location,
						self.version,
						event_id,
						create_ts,
						&input_val,
						Err(&err_str),
						self.loop_location(),
					)
					.await?;

				metrics::ACTIVITY_ERRORS
					.with_label_values(&[&self.name, A::NAME, &err_str])
					.inc();
				metrics::ACTIVITY_DURATION
					.with_label_values(&[&self.name, A::NAME, &err_str])
					.observe(dt);

				Err(err)
			}
		}
	}

	/// Creates a new workflow run with one more depth in the location.
	/// - **Not to be used directly by workflow users. For implementation uses only.**
	/// - **Remember to validate history after this branch is used.**
	#[tracing::instrument(skip_all)]
	pub(crate) async fn branch(&mut self) -> WorkflowResult<Self> {
		self.custom_branch(self.input.clone(), self.version).await
	}

	#[tracing::instrument(skip_all, fields(version))]
	pub(crate) async fn custom_branch(
		&mut self,
		input: Arc<Box<serde_json::value::RawValue>>,
		version: usize,
	) -> WorkflowResult<Self> {
		let history_res = self.cursor.compare_branch(version)?;
		let location = self.cursor.current_location_for(&history_res);

		// Validate history is consistent
		if !matches!(history_res, HistoryResult::Event(_)) {
			self.db
				.commit_workflow_branch_event(
					self.workflow_id,
					&location,
					self.version,
					self.loop_location.as_ref(),
				)
				.await?;
		}

		Ok(self.branch_inner(input, version, location))
	}

	pub(crate) fn branch_inner(
		&mut self,
		input: Arc<Box<serde_json::value::RawValue>>,
		version: usize,
		location: Location,
	) -> WorkflowCtx {
		WorkflowCtx {
			workflow_id: self.workflow_id,
			name: self.name.clone(),
			create_ts: self.create_ts,
			ts: self.ts,
			ray_id: self.ray_id,
			version,
			wake_deadline_ts: self.wake_deadline_ts,

			registry: self.registry.clone(),
			db: self.db.clone(),

			config: self.config.clone(),
			conn: self.conn.clone(),

			input,

			event_history: self.event_history.clone(),
			cursor: Cursor::new(self.event_history.clone(), location),
			loop_location: self.loop_location.clone(),

			msg_ctx: self.msg_ctx.clone(),
			stop: self.stop.clone(),
		}
	}

	/// Like `branch` but it does not add another layer of depth.
	pub fn step(&mut self) -> Self {
		let branch = self.clone();

		self.cursor.inc();

		branch
	}

	pub(crate) fn check_stop(&self) -> WorkflowResult<()> {
		if self.stop.has_changed().unwrap_or(true) {
			Err(WorkflowError::WorkflowStopped)
		} else {
			Ok(())
		}
	}

	pub(crate) async fn wait_stop(&self) -> WorkflowResult<()> {
		// We have to clone here because this function can't have a mutable reference to self. The state of
		// the stop channel doesn't matter because it only ever receives one message
		let _ = self.stop.clone().changed().await;
		Err(WorkflowError::WorkflowStopped)
	}
}

impl WorkflowCtx {
	/// Creates a sub workflow builder.
	pub fn workflow<I>(
		&mut self,
		input: impl WorkflowRepr<I>,
	) -> builder::sub_workflow::SubWorkflowBuilder<impl WorkflowRepr<I>, I>
	where
		I: WorkflowInput,
		<I as WorkflowInput>::Workflow: Workflow<Input = I>,
	{
		builder::sub_workflow::SubWorkflowBuilder::new(self, self.version, input)
	}

	/// Run activity. Will replay on failure.
	#[tracing::instrument(skip_all, fields(activity_name=%I::Activity::NAME))]
	pub async fn activity<I>(
		&mut self,
		input: I,
	) -> GlobalResult<<<I as ActivityInput>::Activity as Activity>::Output>
	where
		I: ActivityInput,
		<I as ActivityInput>::Activity: Activity<Input = I>,
	{
		self.check_stop().map_err(GlobalError::raw)?;

		let event_id = EventId::new(I::Activity::NAME, &input);

		let history_res = self
			.cursor
			.compare_activity(self.version, &event_id)
			.map_err(GlobalError::raw)?;
		let location = self.cursor.current_location_for(&history_res);

		// Activity was ran before
		let output = if let HistoryResult::Event(activity) = history_res {
			tracing::debug!("replaying activity");

			// Activity succeeded
			if let Some(output) = activity.parse_output().map_err(GlobalError::raw)? {
				output
			}
			// Activity failed, retry
			else {
				let error_count = activity.error_count;

				// Backoff
				if let Some(wake_deadline_ts) = self.wake_deadline_ts {
					tracing::debug!("sleeping for activity backoff");

					let duration = (u64::try_from(wake_deadline_ts)?)
						.saturating_sub(u64::try_from(rivet_util::timestamp::now())?);
					tokio::time::sleep(Duration::from_millis(duration))
						.instrument(tracing::info_span!("backoff_sleep"))
						.await;
				}

				match self
					.run_activity::<I::Activity>(&input, &event_id, &location, activity.create_ts)
					.await
				{
					Err(err) => {
						// Convert error in the case of max retries exceeded. This will only act on retryable
						// errors
						let err = match err {
							WorkflowError::ActivityFailure(err, _) => {
								if error_count + 1 >= I::Activity::MAX_RETRIES {
									WorkflowError::ActivityMaxFailuresReached(err)
								} else {
									// Add error count to the error for backoff calculation
									WorkflowError::ActivityFailure(err, error_count)
								}
							}
							WorkflowError::ActivityTimeout(_) => {
								if error_count + 1 >= I::Activity::MAX_RETRIES {
									WorkflowError::ActivityMaxFailuresReached(GlobalError::raw(err))
								} else {
									// Add error count to the error for backoff calculation
									WorkflowError::ActivityTimeout(error_count)
								}
							}
							WorkflowError::OperationTimeout(_) => {
								if error_count + 1 >= I::Activity::MAX_RETRIES {
									WorkflowError::ActivityMaxFailuresReached(GlobalError::raw(err))
								} else {
									// Add error count to the error for backoff calculation
									WorkflowError::OperationTimeout(error_count)
								}
							}
							_ => err,
						};

						return Err(GlobalError::raw(err));
					}
					x => x.map_err(GlobalError::raw)?,
				}
			}
		}
		// This is a new activity
		else {
			self.run_activity::<I::Activity>(
				&input,
				&event_id,
				&location,
				rivet_util::timestamp::now(),
			)
			.await
			.map_err(GlobalError::raw)?
		};

		// Move to next event
		self.cursor.update(&location);

		Ok(output)
	}

	/// Joins multiple executable actions (activities, closures) and awaits them simultaneously. This does not
	/// short circuit in the event of an error to make sure activity side effects are recorded.
	#[tracing::instrument(skip_all)]
	pub async fn join<T: Executable>(&mut self, exec: T) -> GlobalResult<T::Output> {
		self.check_stop().map_err(GlobalError::raw)?;

		exec.execute(self).await
	}

	/// Tests if the given error is unrecoverable. If it is, allows the user to run recovery code safely.
	/// Should always be used when trying to handle activity errors manually.
	#[tracing::instrument(skip_all)]
	pub fn catch_unrecoverable<T>(
		&mut self,
		res: GlobalResult<T>,
	) -> GlobalResult<GlobalResult<T>> {
		match res {
			Err(GlobalError::Raw(inner_err)) => {
				match inner_err.downcast::<WorkflowError>() {
					Ok(inner_err) => {
						// Despite "history diverged" errors being unrecoverable, they should not have be returned
						// by this function because the state of the history is already messed up and no new
						// workflow items should be run.
						if !inner_err.is_recoverable()
							&& !matches!(*inner_err, WorkflowError::HistoryDiverged(_))
						{
							self.cursor.inc();

							Ok(Err(GlobalError::Raw(inner_err)))
						} else {
							Err(GlobalError::Raw(inner_err))
						}
					}
					Err(err) => Err(GlobalError::Raw(err)),
				}
			}
			Err(err) => Err(err),
			Ok(x) => Ok(Ok(x)),
		}
	}

	/// Creates a signal builder.
	pub fn signal<T: Signal + Serialize>(&mut self, body: T) -> builder::signal::SignalBuilder<T> {
		builder::signal::SignalBuilder::new(self, self.version, body)
	}

	/// Listens for a signal for a short time before setting the workflow to sleep. Once the signal is
	/// received, the workflow will be woken up and continue.
	#[tracing::instrument(skip_all, fields(t=std::any::type_name::<T>()))]
	pub async fn listen<T: Listen>(&mut self) -> GlobalResult<T> {
		self.check_stop().map_err(GlobalError::raw)?;

		let history_res = self
			.cursor
			.compare_signal(self.version)
			.map_err(GlobalError::raw)?;
		let location = self.cursor.current_location_for(&history_res);

		// Signal received before
		let signal = if let HistoryResult::Event(signal) = history_res {
			tracing::debug!(
				signal_name=%signal.name,
				"replaying signal"
			);

			T::parse(&signal.name, &signal.body).map_err(GlobalError::raw)?
		}
		// Listen for new signal
		else {
			tracing::debug!("listening for signal");

			let mut wake_sub = self.db.wake_sub().await?;
			let mut retries = self.db.max_signal_poll_retries();
			let mut interval = tokio::time::interval(self.db.signal_poll_interval());
			interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

			// Skip first tick, we wait after the db call instead of before
			interval.tick().await;

			let mut ctx = ListenCtx::new(self, &location);

			loop {
				ctx.reset(retries == 0);

				match T::listen(&mut ctx).in_current_span().await {
					Ok(res) => break res,
					Err(err) if matches!(err, WorkflowError::NoSignalFound(_)) => {
						if retries == 0 {
							return Err(GlobalError::raw(err));
						}
						retries -= 1;
					}
					Err(err) => return Err(GlobalError::raw(err)),
				}

				// Poll and wait for a wake at the same time
				tokio::select! {
					_ = wake_sub.next() => {},
					_ = interval.tick() => {},
					res = self.wait_stop() => res.map_err(GlobalError::raw)?,
				}
			}
		};

		// Move to next event
		self.cursor.update(&location);

		Ok(signal)
	}

	/// Execute a custom listener.
	#[tracing::instrument(skip_all, fields(t=std::any::type_name::<T>()))]
	pub async fn custom_listener<T: CustomListener>(
		&mut self,
		listener: &T,
	) -> GlobalResult<<T as CustomListener>::Output> {
		self.check_stop().map_err(GlobalError::raw)?;

		let history_res = self
			.cursor
			.compare_signal(self.version)
			.map_err(GlobalError::raw)?;
		let location = self.cursor.current_location_for(&history_res);

		// Signal received before
		let signal = if let HistoryResult::Event(signal) = history_res {
			tracing::debug!(
				signal_name=%signal.name,
				"replaying signal",
			);

			T::parse(&signal.name, &signal.body).map_err(GlobalError::raw)?
		}
		// Listen for new signal
		else {
			tracing::debug!("listening for signal");

			let mut wake_sub = self.db.wake_sub().await?;
			let mut retries = self.db.max_signal_poll_retries();
			let mut interval = tokio::time::interval(self.db.signal_poll_interval());
			interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

			// Skip first tick, we wait after the db call instead of before
			interval.tick().await;

			let mut ctx = ListenCtx::new(self, &location);

			loop {
				ctx.reset(retries == 0);

				match listener.listen(&mut ctx).in_current_span().await {
					Ok(res) => break res,
					Err(err) if matches!(err, WorkflowError::NoSignalFound(_)) => {
						if retries == 0 {
							return Err(GlobalError::raw(err));
						}
						retries -= 1;
					}
					Err(err) => return Err(GlobalError::raw(err)),
				}

				// Poll and wait for a wake at the same time
				tokio::select! {
					_ = wake_sub.next() => {},
					_ = interval.tick() => {},
					res = self.wait_stop() => res.map_err(GlobalError::raw)?,
				}
			}
		};

		// Move to next event
		self.cursor.update(&location);

		Ok(signal)
	}

	/// Creates a message builder.
	pub fn msg<M: Message>(&mut self, body: M) -> builder::message::MessageBuilder<M> {
		builder::message::MessageBuilder::new(self, self.version, body)
	}

	/// Runs workflow steps in a loop. If you need side causes, use `WorkflowCtx::loope`.
	#[tracing::instrument(skip_all)]
	pub async fn repeat<F, T>(&mut self, mut cb: F) -> GlobalResult<T>
	where
		F: for<'a> FnMut(&'a mut WorkflowCtx) -> AsyncResult<'a, Loop<T>>,
		T: Serialize + DeserializeOwned,
	{
		self.loop_inner((), |ctx, _| cb(ctx))
			.in_current_span()
			.await
	}

	/// Runs workflow steps in a loop with state.
	#[tracing::instrument(skip_all)]
	pub async fn loope<S, F, T>(&mut self, state: S, cb: F) -> GlobalResult<T>
	where
		S: Serialize + DeserializeOwned,
		F: for<'a> FnMut(&'a mut WorkflowCtx, &'a mut S) -> AsyncResult<'a, Loop<T>>,
		T: Serialize + DeserializeOwned,
	{
		self.loop_inner(state, cb).in_current_span().await
	}

	async fn loop_inner<S, F, T>(&mut self, state: S, mut cb: F) -> GlobalResult<T>
	where
		S: Serialize + DeserializeOwned,
		F: for<'a> FnMut(&'a mut WorkflowCtx, &'a mut S) -> AsyncResult<'a, Loop<T>>,
		T: Serialize + DeserializeOwned,
	{
		self.check_stop().map_err(GlobalError::raw)?;

		let history_res = self
			.cursor
			.compare_loop(self.version)
			.map_err(GlobalError::raw)?;
		let loop_location = self.cursor.current_location_for(&history_res);

		// Loop existed before
		let (mut iteration, mut state, output) =
			if let HistoryResult::Event(loop_event) = history_res {
				let state = loop_event.parse_state().map_err(GlobalError::raw)?;
				let output = loop_event.parse_output().map_err(GlobalError::raw)?;

				(loop_event.iteration, state, output)
			} else {
				let state_val = serde_json::value::to_raw_value(&state)
					.map_err(WorkflowError::SerializeLoopOutput)
					.map_err(GlobalError::raw)?;

				// Insert event before loop is run so the history is consistent
				self.db
					.upsert_workflow_loop_event(
						self.workflow_id,
						&self.name,
						&loop_location,
						self.version,
						0,
						&state_val,
						None,
						self.loop_location(),
					)
					.await?;

				(0, state, None)
			};

		// Create a branch but no branch event (loop event takes its place)
		let mut loop_branch =
			self.branch_inner(self.input.clone(), self.version, loop_location.clone());

		// Loop complete
		let output = if let Some(output) = output {
			tracing::debug!("replaying loop output");

			output
		}
		// Run loop
		else {
			tracing::debug!("running loop");

			loop {
				self.check_stop().map_err(GlobalError::raw)?;

				let start_instant = Instant::now();

				// Create a new branch for each iteration of the loop at location {...loop location, iteration idx}
				let mut iteration_branch = loop_branch.branch_inner(
					self.input.clone(),
					self.version,
					loop_branch
						.cursor
						.root()
						.join(Coordinate::simple(iteration + 1)),
				);

				// Set branch loop location to the current loop
				iteration_branch.loop_location = Some(loop_location.clone());

				let i = iteration;

				// Async block for instrumentation purposes
				let (dt2, res) = async {
					// Insert event if iteration is not a replay
					if !loop_branch
						.cursor
						.compare_loop_branch(iteration)
						.map_err(GlobalError::raw)?
					{
						self.db
							.commit_workflow_branch_event(
								self.workflow_id,
								iteration_branch.cursor.root(),
								self.version,
								Some(&loop_location),
							)
							.await?;
					}

					let start_instant2 = Instant::now();

					// Run loop
					match cb(&mut iteration_branch, &mut state).await? {
						Loop::Continue => {
							let dt2 = start_instant2.elapsed().as_secs_f64();
							iteration += 1;

							let state_val = serde_json::value::to_raw_value(&state)
								.map_err(WorkflowError::SerializeLoopOutput)
								.map_err(GlobalError::raw)?;

							self.db
								.upsert_workflow_loop_event(
									self.workflow_id,
									&self.name,
									&loop_location,
									self.version,
									iteration,
									&state_val,
									None,
									self.loop_location(),
								)
								.await?;

							GlobalResult::Ok((dt2, None))
						}
						Loop::Break(res) => {
							let dt2 = start_instant2.elapsed().as_secs_f64();
							iteration += 1;

							let state_val = serde_json::value::to_raw_value(&state)
								.map_err(WorkflowError::SerializeLoopOutput)
								.map_err(GlobalError::raw)?;
							let output_val = serde_json::value::to_raw_value(&res)
								.map_err(WorkflowError::SerializeLoopOutput)
								.map_err(GlobalError::raw)?;

							self.db
								.upsert_workflow_loop_event(
									self.workflow_id,
									&self.name,
									&loop_location,
									self.version,
									iteration,
									&state_val,
									Some(&output_val),
									self.loop_location(),
								)
								.await?;

							Ok((dt2, Some(res)))
						}
					}
				}
				.instrument(tracing::info_span!("iteration", iteration=%i))
				.await?;

				// Validate no leftover events
				iteration_branch
					.cursor
					.check_clear()
					.map_err(GlobalError::raw)?;

				let dt = start_instant.elapsed().as_secs_f64();
				metrics::LOOP_ITERATION_DURATION
					.with_label_values(&[&self.name])
					.observe(dt - dt2);

				if let Some(res) = res {
					break res;
				}
			}
		};

		// Move to next event
		self.cursor.update(&loop_location);

		Ok(output)
	}

	#[tracing::instrument(skip_all)]
	pub async fn sleep(&mut self, duration: impl DurationToMillis) -> GlobalResult<()> {
		let ts = rivet_util::timestamp::now() as u64 + duration.to_millis()?;

		self.sleep_until(ts as i64).await
	}

	#[tracing::instrument(skip_all, fields(duration))]
	pub async fn sleep_until(&mut self, time: impl TsToMillis) -> GlobalResult<()> {
		self.check_stop().map_err(GlobalError::raw)?;

		let history_res = self
			.cursor
			.compare_sleep(self.version)
			.map_err(GlobalError::raw)?;
		let location = self.cursor.current_location_for(&history_res);

		// Slept before
		let (deadline_ts, replay) = if let HistoryResult::Event(sleep) = history_res {
			tracing::debug!("replaying sleep");

			(sleep.deadline_ts, true)
		}
		// Sleep
		else {
			let deadline_ts = time.to_millis()?;

			self.db
				.commit_workflow_sleep_event(
					self.workflow_id,
					&location,
					self.version,
					deadline_ts,
					self.loop_location(),
				)
				.await?;

			(deadline_ts, false)
		};

		let duration = deadline_ts.saturating_sub(rivet_util::timestamp::now());
		tracing::Span::current().record("duration", &duration);

		// No-op
		if duration <= 0 {
			if !replay && duration < -50 {
				tracing::warn!(%duration, "tried to sleep for a negative duration");
			}
		}
		// Sleep in memory if duration is shorter than the worker tick
		else if duration < self.db.worker_poll_interval().as_millis() as i64 + 1 {
			tracing::debug!(%deadline_ts, "sleeping in memory");

			tokio::select! {
				_ = tokio::time::sleep(Duration::from_millis(duration.try_into()?)) => {},
				res = self.wait_stop() => res?,
			}
		}
		// Workflow sleep
		else {
			tracing::debug!(%deadline_ts, "sleeping");

			return Err(GlobalError::raw(WorkflowError::Sleep(deadline_ts)));
		}

		// Move to next event
		self.cursor.update(&location);

		Ok(())
	}

	/// Listens for a signal with a timeout. Returns `None` if the timeout is reached.
	///
	/// Internally this is a sleep event and a signal event.
	#[tracing::instrument(skip_all, fields(t=std::any::type_name::<T>()))]
	pub async fn listen_with_timeout<T: Listen>(
		&mut self,
		duration: impl DurationToMillis,
	) -> GlobalResult<Option<T>> {
		let time = (rivet_util::timestamp::now() as u64 + duration.to_millis()?) as i64;

		self.listen_until(time).await
	}

	// TODO: Potential bad transaction: if the signal gets pulled and saved in history but an error occurs
	// before the sleep event state is set to "interrupted", the next time this workflow is run it will error
	// because it tries to pull a signal again
	/// Listens for a signal until the given timestamp. Returns `None` if the timestamp is reached.
	///
	/// Internally this is a sleep event and a signal event.
	#[tracing::instrument(skip_all, fields(t=std::any::type_name::<T>(), duration))]
	pub async fn listen_until<T: Listen>(
		&mut self,
		time: impl TsToMillis,
	) -> GlobalResult<Option<T>> {
		self.check_stop().map_err(GlobalError::raw)?;

		let history_res = self
			.cursor
			.compare_sleep(self.version)
			.map_err(GlobalError::raw)?;
		let history_res2 = history_res.equivalent();
		let sleep_location = self.cursor.current_location_for(&history_res);

		// Slept before
		let (deadline_ts, state) = if let HistoryResult::Event(sleep) = history_res {
			tracing::debug!("replaying sleep");

			(sleep.deadline_ts, sleep.state)
		}
		// Sleep
		else {
			let deadline_ts = TsToMillis::to_millis(time)?;

			self.db
				.commit_workflow_sleep_event(
					self.workflow_id,
					&sleep_location,
					self.version,
					deadline_ts,
					self.loop_location(),
				)
				.await?;

			(deadline_ts, SleepState::Normal)
		};

		// Move to next event
		self.cursor.update(&sleep_location);

		// Signal received before
		if matches!(state, SleepState::Interrupted) {
			let history_res = self
				.cursor
				.compare_signal(self.version)
				.map_err(GlobalError::raw)?;
			let signal_location = self.cursor.current_location_for(&history_res);

			if let HistoryResult::Event(signal) = history_res {
				tracing::debug!(
					signal_name=%signal.name,
					"replaying signal",
				);

				let signal = T::parse(&signal.name, &signal.body).map_err(GlobalError::raw)?;

				// Move to next event
				self.cursor.update(&signal_location);

				// Short circuit
				return Ok(Some(signal));
			} else {
				return Err(GlobalError::raw(WorkflowError::HistoryDiverged(format!(
					"expected signal at {}, found nothing",
					signal_location,
				))));
			}
		}

		// Location of the signal event (comes after the sleep event)
		let signal_location = self.cursor.current_location_for(&history_res2);
		let duration = deadline_ts.saturating_sub(rivet_util::timestamp::now());
		tracing::Span::current().record("duration", &duration);

		// Duration is now 0, timeout is over
		let signal = if duration <= 0 {
			// After timeout is over, check once for signal
			if matches!(state, SleepState::Normal) {
				let mut ctx = ListenCtx::new(self, &signal_location);

				match T::listen(&mut ctx).in_current_span().await {
					Ok(x) => Some(x),
					Err(WorkflowError::NoSignalFound(_)) => None,
					Err(err) => return Err(GlobalError::raw(err)),
				}
			} else {
				None
			}
		}
		// Sleep in memory if duration is shorter than the worker tick
		else if duration < self.db.worker_poll_interval().as_millis() as i64 + 1 {
			tracing::debug!(%deadline_ts, "sleeping in memory");

			let res = tokio::time::timeout(
				Duration::from_millis(duration.try_into()?),
				(async {
					tracing::debug!("listening for signal with timeout");

					let mut wake_sub = self.db.wake_sub().await?;
					let mut interval = tokio::time::interval(self.db.signal_poll_interval());
					interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

					// Skip first tick, we wait after the db call instead of before
					interval.tick().await;

					let mut ctx = ListenCtx::new(self, &signal_location);

					loop {
						ctx.reset(false);

						match T::listen(&mut ctx).in_current_span().await {
							// Retry
							Err(WorkflowError::NoSignalFound(_)) => {}
							x => return x,
						}

						// Poll and wait for a wake at the same time
						tokio::select! {
							_ = wake_sub.next() => {},
							_ = interval.tick() => {},
							res = self.wait_stop() => res?,
						}
					}
				})
				.in_current_span(),
			)
			.await;

			match res {
				Ok(res) => Some(res.map_err(GlobalError::raw)?),
				Err(_) => {
					tracing::debug!("timed out listening for signal");

					None
				}
			}
		}
		// Workflow sleep for long durations
		else {
			tracing::debug!("listening for signal with timeout");

			let mut wake_sub = self.db.wake_sub().await?;
			let mut retries = self.db.max_signal_poll_retries();
			let mut interval = tokio::time::interval(self.db.signal_poll_interval());
			interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

			// Skip first tick, we wait after the db call instead of before
			interval.tick().await;

			let mut ctx = ListenCtx::new(self, &signal_location);

			loop {
				ctx.reset(retries == 0);

				match T::listen(&mut ctx).in_current_span().await {
					Ok(res) => break Some(res),
					Err(WorkflowError::NoSignalFound(signals)) => {
						if retries == 0 {
							return Err(GlobalError::raw(WorkflowError::NoSignalFoundAndSleep(
								signals,
								deadline_ts,
							)));
						}
						retries -= 1;
					}
					Err(err) => return Err(GlobalError::raw(err)),
				}

				// Poll and wait for a wake at the same time
				tokio::select! {
					_ = wake_sub.next() => {},
					_ = interval.tick() => {},
					res = self.wait_stop() => res.map_err(GlobalError::raw)?,
				}
			}
		};

		// Update sleep state
		if signal.is_some() {
			self.db
				.update_workflow_sleep_event_state(
					self.workflow_id,
					&sleep_location,
					SleepState::Interrupted,
				)
				.await?;

			// Move to next event
			self.cursor.update(&signal_location);
		} else if matches!(state, SleepState::Normal) {
			self.db
				.update_workflow_sleep_event_state(
					self.workflow_id,
					&sleep_location,
					SleepState::Uninterrupted,
				)
				.await?;
		}

		Ok(signal)
	}

	/// Represents a removed workflow step.
	#[tracing::instrument(skip_all, fields(t=std::any::type_name::<T>()))]
	pub async fn removed<T: Removed>(&mut self) -> GlobalResult<()> {
		self.check_stop().map_err(GlobalError::raw)?;

		// Existing event
		if self
			.cursor
			.compare_removed::<T>()
			.map_err(GlobalError::raw)?
		{
			tracing::debug!("skipping removed step",);
		}
		// New "removed" event
		else {
			tracing::debug!("inserting removed step");

			self.db
				.commit_workflow_removed_event(
					self.workflow_id,
					&self.cursor.current_location(),
					T::event_type(),
					T::name(),
					self.loop_location(),
				)
				.await?;
		};

		// Move to next event
		self.cursor.inc();

		Ok(())
	}

	/// Returns the version of the current event in history. If no event exists, returns `current_version` and
	/// inserts a version check event.
	#[tracing::instrument(skip_all, fields(current_version))]
	pub async fn check_version(&mut self, current_version: usize) -> GlobalResult<usize> {
		self.check_stop().map_err(GlobalError::raw)?;

		if current_version == 0 {
			return Err(GlobalError::raw(WorkflowError::InvalidVersion(
				"version for `check_version` must be greater than 0".into(),
			)));
		}

		let (is_version_check, version) = if let Some((is_version_check, step_version)) = self
			.cursor
			.compare_version_check()
			.map_err(GlobalError::raw)?
		{
			tracing::debug!("checking existing version");

			(is_version_check, step_version)
		} else {
			tracing::debug!("inserting version check");

			self.db
				.commit_workflow_version_check_event(
					self.workflow_id,
					&self.cursor.current_location(),
					current_version + self.version - 1,
					self.loop_location(),
				)
				.await?;

			(true, current_version)
		};

		if is_version_check {
			// Move to next event
			self.cursor.inc();
		}

		Ok(version + 1 - self.version)
	}
}

impl WorkflowCtx {
	pub(crate) fn input(&self) -> &Arc<Box<serde_json::value::RawValue>> {
		&self.input
	}

	pub(crate) fn loop_location(&self) -> Option<&Location> {
		self.loop_location.as_ref()
	}

	pub(crate) fn db(&self) -> &DatabaseHandle {
		&self.db
	}

	pub(crate) fn msg_ctx(&self) -> &MessageCtx {
		&self.msg_ctx
	}

	pub(crate) fn cursor(&self) -> &Cursor {
		&self.cursor
	}

	pub(crate) fn cursor_mut(&mut self) -> &mut Cursor {
		&mut self.cursor
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn workflow_id(&self) -> Uuid {
		self.workflow_id
	}

	pub fn ray_id(&self) -> Uuid {
		self.ray_id
	}

	// Not public because this only denotes the version of the context, use `check_version` instead.
	pub(crate) fn version(&self) -> usize {
		self.version
	}

	pub(crate) fn set_version(&mut self, version: usize) {
		self.version = version;
	}

	/// Timestamp at which this workflow run started.
	pub fn ts(&self) -> i64 {
		self.ts
	}

	/// Timestamp at which the workflow was created.
	pub fn create_ts(&self) -> i64 {
		self.create_ts
	}

	/// Time between when the timestamp was processed and when it was published.
	pub fn req_dt(&self) -> i64 {
		self.ts.saturating_sub(self.create_ts)
	}

	pub fn config(&self) -> &rivet_config::Config {
		&self.config
	}
}

pub enum Loop<T> {
	Continue,
	Break(T),
}
