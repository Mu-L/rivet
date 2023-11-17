/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AuthIdentityCompleteEmailVerificationRequest {
    /// The code sent to the requestee's email.
    #[serde(rename = "code")]
    pub code: String,
    #[serde(rename = "verification_id")]
    pub verification_id: uuid::Uuid,
}

impl AuthIdentityCompleteEmailVerificationRequest {
    pub fn new(code: String, verification_id: uuid::Uuid) -> AuthIdentityCompleteEmailVerificationRequest {
        AuthIdentityCompleteEmailVerificationRequest {
            code,
            verification_id,
        }
    }
}

