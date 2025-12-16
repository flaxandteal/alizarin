use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
// Use core types for internal storage (no WASM wrapper overhead)
use alizarin_core::{StaticNode, StaticTile, PseudoValueCore};
// WASM wrapper types for API boundary
use crate::graph::StaticTile as WasmStaticTile;
use crate::graph::StaticNode as WasmStaticNode;

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

    /// Parent IRIVM reference (opaque JsValue)
    /// PORT: js/pseudos.ts:55 - parent: IRIVM<any> | null
    pub parent: Option<JsValue>,

    /// Parent PseudoValue reference (for getNodePlaceholder traversal)
    /// PORT: js/pseudos.ts:56 - parentNode: PseudoValue<any> | null
    /// Using weak reference to avoid cycles
    pub parent_value: Option<Rc<RefCell<PseudoValueInner>>>,

    /// Whether value has been loaded
    /// PORT: js/pseudos.ts:57 - valueLoaded: boolean | undefined = false
    /// None = not started, Some(false) = loading, Some(true) = loaded
    pub value_loaded: Option<bool>,

    /// Whether this value has been accessed
    /// PORT: js/pseudos.ts:59 - accessed: boolean
    pub accessed: bool,

    /// Optional parent index for navigation (legacy, may remove)
    pub parent_index: Option<usize>,
}

