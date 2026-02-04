/// Tests for PseudoListCore::matching_entries
///
/// This tests the fix for the registry names issue where inner values (like name_use_type)
/// were not being filtered by their parent tile context. The outer/inner pattern requires
/// that when iterating over outer values (e.g., names[0], names[1]), the inner values
/// must be filtered to only those belonging to that specific outer tile.
///
/// Example structure (from Registry graph):
/// ```
/// registry.names[0]           <- outer tile "tile-a"
///   ├── name: "Primary Name"
///   └── name_use_type: "Primary"
/// registry.names[1]           <- outer tile "tile-b"
///   ├── name: "Alternative"
///   └── name_use_type: "Alternative"
/// ```
///
/// Without proper filtering, accessing names[0].name_use_type might return
/// both "Primary" AND "Alternative", causing "is_single=true but has 2 values" errors.

use alizarin_core::{PseudoListCore, PseudoValueCore, StaticNode, StaticTile};
use std::sync::Arc;
use serde_json::json;

/// Create a test node
fn create_node(nodeid: &str, alias: &str, nodegroup_id: &str) -> Arc<StaticNode> {
    let node_json = json!({
        "alias": alias,
        "datatype": "string",
        "graph_id": "test-graph",
        "is_collector": false,
        "isrequired": false,
        "issearchable": false,
        "istopnode": false,
        "name": alias,
        "nodegroup_id": nodegroup_id,
        "nodeid": nodeid,
        "sortorder": 0
    });
    Arc::new(serde_json::from_value(node_json).expect("Failed to create test node"))
}

/// Create a test tile
fn create_tile(tileid: &str, nodegroup_id: &str, parenttile_id: Option<&str>) -> Arc<StaticTile> {
    let tile_json = json!({
        "data": {},
        "nodegroup_id": nodegroup_id,
        "resourceinstance_id": "test-resource",
        "tileid": tileid,
        "parenttile_id": parenttile_id
    });
    Arc::new(serde_json::from_value(tile_json).expect("Failed to create test tile"))
}

/// Create a PseudoValueCore with the specified tile
fn create_pseudo_value(node: Arc<StaticNode>, tile: Arc<StaticTile>) -> PseudoValueCore {
    PseudoValueCore::from_node_and_tile(
        node,
        Some(tile),
        Some(json!("test-value")),
        vec![],
    )
}

/// Test: matching_entries filters by tile context
///
/// Scenario: Two outer tiles (like names[0] and names[1]) each have inner values
/// (like name_use_type). When querying inner values for a specific outer tile,
/// only values from that tile should be returned.
#[test]
fn test_matching_entries_filters_by_parent_tile() {
    let nodegroup_id = "ng-names";

    // Create two outer tiles (like names[0] and names[1])
    let outer_tile_a = create_tile("tile-outer-a", nodegroup_id, None);
    let outer_tile_b = create_tile("tile-outer-b", nodegroup_id, None);

    // Create inner values - each belongs to its respective outer tile
    let inner_node = create_node("node-name-use-type", "name_use_type", nodegroup_id);

    // Value A belongs to tile A (same tile = inner value on outer tile)
    let value_a = create_pseudo_value(inner_node.clone(), outer_tile_a.clone());
    // Value B belongs to tile B
    let value_b = create_pseudo_value(inner_node.clone(), outer_tile_b.clone());

    // Create a PseudoList with both values
    let list = PseudoListCore::from_values_with_cardinality(
        "name_use_type".to_string(),
        vec![value_a, value_b],
        true, // is_single - this is a single value per outer tile
    );

    // Query for tile A - should only get value from tile A
    let matches_a = list.matching_entries(
        Some("tile-outer-a".to_string()),
        Some(nodegroup_id.to_string()),
        None, // parent_nodegroup_id
    );
    assert_eq!(matches_a.len(), 1, "Should only match one value for tile A");
    assert_eq!(
        matches_a[0].tile.as_ref().unwrap().tileid,
        Some("tile-outer-a".to_string()),
        "Should match value from tile A"
    );

    // Query for tile B - should only get value from tile B
    let matches_b = list.matching_entries(
        Some("tile-outer-b".to_string()),
        Some(nodegroup_id.to_string()),
        None,
    );
    assert_eq!(matches_b.len(), 1, "Should only match one value for tile B");
    assert_eq!(
        matches_b[0].tile.as_ref().unwrap().tileid,
        Some("tile-outer-b".to_string()),
        "Should match value from tile B"
    );
}

