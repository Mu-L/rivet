use std::{env, collections::HashMap};

use chirp_worker::prelude::*;
use include_dir::{include_dir, Dir};
use indoc::{formatdoc, indoc};
use s3_util::Provider;

use super::ServerCtx;

pub fn common() -> String {
	[
		format!("apt-get update -y"),
		format!("apt-get install -y apt-transport-https ca-certificates gnupg2 software-properties-common curl jq unzip"),
	].join("\n")
}

pub fn node_exporter() -> String {
	include_str!("files/node_exporter.sh").to_string()
}
pub fn sysctl() -> String {
	include_str!("files/sysctl.sh").to_string()
}

pub fn docker() -> String {
	include_str!("files/docker.sh").to_string()
}

pub fn lz4() -> String {
	format!("apt-get install -y lz4")
}

pub fn skopeo() -> String {
	format!("apt-get install -y skopeo")
}

pub fn umoci() -> String {
	formatdoc!(
		r#"
		curl -Lf -o /usr/bin/umoci "https://github.com/opencontainers/umoci/releases/download/v0.4.7/umoci.amd64"
		chmod +x /usr/bin/umoci
		"#
	)
}

pub fn cnitool() -> String {
	formatdoc!(
		r#"
		curl -Lf -o /usr/bin/cnitool "https://github.com/rivet-gg/cni/releases/download/v1.1.2-build3/cnitool"
		chmod +x /usr/bin/cnitool
		"#
	)
}

pub fn cni_plugins() -> String {
	include_str!("files/cni_plugins.sh").to_string()
}

pub fn nomad(server: &ServerCtx) -> String {
	let servers = &["127.0.0.1:5000"];

	include_str!("files/nomad.sh")
		.replace("__NODE_NAME__", &server.name)
		.replace("__SERVER_ID__", &server.server_id.to_string())
		.replace("__DATACENTER_ID__", &server.datacenter_id.to_string())
		.replace("__CLUSTER_ID__", &server.cluster_id.to_string())
		// HACK: Hardcoded to Linode
		.replace("__PUBLIC_IFACE__", "eth0")
		.replace("__VLAN_IP__", &server.vlan_ip.to_string())
		// HACK: Hardcoded to Linode
		.replace("__VLAN_IFACE__", "eth1")
		.replace(
			"__SERVER_JOIN__",
			&servers
				.iter()
				.map(|x| format!("\"{x}\""))
				.collect::<Vec<_>>()
				.join(", "),
		)
		.replace("__GG_VLAN_SUBNET__", &util::net::gg::vlan_ip_net().to_string())
		.replace("__ATS_VLAN_SUBNET__", &util::net::ats::vlan_ip_net().to_string())
}

/// Installs Traefik, but does not create the Traefik service.
pub fn traefik() -> String {
	include_str!("files/traefik.sh").to_string()
}

pub struct TlsCert {
	pub cert_pem: String,
	pub key_pem: String,
}

pub struct TraefikInstance {
	pub name: String,
	pub static_config: String,
	pub dynamic_config: String,
	pub tls_certs: HashMap<String, TlsCert>,
	pub tcp_server_transports: HashMap<String, ServerTransport>,
}

pub struct ServerTransport {
	pub server_name: String,
	pub root_cas: Vec<String>,
	pub certs: Vec<TlsCert>,
}

/// Creates a Traefik instance.
///
/// Requires `traefik()`.
pub fn traefik_instance(config: TraefikInstance) -> String {
	let config_name = &config.name;

	let mut script = include_str!("files/traefik_instance.sh")
		.replace("__NAME__", &config.name)
		.replace("__STATIC_CONFIG__", &config.static_config)
		.replace("__DYNAMIC_CONFIG__", &config.dynamic_config);

	// Add TLS certs
	for (cert_id, cert) in config.tls_certs {
		script.push_str(&formatdoc!(
			r#"
			cat << 'EOF' > /etc/{config_name}/tls/{cert_id}_cert.pem
			{cert}
			EOF

			cat << 'EOF' > /etc/{config_name}/tls/{cert_id}_key.pem
			{key}
			EOF

			cat << 'EOF' > /etc/{config_name}/dynamic/tls/{cert_id}.toml
			[[tls.certificates]]
				certFile = "/etc/{config_name}/tls/{cert_id}_cert.pem"
				keyFile = "/etc/{config_name}/tls/{cert_id}_key.pem"
			EOF
			"#,
			cert = cert.cert_pem,
			key = cert.key_pem,
		));
	}

	for (transport_id, transport) in config.tcp_server_transports {
		// Build config
		let root_cas = transport
			.root_cas
			.iter()
			.enumerate()
			.map(|(i, _)| {
				format!("\"/etc/{config_name}/tls/transport_{transport_id}_root_ca_{i}_cert.pem\"",)
			})
			.collect::<Vec<_>>()
			.join(", ");
		let mut transport_config = formatdoc!(
			r#"
			[tcp.serversTransports.{transport_id}.tls]
				serverName = "{server_name}"
				rootCAs = [{root_cas}]
			"#,
			server_name = transport.server_name
		);

		// Write root CAs
		for (i, cert) in transport.root_cas.iter().enumerate() {
			script.push_str(&formatdoc!(
				r#"
				cat << 'EOF' > /etc/{config_name}/tls/transport_{transport_id}_root_ca_{i}_cert.pem
				{cert}
				EOF
				"#,
			));
		}

		// Write certs
		for (i, cert) in transport.certs.iter().enumerate() {
			script.push_str(&formatdoc!(
				r#"
				cat << 'EOF' > /etc/{config_name}/tls/transport_{transport_id}_cert_{i}_cert.pem
				{cert}
				EOF

				cat << 'EOF' > /etc/{config_name}/tls/transport_{transport_id}_cert_{i}_key.pem
				{key}
				EOF
				"#,
				cert = cert.cert_pem,
				key = cert.key_pem,
			));
			transport_config.push_str(&formatdoc!(
				r#"
				[[tcp.serversTransports.{transport_id}.tls.certificates]]
					certFile = "/etc/{config_name}/tls/transport_{transport_id}_cert_{i}_cert.pem"
					keyFile = "/etc/{config_name}/tls/transport_{transport_id}_cert_{i}_key.pem"
				"#
			))
		}

		// Write config
		script.push_str(&formatdoc!(
			r#"
			cat << 'EOF' > /etc/{config_name}/dynamic/transport_{transport_id}.toml
			{transport_config}
			EOF
			"#
		));
	}

	script
}

