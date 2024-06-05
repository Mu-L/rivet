use std::{collections::HashMap, sync::Arc, time::Duration};

use indoc::indoc;
use sqlx::{pool::PoolConnection, PgPool, Postgres};
use uuid::Uuid;

use super::{
	ActivityEventRow, Database, PulledWorkflow, PulledWorkflowRow, SignalEventRow, SignalRow,
	SubWorkflowEventRow, WorkflowRow,
};
use crate::{schema::ActivityId, WorkflowError, WorkflowResult};

const NODE_ID: Uuid = Uuid::nil();

pub struct DatabasePostgres {
	pool: PgPool,
}

impl DatabasePostgres {
	pub async fn new(url: &str) -> WorkflowResult<Arc<DatabasePostgres>> {
		let pool = sqlx::postgres::PgPoolOptions::new()
			// The default connection timeout is too high
			.acquire_timeout(Duration::from_secs(15))
			// Increase lifetime to mitigate: https://github.com/launchbadge/sqlx/issues/2854
			//
			// See max lifetime https://www.cockroachlabs.com/docs/stable/connection-pooling#set-the-maximum-lifetime-of-connections
			.max_lifetime(Duration::from_secs(30 * 60))
			// Remove connections after a while in order to reduce load
			// on CRDB after bursts
			.idle_timeout(Some(Duration::from_secs(3 * 60)))
			// Open connections immediately on startup
			.min_connections(1)
			// Raise the cap, since this is effectively the amount of
			// simultaneous requests we can handle. See
			// https://www.cockroachlabs.com/docs/stable/connection-pooling.html
			.max_connections(4096)
			.connect(url)
			.await
			.map_err(WorkflowError::BuildSqlx)?;

		Ok(Arc::new(DatabasePostgres { pool }))
	}

	pub fn from_pool(pool: PgPool) -> Arc<DatabasePostgres> {
		Arc::new(DatabasePostgres { pool })
	}

	pub async fn conn(&self) -> WorkflowResult<PoolConnection<Postgres>> {
		// Attempt to use an existing connection
		if let Some(conn) = self.pool.try_acquire() {
			Ok(conn)
		} else {
			// Create a new connection
			self.pool.acquire().await.map_err(WorkflowError::Sqlx)
		}
	}
}

#[async_trait::async_trait]
impl Database for DatabasePostgres {
	async fn dispatch_workflow(
		&self,
		workflow_id: Uuid,
		workflow_name: &str,
		input: &str,
	) -> WorkflowResult<()> {
		sqlx::query(indoc!(
			"
			INSERT INTO db_workflow.workflows (workflow_id, workflow_name, input, wake_immediate)
			VALUES ($1, $2, $3, true)
			",
		))
		.bind(workflow_id)
		.bind(workflow_name)
		.bind(input)
		.execute(&mut *self.conn().await?)
		.await
		.map_err(WorkflowError::Sqlx)?;

		Ok(())
	}

	async fn get_workflow(&self, workflow_id: Uuid) -> WorkflowResult<Option<WorkflowRow>> {
		sqlx::query_as::<_, WorkflowRow>(indoc!(
			"
			SELECT workflow_id, input, output
			FROM db_workflow.workflows
			WHERE workflow_id = $1
			",
		))
		.bind(workflow_id)
		.fetch_optional(&mut *self.conn().await?)
		.await
		.map_err(WorkflowError::Sqlx)
	}

