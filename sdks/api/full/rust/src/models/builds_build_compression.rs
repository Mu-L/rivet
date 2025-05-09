/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum BuildsBuildCompression {
	#[serde(rename = "none")]
	None,
	#[serde(rename = "lz4")]
	Lz4,
}

impl ToString for BuildsBuildCompression {
	fn to_string(&self) -> String {
		match self {
			Self::None => String::from("none"),
			Self::Lz4 => String::from("lz4"),
		}
	}
}

impl Default for BuildsBuildCompression {
	fn default() -> BuildsBuildCompression {
		Self::None
	}
}
