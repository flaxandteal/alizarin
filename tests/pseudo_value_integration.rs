/**
 * Integration tests for RustPseudoValue hierarchy construction
 *
 * PORT: tests/ensureNodegroup-edge-cases.test.ts
 * These tests verify that Rust lazy loading behavior matches TypeScript exactly
 *
 * Test Coverage (mirrors TS tests):
 * 1. Child node ID population from edges
 * 2. Lazy child creation by alias
 * 3. Tile data inheritance
 * 4. Nodegroup boundary enforcement
 * 5. Collector node handling (PseudoList)
 * 6. Multiple tiles for same node (grouping)
 * 7. Empty child cases
 */

use alizarin::pseudo_value::{RustPseudoValue, RustPseudoList};
use alizarin_core::{StaticNode, StaticTile};
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::json;

/// Helper to create a test node with all required fields
/// PORT: tests/ensureNodegroup-edge-cases.test.ts createTestGraph pattern
fn create_node(alias: &str, nodeid: &str, nodegroup_id: Option<&str>, is_collector: bool) -> Arc<StaticNode> {
    let node_json = json!({
        "alias": alias,
        "datatype": "string",
        "exportable": false,
        "graph_id": "test-graph",
        "hascustomalias": false,
        "is_collector": is_collector,
        "isrequired": false,
        "issearchable": false,
        "istopnode": nodegroup_id.is_none(),
        "name": alias,
        "nodegroup_id": nodegroup_id,
        "nodeid": nodeid,
        "sortorder": 0
    });
    Arc::new(serde_json::from_value(node_json).expect("Failed to create test node"))
}

