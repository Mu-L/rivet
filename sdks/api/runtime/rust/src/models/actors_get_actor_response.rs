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
pub struct ActorsGetActorResponse {
    #[serde(rename = "actor")]
    pub actor: Box<crate::models::ActorsActor>,
}

impl ActorsGetActorResponse {
    pub fn new(actor: crate::models::ActorsActor) -> ActorsGetActorResponse {
        ActorsGetActorResponse {
            actor: Box::new(actor),
        }
    }
}


