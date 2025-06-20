use std::{collections::HashMap, path::PathBuf};

use chirp_workflow::prelude::*;
use url::Url;

use super::{
	super::{
		fdb::FDB_VERSION,
		traefik::{
			TUNNEL_CRDB_PORT, TUNNEL_OTEL_PORT, TUNNEL_PROMETHEUS_PORT,
			TUNNEL_REDIS_EPHEMERAL_PORT, TUNNEL_REDIS_PERSISTENT_PORT, TUNNEL_S3_PORT,
		},
	},
	TUNNEL_API_EDGE_PORT,
};

pub fn install(config: &rivet_config::Config) -> GlobalResult<String> {
	let provision_config = &config.server()?.rivet.provision()?;

	Ok(include_str!("../../files/rivet_worker_install.sh")
		.replace(
			"__EDGE_SERVER_BINARY_URL__",
			provision_config.edge_server_binary_url.as_ref(),
		)
		.replace("__FDB_VERSION__", FDB_VERSION))
}

pub fn configure(config: &rivet_config::Config) -> GlobalResult<String> {
	let server_config = config.server()?;

	use rivet_config::config::*;
	let edge_config = Root {
		server: Some(Server {
			// TODO: Is this safe?
			jwt: server_config.jwt.clone(),
			tls: server_config.tls.clone(),
			rivet: Rivet {
				namespace: server_config.rivet.namespace.clone(),
				instance_id: server_config.rivet.instance_id,
				clusters: Some({
					let mut clusters = HashMap::new();

					clusters.insert(
						"rivet".into(),
						Cluster {
							// NOTE: Gets replaced by a template later
							id: Uuid::nil(),
							bootstrap_datacenters: HashMap::new(),
						},
					);

					clusters
				}),
				auth: server_config.rivet.auth.clone(),
				api_public: ApiPublic {
					// NOTE: Templated later
					public_origin: None,
					respect_forwarded_for: Some(true),
					..server_config.rivet.api_public.clone()
				},
				cache: Cache {
					driver: CacheDriver::InMemory,
				},
				ui: Ui {
					enable: Some(false),
					..Default::default()
				},
				dns: server_config.rivet.dns.clone(),
				edge: Some(Edge {
					// NOTE: Gets replaced by a template later
					cluster_id: Uuid::nil(),
					datacenter_id: Uuid::nil(),
					server_id: Uuid::nil(),
					api_lan_address: None,
					intercom_endpoint: Url::parse(&format!("http://127.0.0.1:{TUNNEL_API_EDGE_PORT}"))?,
					redirect_logs_dir: Some(PathBuf::from("/var/log/rivet-edge-server")),
				}),
				..Default::default()
			},
			cockroachdb: CockroachDb {
				url: Url::parse(&format!(
					"postgres://127.0.0.1:{TUNNEL_CRDB_PORT}/{}.postgres?sslmode=require",
					server_config.cockroachdb.cluster_identifier,
				))?,
				..server_config.cockroachdb.clone()
			},
			redis: RedisTypes {
				ephemeral: Redis {
					url: Url::parse(&format!(
						"rediss://127.0.0.1:{TUNNEL_REDIS_EPHEMERAL_PORT}/#insecure",
					))?,
					..server_config.redis.ephemeral.clone()
				},
				persistent: Redis {
					url: Url::parse(&format!(
						"rediss://127.0.0.1:{TUNNEL_REDIS_PERSISTENT_PORT}/#insecure",
					))?,
					..server_config.redis.persistent.clone()
				},
			},
			clickhouse: server_config.clickhouse.as_ref().map(|clickhouse| GlobalResult::Ok(ClickHouse {
				// TODO: This doesn't work over Traefik since the client needs to specify the same
				// hostname as ClickHouse
				// http_url: Url::parse(&format!(
				// 	"https://127.0.0.1:{TUNNEL_CLICKHOUSE_PORT}",
				// ))?,
				// native_url: Url::parse(&format!(
				// 	"clickhouse://127.0.0.1:{TUNNEL_CLICKHOUSE_NATIVE_PORT}",
				// ))?,
				provision_users: HashMap::new(),
				..clickhouse.clone()
			})).transpose()?,
			prometheus: Some(Prometheus {
				url: Url::parse(&format!(
					"http://127.0.0.1:{TUNNEL_PROMETHEUS_PORT}",
				))?,
			}),
			foundationdb: Some(FoundationDb {
				addresses: Addresses::Dynamic {
					fetch_endpoint: Url::parse(&format!("http://127.0.0.1:{TUNNEL_API_EDGE_PORT}/provision/datacenters/___DATACENTER_ID___/servers?pools=fdb"))?,
				},
				..Default::default()
			}),
			vector_http: Some(VectorHttp::default()),
			nats: Nats {
				addresses: Addresses::Dynamic {
					fetch_endpoint: Url::parse(&format!("http://127.0.0.1:{TUNNEL_API_EDGE_PORT}/provision/datacenters/___DATACENTER_ID___/servers?pools=nats"))?,
				},
				..server_config.nats.clone()
			},
			s3: S3 {
				endpoint_internal: Url::parse(&format!("http://127.0.0.1:{TUNNEL_S3_PORT}"))?,
				..server_config.s3.clone()
			},
			ip_info: server_config.ip_info.clone(),
			turnstile: server_config.turnstile.clone(),
			linode: server_config.linode.clone(),
			..Default::default()
		}),
		guard: None,
	};
	let mut edge_config_json = serde_json::to_value(&edge_config)?;

	// Add placeholders for templating
	edge_config_json["server"]["rivet"]["default_cluster_id"] = "___CLUSTER_ID___".into();
	edge_config_json["server"]["rivet"]["clusters"]["rivet"]["id"] = "___CLUSTER_ID___".into();
	edge_config_json["server"]["rivet"]["edge"]["cluster_id"] = "___CLUSTER_ID___".into();
	edge_config_json["server"]["rivet"]["edge"]["datacenter_id"] = "___DATACENTER_ID___".into();
	edge_config_json["server"]["rivet"]["edge"]["server_id"] = "___SERVER_ID___".into();
	// HACK: The url crate makes all url hostnames lowercase, revert that change
	edge_config_json["server"]["rivet"]["api_public"]["public_origin"] = server_config
		.rivet
		.edge_api_url("___DATACENTER_NAME_ID___")?
		.to_string()
		.replace("___datacenter_name_id___", "___DATACENTER_NAME_ID___")
		.into();

	let otel_enabled = std::env::var("RIVET_OTEL_ENABLED").unwrap_or("0".to_string());
	let otel_sampler_ratio = std::env::var("RIVET_OTEL_SAMPLER_RATIO")
		.ok()
		.and_then(|s| s.parse::<f64>().ok())
		.unwrap_or(0.001);

	Ok(include_str!("../../files/rivet_worker_configure.sh")
		.replace("__NAMESPACE__", &server_config.rivet.namespace)
		.replace(
			"__RIVET_EDGE_CONFIG__",
			&serde_json::to_string_pretty(&edge_config_json)?,
		)
		.replace("__OTEL_PORT__", &TUNNEL_OTEL_PORT.to_string())
		.replace("__OTEL_ENABLED__", &otel_enabled.to_string())
		.replace("__OTEL_SAMPLER_RATIO__", &otel_sampler_ratio.to_string()))
}
