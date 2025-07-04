use std::{
	convert::{TryFrom, TryInto},
	net::{IpAddr, Ipv4Addr},
	str::FromStr,
};

use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend;
use serde::{Deserialize, Serialize};
use strum::FromRepr;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Cluster {
	pub cluster_id: Uuid,
	pub name_id: String,
	/// Unset for the default cluster.
	pub owner_team_id: Option<Uuid>,
	pub create_ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Datacenter {
	pub datacenter_id: Uuid,
	pub cluster_id: Uuid,
	pub name_id: String,
	pub display_name: String,
	pub provider: Provider,
	pub provider_datacenter_id: String,
	pub provider_api_token: Option<String>,
	pub pools: Vec<Pool>,
	pub build_delivery_method: BuildDeliveryMethod,
	pub prebakes_enabled: bool,
	pub create_ts: i64,
	pub guard_public_hostname: GuardPublicHostname,
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum Provider {
	/// Servers are manually provisioned and connected.
	Manual = 1,
	Linode = 0,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Pool {
	pub pool_type: PoolType,
	/// See docs on failover (/docs/packages/cluster/SERVER_PROVISIONING.md#creating-a-new-server)
	pub hardware: Vec<Hardware>,
	pub desired_count: u32,
	pub min_count: u32,
	pub max_count: u32,
	pub drain_timeout: u64,
	#[serde(default)]
	pub margin: u32,
}

// Backwards compatibility
impl TryFrom<backend::cluster::Pool> for Pool {
	type Error = GlobalError;

	fn try_from(value: backend::cluster::Pool) -> GlobalResult<Self> {
		Ok(Pool {
			pool_type: unwrap!(PoolType::from_repr(value.pool_type.try_into()?)),
			hardware: value
				.hardware
				.iter()
				.map(|h| Hardware {
					provider_hardware: h.provider_hardware.clone(),
				})
				.collect(),
			desired_count: value.desired_count,
			min_count: value.min_count,
			max_count: value.max_count,
			drain_timeout: value.drain_timeout,
			margin: 0,
		})
	}
}

#[derive(
	Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromRepr,
)]
pub enum PoolType {
	Job = 0,
	Gg = 1,
	Ats = 2,
	Pegboard = 3,
	PegboardIsolate = 4,
	Fdb = 5,
	Worker = 6,
	Nats = 7,
	Guard = 8,
}

impl std::fmt::Display for PoolType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PoolType::Job => write!(f, "job"),
			PoolType::Gg => write!(f, "gg"),
			PoolType::Ats => write!(f, "ats"),
			PoolType::Pegboard => write!(f, "pegboard"),
			PoolType::PegboardIsolate => write!(f, "pegboard-isolate"),
			PoolType::Fdb => write!(f, "fdb"),
			PoolType::Worker => write!(f, "worker"),
			PoolType::Nats => write!(f, "nats"),
			PoolType::Guard => write!(f, "guard"),
		}
	}
}

impl FromStr for PoolType {
	type Err = GlobalError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"job" => Ok(PoolType::Job),
			"gg" => Ok(PoolType::Gg),
			"ats" => Ok(PoolType::Ats),
			"pegboard" => Ok(PoolType::Pegboard),
			"pegboard-isolate" => Ok(PoolType::PegboardIsolate),
			"fdb" => Ok(PoolType::Fdb),
			"worker" => Ok(PoolType::Worker),
			"nats" => Ok(PoolType::Nats),
			"guard" => Ok(PoolType::Guard),
			_ => bail!("Invalid PoolType string: {}", s),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Hardware {
	pub provider_hardware: String,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct PoolUpdate {
	pub pool_type: PoolType,

	// Each can be optionally updated
	pub hardware: Vec<Hardware>,
	pub desired_count: Option<u32>,
	pub min_count: Option<u32>,
	pub max_count: Option<u32>,
	pub drain_timeout: Option<u64>,
	pub margin: Option<u32>,
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum BuildDeliveryMethod {
	TrafficServer = 0,
	S3Direct = 1,
}

impl From<rivet_config::config::rivet::BuildDeliveryMethod> for BuildDeliveryMethod {
	fn from(value: rivet_config::config::rivet::BuildDeliveryMethod) -> BuildDeliveryMethod {
		match value {
			rivet_config::config::rivet::BuildDeliveryMethod::TrafficServer => {
				BuildDeliveryMethod::TrafficServer
			}
			rivet_config::config::rivet::BuildDeliveryMethod::S3Direct => {
				BuildDeliveryMethod::S3Direct
			}
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
	pub server_id: Uuid,
	pub datacenter_id: Uuid,
	pub pool_type: PoolType,
	pub provider_server_id: Option<String>,
	pub lan_ip: Option<IpAddr>,
	pub wan_ip: Option<IpAddr>,
	pub create_ts: i64,
	pub install_complete_ts: Option<i64>,
	pub cloud_destroy_ts: Option<i64>,
	pub state: ServerState,
}

#[derive(
	Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromRepr,
)]
pub enum ServerState {
	Provisioning = 0,
	Installing = 1,
	Running = 2,
	Tainted = 3,
	Draining = 4,
	TaintedDraining = 5,
	Destroyed = 6,
}

#[derive(Debug, Default, Clone)]
pub struct Filter {
	pub server_ids: Option<Vec<Uuid>>,
	pub datacenter_ids: Option<Vec<Uuid>>,
	pub cluster_ids: Option<Vec<Uuid>>,
	pub pool_types: Option<Vec<PoolType>>,
	pub public_ips: Option<Vec<Ipv4Addr>>,
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum TlsState {
	Creating = 0,
	Active = 1,
	Renewing = 2,
}

/// See `rivet_config::config::server::rivet::GuardPublicHostname` for docs.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash)]
pub enum GuardPublicHostname {
	DnsParent(String),
	Static(String),
}

impl GuardPublicHostname {
	pub fn from_columns(
		config: &rivet_config::Config,
		datacenter_id: Uuid,
		gph_dns_parent: Option<String>,
		gph_static: Option<String>,
	) -> GlobalResult<GuardPublicHostname> {
		let gph = match (gph_dns_parent, gph_static) {
			(Some(x), None) => GuardPublicHostname::DnsParent(x),
			(None, Some(x)) => GuardPublicHostname::Static(x),
			(Some(_), Some(_)) => bail!(
				"guard public hostname dns parent & static cannot be defined at the same time"
			),
			(None, None) => {
				if let Ok(domain_job) = config.server()?.rivet.domain_job() {
					// Fall back to auto-generated hostname
					let hostname = format!("{}.{domain_job}", datacenter_id);
					crate::types::GuardPublicHostname::DnsParent(hostname)
				} else {
					bail!("no guard public hostname specified in dc {datacenter_id}")
				}
			}
		};

		Ok(gph)
	}

	pub fn into_columns(self) -> (Option<String>, Option<String>) {
		match self {
			GuardPublicHostname::DnsParent(x) => (Some(x), None),
			GuardPublicHostname::Static(x) => (None, Some(x)),
		}
	}
}
