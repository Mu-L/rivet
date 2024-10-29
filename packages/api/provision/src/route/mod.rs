use std::net::Ipv4Addr;

use api_helper::define_router;
use hyper::{Body, Request, Response};
use uuid::Uuid;

pub mod datacenters;
pub mod servers;

define_router! {
	routes: {
		"datacenters" / Uuid / "tls": {
			GET: datacenters::tls(
				internal_endpoint: true,
			),
		},

		"servers" / Ipv4Addr / "info": {
			GET: servers::info(
				internal_endpoint: true,
			),
		},
	},
}