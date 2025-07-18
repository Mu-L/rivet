use std::{
	collections::HashMap,
	time::{Duration, Instant},
};

use futures_util::StreamExt;
use global_error::GlobalResult;
use opentelemetry::trace::TraceContextExt;
use tokio::{
	signal::{
		ctrl_c,
		unix::{signal, Signal, SignalKind},
	},
	sync::watch,
	task::JoinHandle,
};
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

use crate::{
	ctx::WorkflowCtx, db::DatabaseHandle, error::WorkflowError, metrics, registry::RegistryHandle,
	utils,
};

/// How often to run gc and update ping.
const PING_INTERVAL: Duration = Duration::from_secs(20);
/// How often to publish metrics.
const METRICS_INTERVAL: Duration = Duration::from_secs(20);
/// Time to allow running workflows to shutdown after receiving a SIGINT or SIGTERM.
const SHUTDOWN_DURATION: Duration = Duration::from_secs(30);

/// Used to spawn a new thread that indefinitely polls the database for new workflows. Only pulls workflows
/// that are registered in its registry. After pulling, the workflows are ran and their state is written to
/// the database.
pub struct Worker {
	worker_instance_id: Uuid,
	registry: RegistryHandle,
	db: DatabaseHandle,
	running_workflows: HashMap<Uuid, WorkflowHandle>,
}

impl Worker {
	pub fn new(registry: RegistryHandle, db: DatabaseHandle) -> Self {
		// Get rid of metrics that don't exist in the db anymore (declarative)
		metrics::LAST_PULL_WORKFLOWS_DURATION.reset();
		metrics::LAST_PULL_WORKFLOWS_HISTORY_DURATION.reset();
		metrics::LAST_PULL_WORKFLOWS_FULL_DURATION.reset();

		Worker {
			worker_instance_id: Uuid::new_v4(),
			registry,
			db,
			running_workflows: HashMap::new(),
		}
	}

	/// Polls the database periodically or wakes immediately when `Database::wake` finishes
	#[tracing::instrument(skip_all, fields(worker_instance_id=%self.worker_instance_id))]
	pub async fn start(
		mut self,
		config: rivet_config::Config,
		pools: rivet_pools::Pools,
	) -> GlobalResult<()> {
		tracing::debug!(
			registered_workflows = ?self.registry.size(),
			"started worker instance",
		);

		let shared_client = chirp_client::SharedClient::from_env(pools.clone())?;
		let cache = rivet_cache::CacheInner::from_env(&config, pools.clone())?;

		let mut wake_sub = { self.db.wake_sub().await? };

		let mut tick_interval = tokio::time::interval(self.db.worker_poll_interval());
		tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

		let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM hook failed");

		let mut gc_handle = self.gc();
		let mut metrics_handle = self.publish_metrics();

		loop {
			tokio::select! {
				_ = tick_interval.tick() => {},
				res = &mut gc_handle => {
					tracing::error!(?res, "metrics task unexpectedly stopped");
					break;
				}
				res = &mut metrics_handle => {
					tracing::error!(?res, "metrics task unexpectedly stopped");
					break;
				},
				res = wake_sub.next() => {
					if res.is_none() {
						return Err(WorkflowError::SubscriptionUnsubscribed.into());
					}

					tick_interval.reset();
				},
				_ = ctrl_c() => break,
				_ = sigterm.recv() => break,
			}

			self.tick(&shared_client, &config, &pools, &cache).await?;
		}

		self.shutdown(sigterm).await;

		Ok(())
	}