const TUNNEL_SERVICES: &[&str] = &["nomad", "api-route", "vector"];

pub fn traefik_tunnel(
) -> GlobalResult<String> {
	// Build transports for each service
	let mut tcp_server_transports = HashMap::new();
	for service in TUNNEL_SERVICES {
		tcp_server_transports.insert(
			service.to_string(),
			ServerTransport {
				server_name: format!("{service}.tunnel.rivet.gg"),
				root_cas: vec![env::var("TLS_ROOT_CA_CERT_PEM")?],
				certs: vec![
					TlsCert {
						cert_pem: env::var("TLS_CERT_LOCALLY_SIGNED_JOB_CERT_PEM")?,
						key_pem: env::var("TLS_CERT_LOCALLY_SIGNED_JOB_KEY_PEM")?,
					},
				],
			},
		);
	}

	Ok(traefik_instance(TraefikInstance {
		name: "tunnel".into(),
		static_config: tunnel_traefik_static_config(),
		dynamic_config: tunnel_traefik_dynamic_config(&env::var("K8S_TRAEFIK_TUNNEL_EXTERNAL_IP")?),
		tls_certs: Default::default(),
		tcp_server_transports,
	}))
}

fn tunnel_traefik_static_config() -> String {
	let mut config = formatdoc!(
		r#"
		[providers]
			[providers.file]
				directory = "/etc/tunnel/dynamic"
		"#
	);

	for (i, service) in TUNNEL_SERVICES.iter().enumerate() {
		config.push_str(&formatdoc!(
			r#"
			[entryPoints.{service}]
				address = "127.0.0.1:{port}"
			"#,
			port = 5000 + i
		))
	}

	config
}

fn tunnel_traefik_dynamic_config(tunnel_external_ip: &str) -> String {
	let mut config = String::new();
	for service in TUNNEL_SERVICES.iter() {
		config.push_str(&formatdoc!(
			r#"
			[tcp.routers.{service}]
				entryPoints = ["{service}"]
				rule = "HostSNI(`*`)"  # Match all ingress, unrelated to the outbound TLS
				service = "{service}"

			[tcp.services.{service}.loadBalancer]
				serversTransport = "{service}"

				[[tcp.services.{service}.loadBalancer.servers]]
					address = "{tunnel_external_ip}:5000"
					tls = true
			"#
		))
	}

	config
}

pub struct VectorConfig {
	pub prometheus_targets: HashMap<String, VectorPrometheusTarget>,
}

pub struct VectorPrometheusTarget {
	pub endpoint: String,
	pub scrape_interval: usize,
}

pub fn vector(config: &VectorConfig) -> String {
	let sources = config
		.prometheus_targets
		.keys()
		.map(|x| format!("\"prometheus_{x}\""))
		.collect::<Vec<_>>()
		.join(", ");

	let mut config_str = formatdoc!(
		r#"
		[api]
			enabled = true

		[sinks.vector_sink]
			type = "vector"
			inputs = [{sources}]
			address = "127.0.0.1:5002"
			healthcheck.enabled = false
			compression = true
		"#
	);

	for (
		key,
		VectorPrometheusTarget {
			endpoint,
			scrape_interval,
		},
	) in &config.prometheus_targets
	{
		config_str.push_str(&formatdoc!(
			r#"
			[sources.prometheus_{key}]
				type = "prometheus_scrape"
				endpoints = ["{endpoint}"]
				scrape_interval_secs = {scrape_interval}
			"#
		));
	}

	include_str!("files/vector.sh").replace("__VECTOR_CONFIG__", &config_str)
}

