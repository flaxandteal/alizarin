use std::cell::RefCell;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
// Use core types for internal storage (no WASM wrapper overhead)
use alizarin_core::type_serialization::{SerializationContext, SerializationOptions};
use alizarin_core::{PseudoValueCore, StaticNode, StaticTile};
// Import traits and generic functions for visitor pattern delegation
use alizarin_core::pseudo_value_core::{PseudoListLike, PseudoValueLike};
// WASM wrapper types for API boundary
use crate::graph::StaticNode as WasmStaticNode;
use crate::graph::StaticTile as WasmStaticTile;

/// Internal pseudo-value data structure.
/// Used internally for efficient iteration without RefCell overhead.
/// The public API uses `PseudoValue` (WASM wrapper) instead.
///
/// Contains a `PseudoValueCore` for the platform-agnostic fields,
/// plus WASM-specific fields for JS interop.
///
/// PORT: Combines logic from:
/// - js/pseudos.ts: PseudoValue class (constructor lines 112-144)
/// - js/graphManager.ts: makePseudoCls hierarchy construction
/// - alizarin_core::PseudoValueCore: shared data fields
#[derive(Clone, Debug)]
pub(crate) struct PseudoValueInner {
    // ============ Core fields (shared with json_conversion) ============
    /// Platform-agnostic core data. Note: core.inner is unused here;
    /// we use our own `inner` field to preserve WASM-specific fields.
    pub core: PseudoValueCore,

    /// Inner value for outer/inner pattern (semantic nodes)
    /// This shadows core.inner to preserve WASM-specific fields in the inner.
    /// PORT: js/pseudos.ts lines 134-143, 467-470
    pub inner: Option<Box<PseudoValueInner>>,

    // ============ WASM-specific Runtime State ============
    /// The loaded ViewModel (opaque JsValue - Rust doesn't inspect it)
    /// PORT: js/pseudos.ts:54 - value: any
    pub value: Option<JsValue>,

    /// Whether value has been loaded
    /// PORT: js/pseudos.ts:57 - valueLoaded: boolean | undefined = false
    /// None = not started, Some(false) = loading, Some(true) = loaded
    pub value_loaded: Option<bool>,

    /// Whether this value has been accessed
    /// PORT: js/pseudos.ts:59 - accessed: boolean
    pub accessed: bool,
}

// ============ Convenience accessors for core fields ============
// These are primarily used in tests
#[allow(dead_code)]
impl PseudoValueInner {
    /// The graph node this value represents
    #[inline]
    pub fn node(&self) -> &Arc<StaticNode> {
        &self.core.node
    }

    /// Child node IDs (metadata) - NOT instantiated children
    #[inline]
    pub fn child_node_ids(&self) -> &Vec<String> {
        &self.core.child_node_ids
    }

    /// Mutable access to child node IDs
    #[inline]
    pub fn child_node_ids_mut(&mut self) -> &mut Vec<String> {
        &mut self.core.child_node_ids
    }

    /// Whether this is a collector node (has multiple tiles)
    #[inline]
    pub fn is_collector(&self) -> bool {
        self.core.is_collector
    }

    /// Set is_collector flag
    #[inline]
    pub fn set_is_collector(&mut self, val: bool) {
        self.core.is_collector = val;
    }

    /// Whether this IS the inner (datatype overridden to "semantic")
    #[inline]
    pub fn is_inner(&self) -> bool {
        self.core.is_inner
    }

    /// Optional tile containing the data
    #[inline]
    pub fn tile(&self) -> &Option<Arc<StaticTile>> {
        &self.core.tile
    }

    /// Set tile
    #[inline]
    pub fn set_tile(&mut self, tile: Option<Arc<StaticTile>>) {
        self.core.tile = tile;
    }

    /// Raw tile data (unparsed, as serde_json::Value)
    #[inline]
    pub fn tile_data(&self) -> &Option<serde_json::Value> {
        &self.core.tile_data
    }

    /// Mutable access to tile data
    #[inline]
    pub fn tile_data_mut(&mut self) -> &mut Option<serde_json::Value> {
        &mut self.core.tile_data
    }

