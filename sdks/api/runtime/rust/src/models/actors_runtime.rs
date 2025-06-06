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
pub struct ActorsRuntime {
    #[serde(rename = "build")]
    pub build: uuid::Uuid,
    #[serde(rename = "arguments", skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,
    #[serde(rename = "environment", skip_serializing_if = "Option::is_none")]
    pub environment: Option<::std::collections::HashMap<String, String>>,
}

impl ActorsRuntime {
    pub fn new(build: uuid::Uuid) -> ActorsRuntime {
        ActorsRuntime {
            build,
            arguments: None,
            environment: None,
        }
    }
}


