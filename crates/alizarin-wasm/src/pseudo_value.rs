use std::sync::Arc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
// Use core types for internal storage (no WASM wrapper overhead)
use alizarin_core::{StaticNode, StaticTile, PseudoValueCore};
use alizarin_core::node_config::NodeConfigManager;
use alizarin_core::rdm_cache::RdmCache;
use alizarin_core::type_serialization::{
    SerializationOptions, SerializationContext,
};
// Import traits and generic functions for visitor pattern delegation
use alizarin_core::pseudo_value_core::{PseudoValueLike, PseudoListLike, to_json_generic};
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
        // Delegate to new() with no parent and is_inner=false
        // This ensures inner/outer pattern is properly applied
        Self::new(node, tile, tile_data, None, child_node_ids, false)
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
                value_loaded: Some(false),
                accessed: false,
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
        self.core.serialize_own_value(serialization_options, serialization_context)
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
    /// Create a new empty PseudoList (defaults to list, not single)
    pub fn new(node_alias: String) -> Self {
        PseudoListInner {
            node_alias,
            values: Vec::new(),
            is_loaded: false,
            is_single: false,
        }
    }

    /// Group values by their tiles
    ///
    /// PORT: js/pseudos.ts PseudoList grouping logic
    /// PORT: js/graphManager.ts lines 987-1011 - PseudoList merging
    pub fn from_values(node_alias: String, values: Vec<PseudoValueInner>) -> Self {
        // Sort values by sortorder at construction time for consistent ordering
        let mut sorted_values = values;
        sorted_values.sort_by_key(|v| {
            v.core.tile.as_ref()
                .and_then(|t| t.sortorder)
                .unwrap_or(i32::MAX)
        });
        PseudoListInner {
            node_alias,
            values: sorted_values,
            is_loaded: true,
            is_single: false,
        }
    }

    /// Create from values with explicit is_single flag
    pub fn from_values_with_cardinality(node_alias: String, values: Vec<PseudoValueInner>, is_single: bool) -> Self {
        // Sort values by sortorder at construction time for consistent ordering
        let mut sorted_values = values;
        sorted_values.sort_by_key(|v| {
            v.core.tile.as_ref()
                .and_then(|t| t.sortorder)
                .unwrap_or(i32::MAX)
        });
        PseudoListInner {
            node_alias,
            values: sorted_values,
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

    /// Get all values across
    pub fn all_values(&self) -> Vec<&PseudoValueInner> {
        self.values.iter()
            .collect()
    }

    pub fn matching_entries(
        &self,
        parent_tile_id: Option<String>,
        nodegroup_id: Option<String>,
        parent_nodegroup_id: Option<String>,
    ) -> Vec<&PseudoValueInner> {
        use alizarin_core::matches_tile_filter;
        let mut entries: Vec<&PseudoValueInner> = self.values.iter()
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
            v.core.tile.as_ref()
                .and_then(|t| t.sortorder)
                .unwrap_or(i32::MAX)
        });

        entries
    }
}

// ============================================================================
// Visitor pattern for JSON serialization
// ============================================================================

use std::collections::HashMap;

/// Visitor context for JSON serialization - reuses core's generic VisitorContext
pub(crate) type VisitorContext<'a> = alizarin_core::pseudo_value_core::VisitorContext<'a, PseudoListInner>;

impl PseudoValueInner {
    /// Convert this pseudo value to JSON using core's generic visitor.
    pub fn to_json(&self, ctx: &VisitorContext) -> serde_json::Value {
        to_json_generic(self, ctx)
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

        // Sort values by sortorder for consistent output
        let mut sorted_values: Vec<&PseudoValueInner> = self.values.iter().collect();
        sorted_values.sort_by_key(|v| {
            v.core.tile.as_ref()
                .and_then(|t| t.sortorder)
                .unwrap_or(i32::MAX)
        });

        if self.is_single {
            // Single value - use first sorted value
            if let Some(first) = sorted_values.first() {
                return first.to_json(ctx);
            }
            return serde_json::Value::Null;
        }

        // Multiple values - array (using sorted order)
        let arr: Vec<serde_json::Value> = sorted_values.iter()
            .map(|v| v.to_json(ctx))
            .filter(|v| !v.is_null())
            .collect();

        if arr.is_empty() {
            serde_json::Value::Null
        } else if arr.len() == 1 {
            // If there's exactly one value and it's already an array, return it directly.
            // This handles list datatypes (reference, file-list, etc.) where all values
            // are stored as an array in a single tile slot rather than separate tiles.
            if let Some(first) = arr.first() {
                if first.is_array() {
                    return first.clone();
                }
            }
            serde_json::Value::Array(arr)
        } else {
            serde_json::Value::Array(arr)
        }
    }
}