    /// Whether this tile is independent (not inherited from parent)
    #[inline]
    pub fn independent(&self) -> bool {
        self.core.independent
    }
}

/// Internal pseudo-list data structure.
/// Used internally for efficient iteration without RefCell overhead.
/// The public API uses `PseudoList` (WASM wrapper) instead.
///
/// PORT: js/pseudos.ts PseudoList class
#[derive(Clone, Debug)]
pub(crate) struct PseudoListInner {
    /// The node alias this list represents
    pub node_alias: String,

    /// Grouped values by tile
    pub values: Vec<PseudoValueInner>,

    /// Whether this list has been fully loaded
    pub is_loaded: bool,

    /// Whether this should be unwrapped to a single value when returned
    /// (cardinality-1 nodes store as list internally but return single value)
    pub is_single: bool,
}

impl PseudoListInner {
    /// Create from a PseudoListCore, wrapping each value with WASM runtime state.
    pub fn from_core(core: alizarin_core::PseudoListCore) -> Self {
        PseudoListInner {
            node_alias: core.node_alias,
            values: core
                .values
                .into_iter()
                .map(PseudoValueInner::from_core)
                .collect(),
            is_loaded: core.is_loaded,
            is_single: core.is_single,
        }
    }
}

/// Result returned from nodegroup operations
/// Contains structured hierarchy instead of flat recipes
#[derive(Clone)]
pub(crate) struct NodegroupResult {
    /// Structured values with full hierarchy
    pub values: Vec<PseudoValueInner>,

    /// Implied nodegroups that need loading
    pub implied_nodegroups: Vec<String>,
}

impl PseudoValueInner {
    /// Wrap a PseudoValueCore with WASM-specific runtime state.
    ///
    /// Recursively wraps core's inner (if present) so the WASM inner/outer
    /// shadowing stays consistent with core's structural decision.
    pub fn from_core(core: PseudoValueCore) -> Self {
        let inner = core
            .inner
            .as_ref()
            .map(|inner_core| Box::new(PseudoValueInner::from_core((**inner_core).clone())));
        PseudoValueInner {
            core,
            inner,
            value: None,
            value_loaded: Some(false),
            accessed: false,
        }
    }

    /// Check if this is an outer node (has inner)
    pub fn is_outer(&self) -> bool {
        self.inner.is_some()
    }

    /// Get effective datatype (overridden to "semantic" for inner nodes)
    /// PORT: js/pseudos.ts:64-70 - datatype getter
    pub fn datatype(&self) -> &str {
        self.core.datatype()
    }

    /// Get the node alias
    pub fn node_alias(&self) -> Option<&str> {
        self.core.node_alias()
    }

    /// Check if this value has a tile
    pub fn has_tile(&self) -> bool {
        self.core.has_tile()
    }

    /// Get tile ID if present
    pub fn tile_id(&self) -> Option<String> {
        self.core.tile_id()
    }
}

// ============================================================================
// Trait implementations for visitor pattern delegation to core
// ============================================================================

impl PseudoValueLike for PseudoValueInner {
    fn tile(&self) -> &Option<Arc<StaticTile>> {
        &self.core.tile
    }

    fn node(&self) -> &Arc<StaticNode> {
        &self.core.node
    }

    fn is_inner(&self) -> bool {
        self.core.is_inner
    }

    fn is_outer(&self) -> bool {
        self.inner.is_some()
    }

    fn datatype(&self) -> &str {
        self.core.datatype()
    }

    fn serialize_own_value(
        &self,
        serialization_options: &SerializationOptions,
        serialization_context: Option<&SerializationContext>,
    ) -> serde_json::Value {
        self.core
            .serialize_own_value(serialization_options, serialization_context)
    }

    fn has_children(&self, edges: &std::collections::HashMap<String, Vec<String>>) -> bool {
        self.core.has_children(edges)
    }

    fn inner(&self) -> Option<&Self> {
        self.inner.as_ref().map(|b| b.as_ref())
    }
}

impl PseudoListLike for PseudoListInner {
    type Value = PseudoValueInner;

