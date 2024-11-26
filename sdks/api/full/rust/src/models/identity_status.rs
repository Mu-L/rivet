/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

/// IdentityStatus : The current status of an identity. This helps players understand if another player is currently playing or has their game in the background.

/// The current status of an identity. This helps players understand if another player is currently playing or has their game in the background.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum IdentityStatus {
	#[serde(rename = "online")]
	Online,
	#[serde(rename = "away")]
	Away,
	#[serde(rename = "offline")]
	Offline,
}

impl ToString for IdentityStatus {
	fn to_string(&self) -> String {
		match self {
			Self::Online => String::from("online"),
			Self::Away => String::from("away"),
			Self::Offline => String::from("offline"),
		}
	}
}

impl Default for IdentityStatus {
	fn default() -> IdentityStatus {
		Self::Online
	}
}