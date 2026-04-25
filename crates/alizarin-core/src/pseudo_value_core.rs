use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
/// Core PseudoValue types for JSON conversion
///
/// These types contain the platform-agnostic parts of pseudo values
/// that can be used from any binding (WASM, Python, etc.)
///
/// The WASM bindings extend these with JsValue fields for ViewModel integration.
use std::sync::Arc;

use crate::node_config::NodeConfigManager;
use crate::type_serialization::{
    serialize_value, ExternalResolver, ResourceDisplayResolver, SerializationContext,
    SerializationMode, SerializationOptions,
};
use crate::{StaticNode, StaticTile};

// =============================================================================
// Traits for abstracting over PseudoValue and PseudoList implementations
// =============================================================================

/// Trait for PseudoValue-like types (core, WASM, Python)
/// Defines the operations needed for visitor pattern traversal
pub trait PseudoValueLike {
    /// Get the tile reference
    fn tile(&self) -> &Option<Arc<StaticTile>>;

    /// Get the node reference
    fn node(&self) -> &Arc<StaticNode>;

    /// Check if this is an inner value
    fn is_inner(&self) -> bool;

    /// Check if this is an outer value (has inner)
    fn is_outer(&self) -> bool;

    /// Get the effective datatype
    fn datatype(&self) -> &str;

    /// Serialize this value's own tile_data
    fn serialize_own_value(
        &self,
        serialization_options: &SerializationOptions,
        serialization_context: Option<&SerializationContext>,
    ) -> Value;

    /// Check if this node has children in the graph
    fn has_children(&self, edges: &HashMap<String, Vec<String>>) -> bool;

    /// Get the inner value (for outer/inner pattern)
    fn inner(&self) -> Option<&Self>;
}

/// Trait for PseudoList-like types (core, WASM, Python)
/// Defines the operations needed for visitor pattern traversal
pub trait PseudoListLike {
    /// The value type this list contains
    type Value: PseudoValueLike;

    /// Get matching entries based on tile filtering
    fn matching_entries(
        &self,
        parent_tile_id: Option<String>,
        nodegroup_id: Option<String>,
        parent_nodegroup_id: Option<String>,
    ) -> Vec<&Self::Value>;

    /// Check if this is a single-cardinality list (unwraps to single value)
    fn is_single(&self) -> bool;

    /// Get the number of values in this list (for debug logging)
    fn values_count(&self) -> usize;
}

/// Core pseudo value without platform-specific fields
///
/// Contains only the fields needed for:
/// - JSON conversion (tiles_to_tree, tree_to_tiles)
/// - Tree traversal
/// - Tile building
#[derive(Clone, Debug)]
pub struct PseudoValueCore {
    /// The graph node this value represents
    pub node: Arc<StaticNode>,

    /// Child node IDs (metadata) - NOT instantiated children
    pub child_node_ids: Vec<String>,

    /// Whether this is a collector node (has multiple tiles)
    pub is_collector: bool,

    /// Inner value for outer/inner pattern (semantic nodes)
    pub inner: Option<Box<PseudoValueCore>>,

    /// Whether this IS the inner (datatype overridden to "semantic")
    pub is_inner: bool,

    /// Optional tile containing the data
    pub tile: Option<Arc<StaticTile>>,

    /// Raw tile data (unparsed, as serde_json::Value)
    pub tile_data: Option<Value>,

    /// Whether this tile is independent (not inherited from parent)
    pub independent: bool,

    /// Original tile reference (before any modifications)
    pub original_tile: Option<Arc<StaticTile>>,
}

/// Core pseudo list type
#[derive(Clone, Debug)]
pub struct PseudoListCore {
    /// The node alias this list represents
    pub node_alias: String,

    /// Grouped values by tile
    pub values: Vec<PseudoValueCore>,

    /// Whether this list has been fully loaded
    pub is_loaded: bool,

    /// Whether this should be unwrapped to a single value when returned
    pub is_single: bool,
}

