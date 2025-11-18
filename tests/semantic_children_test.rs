/// Tests for findSemanticChildren implementation
/// Ensures Rust logic exactly matches TypeScript semantic.ts lines 269-340
use alizarin::graph::{StaticTile, StaticNode};
use alizarin::instance_wrapper::WASMResourceInstanceWrapper;
use std::collections::HashMap;
use serde_json::Value as JsonValue;
use serde_json::json;

/// Helper to create a test node
fn create_test_node(
    nodeid: &str,
    alias: &str,
    nodegroup_id: Option<String>,
    is_collector: bool,
) -> StaticNode {
    let node_json = json!({
        "alias": alias,
        "datatype": "string",
        "exportable": false,
        "graph_id": "test-graph",
        "hascustomalias": false,
        "is_collector": is_collector,
        "isrequired": false,
        "issearchable": false,
        "istopnode": false,
        "name": alias,
        "nodegroup_id": nodegroup_id,
        "nodeid": nodeid,
        "sortorder": 0
    });
    serde_json::from_value(node_json).expect("Failed to create test node")
}

/// Helper to create a test tile
fn create_test_tile(
    tileid: &str,
    nodegroup_id: &str,
    parenttile_id: Option<String>,
    node_data: Vec<(&str, JsonValue)>,
) -> StaticTile {
    let mut data = HashMap::new();
    for (nodeid, value) in node_data {
        data.insert(nodeid.to_string(), value);
    }

    let tile_json = json!({
        "data": data,
        "nodegroup_id": nodegroup_id,
        "resourceinstance_id": "test-resource",
        "tileid": tileid,
        "parenttile_id": parenttile_id
    });
    serde_json::from_value(tile_json).expect("Failed to create test tile")
}

/// Wrapper function to call the actual implementation
/// Uses WASMResourceInstanceWrapper::matches_semantic_child which has access to private fields
fn matches_semantic_child(
    parent_tile_id: Option<&String>,
    parent_nodegroup_id: Option<&String>,
    child_node: &StaticNode,
    tile: Option<&StaticTile>,
    tile_nodegroup: &str,
) -> bool {
    WASMResourceInstanceWrapper::matches_semantic_child(
        parent_tile_id,
        parent_nodegroup_id,
        child_node,
        tile,
        tile_nodegroup,
    )
}

/// Test Branch 1: Different nodegroup + null parenttile_id
/// PORT: js/semantic.ts line 311 - (!(value.tile.parenttile_id) || ...)
#[test]
fn test_different_nodegroup_null_parent_tile() {
    // Parent is in nodegroup "ng-parent"
    let parent_tile_id = Some("tile-parent".to_string());
    let parent_nodegroup_id = Some("ng-parent".to_string());

    // Child is in different nodegroup "ng-child" with null parenttile_id
    let child_node = create_test_node("node-child", "child_alias", Some("ng-child".to_string()), false);
    let child_tile = create_test_tile(
        "tile-child",
        "ng-child",
        None,  // parenttile_id is null
        vec![("node-child", JsonValue::String("test".to_string()))],
    );

    let matches = matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        Some(&child_tile),
        "ng-child",
    );

    assert!(matches, "Branch 1: Different nodegroup + null parenttile_id should match");
}

/// Test Branch 1: Different nodegroup + matching parenttile_id
/// PORT: js/semantic.ts line 311 - (... || value.tile.parenttile_id == tile.tileid)
#[test]
fn test_different_nodegroup_matching_parent_tile() {
    let parent_tile_id = Some("tile-parent".to_string());
    let parent_nodegroup_id = Some("ng-parent".to_string());

    let child_node = create_test_node("node-child", "child_alias", Some("ng-child".to_string()), false);
    let child_tile = create_test_tile(
        "tile-child",
        "ng-child",
        Some("tile-parent".to_string()),  // parenttile_id matches parent
        vec![("node-child", JsonValue::String("test".to_string()))],
    );

    let matches = matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        Some(&child_tile),
        "ng-child",
    );

    assert!(matches, "Branch 1: Different nodegroup + matching parenttile_id should match");
}

