//! Node, Nodegroup, and Edge types for the graph structure.

use super::translatable::StaticTranslatableString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the graph representing a data field or structural element
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticNode {
    pub nodeid: String,
    pub name: String,
    #[serde(default)]
    pub alias: Option<String>,
    pub datatype: String,
    #[serde(default)]
    pub nodegroup_id: Option<String>,
    pub graph_id: String,
    #[serde(default)]
    pub is_collector: bool,
    #[serde(default)]
    pub isrequired: bool,
    #[serde(default)]
    pub exportable: bool,
    #[serde(default)]
    pub sortorder: Option<i32>,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub parentproperty: Option<String>,
    /// Ontology class URIs for this node. Accepts a single string or an array
    /// of strings on the wire; a single-element list is serialised as a plain
    /// string for round-trip compatibility with upstream Arches.
    #[serde(default, with = "super::serde_helpers::optional_string_or_vec")]
    pub ontologyclass: Option<Vec<String>>,
    #[serde(default)]
    pub description: Option<StaticTranslatableString>,
    #[serde(default)]
    pub fieldname: Option<String>,
    #[serde(default)]
    pub hascustomalias: bool,
    #[serde(default)]
    pub issearchable: bool,
    #[serde(default)]
    pub istopnode: bool,
    #[serde(default)]
    pub sourcebranchpublication_id: Option<String>,
    // Arches-HER 2.0+ fields
    /// Source identifier for import/export tracking
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_identifier_id: Option<String>,
    /// Whether node is immutable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_immutable: Option<bool>,
}

impl StaticNode {
    /// Check if this node is the root node (no nodegroup_id)
    pub fn is_root(&self) -> bool {
        self.nodegroup_id.is_none()
            || self
                .nodegroup_id
                .as_ref()
                .map(|s| s.is_empty())
                .unwrap_or(true)
    }

    /// Get the display name
    pub fn display_name(&self) -> &str {
        &self.name
    }

    /// Get the alias or empty string
    pub fn display_alias(&self) -> &str {
        self.alias.as_deref().unwrap_or("")
    }

    /// Serialize to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

/// A nodegroup defining cardinality and grouping of nodes
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticNodegroup {
    pub nodegroupid: String,
    #[serde(default)]
    pub cardinality: Option<String>,
    #[serde(default)]
    pub parentnodegroup_id: Option<String>,
    #[serde(default)]
    pub legacygroupid: Option<String>,
    // Arches-HER 2.0+ fields
    /// Reference to the grouping/semantic node for this nodegroup
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grouping_node_id: Option<String>,
}

impl StaticNodegroup {
    /// Serialize to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

/// An edge connecting two nodes (domain -> range)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticEdge {
    pub domainnode_id: String,
    pub rangenode_id: String,
    #[serde(default)]
    pub edgeid: String,
    #[serde(default)]
    pub graph_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub ontologyproperty: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    // Arches-HER 2.0+ fields
    /// Source identifier for import/export tracking
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_identifier_id: Option<String>,
}

impl StaticEdge {
    /// Serialize to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_is_root() {
        let node = StaticNode {
            nodeid: "123".to_string(),
            name: "Root".to_string(),
            alias: None,
            datatype: "semantic".to_string(),
            nodegroup_id: None,
            graph_id: "graph1".to_string(),
            is_collector: false,
            isrequired: false,
            exportable: true,
            sortorder: None,
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: None,
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: false,
            istopnode: true,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        };
        assert!(node.is_root());
    }
}
