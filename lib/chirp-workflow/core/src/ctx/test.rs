use global_error::{GlobalError, GlobalResult};
use rivet_pools::prelude::*;
use serde::Serialize;
use tokio::time::Duration;
use uuid::Uuid;

use crate::{
	util, DatabaseHandle, DatabasePostgres, Operation, OperationCtx, OperationInput, Signal,
	Workflow, WorkflowError, WorkflowInput,
};

pub struct TestCtx {
	name: String,
	ray_id: Uuid,
	ts: i64,

	db: DatabaseHandle,

	conn: rivet_connection::Connection,

	// Backwards compatibility
	op_ctx: rivet_operation::OperationContext<()>,
}

impl TestCtx {
	pub async fn from_env(test_name: &str) -> TestCtx {
		let service_name = format!(
			"{}-test--{}",
			std::env::var("CHIRP_SERVICE_NAME").unwrap(),
			test_name
		);

		let ray_id = Uuid::new_v4();
		let pools = rivet_pools::from_env(service_name.clone())
			.await
			.expect("failed to create pools");
		let shared_client = chirp_client::SharedClient::from_env(pools.clone())
			.expect("failed to create chirp client");
		let cache =
			rivet_cache::CacheInner::from_env(pools.clone()).expect("failed to create cache");
		let conn = util::new_conn(
			&shared_client,
			&pools,
			&cache,
			ray_id,
			Uuid::new_v4(),
			&service_name,
		);
		let ts = rivet_util::timestamp::now();
		let req_id = Uuid::new_v4();
		let op_ctx = rivet_operation::OperationContext::new(
			service_name.to_string(),
			std::time::Duration::from_secs(60),
			conn.clone(),
			req_id,
			ray_id,
			ts,
			ts,
			(),
		);

		let db = DatabasePostgres::from_pool(pools.crdb().unwrap());

		TestCtx {
			name: service_name,
			ray_id,
			ts: rivet_util::timestamp::now(),
			db,
			conn,
			op_ctx,
		}
	}
}

impl TestCtx {
	pub async fn dispatch_workflow<I>(&self, input: I) -> GlobalResult<Uuid>
	where
		I: WorkflowInput,
		<I as WorkflowInput>::Workflow: Workflow<Input = I>,
	{
		let name = I::Workflow::NAME;

		tracing::debug!(%name, ?input, "dispatching workflow");

		let id = Uuid::new_v4();

		// Serialize input
		let input_val = serde_json::to_value(input)
			.map_err(WorkflowError::SerializeWorkflowOutput)
			.map_err(GlobalError::raw)?;

		self.db
			.dispatch_workflow(self.ray_id, id, &name, input_val)
			.await
			.map_err(GlobalError::raw)?;

		tracing::info!(%name, ?id, "workflow dispatched");

		Ok(id)
	}

	pub async fn wait_for_workflow<W: Workflow>(
		&self,
		workflow_id: Uuid,
	) -> GlobalResult<W::Output> {
		tracing::info!(name=W::NAME, id=?workflow_id, "waiting for workflow");

		let period = Duration::from_millis(50);
		let mut interval = tokio::time::interval(period);
		loop {
			interval.tick().await;

			// Check if state finished
			let workflow = self
				.db
				.get_workflow(workflow_id)
				.await
				.map_err(GlobalError::raw)?
				.ok_or(WorkflowError::WorkflowNotFound)
				.map_err(GlobalError::raw)?;
			if let Some(output) = workflow.parse_output::<W>().map_err(GlobalError::raw)? {
				return Ok(output);
			}
		}
	}

	pub async fn workflow<I>(
		&self,
		input: I,
	) -> GlobalResult<<<I as WorkflowInput>::Workflow as Workflow>::Output>
	where
		I: WorkflowInput,
		<I as WorkflowInput>::Workflow: Workflow<Input = I>,
	{
		let workflow_id = self.dispatch_workflow(input).await?;
		let output = self.wait_for_workflow::<I::Workflow>(workflow_id).await?;
		Ok(output)
	}

	pub async fn signal<T: Signal + Serialize>(
		&self,
		workflow_id: Uuid,
		input: T,
	) -> GlobalResult<Uuid> {
		tracing::debug!(name=%T::NAME, %workflow_id, "dispatching signal");

		let signal_id = Uuid::new_v4();

		// Serialize input
		let input_val = serde_json::to_value(input)
			.map_err(WorkflowError::SerializeSignalBody)
			.map_err(GlobalError::raw)?;

		self.db
			.publish_signal(self.ray_id, workflow_id, signal_id, T::NAME, input_val)
			.await
			.map_err(GlobalError::raw)?;

		Ok(signal_id)
	}