// ============ Convenience accessors for core fields ============
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

    /// Original tile reference (before any modifications)
    #[inline]
    pub fn original_tile(&self) -> &Option<Arc<StaticTile>> {
        &self.core.original_tile
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
    /// Create a new PseudoValueInner from a node and optional tile
    ///
    /// PORT: js/pseudos.ts PseudoValue constructor (lines 112-144)
    /// PORT: js/pseudos.ts makePseudoCls() function (lines 435-484)
    pub fn from_node_and_tile(
        node: Arc<StaticNode>,
        tile: Option<Arc<StaticTile>>,
        tile_data: Option<serde_json::Value>,
        child_node_ids: Vec<String>,
    ) -> Self {
        let core = PseudoValueCore::from_node_and_tile(
            node,
            tile,
            tile_data,
            child_node_ids,
        );
        PseudoValueInner {
            core,
            inner: None,
            value: None,
            parent: None,
            parent_value: None,
            value_loaded: Some(false),
            accessed: false,
            parent_index: None,
        }
    }

    /// Create a new PseudoValueInner with full configuration
    ///
    /// PORT: js/pseudos.ts PseudoValue constructor with inner/outer handling
    pub fn new(
        node: Arc<StaticNode>,
        tile: Option<Arc<StaticTile>>,
        tile_data: Option<serde_json::Value>,
        parent: Option<JsValue>,
        child_node_ids: Vec<String>,
        is_inner_flag: bool,
    ) -> Self {
        let independent = tile.is_none();

        // PORT: js/pseudos.ts:467-470 - create inner if has children and not semantic
        let has_children = !child_node_ids.is_empty();
        let is_semantic = node.datatype == "semantic";
        let should_have_inner = has_children && !is_semantic && !is_inner_flag;

        let inner = if should_have_inner {
            // Create inner PseudoValue with same node but marked as inner
            let inner_core = PseudoValueCore {
                node: node.clone(),
                child_node_ids: child_node_ids.clone(),
                is_collector: node.is_collector,
                inner: None,  // core.inner unused
                is_inner: true,  // This IS the inner
                tile: tile.clone(),
                tile_data: None,  // Inner doesn't get tile_data directly
                independent,
                original_tile: tile.clone(),
            };
            Some(Box::new(PseudoValueInner {
                core: inner_core,
                inner: None,
                value: None,
                parent: parent.clone(),
                parent_value: None,
                value_loaded: Some(false),
                accessed: false,
                parent_index: None,
            }))
        } else {
            None
        };

        let core = PseudoValueCore {
            node,
            // Outer gets empty child_node_ids if it has inner (children go through inner)
            child_node_ids: if inner.is_some() { vec![] } else { child_node_ids },
            is_collector: false,
            inner: None,  // core.inner unused, we use our own
            is_inner: is_inner_flag,
            tile: tile.clone(),
            tile_data,
            independent,
            original_tile: tile,
        };

        PseudoValueInner {
            core,
            inner,
            value: None,
            parent,
            parent_value: None,
            value_loaded: Some(false),
            accessed: false,
            parent_index: None,
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

    /// Populate child node IDs from edges (metadata only, not instantiated)
    ///
    /// PORT: js/pseudos.ts:466 - const childNodes: Map<string, StaticNode> = model.getChildNodes(nodeObj.nodeid);
    /// This ONLY stores the IDs, children are created lazily when accessed
    pub fn populate_child_ids(
        &mut self,
        edges: &std::collections::HashMap<String, Vec<String>>,
    ) {
        // PORT: js/model_wrapper.rs get_child_nodes() - uses edges map
        if let Some(child_ids) = edges.get(&self.core.node.nodeid) {
            self.core.child_node_ids = child_ids.clone();
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
    ) -> Option<PseudoValueInner> {
        // PORT: js/semantic.ts:807 - rustChild = this.rust.getChildByAlias(alias)
        // Find child node with matching alias
        for child_id in &self.core.child_node_ids {
            if let Some(child_node) = node_objs.get(child_id) {
                // PORT: Check alias matches
                if child_node.alias.as_deref() == Some(alias) {
                    // PORT: Skip if different nodegroup (maintains nodegroup boundaries)
                    // This prevents loading across nodegroup boundaries
                    if child_node.nodegroup_id != self.core.node.nodegroup_id {
                        continue;
                    }

                    // PORT: js/pseudos.ts:120,133 - tile inheritance from parent
                    // Extract tile data for this child from parent's tile
                    let child_tile_data = self.core.tile.as_ref()
                        .and_then(|t| t.data.get(&child_node.nodeid))
                        .cloned();

                    // PORT: js/pseudos.ts:466 - get childNodes for the new child
                    // Get child's child node IDs (for next level of lazy loading)
                    let child_child_ids = edges.get(&child_node.nodeid)
                        .cloned()
                        .unwrap_or_default();

                    // PORT: js/pseudos.ts:471 - new PseudoValue(nodeObj, tile, null, wkri, childNodes, inner)
                    // Create child value using new() which handles inner/outer pattern
                    return Some(PseudoValueInner::new(
                        Arc::clone(child_node),
                        self.core.tile.clone(), // PORT: js/pseudos.ts:120 - inherit tile
                        child_tile_data,
                        self.parent.clone(), // Inherit parent
                        child_child_ids,
                        false, // Not inner (new() will create inner if needed)
                    ));
                }
            }
        }

        None
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

impl PseudoListInner {
    /// Create a new empty PseudoList (defaults to list, not single)
    pub fn new(node_alias: String) -> Self {
        PseudoListInner {
            node_alias,
            values: Vec::new(),
            is_loaded: false,
            is_single: false,
        }
    }

    /// Create a new empty PseudoList with explicit is_single flag
    pub fn new_with_cardinality(node_alias: String, is_single: bool) -> Self {
        PseudoListInner {
            node_alias,
            values: Vec::new(),
            is_loaded: false,
            is_single,
        }
    }

    /// Group values by their tiles
    ///
    /// PORT: js/pseudos.ts PseudoList grouping logic
    /// PORT: js/graphManager.ts lines 987-1011 - PseudoList merging
    pub fn from_values(node_alias: String, values: Vec<PseudoValueInner>) -> Self {
        PseudoListInner {
            node_alias,
            values,
            is_loaded: true,
            is_single: false,
        }
    }

    /// Create from values with explicit is_single flag
    pub fn from_values_with_cardinality(node_alias: String, values: Vec<PseudoValueInner>, is_single: bool) -> Self {
        PseudoListInner {
            node_alias,
            values,
            is_loaded: true,
            is_single,
        }
    }

    /// Create from multiple tiles for the same node (collector pattern)
    ///
    /// PORT: js/graphManager.ts ensureNodegroup recipe processing
    /// PORT: js/pseudos.ts:452-461 - collector node handling
    /// When a node is a collector (is_collector=true) with multiple tiles,
    /// group them into a PseudoList
    ///
    /// is_single: true if this should unwrap to a single value (cardinality-1 node)
    pub fn from_node_tiles(
        node: Arc<StaticNode>,
        tiles: Vec<Option<Arc<StaticTile>>>,
        edges: &std::collections::HashMap<String, Vec<String>>,
        parent: Option<JsValue>,
        is_single: bool,
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
            // Use new() which handles inner/outer pattern
            let mut value = PseudoValueInner::new(
                Arc::clone(&node),
                tile_opt,
                tile_data,
                parent.clone(),
                child_node_ids.clone(),
                false,
            );
            value.core.is_collector = node.is_collector;

            values.push(value);
        }

        Self::from_values_with_cardinality(node.alias.clone().unwrap_or_default(), values, is_single)
    }

    /// Merge another PseudoList into this one
    ///
    /// PORT: js/graphManager.ts lines 992-1003 - merging logic
    pub fn merge(&mut self, mut other: PseudoListInner) {
        self.values.extend(other.values.drain(..));
    }

    /// Get all values across
    pub fn all_values(&self) -> Vec<&PseudoValueInner> {
        self.values.iter()
            .collect()
    }

    /// Get values from a specific tile
    pub fn values_from_tile(&self, tile_id: Option<&str>) -> Vec<&PseudoValueInner> {
        self.values.iter()
            .filter(|v| {
                match (v.core.tile.as_ref(), tile_id) {
                    // Both have tiles - compare tile IDs
                    (Some(tile), Some(id)) => tile.tileid.as_deref() == Some(id),
                    // Looking for values without a tile
                    (None, None) => true,
                    // Mismatch: one has tile, other doesn't
                    _ => false,
                }
            })
            .collect()
    }

    pub fn matching_entries(
        &self,
        parent_tile_id: Option<String>,
        nodegroup_id: Option<String>
    ) -> Vec<&PseudoValueInner> {
        let result: Vec<&PseudoValueInner> = self.values.iter()
            .filter(|v| {
                if let Some(tile) = v.core.tile.as_ref() {
                    if let Some(ng) = nodegroup_id.as_ref() {
                        if &tile.nodegroup_id == ng {
                            // If the value's tile is the same as the parent tile, it's a match
                            // (values on the same tile as the parent semantic node)
                            if tile.tileid == parent_tile_id {
                                return true;
                            }
                            // If the value's tile has a parenttile_id matching the parent, it's a match
                            // (values on child tiles of the parent semantic node's tile)
                            if tile.parenttile_id.is_some() && tile.parenttile_id == parent_tile_id {
                                return true;
                            }
                            // If no parent_tile_id provided and tile has no parenttile_id, it's a root match
                            if parent_tile_id.is_none() && tile.parenttile_id.is_none() {
                                return true;
                            }
                        }
                    }
                }
                false
            }).collect();

        result
    }
}

// ============================================================================
// Visitor pattern for JSON serialization
// ============================================================================

use std::collections::HashMap;

/// Context passed to visitors during traversal
pub(crate) struct VisitorContext<'a> {
    /// The pseudo_cache from the instance wrapper (alias -> PseudoListInner)
    pub pseudo_cache: &'a HashMap<String, PseudoListInner>,
    /// Node alias to node mapping from the model
    pub nodes_by_alias: &'a HashMap<String, Arc<StaticNode>>,
    /// Node edges from the model (parent_nodeid -> child_nodeids)
    pub edges: &'a HashMap<String, Vec<String>>,
    /// Current traversal depth (for preventing infinite recursion)
    pub depth: usize,
    /// Maximum depth
    pub max_depth: usize,
}

impl PseudoValueInner {
    /// Convert this pseudo value to JSON, recursively visiting children for semantic nodes.
    ///
    /// This is the Rust-side implementation of forJson() that avoids WASM boundary crossings.
    ///
    /// Handles the inner/outer pattern:
    /// - Outer nodes (non-semantic with children): Have tile_data as value, inner holds children
    /// - Inner nodes: Synthetic semantic nodes that give structure to outer's children
    /// - Pure semantic nodes: No tile_data, only children
    /// - Leaf nodes: Just tile_data, no children
    pub fn to_json(&self, ctx: &VisitorContext) -> serde_json::Value {
        if ctx.depth > ctx.max_depth {
            return serde_json::Value::Null;
        }

        // Handle outer nodes with inner (non-semantic with children)
        // The outer has the value, the inner has the children structure
        if self.is_outer() {
            return self.outer_to_json(ctx);
        }

        // Handle inner nodes (synthetic semantic) - traverse children
        if self.core.is_inner {
            return self.semantic_to_json(ctx);
        }

        let datatype = self.datatype();

        // Pure semantic nodes - traverse children
        if datatype == "semantic" {
            return self.semantic_to_json(ctx);
        }

        // Leaf nodes - return tile_data
        if let Some(ref data) = self.core.tile_data {
            return data.clone();
        }

        serde_json::Value::Null
    }

    /// Convert an outer node to JSON
    /// Outer nodes have their own value (tile_data) PLUS children via their inner
    fn outer_to_json(&self, ctx: &VisitorContext) -> serde_json::Value {
        // Get the outer's own value
        let own_value = self.core.tile_data.clone().unwrap_or(serde_json::Value::Null);

        // If there's an inner, get children from it
        if let Some(ref inner) = self.inner {
            let children_json = inner.semantic_to_json(ctx);

            // If children is an object, merge with own value
            // The convention is to put own value under special key or return structured
            if let serde_json::Value::Object(mut children_map) = children_json {
                // Insert own value - using the datatype or "_value" as key
                // This matches how the JS ViewModels serialize
                if !own_value.is_null() {
                    children_map.insert("_value".to_string(), own_value);
                }
                return serde_json::Value::Object(children_map);
            }
        }

        // No inner or no children - just return own value
        own_value
    }

    /// Convert a semantic node (or inner node) to JSON by finding and visiting its children
    fn semantic_to_json(&self, ctx: &VisitorContext) -> serde_json::Value {
        let mut obj = serde_json::Map::new();

        // Get the tile ID for this semantic node (used to find matching children)
        let parent_tile_id = self.core.tile.as_ref().and_then(|t| t.tileid.clone());

        // Get child node IDs from the edges map
        let child_node_ids = match ctx.edges.get(&self.core.node.nodeid) {
            Some(ids) => ids,
            None => {
                return serde_json::Value::Object(obj);
            }
        };

        // For each child node, find matching values in the pseudo_cache
        for child_node_id in child_node_ids {
            // Find the child node to get its alias and nodegroup_id
            let child_node = ctx.nodes_by_alias.values()
                .find(|n| &n.nodeid == child_node_id);

            let child_node = match child_node {
                Some(n) => n,
                None => continue,
            };

            let child_alias = match &child_node.alias {
                Some(alias) if !alias.is_empty() => alias,
                _ => continue,
            };

            // Look up the pseudo list for this child alias
            let pseudo_list = match ctx.pseudo_cache.get(child_alias) {
                Some(list) => list,
                None => continue,
            };

            // Find values that match based on tile relationship
            let matching_values = pseudo_list.matching_entries(
                parent_tile_id.clone(),
                child_node.nodegroup_id.clone(),
            );

            if matching_values.is_empty() {
                continue;
            }

            // Create child context with incremented depth
            let child_ctx = VisitorContext {
                pseudo_cache: ctx.pseudo_cache,
                nodes_by_alias: ctx.nodes_by_alias,
                edges: ctx.edges,
                depth: ctx.depth + 1,
                max_depth: ctx.max_depth,
            };

            // If single value (cardinality 1), serialize directly
            // If multiple values (cardinality n / collector), serialize as array
            if pseudo_list.is_single || matching_values.len() == 1 {
                if let Some(first_value) = matching_values.first() {
                    let json_value = first_value.to_json(&child_ctx);
                    if !json_value.is_null() {
                        obj.insert(child_alias.clone(), json_value);
                    }
                }
            } else {
                // Multiple values - create array
                let arr: Vec<serde_json::Value> = matching_values.iter()
                    .map(|v| v.to_json(&child_ctx))
                    .filter(|v| !v.is_null())
                    .collect();

                if !arr.is_empty() {
                    obj.insert(child_alias.clone(), serde_json::Value::Array(arr));
                }
            }
        }

        serde_json::Value::Object(obj)
    }
}

impl PseudoListInner {
    /// Convert this pseudo list to JSON
    ///
    /// For single-cardinality nodes, returns the single value's JSON.
    /// For multi-cardinality (collector) nodes, returns an array.
    pub fn to_json(&self, ctx: &VisitorContext) -> serde_json::Value {
        if self.values.is_empty() {
            return serde_json::Value::Null;
        }

        if self.is_single {
            // Single value
            if let Some(first) = self.values.first() {
                return first.to_json(ctx);
            }
            return serde_json::Value::Null;
        }

        // Multiple values - array
        let arr: Vec<serde_json::Value> = self.values.iter()
            .map(|v| v.to_json(ctx))
            .filter(|v| !v.is_null())
            .collect();

        if arr.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::Value::Array(arr)
        }
    }

    /// Convert to JSON starting from root (no parent tile filter)
    pub fn to_json_from_root(&self, ctx: &VisitorContext) -> serde_json::Value {
        // For root-level, we want values where tile has no parenttile_id
        let root_values: Vec<&PseudoValueInner> = self.values.iter()
            .filter(|v| {
                match v.core.tile.as_ref() {
                    Some(tile) => tile.parenttile_id.is_none(),
                    None => true,
                }
            })
            .collect();

        if root_values.is_empty() {
            return serde_json::Value::Null;
        }

        if self.is_single || root_values.len() == 1 {
            if let Some(first) = root_values.first() {
                return first.to_json(ctx);
            }
            return serde_json::Value::Null;
        }

        // Multiple values - array
        let arr: Vec<serde_json::Value> = root_values.iter()
            .map(|v| v.to_json(ctx))
            .filter(|v| !v.is_null())
            .collect();

        if arr.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::Value::Array(arr)
        }
    }
}

// ============================================================================
// Tile Builder - Visitor pattern for building StaticTiles from pseudo cache
// ============================================================================

/// A tile being built - accumulates data from multiple leaf nodes that share the same tile
#[derive(Debug, Clone)]
pub(crate) struct TileBuilder {
    pub tileid: Option<String>,
    pub nodegroup_id: String,
    pub parenttile_id: Option<String>,
    pub resourceinstance_id: String,
    pub sortorder: Option<i32>,
    pub data: HashMap<String, serde_json::Value>,
}

impl TileBuilder {
    /// Create a new tile builder from a PseudoValueInner's tile
    pub fn from_pseudo(pseudo: &PseudoValueInner, resourceinstance_id: &str) -> Option<Self> {
        let tile = pseudo.core.tile.as_ref()?;
        Some(TileBuilder {
            tileid: tile.tileid.clone(),
            nodegroup_id: tile.nodegroup_id.clone(),
            parenttile_id: tile.parenttile_id.clone(),
            resourceinstance_id: resourceinstance_id.to_string(),
            sortorder: tile.sortorder,
            data: HashMap::new(),
        })
    }

    /// Create from a StaticTile reference
    pub fn from_tile(tile: &alizarin_core::StaticTile) -> Self {
        TileBuilder {
            tileid: tile.tileid.clone(),
            nodegroup_id: tile.nodegroup_id.clone(),
            parenttile_id: tile.parenttile_id.clone(),
            resourceinstance_id: tile.resourceinstance_id.clone(),
            sortorder: tile.sortorder,
            data: HashMap::new(),
        }
    }

    /// Convert to a StaticTile
    pub fn to_static_tile(&self) -> alizarin_core::StaticTile {
        alizarin_core::StaticTile {
            tileid: self.tileid.clone(),
            nodegroup_id: self.nodegroup_id.clone(),
            parenttile_id: self.parenttile_id.clone(),
            resourceinstance_id: self.resourceinstance_id.clone(),
            sortorder: self.sortorder,
            provisionaledits: None,
            data: self.data.clone(),
        }
    }

    /// Get a unique key for this tile (tileid if exists, or generated from nodegroup + sortorder)
    pub fn key(&self) -> String {
        self.tileid.clone().unwrap_or_else(|| {
            format!("new_{}_{}", self.nodegroup_id, self.sortorder.unwrap_or(0))
        })
    }
}

/// Context for tile building traversal
pub(crate) struct TileBuilderContext<'a> {
    /// The pseudo_cache from the instance wrapper (alias -> PseudoListInner)
    pub pseudo_cache: &'a HashMap<String, PseudoListInner>,
    /// Node alias to node mapping from the model
    pub nodes_by_alias: &'a HashMap<String, Arc<StaticNode>>,
    /// Node edges from the model (parent_nodeid -> child_nodeids)
    pub edges: &'a HashMap<String, Vec<String>>,
    /// Resource instance ID for new tiles
    pub resourceinstance_id: String,
    /// Current traversal depth
    pub depth: usize,
    /// Maximum depth
    pub max_depth: usize,
}

