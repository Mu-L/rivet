use std::collections::HashMap;

use global_error::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::secret::Secret;

pub mod rivet;
pub use rivet::*;

pub mod default_ports {
	pub use super::rivet::default_ports::*;

	pub const FDB: u16 = 4500;
	pub const NATS: u16 = 4222;
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
#[derive(Default)]
pub struct Server {
	// Secrets
	pub jwt: JwtKey,
	#[serde(default)]
	pub tls: Option<Tls>,
	#[serde(default)]
	pub ssh: Option<Ssh>,

	#[serde(default)]
	pub rivet: rivet::Rivet,

	// Databases
	#[serde(default)]
	pub cockroachdb: CockroachDb,
	#[serde(default)]
	pub redis: RedisTypes,
	#[serde(default)]
	pub clickhouse: Option<ClickHouse>,
	#[serde(default)]
	pub prometheus: Option<Prometheus>,
	#[serde(default)]
	pub foundationdb: Option<FoundationDb>,
	#[serde(default)]
	pub vector_http: Option<VectorHttp>,

	// Services
	#[serde(default)]
	pub cloudflare: Option<Cloudflare>,
	#[serde(default)]
	pub nats: Nats,
	#[serde(default)]
	pub s3: S3,
	#[serde(default)]
	pub sendgrid: Option<Sendgrid>,
	#[serde(default)]
	pub loops: Option<Loops>,
	#[serde(default)]
	pub ip_info: Option<IpInfo>,
	#[serde(default)]
	pub hcaptcha: Option<Hcaptcha>,
	#[serde(default)]
	pub turnstile: Option<Turnstile>,

	// Enterprise Services
	#[serde(default)]
	pub stripe: Option<Stripe>,
	#[serde(default)]
	pub neon: Option<Neon>,

	#[serde(default)]
	pub linode: Option<Linode>,

	/// Deprecated
	#[serde(default)]
	pub nomad: Option<Nomad>,
}

impl Server {
	pub fn tls(&self) -> GlobalResult<&Tls> {
		Ok(unwrap_ref!(self.tls, "tls disabled"))
	}

	pub fn prometheus(&self) -> GlobalResult<&Prometheus> {
		Ok(unwrap_ref!(self.prometheus, "prometheus disabled"))
	}

	pub fn cloudflare(&self) -> GlobalResult<&Cloudflare> {
		Ok(unwrap_ref!(self.cloudflare, "cloudflare disabled"))
	}

	pub fn stripe(&self) -> GlobalResult<&Stripe> {
		Ok(unwrap_ref!(self.stripe, "stripe disabled"))
	}

	pub fn neon(&self) -> GlobalResult<&Neon> {
		Ok(unwrap_ref!(self.neon, "neon disabled"))
	}

	pub fn nomad(&self) -> GlobalResult<&Nomad> {
		Ok(unwrap_ref!(self.nomad, "nomad disabled"))
	}

	pub fn ssh(&self) -> GlobalResult<&Ssh> {
		Ok(unwrap_ref!(self.ssh, "ssh disabled"))
	}

	pub fn linode(&self) -> GlobalResult<&Linode> {
		Ok(unwrap_ref!(self.linode, "linode disabled"))
	}

	pub fn hcaptcha(&self) -> GlobalResult<&Hcaptcha> {
		Ok(unwrap_ref!(self.hcaptcha, "hcaptcha disabled"))
	}

