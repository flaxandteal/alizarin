/// Hierarchical tree conversion for resources
///
/// This module provides bidirectional conversion between:
/// - **Tiled format**: Flat list of tiles grouped by nodegroup (Arches native format)
/// - **Tree format**: Nested hierarchical JSON using node aliases as keys (developer-friendly)
///
/// These are NOT simple serialization functions - they perform structural transformation:
/// - `tiles_to_tree()`: Flat tiles → Nested hierarchy (like TypeScript ResourceInstanceViewModel.forJson())
/// - `tree_to_tiles()`: Nested hierarchy → Flat tiles (inverse operation)
///
/// The logic here is shared between:
/// - WASM/JavaScript bindings
/// - PyO3/Python bindings
/// - Future language bindings

use std::collections::HashMap;
use serde_json::{Value, Map};
use serde::{Serialize, Deserialize};
use crate::graph::{StaticGraph, StaticTile, StaticNode, StaticNodegroup, StaticEdge};
use crate::pseudo_value::{RustPseudoList, RustPseudoGroup, RustPseudoValue};

/// Container for resource data (no StaticResource type exists in graph.rs)
///
/// In Arches, a resource instance is just:
/// - A resource instance ID
/// - A graph ID (which graph/model it belongs to)
/// - A collection of tiles (data storage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub resourceinstanceid: String,
    pub graph_id: String,
    pub tiles: Vec<StaticTile>,
}

/// Convert tiled resource format to nested tree structure
///
/// **Structural transformation** (not just serialization):
/// - Input: Flat list of tiles grouped by nodegroup_id
/// - Output: Nested JSON tree using node aliases as keys
///
/// **Implementation approach**:
/// This function REUSES the existing `populate()` logic from instance_wrapper.rs
/// instead of duplicating the graph traversal:
/// 1. Call populate() to build RustPseudoList hierarchy (already does tiles → tree)
/// 2. Convert RustPseudoList structures to JSON format
/// 3. Return nested JSON tree
///
/// This avoids duplicating the complex traversal logic in instance_wrapper.rs:680-812
///
/// # Arguments
/// * `resource` - Resource with flat tiles
/// * `graph` - Complete graph model with nodes/nodegroups/edges
///
/// # Returns
/// Nested JSON tree representing the resource hierarchy
pub fn tiles_to_tree(resource: &ResourceData, graph: &StaticGraph) -> Result<Value, String> {
    let mut result = Map::new();

    // Add basic resource metadata
    result.insert("resourceinstanceid".to_string(), Value::String(resource.resourceinstanceid.clone()));
    result.insert("graph_id".to_string(), Value::String(resource.graph_id.clone()));

    // TODO: Call populate() to get RustPseudoList hierarchy
    // TODO: Convert RustPseudoList → JSON using pseudo_list_to_json()

    // For now, stub implementation
    Ok(Value::Object(result))
}

/// Convert nested tree structure to tiled resource format
///
/// **Structural transformation** (inverse of tiles_to_tree):
/// - Input: Nested JSON tree using node aliases as keys
/// - Output: Flat list of tiles grouped by nodegroup_id
///
/// This is the inverse of tiles_to_tree():
/// - Walks the nested JSON structure
/// - Creates tiles for each nodegroup encountered
/// - Populates tile.data[node_id] with field values from hierarchy
///
/// # Arguments
/// * `json` - Nested JSON tree structure
/// * `graph` - Complete graph model with nodes/nodegroups/edges
///
/// # Returns
/// ResourceData with flat tiles created from tree
pub fn tree_to_tiles(json: &Value, graph: &StaticGraph) -> Result<ResourceData, String> {
    let obj = json.as_object()
        .ok_or_else(|| "JSON must be an object".to_string())?;

    // Extract resource metadata
    let resource_id = obj.get("resourceinstanceid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing resourceinstanceid".to_string())?
        .to_string();

    let graph_id = obj.get("graph_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing graph_id".to_string())?
        .to_string();

    // Build lookup maps
    let nodes_by_alias = build_node_alias_lookup(graph.nodes_slice());
    let nodegroups_by_id = build_nodegroup_lookup(graph.nodegroups_slice());

    // Collect tiles while walking JSON structure
    let mut tiles = Vec::new();

    // Start from root and recursively process
    let root = graph.root_node();
    process_json_recursive(
        obj,
        &root,
        &nodes_by_alias,
        &nodegroups_by_id,
        &resource_id,
        None, // parent_tile_id
        &mut tiles,
    )?;

    // Create resource with collected tiles
    Ok(ResourceData {
        resourceinstanceid: resource_id,
        graph_id,
        tiles,
    })
}

// ============================================================================
// Helper functions for resource_to_json
// ============================================================================