    fn matching_entries(
        &self,
        parent_tile_id: Option<String>,
        nodegroup_id: Option<String>,
        parent_nodegroup_id: Option<String>,
    ) -> Vec<&Self::Value> {
        self.matching_entries(parent_tile_id, nodegroup_id, parent_nodegroup_id)
    }

    fn is_single(&self) -> bool {
        self.is_single
    }

    fn values_count(&self) -> usize {
        self.values.len()
    }
}

impl PseudoListInner {
    /// Get all values across
    pub fn all_values(&self) -> Vec<&PseudoValueInner> {
        self.values.iter().collect()
    }

    pub fn matching_entries(
        &self,
        parent_tile_id: Option<String>,
        nodegroup_id: Option<String>,
        parent_nodegroup_id: Option<String>,
    ) -> Vec<&PseudoValueInner> {
        use alizarin_core::matches_tile_filter;
        let mut entries: Vec<&PseudoValueInner> = self
            .values
            .iter()
            .filter(|v| {
                match v.core.tile.as_ref() {
                    Some(tile) => matches_tile_filter(
                        tile,
                        parent_tile_id.as_ref(),
                        nodegroup_id.as_ref(),
                        parent_nodegroup_id.as_ref(),
                    ),
                    // Synthetic entries (no tile) match when at root level
                    // These are created for parent semantic collectors that don't have their own tiles
                    None => parent_tile_id.is_none(),
                }
            })
            .collect();

        // Sort by sortorder to ensure consistent ordering (lowest first = primary)
        entries.sort_by_key(|v| {
            v.core
                .tile
                .as_ref()
                .and_then(|t| t.sortorder)
                .unwrap_or(i32::MAX)
        });

        entries
    }
}

// WASM bindings for JavaScript interop
#[derive(Debug)]
#[wasm_bindgen]
pub struct PseudoValue {
    inner: RefCell<PseudoValueInner>,
}

#[wasm_bindgen]
impl PseudoValue {
    /// PORT: js/pseudos.ts:64-66 - datatype getter (respects isInner override)
    #[wasm_bindgen(getter, js_name = datatype)]
    pub fn datatype(&self) -> String {
        self.inner.borrow().datatype().to_string()
    }

    /// PORT: js/pseudos.ts - nodeAlias property
    #[wasm_bindgen(getter, js_name = nodeAlias, unchecked_return_type = "string | null")]
    pub fn node_alias(&self) -> JsValue {
        crate::js_json::or_null(self.inner.borrow().node_alias().map(|s| s.to_string()))
    }

    /// PORT: js/pseudos.ts:52 - node.nodeid
    #[wasm_bindgen(getter, js_name = nodeId)]
    pub fn node_id(&self) -> String {
        self.inner.borrow().core.node.nodeid.clone()
    }

    /// PORT: js/pseudos.ts - hasTile check
    #[wasm_bindgen(getter, js_name = hasTile)]
    pub fn has_tile(&self) -> bool {
        self.inner.borrow().has_tile()
    }

    /// PORT: js/pseudos.ts:53 - tile property
    #[wasm_bindgen(getter, js_name = tile)]
    pub fn tile(&self) -> JsValue {
        match &self.inner.borrow().core.tile {
            Some(tile) => {
                // Return the StaticTile WASM object directly, not serialized
                // This preserves the WASM bindings and getters (especially .data getter)
                // Convert from core type to WASM wrapper type
                let core_tile = tile.as_ref().clone();
                let wasm_tile = WasmStaticTile::from(core_tile);
                wasm_tile.into()
            }
            None => JsValue::NULL,
        }
    }

    /// PORT: js/pseudos.ts - tile.tileid access
    #[wasm_bindgen(getter, js_name = tileId, unchecked_return_type = "string | null")]
    pub fn tile_id(&self) -> JsValue {
        crate::js_json::or_null(self.inner.borrow().tile_id())
    }

