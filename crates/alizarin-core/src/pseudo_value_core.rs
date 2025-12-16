/// Core PseudoValue types for JSON conversion
///
/// These types contain the platform-agnostic parts of pseudo values
/// that can be used from any binding (WASM, Python, etc.)
///
/// The WASM bindings extend these with JsValue fields for ViewModel integration.

use std::sync::Arc;
use std::collections::HashMap;
use serde_json::{Value, Map};

use crate::{StaticNode, StaticTile};

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
pub struct VisitorContext<'a> {
    /// The pseudo_cache from the instance wrapper (alias -> PseudoListCore)
    pub pseudo_cache: &'a HashMap<String, PseudoListCore>,
    /// Node alias to node mapping from the model
    pub nodes_by_alias: &'a HashMap<String, Arc<StaticNode>>,
    /// Node edges from the model (parent_nodeid -> child_nodeids)
    pub edges: &'a HashMap<String, Vec<String>>,
    /// Current traversal depth (for preventing infinite recursion)
    pub depth: usize,
    /// Maximum depth
    pub max_depth: usize,
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
}

impl PseudoValueCore {
    /// Create a new PseudoValueCore from a node and optional tile
    pub fn from_node_and_tile(
        node: Arc<StaticNode>,
        tile: Option<Arc<StaticTile>>,
        tile_data: Option<Value>,
        child_node_ids: Vec<String>,
    ) -> Self {
        let independent = tile.is_none();
        PseudoValueCore {
            node,
            child_node_ids,
            is_collector: false,
            inner: None,
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

    /// Convert this pseudo value to JSON
    pub fn to_json(&self, ctx: &VisitorContext) -> Value {
        if ctx.depth > ctx.max_depth {
            return Value::Null;
        }

        // Handle outer nodes with inner (non-semantic with children)
        if self.is_outer() {
            return self.outer_to_json(ctx);
        }

        // Handle inner nodes (synthetic semantic)
        if self.is_inner {
            return self.semantic_to_json(ctx);
        }

        let datatype = self.datatype();

        // Pure semantic nodes - traverse children
        if datatype == "semantic" {
            return self.semantic_to_json(ctx);
        }

        // Leaf nodes - return tile_data
        if let Some(ref data) = self.tile_data {
            return data.clone();
        }

        Value::Null
    }

    /// Convert an outer node to JSON
    fn outer_to_json(&self, ctx: &VisitorContext) -> Value {
        let own_value = self.tile_data.clone().unwrap_or(Value::Null);

        if let Some(ref inner) = self.inner {
            let children_json = inner.semantic_to_json(ctx);

            if let Value::Object(mut children_map) = children_json {
                if !own_value.is_null() {
                    children_map.insert("_value".to_string(), own_value);
                }
                return Value::Object(children_map);
            }
        }

        own_value
    }

    /// Convert a semantic node to JSON by finding and visiting its children
    fn semantic_to_json(&self, ctx: &VisitorContext) -> Value {
        let mut obj = Map::new();

        let parent_tile_id = self.tile.as_ref().and_then(|t| t.tileid.clone());

        let child_node_ids = match ctx.edges.get(&self.node.nodeid) {
            Some(ids) => ids,
            None => return Value::Object(obj),
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

            if matching_values.is_empty() {
                continue;
            }

            let child_ctx = VisitorContext {
                pseudo_cache: ctx.pseudo_cache,
                nodes_by_alias: ctx.nodes_by_alias,
                edges: ctx.edges,
                depth: ctx.depth + 1,
                max_depth: ctx.max_depth,
            };

            if pseudo_list.is_single || matching_values.len() == 1 {
                if let Some(first_value) = matching_values.first() {
                    let json_value = first_value.to_json(&child_ctx);
                    if !json_value.is_null() {
                        obj.insert(child_alias.clone(), json_value);
                    }
                }
            } else {
                let arr: Vec<Value> = matching_values.iter()
                    .map(|v| v.to_json(&child_ctx))
                    .filter(|v| !v.is_null())
                    .collect();

                if !arr.is_empty() {
                    obj.insert(child_alias.clone(), Value::Array(arr));
                }
            }
        }

        Value::Object(obj)
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
                        tile_builder.data.insert(self.node.nodeid.clone(), data.clone());
                    }
                }
            }
        }

        if self.datatype() == "semantic" || self.is_inner {
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

            if let Some(pseudo_list) = ctx.pseudo_cache.get(child_alias) {
                let child_ctx = TileBuilderContext {
                    pseudo_cache: ctx.pseudo_cache,
                    nodes_by_alias: ctx.nodes_by_alias,
                    edges: ctx.edges,
                    resourceinstance_id: ctx.resourceinstance_id.clone(),
                    depth: ctx.depth + 1,
                    max_depth: ctx.max_depth,
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
    pub fn from_values_with_cardinality(node_alias: String, values: Vec<PseudoValueCore>, is_single: bool) -> Self {
        PseudoListCore {
            node_alias,
            values,
            is_loaded: true,
            is_single,
        }
    }

    /// Merge another PseudoListCore into this one
    pub fn merge(&mut self, mut other: PseudoListCore) {
        self.values.extend(other.values.drain(..));
    }

    /// Get matching entries based on tile relationship
    pub fn matching_entries(
        &self,
        parent_tile_id: Option<String>,
        nodegroup_id: Option<String>,
    ) -> Vec<&PseudoValueCore> {
        self.values.iter()
            .filter(|v| {
                if let Some(tile) = v.tile.as_ref() {
                    if let Some(ng) = nodegroup_id.as_ref() {
                        if &tile.nodegroup_id == ng {
                            if tile.tileid == parent_tile_id {
                                return true;
                            }
                            if tile.parenttile_id.is_some() && tile.parenttile_id == parent_tile_id {
                                return true;
                            }
                            if parent_tile_id.is_none() && tile.parenttile_id.is_none() {
                                return true;
                            }
                        }
                    }
                }
                false
            })
            .collect()
    }

    /// Convert to JSON
    pub fn to_json(&self, ctx: &VisitorContext) -> Value {
        if self.values.is_empty() {
            return Value::Null;
        }

        if self.is_single {
            if let Some(first) = self.values.first() {
                return first.to_json(ctx);
            }
            return Value::Null;
        }

        let arr: Vec<Value> = self.values.iter()
            .map(|v| v.to_json(ctx))
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

/// Check if a tile/node matches as a semantic child
///
/// This is used by both WASM and Python bindings for filtering tiles
/// that belong to a particular parent node in the graph hierarchy.
pub fn matches_semantic_child(
    parent_tile_id: Option<&String>,
    parent_node_id: &str,
    child_node: &StaticNode,
    tile: &StaticTile,
) -> bool {
    // Check nodegroup match
    if tile.nodegroup_id != *child_node.nodegroup_id.as_ref().unwrap_or(&"".into()) {
        return false;
    }

    // We do not have a child value, unless there is a value, or the whole tile is the (semantic) value
    if !(Some(&child_node.nodeid) == child_node.nodegroup_id.as_ref() || tile.data.contains_key(&child_node.nodeid)) {
        return false;
    }

    // Branch 1: Different nodegroup + correct parent tile relationship
    if tile.nodegroup_id != parent_node_id {
        if let Some(parent_tid) = parent_tile_id {
            let parent_matches = tile.parenttile_id.is_none()
                || tile.parenttile_id.as_ref() == Some(parent_tid);

            if parent_matches {
                return true;
            }
        }
    }

    // Branch 2: Same nodegroup as parent - child exists on same tile
    if tile.nodegroup_id == parent_node_id {
        if let Some(parent_tid) = parent_tile_id {
            if tile.tileid.as_ref() == Some(parent_tid) {
                return true;
            }
        }
    }

    false
}