/// Context passed to visitors during traversal
///
/// Generic over `L: PseudoListLike` to support both core and WASM types
pub struct VisitorContext<'a, L: PseudoListLike> {
    /// The pseudo_cache from the instance wrapper
    pub pseudo_cache: &'a HashMap<String, L>,
    /// Node alias to node mapping from the model
    pub nodes_by_alias: &'a HashMap<String, Arc<StaticNode>>,
    /// Node edges from the model (parent_nodeid -> child_nodeids)
    pub edges: &'a HashMap<String, Vec<String>>,
    /// Current traversal depth (for preventing infinite recursion)
    pub depth: usize,
    /// Maximum depth
    pub max_depth: usize,
    /// Serialization options (controls output format)
    pub serialization_options: SerializationOptions,
    /// Serialization context (shared external_resolver + extension_registry; node_config is None at tree level)
    pub serialization_context: SerializationContext<'a>,
    /// Node config manager for per-node config lookups at leaf serialization
    pub node_config_manager: Option<&'a NodeConfigManager>,
}

/// Type alias for core VisitorContext (for backward compatibility)
pub type VisitorContextCore<'a> = VisitorContext<'a, PseudoListCore>;

impl<'a, L: PseudoListLike> VisitorContext<'a, L> {
    /// Create a new VisitorContext with default tile_data serialization
    pub fn new(
        pseudo_cache: &'a HashMap<String, L>,
        nodes_by_alias: &'a HashMap<String, Arc<StaticNode>>,
        edges: &'a HashMap<String, Vec<String>>,
    ) -> Self {
        VisitorContext {
            pseudo_cache,
            nodes_by_alias,
            edges,
            depth: 0,
            max_depth: 50,
            serialization_options: SerializationOptions::tile_data(),
            serialization_context: SerializationContext::empty(),
            node_config_manager: None,
        }
    }

    /// Create a VisitorContext for display mode
    pub fn display(
        pseudo_cache: &'a HashMap<String, L>,
        nodes_by_alias: &'a HashMap<String, Arc<StaticNode>>,
        edges: &'a HashMap<String, Vec<String>>,
        language: &str,
    ) -> Self {
        VisitorContext {
            pseudo_cache,
            nodes_by_alias,
            edges,
            depth: 0,
            max_depth: 50,
            serialization_options: SerializationOptions::display(language),
            serialization_context: SerializationContext::empty(),
            node_config_manager: None,
        }
    }

    /// Create a child context with incremented depth.
    ///
    /// Propagates external_resolver, extension_registry, and node_config_manager.
    /// node_config is reset to None (set per-node at leaf serialization).
    pub fn child(&self) -> Self {
        VisitorContext {
            pseudo_cache: self.pseudo_cache,
            nodes_by_alias: self.nodes_by_alias,
            edges: self.edges,
            depth: self.depth + 1,
            max_depth: self.max_depth,
            serialization_options: self.serialization_options.clone(),
            serialization_context: self.serialization_context.with_node_config(None),
            node_config_manager: self.node_config_manager,
        }
    }

    /// Check if we're in display mode
    pub fn is_display(&self) -> bool {
        self.serialization_options.mode == SerializationMode::Display
    }
}

/// A tile being built - accumulates data from multiple leaf nodes
#[derive(Debug, Clone)]
pub struct TileBuilder {
    pub tileid: Option<String>,
    pub nodegroup_id: String,
    pub parenttile_id: Option<String>,
    pub resourceinstance_id: String,
    pub sortorder: Option<i32>,
    pub data: HashMap<String, Value>,
}

/// Context for tile building
pub struct TileBuilderContext<'a> {
    pub pseudo_cache: &'a HashMap<String, PseudoListCore>,
    pub nodes_by_alias: &'a HashMap<String, Arc<StaticNode>>,
    pub edges: &'a HashMap<String, Vec<String>>,
    pub resourceinstance_id: String,
    pub depth: usize,
    pub max_depth: usize,
    /// Track visited aliases to prevent O(n²) duplicate traversal
    pub visited_aliases: &'a std::cell::RefCell<HashSet<String>>,
}

