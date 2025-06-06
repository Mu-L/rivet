use indoc::indoc;
use rivet_operation::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::{collections::HashMap, time::Duration};
use sysinfo::System;

// This information is intended for use with diagnosing errors in the wild.
//
// Events are sent to PostHog (https://posthog.com/). These events enrich our Sentry errors in
// order to better understand the unique edge cases that might cause an error.
//
// Rivet is a powerful yet complicated system and these metrics help us support the open-source
// community. If you have questions or concerns about this data, please reach out to us on Discord:
// https://rivet.gg/discord

// This API key is safe to hardcode. It will not change and is intended to be public.
const POSTHOG_API_KEY: &str = "phc_1lUNmul6sAdFzDK1VHXNrikCfD7ivQZSpf2yzrPvr4m";

pub async fn start(config: rivet_config::Config, pools: rivet_pools::Pools) -> GlobalResult<()> {
	run_from_env(config.clone(), pools.clone(), util::timestamp::now()).await
}

#[tracing::instrument(skip_all)]
pub async fn run_from_env(
	config: rivet_config::Config,
	pools: rivet_pools::Pools,
	_ts: i64,
) -> GlobalResult<()> {
	let client = chirp_client::SharedClient::from_env(pools.clone())?.wrap_new("telemetry-beacon");
	let cache = rivet_cache::CacheInner::from_env(&config, pools.clone())?;
	let ctx = OperationContext::new(
		"telemetry-beacon".into(),
		Duration::from_secs(300),
		config.clone(),
		rivet_connection::Connection::new(client, pools, cache),
		Uuid::new_v4(),
		Uuid::new_v4(),
		util::timestamp::now(),
		util::timestamp::now(),
		(),
	);

	// Get the rivet instance ID
	let instance_id = chirp_workflow::compat::op(&ctx, dynamic_config::ops::get_config::Input {})
		.await?
		.instance_id;

	// Build events
	let mut events = Vec::new();
	let distinct_id = format!("instance:{instance_id}");

	// Send beacon
	let mut event = async_posthog::Event::new("instance_beacon", &distinct_id);
	event.insert_prop("$groups", json!({ "instance": instance_id }))?;
	event.insert_prop(
		"$set",
		json!({
			"instance_id": instance_id,
			"os": os_report(),
			"source_hash": rivet_env::source_hash(),
			"config": get_config_data(&ctx)?,
			"infrastructure": get_infrastructure_data(&ctx)?,
			// "pegboard": get_pegboard_data(&ctx).await?,
		}),
	)?;
	events.push(event);

	// Add instance identification data
	let mut event = async_posthog::Event::new("$groupidentify", &distinct_id);
	event.insert_prop("$group_type", "instance")?;
	event.insert_prop("$group_key", instance_id)?;
	event.insert_prop(
		"$group_set",
		json!({
			"name": ctx.config().server()?.rivet.namespace,
		}),
	)?;
	events.push(event);

	tracing::debug!(len = ?events.len(), "built events");

	// Send events in chunks
	let client = async_posthog::client(POSTHOG_API_KEY);
	client.capture_batch(events).await?;

	tracing::debug!("all events sent");

	Ok(())
}

/// Returns information about the operating system running the instance.
///
/// This helps Rivet diagnose crash reports to easily pinpoint if issues are
/// coming from a specific operating system.
fn os_report() -> serde_json::Value {
	// Create a new System object
	let system = sysinfo::System::new_with_specifics(
		sysinfo::RefreshKind::new()
			.with_cpu(sysinfo::CpuRefreshKind::everything())
			.with_memory(sysinfo::MemoryRefreshKind::everything()),
	);

	// Gather OS information
	let os_name = std::env::consts::OS;
	let os_version = System::os_version()
		.unwrap_or_else(|| String::from("Unknown"))
		.to_string();
	let architecture = std::env::consts::ARCH;
	let hostname = System::host_name().unwrap_or_else(|| String::from("unknown"));

	// Gather memory information
	let total_memory = system.total_memory();
	let available_memory = system.available_memory();

	// Gather CPU information
	let cpu_info = system.cpus();
	let cpu_model = cpu_info
		.first()
		.map(|p| p.brand().to_string())
		.unwrap_or_else(|| String::from("Unknown CPU"));

	// Combine everything into a JSON object
	json!({
		"name": os_name,
		"version": os_version,
		"architecture": architecture,
		"hostname": hostname,
		"memory": {
			"total": total_memory,
			"available": available_memory,
		},
		"cpu": {
			"model": cpu_model,
			"cores": cpu_info.len(),
		}
	})
}