impl PseudoValueInner {
    /// Collect tile data from this pseudo value and its children.
    ///
    /// For leaf nodes: adds tile_data to the appropriate tile
    /// For semantic nodes: recursively collects from children
    ///
    /// The tiles map is keyed by tile key (tileid or generated key for new tiles)
    pub fn collect_tiles(
        &self,
        ctx: &TileBuilderContext,
        tiles: &mut HashMap<String, TileBuilder>,
    ) {
        if ctx.depth > ctx.max_depth {
            return;
        }

        // If this pseudo has a tile, ensure it exists in the map
        if let Some(ref tile) = self.core.tile {
            let tile_key = tile.tileid.clone().unwrap_or_else(|| {
                format!("new_{}_{}", tile.nodegroup_id, tile.sortorder.unwrap_or(0))
            });

            if !tiles.contains_key(&tile_key) {
                tiles.insert(tile_key.clone(), TileBuilder::from_tile(tile));
            }

            // For leaf nodes with tile_data, add to the tile's data map
            let datatype = self.datatype();
            if datatype != "semantic" && !self.core.is_inner {
                if let Some(ref data) = self.core.tile_data {
                    if let Some(tile_builder) = tiles.get_mut(&tile_key) {
                        tile_builder.data.insert(self.core.node.nodeid.clone(), data.clone());
                    }
                }
            }
        }

        // For semantic nodes and inner nodes, collect from children
        if self.datatype() == "semantic" || self.core.is_inner {
            self.collect_children_tiles(ctx, tiles);
        }

        // For outer nodes, also collect from inner
        if let Some(ref inner) = self.inner {
            let child_ctx = TileBuilderContext {
                pseudo_cache: ctx.pseudo_cache,
                nodes_by_alias: ctx.nodes_by_alias,
                edges: ctx.edges,
                resourceinstance_id: ctx.resourceinstance_id.clone(),
                depth: ctx.depth + 1,
                max_depth: ctx.max_depth,
            };
            inner.collect_tiles(&child_ctx, tiles);
        }
    }

