//! Lightweight graph metadata type.

use super::nodes::StaticNode;
use super::translatable::StaticTranslatableString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lightweight metadata about a graph, without the full nodes/edges arrays.
/// Used for listing graphs without loading all their data.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticGraphMeta {
    pub graphid: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub cards: Option<u32>,
    #[serde(default)]
    pub cards_x_nodes_x_widgets: Option<u32>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub description: Option<StaticTranslatableString>,
    #[serde(default)]
    pub edges: Option<u32>,
    #[serde(default)]
    pub iconclass: Option<String>,
    #[serde(default)]
    pub is_editable: Option<bool>,
    #[serde(default)]
    pub isresource: Option<bool>,
    /// JSON-LD context - can be a string (URL) or an object (inline context)
    #[serde(default)]
    pub jsonldcontext: Option<serde_json::Value>,
    #[serde(default)]
    pub name: Option<StaticTranslatableString>,
    #[serde(default)]
    pub nodegroups: Option<u32>,
    #[serde(default)]
    pub nodes: Option<u32>,
    #[serde(default)]
    pub ontology_id: Option<String>,
    #[serde(default)]
    pub publication: Option<HashMap<String, Option<String>>>,
    #[serde(default)]
    pub relatable_resource_model_ids: Vec<String>,
    #[serde(default)]
    pub resource_2_resource_constraints: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub root: Option<Box<StaticNode>>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub subtitle: Option<StaticTranslatableString>,
    #[serde(default)]
    pub version: Option<String>,
    /// Extra fields not explicitly defined
    #[serde(default, flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

impl StaticGraphMeta {
    /// Get the display name of the graph
    pub fn display_name(&self) -> String {
        self.name
            .as_ref()
            .map(|n| n.to_string_default())
            .unwrap_or_default()
    }

    /// Get the display subtitle
    pub fn display_subtitle(&self) -> String {
        self.subtitle
            .as_ref()
            .map(|s| s.to_string_default())
            .unwrap_or_default()
    }

    /// Get the display description
    pub fn display_description(&self) -> String {
        self.description
            .as_ref()
            .map(|d| d.to_string_default())
            .unwrap_or_default()
    }

    /// Get the author
    pub fn display_author(&self) -> String {
        self.author.clone().unwrap_or_default()
    }
}
