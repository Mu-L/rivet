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
pub struct ConsulIngressListener {
    #[serde(rename = "Port", skip_serializing_if = "Option::is_none")]
    pub port: Option<i32>,
    #[serde(rename = "Protocol", skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(rename = "Services", skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<crate::models::ConsulIngressService>>,
}

impl ConsulIngressListener {
    pub fn new() -> ConsulIngressListener {
        ConsulIngressListener {
            port: None,
            protocol: None,
            services: None,
        }
    }
}

