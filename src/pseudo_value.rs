use std::sync::Arc;
use wasm_bindgen::prelude::*;
use crate::graph::{StaticNode, StaticTile};

/// RustPseudoValue: Structured hierarchy representation
/// Replaces the flat "recipe" pattern with nested children
///
/// PORT: Combines logic from:
/// - js/pseudos.ts: PseudoValue class (constructor lines 112-144)
/// - js/graphManager.ts: makePseudoCls hierarchy construction
#[derive(Clone, Debug)]
pub struct RustPseudoValue {
    /// The graph node this value represents
    pub node: Arc<StaticNode>,

    /// Optional tile containing the data
    pub tile: Option<Arc<StaticTile>>,

    /// Raw tile data (unparsed, as serde_json::Value)
    /// ViewModels in JS/Python will parse this
    pub tile_data: Option<serde_json::Value>,

    /// Child node IDs (metadata) - NOT instantiated children!
    /// PORT: js/pseudos.ts line 117 - childNodes: Map<string, StaticNode>
    /// Children are created lazily when accessed, not during construction
    pub child_node_ids: Vec<String>,

    /// Whether this is a collector node (has multiple tiles)
    pub is_collector: bool,

    /// Optional parent index for navigation
    pub parent_index: Option<usize>,

    /// Inner value for outer/inner pattern (semantic nodes)
    /// PORT: js/pseudos.ts lines 134-143, 467-470
    pub inner: Option<Box<RustPseudoValue>>,
}

/// RustPseudoList: Represents a list of values grouped by tiles
///
/// PORT: js/pseudos.ts PseudoList class
#[derive(Clone, Debug)]
pub struct RustPseudoList {
    /// The node alias this list represents
    pub node_alias: String,

    /// Grouped values by tile
    pub groups: Vec<RustPseudoGroup>,

    /// Whether this list has been fully loaded
    pub is_loaded: bool,
}

/// RustPseudoGroup: A group of values from the same tile
#[derive(Clone, Debug)]
pub struct RustPseudoGroup {
    /// Optional tile ID
    pub tile_id: Option<String>,

    /// Values in this group
    pub values: Vec<RustPseudoValue>,
}

/// Result returned from nodegroup operations
/// Contains structured hierarchy instead of flat recipes
#[derive(Clone, Debug)]
pub struct NodegroupResult {
    /// Structured values with full hierarchy
    pub values: Vec<RustPseudoValue>,

    /// Implied nodegroups that need loading
    pub implied_nodegroups: Vec<String>,
}

impl RustPseudoValue {
    /// Create a new RustPseudoValue from a node and optional tile
    ///
    /// PORT: js/pseudos.ts PseudoValue constructor (lines 112-144)
    /// PORT: js/pseudos.ts makePseudoCls() function (lines 435-484)
    pub fn from_node_and_tile(
        node: Arc<StaticNode>,
        tile: Option<Arc<StaticTile>>,
        tile_data: Option<serde_json::Value>,
        child_node_ids: Vec<String>,
    ) -> Self {
        RustPseudoValue {
            node,
            tile,
            tile_data,
            child_node_ids,
            is_collector: false,
            parent_index: None,
            inner: None,
        }
    }

    /// Populate child node IDs from edges (metadata only, not instantiated)
    ///
    /// PORT: js/pseudos.ts:466 - const childNodes: Map<string, StaticNode> = model.getChildNodes(nodeObj.nodeid);
    /// This ONLY stores the IDs, children are created lazily when accessed
    pub fn populate_child_ids(
        &mut self,
        edges: &std::collections::HashMap<String, Vec<String>>,
    ) {
        // PORT: js/model_wrapper.rs get_child_nodes() - uses edges map
        if let Some(child_ids) = edges.get(&self.node.nodeid) {
            self.child_node_ids = child_ids.clone();
        }
    }

