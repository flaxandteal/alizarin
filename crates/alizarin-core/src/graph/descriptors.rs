//! Resource descriptor types and configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Descriptor function UUID (from Arches)
pub const DESCRIPTOR_FUNCTION_ID: &str = "60000000-0000-0000-0000-000000000001";

/// Descriptors for resource display
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StaticResourceDescriptors {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_popup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl StaticResourceDescriptors {
    /// Check if all descriptors are empty
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.map_popup.is_none() && self.description.is_none()
    }

    /// Create empty descriptors
    pub fn empty() -> Self {
        Self::default()
    }
}

/// Configuration for a single descriptor type (name, description, map_popup)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DescriptorTypeConfig {
    pub nodegroup_id: String,
    pub string_template: String,
}

/// Complete descriptor configuration from functions_x_graphs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DescriptorConfig {
    pub descriptor_types: HashMap<String, DescriptorTypeConfig>,
}