	async fn pull_workflows(&self, filter: &[&str]) -> WorkflowResult<Vec<PulledWorkflow>> {
		// TODO(RVT-3753): include limit on query to allow better workflow spread between nodes?
		// Select all workflows that haven't started or that have a wake condition
		let rows = sqlx::query_as::<_, PulledWorkflowRow>(indoc!(
			"
			UPDATE db_workflow.workflows as w
			-- Assign this node to this workflow
			SET node_id = $1
			WHERE
				-- Filter
				workflow_name = ANY($2) AND
				-- Not already complete
				output IS NULL AND
				-- No assigned node (not running)
				node_id IS NULL AND
				-- Check for wake condition
				(
					wake_immediate OR
					wake_deadline_ts IS NOT NULL OR
					(
						SELECT true
						FROM db_workflow.signals AS s
						WHERE s.signal_name = ANY(wake_signals)
						LIMIT 1
					) OR
					(
						SELECT true
						FROM db_workflow.workflows AS w2
						WHERE
							w2.workflow_id = w.wake_sub_workflow_id AND
							output IS NOT NULL
					)
				)
			RETURNING workflow_id, workflow_name, input, wake_deadline_ts
			",
		))
		.bind(NODE_ID)
		.bind(filter)
		.fetch_all(&mut *self.conn().await?)
		.await
		.map_err(WorkflowError::Sqlx)?;

		// Turn rows into hashmap
		let workflow_ids = rows.iter().map(|row| row.workflow_id).collect::<Vec<_>>();
		let mut workflows_by_id = rows
			.into_iter()
			.map(|row| {
				(
					row.workflow_id,
					PulledWorkflow {
						workflow_id: row.workflow_id,
						workflow_name: row.workflow_name,
						input: row.input,
						wake_deadline_ts: row.wake_deadline_ts,
						activity_events: Vec::new(),
						signal_events: Vec::new(),
						sub_workflow_events: Vec::new(),
					},
				)
			})
			.collect::<HashMap<_, _>>();

		// Fetch all events for all fetched workflows
		let (activity_events, signal_events, sub_workflow_events) = tokio::try_join!(
			async {
				sqlx::query_as::<_, ActivityEventRow>(indoc!(
					"
					SELECT
						workflow_id, location, activity_name, input_hash, output
					FROM db_workflow.workflow_activity_events
					WHERE workflow_id = ANY($1)
					ORDER BY workflow_id, location ASC
					",
				))
				.bind(&workflow_ids)
				.fetch_all(&mut *self.conn().await?)
				.await
				.map_err(WorkflowError::Sqlx)
			},
			async {
				sqlx::query_as::<_, SignalEventRow>(indoc!(
					"
					SELECT
						workflow_id, location, signal_name, body
					FROM db_workflow.workflow_signal_events
					WHERE workflow_id = ANY($1)
					ORDER BY workflow_id, location ASC
					",
				))
				.bind(&workflow_ids)
				.fetch_all(&mut *self.conn().await?)
				.await
				.map_err(WorkflowError::Sqlx)
			},
			async {
				sqlx::query_as::<_, SubWorkflowEventRow>(indoc!(
					"
					SELECT
						sw.workflow_id, sw.location, sw.sub_workflow_id, w.name as sub_workflow_name
					FROM db_workflow.workflow_sub_workflow_events AS sw
					JOIN db_workflow.workflows AS w
					ON sw.sub_workflow_id = w.workflow_id
					WHERE workflow_id = ANY($1)
					ORDER BY workflow_id, location ASC
					",
				))
				.bind(&workflow_ids)
				.fetch_all(&mut *self.conn().await?)
				.await
				.map_err(WorkflowError::Sqlx)
			}
		)?;

		// Insert events into hashmap
		for event in activity_events {
			workflows_by_id
				.get_mut(&event.workflow_id)
				.expect("unreachable, workflow for event not found")
				.activity_events
				.push(event);
		}
		for event in signal_events {
			workflows_by_id
				.get_mut(&event.workflow_id)
				.expect("unreachable, workflow for event not found")
				.signal_events
				.push(event);
		}
		for event in sub_workflow_events {
			workflows_by_id
				.get_mut(&event.workflow_id)
				.expect("unreachable, workflow for event not found")
				.sub_workflow_events
				.push(event);
		}

		Ok(workflows_by_id.into_values().collect())
	}

	async fn commit_workflow(&self, workflow_id: Uuid, output: &str) -> WorkflowResult<()> {
		sqlx::query(indoc!(
			"
			UPDATE db_workflow.workflows
			SET output = $2
			WHERE workflow_id = $1
			",
		))
		.bind(workflow_id)
		.bind(output)
		.execute(&mut *self.conn().await?)
		.await
		.map_err(WorkflowError::Sqlx)?;

		Ok(())
	}

	async fn fail_workflow(
		&self,
		workflow_id: Uuid,
		immediate: bool,
		deadline_ts: Option<i64>,
		wake_signals: &[&str],
		wake_sub_workflow_id: Option<Uuid>,
	) -> WorkflowResult<()> {
		// TODO: Should this compare `wake_deadline_ts` before setting it?
		sqlx::query(indoc!(
			"
			UPDATE db_workflow.workflows
			SET
				node_id = NULL,
				wake_immediate = $2,
				wake_deadline_ts = $3,
				wake_signals = $4,
				wake_sub_workflow_id = $5
			WHERE workflow_id = $1
			",
		))
		.bind(workflow_id)
		.bind(immediate)
		.bind(deadline_ts)
		.bind(wake_signals)
		.bind(wake_sub_workflow_id)
		.execute(&mut *self.conn().await?)
		.await
		.map_err(WorkflowError::Sqlx)?;

		Ok(())
	}