/// Test: matching_entries handles child tiles (different nodegroups)
///
/// When inner values are in child tiles (tiles with parenttile_id pointing to outer tile),
/// they should still be matched correctly.
#[test]
fn test_matching_entries_handles_child_tiles() {
    let outer_nodegroup = "ng-outer";
    let inner_nodegroup = "ng-inner";

    // Create outer tile
    let _outer_tile = create_tile("tile-outer", outer_nodegroup, None);

    // Create child tiles that point to the outer tile
    let child_tile_a = create_tile("tile-child-a", inner_nodegroup, Some("tile-outer"));
    let child_tile_b = create_tile("tile-child-b", inner_nodegroup, Some("tile-outer"));
    let unrelated_tile = create_tile("tile-unrelated", inner_nodegroup, Some("tile-other-parent"));

    let inner_node = create_node("node-inner", "inner_value", inner_nodegroup);

    let value_a = create_pseudo_value(inner_node.clone(), child_tile_a.clone());
    let value_b = create_pseudo_value(inner_node.clone(), child_tile_b.clone());
    let value_unrelated = create_pseudo_value(inner_node.clone(), unrelated_tile.clone());

    let list = PseudoListCore::from_values_with_cardinality(
        "inner_value".to_string(),
        vec![value_a, value_b, value_unrelated],
        false, // is_single = false, can have multiple
    );

    // Query for outer tile - should get child tiles A and B, not unrelated
    let matches = list.matching_entries(
        Some("tile-outer".to_string()),
        Some(inner_nodegroup.to_string()),
        Some(outer_nodegroup.to_string()), // parent_nodegroup_id
    );

    assert_eq!(matches.len(), 2, "Should match both child tiles pointing to outer tile");

    // Verify we got the right tiles
    let tile_ids: Vec<_> = matches.iter()
        .map(|m| m.tile.as_ref().unwrap().tileid.clone())
        .collect();
    assert!(tile_ids.contains(&Some("tile-child-a".to_string())));
    assert!(tile_ids.contains(&Some("tile-child-b".to_string())));
    assert!(!tile_ids.contains(&Some("tile-unrelated".to_string())));
}

/// Test: matching_entries handles root level (no parent tile)
///
/// When querying at the root level (no parent tile), only tiles without
/// a parenttile_id should match.
#[test]
fn test_matching_entries_handles_root_level() {
    let nodegroup_id = "ng-root";

    // Create root-level tiles (no parenttile_id)
    let root_tile_a = create_tile("tile-root-a", nodegroup_id, None);
    let root_tile_b = create_tile("tile-root-b", nodegroup_id, None);

    // Create a child tile (has parenttile_id)
    let child_tile = create_tile("tile-child", nodegroup_id, Some("tile-some-parent"));

    let node = create_node("node-root", "root_value", nodegroup_id);

    let value_root_a = create_pseudo_value(node.clone(), root_tile_a.clone());
    let value_root_b = create_pseudo_value(node.clone(), root_tile_b.clone());
    let value_child = create_pseudo_value(node.clone(), child_tile.clone());

    let list = PseudoListCore::from_values_with_cardinality(
        "root_value".to_string(),
        vec![value_root_a, value_root_b, value_child],
        false,
    );

    // Query at root level (no parent tile) - should get only root tiles
    let matches = list.matching_entries(
        None, // No parent tile = root query
        Some(nodegroup_id.to_string()),
        None,
    );

    assert_eq!(matches.len(), 2, "Should match only root-level tiles");

    let tile_ids: Vec<_> = matches.iter()
        .map(|m| m.tile.as_ref().unwrap().tileid.clone())
        .collect();
    assert!(tile_ids.contains(&Some("tile-root-a".to_string())));
    assert!(tile_ids.contains(&Some("tile-root-b".to_string())));
    assert!(!tile_ids.contains(&Some("tile-child".to_string())));
}