// ============================================================================
// Display JSON Visitor - returns display values instead of tile_data
// ============================================================================

/// Context for display JSON traversal
///
/// Includes references to RDM cache and node config manager for resolving
/// UUIDs to display values.
pub(crate) struct DisplayVisitorContext<'a> {
    /// The pseudo_cache from the instance wrapper (alias -> PseudoListInner)
    pub pseudo_cache: &'a HashMap<String, PseudoListInner>,
    /// Node alias to node mapping from the model
    pub nodes_by_alias: &'a HashMap<String, Arc<StaticNode>>,
    /// Node edges from the model (parent_nodeid -> child_nodeids)
    pub edges: &'a HashMap<String, Vec<String>>,
    /// RDM cache for concept lookups
    pub rdm_cache: Option<&'a RdmCache>,
    /// Node config manager for domain value lookups
    pub node_config_manager: Option<&'a NodeConfigManager>,
    /// Language for display labels
    pub language: &'a str,
    /// Current traversal depth (for preventing infinite recursion)
    pub depth: usize,
    /// Maximum depth
    pub max_depth: usize,
    /// Extension display serializer registry for custom datatypes
    pub display_registry: Option<&'a alizarin_core::DisplaySerializerRegistry>,
}

impl PseudoValueInner {
    /// Convert this pseudo value to display JSON.
    ///
    /// Unlike `to_json()` which returns tile_data (UUIDs) for leaf nodes,
    /// this method resolves UUIDs to display values:
    /// - domain-value: looks up label from node config
    /// - concept/concept-list: looks up label from RDM cache
    /// - reference (CLM): already contains StaticReference, extracts display string
    /// - Other types: returns tile_data as-is
    pub fn to_display_json(&self, ctx: &DisplayVisitorContext) -> serde_json::Value {
        if ctx.depth > ctx.max_depth {
            return serde_json::Value::Null;
        }

        // Handle outer nodes with inner
        if self.is_outer() {
            return self.outer_to_display_json(ctx);
        }

        // Handle inner nodes (synthetic semantic)
        if self.core.is_inner {
            return self.semantic_to_display_json(ctx);
        }

        let datatype = self.datatype();

        // Pure semantic nodes - traverse children
        if datatype == "semantic" {
            return self.semantic_to_display_json(ctx);
        }

        // Leaf nodes - resolve to display value based on datatype
        self.leaf_to_display_json(ctx, datatype)
    }

    /// Convert a leaf node to display JSON
    fn leaf_to_display_json(&self, ctx: &DisplayVisitorContext, datatype: &str) -> serde_json::Value {
        let tile_data = match &self.core.tile_data {
            Some(data) => data,
            None => return serde_json::Value::Null,
        };

        // Check extension registry first for custom datatypes
        if let Some(registry) = ctx.display_registry {
            if let Some(serializer) = registry.get(datatype) {
                let options = alizarin_core::SerializationOptions::display(ctx.language);
                let result = serializer.serialize_display(tile_data, &options);
                if !result.is_error() {
                    return result.value;
                }
            }
        }

        match datatype {
            // Domain value - lookup from node config
            "domain-value" => {
                self.resolve_domain_value(ctx, tile_data)
            }

            // Domain value list - lookup each from node config
            "domain-value-list" => {
                self.resolve_domain_value_list(ctx, tile_data)
            }

            // Concept value - lookup from RDM cache
            "concept" | "concept-value" => {
                self.resolve_concept_value(ctx, tile_data)
            }

            // Concept list - lookup each from RDM cache
            "concept-list" => {
                self.resolve_concept_list(ctx, tile_data)
            }

            // Boolean with labels
            "boolean" => {
                self.resolve_boolean_value(ctx, tile_data)
            }

            // String - extract display value from language map
            "string" => {
                use alizarin_core::type_serialization::serialize_string;
                let options = alizarin_core::SerializationOptions::display(ctx.language);
                let result = serialize_string(tile_data, &options);
                result.unwrap_or(tile_data.clone())
            }

            // URL - extract display value
            "url" => {
                use alizarin_core::type_serialization::serialize_url;
                let options = alizarin_core::SerializationOptions::display(ctx.language);
                let result = serialize_url(tile_data, &options);
                result.unwrap_or(tile_data.clone())
            }

            // Other types - return tile_data as-is
            _ => tile_data.clone(),
        }
    }

