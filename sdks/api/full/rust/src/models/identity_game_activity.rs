/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

/// IdentityGameActivity : The game an identity is currently participating in.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct IdentityGameActivity {
	#[serde(rename = "game")]
	pub game: Box<crate::models::GameHandle>,
	/// A short activity message about the current game activity.
	#[serde(rename = "message")]
	pub message: String,
	/// JSON data seen only by the given identity and their mutual followers.
	#[serde(
		rename = "mutual_metadata",
		default,
		with = "::serde_with::rust::double_option",
		skip_serializing_if = "Option::is_none"
	)]
	pub mutual_metadata: Option<Option<serde_json::Value>>,
	/// JSON data seen by anyone.
	#[serde(
		rename = "public_metadata",
		default,
		with = "::serde_with::rust::double_option",
		skip_serializing_if = "Option::is_none"
	)]
	pub public_metadata: Option<Option<serde_json::Value>>,
}

impl IdentityGameActivity {
	/// The game an identity is currently participating in.
	pub fn new(game: crate::models::GameHandle, message: String) -> IdentityGameActivity {
		IdentityGameActivity {
			game: Box::new(game),
			message,
			mutual_metadata: None,
			public_metadata: None,
		}
	}
}