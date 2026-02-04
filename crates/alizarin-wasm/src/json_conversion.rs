/// Hierarchical tree conversion for resources - WASM wrapper
///
/// Re-exports core json_conversion functionality with WASM-compatible types.
use serde_json::Value;
use crate::graph::StaticGraph;

// Re-export core types
pub use alizarin_core::json_conversion::{BusinessData, create_static_resource};
use alizarin_core::json_conversion::BusinessDataWrapper;

/// Convert tiled resource format to nested tree structure
///
/// Wrapper around core function that accepts WASM StaticGraph wrapper.
///
/// Input: `{"business_data": {"resources": [StaticResource, ...]}}` OR single StaticResource
/// Output: Array of nested JSON tree objects `[{...}, {...}]`
pub fn tiles_to_tree(input: &Value, graph: &StaticGraph) -> Result<Value, String> {
    // StaticGraph Derefs to CoreStaticGraph
    alizarin_core::tiles_to_tree(input, graph)
}

/// Convert nested tree structure to tiled resource format
///
/// Input: Array of nested tree objects `[{...}, {...}]` OR single tree object `{...}`
/// Output: `{"business_data": {"resources": [StaticResource, ...]}}`
///
/// # Arguments
/// * `json` - Tree structure to convert
/// * `graph` - Graph definition
/// * `strict` - If true, fails on unknown fields (default: true)
/// * `id_key` - Optional key for deterministic UUID v5 generation
pub fn tree_to_tiles(
    json: &Value,
    graph: &StaticGraph,
    strict: bool,
    id_key: Option<&str>,
) -> Result<BusinessDataWrapper, String> {
    alizarin_core::tree_to_tiles(json, graph, strict, id_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use alizarin_core::StaticGraph as CoreStaticGraph;

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

    fn create_test_business_data(graph: &StaticGraph) -> Value {
        use alizarin_core::StaticTile;
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

        // Return in business_data format
        serde_json::json!({
            "business_data": {
                "resources": [{
                    "resourceinstance": {
                        "resourceinstanceid": "test-resource-123",
                        "graph_id": graph.graph_id(),
                        "name": "Test Group",
                        "descriptors": {}
                    },
                    "tiles": tiles
                }]
            }
        })
    }

    #[test]
    fn test_tiles_to_tree_basic() {
        let graph = load_group_graph();
        let input = create_test_business_data(&graph);

        let tree = tiles_to_tree(&input, &graph)
            .expect("tiles_to_tree failed");

        assert!(tree.is_array(), "Result should be an array");
        let arr = tree.as_array().unwrap();
        assert!(!arr.is_empty(), "Array should not be empty");
        assert!(arr[0].is_object(), "First element should be an object");
    }

    #[test]
    fn test_tree_to_tiles_basic() {
        let graph = load_group_graph();

        // Note: description is in statement nodegroup, not basic_info
        let tree = serde_json::json!([{
            "basic_info": [{
                "name": {"en": "JSON Test Group", "ga": "Grúpa Tástála JSON"}
            }],
            "statement": [{
                "description": "Created from JSON tree"
            }]
        }]);

        let result = tree_to_tiles(&tree, &graph, true, None)
            .expect("tree_to_tiles failed");

        assert!(!result.business_data.resources.is_empty(), "Should have created resources");
        let resource = &result.business_data.resources[0];
        assert!(resource.tiles.is_some(), "Resource should have tiles");
        assert!(!resource.tiles.as_ref().unwrap().is_empty(), "Should have created tiles");
    }
}