    /// Get a child node by creating a PseudoValue for it on-demand
    ///
    /// PORT: Called when JS SemanticViewModel Proxy intercepts property access
    /// PORT: js/semantic.ts:783-824 - _getChild() method
    /// PORT: js/pseudos.ts:466-471 - child creation logic in makePseudoCls
    pub fn get_child_by_alias(
        &self,
        alias: &str,
        node_objs: &std::collections::HashMap<String, Arc<StaticNode>>,
        edges: &std::collections::HashMap<String, Vec<String>>,
    ) -> Option<RustPseudoValue> {
        // PORT: js/semantic.ts:807 - rustChild = this.rust.getChildByAlias(alias)
        // Find child node with matching alias
        for child_id in &self.child_node_ids {
            if let Some(child_node) = node_objs.get(child_id) {
                // PORT: Check alias matches
                if child_node.alias.as_deref() == Some(alias) {
                    // PORT: Skip if different nodegroup (maintains nodegroup boundaries)
                    // This prevents loading across nodegroup boundaries
                    if child_node.nodegroup_id != self.node.nodegroup_id {
                        continue;
                    }

                    // PORT: js/pseudos.ts:120,133 - tile inheritance from parent
                    // Extract tile data for this child from parent's tile
                    let child_tile_data = self.tile.as_ref()
                        .and_then(|t| t.data.get(&child_node.nodeid))
                        .cloned();

                    // PORT: js/pseudos.ts:466 - get childNodes for the new child
                    // Get child's child node IDs (for next level of lazy loading)
                    let child_child_ids = edges.get(&child_node.nodeid)
                        .cloned()
                        .unwrap_or_default();

                    // PORT: js/pseudos.ts:471 - new PseudoValue(nodeObj, tile, null, wkri, childNodes, inner)
                    // Create child value (lazy - no recursion, just store metadata!)
                    return Some(RustPseudoValue {
                        node: Arc::clone(child_node),
                        tile: self.tile.clone(), // PORT: js/pseudos.ts:120 - inherit tile
                        tile_data: child_tile_data,
                        child_node_ids: child_child_ids, // Metadata for next level
                        is_collector: child_node.is_collector,
                        parent_index: None,
                        inner: None, // TODO: PORT js/pseudos.ts:467-470 inner/outer pattern
                    });
                }
            }
        }

        None
    }

    /// Get the node alias
    pub fn node_alias(&self) -> Option<&str> {
        self.node.alias.as_deref()
    }

    /// Check if this value has a tile
    pub fn has_tile(&self) -> bool {
        self.tile.is_some()
    }

    /// Get tile ID if present
    pub fn tile_id(&self) -> Option<String> {
        self.tile.as_ref().and_then(|t| t.tileid.clone())
    }
}

impl RustPseudoList {
    /// Create a new empty PseudoList
    pub fn new(node_alias: String) -> Self {
        RustPseudoList {
            node_alias,
            groups: Vec::new(),
            is_loaded: false,
        }
    }

    /// Group values by their tiles
    ///
    /// PORT: js/pseudos.ts PseudoList grouping logic
    /// PORT: js/graphManager.ts lines 987-1011 - PseudoList merging
    pub fn from_values(node_alias: String, values: Vec<RustPseudoValue>) -> Self {
        let mut groups_map: std::collections::HashMap<Option<String>, Vec<RustPseudoValue>> =
            std::collections::HashMap::new();

        // Group values by tile_id
        for value in values {
            let tile_id = value.tile_id();
            groups_map.entry(tile_id.clone()).or_insert_with(Vec::new).push(value);
        }

        // Convert to groups
        let groups: Vec<RustPseudoGroup> = groups_map.into_iter()
            .map(|(tile_id, values)| RustPseudoGroup { tile_id, values })
            .collect();

        RustPseudoList {
            node_alias,
            groups,
            is_loaded: true,
        }
    }

