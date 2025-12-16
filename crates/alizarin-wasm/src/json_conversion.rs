/// Hierarchical tree conversion for resources - WASM wrapper
///
/// Re-exports core json_conversion functionality with WASM-compatible types.

use serde_json::Value;
use crate::graph::StaticGraph;

// Re-export core types
pub use alizarin_core::json_conversion::ResourceData;

/// Convert tiled resource format to nested tree structure
///
/// Wrapper around core function that accepts WASM StaticGraph wrapper.
pub fn tiles_to_tree(resource: &ResourceData, graph: &StaticGraph) -> Result<Value, String> {
    // StaticGraph Derefs to CoreStaticGraph
    alizarin_core::tiles_to_tree(resource, &**graph)
}

/// Convert nested tree structure to tiled resource format
pub fn tree_to_tiles(
    json: &Value,
    graph: &StaticGraph,
    resource_id: &str,
    graph_id: &str,
) -> Result<ResourceData, String> {
    alizarin_core::tree_to_tiles(json, &**graph, resource_id, graph_id)
}

/// Convert with strict validation (fails on unknown fields)
pub fn tree_to_tiles_strict(
    json: &Value,
    graph: &StaticGraph,
    resource_id: &str,
    graph_id: &str,
) -> Result<ResourceData, String> {
    alizarin_core::tree_to_tiles_strict(json, &**graph, resource_id, graph_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use alizarin_core::StaticGraph as CoreStaticGraph;
    use alizarin_core::StaticTile;

    fn load_group_graph() -> StaticGraph {
        // Use workspace root for test data
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap()
            .to_path_buf();
        let test_file = workspace_root.join("tests/data/models/Group.json");
        let json_str = fs::read_to_string(&test_file)
            .expect("Failed to read Group.json");
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .expect("Failed to parse Group.json");

        let graph_data = json["graph"][0].clone();

        let mut core_graph: CoreStaticGraph = serde_json::from_value(graph_data)
            .expect("Failed to deserialize StaticGraph");
        core_graph.build_indices();

        StaticGraph::from(core_graph)
    }

    fn create_test_resource(graph: &StaticGraph) -> ResourceData {
        let mut tiles = Vec::new();

        let basic_info_ng = graph.nodegroups_slice()
            .iter()
            .find(|ng| {
                graph.nodes_slice()
                    .iter()
                    .any(|n| n.alias.as_deref() == Some("basic_info")
                          && n.nodegroup_id.as_ref() == Some(&ng.nodegroupid))
            })
            .expect("Could not find basic_info nodegroup");

        let mut tile = StaticTile::new_empty(basic_info_ng.nodegroupid.clone());
        tile.resourceinstance_id = "test-resource-123".to_string();

        if let Some(name_node) = graph.nodes_slice()
            .iter()
            .find(|n| n.alias.as_deref() == Some("name")) {
            tile.data.insert(
                name_node.nodeid.clone(),
                serde_json::json!({"en": "Test Group", "ga": "Grúpa Tástála"})
            );
        }

        if let Some(desc_node) = graph.nodes_slice()
            .iter()
            .find(|n| n.alias.as_deref() == Some("description")) {
            tile.data.insert(
                desc_node.nodeid.clone(),
                serde_json::json!("A test group for unit testing")
            );
        }

        tiles.push(tile);

        ResourceData {
            resourceinstanceid: "test-resource-123".to_string(),
            graph_id: graph.graph_id().to_string(),
            tiles,
        }
    }

    #[test]
    fn test_tiles_to_tree_basic() {
        let graph = load_group_graph();
        let resource = create_test_resource(&graph);

        let tree = tiles_to_tree(&resource, &graph)
            .expect("tiles_to_tree failed");

        assert!(tree.is_object(), "Result should be an object");
        assert!(tree.get("resourceinstanceid").is_none(), "Should not include resourceinstanceid");
        assert!(tree.get("graph_id").is_none(), "Should not include graph_id");
    }

    #[test]
    fn test_tree_to_tiles_basic() {
        let graph = load_group_graph();

        let tree = serde_json::json!({
            "basic_info": [{
                "name": {"en": "JSON Test Group", "ga": "Grúpa Tástála JSON"},
                "description": "Created from JSON tree"
            }]
        });

        let resource_id = "test-resource-456";
        let graph_id = graph.graph_id();

        let resource = tree_to_tiles(&tree, &graph, resource_id, &graph_id)
            .expect("tree_to_tiles failed");

        assert_eq!(resource.resourceinstanceid, "test-resource-456");
        assert!(!resource.tiles.is_empty(), "Should have created tiles");
    }
}