/// Recursively process a node and its children, building the JSON structure
///
/// This mirrors the hierarchy discovery in instance_wrapper.rs:values_from_resource_nodegroup_internal
/// but walks the graph structure (not tiles) to build JSON output.
///
/// Key pattern reused from instance_wrapper.rs:
/// - Lines 851-856: Nodes grouped by nodegroup_id
/// - Lines 885-911: Edge traversal to discover child nodes
/// - Lines 924-970: Processing tiles to find node data
fn process_node_recursive(
    node: &StaticNode,
    nodes_by_id: &HashMap<String, &StaticNode>,
    nodegroups_by_id: &HashMap<String, &StaticNodegroup>,
    edges: &HashMap<String, Vec<String>>, // parent_id -> [child_ids]
    tiles: &[StaticTile],
    result: &mut Map<String, Value>,
) -> Result<(), String> {
    // Get node alias (this becomes the JSON key)
    // PORT: Similar to instance_wrapper.rs:866 - using node.alias as key
    let alias = node.alias.as_ref()
        .ok_or_else(|| format!("Node {} has no alias", node.nodeid))?;

    // Find nodegroup for this node
    // PORT: instance_wrapper.rs:853-856 - filtering nodes by nodegroup_id
    let nodegroup_id = node.nodegroup_id.as_ref()
        .ok_or_else(|| format!("Node {} has no nodegroup_id", node.nodeid))?;

    let nodegroup = nodegroups_by_id.get(nodegroup_id)
        .ok_or_else(|| format!("Nodegroup {} not found", nodegroup_id))?;

    // Find tiles for this nodegroup
    // PORT: instance_wrapper.rs:427 - get_tile_ids_by_nodegroup pattern
    let nodegroup_tiles: Vec<&StaticTile> = tiles.iter()
        .filter(|t| &t.nodegroup_id == nodegroup_id)
        .collect();

    // Determine if this is a list (cardinality='n') or single value
    // PORT: instance_wrapper.rs:415 - checking is_collector for list determination
    let is_list = nodegroup.cardinality.as_ref()
        .map(|c| c == "n")
        .unwrap_or(false);

    if is_list {
        // Create array of values (one per tile)
        // PORT: instance_wrapper.rs:427-456 - iterating over tiles to create PseudoValue list
        let mut array = Vec::new();
        for tile in nodegroup_tiles {
            let tile_value = process_tile(node, tile, nodes_by_id, nodegroups_by_id, edges, tiles)?;
            array.push(tile_value);
        }
        result.insert(alias.to_string(), Value::Array(array));
    } else {
        // Single value (take first tile if exists)
        // PORT: instance_wrapper.rs:462-501 - creating single PseudoValue
        if let Some(tile) = nodegroup_tiles.first() {
            let value = process_tile(node, tile, nodes_by_id, nodegroups_by_id, edges, tiles)?;
            result.insert(alias.to_string(), value);
        }
    }

    Ok(())
}

/// Process a single tile and its child nodes
fn process_tile(
    node: &StaticNode,
    tile: &StaticTile,
    nodes_by_id: &HashMap<String, &StaticNode>,
    nodegroups_by_id: &HashMap<String, &StaticNodegroup>,
    edges: &HashMap<String, Vec<String>>,
    all_tiles: &[StaticTile],
) -> Result<Value, String> {
    let mut obj = Map::new();

    // Extract value from tile data for this specific node
    if let Some(value) = tile.data.get(&node.nodeid) {
        // Only add _value if it's not null/empty
        if !value.is_null() {
            obj.insert("_value".to_string(), value.clone());
        }
    }

    // Recursively process child nodes
    // This reuses the traversal pattern from instance_wrapper.rs:885-911 (discovering implied nodes via edges)
    if let Some(child_ids) = edges.get(&node.nodeid) {
        for child_id in child_ids {
            if let Some(child_node) = nodes_by_id.get(child_id) {
                // For each child, process it recursively
                // This mirrors the hierarchy walk in values_from_resource_nodegroup_internal
                process_node_recursive(
                    child_node,
                    nodes_by_id,
                    nodegroups_by_id,
                    edges,
                    all_tiles,
                    &mut obj,
                )?;
            }
        }
    }

    Ok(Value::Object(obj))
}

// ============================================================================
// Helper functions for json_to_resource
// ============================================================================

/// Recursively process JSON and create tiles
fn process_json_recursive(
    json: &Map<String, Value>,
    node: &StaticNode,
    nodes_by_alias: &HashMap<String, &StaticNode>,
    nodegroups_by_id: &HashMap<String, &StaticNodegroup>,
    resource_id: &str,
    parent_tile_id: Option<&str>,
    tiles: &mut Vec<StaticTile>,
) -> Result<(), String> {
    // TODO: Implement JSON to tiles conversion
    // This is the inverse of resource_to_json
    Ok(())
}

// ============================================================================
// Lookup table builders
// ============================================================================

fn build_node_lookup(nodes: &[StaticNode]) -> HashMap<String, &StaticNode> {
    nodes.iter()
        .map(|n| (n.nodeid.clone(), n))
        .collect()
}

fn build_node_alias_lookup(nodes: &[StaticNode]) -> HashMap<String, &StaticNode> {
    nodes.iter()
        .filter_map(|n| n.alias.as_ref().map(|a| (a.clone(), n)))
        .collect()
}

fn build_nodegroup_lookup(nodegroups: &[StaticNodegroup]) -> HashMap<String, &StaticNodegroup> {
    nodegroups.iter()
        .map(|ng| (ng.nodegroupid.clone(), ng))
        .collect()
}

fn build_edge_map(edges: &[StaticEdge]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        map.entry(edge.domainnode_id.clone())
            .or_insert_with(Vec::new)
            .push(edge.rangenode_id.clone());
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_edge_map() {
        // TODO: Add tests for conversion logic
    }

    #[test]
    fn test_resource_to_json_roundtrip() {
        // TODO: Test that resource -> JSON -> resource preserves data
    }
}