/// Test Branch 1: Different nodegroup + NON-matching parenttile_id
/// Should NOT match
#[test]
fn test_different_nodegroup_non_matching_parent_tile() {

    let parent_tile_id = Some("tile-parent".to_string());
    let parent_nodegroup_id = Some("ng-parent".to_string());

    let child_node = create_test_node("node-child", "child_alias", Some("ng-child".to_string()), false);
    let child_tile = create_test_tile(
        "tile-child",
        "ng-child",
        Some("tile-different-parent".to_string()),  // parenttile_id does NOT match parent
        vec![("node-child", JsonValue::String("test".to_string()))],
    );

    let matches = matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        Some(&child_tile),
        "ng-child",
    );

    assert!(!matches, "Branch 1: Different nodegroup + non-matching parenttile_id should NOT match");
}

/// Test Branch 2: Same nodegroup + shared tile + not collector
/// PORT: js/semantic.ts lines 312-315
#[test]
fn test_same_nodegroup_shared_tile_not_collector() {

    let parent_tile_id = Some("tile-shared".to_string());
    let parent_nodegroup_id = Some("ng-shared".to_string());

    // Child is in same nodegroup, not a collector
    let child_node = create_test_node("node-child", "child_alias", Some("ng-shared".to_string()), false);
    let child_tile = create_test_tile(
        "tile-shared",
        "ng-shared",
        None,
        vec![("node-child", JsonValue::String("test".to_string()))],
    );

    let matches = matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        Some(&child_tile),
        "ng-shared",  // Tile is in ng-shared
    );

    assert!(matches, "Branch 2: Same nodegroup + shared tile + not collector should match");
}

/// Test Branch 2: Same nodegroup + shared tile + IS collector
/// Should NOT match (collector nodes don't share tiles)
#[test]
fn test_same_nodegroup_shared_tile_is_collector() {

    let parent_tile_id = Some("tile-shared".to_string());
    let parent_nodegroup_id = Some("ng-shared".to_string());

    // Child is a collector - should NOT match Branch 2
    let child_node = create_test_node("node-child", "child_alias", Some("ng-shared".to_string()), true);
    let child_tile = create_test_tile(
        "tile-shared",
        "ng-shared",
        None,
        vec![("node-child", JsonValue::String("test".to_string()))],
    );

    let matches = matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        Some(&child_tile),
        "ng-shared",  // Tile is in ng-shared
    );

    assert!(!matches, "Branch 2: Same nodegroup + shared tile + IS collector should NOT match");
}

/// Test Branch 3: Different nodegroup + is_collector
/// PORT: js/semantic.ts lines 324-327
#[test]
fn test_different_nodegroup_is_collector() {

    let parent_tile_id = Some("tile-parent".to_string());
    let parent_nodegroup_id = Some("ng-parent".to_string());

    // Child is in different nodegroup AND is a collector
    let child_node = create_test_node("node-child", "child_alias", Some("ng-child".to_string()), true);
    let child_tile = create_test_tile(
        "tile-child",
        "ng-child",
        Some("tile-some-other-parent".to_string()),  // parenttile_id doesn't match
        vec![("node-child", JsonValue::String("test".to_string()))],
    );

    let matches = matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        Some(&child_tile),
        "ng-child",
    );

    assert!(matches, "Branch 3: Different nodegroup + is_collector should match");
}

/// Test Edge Case: Same nodegroup + different tile
/// Should NOT match (not shared)
#[test]
fn test_same_nodegroup_different_tile() {

    let parent_tile_id = Some("tile-parent".to_string());
    let parent_nodegroup_id = Some("ng-shared".to_string());

    let child_node = create_test_node("node-child", "child_alias", Some("ng-shared".to_string()), false);
    let child_tile = create_test_tile(
        "tile-different",  // Different tile ID
        "ng-shared",
        None,
        vec![("node-child", JsonValue::String("test".to_string()))],
    );

    let matches = matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        Some(&child_tile),
        "ng-shared",  // Tile is in ng-shared
    );

    assert!(!matches, "Same nodegroup + different tile should NOT match");
}

