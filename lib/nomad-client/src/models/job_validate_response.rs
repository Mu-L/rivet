// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

/*
 * Nomad
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.1.4
 * Contact: support@hashicorp.com
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JobValidateResponse {
    #[serde(rename = "DriverConfigValidated", skip_serializing_if = "Option::is_none")]
    pub driver_config_validated: Option<bool>,
    #[serde(rename = "Error", skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(rename = "ValidationErrors", skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<String>>,
    #[serde(rename = "Warnings", skip_serializing_if = "Option::is_none")]
    pub warnings: Option<String>,
}

impl JobValidateResponse {
    pub fn new() -> JobValidateResponse {
        JobValidateResponse {
            driver_config_validated: None,
            error: None,
            validation_errors: None,
            warnings: None,
        }
    }
}