    /// Create from multiple tiles for the same node (collector pattern)
    ///
    /// PORT: js/graphManager.ts ensureNodegroup recipe processing
    /// PORT: js/pseudos.ts:452-461 - collector node handling
    /// When a node is a collector (is_collector=true) with multiple tiles,
    /// group them into a PseudoList
    pub fn from_node_tiles(
        node: Arc<StaticNode>,
        tiles: Vec<Option<Arc<StaticTile>>>,
        edges: &std::collections::HashMap<String, Vec<String>>,
    ) -> Self {
        let mut values = Vec::new();

        // PORT: js/pseudos.ts:466 - get child node IDs once (same for all tiles)
        let child_node_ids = edges.get(&node.nodeid)
            .cloned()
            .unwrap_or_default();

        for tile_opt in tiles {
            // PORT: Extract tile data for this specific tile
            let tile_data = tile_opt.as_ref()
                .and_then(|t| t.data.get(&node.nodeid))
                .cloned();

            // PORT: js/pseudos.ts:471 - create PseudoValue with childNodes metadata
            let value = RustPseudoValue {
                node: Arc::clone(&node),
                tile: tile_opt,
                tile_data,
                child_node_ids: child_node_ids.clone(), // Same metadata for all
                is_collector: node.is_collector,
                parent_index: None,
                inner: None,
            };

            values.push(value);
        }

        Self::from_values(node.alias.clone().unwrap_or_default(), values)
    }

    /// Merge another PseudoList into this one
    ///
    /// PORT: js/graphManager.ts lines 992-1003 - merging logic
    pub fn merge(&mut self, mut other: RustPseudoList) {
        // Merge groups
        for mut other_group in other.groups.drain(..) {
            // Check if we already have a group for this tile
            let tile_id = other_group.tile_id.clone();
            let mut found_match = false;

            for existing_group in &mut self.groups {
                if existing_group.tile_id == tile_id {
                    // Merge values - drain from other_group to avoid move issues
                    existing_group.values.extend(other_group.values.drain(..));
                    found_match = true;
                    break;
                }
            }

            if !found_match {
                // Add as new group (other_group.values is still valid if we didn't drain it)
                self.groups.push(other_group);
            }
        }
    }

    /// Get all values across all groups
    pub fn all_values(&self) -> Vec<&RustPseudoValue> {
        self.groups.iter()
            .flat_map(|g| g.values.iter())
            .collect()
    }

    /// Get values from a specific tile
    pub fn values_from_tile(&self, tile_id: Option<&str>) -> Vec<&RustPseudoValue> {
        self.groups.iter()
            .filter(|g| g.tile_id.as_deref() == tile_id)
            .flat_map(|g| g.values.iter())
            .collect()
    }
}

// WASM bindings for JavaScript interop
#[wasm_bindgen]
pub struct WasmPseudoValue {
    inner: RustPseudoValue,
}

#[wasm_bindgen]
impl WasmPseudoValue {
    /// PORT: js/pseudos.ts:64-66 - datatype getter
    #[wasm_bindgen(getter, js_name = datatype)]
    pub fn datatype(&self) -> String {
        self.inner.node.datatype.clone()
    }

    /// PORT: js/pseudos.ts - nodeAlias property
    #[wasm_bindgen(getter, js_name = nodeAlias)]
    pub fn node_alias(&self) -> Option<String> {
        self.inner.node_alias().map(|s| s.to_string())
    }

    /// PORT: js/pseudos.ts:52 - node.nodeid
    #[wasm_bindgen(getter, js_name = nodeId)]
    pub fn node_id(&self) -> String {
        self.inner.node.nodeid.clone()
    }

    /// PORT: js/pseudos.ts - hasTile check
    #[wasm_bindgen(getter, js_name = hasTile)]
    pub fn has_tile(&self) -> bool {
        self.inner.has_tile()
    }

