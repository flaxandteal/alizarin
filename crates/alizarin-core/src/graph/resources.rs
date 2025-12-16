//! Resource types for resource instances and metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::descriptors::StaticResourceDescriptors;
use super::tile::StaticTile;

/// Metadata about a resource instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceMetadata {
    pub descriptors: StaticResourceDescriptors,
    pub graph_id: String,
    pub name: String,
    pub resourceinstanceid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principaluser_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacyid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createdtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmodified: Option<String>,
}

/// Summary info for a resource (used for lazy loading)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceSummary {
    pub resourceinstanceid: String,
    pub graph_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptors: Option<StaticResourceDescriptors>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createdtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmodified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principaluser_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacyid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_publication_id: Option<String>,
}

impl StaticResourceSummary {
    /// Convert summary to metadata
    pub fn to_metadata(&self) -> StaticResourceMetadata {
        StaticResourceMetadata {
            descriptors: self.descriptors.clone().unwrap_or_default(),
            graph_id: self.graph_id.clone(),
            name: self.name.clone(),
            resourceinstanceid: self.resourceinstanceid.clone(),
            publication_id: self.publication_id.clone(),
            principaluser_id: self.principaluser_id,
            legacyid: self.legacyid.clone(),
            graph_publication_id: self.graph_publication_id.clone(),
            createdtime: self.createdtime.clone(),
            lastmodified: self.lastmodified.clone(),
        }
    }
}

/// Reference to another resource instance (for resource-instance datatype)
///
/// Used in ResourceInstanceViewModel to represent relationships between resources.
/// Can include the full resource tree if cascade is enabled.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResourceReference {
    /// Resource instance ID
    pub id: String,
    /// Graph ID for the resource model
    #[serde(rename = "graphId")]
    pub graph_id: String,
    /// Resource model type/name (optional)
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub resource_type: Option<String>,
    /// Display title for the resource (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Full resource tree data (when cascaded, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<serde_json::Value>,
    /// Additional metadata (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

impl StaticResourceReference {
    /// Create a minimal resource reference with just ID and graph ID
    pub fn new(id: String, graph_id: String) -> Self {
        StaticResourceReference {
            id,
            graph_id,
            resource_type: None,
            title: None,
            root: None,
            meta: None,
        }
    }

    /// Create a reference with type information
    pub fn with_type(id: String, graph_id: String, resource_type: String) -> Self {
        StaticResourceReference {
            id,
            graph_id,
            resource_type: Some(resource_type),
            title: None,
            root: None,
            meta: None,
        }
    }

    /// Add title to reference (builder pattern)
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Add metadata to reference (builder pattern)
    pub fn with_meta(mut self, meta: HashMap<String, serde_json::Value>) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Add root data to reference for cascaded loading (builder pattern)
    pub fn with_root(mut self, root: serde_json::Value) -> Self {
        self.root = Some(root);
        self
    }
}

/// Complete resource data with tiles
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticResource {
    pub resourceinstance: StaticResourceMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiles: Option<Vec<StaticTile>>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    // Optional cache and scopes - stored as JSON for platform independence
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "__cache")]
    pub cache: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "__scopes")]
    pub scopes: Option<serde_json::Value>,

    // Tracking flag for lazy loading
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tiles_loaded: Option<bool>,
}
