use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod docker;
pub mod javascript;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Runtime {
	Docker(docker::Build),
	#[serde(rename = "javascript")]
	JavaScript(javascript::Build),
}

#[derive(
	Debug, Copy, Clone, Serialize, Deserialize, strum::AsRefStr, JsonSchema, clap::ValueEnum,
)]
#[serde(rename_all = "camelCase")]
pub enum Compression {
	/// No compression.
	#[strum(serialize = "none")]
	None,

	/// LZ4 compression. Fast compression optimized for fast lobby start times.
	#[strum(serialize = "lz4")]
	Lz4,
}

impl Compression {
	pub fn default_from_bundle_kind(build_kind: docker::BundleKind) -> Self {
		match build_kind {
			docker::BundleKind::DockerImage => Compression::None,
			docker::BundleKind::OciBundle => Compression::Lz4,
		}
	}
}