	#[tracing::instrument(skip_all)]
	async fn shutdown(mut self, mut sigterm: Signal) {
		// For an accurate count of how many remaining workflows there are, retain before checking length
		self.running_workflows
			.retain(|_, wf| !wf.handle.is_finished());

		// Shutdown sequence
		tracing::info!(
			duration=?SHUTDOWN_DURATION,
			remaining_workflows=?self.running_workflows.len(),
			"starting worker shutdown"
		);

		let shutdown_start = Instant::now();

		for (workflow_id, wf) in &self.running_workflows {
			if wf.stop.send(()).is_err() {
				tracing::warn!(?workflow_id, "stop channel closed");
			}
		}

		let mut second_sigterm = false;
		loop {
			self.running_workflows
				.retain(|_, wf| !wf.handle.is_finished());

			// Shutdown complete
			if self.running_workflows.is_empty() {
				break;
			}

			if shutdown_start.elapsed() > SHUTDOWN_DURATION {
				tracing::debug!("shutdown timed out");
				break;
			}

			tokio::select! {
				_ = ctrl_c() => {
					if second_sigterm {
						tracing::warn!("received third SIGTERM, aborting shutdown");
						break;
					}

					tracing::warn!("received second SIGTERM");
					second_sigterm = true;

					continue;
				}
				_ = sigterm.recv() => {
					if second_sigterm {
						tracing::warn!("received third SIGTERM, aborting shutdown");
						break;
					}

					tracing::warn!("received second SIGTERM");
					second_sigterm = true;

					continue;
				}
				_ = tokio::time::sleep(Duration::from_secs(2)) => {}
			}
		}

		if self.running_workflows.is_empty() {
			tracing::info!("all workflows evicted");
		} else {
			tracing::warn!(remaining_workflows=?self.running_workflows.len(), "not all workflows evicted");
		}

		tracing::info!("shutdown complete");

		rivet_runtime::shutdown().await;
	}

	/// Query the database for new workflows and run them.
	#[tracing::instrument(skip_all)]
	async fn tick(
		&mut self,
		shared_client: &chirp_client::SharedClientHandle,
		config: &rivet_config::Config,
		pools: &rivet_pools::Pools,
		cache: &rivet_cache::Cache,
	) -> GlobalResult<()> {
		// Create filter from registered workflow names
		let filter = self
			.registry
			.workflows
			.keys()
			.map(|k| k.as_str())
			.collect::<Vec<_>>();

		// Query awake workflows
		let workflows = self
			.db
			.pull_workflows(self.worker_instance_id, &filter)
			.await?;

		// Remove join handles for completed workflows. This must happen after we pull workflows to ensure an
		// accurate state of the current workflows
		self.running_workflows
			.retain(|_, wf| !wf.handle.is_finished());

		for workflow in workflows {
			let workflow_id = workflow.workflow_id;

			if self.running_workflows.contains_key(&workflow_id) {
				tracing::error!(?workflow_id, "workflow already running");
				continue;
			}

			let (stop_tx, stop_rx) = watch::channel(());

			let conn = utils::new_conn(
				shared_client,
				pools,
				cache,
				workflow.ray_id,
				workflow.workflow_id,
				&workflow.workflow_name,
			);
			let ctx = WorkflowCtx::new(
				self.registry.clone(),
				self.db.clone(),
				config.clone(),
				conn,
				workflow,
				stop_rx,
			)
			.await?;

			let current_span_ctx = tracing::Span::current()
				.context()
				.span()
				.span_context()
				.clone();

			let handle = tokio::task::spawn(
				// NOTE: No .in_current_span() because we want this to be a separate trace
				async move {
					if let Err(err) = ctx.run(current_span_ctx).await {
						tracing::error!(?err, "unhandled workflow error");
					}
				},
			);

			self.running_workflows.insert(
				workflow_id,
				WorkflowHandle {
					stop: stop_tx,
					handle,
				},
			);
		}

		Ok(())
	}

	fn gc(&self) -> JoinHandle<()> {
		let db = self.db.clone();
		let worker_instance_id = self.worker_instance_id;

		tokio::task::spawn(
			async move {
				let mut ping_interval = tokio::time::interval(PING_INTERVAL);
				ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

				loop {
					ping_interval.tick().await;

					if let Err(err) = db.update_worker_ping(worker_instance_id).await {
						tracing::error!(?err, "unhandled update ping error");
					}

					if let Err(err) = db.clear_expired_leases(worker_instance_id).await {
						tracing::error!(?err, "unhandled gc error");
					}
				}
			}
			.instrument(tracing::info_span!("worker_gc_task")),
		)
	}

	fn publish_metrics(&self) -> JoinHandle<()> {
		let db = self.db.clone();
		let worker_instance_id = self.worker_instance_id;

		tokio::task::spawn(
			async move {
				let mut metrics_interval = tokio::time::interval(METRICS_INTERVAL);
				metrics_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

				loop {
					metrics_interval.tick().await;

					if let Err(err) = db.publish_metrics(worker_instance_id).await {
						tracing::error!(?err, "unhandled metrics error");
					}
				}
			}
			.instrument(tracing::info_span!("worker_metrics_task")),
		)
	}
}

struct WorkflowHandle {
	stop: watch::Sender<()>,
	handle: JoinHandle<()>,
}