    /// PORT: js/pseudos.ts:53 - tile property
    #[wasm_bindgen(getter, js_name = tile)]
    pub fn tile(&self) -> JsValue {
        match &self.inner.tile {
            Some(tile) => serde_wasm_bindgen::to_value(tile.as_ref()).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// PORT: js/pseudos.ts - tile.tileid access
    #[wasm_bindgen(getter, js_name = tileId)]
    pub fn tile_id(&self) -> Option<String> {
        self.inner.tile_id()
    }

    /// PORT: js/pseudos.ts:54 - value property (tile data)
    #[wasm_bindgen(getter, js_name = tileData)]
    pub fn tile_data(&self) -> JsValue {
        match &self.inner.tile_data {
            Some(data) => serde_wasm_bindgen::to_value(data).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// PORT: js/pseudos.ts - child node IDs count (for lazy loading)
    #[wasm_bindgen(getter, js_name = childNodeIdsCount)]
    pub fn child_node_ids_count(&self) -> usize {
        self.inner.child_node_ids.len()
    }

    /// PORT: js/pseudos.ts - get child node ID by index
    #[wasm_bindgen(js_name = getChildNodeId)]
    pub fn get_child_node_id(&self, index: usize) -> Option<String> {
        self.inner.child_node_ids.get(index).cloned()
    }

    /// PORT: js/pseudos.ts - is_collector check
    #[wasm_bindgen(getter, js_name = isCollector)]
    pub fn is_collector(&self) -> bool {
        self.inner.is_collector
    }

    /// Get inner value (for outer/inner pattern)
    /// PORT: js/pseudos.ts:61-62, 76-78 - inner property
    #[wasm_bindgen(getter, js_name = inner)]
    pub fn inner(&self) -> Option<WasmPseudoValue> {
        self.inner.inner.as_ref().map(|inner| WasmPseudoValue {
            inner: (**inner).clone(),
        })
    }
}

#[wasm_bindgen]
pub struct WasmPseudoList {
    inner: RustPseudoList,
}

#[wasm_bindgen]
impl WasmPseudoList {
    /// PORT: js/pseudos.ts:337,398 - node property
    #[wasm_bindgen(getter, js_name = nodeAlias)]
    pub fn node_alias(&self) -> String {
        self.inner.node_alias.clone()
    }

    /// PORT: js/pseudos.ts:336 - PseudoList extends Array (groups count)
    #[wasm_bindgen(getter, js_name = groupCount)]
    pub fn group_count(&self) -> usize {
        self.inner.groups.len()
    }

    /// PORT: js/pseudos.ts:336 - total values across all groups (Array.length equivalent)
    #[wasm_bindgen(getter, js_name = totalValues)]
    pub fn total_values(&self) -> usize {
        self.inner.all_values().len()
    }

    /// PORT: js/pseudos.ts - loaded state check
    #[wasm_bindgen(getter, js_name = isLoaded)]
    pub fn is_loaded(&self) -> bool {
        self.inner.is_loaded
    }

    /// Get tile ID for a specific group
    /// PORT: js/pseudos.ts:340,406 - tile property access
    #[wasm_bindgen(js_name = getGroupTileId)]
    pub fn get_group_tile_id(&self, group_index: usize) -> Option<String> {
        self.inner.groups.get(group_index).and_then(|g| g.tile_id.clone())
    }

    /// Get number of values in a specific group
    /// PORT: js/pseudos.ts:336 - Array length for group
    #[wasm_bindgen(js_name = getGroupValueCount)]
    pub fn get_group_value_count(&self, group_index: usize) -> usize {
        self.inner.groups.get(group_index).map(|g| g.values.len()).unwrap_or(0)
    }

    /// Get a specific value from a specific group
    /// PORT: js/pseudos.ts:336 - Array element access
    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(&self, group_index: usize, value_index: usize) -> Option<WasmPseudoValue> {
        self.inner.groups
            .get(group_index)
            .and_then(|g| g.values.get(value_index))
            .map(|v| WasmPseudoValue { inner: v.clone() })
    }

    /// Get all values from all groups as a flat array
    /// PORT: js/pseudos.ts:350 - map over array elements
    #[wasm_bindgen(js_name = getAllValues)]
    pub fn get_all_values(&self) -> Vec<WasmPseudoValue> {
        self.inner.all_values()
            .into_iter()
            .map(|v| WasmPseudoValue { inner: v.clone() })
            .collect()
    }

    /// Check if iterable
    /// PORT: js/pseudos.ts:345-347 - isIterable() method
    #[wasm_bindgen(js_name = isIterable)]
    pub fn is_iterable(&self) -> bool {
        true
    }
}

#[wasm_bindgen]
pub struct WasmNodegroupResult {
    inner: NodegroupResult,
}

#[wasm_bindgen]
impl WasmNodegroupResult {
    /// Get number of values
    #[wasm_bindgen(getter, js_name = valueCount)]
    pub fn value_count(&self) -> usize {
        self.inner.values.len()
    }

    /// Get a value by index
    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(&self, index: usize) -> Option<WasmPseudoValue> {
        self.inner.values.get(index).map(|v| {
            WasmPseudoValue {
                inner: v.clone(),
            }
        })
    }

    /// Get implied nodegroups
    #[wasm_bindgen(getter, js_name = impliedNodegroups)]
    pub fn implied_nodegroups(&self) -> Vec<String> {
        self.inner.implied_nodegroups.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_node(alias: &str) -> Arc<StaticNode> {
        let node_json = json!({
            "alias": alias,
            "datatype": "string",
            "exportable": false,
            "graph_id": "test-graph",
            "hascustomalias": false,
            "is_collector": false,
            "isrequired": false,
            "issearchable": false,
            "istopnode": false,
            "name": "Test Node",
            "nodegroup_id": "test-ng",
            "nodeid": "test-node-id",
            "sortorder": 0
        });
        Arc::new(serde_json::from_value(node_json).expect("Failed to create test node"))
    }

    fn create_test_tile(tileid: &str) -> Arc<StaticTile> {
        let tile_json = json!({
            "data": {},
            "nodegroup_id": "test-ng",
            "resourceinstance_id": "test-ri",
            "tileid": tileid
        });
        Arc::new(serde_json::from_value(tile_json).expect("Failed to create test tile"))
    }

    #[test]
    fn test_pseudo_value_creation() {
        let node = create_test_node("TestNode");
        let pv = RustPseudoValue::from_node_and_tile(node.clone(), None, None, vec![]);

        assert_eq!(pv.node_alias(), Some("TestNode"));
        assert!(!pv.has_tile());
        assert_eq!(pv.child_node_ids.len(), 0);
        assert!(!pv.is_collector);
    }

    #[test]
    fn test_pseudo_list_grouping() {
        let node = create_test_node("TestNode");
        let tile1 = create_test_tile("tile1");
        let tile2 = create_test_tile("tile2");

        let values = vec![
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile2.clone()), None, vec![]),
        ];

        let list = RustPseudoList::from_values("TestNode".to_string(), values);

        assert_eq!(list.node_alias, "TestNode");
        assert_eq!(list.groups.len(), 2); // Two different tiles
        assert_eq!(list.all_values().len(), 3); // Three total values
        assert!(list.is_loaded);
    }

    #[test]
    fn test_pseudo_list_tile_filtering() {
        let node = create_test_node("TestNode");
        let tile1 = create_test_tile("tile1");

        let values = vec![
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            RustPseudoValue::from_node_and_tile(node.clone(), None, None, vec![]),
        ];

        let list = RustPseudoList::from_values("TestNode".to_string(), values);

        let tile1_values = list.values_from_tile(Some("tile1"));
        assert_eq!(tile1_values.len(), 2);

        let null_tile_values = list.values_from_tile(None);
        assert_eq!(null_tile_values.len(), 1);
    }

    #[test]
    fn test_populate_child_ids() {
        use std::collections::HashMap;

        let parent_node = create_test_node("ParentNode");
        let child1 = create_test_node("Child1");
        let child2 = create_test_node("Child2");

        let mut edges = HashMap::new();
        edges.insert(parent_node.nodeid.clone(), vec![child1.nodeid.clone(), child2.nodeid.clone()]);

        let mut parent_value = RustPseudoValue::from_node_and_tile(
            parent_node.clone(),
            None,
            None,
            vec![],
        );

        parent_value.populate_child_ids(&edges);

        assert_eq!(parent_value.child_node_ids.len(), 2);
    }

    #[test]
    fn test_get_child_by_alias() {
        use std::collections::HashMap;

        let parent_node = create_test_node("ParentNode");
        let child_node = create_test_node("ChildNode");

        let mut node_objs = HashMap::new();
        node_objs.insert(parent_node.nodeid.clone(), parent_node.clone());
        node_objs.insert(child_node.nodeid.clone(), child_node.clone());

        let mut edges = HashMap::new();
        edges.insert(parent_node.nodeid.clone(), vec![child_node.nodeid.clone()]);

        let parent_value = RustPseudoValue::from_node_and_tile(
            parent_node.clone(),
            None,
            None,
            vec![child_node.nodeid.clone()],
        );

        let child_opt = parent_value.get_child_by_alias("ChildNode", &node_objs, &edges);

        assert!(child_opt.is_some());
        let child = child_opt.unwrap();
        assert_eq!(child.node_alias(), Some("ChildNode"));
    }

    #[test]
    fn test_get_child_with_tile_data() {
        use std::collections::HashMap;

        let parent_node = create_test_node("ParentNode");
        let child_node = create_test_node("ChildNode");

        let mut tile = create_test_tile("test-tile");
        let tile_mut = Arc::make_mut(&mut tile);
        tile_mut.data.insert(
            child_node.nodeid.clone(),
            serde_json::json!({"value": "child_data"})
        );

        let mut node_objs = HashMap::new();
        node_objs.insert(parent_node.nodeid.clone(), parent_node.clone());
        node_objs.insert(child_node.nodeid.clone(), child_node.clone());

        let mut edges = HashMap::new();
        edges.insert(parent_node.nodeid.clone(), vec![child_node.nodeid.clone()]);

        let parent_value = RustPseudoValue::from_node_and_tile(
            parent_node.clone(),
            Some(tile.clone()),
            None,
            vec![child_node.nodeid.clone()],
        );

        let child = parent_value.get_child_by_alias("ChildNode", &node_objs, &edges).unwrap();

        assert!(child.tile_data.is_some());
        let child_data = child.tile_data.as_ref().unwrap();
        assert_eq!(child_data["value"], "child_data");
    }

    #[test]
    fn test_pseudo_list_merge() {
        let node = create_test_node("TestNode");
        let tile1 = create_test_tile("tile1");
        let tile2 = create_test_tile("tile2");

        let values1 = vec![
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
        ];
        let mut list1 = RustPseudoList::from_values("TestNode".to_string(), values1);

        let values2 = vec![
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile2.clone()), None, vec![]),
        ];
        let list2 = RustPseudoList::from_values("TestNode".to_string(), values2);

        list1.merge(list2);

        assert_eq!(list1.groups.len(), 2);

        let tile1_group = list1.groups.iter()
            .find(|g| g.tile_id.as_deref() == Some("tile1"))
            .unwrap();
        assert_eq!(tile1_group.values.len(), 2);

        let tile2_group = list1.groups.iter()
            .find(|g| g.tile_id.as_deref() == Some("tile2"))
            .unwrap();
        assert_eq!(tile2_group.values.len(), 1);
    }
}
