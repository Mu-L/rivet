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
pub struct CsiControllerInfo {
    #[serde(rename = "SupportsAttachDetach", skip_serializing_if = "Option::is_none")]
    pub supports_attach_detach: Option<bool>,
    #[serde(rename = "SupportsClone", skip_serializing_if = "Option::is_none")]
    pub supports_clone: Option<bool>,
    #[serde(rename = "SupportsCondition", skip_serializing_if = "Option::is_none")]
    pub supports_condition: Option<bool>,
    #[serde(rename = "SupportsCreateDelete", skip_serializing_if = "Option::is_none")]
    pub supports_create_delete: Option<bool>,
    #[serde(rename = "SupportsCreateDeleteSnapshot", skip_serializing_if = "Option::is_none")]
    pub supports_create_delete_snapshot: Option<bool>,
    #[serde(rename = "SupportsExpand", skip_serializing_if = "Option::is_none")]
    pub supports_expand: Option<bool>,
    #[serde(rename = "SupportsGet", skip_serializing_if = "Option::is_none")]
    pub supports_get: Option<bool>,
    #[serde(rename = "SupportsGetCapacity", skip_serializing_if = "Option::is_none")]
    pub supports_get_capacity: Option<bool>,
    #[serde(rename = "SupportsListSnapshots", skip_serializing_if = "Option::is_none")]
    pub supports_list_snapshots: Option<bool>,
    #[serde(rename = "SupportsListVolumes", skip_serializing_if = "Option::is_none")]
    pub supports_list_volumes: Option<bool>,
    #[serde(rename = "SupportsListVolumesAttachedNodes", skip_serializing_if = "Option::is_none")]
    pub supports_list_volumes_attached_nodes: Option<bool>,
    #[serde(rename = "SupportsReadOnlyAttach", skip_serializing_if = "Option::is_none")]
    pub supports_read_only_attach: Option<bool>,
}

impl CsiControllerInfo {
    pub fn new() -> CsiControllerInfo {
        CsiControllerInfo {
            supports_attach_detach: None,
            supports_clone: None,
            supports_condition: None,
            supports_create_delete: None,
            supports_create_delete_snapshot: None,
            supports_expand: None,
            supports_get: None,
            supports_get_capacity: None,
            supports_list_snapshots: None,
            supports_list_volumes: None,
            supports_list_volumes_attached_nodes: None,
            supports_read_only_attach: None,
        }
    }
}