impl PseudoValueCore {
    /// Create a new PseudoValueCore from a node and optional tile.
    ///
    /// Handles inner/outer split: if the node has children and is not semantic,
    /// an inner PseudoValueCore is created to hold the child traversal path.
    /// The outer gets the tile_data; the inner gets the child_node_ids.
    pub fn from_node_and_tile(
        node: Arc<StaticNode>,
        tile: Option<Arc<StaticTile>>,
        tile_data: Option<Value>,
        child_node_ids: Vec<String>,
    ) -> Self {
        let independent = tile.is_none();

        let has_children = !child_node_ids.is_empty();
        let is_semantic = node.datatype == "semantic";
        let should_have_inner = has_children && !is_semantic;

        let inner = if should_have_inner {
            Some(Box::new(PseudoValueCore {
                node: node.clone(),
                child_node_ids: child_node_ids.clone(),
                is_collector: node.is_collector,
                inner: None,
                is_inner: true,
                tile: tile.clone(),
                tile_data: None, // Inner doesn't get tile_data
                independent,
                original_tile: tile.clone(),
            }))
        } else {
            None
        };

        PseudoValueCore {
            node,
            // Outer gets empty child_node_ids if it has inner
            // (children traverse through inner instead)
            child_node_ids: if inner.is_some() {
                vec![]
            } else {
                child_node_ids
            },
            is_collector: false,
            inner,
            is_inner: false,
            tile: tile.clone(),
            tile_data,
            independent,
            original_tile: tile,
        }
    }

    /// Check if this is an outer node (has inner)
    pub fn is_outer(&self) -> bool {
        self.inner.is_some()
    }

