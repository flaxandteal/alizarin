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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}

impl StaticResourceDescriptors {
    /// Check if all descriptors are empty
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.map_popup.is_none()
            && self.description.is_none()
            && self.slug.is_none()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{IndexedGraph, StaticGraph, StaticTile};

    /// Build a minimal graph via JSON deserialization with descriptor function config
    fn build_test_graph(descriptor_types: Vec<(&str, &str, &str)>) -> IndexedGraph {
        let dt: HashMap<String, serde_json::Value> = descriptor_types
            .into_iter()
            .map(|(dtype, ng_id, template)| {
                (
                    dtype.to_string(),
                    serde_json::json!({
                        "nodegroup_id": ng_id,
                        "string_template": template,
                    }),
                )
            })
            .collect();

        let graph_json = serde_json::json!({
            "graphid": "test-graph",
            "name": {"en": "Test Graph"},
            "root": {
                "nodeid": "root-id",
                "name": "Root",
                "datatype": "semantic",
                "graph_id": "test-graph"
            },
            "nodes": [
                {
                    "nodeid": "root-id",
                    "name": "Root",
                    "datatype": "semantic",
                    "graph_id": "test-graph"
                },
                {
                    "nodeid": "name-node-id",
                    "name": "Name",
                    "alias": "name",
                    "datatype": "string",
                    "nodegroup_id": "name-ng",
                    "graph_id": "test-graph"
                },
                {
                    "nodeid": "slug-node-id",
                    "name": "Slug",
                    "alias": "slug",
                    "datatype": "string",
                    "nodegroup_id": "slug-ng",
                    "graph_id": "test-graph"
                }
            ],
            "nodegroups": [
                { "nodegroupid": "name-ng", "cardinality": "1" },
                { "nodegroupid": "slug-ng", "cardinality": "1" }
            ],
            "edges": [
                { "domainnode_id": "root-id", "rangenode_id": "name-node-id" },
                { "domainnode_id": "root-id", "rangenode_id": "slug-node-id" }
            ],
            "functions_x_graphs": [
                {
                    "config": { "descriptor_types": dt },
                    "function_id": DESCRIPTOR_FUNCTION_ID,
                    "graph_id": "test-graph",
                    "id": "fxg-1"
                }
            ]
        });

        let graph: StaticGraph = serde_json::from_value(graph_json).expect("test graph JSON");
        IndexedGraph::new(graph)
    }

    fn make_tile(nodegroup_id: &str, node_id: &str, value: &str) -> StaticTile {
        let mut tile = StaticTile::new_empty(nodegroup_id.to_string());
        tile.resourceinstance_id = "res-1".to_string();
        tile.tileid = Some("tile-1".to_string());
        tile.data.insert(
            node_id.to_string(),
            serde_json::json!({"en": {"value": value, "direction": "ltr"}}),
        );
        tile
    }

    #[test]
    fn test_build_descriptors_slug() {
        let indexed = build_test_graph(vec![
            ("name", "name-ng", "<Name>"),
            ("slug", "slug-ng", "<Slug>"),
        ]);

        let tiles = vec![
            make_tile("name-ng", "name-node-id", "My Resource"),
            make_tile("slug-ng", "slug-node-id", "My Resource"),
        ];

        let descriptors = indexed.build_descriptors(&tiles);

        assert_eq!(descriptors.name.as_deref(), Some("My Resource"));
        assert_eq!(descriptors.slug.as_deref(), Some("my-resource"));
        assert_eq!(descriptors.description, None);
        assert_eq!(descriptors.map_popup, None);
    }

    #[test]
    fn test_build_descriptors_slug_absent_is_none() {
        let indexed = build_test_graph(vec![("name", "name-ng", "<Name>")]);

        let tiles = vec![make_tile("name-ng", "name-node-id", "My Resource")];

        let descriptors = indexed.build_descriptors(&tiles);

        assert_eq!(descriptors.name.as_deref(), Some("My Resource"));
        assert!(descriptors.slug.is_none());
    }

    #[test]
    fn test_descriptors_is_empty() {
        let d = StaticResourceDescriptors::default();
        assert!(d.is_empty());

        let d = StaticResourceDescriptors {
            slug: Some("test".to_string()),
            ..Default::default()
        };
        assert!(!d.is_empty());
    }

    #[test]
    fn test_descriptors_serde_roundtrip_with_slug() {
        let d = StaticResourceDescriptors {
            name: Some("Test".to_string()),
            slug: Some("test-slug".to_string()),
            description: None,
            map_popup: None,
        };
        let json = serde_json::to_string(&d).unwrap();
        let parsed: StaticResourceDescriptors = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.slug.as_deref(), Some("test-slug"));
        assert_eq!(parsed.name.as_deref(), Some("Test"));
    }

    #[test]
    fn test_descriptors_deserialize_without_slug_is_backwards_compatible() {
        let json = r#"{"name": "Test"}"#;
        let parsed: StaticResourceDescriptors = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.name.as_deref(), Some("Test"));
        assert!(parsed.slug.is_none());
    }
}