	async fn commit_workflow_activity_event(
		&self,
		workflow_id: Uuid,
		location: &[usize],
		activity_id: &ActivityId,
		input: &str,
		output: Option<&str>,
	) -> WorkflowResult<()> {
		sqlx::query(indoc!(
			"
			UPSERT INTO db_workflow.workflow_activity_events (
				workflow_id, location, activity_name, input_hash, input, output
			)
			VALUES ($1, $2, $3, $4, $5)
			",
		))
		.bind(workflow_id)
		.bind(location.iter().map(|x| *x as i64).collect::<Vec<_>>())
		.bind(&activity_id.name)
		.bind(activity_id.input_hash.to_le_bytes())
		.bind(input)
		.bind(output)
		.execute(&mut *self.conn().await?)
		.await
		.map_err(WorkflowError::Sqlx)?;

		Ok(())
	}

	async fn pull_latest_signal(
		&self,
		workflow_id: Uuid,
		filter: &[&str],
		location: &[usize],
	) -> WorkflowResult<Option<SignalRow>> {
		// TODO: RVT-3752
		let signal = sqlx::query_as::<_, SignalRow>(indoc!(
			"
			WITH
				latest_signal AS (
					DELETE FROM db_workflow.signals
					WHERE
						workflow_id = $1 AND
						signal_name = ANY($2)
					ORDER BY create_ts ASC
					LIMIT 1
					RETURNING workflow_id, signal_id, signal_name, body
				),
				clear_wake AS (
					UPDATE db_workflow.workflows AS w
					SET wake_signals = ARRAY[]
					FROM db_workflow.latest_signal AS s
					WHERE w.workflow_id = s.workflow_id
					RETURNING 1
				),
				insert_event AS (
					INSERT INTO db_workflow.workflow_signal_events(
						workflow_id, location, signal_id, signal_name, body
					)
					SELECT workflow_id, $3 AS location, signal_id, signal_name, body
					FROM db_workflow.latest_signal
					RETURNING 1
				)
			SELECT * FROM latest_signal
			",
		))
		.bind(workflow_id)
		.bind(filter)
		.bind(location.iter().map(|x| *x as i64).collect::<Vec<_>>())
		.fetch_optional(&mut *self.conn().await?)
		.await
		.map_err(WorkflowError::Sqlx)?;

		Ok(signal)
	}

	async fn publish_signal(
		&self,
		workflow_id: Uuid,
		signal_id: Uuid,
		signal_name: &str,
		body: &str,
	) -> WorkflowResult<()> {
		sqlx::query(indoc!(
			"
			INSERT INTO db_workflow.signals (signal_id, workflow_id, signal_name, body, create_ts)			
			VALUES ($1, $2, $3, $4, $5)
			",
		))
		.bind(signal_id)
		.bind(workflow_id)
		.bind(signal_name)
		.bind(body)
		.bind(rivet_util::timestamp::now())
		.execute(&mut *self.conn().await?)
		.await
		.map_err(WorkflowError::Sqlx)?;

		Ok(())
	}

	async fn dispatch_sub_workflow(
		&self,
		workflow_id: Uuid,
		location: &[usize],
		sub_workflow_id: Uuid,
		sub_workflow_name: &str,
		input: &str,
	) -> WorkflowResult<()> {
		sqlx::query(indoc!(
			"
			WITH
				workflow AS (
					INSERT INTO db_workflow.workflows (workflow_id, workflow_name, input, wake_immediate)
					VALUES ($5, $2, $3, true)
					RETURNING 1
			 	),
				sub_workflow AS (
					INSERT INTO db_workflow.workflow_sub_workflow_events(
						workflow_id, location, sub_workflow_id
					)
					VALUES($1, $4, $5)
					RETURNING 1
				)
			SELECT 1
			",
		))
		.bind(workflow_id)
		.bind(sub_workflow_name)
		.bind(input)
		.bind(location.iter().map(|x| *x as i64).collect::<Vec<_>>())
		.bind(sub_workflow_id)
		.execute(&mut *self.conn().await?)
		.await
		.map_err(WorkflowError::Sqlx)?;

		Ok(())
	}
}