	/// Waits for a workflow to be triggered with a superset of given input. Strictly for tests.
	pub fn observe<W: Workflow>(&self, input: serde_json::Value) -> GlobalResult<ObserveHandle> {
		// Serialize input
		let input_val = serde_json::to_value(input)
			.map_err(WorkflowError::SerializeWorkflowOutput)
			.map_err(GlobalError::raw)?;

		Ok(ObserveHandle {
			db: self.db.clone(),
			name: W::NAME,
			input: input_val,
			ts: rivet_util::timestamp::now(),
		})
	}

	pub async fn op<I>(
		&self,
		input: I,
	) -> GlobalResult<<<I as OperationInput>::Operation as Operation>::Output>
	where
		I: OperationInput,
		<I as OperationInput>::Operation: Operation<Input = I>,
	{
		let ctx = OperationCtx::new(
			self.db.clone(),
			&self.conn,
			self.ray_id,
			self.ts,
			false,
			I::Operation::NAME,
		);

		I::Operation::run(&ctx, &input)
			.await
			.map_err(WorkflowError::OperationFailure)
			.map_err(GlobalError::raw)
	}
}

impl TestCtx {
	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn req_id(&self) -> Uuid {
		self.op_ctx.req_id()
	}

	pub fn ray_id(&self) -> Uuid {
		self.ray_id
	}

	/// Timestamp at which the request started.
	pub fn ts(&self) -> i64 {
		self.ts
	}

	/// Timestamp at which the request was published.
	pub fn req_ts(&self) -> i64 {
		self.op_ctx.req_ts()
	}

	/// Time between when the timestamp was processed and when it was published.
	pub fn req_dt(&self) -> i64 {
		self.ts.saturating_sub(self.op_ctx.req_ts())
	}

	// pub fn perf(&self) -> &chirp_perf::PerfCtx {
	// 	self.conn.perf()
	// }

	pub fn trace(&self) -> &[chirp_client::TraceEntry] {
		self.conn.trace()
	}

	pub fn test(&self) -> bool {
		self.trace()
			.iter()
			.any(|x| x.run_context == chirp_client::RunContext::Test as i32)
	}

	pub fn chirp(&self) -> &chirp_client::Client {
		self.conn.chirp()
	}

	pub fn cache(&self) -> rivet_cache::RequestConfig {
		self.conn.cache()
	}

	pub fn cache_handle(&self) -> rivet_cache::Cache {
		self.conn.cache_handle()
	}

	pub async fn crdb(&self) -> Result<CrdbPool, rivet_pools::Error> {
		self.conn.crdb().await
	}

	pub async fn redis_cache(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.conn.redis_cache().await
	}

	pub async fn redis_cdn(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.conn.redis_cdn().await
	}

	pub async fn redis_job(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.conn.redis_job().await
	}

	pub async fn redis_mm(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.conn.redis_mm().await
	}

	pub async fn redis_user_presence(&self) -> Result<RedisPool, rivet_pools::Error> {
		self.conn.redis_user_presence().await
	}

	pub async fn clickhouse(&self) -> GlobalResult<ClickHousePool> {
		self.conn.clickhouse().await
	}

	// Backwards compatibility
	pub fn op_ctx(&self) -> &rivet_operation::OperationContext<()> {
		&self.op_ctx
	}
}

pub struct ObserveHandle {
	db: DatabaseHandle,
	name: &'static str,
	input: serde_json::Value,
	ts: i64,
}

impl ObserveHandle {
	pub async fn next(&mut self) -> GlobalResult<Uuid> {
		tracing::info!(name=%self.name, input=?self.input, "observing workflow");
		tracing::info!(ts=%self.ts);

		let (workflow_id, create_ts) = loop {
			if let Some((workflow_id, create_ts)) = self
				.db
				.poll_workflow(self.name, &self.input, self.ts)
				.await
				.map_err(GlobalError::raw)?
			{
				break (workflow_id, create_ts);
			}

			tokio::time::sleep(Duration::from_millis(200)).await;
		};

		tracing::info!(name=%self.name, id=?workflow_id, "workflow found");

		self.ts = create_ts + 1;

		Ok(workflow_id)
	}
}