    /// Get effective datatype (overridden to "semantic" for inner nodes)
    pub fn datatype(&self) -> &str {
        if self.is_inner {
            "semantic"
        } else {
            &self.node.datatype
        }
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

    /// Check if this node has children in the graph edges
    pub fn has_children(&self, edges: &HashMap<String, Vec<String>>) -> bool {
        edges
            .get(&self.node.nodeid)
            .map(|ids| !ids.is_empty())
            .unwrap_or(false)
    }

    /// Serialize this node's own value (tile_data) using type_serialization
    ///
    /// This is the common logic used by outer_to_json, non_semantic_with_children_to_json,
    /// and leaf node serialization. Extracted to allow delegation from WASM.
    ///
    /// Takes serialization options and context directly to allow use from different
    /// VisitorContext types (core vs WASM).
    pub fn serialize_own_value(
        &self,
        serialization_options: &SerializationOptions,
        serialization_context: Option<&SerializationContext>,
    ) -> Value {
        if let Some(ref data) = self.tile_data {
            let result = serialize_value(
                self.datatype(),
                data,
                serialization_options,
                serialization_context,
            );
            result.unwrap_or(Value::Null)
        } else {
            Value::Null
        }
    }

    /// Serialize this value in display mode without tree traversal.
    ///
    /// Resolves UUIDs to labels, extracts display strings, applies domain lookups
    /// for this node's own tile_data only (no children).
    pub fn serialize_display(
        &self,
        language: &str,
        node_config_manager: Option<&NodeConfigManager>,
        external_resolver: Option<&dyn ExternalResolver>,
        resource_resolver: Option<&dyn ResourceDisplayResolver>,
    ) -> Value {
        let opts = SerializationOptions::display(language);
        let node_config = node_config_manager.and_then(|ncm| ncm.get(&self.node.nodeid));
        let ser_ctx = SerializationContext {
            node_config,
            external_resolver,
            resource_resolver,
            extension_registry: None,
        };
        self.serialize_own_value(&opts, Some(&ser_ctx))
    }

    /// Convert this pseudo value to JSON
    ///
    /// Uses the serialization options in the context to determine output format:
    /// - TileData mode: returns raw tile_data (UUIDs, language maps)
    /// - Display mode: resolves UUIDs to labels, extracts display strings
    ///
    /// Delegates to the generic to_json_generic function.
    pub fn to_json(&self, ctx: &VisitorContext<PseudoListCore>) -> Value {
        to_json_generic(self, ctx)
    }

    /// Collect tiles from this pseudo value
    pub fn collect_tiles(
        &self,
        ctx: &TileBuilderContext,
        tiles: &mut HashMap<String, TileBuilder>,
    ) {
        if ctx.depth > ctx.max_depth {
            return;
        }

        if let Some(ref tile) = self.tile {
            let tile_key = tile.tileid.clone().unwrap_or_else(|| {
                format!("new_{}_{}", tile.nodegroup_id, tile.sortorder.unwrap_or(0))
            });

            if !tiles.contains_key(&tile_key) {
                tiles.insert(tile_key.clone(), TileBuilder::from_tile(tile));
            }

            let datatype = self.datatype();
            if datatype != "semantic" && !self.is_inner {
                if let Some(ref data) = self.tile_data {
                    if let Some(tile_builder) = tiles.get_mut(&tile_key) {
                        tile_builder
                            .data
                            .insert(self.node.nodeid.clone(), data.clone());
                    }
                }
            }
        }

        // Collect children tiles for semantic nodes, inner nodes, or any node with children
        // This handles "outer" nodes (non-semantic with children like file-list with copyright)
        // Uses visited_aliases to prevent O(n²) duplicate traversal
        let has_children = ctx
            .edges
            .get(&self.node.nodeid)
            .map(|ids| !ids.is_empty())
            .unwrap_or(false);
        if self.datatype() == "semantic" || self.is_inner || has_children {
            self.collect_children_tiles(ctx, tiles);
        }

        if let Some(ref inner) = self.inner {
            let child_ctx = TileBuilderContext {
                pseudo_cache: ctx.pseudo_cache,
                nodes_by_alias: ctx.nodes_by_alias,
                edges: ctx.edges,
                resourceinstance_id: ctx.resourceinstance_id.clone(),
                depth: ctx.depth + 1,
                max_depth: ctx.max_depth,
                visited_aliases: ctx.visited_aliases,
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
        let child_node_ids = match ctx.edges.get(&self.node.nodeid) {
            Some(ids) => ids,
            None => return,
        };

        for child_node_id in child_node_ids {
            let child_node = ctx
                .nodes_by_alias
                .values()
                .find(|n| &n.nodeid == child_node_id);

            let child_node = match child_node {
                Some(n) => n,
                None => continue,
            };

            let child_alias = match &child_node.alias {
                Some(alias) if !alias.is_empty() => alias,
                _ => continue,
            };

            // Skip if already visited (prevents O(n²) duplicate traversal)
            {
                let visited = ctx.visited_aliases.borrow();
                if visited.contains(child_alias) {
                    continue;
                }
            }
            // Mark as visited before processing
            ctx.visited_aliases.borrow_mut().insert(child_alias.clone());

            if let Some(pseudo_list) = ctx.pseudo_cache.get(child_alias) {
                let child_ctx = TileBuilderContext {
                    pseudo_cache: ctx.pseudo_cache,
                    nodes_by_alias: ctx.nodes_by_alias,
                    edges: ctx.edges,
                    resourceinstance_id: ctx.resourceinstance_id.clone(),
                    depth: ctx.depth + 1,
                    max_depth: ctx.max_depth,
                    visited_aliases: ctx.visited_aliases,
                };
                pseudo_list.collect_tiles(&child_ctx, tiles);
            }
        }
    }
}

impl PseudoListCore {
    /// Create a new empty PseudoListCore
    pub fn new(node_alias: String) -> Self {
        PseudoListCore {
            node_alias,
            values: Vec::new(),
            is_loaded: false,
            is_single: false,
        }
    }

    /// Create a new empty PseudoListCore with explicit is_single flag
    pub fn new_with_cardinality(node_alias: String, is_single: bool) -> Self {
        PseudoListCore {
            node_alias,
            values: Vec::new(),
            is_loaded: false,
            is_single,
        }
    }

    /// Create from values
    pub fn from_values(node_alias: String, values: Vec<PseudoValueCore>) -> Self {
        PseudoListCore {
            node_alias,
            values,
            is_loaded: true,
            is_single: false,
        }
    }

    /// Create from values with explicit is_single flag
    pub fn from_values_with_cardinality(
        node_alias: String,
        values: Vec<PseudoValueCore>,
        is_single: bool,
    ) -> Self {
        PseudoListCore {
            node_alias,
            values,
            is_loaded: true,
            is_single,
        }
    }

    /// Merge another PseudoListCore into this one
    pub fn merge(&mut self, mut other: PseudoListCore) {
        self.values.append(&mut other.values);
    }

    /// Get matching entries based on tile relationship
    ///
    /// Entries match if:
    /// 1. They have a tile that passes the tile filter, OR
    /// 2. They have no tile (synthetic entries) and parent_tile_id is None
    ///    (root-level match for synthetic parent collectors)
    ///
    /// The parent_nodegroup_id is used to determine if the child is in the same
    /// nodegroup as the parent, which affects fallback matching behavior.
    pub fn matching_entries(
        &self,
        parent_tile_id: Option<String>,
        nodegroup_id: Option<String>,
        parent_nodegroup_id: Option<String>,
    ) -> Vec<&PseudoValueCore> {
        let mut entries: Vec<&PseudoValueCore> = self
            .values
            .iter()
            .filter(|v| {
                match v.tile.as_ref() {
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
            v.tile
                .as_ref()
                .and_then(|t| t.sortorder)
                .unwrap_or(i32::MAX)
        });

        entries
    }

    /// Convert to JSON
    pub fn to_json(&self, ctx: &VisitorContext<PseudoListCore>) -> Value {
        if self.values.is_empty() {
            return Value::Null;
        }

        // Sort values by sortorder for consistent output
        let mut sorted_values: Vec<&PseudoValueCore> = self.values.iter().collect();
        sorted_values.sort_by_key(|v| {
            v.tile
                .as_ref()
                .and_then(|t| t.sortorder)
                .unwrap_or(i32::MAX)
        });

        if self.is_single {
            if let Some(first) = sorted_values.first() {
                return to_json_generic(*first, ctx);
            }
            return Value::Null;
        }

        let arr: Vec<Value> = sorted_values
            .iter()
            .map(|v| to_json_generic(*v, ctx))
            .filter(|v| !v.is_null())
            .collect();

        if arr.is_empty() {
            Value::Null
        } else {
            Value::Array(arr)
        }
    }

    /// Collect tiles from all values
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

// =============================================================================
// Trait implementations for core types
// =============================================================================

impl PseudoValueLike for PseudoValueCore {
    fn tile(&self) -> &Option<Arc<StaticTile>> {
        &self.tile
    }

    fn node(&self) -> &Arc<StaticNode> {
        &self.node
    }

    fn is_inner(&self) -> bool {
        self.is_inner
    }

    fn is_outer(&self) -> bool {
        self.is_outer()
    }

    fn datatype(&self) -> &str {
        self.datatype()
    }

    fn serialize_own_value(
        &self,
        serialization_options: &SerializationOptions,
        serialization_context: Option<&SerializationContext>,
    ) -> Value {
        self.serialize_own_value(serialization_options, serialization_context)
    }

    fn has_children(&self, edges: &HashMap<String, Vec<String>>) -> bool {
        self.has_children(edges)
    }

    fn inner(&self) -> Option<&Self> {
        self.inner.as_ref().map(|b| b.as_ref())
    }
}

impl PseudoListLike for PseudoListCore {
    type Value = PseudoValueCore;

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

impl TileBuilder {
    /// Create from an existing StaticTile
    pub fn from_tile(tile: &StaticTile) -> Self {
        TileBuilder {
            tileid: tile.tileid.clone(),
            nodegroup_id: tile.nodegroup_id.clone(),
            parenttile_id: tile.parenttile_id.clone(),
            resourceinstance_id: tile.resourceinstance_id.clone(),
            sortorder: tile.sortorder,
            data: HashMap::new(),
        }
    }

    /// Convert to StaticTile
    pub fn to_static_tile(&self) -> StaticTile {
        let mut tile = StaticTile::new_empty(self.nodegroup_id.clone());
        tile.tileid = self.tileid.clone();
        tile.parenttile_id = self.parenttile_id.clone();
        tile.resourceinstance_id = self.resourceinstance_id.clone();
        tile.sortorder = self.sortorder;
        tile.data = self.data.clone();
        tile
    }
}

/// Check if a tile matches the parent tile filter for matching_entries
///
/// This predicate is used by `matching_entries` in both WASM and Core
/// to filter tiles based on their relationship to a parent tile.
///
/// Returns true if:
/// 1. Tile's nodegroup matches the expected nodegroup AND
/// 2. One of:
///    - Tile's tileid equals parent_tile_id (same tile)
///    - Tile's parenttile_id equals parent_tile_id (child tile)
///    - Both parent_tile_id and tile's parenttile_id are None (root level)
///    - Tile's parenttile_id is None and child is in different nodegroup than parent (fallback)
pub fn matches_tile_filter(
    tile: &StaticTile,
    parent_tile_id: Option<&String>,
    nodegroup_id: Option<&String>,
    parent_nodegroup_id: Option<&String>,
) -> bool {
    if let Some(ng) = nodegroup_id {
        if &tile.nodegroup_id == ng {
            // Tile's tileid equals parent_tile_id (same tile)
            if tile.tileid.as_ref() == parent_tile_id {
                return true;
            }
            // Tile's parenttile_id equals parent_tile_id (child tile)
            if tile.parenttile_id.is_some() && tile.parenttile_id.as_ref() == parent_tile_id {
                return true;
            }
            // Both are None (root level)
            if parent_tile_id.is_none() && tile.parenttile_id.is_none() {
                return true;
            }
            // Fallback: If tile has no parenttile_id but is in the correct nodegroup,
            // allow matching. This handles business data that doesn't set parenttile_id
            // on child tiles.
            // IMPORTANT: Only apply this fallback when child is in a DIFFERENT nodegroup
            // than the parent. When same nodegroup, we need exact tileid match (handled above).
            let is_different_nodegroup = match parent_nodegroup_id {
                Some(parent_ng) => ng != parent_ng,
                // When parent_nodegroup_id is not provided but parent_tile_id IS provided,
                // we can't confirm different nodegroup, so don't use fallback.
                // (Root level case where both are None is handled above at line 604)
                None => false,
            };
            if tile.parenttile_id.is_none() && parent_tile_id.is_some() && is_different_nodegroup {
                return true;
            }
        }
    }
    false
}

// =============================================================================
// Generic visitor functions - work with any PseudoValueLike/PseudoListLike
// =============================================================================

/// Generic semantic_to_json function that works with any PseudoValueLike type
///
/// This is the extracted traversal logic that was duplicated between core and WASM.
/// Now both can delegate to this single implementation.
pub fn semantic_to_json<V, L>(value: &V, ctx: &VisitorContext<L>) -> Value
where
    V: PseudoValueLike,
    L: PseudoListLike<Value = V>,
{
    let mut obj = Map::new();

    let parent_tile_id = value.tile().as_ref().and_then(|t| t.tileid.clone());
    let parent_nodegroup_id = value.node().nodegroup_id.clone();

    let child_node_ids = match ctx.edges.get(&value.node().nodeid) {
        Some(ids) => ids,
        None => {
            return Value::Object(obj);
        }
    };

    for child_node_id in child_node_ids {
        let child_node = ctx
            .nodes_by_alias
            .values()
            .find(|n| &n.nodeid == child_node_id);

        let child_node = match child_node {
            Some(n) => n,
            None => {
                continue;
            }
        };

        let child_alias = match &child_node.alias {
            Some(alias) if !alias.is_empty() => alias,
            _ => {
                continue;
            }
        };

        let pseudo_list = match ctx.pseudo_cache.get(child_alias) {
            Some(list) => list,
            None => {
                continue;
            }
        };

        let matching_values = pseudo_list.matching_entries(
            parent_tile_id.clone(),
            child_node.nodegroup_id.clone(),
            parent_nodegroup_id.clone(),
        );

        if matching_values.is_empty() {
            continue;
        }

        let child_ctx = ctx.child();

        if pseudo_list.is_single() {
            // Single-cardinality node: serialize as a direct value
            if let Some(first_value) = matching_values.first() {
                let json_value = to_json_generic(*first_value, &child_ctx);
                if !json_value.is_null() {
                    obj.insert(child_alias.clone(), json_value);
                }
            }
        } else {
            // Multi-cardinality (collector) node: always serialize as array,
            // even with only 1 matching value. Consumers expect array access (e.g. [0]).
            let arr: Vec<Value> = matching_values
                .iter()
                .map(|v| to_json_generic(*v, &child_ctx))
                .filter(|v| !v.is_null())
                .collect();

            if !arr.is_empty() {
                obj.insert(child_alias.clone(), Value::Array(arr));
            }
        }
    }

    Value::Object(obj)
}

/// Build a per-node SerializationContext and serialize the value's own tile_data.
///
/// This looks up node config from the context's NodeConfigManager and creates
/// a SerializationContext with the node-specific config plus shared resolvers.
fn serialize_leaf_value<V, L>(value: &V, ctx: &VisitorContext<L>) -> Value
where
    V: PseudoValueLike,
    L: PseudoListLike<Value = V>,
{
    let node_config = ctx
        .node_config_manager
        .and_then(|ncm| ncm.get(&value.node().nodeid));
    let per_node_ctx = ctx.serialization_context.with_node_config(node_config);
    value.serialize_own_value(&ctx.serialization_options, Some(&per_node_ctx))
}

/// Generic outer_to_json function that works with any PseudoValueLike type
pub fn outer_to_json<V, L>(value: &V, ctx: &VisitorContext<L>) -> Value
where
    V: PseudoValueLike,
    L: PseudoListLike<Value = V>,
{
    let own_value = serialize_leaf_value(value, ctx);

    if let Some(inner) = value.inner() {
        let children_json = semantic_to_json(inner, ctx);
        return ctx
            .serialization_options
            .merge_outer_with_children(own_value, children_json);
    }

    own_value
}

/// Generic non_semantic_with_children_to_json function
pub fn non_semantic_with_children_to_json<V, L>(value: &V, ctx: &VisitorContext<L>) -> Value
where
    V: PseudoValueLike,
    L: PseudoListLike<Value = V>,
{
    let own_value = serialize_leaf_value(value, ctx);
    let children_json = semantic_to_json(value, ctx);
    ctx.serialization_options
        .merge_outer_with_children(own_value, children_json)
}

/// Generic to_json function that works with any PseudoValueLike type
///
/// This is the main entry point for converting a pseudo value to JSON.
/// Both core and WASM can delegate to this function.
pub fn to_json_generic<V, L>(value: &V, ctx: &VisitorContext<L>) -> Value
where
    V: PseudoValueLike,
    L: PseudoListLike<Value = V>,
{
    if ctx.depth > ctx.max_depth {
        return Value::Null;
    }

    // Handle outer nodes with inner (non-semantic with children)
    if value.is_outer() {
        return outer_to_json(value, ctx);
    }

    // Handle inner nodes (synthetic semantic)
    if value.is_inner() {
        return semantic_to_json(value, ctx);
    }

    let datatype = value.datatype();

    // Pure semantic nodes - traverse children
    if datatype == "semantic" {
        return semantic_to_json(value, ctx);
    }

    // Non-semantic nodes WITH children (outer pattern without explicit inner)
    if value.has_children(ctx.edges) {
        return non_semantic_with_children_to_json(value, ctx);
    }

    // Leaf nodes - build per-node context for display/search resolution
    serialize_leaf_value(value, ctx)
}
