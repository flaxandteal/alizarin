use std::sync::Arc;
use wasm_bindgen::prelude::*;
use crate::graph::{StaticNode, StaticTile};

/// RustPseudoValue: Structured hierarchy representation
/// Replaces the flat "recipe" pattern with nested children
///
/// PORT: Combines logic from:
/// - js/pseudos.ts: PseudoValue class
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

    /// Nested children - full hierarchy!
    pub children: Vec<RustPseudoValue>,

    /// Whether this is a collector node (has multiple tiles)
    pub is_collector: bool,

    /// Optional parent index for navigation
    pub parent_index: Option<usize>,
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
    /// PORT: js/pseudos.ts makePseudoCls() function
    pub fn from_node_and_tile(
        node: Arc<StaticNode>,
        tile: Option<Arc<StaticTile>>,
        tile_data: Option<serde_json::Value>,
    ) -> Self {
        RustPseudoValue {
            node,
            tile,
            tile_data,
            children: Vec::new(),
            is_collector: false,
            parent_index: None,
        }
    }

    /// Build children for this pseudo value by traversing the graph
    ///
    /// PORT: js/graphManager.ts logic in makePseudoCls for building children
    /// This will be implemented in Phase 2
    pub fn build_children(
        &mut self,
        // Parameters TBD based on what's needed for traversal
    ) -> Result<(), String> {
        // TODO: Phase 2 implementation
        Ok(())
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
    /// Get the node alias
    #[wasm_bindgen(getter, js_name = nodeAlias)]
    pub fn node_alias(&self) -> Option<String> {
        self.inner.node_alias().map(|s| s.to_string())
    }

    /// Check if this value has a tile
    #[wasm_bindgen(getter, js_name = hasTile)]
    pub fn has_tile(&self) -> bool {
        self.inner.has_tile()
    }

    /// Get tile ID if present
    #[wasm_bindgen(getter, js_name = tileId)]
    pub fn tile_id(&self) -> Option<String> {
        self.inner.tile_id()
    }

    /// Get the tile data as JSON string
    #[wasm_bindgen(getter, js_name = tileData)]
    pub fn tile_data(&self) -> JsValue {
        match &self.inner.tile_data {
            Some(data) => serde_wasm_bindgen::to_value(data).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// Get number of children
    #[wasm_bindgen(getter, js_name = childrenCount)]
    pub fn children_count(&self) -> usize {
        self.inner.children.len()
    }

    /// Get a child by index
    #[wasm_bindgen(js_name = getChild)]
    pub fn get_child(&self, index: usize) -> Option<WasmPseudoValue> {
        self.inner.children.get(index).map(|child| {
            WasmPseudoValue {
                inner: child.clone(),
            }
        })
    }

    /// Check if this is a collector
    #[wasm_bindgen(getter, js_name = isCollector)]
    pub fn is_collector(&self) -> bool {
        self.inner.is_collector
    }
}

#[wasm_bindgen]
pub struct WasmPseudoList {
    inner: RustPseudoList,
}

#[wasm_bindgen]
impl WasmPseudoList {
    /// Get the node alias
    #[wasm_bindgen(getter, js_name = nodeAlias)]
    pub fn node_alias(&self) -> String {
        self.inner.node_alias.clone()
    }

    /// Get number of groups
    #[wasm_bindgen(getter, js_name = groupCount)]
    pub fn group_count(&self) -> usize {
        self.inner.groups.len()
    }

    /// Get total number of values across all groups
    #[wasm_bindgen(getter, js_name = totalValues)]
    pub fn total_values(&self) -> usize {
        self.inner.all_values().len()
    }

    /// Check if loaded
    #[wasm_bindgen(getter, js_name = isLoaded)]
    pub fn is_loaded(&self) -> bool {
        self.inner.is_loaded
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
        let pv = RustPseudoValue::from_node_and_tile(node.clone(), None, None);

        assert_eq!(pv.node_alias(), Some("TestNode"));
        assert!(!pv.has_tile());
        assert_eq!(pv.children.len(), 0);
        assert!(!pv.is_collector);
    }

    #[test]
    fn test_pseudo_list_grouping() {
        let node = create_test_node("TestNode");
        let tile1 = create_test_tile("tile1");
        let tile2 = create_test_tile("tile2");

        let values = vec![
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile1.clone()), None),
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile1.clone()), None),
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile2.clone()), None),
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
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile1.clone()), None),
            RustPseudoValue::from_node_and_tile(node.clone(), Some(tile1.clone()), None),
            RustPseudoValue::from_node_and_tile(node.clone(), None, None),
        ];

        let list = RustPseudoList::from_values("TestNode".to_string(), values);

        let tile1_values = list.values_from_tile(Some("tile1"));
        assert_eq!(tile1_values.len(), 2);

        let null_tile_values = list.values_from_tile(None);
        assert_eq!(null_tile_values.len(), 1);
    }
}