pub async fn traffic_server(server: &ServerCtx) -> GlobalResult<String> {
	// Write config to files
	let config = traffic_server_config(server).await?;
	let mut config_scripts = config
		.into_iter()
		.map(|(k, v)| format!("cat << 'EOF' > /etc/trafficserver/{k}\n{v}\nEOF\n"))
		.collect::<Vec<_>>();

	// Update default storage config size to be entire filesystem size
	config_scripts.push(
		indoc!(
			"
		df -h / |
		awk 'NR==2 {print $2}' |
		xargs -I {} sed -i 's/64G/{}/' /etc/trafficserver/storage.config
		"
		)
		.to_string(),
	);

	let script = include_str!("files/traffic_server.sh")
		.replace(
			"__IMAGE__",
			"ghcr.io/rivet-gg/apache-traffic-server:9934dc2",
		)
		.replace("__CONFIG__", &config_scripts.join("\n\n"));

	Ok(script)
}

static TRAFFIC_SERVER_CONFIG_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/workers/server_install/install_scripts/files/traffic_server");

async fn traffic_server_config(
	server: &ServerCtx,
) -> GlobalResult<Vec<(String, String)>> {

	// Static files
	let mut config_files = Vec::<(String, String)>::new();
	for entry in TRAFFIC_SERVER_CONFIG_DIR.entries() {
		if let Some(file) = entry.as_file() {
			let key = unwrap!(
				unwrap!(file
					.path()
					.file_name()
				)
				.to_str()
			)
			.to_string();
			
			let value = unwrap!(file.contents_utf8());
			let value = value.replace("__VLAN_IP__", &server.vlan_ip.to_string());
			config_files.push((key, value));
		}
	}

	// Storage (default value of 64 gets overwritten in config script)
	let volume_size = 64;
	config_files.push((
		"storage.config".to_string(),
		format!("/var/cache/trafficserver {volume_size}G"),
	));

	// Remap & S3
	let mut remap = String::new();
	let default_s3_provider = Provider::default()?;
	if s3_util::s3_provider_active("bucket-build", Provider::Minio) {
		let output = gen_s3_provider(Provider::Minio, default_s3_provider).await?;
		remap.push_str(&output.append_remap);
		config_files.extend(output.config_files);
	}
	if s3_util::s3_provider_active("bucket-build", Provider::Backblaze) {
		let output =
			gen_s3_provider(Provider::Backblaze, default_s3_provider).await?;
		remap.push_str(&output.append_remap);
		config_files.extend(output.config_files);
	}
	if s3_util::s3_provider_active("bucket-build", Provider::Aws) {
		let output = gen_s3_provider(Provider::Aws, default_s3_provider).await?;
		remap.push_str(&output.append_remap);
		config_files.extend(output.config_files);
	}
	config_files.push(("remap.config".to_string(), remap));

	Ok(config_files)
}

struct GenRemapS3ProviderOutput {
	/// Append to remap.config
	append_remap: String,

	/// Concat with config files
	config_files: Vec<(String, String)>,
}

async fn gen_s3_provider(
	provider: Provider,
	default_s3_provider: Provider,
) -> GlobalResult<GenRemapS3ProviderOutput> {
	let mut remap = String::new();
	let provider_name = provider.as_str();
	let endpoint_external = s3_util::s3_endpoint_external("bucket-build", provider)?;
	let region = s3_util::s3_region("bucket-build", provider)?;
	let (access_key_id, secret_access_key) = s3_util::s3_credentials("bucket-build", provider)?;

	// Build plugin chain
	let plugins = format!("@plugin=tslua.so @pparam=/etc/trafficserver/strip_headers.lua @plugin=s3_auth.so @pparam=--config @pparam=s3_auth_v4_{provider_name}.config");

	// Add remap
	remap.push_str(&format!(
		"map /s3-cache/{provider_name} {endpoint_external} {plugins}\n",
	));

	// Add default route
	if default_s3_provider == provider {
		remap.push_str(&format!(
			"map /s3-cache {endpoint_external} {plugins}\n",
		));
	}

	// Add credentials
	let mut config_files = Vec::<(String, String)>::new();
	config_files.push((
		format!("s3_auth_v4_{provider_name}.config"),
		formatdoc!(
			r#"
			access_key={access_key_id}
			secret_key={secret_access_key}
			version=4
			v4-region-map=s3_region_map_{provider_name}.config
			"#,
		),
	));
	config_files.push((
		format!("s3_region_map_{provider_name}.config"),
		formatdoc!(
			r#"
			# Default region
			{s3_host}: {s3_region}
			"#,
			s3_host = endpoint_external.split_once("://").unwrap().1,
			s3_region = region,
		),
	));

	Ok(GenRemapS3ProviderOutput {
		append_remap: remap,
		config_files,
	})
}