    /// Collect tiles from children of a semantic node
    fn collect_children_tiles(
        &self,
        ctx: &TileBuilderContext,
        tiles: &mut HashMap<String, TileBuilder>,
    ) {
        let parent_tile_id = self.core.tile.as_ref().and_then(|t| t.tileid.clone());

        let child_node_ids = match ctx.edges.get(&self.core.node.nodeid) {
            Some(ids) => ids,
            None => return,
        };

        for child_node_id in child_node_ids {
            let child_node = ctx.nodes_by_alias.values()
                .find(|n| &n.nodeid == child_node_id);

            let child_node = match child_node {
                Some(n) => n,
                None => continue,
            };

            let child_alias = match &child_node.alias {
                Some(alias) if !alias.is_empty() => alias,
                _ => continue,
            };

            let pseudo_list = match ctx.pseudo_cache.get(child_alias) {
                Some(list) => list,
                None => continue,
            };

            let matching_values = pseudo_list.matching_entries(
                parent_tile_id.clone(),
                child_node.nodegroup_id.clone(),
            );

            let child_ctx = TileBuilderContext {
                pseudo_cache: ctx.pseudo_cache,
                nodes_by_alias: ctx.nodes_by_alias,
                edges: ctx.edges,
                resourceinstance_id: ctx.resourceinstance_id.clone(),
                depth: ctx.depth + 1,
                max_depth: ctx.max_depth,
            };

            for value in matching_values {
                value.collect_tiles(&child_ctx, tiles);
            }
        }
    }
}