    /// Resolve a domain value UUID to its display label
    fn resolve_domain_value(&self, ctx: &DisplayVisitorContext, tile_data: &serde_json::Value) -> serde_json::Value {
        let uuid = match tile_data.as_str() {
            Some(s) => s,
            None => return tile_data.clone(),
        };

        if let Some(config_mgr) = ctx.node_config_manager {
            if let Some(domain_value) = config_mgr.lookup_domain_value(&self.core.node.nodeid, uuid) {
                // Return the label for the requested language
                if let Some(label) = domain_value.lang(ctx.language) {
                    return serde_json::Value::String(label.to_string());
                }
                // Fall back to display()
                return serde_json::Value::String(domain_value.display().to_string());
            }
        }

        // Could not resolve - return original
        tile_data.clone()
    }

    /// Resolve a domain value list to display labels
    fn resolve_domain_value_list(&self, ctx: &DisplayVisitorContext, tile_data: &serde_json::Value) -> serde_json::Value {
        let arr = match tile_data.as_array() {
            Some(a) => a,
            None => return tile_data.clone(),
        };

        let resolved: Vec<serde_json::Value> = arr.iter()
            .map(|item| self.resolve_domain_value(ctx, item))
            .collect();

        serde_json::Value::Array(resolved)
    }

    /// Resolve a concept UUID to its display label using RDM cache
    fn resolve_concept_value(&self, ctx: &DisplayVisitorContext, tile_data: &serde_json::Value) -> serde_json::Value {
        // tile_data could be a UUID string or a concept object with "id" field
        let uuid = if let Some(s) = tile_data.as_str() {
            s.to_string()
        } else if let Some(obj) = tile_data.as_object() {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                id.to_string()
            } else {
                return tile_data.clone();
            }
        } else {
            return tile_data.clone();
        };