/// Test Edge Case: null parent tile
/// Should NOT match any branch
#[test]
fn test_null_parent_tile() {

    let parent_tile_id: Option<String> = None;
    let parent_nodegroup_id = Some("ng-parent".to_string());

    let child_node = create_test_node("node-child", "child_alias", Some("ng-child".to_string()), false);
    let child_tile = create_test_tile(
        "tile-child",
        "ng-child",
        None,
        vec![("node-child", JsonValue::String("test".to_string()))],
    );

    let matches = matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        Some(&child_tile),
        "ng-child",
    );

    // With null parent tile, Branch 1 and Branch 2 won't match
    // But Branch 3 might match if different nodegroup + collector
    assert!(!matches, "Null parent tile + not collector should NOT match");
}

/// Test Edge Case: null parent tile + collector
/// Should match Branch 3
#[test]
fn test_null_parent_tile_collector() {

    let parent_tile_id: Option<String> = None;
    let parent_nodegroup_id = Some("ng-parent".to_string());

    let child_node = create_test_node("node-child", "child_alias", Some("ng-child".to_string()), true);
    let child_tile = create_test_tile(
        "tile-child",
        "ng-child",
        None,
        vec![("node-child", JsonValue::String("test".to_string()))],
    );

    let matches = matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child_node,
        Some(&child_tile),
        "ng-child",
    );

    assert!(matches, "Null parent tile + different nodegroup + collector should match Branch 3");
}

/// Test comprehensive scenario: Multiple children with different relationships
#[test]
fn test_multiple_children_relationships() {

    let parent_tile_id = Some("tile-parent".to_string());
    let parent_nodegroup_id = Some("ng-parent".to_string());

    // Child 1: Different nodegroup, null parent (Branch 1) - MATCH
    let child1 = create_test_node("node-child1", "child1", Some("ng-child1".to_string()), false);
    let tile1 = create_test_tile(
        "tile-child1",
        "ng-child1",
        None,
        vec![("node-child1", JsonValue::String("test".to_string()))],
    );
    assert!(matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child1,
        Some(&tile1),
        "ng-child1",
    ));

    // Child 2: Same nodegroup, shared tile, not collector (Branch 2) - MATCH
    let child2 = create_test_node("node-child2", "child2", parent_nodegroup_id.clone(), false);
    let tile2 = create_test_tile(
        "tile-parent",
        "ng-parent",
        None,
        vec![("node-child2", JsonValue::String("test".to_string()))],
    );
    assert!(matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child2,
        Some(&tile2),
        "ng-parent",
    ));

    // Child 3: Different nodegroup, collector (Branch 3) - MATCH
    let child3 = create_test_node("node-child3", "child3", Some("ng-child3".to_string()), true);
    let tile3 = create_test_tile(
        "tile-child3",
        "ng-child3",
        Some("tile-unrelated".to_string()),
        vec![("node-child3", JsonValue::String("test".to_string()))],
    );
    assert!(matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child3,
        Some(&tile3),
        "ng-child3",
    ));

    // Child 4: Same nodegroup, collector (no branch matches) - NO MATCH
    let child4 = create_test_node("node-child4", "child4", parent_nodegroup_id.clone(), true);
    let tile4 = create_test_tile(
        "tile-parent",
        "ng-parent",
        None,
        vec![("node-child4", JsonValue::String("test".to_string()))],
    );
    assert!(!matches_semantic_child(
        parent_tile_id.as_ref(),
        parent_nodegroup_id.as_ref(),
        &child4,
        Some(&tile4),
        "ng-parent",
    ));
}