/// Test: matching_entries requires nodegroup match
///
/// Values should only be returned if they're in the requested nodegroup.
#[test]
fn test_matching_entries_requires_nodegroup_match() {
    let nodegroup_a = "ng-a";
    let nodegroup_b = "ng-b";

    // Create tiles in different nodegroups
    let tile_a = create_tile("tile-a", nodegroup_a, None);
    let tile_b = create_tile("tile-b", nodegroup_b, None);

    let node_a = create_node("node-a", "value_a", nodegroup_a);
    let node_b = create_node("node-b", "value_b", nodegroup_b);

    let value_a = create_pseudo_value(node_a, tile_a);
    let value_b = create_pseudo_value(node_b, tile_b);

    let list = PseudoListCore::from_values_with_cardinality(
        "mixed_values".to_string(),
        vec![value_a, value_b],
        false,
    );

    // Query for nodegroup A - should only get value from nodegroup A
    let matches = list.matching_entries(
        None,
        Some(nodegroup_a.to_string()),
        None,
    );

    assert_eq!(matches.len(), 1, "Should only match values in requested nodegroup");
    assert_eq!(
        matches[0].tile.as_ref().unwrap().nodegroup_id,
        nodegroup_a,
        "Should match value from nodegroup A"
    );
}

/// Regression test: Original registry names bug
///
/// This simulates the exact scenario that was failing:
/// - Registry has multiple names (outer loop)
/// - Each name has a name_use_type (inner value, is_single=true)
/// - Without proper filtering, ALL name_use_type values were returned for each name
#[test]
fn test_registry_names_regression() {
    let names_nodegroup = "ng-names";

    // Create two name tiles (simulating names[0] and names[1])
    let name_tile_0 = create_tile("name-tile-0", names_nodegroup, None);
    let name_tile_1 = create_tile("name-tile-1", names_nodegroup, None);

    // Each name tile has its own name_use_type value
    let name_use_type_node = create_node("node-name-use-type", "name_use_type", names_nodegroup);

    // Value for names[0] - "Primary"
    let mut value_0 = create_pseudo_value(name_use_type_node.clone(), name_tile_0.clone());
    value_0.tile_data = Some(json!("Primary"));

    // Value for names[1] - "Alternative"
    let mut value_1 = create_pseudo_value(name_use_type_node.clone(), name_tile_1.clone());
    value_1.tile_data = Some(json!("Alternative"));

    // Create the PseudoList with is_single=true (cardinality-1)
    let list = PseudoListCore::from_values_with_cardinality(
        "name_use_type".to_string(),
        vec![value_0, value_1],
        true, // CRITICAL: is_single=true means we expect exactly 0 or 1 value per context
    );

    // Query for names[0]'s tile - should get exactly 1 value
    let matches_0 = list.matching_entries(
        Some("name-tile-0".to_string()),
        Some(names_nodegroup.to_string()),
        None,
    );

    assert_eq!(
        matches_0.len(), 1,
        "REGRESSION: names[0].name_use_type should return exactly 1 value, not {}",
        matches_0.len()
    );
    assert_eq!(
        matches_0[0].tile_data.as_ref().map(|v| v.as_str()),
        Some(Some("Primary")),
        "names[0].name_use_type should be 'Primary'"
    );

    // Query for names[1]'s tile - should get exactly 1 value
    let matches_1 = list.matching_entries(
        Some("name-tile-1".to_string()),
        Some(names_nodegroup.to_string()),
        None,
    );

    assert_eq!(
        matches_1.len(), 1,
        "REGRESSION: names[1].name_use_type should return exactly 1 value, not {}",
        matches_1.len()
    );
    assert_eq!(
        matches_1[0].tile_data.as_ref().map(|v| v.as_str()),
        Some(Some("Alternative")),
        "names[1].name_use_type should be 'Alternative'"
    );
}