        // Get collection ID from node config
        let collection_id = if let Some(config_mgr) = ctx.node_config_manager {
            if let Some(concept_config) = config_mgr.get_concept(&self.core.node.nodeid) {
                Some(concept_config.rdm_collection.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Look up in RDM cache
        if let (Some(rdm_cache), Some(collection_id)) = (ctx.rdm_cache, collection_id) {
            if let Some(label) = rdm_cache.lookup_label(&collection_id, &uuid, ctx.language) {
                return serde_json::Value::String(label);
            }
        }

        // Could not resolve - return original
        tile_data.clone()
    }

    /// Resolve a concept list to display labels
    fn resolve_concept_list(&self, ctx: &DisplayVisitorContext, tile_data: &serde_json::Value) -> serde_json::Value {
        let arr = match tile_data.as_array() {
            Some(a) => a,
            None => return tile_data.clone(),
        };

        let resolved: Vec<serde_json::Value> = arr.iter()
            .map(|item| self.resolve_concept_value(ctx, item))
            .collect();

        serde_json::Value::Array(resolved)
    }

    /// Resolve a boolean value to its display label
    fn resolve_boolean_value(&self, ctx: &DisplayVisitorContext, tile_data: &serde_json::Value) -> serde_json::Value {
        let bool_value = match tile_data.as_bool() {
            Some(b) => b,
            None => return tile_data.clone(),
        };

        if let Some(config_mgr) = ctx.node_config_manager {
            if let Some(bool_config) = config_mgr.get_boolean(&self.core.node.nodeid) {
                if let Some(label) = bool_config.get_label(bool_value, ctx.language) {
                    return serde_json::Value::String(label.to_string());
                }
            }
        }

        // Fall back to true/false string
        serde_json::Value::String(if bool_value { "true" } else { "false" }.to_string())
    }

    /// Convert an outer node to display JSON
    fn outer_to_display_json(&self, ctx: &DisplayVisitorContext) -> serde_json::Value {
        let datatype = self.datatype();
        let own_value = self.leaf_to_display_json(ctx, datatype);

        if let Some(ref inner) = self.inner {
            let children_json = inner.semantic_to_display_json(ctx);
            // Use display serialization options for consistent merge behavior
            let display_opts = SerializationOptions::display(ctx.language);
            return display_opts.merge_outer_with_children(own_value, children_json);
        }

        own_value
    }

    /// Convert a semantic node to display JSON by visiting its children
    fn semantic_to_display_json(&self, ctx: &DisplayVisitorContext) -> serde_json::Value {
        let mut obj = serde_json::Map::new();

        let parent_tile_id = self.core.tile.as_ref().and_then(|t| t.tileid.clone());
        let parent_nodegroup_id = self.core.node.nodegroup_id.clone();

        let child_node_ids = match ctx.edges.get(&self.core.node.nodeid) {
            Some(ids) => ids,
            None => return serde_json::Value::Object(obj),
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
                parent_nodegroup_id.clone(),
            );

            if matching_values.is_empty() {
                continue;
            }

            let child_ctx = DisplayVisitorContext {
                pseudo_cache: ctx.pseudo_cache,
                nodes_by_alias: ctx.nodes_by_alias,
                edges: ctx.edges,
                rdm_cache: ctx.rdm_cache,
                node_config_manager: ctx.node_config_manager,
                language: ctx.language,
                depth: ctx.depth + 1,
                max_depth: ctx.max_depth,
                display_registry: ctx.display_registry,
            };

            // Helper to check if a value is "empty" for display purposes
            // Filters null and empty objects (needed for outer node flattening)
            fn is_empty_for_flattening(v: &serde_json::Value) -> bool {
                match v {
                    serde_json::Value::Null => true,
                    serde_json::Value::Object(m) => m.is_empty(),
                    _ => false,
                }
            }

            // Only unwrap to single value for cardinality-1 nodes (is_single=true).
            // For cardinality-n nodes, always return an array even with 1 item,
            // otherwise Handlebars {{#each}} iterates over object properties instead of items.
            if pseudo_list.is_single {
                if let Some(first_value) = matching_values.first() {
                    let json_value = first_value.to_display_json(&child_ctx);
                    if !is_empty_for_flattening(&json_value) {
                        obj.insert(child_alias.clone(), json_value);
                    }
                }
            } else {
                let arr: Vec<serde_json::Value> = matching_values.iter()
                    .map(|v| v.to_display_json(&child_ctx))
                    .filter(|v| !is_empty_for_flattening(v))
                    .collect();

                if !arr.is_empty() {
                    // If there's exactly one value and it's already an array, insert it directly.
                    // This handles list datatypes (reference, file-list, etc.) where all values
                    // are stored as an array in a single tile slot rather than separate tiles.
                    if arr.len() == 1 {
                        if let Some(first) = arr.first() {
                            if first.is_array() {
                                obj.insert(child_alias.clone(), first.clone());
                                continue;
                            }
                        }
                    }
                    obj.insert(child_alias.clone(), serde_json::Value::Array(arr));
                }
            }
        }

        serde_json::Value::Object(obj)
    }
}

impl PseudoListInner {
    /// Convert this pseudo list to display JSON
    ///
    /// For single-cardinality nodes, returns the single value's display JSON.
    /// For multi-cardinality (collector) nodes, returns an array.
    pub fn to_display_json(&self, ctx: &DisplayVisitorContext) -> serde_json::Value {
        if self.values.is_empty() {
            return serde_json::Value::Null;
        }

        // Sort values by sortorder for consistent output
        let mut sorted_values: Vec<&PseudoValueInner> = self.values.iter().collect();
        sorted_values.sort_by_key(|v| {
            v.core.tile.as_ref()
                .and_then(|t| t.sortorder)
                .unwrap_or(i32::MAX)
        });

        if self.is_single {
            // Single value - use first sorted value
            if let Some(first) = sorted_values.first() {
                return first.to_display_json(ctx);
            }
            return serde_json::Value::Null;
        }

        // Multiple values - array (using sorted order)
        let arr: Vec<serde_json::Value> = sorted_values.iter()
            .map(|v| v.to_display_json(ctx))
            .filter(|v| !v.is_null())
            .collect();

        if arr.is_empty() {
            serde_json::Value::Null
        } else if arr.len() == 1 {
            // If there's exactly one value and it's already an array, return it directly.
            // This handles list datatypes (reference, file-list, etc.) where all values
            // are stored as an array in a single tile slot rather than separate tiles.
            if let Some(first) = arr.first() {
                if first.is_array() {
                    return first.clone();
                }
            }
            serde_json::Value::Array(arr)
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
        let parent_nodegroup_id = self.core.node.nodegroup_id.clone();

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
                parent_nodegroup_id.clone(),
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

}