/// Helper to create a test tile
/// PORT: tests/ensureNodegroup-edge-cases.test.ts createTestTile
fn create_tile(tileid: &str, nodegroup_id: &str, data: HashMap<String, serde_json::Value>) -> Arc<StaticTile> {
    let tile_json = json!({
        "data": data,
        "nodegroup_id": nodegroup_id,
        "resourceinstance_id": "test-resource",
        "tileid": tileid
    });
    Arc::new(serde_json::from_value(tile_json).expect("Failed to create test tile"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PORT: tests/ensureNodegroup-edge-cases.test.ts "should skip when sentinel is true"
    /// Test that child IDs are populated correctly from edges
    #[test]
    fn test_child_ids_populated_from_edges() {
        let parent = create_node("parent", "parent-id", Some("ng1"), false);
        let _child1 = create_node("child1", "child1-id", Some("ng1"), false);
        let _child2 = create_node("child2", "child2-id", Some("ng1"), false);

        let mut edges = HashMap::new();
        edges.insert("parent-id".to_string(), vec!["child1-id".to_string(), "child2-id".to_string()]);

        let mut parent_value = RustPseudoValue::from_node_and_tile(
            parent.clone(),
            None,
            None,
            vec![],
        );

        // PORT: js/pseudos.ts:466 - const childNodes = model.getChildNodes(nodeObj.nodeid)
        parent_value.populate_child_ids(&edges);

        assert_eq!(parent_value.child_node_ids.len(), 2);
        assert_eq!(parent_value.child_node_ids[0], "child1-id");
        assert_eq!(parent_value.child_node_ids[1], "child2-id");
    }

    /// PORT: tests/ensureNodegroup-edge-cases.test.ts "should handle null tiles"
    /// Test that child can be created without tile (lazy loading)
    #[test]
    fn test_lazy_child_creation_without_tile() {
        let parent = create_node("parent", "parent-id", Some("ng1"), false);
        let child = create_node("child", "child-id", Some("ng1"), false);

        let mut node_objs = HashMap::new();
        node_objs.insert("parent-id".to_string(), parent.clone());
        node_objs.insert("child-id".to_string(), child.clone());

        let mut edges = HashMap::new();
        edges.insert("parent-id".to_string(), vec!["child-id".to_string()]);

        // Create parent with child IDs
        let parent_value = RustPseudoValue::from_node_and_tile(
            parent.clone(),
            None,
            None,
            vec!["child-id".to_string()],
        );

        // PORT: js/semantic.ts:807 - rustChild = this.rust.getChildByAlias(alias)
        // Lazily get child when accessed
        let child_opt = parent_value.get_child_by_alias("child", &node_objs, &edges);

        assert!(child_opt.is_some());
        let child_value = child_opt.unwrap();
        assert_eq!(child_value.node_alias(), Some("child"));
        assert!(child_value.tile.is_none());
        assert!(child_value.tile_data.is_none());
    }

    /// PORT: tests/ensureNodegroup-edge-cases.test.ts "should filter tiles by nodegroup_id"
    /// Test that tile data is correctly extracted for child
    #[test]
    fn test_lazy_child_creation_with_tile_data() {
        let parent = create_node("parent", "parent-id", Some("ng1"), false);
        let child = create_node("child", "child-id", Some("ng1"), false);

        let mut node_objs = HashMap::new();
        node_objs.insert("parent-id".to_string(), parent.clone());
        node_objs.insert("child-id".to_string(), child.clone());

        // Create tile with data for child
        let mut tile_data = HashMap::new();
        tile_data.insert("child-id".to_string(), json!({"value": "test_value", "number": 42}));
        let tile = create_tile("tile1", "ng1", tile_data);

        let mut edges = HashMap::new();
        edges.insert("parent-id".to_string(), vec!["child-id".to_string()]);

        // Create parent with tile and child IDs
        let parent_value = RustPseudoValue::from_node_and_tile(
            parent.clone(),
            Some(tile.clone()),
            None,
            vec!["child-id".to_string()],
        );

        // PORT: Lazy child creation extracts tile data from parent's tile
        let child_value = parent_value.get_child_by_alias("child", &node_objs, &edges).unwrap();

        // Verify tile data was extracted
        assert!(child_value.tile_data.is_some());
        let data = child_value.tile_data.as_ref().unwrap();
        assert_eq!(data["value"], "test_value");
        assert_eq!(data["number"], 42);
    }

    /// PORT: tests/ensureNodegroup-edge-cases.test.ts implied nodegroups
    /// Test that children from different nodegroups are NOT included
    #[test]
    fn test_nodegroup_boundary_enforcement() {
        let parent = create_node("parent", "parent-id", Some("ng1"), false);
        let child_same_ng = create_node("child_same", "child1-id", Some("ng1"), false);
        let child_diff_ng = create_node("child_diff", "child2-id", Some("ng2"), false);

        let mut node_objs = HashMap::new();
        node_objs.insert("parent-id".to_string(), parent.clone());
        node_objs.insert("child1-id".to_string(), child_same_ng.clone());
        node_objs.insert("child2-id".to_string(), child_diff_ng.clone());

        let mut edges = HashMap::new();
        // Parent has edges to both children
        edges.insert("parent-id".to_string(), vec!["child1-id".to_string(), "child2-id".to_string()]);

        let parent_value = RustPseudoValue::from_node_and_tile(
            parent.clone(),
            None,
            None,
            vec!["child1-id".to_string(), "child2-id".to_string()],
        );

        // PORT: Should get child from same nodegroup
        let child_same = parent_value.get_child_by_alias("child_same", &node_objs, &edges);
        assert!(child_same.is_some());

        // PORT: Should NOT get child from different nodegroup (implies nodegroup boundary)
        let child_diff = parent_value.get_child_by_alias("child_diff", &node_objs, &edges);
        assert!(child_diff.is_none(), "Child from different nodegroup should not be accessible");
    }

    /// PORT: tests/ensureNodegroup-edge-cases.test.ts "should handle multiple tiles for same nodegroup"
    /// Test collector node with multiple tiles (PseudoList)
    #[test]
    fn test_collector_node_multiple_tiles() {
        let collector_node = create_node("collector", "collector-id", Some("ng1"), true);

        // Create two tiles with different data
        let mut tile1_data = HashMap::new();
        tile1_data.insert("collector-id".to_string(), json!({"value": "first"}));
        let tile1 = create_tile("tile1", "ng1", tile1_data);

        let mut tile2_data = HashMap::new();
        tile2_data.insert("collector-id".to_string(), json!({"value": "second"}));
        let tile2 = create_tile("tile2", "ng1", tile2_data);

        let edges = HashMap::new(); // No children for this test

        // PORT: js/graphManager.ts ensureNodegroup - collector with multiple tiles creates PseudoList
        let list = RustPseudoList::from_node_tiles(
            collector_node.clone(),
            vec![Some(tile1.clone()), Some(tile2.clone())],
            &edges,
            None, // parent JsValue
            false, // is_single - collectors can have multiple values
        );

        // Verify list has two groups (one per tile)
        assert_eq!(list.all_values().len(), 2);

        // Verify each value has correct tile data
        let values = list.all_values();
        assert!(values[0].tile_data.is_some());
        assert!(values[1].tile_data.is_some());
    }

    /// PORT: tests/ensureNodegroup-edge-cases.test.ts merging logic
    /// Test PseudoList merging when same tile appears multiple times
    #[test]
    fn test_pseudo_list_merge_same_tile() {
        let node = create_node("node", "node-id", Some("ng1"), true);
        let tile1 = create_tile("tile1", "ng1", HashMap::new());

        let edges = HashMap::new();

        // Create first list with tile1
        let list1 = RustPseudoList::from_node_tiles(
            node.clone(),
            vec![Some(tile1.clone())],
            &edges,
            None, // parent JsValue
            false, // is_single
        );

        // Create second list also with tile1
        let list2 = RustPseudoList::from_node_tiles(
            node.clone(),
            vec![Some(tile1.clone())],
            &edges,
            None, // parent JsValue
            false, // is_single
        );

        let mut merged = list1;
        merged.merge(list2);

        // PORT: js/graphManager.ts:992-1003 - should merge values from same tile
        // Should have 2 values merged (both from tile1)
        assert_eq!(merged.values.len(), 2);

        // Both values should be from the same tile
        let tile_ids: Vec<_> = merged.values.iter()
            .map(|v| v.tile.as_ref().and_then(|t| t.tileid.as_deref()))
            .collect();
        assert!(tile_ids.iter().all(|t| *t == Some("tile1")));
    }

    /// PORT: tests/ensureNodegroup-edge-cases.test.ts "should use [null] when no tiles match and addIfMissing is true"
    /// Test creating value with null tile
    #[test]
    fn test_value_with_null_tile() {
        let node = create_node("node", "node-id", Some("ng1"), false);

        let value = RustPseudoValue::from_node_and_tile(
            node.clone(),
            None,
            None,
            vec![],
        );

        assert!(value.tile.is_none());
        assert!(value.tile_data.is_none());
        assert_eq!(value.child_node_ids.len(), 0);
    }

    /// PORT: tests/ensureNodegroup-edge-cases.test.ts "should not merge undefined values"
    /// Test that grandchildren get child IDs populated recursively
    #[test]
    fn test_nested_child_ids() {
        let parent = create_node("parent", "parent-id", Some("ng1"), false);
        let child = create_node("child", "child-id", Some("ng1"), false);
        let grandchild = create_node("grandchild", "grandchild-id", Some("ng1"), false);

        let mut node_objs = HashMap::new();
        node_objs.insert("parent-id".to_string(), parent.clone());
        node_objs.insert("child-id".to_string(), child.clone());
        node_objs.insert("grandchild-id".to_string(), grandchild.clone());

        let mut edges = HashMap::new();
        edges.insert("parent-id".to_string(), vec!["child-id".to_string()]);
        edges.insert("child-id".to_string(), vec!["grandchild-id".to_string()]);

        let parent_value = RustPseudoValue::from_node_and_tile(
            parent.clone(),
            None,
            None,
            vec!["child-id".to_string()],
        );

        // Get child (lazy)
        let child_value = parent_value.get_child_by_alias("child", &node_objs, &edges).unwrap();

        // PORT: Child should have its child IDs populated
        // For non-semantic nodes with children, an inner is created and children go through inner
        // The outer's child_node_ids will be empty, inner's will have the children
        assert!(child_value.inner.is_some(), "Non-semantic node with children should have inner");
        let inner = child_value.inner.as_ref().unwrap();
        assert_eq!(inner.child_node_ids.len(), 1);
        assert_eq!(inner.child_node_ids[0], "grandchild-id");

        // Can lazily get grandchild from child's inner
        let grandchild_value = inner.get_child_by_alias("grandchild", &node_objs, &edges);
        assert!(grandchild_value.is_some());
    }

    /// PORT: tests/ensureNodegroup-edge-cases.test.ts "should return empty maps when sentinel prevents processing"
    /// Test that non-existent child returns None
    #[test]
    fn test_nonexistent_child_returns_none() {
        let parent = create_node("parent", "parent-id", Some("ng1"), false);
        let child = create_node("child", "child-id", Some("ng1"), false);

        let mut node_objs = HashMap::new();
        node_objs.insert("parent-id".to_string(), parent.clone());
        node_objs.insert("child-id".to_string(), child.clone());

        let edges = HashMap::new();

        let parent_value = RustPseudoValue::from_node_and_tile(
            parent.clone(),
            None,
            None,
            vec!["child-id".to_string()],
        );

        // Try to get child with wrong alias
        let result = parent_value.get_child_by_alias("nonexistent", &node_objs, &edges);
        assert!(result.is_none());
    }

    /// PORT: tests/ensureNodegroup-edge-cases.test.ts tile data extraction
    /// Test that child inherits parent's tile reference
    #[test]
    fn test_child_inherits_parent_tile() {
        let parent = create_node("parent", "parent-id", Some("ng1"), false);
        let child = create_node("child", "child-id", Some("ng1"), false);

        let mut node_objs = HashMap::new();
        node_objs.insert("parent-id".to_string(), parent.clone());
        node_objs.insert("child-id".to_string(), child.clone());

        let tile = create_tile("tile1", "ng1", HashMap::new());

        let mut edges = HashMap::new();
        edges.insert("parent-id".to_string(), vec!["child-id".to_string()]);

        let parent_value = RustPseudoValue::from_node_and_tile(
            parent.clone(),
            Some(tile.clone()),
            None,
            vec!["child-id".to_string()],
        );

        let child_value = parent_value.get_child_by_alias("child", &node_objs, &edges).unwrap();

        // PORT: js/pseudos.ts:120 - tile inheritance
        assert!(child_value.tile.is_some());
        // Compare tile references (same Arc)
        assert!(Arc::ptr_eq(child_value.tile.as_ref().unwrap(), &tile));
    }
}