impl PseudoListInner {
    /// Collect tiles from all values in this list
    pub fn collect_tiles(
        &self,
        ctx: &TileBuilderContext,
        tiles: &mut HashMap<String, TileBuilder>,
    ) {
        for value in &self.values {
            value.collect_tiles(ctx, tiles);
        }
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
    #[wasm_bindgen(getter, js_name = nodeAlias)]
    pub fn node_alias(&self) -> Option<String> {
        self.inner.borrow().node_alias().map(|s| s.to_string())
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
            },
            None => JsValue::NULL,
        }
    }

    /// PORT: js/pseudos.ts - tile.tileid access
    #[wasm_bindgen(getter, js_name = tileId)]
    pub fn tile_id(&self) -> Option<String> {
        self.inner.borrow().tile_id()
    }

    /// PORT: js/pseudos.ts:54 - value property (tile data)
    #[wasm_bindgen(getter, js_name = tileData)]
    pub fn tile_data(&self) -> JsValue {
        match &self.inner.borrow().core.tile_data {
            Some(data) => serde_wasm_bindgen::to_value(data).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// PORT: js/pseudos.ts - child node IDs count (for lazy loading)
    #[wasm_bindgen(getter, js_name = childNodeIdsCount)]
    pub fn child_node_ids_count(&self) -> usize {
        self.inner.borrow().core.child_node_ids.len()
    }

    /// PORT: js/pseudos.ts - get child node ID by index
    #[wasm_bindgen(js_name = getChildNodeId)]
    pub fn get_child_node_id(&self, index: usize) -> Option<String> {
        self.inner.borrow().core.child_node_ids.get(index).cloned()
    }

    /// PORT: js/pseudos.ts - is_collector check
    #[wasm_bindgen(getter, js_name = isCollector)]
    pub fn is_collector(&self) -> bool {
        self.inner.borrow().core.is_collector
    }

    /// Get inner value (for outer/inner pattern)
    /// PORT: js/pseudos.ts:61-62, 76-78 - inner property
    #[wasm_bindgen(getter, js_name = inner)]
    pub fn get_inner(&self) -> Option<PseudoValue> {
        self.inner.borrow().inner.as_ref().map(|inner| PseudoValue {
            inner: RefCell::new((**inner).clone()),
        })
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

    /// PORT: js/pseudos.ts:57 - valueLoaded getter
    #[wasm_bindgen(getter, js_name = valueLoaded)]
    pub fn value_loaded(&self) -> JsValue {
        match self.inner.borrow().value_loaded {
            None => JsValue::UNDEFINED,
            Some(false) => JsValue::FALSE,
            Some(true) => JsValue::TRUE,
        }
    }

    /// PORT: js/pseudos.ts:59 - accessed getter
    #[wasm_bindgen(getter, js_name = accessed)]
    pub fn accessed(&self) -> bool {
        self.inner.borrow().accessed
    }

    /// PORT: js/pseudos.ts:60 - independent getter
    #[wasm_bindgen(getter, js_name = independent)]
    pub fn independent(&self) -> bool {
        self.inner.borrow().core.independent
    }

    /// PORT: js/pseudos.ts:54 - value getter (the loaded ViewModel)
    #[wasm_bindgen(getter, js_name = value)]
    pub fn value(&self) -> JsValue {
        self.inner.borrow().value.clone().unwrap_or(JsValue::NULL)
    }

    /// PORT: js/pseudos.ts:55 - parent getter
    #[wasm_bindgen(getter, js_name = parent)]
    pub fn parent(&self) -> JsValue {
        self.inner.borrow().parent.clone().unwrap_or(JsValue::NULL)
    }

    /// PORT: js/pseudos.ts:55 - parent setter
    #[wasm_bindgen(setter, js_name = parent)]
    pub fn set_parent(&self, parent: JsValue) {
        self.inner.borrow_mut().parent = if parent.is_null() || parent.is_undefined() {
            None
        } else {
            Some(parent)
        };
    }

    /// PORT: js/pseudos.ts:267-356 - updateValue method
    /// This is the core method that loads the ViewModel via callback
    ///
    /// get_view_model callback signature:
    ///   (pseudo: PseudoValue, tile: StaticTile|null, node: StaticNode, data: any, isInner: bool) -> Promise<ViewModel>
    #[wasm_bindgen(js_name = updateValue)]
    pub fn update_value(&self, get_view_model: &js_sys::Function) -> Result<JsValue, JsValue> {
        let mut inner = self.inner.borrow_mut();

        // PORT: js/pseudos.ts:271 - if (tile) { this.tile = tile; }
        // Tile update is handled by setTile method if needed

        // PORT: js/pseudos.ts:272-274 - mark as accessed
        inner.accessed = true;
        if let Some(ref mut inner_val) = inner.inner {
            inner_val.accessed = true;
        }

        // PORT: js/pseudos.ts:275-284 - handle case where tile is null
        // If no tile and has inner, delegate to inner's getTile
        // This is complex async logic - for now, require tile to be set

        // PORT: js/pseudos.ts:302-314 - check if already loaded
        if inner.value_loaded == Some(true) {
            return Ok(inner.value.clone().unwrap_or(JsValue::NULL));
        }

        // PORT: js/pseudos.ts:302 - mark as loading
        inner.value_loaded = None; // undefined in JS = loading

        // PORT: js/pseudos.ts:304-314 - extract data from tile
        let data = if inner.value.is_some() {
            // Use existing value
            inner.value.clone().unwrap_or(JsValue::NULL)
        } else if let (Some(tile), false) = (&inner.core.tile, inner.datatype() == "semantic") {
            // Get data from tile.data[nodeid]
            tile.data.get(&inner.core.node.nodeid)
                .map(|d| serde_wasm_bindgen::to_value(d).unwrap_or(JsValue::NULL))
                .unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        };

        // PORT: js/pseudos.ts:316-329 - handle outer/inner data splitting
        // For outer nodes with inner, check if data has "_" key for outer portion
        // This is handled in JS for now as it's complex object manipulation

        // Prepare arguments for callback
        let tile_js = match &inner.core.tile {
            Some(t) => {
                let wasm_tile = WasmStaticTile::from(t.as_ref().clone());
                JsValue::from(wasm_tile)
            },
            None => JsValue::NULL,
        };

        let node_js = inner.core.node.to_json();
        let node_js = serde_wasm_bindgen::to_value(&node_js).unwrap_or(JsValue::NULL);

        let is_inner_js = JsValue::from(inner.core.is_inner);

        // Drop borrow before async call
        drop(inner);

        // PORT: js/pseudos.ts:330-337 - call getViewModel
        // Returns a Promise that resolves to the ViewModel
        // Note: We pass tile, node, data, isInner - the JS caller already has 'this'
        let vm_promise = get_view_model.call4(
            &JsValue::NULL,
            &tile_js,      // tile
            &node_js,      // node
            &data,         // data
            &is_inner_js,  // isInner
        ).map_err(|e| e)?;

        // Store the promise as the value (JS will await it)
        self.inner.borrow_mut().value = Some(vm_promise.clone());

        Ok(vm_promise)
    }

    /// PORT: js/pseudos.ts:358-360 - getValue method
    /// Returns the value (or triggers updateValue if needed)
    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(&self, get_view_model: &js_sys::Function) -> Result<JsValue, JsValue> {
        self.update_value(get_view_model)
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
        if tile.is_null() || tile.is_undefined() {
            self.inner.borrow_mut().core.tile = None;
        } else {
            // Convert JsValue to StaticTile via serde deserialization
            if let Ok(core_tile) = serde_wasm_bindgen::from_value::<alizarin_core::StaticTile>(tile) {
                self.inner.borrow_mut().core.tile = Some(Arc::new(core_tile));
            }
        }
    }

    /// Set the tile data for this pseudo value.
    /// ViewModels should call this when their value changes so that
    /// Rust can serialize tiles without calling back to JS.
    ///
    /// The value should be the result of __asTileData() - the serializable
    /// form ready for the tile's data map.
    #[wasm_bindgen(js_name = setTileData)]
    pub fn set_tile_data(&self, value: JsValue) {
        let mut inner = self.inner.borrow_mut();
        if value.is_null() || value.is_undefined() {
            inner.core.tile_data = None;
        } else {
            // Convert JsValue to serde_json::Value for storage
            match serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
                Ok(json_value) => {
                    inner.core.tile_data = Some(json_value);
                }
                Err(e) => {
                    web_sys::console::warn_1(
                        &format!("setTileData: failed to convert value: {:?}", e).into()
                    );
                }
            }
        }
    }

    /// Get the current tile data (for debugging/inspection)
    #[wasm_bindgen(getter, js_name = tileDataJson)]
    pub fn tile_data_json(&self) -> JsValue {
        let inner = self.inner.borrow();
        match &inner.core.tile_data {
            Some(data) => {
                let json_str = serde_json::to_string(data).unwrap_or_default();
                js_sys::JSON::parse(&json_str).unwrap_or(JsValue::NULL)
            }
            None => JsValue::NULL,
        }
    }

    /// Check if this pseudo has tile data set
    #[wasm_bindgen(js_name = hasTileData)]
    pub fn has_tile_data(&self) -> bool {
        self.inner.borrow().core.tile_data.is_some()
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
        const ITERABLE_DATATYPES: &[&str] = &[
            "concept-list",
            "resource-instance-list",
            "domain-value-list"
        ];
        ITERABLE_DATATYPES.contains(&self.inner.borrow().core.node.datatype.as_str())
    }

    /// Get number of child nodes
    /// PORT: PseudoNode.size
    #[wasm_bindgen(getter, js_name = size)]
    pub fn size(&self) -> usize {
        self.inner.borrow().core.child_node_ids.len()
    }

    /// Get child node aliases as an array
    /// PORT: PseudoNode.childNodeAliases
    #[wasm_bindgen(getter, js_name = childNodeAliases)]
    pub fn child_node_aliases(&self) -> js_sys::Array {
        let arr = js_sys::Array::new();
        // child_node_ids contains node IDs, not aliases
        // We would need access to the node map to get aliases
        // For now, return the IDs (caller may need to look up aliases)
        for id in &self.inner.borrow().core.child_node_ids {
            arr.push(&JsValue::from_str(id));
        }
        arr
    }

    /// Get the node's alias
    /// PORT: PseudoNode.alias
    #[wasm_bindgen(getter, js_name = alias)]
    pub fn alias(&self) -> JsValue {
        match &self.inner.borrow().core.node.alias {
            Some(s) => JsValue::from_str(s),
            None => JsValue::from_str(""),
        }
    }

    /// Get the node's exportable flag
    #[wasm_bindgen(getter, js_name = exportable)]
    pub fn exportable(&self) -> bool {
        self.inner.borrow().core.node.exportable
    }

    /// Get the node's isrequired flag
    #[wasm_bindgen(getter, js_name = isrequired)]
    pub fn isrequired(&self) -> bool {
        self.inner.borrow().core.node.isrequired
    }

    /// Get the node's issearchable flag
    #[wasm_bindgen(getter, js_name = issearchable)]
    pub fn issearchable(&self) -> bool {
        self.inner.borrow().core.node.issearchable
    }

    /// Get the node's istopnode flag
    #[wasm_bindgen(getter, js_name = istopnode)]
    pub fn istopnode(&self) -> bool {
        self.inner.borrow().core.node.istopnode
    }

    /// Get the node's sortorder
    #[wasm_bindgen(getter, js_name = sortorder)]
    pub fn sortorder(&self) -> JsValue {
        match self.inner.borrow().core.node.sortorder {
            Some(n) => JsValue::from(n),
            None => JsValue::NULL,
        }
    }

    /// Get the node's config as a JS object
    #[wasm_bindgen(getter, js_name = config)]
    pub fn config(&self) -> JsValue {
        let obj = js_sys::Object::new();
        for (key, value) in &self.inner.borrow().core.node.config {
            let js_value = serde_wasm_bindgen::to_value(value).unwrap_or(JsValue::NULL);
            js_sys::Reflect::set(&obj, &JsValue::from_str(key), &js_value).ok();
        }
        obj.into()
    }

    /// Get the node's ontologyclass
    #[wasm_bindgen(getter, js_name = ontologyclass)]
    pub fn ontologyclass(&self) -> JsValue {
        match &self.inner.borrow().core.node.ontologyclass {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    /// Get the node's fieldname
    #[wasm_bindgen(getter, js_name = fieldname)]
    pub fn fieldname(&self) -> JsValue {
        match &self.inner.borrow().core.node.fieldname {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    /// Get the node's parentproperty
    #[wasm_bindgen(getter, js_name = parentproperty)]
    pub fn parentproperty(&self) -> JsValue {
        match &self.inner.borrow().core.node.parentproperty {
            Some(s) => JsValue::from_str(s),
            None => JsValue::NULL,
        }
    }

    /// Get the node's graph_id
    #[wasm_bindgen(getter, js_name = graphId)]
    pub fn graph_id(&self) -> String {
        self.inner.borrow().core.node.graph_id.clone()
    }

    /// Get the node's description
    #[wasm_bindgen(getter, js_name = description)]
    pub fn description(&self) -> JsValue {
        match &self.inner.borrow().core.node.description {
            Some(desc) => serde_wasm_bindgen::to_value(&desc.to_json()).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// Get the node's hascustomalias flag
    #[wasm_bindgen(getter, js_name = hascustomalias)]
    pub fn hascustomalias(&self) -> bool {
        self.inner.borrow().core.node.hascustomalias
    }

    /// Get all commonly-used properties in a single boundary crossing.
    /// This avoids the overhead of multiple individual property getters.
    /// Returns a plain JS object with all properties cached on the JS side.
    #[wasm_bindgen(js_name = toSnapshot)]
    pub fn to_snapshot(&self) -> JsValue {
        let inner = self.inner.borrow();
        let mut map = serde_json::Map::new();

        // Core identity
        map.insert("datatype".to_string(), serde_json::Value::String(inner.datatype().to_string()));
        map.insert("nodeId".to_string(), serde_json::Value::String(inner.core.node.nodeid.clone()));
        map.insert("name".to_string(), serde_json::Value::String(inner.core.node.name.clone()));

        // Node alias
        if let Some(ref alias) = inner.core.node.alias {
            map.insert("alias".to_string(), serde_json::Value::String(alias.to_string()));
        }

        // Nodegroup ID
        if let Some(ref ng_id) = inner.core.node.nodegroup_id {
            map.insert("nodegroupId".to_string(), serde_json::Value::String(ng_id.to_string()));
        }

        // Sortorder
        if let Some(so) = inner.core.node.sortorder {
            map.insert("sortorder".to_string(), serde_json::Value::Number(so.into()));
        }

        // Flags
        map.insert("isOuter".to_string(), serde_json::Value::Bool(inner.is_outer()));
        map.insert("isInner".to_string(), serde_json::Value::Bool(inner.core.is_inner));
        map.insert("isCollector".to_string(), serde_json::Value::Bool(inner.core.is_collector));
        map.insert("accessed".to_string(), serde_json::Value::Bool(inner.accessed));
        map.insert("independent".to_string(), serde_json::Value::Bool(inner.core.independent));
        map.insert("hasTile".to_string(), serde_json::Value::Bool(inner.has_tile()));

        // Tile data
        if let Some(ref tile_data) = inner.core.tile_data {
            if let Ok(json_val) = serde_json::to_value(tile_data) {
                map.insert("tileData".to_string(), json_val);
            }
        }

        // Tile ID
        if let Some(tile_id) = inner.tile_id() {
            map.insert("tileId".to_string(), serde_json::Value::String(tile_id));
        }

        // Value loaded state
        if let Some(vl) = inner.value_loaded {
            map.insert("valueLoaded".to_string(), serde_json::Value::Bool(vl));
        }

        // Convert to JS via JSON parse (single crossing)
        let json_string = serde_json::to_string(&serde_json::Value::Object(map)).unwrap_or_default();
        js_sys::JSON::parse(&json_string).unwrap_or(JsValue::NULL)
    }
}

// Non-WASM methods for internal Rust use
impl PseudoValue {
    /// Create a PseudoValue from PseudoValueInner (for Rust-internal use)
    pub(crate) fn from_rust(inner: PseudoValueInner) -> Self {
        PseudoValue { inner: RefCell::new(inner) }
    }

    /// Get the inner PseudoValueInner (for Rust-internal use)
    /// Phase 4g: Added for cache storage
    pub(crate) fn into_inner(self) -> PseudoValueInner {
        self.inner.into_inner()
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
    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(&self, value_index: usize) -> Option<PseudoValue> {
        self.inner.values
            .get(value_index)
            .map(|v| PseudoValue::from_rust(v.clone()))
    }

    /// Get all values from all groups as a flat array
    /// PORT: js/pseudos.ts:350 - map over array elements
    #[wasm_bindgen(js_name = getAllValues)]
    pub fn get_all_values(&self) -> Vec<PseudoValue> {
        self.inner.all_values()
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

    /// Get the inner PseudoListInner (for Rust-internal use)
    pub(crate) fn into_inner(self) -> PseudoListInner {
        self.inner
    }

    /// Get a reference to the inner PseudoListInner (for Rust-internal use)
    pub(crate) fn inner_ref(&self) -> &PseudoListInner {
        &self.inner
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
    pub fn get_value(&self, index: usize) -> Option<PseudoValue> {
        self.inner.values.get(index).map(|v| {
            PseudoValue::from_rust(v.clone())
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
        let pv = PseudoValueInner::from_node_and_tile(node.clone(), None, None, vec![]);

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
            PseudoValueInner::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            PseudoValueInner::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            PseudoValueInner::from_node_and_tile(node.clone(), Some(tile2.clone()), None, vec![]),
        ];

        let list = PseudoListInner::from_values("TestNode".to_string(), values);

        assert_eq!(list.node_alias, "TestNode");
        assert_eq!(list.values.len(), 3); // Three total values (grouped by tile internally)
        assert_eq!(list.all_values().len(), 3); // Three total values
        assert!(list.is_loaded);
    }

    #[test]
    fn test_pseudo_list_tile_filtering() {
        let node = create_test_node("TestNode");
        let tile1 = create_test_tile("tile1");

        let values = vec![
            PseudoValueInner::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            PseudoValueInner::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            PseudoValueInner::from_node_and_tile(node.clone(), None, None, vec![]),
        ];

        let list = PseudoListInner::from_values("TestNode".to_string(), values);

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

        let mut parent_value = PseudoValueInner::from_node_and_tile(
            parent_node.clone(),
            None,
            None,
            vec![],
        );

        parent_value.populate_child_ids(&edges);

        assert_eq!(parent_value.child_node_ids().len(), 2);
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

        let parent_value = PseudoValueInner::from_node_and_tile(
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

        let parent_value = PseudoValueInner::from_node_and_tile(
            parent_node.clone(),
            Some(tile.clone()),
            None,
            vec![child_node.nodeid.clone()],
        );

        let child = parent_value.get_child_by_alias("ChildNode", &node_objs, &edges).unwrap();

        assert!(child.tile_data().is_some());
        let child_data = child.tile_data().as_ref().unwrap();
        assert_eq!(child_data["value"], "child_data");
    }

    #[test]
    fn test_pseudo_list_merge() {
        let node = create_test_node("TestNode");
        let tile1 = create_test_tile("tile1");
        let tile2 = create_test_tile("tile2");

        let values1 = vec![
            PseudoValueInner::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
        ];
        let mut list1 = PseudoListInner::from_values("TestNode".to_string(), values1);

        let values2 = vec![
            PseudoValueInner::from_node_and_tile(node.clone(), Some(tile1.clone()), None, vec![]),
            PseudoValueInner::from_node_and_tile(node.clone(), Some(tile2.clone()), None, vec![]),
        ];
        let list2 = PseudoListInner::from_values("TestNode".to_string(), values2);

        list1.merge(list2);

        // After merge: 3 total values (1 from list1 for tile1, 2 from list2 for tile1 and tile2)
        assert_eq!(list1.values.len(), 3);

        // Count values per tile
        let tile1_count = list1.values.iter()
            .filter(|v| v.tile().as_ref().and_then(|t| t.tileid.as_deref()) == Some("tile1"))
            .count();
        assert_eq!(tile1_count, 2, "Should have 2 values for tile1");

        let tile2_count = list1.values.iter()
            .filter(|v| v.tile().as_ref().and_then(|t| t.tileid.as_deref()) == Some("tile2"))
            .count();
        assert_eq!(tile2_count, 1, "Should have 1 value for tile2");
    }
}