/// Returns information about what feature configs are enabled.
///
/// This helps Rivet diagnose crash reports to understand if specific features
/// are causing crashes.
fn get_config_data(ctx: &OperationContext<()>) -> GlobalResult<serde_json::Value> {
	let server_config = ctx.config().server()?;

	Ok(json!({
		"rivet": {
			"namespace": server_config.rivet.namespace,
			"auth_access_kind": format!("{:?}", server_config.rivet.auth.access_kind),
			"dns_enabled": server_config.rivet.dns.is_some(),
			"job_run_enabled": server_config.rivet.job_run.is_some(),
			"api_origin": server_config.rivet.api_public.public_origin(),
			"hub_origin": server_config.rivet.ui.public_origin(),
		},
	}))
}

/// Returns information about what infrastructure is enabled.
///
/// This helps Rivet diagnose crash reports to understand if parts of the
/// infrastructure are causing issues.
fn get_infrastructure_data(ctx: &OperationContext<()>) -> GlobalResult<serde_json::Value> {
	let server_config = ctx.config().server()?;
	Ok(json!({
		"nomad_enabled": server_config.nomad.is_some(),
		"cloudflare_enabled": server_config.cloudflare.is_some(),
		"sendgrid_enabled": server_config.sendgrid.is_some(),
		"loops_enabled": server_config.loops.is_some(),
		"ip_info_enabled": server_config.ip_info.is_some(),
		"hcaptcha_enabled": server_config.hcaptcha.is_some(),
		"turnstile_enabled": server_config.turnstile.is_some(),
		"stripe_enabled": server_config.stripe.is_some(),
		"neon_enabled": server_config.neon.is_some(),
		"linode_enabled": server_config.linode.is_some(),
		"clickhouse_enabled": server_config.clickhouse.is_some(),
		"prometheus_enabled": server_config.prometheus.is_some(),
		"s3_endpoint": server_config.s3.endpoint_external,
	}))
}

#[derive(sqlx::FromRow, Serialize)]
struct ResourceAggregateRow {
	count: i64,
	cpu_core_sum: i64,
	cpu_frequency_sum: i64,
	memory_sum: i64,
	swap_sum: i64,
}

// /// Returns information about the pegboard configuration.
// ///
// /// This is helpful for diagnosing issues with the self-hosted instances under
// /// load. e.g. if a instance is running on constraint resources (see os_report),
// /// does the instance configuration affect it?
// async fn get_pegboard_data(ctx: &OperationContext<()>) -> GlobalResult<serde_json::Value> {
// 	use pegboard::protocol::ClientFlavor;

// 	let mut clients = HashMap::new();
// 	for flavor in [ClientFlavor::Container, ClientFlavor::Isolate] {
// 		let row = sql_fetch_one!(
// 			[ctx, ResourceAggregateRow]
// 			"
// 			SELECT
// 				count(*)::int AS count,
// 				coalesce(sum((system_info->'cpu'->'physical_core_count')::int), 0)::int AS cpu_core_sum,
// 				coalesce(sum((system_info->'cpu'->'frequency')::int), 0)::int AS cpu_frequency_sum,
// 				coalesce(sum((system_info->'memory'->'total_memory')::int), 0)::int AS memory_sum,
// 				coalesce(sum((system_info->'memory'->'total_swap')::int), 0)::int AS swap_sum
// 			FROM db_pegboard.clients AS OF SYSTEM TIME '-5s'
// 			WHERE
// 				delete_ts IS NULL
// 				AND flavor = $1
// 			",
// 			flavor as i32,
// 		)
// 		.await?;

// 		let cpu_arch_counts = sql_fetch_all!(
// 			[ctx, (String, i64)]
// 			"
// 			SELECT
// 				system_info->'cpu'->>'cpu_arch' AS cpu_arch,
// 				count(*)::int AS count
// 			FROM db_pegboard.clients AS OF SYSTEM TIME '-5s'
// 			WHERE
// 				delete_ts IS NULL
// 				AND flavor = $1
// 			GROUP BY cpu_arch
// 			",
// 			flavor as i32,
// 		)
// 		.await?;
// 		let cpu_archs: HashMap<String, i64> = cpu_arch_counts.into_iter().collect();

// 		clients.insert(
// 			flavor.to_string(),
// 			json!({
// 				"aggregates": row,
// 				"cpu_archs": cpu_archs
// 			}),
// 		);
// 	}

// 	let (total_count, running_count) = sql_fetch_one!(
// 		[ctx, (i64, i64)]
// 		"
// 		SELECT count(*)::int, count(CASE WHEN stop_ts IS NULL THEN 1 END)::int
// 		FROM db_pegboard.actors AS OF SYSTEM TIME '-5s'
// 		",
// 	)
// 	.await?;

// 	Ok(json!({
// 		"clients": clients,
// 		"actors": {
// 			"total": total_count,
// 			"running": running_count,
// 		}
// 	}))
// }