	/// If automatically issuing TLS certs is enabled.
	pub fn is_tls_enabled(&self) -> bool {
		self.rivet
			.dns
			.as_ref()
			.map_or(false, |x| x.domain_main.is_some() && x.domain_job.is_some())
			&& self
				.cloudflare
				.as_ref()
				.map_or(false, |x| x.zone.main.is_some() && x.zone.job.is_some())
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct JwtKey {
	/// The public EdDSA key in a PEM format.
	pub public: String,
	/// The private EdDSA key in a PEM format.
	pub private: Secret<String>,
}

impl Default for JwtKey {
	fn default() -> Self {
		Self {
			public: "".into(),
			private: Secret::new("".into()),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Nomad {
	pub url: Url,
	pub server_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Ssh {
	pub server: SSHEntry,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct SSHEntry {
	pub private_key_openssh: Secret<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Linode {
	pub api_token: Secret<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct S3 {
	pub region: String,
	pub endpoint_internal: Url,
	/// If not defined, will default to endpoint_external.
	pub endpoint_edge_internal: Option<Url>,
	pub endpoint_external: Url,
	pub access_key_id: Secret<String>,
	pub secret_access_key: Secret<String>,
}

impl Default for S3 {
	fn default() -> Self {
		Self {
			region: "us-east-1".into(),
			endpoint_internal: Url::parse("http://127.0.0.1:9000").unwrap(),
			endpoint_edge_internal: None,
			endpoint_external: Url::parse("http://127.0.0.1:9000").unwrap(),
			access_key_id: Secret::new("root".into()),
			secret_access_key: Secret::new("root".into()),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct RedisTypes {
	pub ephemeral: Redis,
	pub persistent: Redis,
}

impl Default for RedisTypes {
	fn default() -> Self {
		// We assume that we're using the same Redis instance for both on a local instance.
		Self {
			ephemeral: Redis {
				url: Url::parse("redis://127.0.0.1:6379").unwrap(),
				username: None,
				password: None,
			},
			persistent: Redis {
				url: Url::parse("redis://127.0.0.1:6379").unwrap(),
				username: None,
				password: None,
			},
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Redis {
	pub url: Url,
	#[serde(default)]
	pub username: Option<String>,
	#[serde(default)]
	pub password: Option<Secret<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Cloudflare {
	pub account_id: String,
	pub zone: CloudflareZone,
	pub backend_dispatcher_namespace: Option<String>,
	// TODO: Specify permissions required
	pub auth_token: Secret<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct CloudflareZone {
	pub main: Option<String>,
	pub game: Option<String>,
	pub job: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Tls {
	pub root_ca_cert_pem: Secret<String>,
	pub root_ca_key_pem: Secret<String>,
	pub acme: TlsAcme,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct TlsAcme {
	#[serde(default)]
	pub directory: TlsAcmeDirectory,
	pub account_private_key_pem: Secret<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum TlsAcmeDirectory {
	LetsEncrypt,
	LetsEncryptStaging,
}

impl Default for TlsAcmeDirectory {
	fn default() -> Self {
		Self::LetsEncrypt
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Prometheus {
	pub url: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Hcaptcha {
	pub site_key_fallback: Option<Secret<String>>,
	pub secret_fallback: Option<Secret<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Turnstile {
	pub main_site_key: Option<String>,
	pub main_secret_key: Option<Secret<String>>,
	pub cdn_site_key: Option<String>,
	pub cdn_secret_key: Option<Secret<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Sendgrid {
	pub key: Secret<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Loops {
	pub token: Secret<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Stripe {
	pub secret_key: Secret<String>,
	/// The current Stripe webhook secret.
	///
	/// Secrets can be added at: https://dashboard.stripe.com/webhooks
	///
	/// Add the following events to the WebHook:
	/// - invoice.payment_succeeded
	/// - checkout.session.completed
	/// - payment_intent.succeeded
	pub webhook_secret: Secret<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Neon {
	pub api_key: Secret<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct IpInfo {
	pub token: Secret<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct CockroachDb {
	pub url: Url,
	pub cluster_identifier: String,
	pub username: String,
	pub password: Option<Secret<String>>,
	/// Automatically provisions new users when migrating the database.
	#[serde(default)]
	pub provision_users: HashMap<String, CockroachDbUser>,
	#[serde(default = "CockroachDb::default_min_connections")]
	pub min_connections: u32,
	#[serde(default = "CockroachDb::default_max_connections")]
	pub max_connections: u32,
}

impl Default for CockroachDb {
	fn default() -> Self {
		Self {
			url: Url::parse("postgresql://localhost:26257/defaultdb?sslmode=disable").unwrap(),
			cluster_identifier: "default".into(),
			username: "root".into(),
			password: None,
			provision_users: Default::default(),
			min_connections: Self::default_min_connections(),
			max_connections: Self::default_max_connections(),
		}
	}
}

impl CockroachDb {
	fn default_min_connections() -> u32 {
		4
	}

	fn default_max_connections() -> u32 {
		4096
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct CockroachDbUser {
	pub username: String,
	pub password: Secret<String>,
	pub role: CockroachDbUserRole,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum CockroachDbUserRole {
	Read,
	ReadWrite,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Nats {
	pub addresses: Addresses,
	pub port: Option<u16>,
	#[serde(default)]
	pub username: Option<String>,
	#[serde(default)]
	pub password: Option<Secret<String>>,
}

impl Default for Nats {
	fn default() -> Self {
		Self {
			addresses: Addresses::Static(vec!["127.0.0.1:4222".to_string()]),
			port: None,
			username: None,
			password: None,
		}
	}
}

impl Nats {
	pub fn port(&self) -> u16 {
		self.port.unwrap_or(self::default_ports::NATS)
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct ClickHouse {
	/// URL to the HTTP access port for ClickHouse.
	pub http_url: Url,
	/// URL to the native access port for ClickHouse.
	pub native_url: Url,
	pub username: String,
	#[serde(default)]
	pub password: Option<Secret<String>>,
	#[serde(default)]
	pub provision_users: HashMap<String, ClickHouseUser>,
	#[serde(default)]
	pub secure: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct ClickHouseUser {
	pub username: String,
	pub password: Secret<String>,
	pub role: ClickHouseUserRole,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ClickHouseUserRole {
	Admin,
	Write,
	ReadOnly,
}

impl ClickHouseUserRole {
	pub fn to_clickhouse_role(&self) -> &'static str {
		use ClickHouseUserRole::*;
		match self {
			Admin => "admin",
			Write => "write",
			ReadOnly => "readonly",
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct FoundationDb {
	pub cluster_description: String,
	pub cluster_id: String,
	pub addresses: Addresses,
	pub port: Option<u16>,
}

impl Default for FoundationDb {
	fn default() -> Self {
		Self {
			cluster_description: "fdb".to_string(),
			cluster_id: "fdb".to_string(),
			addresses: Addresses::Static(vec!["127.0.0.1:4500".to_string()]),
			port: None,
		}
	}
}

impl FoundationDb {
	pub fn port(&self) -> u16 {
		self.port.unwrap_or(self::default_ports::FDB)
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct VectorHttp {
	pub host: String,
	pub port: u16,
}

impl Default for VectorHttp {
	fn default() -> Self {
		Self {
			host: "127.0.0.1".into(),
			port: 5022,
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Addresses {
	Dynamic { fetch_endpoint: Url },
	Static(Vec<String>),
}