    /// PORT: js/pseudos.ts:54 - value property (tile data)
    #[wasm_bindgen(getter, js_name = tileData)]
    pub fn tile_data(&self) -> JsValue {
        match &self.inner.borrow().core.tile_data {
            Some(data) => crate::js_json::to_value(data).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// PORT: js/pseudos.ts - is_collector check
    #[wasm_bindgen(getter, js_name = isCollector)]
    pub fn is_collector(&self) -> bool {
        self.inner.borrow().core.is_collector
    }

    /// Get inner value (for outer/inner pattern)
    /// PORT: js/pseudos.ts:61-62, 76-78 - inner property
    #[wasm_bindgen(getter, js_name = inner, unchecked_return_type = "PseudoValue | null")]
    pub fn get_inner(&self) -> JsValue {
        crate::js_json::or_null(self.inner.borrow().inner.as_ref().map(|inner| PseudoValue {
            inner: RefCell::new((**inner).clone()),
        }))
    }

    /// PORT: js/pseudos.ts:72-74 - isOuter getter
    #[wasm_bindgen(getter, js_name = isOuter)]
    pub fn is_outer(&self) -> bool {
        self.inner.borrow().is_outer()
    }

    /// PORT: js/pseudos.ts:73-75 - isInner getter
    #[wasm_bindgen(getter, js_name = isInner)]
    pub fn is_inner(&self) -> bool {
        self.inner.borrow().core.is_inner
    }

    /// PORT: js/pseudos.ts:60 - independent getter
    #[wasm_bindgen(getter, js_name = independent)]
    pub fn independent(&self) -> bool {
        self.inner.borrow().core.independent
    }

    /// PORT: js/pseudos.ts:260-265 - clear method
    #[wasm_bindgen(js_name = clear)]
    pub fn clear(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.value = None;
        inner.value_loaded = Some(false);
        // Also clear from tile.data if present
        // This would need mutable tile access, which is complex
    }

    /// Set the tile (for cases where tile changes)
    #[wasm_bindgen(setter, js_name = tile)]
    pub fn set_tile(&self, tile: JsValue) {
        // Core's, keeping tileData in step, as on NAPI.
        if tile.is_null() || tile.is_undefined() {
            self.inner.borrow_mut().core.set_tile(None);
        } else {
            // Convert JsValue to StaticTile via serde deserialization
            if let Ok(core_tile) = crate::js_json::from_value::<alizarin_core::StaticTile>(tile) {
                self.inner
                    .borrow_mut()
                    .core
                    .set_tile(Some(Arc::new(core_tile)));
            }
        }
    }

    /// PORT: js/pseudos.ts node getter - return node as WASM StaticNode wrapper
    #[wasm_bindgen(getter, js_name = node)]
    pub fn node(&self) -> WasmStaticNode {
        let inner = self.inner.borrow();
        // Clone the core node and wrap it in the WASM wrapper type
        let core_node: StaticNode = (*inner.core.node).clone();
        core_node.into()
    }

    /// Get the node's name
    #[wasm_bindgen(getter, js_name = name)]
    pub fn name(&self) -> String {
        self.inner.borrow().core.node.name.clone()
    }

    /// Get the node's nodegroup_id
    #[wasm_bindgen(getter, js_name = nodegroupId)]
    pub fn nodegroup_id(&self) -> JsValue {
        match &self.inner.borrow().core.node.nodegroup_id {
            Some(id) => JsValue::from_str(id),
            None => JsValue::NULL,
        }
    }

    // ============ PseudoNode compatibility methods ============
    // These mirror the PseudoNode API so PseudoValue can replace it

    /// Check if this node's datatype is iterable
    /// PORT: PseudoNode.isIterable()
    #[wasm_bindgen(js_name = isIterable)]
    pub fn is_iterable(&self) -> bool {
        // Core's, as on NAPI and Python.
        alizarin_core::is_iterable_datatype(&self.inner.borrow().core.node.datatype)
    }

    /// Get all commonly-used properties in a single boundary crossing.
    /// This avoids the overhead of multiple individual property getters.
    /// Returns a plain JS object with all properties cached on the JS side.
    #[wasm_bindgen(js_name = toSnapshot)]
    pub fn to_snapshot(&self) -> JsValue {
        let inner = self.inner.borrow();
        // Core's, as on NAPI, with WASM's runtime state added.
        let mut map = inner.core.snapshot();
        map.insert(
            "accessed".to_string(),
            serde_json::Value::Bool(inner.accessed),
        );
        if let Some(vl) = inner.value_loaded {
            map.insert("valueLoaded".to_string(), serde_json::Value::Bool(vl));
        }

        // Convert to JS via JSON parse (single crossing)
        let json_string =
            serde_json::to_string(&serde_json::Value::Object(map)).unwrap_or_default();
        js_sys::JSON::parse(&json_string).unwrap_or(JsValue::NULL)
    }
}

// Non-WASM methods for internal Rust use
impl PseudoValue {
    /// Create a PseudoValue from PseudoValueInner (for Rust-internal use)
    pub(crate) fn from_rust(inner: PseudoValueInner) -> Self {
        PseudoValue {
            inner: RefCell::new(inner),
        }
    }
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct PseudoList {
    inner: PseudoListInner,
}

#[wasm_bindgen]
impl PseudoList {
    /// PORT: js/pseudos.ts:337,398 - node property
    #[wasm_bindgen(getter, js_name = nodeAlias)]
    pub fn node_alias(&self) -> String {
        self.inner.node_alias.clone()
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

    /// Whether this list represents a cardinality-1 node (should unwrap to single value)
    #[wasm_bindgen(getter, js_name = isSingle)]
    pub fn is_single(&self) -> bool {
        self.inner.is_single
    }

    /// Get a specific value
    /// PORT: js/pseudos.ts:336 - Array element access
    #[wasm_bindgen(js_name = getValue, unchecked_return_type = "PseudoValue | null")]
    pub fn get_value(&self, value_index: usize) -> JsValue {
        crate::js_json::or_null(
            self.inner
                .values
                .get(value_index)
                .map(|v| PseudoValue::from_rust(v.clone())),
        )
    }

    /// Get all values from all groups as a flat array
    /// PORT: js/pseudos.ts:350 - map over array elements
    #[wasm_bindgen(js_name = getAllValues)]
    pub fn get_all_values(&self) -> Vec<PseudoValue> {
        self.inner
            .all_values()
            .into_iter()
            .map(|v| PseudoValue::from_rust(v.clone()))
            .collect()
    }

    /// Check if iterable
    /// PORT: js/pseudos.ts:345-347 - isIterable() method
    #[wasm_bindgen(js_name = isIterable)]
    pub fn is_iterable(&self) -> bool {
        true
    }
}

// Non-WASM methods for internal Rust use
impl PseudoList {
    /// Create a PseudoList from PseudoListInner (for Rust-internal use)
    pub(crate) fn from_rust(inner: PseudoListInner) -> Self {
        PseudoList { inner }
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
    #[wasm_bindgen(js_name = getValue, unchecked_return_type = "PseudoValue | null")]
    pub fn get_value(&self, index: usize) -> JsValue {
        crate::js_json::or_null(
            self.inner
                .values
                .get(index)
                .map(|v| PseudoValue::from_rust(v.clone())),
        )
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
        let pv = PseudoValueInner::from_core(PseudoValueCore::from_node_and_tile(
            node.clone(),
            None,
            None,
            vec![],
        ));

        assert_eq!(pv.node_alias(), Some("TestNode"));
        assert!(!pv.has_tile());
        assert_eq!(pv.child_node_ids().len(), 0);
        assert!(!pv.is_collector());
    }

    #[test]
    fn test_pseudo_list_grouping() {
        let node = create_test_node("TestNode");
        let tile1 = create_test_tile("tile1");
        let tile2 = create_test_tile("tile2");

        let values = vec![
            PseudoValueCore::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            PseudoValueCore::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            PseudoValueCore::from_node_and_tile(node.clone(), Some(tile2.clone()), None, vec![]),
        ];

        let list = PseudoListInner::from_core(alizarin_core::PseudoListCore::from_values(
            "TestNode".to_string(),
            values,
        ));

        assert_eq!(list.node_alias, "TestNode");
        assert_eq!(list.values.len(), 3); // Three total values (grouped by tile internally)
        assert_eq!(list.all_values().len(), 3); // Three total values
        assert!(list.is_loaded);
    }
}
