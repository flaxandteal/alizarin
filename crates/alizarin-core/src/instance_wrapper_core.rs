/// Core ResourceInstanceWrapper types and business logic
///
/// This module contains platform-agnostic types and algorithms for:
/// - Tile storage and indexing
/// - Pseudo cache population (populate, ensure_nodegroup)
/// - Semantic child value retrieval
///
/// The WASM and Python bindings wrap these types with platform-specific
/// concerns (RefCell for WASM async, PyO3 for Python).
use crate::shared_mut::SharedMut;
use indexmap::IndexMap;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::path_resolution::{resolve_path_segments, PathError, PathResolutionInfo};
use crate::permissions::PermissionRule;
use crate::pseudo_value_core::{sortorder_key, PseudoListCore, PseudoValueCore};
use crate::{
    ExtensionTypeRegistry, StaticGraph, StaticNode, StaticNodegroup, StaticResourceDescriptors,
    StaticResourceMetadata, StaticTile,
};

// =============================================================================
// Error types
// =============================================================================

/// Error type for semantic child value retrieval
#[derive(Debug, Clone)]
pub enum SemanticChildError {
    /// Tiles the lookup needs have not been loaded yet: every nodegroup to
    /// fetch before asking again (more than one when an on-load Derive target
    /// needs its sources).
    TilesNotLoaded { nodegroup_ids: Vec<String> },
    /// Child node not found
    ChildNotFound { alias: String },
    /// Tiles storage not initialized
    TilesNotInitialized,
    /// Model not initialized
    ModelNotInitialized(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for SemanticChildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // A stable prefix callers can match on, the same on every binding:
            // fetch each listed nodegroup's tiles (comma-separated), then ask again.
            SemanticChildError::TilesNotLoaded { nodegroup_ids } => {
                write!(f, "TILES_NOT_LOADED:{}", nodegroup_ids.join(","))
            }
            SemanticChildError::ChildNotFound { alias } => {
                write!(f, "Child node not found: {}", alias)
            }
            SemanticChildError::TilesNotInitialized => {
                write!(f, "Tiles not initialized")
            }
            SemanticChildError::ModelNotInitialized(msg) => {
                write!(f, "Model not initialized: {}", msg)
            }
            SemanticChildError::Other(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl std::error::Error for SemanticChildError {}

impl From<String> for SemanticChildError {
    fn from(s: String) -> Self {
        SemanticChildError::Other(s)
    }
}

// =============================================================================
// Load state tracking
// =============================================================================

/// Track loading state to prevent race conditions in lazy loading
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadState {
    NotLoaded,
    Loading,
    Loaded,
}

// =============================================================================
// Result types for nodegroup operations
// =============================================================================

/// Result from values_from_resource_nodegroup
#[derive(Clone, Debug)]
pub struct ValuesFromNodegroupResult {
    /// Map of node alias → PseudoListCore
    pub values: HashMap<String, PseudoListCore>,
    /// Implied nodegroups that need loading
    pub implied_nodegroups: Vec<String>,
}

/// Result from ensure_nodegroup
#[derive(Clone, Debug)]
pub struct EnsureNodegroupResult {
    /// Structured values by alias
    pub values: HashMap<String, PseudoListCore>,
    /// Implied nodegroups
    pub implied_nodegroups: Vec<String>,
    /// All nodegroups map (nodegroup_id -> is_loaded)
    pub all_nodegroups_map: HashMap<String, bool>,
}

/// Result from populate
#[derive(Clone, Debug)]
pub struct PopulateResult {
    /// Map of alias → PseudoListCore
    pub values: HashMap<String, PseudoListCore>,
    /// All values map (alias -> is_truthy)
    pub all_values_map: HashMap<String, Option<bool>>,
    /// All nodegroups map (nodegroup_id -> is_loaded)
    pub all_nodegroups_map: HashMap<String, bool>,
}

/// Result of get_semantic_child_value
#[derive(Debug)]
pub enum SemanticChildResult {
    /// A list of pseudo values (for collectors or multiple matches)
    List(PseudoListCore),
    /// A single pseudo value
    Single(PseudoValueCore),
    /// No matching values found (not an error, just empty)
    Empty,
}

// =============================================================================
// Model access trait
// =============================================================================

/// Trait for accessing model data (nodes, edges, nodegroups)
///
/// This abstracts the model registry access pattern so that:
/// - WASM can use thread-local MODEL_REGISTRY
/// - Python can use a different storage mechanism
/// - Tests can use mock implementations
pub trait ModelAccess {
    /// Get all nodes by ID
    fn get_nodes(&self) -> Option<&HashMap<String, Arc<StaticNode>>>;

    /// Get edges (parent_nodeid -> child_nodeids)
    fn get_edges(&self) -> Option<&HashMap<String, Vec<String>>>;

    /// Get reverse edges (child_nodeid -> parent_nodeids)
    fn get_reverse_edges(&self) -> Option<&HashMap<String, Vec<String>>>;

    /// Get nodes grouped by nodegroup
    fn get_nodes_by_nodegroup(&self) -> Option<&HashMap<String, Vec<Arc<StaticNode>>>>;

    /// Get all nodegroups by ID
    fn get_nodegroups(&self) -> Option<&HashMap<String, Arc<StaticNodegroup>>>;

    /// Get the root node of the graph.
    /// Default implementation prefers `istopnode`, with fallback to
    /// scanning for nodes with no `nodegroup_id`.
    fn get_root_node(&self) -> Result<Arc<StaticNode>, String> {
        let nodes = self
            .get_nodes()
            .ok_or_else(|| "Nodes not initialized".to_string())?;
        // Prefer istopnode
        for node in nodes.values() {
            if node.istopnode {
                return Ok(Arc::clone(node));
            }
        }
        // Fallback: node with no nodegroup_id
        for node in nodes.values() {
            if node.nodegroup_id.is_none()
                || node
                    .nodegroup_id
                    .as_ref()
                    .map(|s| s.is_empty())
                    .unwrap_or(true)
            {
                return Ok(Arc::clone(node));
            }
        }
        Err("Could not find root node".to_string())
    }

    /// Get child nodes for a parent node, keyed by alias.
    /// Default implementation derives from `get_edges()` and `get_nodes()`.
    fn get_child_nodes(&self, node_id: &str) -> Result<HashMap<String, Arc<StaticNode>>, String> {
        let edges = self.get_edges().ok_or("Edges not initialized")?;
        let nodes = self.get_nodes().ok_or("Nodes not initialized")?;
        let child_ids = edges.get(node_id).cloned().unwrap_or_default();
        let mut children = HashMap::new();
        for child_id in child_ids {
            if let Some(node) = nodes.get(&child_id) {
                if let Some(ref alias) = node.alias {
                    if !alias.is_empty() {
                        children.insert(alias.clone(), Arc::clone(node));
                    }
                }
            }
        }
        Ok(children)
    }

    /// The node with this alias. The default scans every node; models with an
    /// alias index override it.
    fn get_node_by_alias(&self, alias: &str) -> Option<Arc<StaticNode>> {
        self.get_nodes()?
            .values()
            .find(|node| node.alias.as_deref() == Some(alias))
            .cloned()
    }

    /// Get permitted nodegroups as permission rules.
    /// Each rule may be a simple boolean or a conditional (per-tile) filter.
    fn get_permitted_nodegroups(&self) -> HashMap<String, PermissionRule>;

    /// The graph's on-load Derive declarations. The default finds none; a model
    /// holding the graph ([`GraphModelAccess`](crate::graph_model_access::GraphModelAccess))
    /// parses them once.
    fn on_load_functions(&self) -> Arc<Vec<crate::DeriveConfig>> {
        Arc::new(Vec::new())
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Determine whether a node should be treated as single-cardinality.
///
/// A node is single-cardinality if:
/// - It's the grouping node of its nodegroup AND the nodegroup's cardinality is not "n"
/// - OR it's not a collector (default for leaf nodes)
///
/// Used by both the static path (populate → toJson) and dynamic path
/// (getSemanticChildValue) to ensure consistent array/object serialization.
///
/// This version takes a closure for flexible nodegroup cardinality lookup,
/// enabling use with different storage types (HashMap<String, Arc<StaticNodegroup>>,
/// HashMap<String, StaticNodegroup>, GraphWrapper, etc.)
///
/// The closure should return `Some(cardinality_string)` if the nodegroup is found,
/// or `None` if not found. Returns `Option<String>` to avoid lifetime complexity.
pub fn is_node_single_cardinality_with<F>(node: &StaticNode, get_cardinality: F) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(ref ng_id) = node.nodegroup_id {
        // Only check cardinality for the grouping node (nodeid == nodegroup_id)
        if &node.nodeid == ng_id {
            if let Some(cardinality) = get_cardinality(ng_id) {
                return cardinality != "n";
            }
        }
    }
    // Default: collectors are multi, others single
    !node.is_collector
}

/// Convenience wrapper for is_node_single_cardinality_with that takes a HashMap.
///
/// Used by instance_wrapper code that already has nodegroups indexed by ID.
pub fn is_node_single_cardinality(
    node: &StaticNode,
    nodegroups: Option<&HashMap<String, Arc<StaticNodegroup>>>,
) -> bool {
    is_node_single_cardinality_with(node, |ng_id| {
        nodegroups
            .and_then(|ngs| ngs.get(ng_id))
            .and_then(|ng| ng.cardinality.clone())
    })
}

/// Check if a tile matches semantic parent-child relationship criteria
///
/// This implements the exact logic from SemanticViewModel.__getChildValues
/// to determine if a value should be included as a child of a semantic node.
pub fn matches_semantic_child(
    parent_tile_id: Option<&String>,
    parent_nodegroup_id: Option<&String>,
    child_node: &StaticNode,
    tile: &StaticTile,
) -> bool {
    // Check if tile's nodegroup matches the child node's nodegroup
    if tile.nodegroup_id != *child_node.nodegroup_id.as_ref().unwrap_or(&"".into()) {
        return false;
    }

    // We do not have a child value, unless there is a value, or the whole tile is the
    // (semantic) value, or the child is a semantic node (which doesn't have direct tile data).
    let is_semantic = child_node.datatype == "semantic";
    if !(Some(&child_node.nodeid) == child_node.nodegroup_id.as_ref()
        || tile.data.contains_key(&child_node.nodeid)
        || is_semantic)
    {
        return false;
    }

    // Get the parent's nodegroup ID for comparisons
    let parent_ng = parent_nodegroup_id.map(|s| s.as_str()).unwrap_or("");

    // Branch 1: Different nodegroup + correct parent tile relationship
    // This handles child nodes in a NESTED nodegroup (different from parent's nodegroup)
    if tile.nodegroup_id != parent_ng {
        if let Some(parent_tid) = parent_tile_id {
            // Check if tile.parenttile_id is null or equals parent_tid
            let parent_matches =
                tile.parenttile_id.is_none() || tile.parenttile_id.as_ref() == Some(parent_tid);

            if parent_matches {
                return true;
            }
        } else {
            // Parent has no tile (e.g. root node) — match if child tile also has no parent tile
            if tile.parenttile_id.is_none() {
                return true;
            }
        }
    }

    // Branch 2: Same nodegroup + shared tile + not collector
    // This handles child nodes in the SAME nodegroup as parent (sharing a tile)
    if tile.nodegroup_id == parent_ng {
        if let Some(parent_tid) = parent_tile_id {
            // Check if this tile IS the parent tile and child is not a collector
            // For semantic nodes, we don't require tile data - they get their value
            // from their children, not from tile data directly.
            let has_data_or_is_semantic =
                tile.data.contains_key(&child_node.nodeid) || child_node.datatype == "semantic";
            if tile.tileid.as_ref() == Some(parent_tid)
                && !child_node.is_collector
                && has_data_or_is_semantic
            {
                return true;
            }
        }
    }

    // Branch 3: Different nodegroup + is_collector
    // This handles collector nodes that don't share tiles with their parent
    if tile.nodegroup_id != parent_ng && child_node.is_collector {
        if let Some(parent_tid) = parent_tile_id {
            return tile.parenttile_id.is_none() || tile.parenttile_id.as_ref() == Some(parent_tid);
        }
        return true;
    }

    false
}

/// Resolve a dot-separated path through the graph model and return matching tiles.
///
/// This is the shared implementation for path-based tile access. Takes tile storage
/// and nodegroup index as parameters so it can be called by any wrapper (core, WASM,
/// Python) with its own tile store.
///
/// If `filter_tile_id` is provided, only tiles matching the parent-child relationship
/// are included (using `matches_semantic_child`).
pub fn resolve_and_filter_tiles(
    path: &str,
    model: &dyn ModelAccess,
    tiles_store: &TileStore,
    nodegroup_index: &IndexMap<String, Vec<String>>,
    filter_tile_id: Option<&str>,
) -> Result<(PathResolutionInfo, Vec<StaticTile>), PathError> {
    let root_node = model
        .get_root_node()
        .map_err(PathError::ModelNotInitialized)?;
    let nodes = model
        .get_nodes()
        .ok_or_else(|| PathError::ModelNotInitialized("Nodes not initialized".into()))?;
    let edges = model
        .get_edges()
        .ok_or_else(|| PathError::ModelNotInitialized("Edges not initialized".into()))?;
    let nodegroups = model.get_nodegroups();

    let info = resolve_path_segments(path, &root_node, nodes, edges, nodegroups)?;

    let tile_ids = nodegroup_index
        .get(&info.nodegroup_id)
        .cloned()
        .unwrap_or_default();

    let tiles: Vec<StaticTile> = if let Some(filter_id) = filter_tile_id {
        let filter_id_string = filter_id.to_string();
        let parent_ng = info.parent_nodegroup_id.as_ref();
        tile_ids
            .iter()
            .filter_map(|tid| {
                tiles_store.get(tid).and_then(|t| {
                    if matches_semantic_child(
                        Some(&filter_id_string),
                        parent_ng,
                        &info.target_node,
                        t,
                    ) {
                        Some(t.clone())
                    } else {
                        None
                    }
                })
            })
            .collect()
    } else {
        tile_ids
            .iter()
            .filter_map(|tid| tiles_store.get(tid).cloned())
            .collect()
    };

    Ok((info, tiles))
}

// =============================================================================
// Core resource instance wrapper
// =============================================================================

/// Type alias for alias -> (node, tiles) mapping used in values_from_resource_nodegroup
type AliasTilesMap = HashMap<String, (Arc<StaticNode>, Vec<Option<Arc<StaticTile>>>)>;

/// Create a PseudoListCore from a node and its tiles (standalone version).
pub fn create_pseudo_list_from_tiles(
    node: Arc<StaticNode>,
    tiles: Vec<Option<Arc<StaticTile>>>,
    edges: &HashMap<String, Vec<String>>,
    is_single: bool,
) -> PseudoListCore {
    let alias = node.alias.clone().unwrap_or_default();
    let child_node_ids = edges.get(&node.nodeid).cloned().unwrap_or_default();

    let values: Vec<PseudoValueCore> = tiles
        .into_iter()
        .map(|tile| {
            let tile_data = tile
                .as_ref()
                .and_then(|t| t.data.get(&node.nodeid).cloned());

            PseudoValueCore::from_node_and_tile(
                Arc::clone(&node),
                tile,
                tile_data,
                child_node_ids.clone(),
            )
        })
        .collect();

    PseudoListCore::from_values_with_cardinality(alias, values, is_single)
}

/// Standalone values_from_resource_nodegroup — processes tiles and creates PseudoListCore
/// entries for each node alias in the nodegroup.
///
/// Extracted from `ResourceInstanceWrapperCore::values_from_resource_nodegroup` so that
/// WASM (and other bindings) can call this directly with their own tile store.
pub fn values_from_resource_nodegroup(
    existing_values: &HashMap<String, Option<bool>>,
    nodegroup_tile_ids: &[String],
    nodegroup_id: &str,
    model: &dyn ModelAccess,
    tiles_store: &TileStore,
) -> Result<ValuesFromNodegroupResult, SemanticChildError> {
    let node_objs = model.get_nodes().ok_or_else(|| {
        SemanticChildError::ModelNotInitialized("Model nodes not initialized".to_string())
    })?;
    let edges = model.get_edges().ok_or_else(|| {
        SemanticChildError::ModelNotInitialized("Model edges not initialized".to_string())
    })?;
    let reverse_edges = model.get_reverse_edges().ok_or_else(|| {
        SemanticChildError::ModelNotInitialized("Model reverse edges not initialized".to_string())
    })?;
    let nodes_by_nodegroup = model.get_nodes_by_nodegroup().ok_or_else(|| {
        SemanticChildError::ModelNotInitialized(
            "Model nodes-by-nodegroup not initialized".to_string(),
        )
    })?;
    let nodegroups = model.get_nodegroups();

    let mut values: HashMap<String, PseudoListCore> = HashMap::new();
    let mut implied_nodegroups: HashSet<String> = HashSet::new();
    let mut implied_nodes: HashMap<(String, String), (Arc<StaticNode>, Arc<StaticTile>)> =
        HashMap::new();
    let mut tile_nodes_seen: HashSet<(String, String)> = HashSet::new();

    let mut alias_tiles: AliasTilesMap = HashMap::new();
    let nodegroup_nodes = nodes_by_nodegroup.get(nodegroup_id);

    for tile_id in nodegroup_tile_ids {
        let tile = if tile_id.is_empty() {
            None
        } else {
            tiles_store.get(tile_id).map(|t| Arc::new(t.clone()))
        };
        let tile_nodegroup_id = tile.as_ref().map(|t| t.nodegroup_id.clone());

        if let Some(nodes_in_ng) = nodegroup_nodes {
            for node in nodes_in_ng.iter() {
                let alias = match &node.alias {
                    Some(a) if !a.is_empty() => a.clone(),
                    _ => continue,
                };

                if !tile_id.is_empty() {
                    tile_nodes_seen.insert((node.nodeid.clone(), tile_id.clone()));
                }

                if let Some(Some(true)) = existing_values.get(&alias) {
                    continue;
                }

                let entry = alias_tiles
                    .entry(alias.clone())
                    .or_insert_with(|| (Arc::clone(node), Vec::new()));
                entry.1.push(tile.clone());

                if let Some(parent_ids) = reverse_edges.get(&node.nodeid) {
                    if let Some(parent_id) = parent_ids.first() {
                        if let Some(domain_node) = node_objs.get(parent_id) {
                            if let Some(ref domain_ng_id) = domain_node.nodegroup_id {
                                if !domain_ng_id.is_empty() && domain_ng_id != nodegroup_id {
                                    implied_nodegroups.insert(domain_ng_id.clone());
                                }
                                if let Some(ref tile_ng_id) = tile_nodegroup_id {
                                    if domain_ng_id == tile_ng_id
                                        && domain_ng_id != &domain_node.nodeid
                                        && !tile_id.is_empty()
                                    {
                                        let key = (domain_node.nodeid.clone(), tile_id.clone());
                                        if let Some(t) = tile.as_ref() {
                                            implied_nodes.entry(key).or_insert_with(|| {
                                                (Arc::clone(domain_node), Arc::clone(t))
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Process implied nodes
    for (node, tile) in implied_nodes.values() {
        if let Some(tid) = tile.tileid.as_ref() {
            let key = (node.nodeid.clone(), tid.clone());
            if !tile_nodes_seen.contains(&key) {
                let alias = match &node.alias {
                    Some(a) if !a.is_empty() => a.clone(),
                    _ => continue,
                };
                tile_nodes_seen.insert(key);
                if existing_values.get(&alias) != Some(&Some(true)) {
                    let entry = alias_tiles
                        .entry(alias.clone())
                        .or_insert_with(|| (Arc::clone(node), Vec::new()));
                    entry.1.push(Some(Arc::clone(tile)));
                }
            }
        }
    }

    // Convert to PseudoListCore
    for (alias, (node, tiles)) in alias_tiles {
        let is_single = is_node_single_cardinality(&node, nodegroups);
        let pseudo_list = create_pseudo_list_from_tiles(node, tiles, edges, is_single);
        values.insert(alias, pseudo_list);
    }

    Ok(ValuesFromNodegroupResult {
        values,
        implied_nodegroups: implied_nodegroups.into_iter().collect(),
    })
}

/// Standalone ensure_nodegroup — processes a single nodegroup and returns structured values.
///
/// Extracted from `ResourceInstanceWrapperCore::ensure_nodegroup` so that WASM
/// (and other bindings) can call this directly with their own tile store.
#[allow(clippy::too_many_arguments)]
pub fn ensure_nodegroup(
    all_values_map: &HashMap<String, Option<bool>>,
    all_nodegroups: &mut HashMap<String, bool>,
    nodegroup_id: &str,
    add_if_missing: bool,
    nodegroup_permissions: &HashMap<String, PermissionRule>,
    do_implied_nodegroups: bool,
    model: &dyn ModelAccess,
    tiles_store: &TileStore,
) -> Result<EnsureNodegroupResult, SemanticChildError> {
    let sentinel = all_nodegroups.get(nodegroup_id);
    let should_process = match sentinel {
        Some(&false) => true,
        Some(&true) => false,
        None => add_if_missing,
    };

    let mut all_values: HashMap<String, PseudoListCore> = HashMap::new();
    let mut implied_nodegroups_set: HashSet<String> = HashSet::new();

    if should_process {
        let mut nodegroup_tiles: Vec<String> = Vec::new();

        for (tile_id, tile) in tiles_store.iter() {
            if tile.nodegroup_id == nodegroup_id {
                let permitted = nodegroup_permissions
                    .get(&tile.nodegroup_id)
                    .map(|rule| rule.permits_tile(tile))
                    .unwrap_or(true);
                if permitted {
                    nodegroup_tiles.push(tile_id.clone());
                }
            }
        }

        if nodegroup_tiles.is_empty() && add_if_missing {
            nodegroup_tiles.push(String::new());
        }

        let values_result = values_from_resource_nodegroup(
            all_values_map,
            &nodegroup_tiles,
            nodegroup_id,
            model,
            tiles_store,
        )?;

        for (alias, pseudo_list) in values_result.values {
            all_values.insert(alias, pseudo_list);
        }
        for ng in values_result.implied_nodegroups.iter() {
            implied_nodegroups_set.insert(ng.clone());
        }

        all_nodegroups.insert(nodegroup_id.to_string(), true);

        if do_implied_nodegroups && !implied_nodegroups_set.is_empty() {
            let implied_list: Vec<String> = implied_nodegroups_set.iter().cloned().collect();
            for implied_ng in implied_list.iter() {
                let implied_result = ensure_nodegroup(
                    all_values_map,
                    all_nodegroups,
                    implied_ng,
                    true,
                    nodegroup_permissions,
                    true,
                    model,
                    tiles_store,
                )?;
                for (alias, pseudo_list) in implied_result.values {
                    all_values.insert(alias, pseudo_list);
                }
            }
            implied_nodegroups_set.clear();
        }
    }

    Ok(EnsureNodegroupResult {
        values: all_values,
        implied_nodegroups: implied_nodegroups_set.into_iter().collect(),
        all_nodegroups_map: all_nodegroups.clone(),
    })
}

/// A wrapper's tiles by id, in the order they were loaded or added, so every
/// export follows that order rather than hash order.
pub type TileStore = IndexMap<String, StaticTile>;

/// Core resource instance wrapper - platform-agnostic business logic
///
/// Contains all tile storage, indexing, and business logic.
/// Used by NAPI, Python, and WASM bindings. SharedMut provides
/// Rc<RefCell> on wasm32 and Arc<Mutex> on native targets.
pub struct ResourceInstanceWrapperCore {
    /// Graph ID to look up model
    pub graph_id: String,

    /// Resource metadata
    pub resource_instance: Option<StaticResourceMetadata>,

    /// Tile storage (tileid -> tile)
    pub tiles: Option<TileStore>,

    /// Index: nodegroup_id -> list of tile_ids
    pub nodegroup_index: IndexMap<String, Vec<String>>,

    /// Track which nodegroups have been loaded/loading
    pub loaded_nodegroups: SharedMut<HashMap<String, LoadState>>,

    /// Cache of PseudoValues (alias -> PseudoListCore)
    pub pseudo_cache: SharedMut<HashMap<String, PseudoListCore>>,

    /// Lazy mode: tiles arrive per nodegroup as the host fetches them, so a
    /// nodegroup not yet loaded is unknown, not empty. Core keeps only this
    /// state (and `loaded_nodegroups`) and reports what is missing; the
    /// fetching is the host's (async, outside core).
    pub lazy: bool,

    /// Ids of the tiles on-load Derive functions added
    /// ([`derive_on_load`](Self::derive_on_load)): recomputed on every load,
    /// never stored, so left out of [`stored_tiles`](Self::stored_tiles).
    pub on_load_derived: HashSet<String>,

    /// A compiled-in source of this resource's tiles, tried before the host's
    /// loader on a lazy fetch ([`fetch_from_tile_source`](Self::fetch_from_tile_source)).
    /// Set from Rust, the same way on every binding.
    pub tile_source: Option<Arc<dyn crate::TileSource>>,
}

impl ResourceInstanceWrapperCore {
    /// Create a new empty core with just a graph ID
    pub fn new(graph_id: String) -> Self {
        ResourceInstanceWrapperCore {
            graph_id,
            resource_instance: None,
            tiles: None,
            nodegroup_index: IndexMap::new(),
            loaded_nodegroups: SharedMut::new(HashMap::new()),
            pseudo_cache: SharedMut::new(HashMap::new()),
            lazy: false,
            on_load_derived: HashSet::new(),
            tile_source: None,
        }
    }

    /// Load tiles into the wrapper
    pub fn load_tiles(&mut self, tiles: Vec<StaticTile>) {
        let mut tiles_map = TileStore::new();
        let mut nodegroup_index: IndexMap<String, Vec<String>> = IndexMap::new();

        for tile in tiles {
            let tile_id = tile
                .tileid
                .clone()
                .unwrap_or_else(|| format!("synthetic_{}", tiles_map.len()));

            // Index by nodegroup
            nodegroup_index
                .entry(tile.nodegroup_id.clone())
                .or_default()
                .push(tile_id.clone());

            tiles_map.insert(tile_id, tile);
        }

        self.tiles = Some(tiles_map);
        self.nodegroup_index = nodegroup_index;

        // Lazy mode: exactly the nodegroups loaded now count as fetched.
        if self.lazy {
            if let Ok(mut loaded) = self.loaded_nodegroups.write() {
                loaded.clear();
                for nodegroup_id in self.nodegroup_index.keys() {
                    loaded.insert(nodegroup_id.clone(), LoadState::Loaded);
                }
            }
        }
    }

    /// Run the graph's **on-load Derive functions** over the loaded tiles, with
    /// providers from `registry`, replacing the tiles an earlier call added.
    /// The load and merge methods below call it with the global registry, so a
    /// binding gets on-load tiles by loading; call it directly only after
    /// changing tiles some other way.
    ///
    /// - Each function's tiles get [`derived_tile_id`](crate::derived_tile_id)s,
    ///   the same on every load, and are recorded in
    ///   [`on_load_derived`](Self::on_load_derived), so
    ///   [`stored_tiles`](Self::stored_tiles) leaves them out.
    /// - A target nodegroup that holds other tiles is left alone (attested data
    ///   wins). Functions chain, in declaration order.
    /// - Lazy mode: a function runs once all its declared `sources` are loaded;
    ///   one declaring none can't tell, so it runs on eager loads only. Until
    ///   it runs, its target counts as unloaded (unless it holds other tiles),
    ///   so [`nodegroups_to_fetch`](Self::nodegroups_to_fetch) has the host
    ///   fetch the sources rather than read the target as empty.
    /// - Cached values of a target whose tiles changed are dropped, so a read
    ///   after a later fetch sees the new tiles.
    /// - Membership is trivially true: without a layer stack, a resource is a
    ///   member of its own graph. A provider missing from `registry`, or one
    ///   that errors, yields no tiles.
    /// - Providers get the tiles and must not reach back into this wrapper
    ///   (the [`DeriveProvider`](crate::DeriveProvider) purity contract): every
    ///   binding holds it for the whole load.
    pub fn derive_on_load(
        &mut self,
        model: &crate::graph_model_access::GraphModelAccess,
        registry: &crate::graph::FunctionsRegistry,
    ) {
        let functions = model.on_load_functions();
        if functions.is_empty() && self.on_load_derived.is_empty() {
            return;
        }
        let graph = model.get_graph();

        // Take out the earlier on-load tiles: they're recomputed from what is
        // now loaded.
        let mut previous: HashMap<String, StaticTile> = HashMap::new();
        for id in std::mem::take(&mut self.on_load_derived) {
            if let Some(tile) = self.tiles.as_mut().and_then(|t| t.shift_remove(&id)) {
                if let Some(ids) = self.nodegroup_index.get_mut(&tile.nodegroup_id) {
                    ids.retain(|i| *i != id);
                }
                previous.insert(id, tile);
            }
        }
        let mut tiles: Vec<StaticTile> = self
            .tiles
            .as_ref()
            .map(|t| t.values().cloned().collect())
            .unwrap_or_default();
        let resource_id = self
            .resource_instance
            .as_ref()
            .map(|r| r.resourceinstanceid.clone())
            .or_else(|| tiles.first().map(|t| t.resourceinstance_id.clone()))
            .unwrap_or_default();

        let mut derived: Vec<StaticTile> = Vec::new();
        for func in functions.iter() {
            let attested = tiles.iter().any(|t| t.nodegroup_id == func.nodegroup);
            if self.lazy && !func.sources.iter().all(|s| self.is_nodegroup_loaded(s)) {
                if !func.sources.is_empty() && !attested {
                    if let Ok(mut loaded) = self.loaded_nodegroups.write() {
                        loaded.remove(&func.nodegroup);
                    }
                }
                continue;
            }
            if self.lazy && func.sources.is_empty() {
                continue;
            }
            if attested {
                continue;
            }
            let Some(provider) = registry.derive(&func.provider) else {
                continue;
            };
            let Ok(produced) = provider.derive(&resource_id, graph, &tiles, func) else {
                continue;
            };
            let mut next: HashMap<Option<String>, usize> = HashMap::new();
            for mut tile in produced
                .into_iter()
                .filter(|t| t.nodegroup_id == func.nodegroup)
            {
                let k = next.entry(tile.parenttile_id.clone()).or_insert(0);
                tile.tileid = Some(crate::derived_tile_id(
                    &resource_id,
                    &func.nodegroup,
                    tile.parenttile_id.as_deref(),
                    &func.provider,
                    *k,
                ));
                *k += 1;
                tile.resourceinstance_id = resource_id.clone();
                tiles.push(tile.clone());
                derived.push(tile);
            }
            if self.lazy {
                self.mark_nodegroup_loaded(&func.nodegroup);
            }
        }

        // Store them, noting the targets whose tiles changed.
        let mut changed: HashSet<String> = HashSet::new();
        for tile in derived {
            let id = tile.tileid.clone().unwrap_or_default();
            let same = previous.remove(&id).is_some_and(|old| {
                old.data == tile.data
                    && old.parenttile_id == tile.parenttile_id
                    && old.sortorder == tile.sortorder
            });
            if !same {
                changed.insert(tile.nodegroup_id.clone());
            }
            self.nodegroup_index
                .entry(tile.nodegroup_id.clone())
                .or_default()
                .push(id.clone());
            self.tiles
                .get_or_insert_with(TileStore::new)
                .insert(id.clone(), tile);
            self.on_load_derived.insert(id);
        }
        // Earlier tiles not produced again are gone.
        changed.extend(previous.into_values().map(|t| t.nodegroup_id));

        // Drop the cached values those targets' nodes held.
        if !changed.is_empty() {
            if let (Some(by_nodegroup), Ok(mut cache)) =
                (model.get_nodes_by_nodegroup(), self.pseudo_cache.write())
            {
                for nodegroup_id in &changed {
                    for node in by_nodegroup.get(nodegroup_id).into_iter().flatten() {
                        if let Some(alias) = &node.alias {
                            cache.remove(alias);
                        }
                    }
                }
            }
        }
    }

    /// Drop every loaded tile `model` doesn't permit, with its index entry, as
    /// every binding's `pruneResourceTiles` does. An error if no tiles are
    /// loaded.
    pub fn prune_to_permitted(
        &mut self,
        model: &crate::graph_model_access::GraphModelAccess,
    ) -> Result<(), String> {
        let tiles = self.tiles.as_mut().ok_or("tiles not loaded")?;
        tiles.retain(|_, tile| model.is_tile_permitted(tile));
        let kept: HashSet<String> = tiles.keys().cloned().collect();
        for ids in self.nodegroup_index.values_mut() {
            ids.retain(|id| kept.contains(id));
        }
        Ok(())
    }

    /// Serialise card `card_id` (with its parent tile and nodegroup when
    /// nested), or, given none, every root card as an array, from the pseudo
    /// cache: populate the nodegroups `nodegroup_ids_for_card` lists first.
    /// `params` gives display mode; `None` is tile-data mode. What each
    /// binding's `serializeCard`/`serializeRootCards` (and display forms) call.
    pub fn serialize_cards(
        &self,
        card_id: Option<&str>,
        parent_tile_id: Option<&str>,
        parent_nodegroup_id: Option<&str>,
        max_depth: Option<usize>,
        model: &crate::graph_model_access::GraphModelAccess,
        params: Option<crate::card_traversal::CardSerializationParams>,
    ) -> Result<serde_json::Value, String> {
        let graph = model.get_graph_arc();
        let card_index = graph
            .card_index()
            .ok_or("Graph has no card index — cards may not be loaded")?;
        let cache = self
            .pseudo_cache
            .read()
            .map_err(|e| format!("Cache lock failed: {e}"))?;
        match card_id {
            Some(card_id) => crate::card_traversal::serialize_card(
                card_id,
                card_index,
                &cache,
                parent_tile_id,
                parent_nodegroup_id,
                &graph,
                max_depth,
                params,
            ),
            None => Ok(serde_json::Value::Array(
                crate::card_traversal::serialize_root_cards(
                    card_index, &cache, &graph, max_depth, params,
                ),
            )),
        }
    }

    /// The nodegroups card `card_id` and its descendants (to `max_depth`)
    /// need populated before `serialize_cards`.
    pub fn nodegroup_ids_for_card(
        card_id: &str,
        max_depth: Option<usize>,
        model: &crate::graph_model_access::GraphModelAccess,
    ) -> Result<Vec<String>, String> {
        let graph = model.get_graph_arc();
        let card_index = graph
            .card_index()
            .ok_or("Graph has no card index — cards may not be loaded")?;
        Ok(card_index.nodegroup_ids_for_card(card_id, max_depth))
    }

    /// Populate the pseudo cache from the loaded tiles if it hasn't been: what
    /// serialising (`toJson`, `toDisplayJson`) needs, so no binding's caller
    /// has to populate first. Loads nothing: tiles not loaded aren't there.
    pub fn ensure_populated(&self, model: &dyn ModelAccess) -> Result<(), SemanticChildError> {
        let populated = self
            .pseudo_cache
            .read()
            .map(|cache| cache.len() > 1)
            .unwrap_or(false);
        if populated {
            return Ok(());
        }
        let root_alias = model
            .get_root_node()
            .map_err(SemanticChildError::ModelNotInitialized)?
            .alias
            .clone()
            .ok_or_else(|| SemanticChildError::Other("root node has no alias".to_string()))?;
        let nodegroup_ids: Vec<String> = model
            .get_nodegroups()
            .map(|ngs| crate::sorted_serde::sorted_keys(ngs.keys()))
            .unwrap_or_default();
        self.populate(false, &nodegroup_ids, &root_alias, model)
            .map(|_| ())
    }

    /// Whether the resource's tiles are loaded: once loaded, even with none,
    /// except in lazy mode, where an empty store means nothing fetched yet.
    /// What every binding's `tilesLoaded` reports.
    pub fn tiles_loaded(&self) -> bool {
        self.tiles
            .as_ref()
            .is_some_and(|t| !self.lazy || !t.is_empty())
    }

    /// The tiles that are the resource's data: every loaded tile except those
    /// on-load Derive functions added, which are recomputed on each load and
    /// never stored. What exports write.
    pub fn stored_tiles(&self) -> Vec<&StaticTile> {
        self.tiles
            .as_ref()
            .map(|t| {
                t.iter()
                    .filter(|(id, _)| !self.on_load_derived.contains(*id))
                    .map(|(_, tile)| tile)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// [`derive_on_load`](Self::derive_on_load) with the global functions
    /// registry, if there is anything to do: what the load and merge methods
    /// below run.
    pub fn derive_on_load_with(&mut self, model: &crate::graph_model_access::GraphModelAccess) {
        if self.on_load_derived.is_empty() && model.on_load_functions().is_empty() {
            return;
        }
        self.derive_on_load(model, &crate::get_global_functions_registry());
    }

    /// [`load_tiles`](Self::load_tiles), keeping only the tiles `model`
    /// permits, then [`derive_on_load`](Self::derive_on_load): how a binding
    /// loads a resource, so permissions and on-load tiles are handled in one
    /// place.
    pub fn load_permitted_tiles(
        &mut self,
        tiles: impl IntoIterator<Item = StaticTile>,
        model: &crate::graph_model_access::GraphModelAccess,
    ) {
        self.on_load_derived.clear();
        self.load_tiles(
            tiles
                .into_iter()
                .filter(|tile| model.is_tile_permitted(tile))
                .collect(),
        );
        self.derive_on_load_with(model);
    }

    /// [`merge_tiles`](Self::merge_tiles), keeping only the tiles `model`
    /// permits, then [`derive_on_load`](Self::derive_on_load): how a binding
    /// adds tiles incrementally.
    pub fn merge_permitted_tiles(
        &mut self,
        tiles: impl IntoIterator<Item = StaticTile>,
        model: &crate::graph_model_access::GraphModelAccess,
    ) {
        self.merge_tiles(
            tiles
                .into_iter()
                .filter(|tile| model.is_tile_permitted(tile))
                .collect(),
        );
        self.derive_on_load_with(model);
    }

    /// Merge the tiles a host fetched for `nodegroup_id` (`None`: the whole
    /// resource), as [`merge_permitted_tiles`](Self::merge_permitted_tiles),
    /// and record the fetch: that nodegroup counts as loaded even if it had no
    /// tiles, so a lazy lookup neither reads an unfetched nodegroup as empty
    /// nor asks for a fetched one again. `None` marks every nodegroup.
    pub fn merge_fetched_tiles(
        &mut self,
        tiles: impl IntoIterator<Item = StaticTile>,
        nodegroup_id: Option<&str>,
        model: &crate::graph_model_access::GraphModelAccess,
    ) {
        self.merge_tiles(
            tiles
                .into_iter()
                .filter(|tile| model.is_tile_permitted(tile))
                .collect(),
        );
        match nodegroup_id {
            Some(nodegroup_id) => self.mark_nodegroup_loaded(nodegroup_id),
            None => self.mark_all_nodegroups_loaded(model),
        }
        self.derive_on_load_with(model);
    }

    /// Record that the tiles just loaded or merged are the resource's complete
    /// set: every nodegroup counts as loaded. In lazy mode that can let on-load
    /// Derive functions run that were waiting for their sources, so they run
    /// again here. What every binding calls after a comprehensive load.
    pub fn mark_loaded_comprehensively(
        &mut self,
        model: &crate::graph_model_access::GraphModelAccess,
    ) {
        self.mark_all_nodegroups_loaded(model);
        if self.lazy {
            self.derive_on_load_with(model);
        }
    }

    /// Fetch `nodegroup_ids` from the [`tile_source`](Self::tile_source), if
    /// there is one, merging each as [`merge_fetched_tiles`](Self::merge_fetched_tiles)
    /// does. Returns the nodegroups still to fetch, for the host's loader: all
    /// of them without a source or a loaded resource id, and those the source
    /// doesn't hold this resource for. A source failure is an error.
    pub fn fetch_from_tile_source(
        &mut self,
        nodegroup_ids: &[String],
        model: &crate::graph_model_access::GraphModelAccess,
    ) -> Result<Vec<String>, crate::TileSourceError> {
        let source = self.tile_source.clone();
        let resource_id = self
            .resource_instance
            .as_ref()
            .map(|r| r.resourceinstanceid.clone());
        let (Some(source), Some(resource_id)) = (source, resource_id) else {
            return Ok(nodegroup_ids.to_vec());
        };
        let mut remaining = Vec::new();
        for nodegroup_id in nodegroup_ids {
            match source.load_tiles(&resource_id, Some(nodegroup_id)) {
                Ok(tiles) => self.merge_fetched_tiles(tiles, Some(nodegroup_id), model),
                Err(crate::TileSourceError::ResourceNotFound { .. }) => {
                    remaining.push(nodegroup_id.clone())
                }
                Err(e) => return Err(e),
            }
        }
        Ok(remaining)
    }

    /// Merge tiles into the wrapper, keeping whatever is already loaded.
    ///
    /// Unlike [`load_tiles`](Self::load_tiles), which replaces the tile store,
    /// this appends so a resource's nodegroups can be hydrated incrementally
    /// (lazy per-nodegroup loading). A tile whose `tileid` is already present
    /// is overwritten, so re-merging the same nodegroup is idempotent.
    pub fn merge_tiles(&mut self, tiles: Vec<StaticTile>) {
        let tiles_map = self.tiles.get_or_insert_with(TileStore::new);
        let mut merged_nodegroups: Vec<String> = Vec::new();
        for tile in tiles {
            let tile_id = tile
                .tileid
                .clone()
                .unwrap_or_else(|| format!("synthetic_{}", tiles_map.len()));
            let entry = self
                .nodegroup_index
                .entry(tile.nodegroup_id.clone())
                .or_default();
            if !entry.contains(&tile_id) {
                entry.push(tile_id.clone());
            }
            if !merged_nodegroups.contains(&tile.nodegroup_id) {
                merged_nodegroups.push(tile.nodegroup_id.clone());
            }
            tiles_map.insert(tile_id, tile);
        }

        // Lazy mode: the merged nodegroups now count as fetched.
        if self.lazy {
            for nodegroup_id in &merged_nodegroups {
                self.mark_nodegroup_loaded(nodegroup_id);
            }
        }
    }

    /// Get a tile by ID
    pub fn get_tile(&self, tile_id: &str) -> Option<&StaticTile> {
        self.tiles.as_ref().and_then(|t| t.get(tile_id))
    }

    /// Get tiles for a nodegroup
    pub fn get_tiles_for_nodegroup(&self, nodegroup_id: &str) -> Vec<&StaticTile> {
        let tile_ids = self.nodegroup_index.get(nodegroup_id);
        match (tile_ids, &self.tiles) {
            (Some(ids), Some(tiles)) => ids.iter().filter_map(|id| tiles.get(id)).collect(),
            _ => Vec::new(),
        }
    }

    /// Check if a nodegroup is loaded
    pub fn is_nodegroup_loaded(&self, nodegroup_id: &str) -> bool {
        if let Ok(loaded) = self.loaded_nodegroups.read() {
            matches!(loaded.get(nodegroup_id), Some(LoadState::Loaded))
        } else {
            false
        }
    }

    /// Mark a nodegroup as loaded
    pub fn mark_nodegroup_loaded(&self, nodegroup_id: &str) {
        if let Ok(mut loaded) = self.loaded_nodegroups.write() {
            loaded.insert(nodegroup_id.to_string(), LoadState::Loaded);
        }
    }

    /// Mark every nodegroup of the model loaded: for tiles the host says are
    /// the resource's complete set, so a nodegroup without tiles is known to
    /// be empty rather than unfetched. A nodegroup already tracked (loading,
    /// say) is left as it is.
    pub fn mark_all_nodegroups_loaded(&self, model: &dyn ModelAccess) {
        let Some(nodegroups) = model.get_nodegroups() else {
            return;
        };
        if let Ok(mut loaded) = self.loaded_nodegroups.write() {
            for nodegroup_id in nodegroups.keys() {
                loaded
                    .entry(nodegroup_id.clone())
                    .or_insert(LoadState::Loaded);
            }
        }
    }

    /// The nodegroups of `parent_node_id`'s children whose tiles aren't loaded
    /// yet, each listed once (children often share one): what a lazy caller
    /// must fetch before visiting them. In eager mode (`lazy` false) every
    /// tile is loaded up front, so none.
    pub fn missing_nodegroups_for_children(
        &self,
        parent_node_id: &str,
        model: &dyn ModelAccess,
        lazy: bool,
    ) -> Vec<String> {
        if !lazy {
            return Vec::new();
        }
        let (Some(edges), Some(nodes)) = (model.get_edges(), model.get_nodes()) else {
            return Vec::new();
        };
        let mut missing: Vec<String> = Vec::new();
        for child_id in edges.get(parent_node_id).into_iter().flatten() {
            if let Some(nodegroup_id) = nodes.get(child_id).and_then(|n| n.nodegroup_id.as_ref()) {
                for needed in self.unloaded_for(nodegroup_id, model) {
                    if !missing.contains(&needed) {
                        missing.push(needed);
                    }
                }
            }
        }
        missing
    }

    /// The nodegroups to fetch before `nodegroup_id` can be read, in lazy mode:
    /// itself, if not loaded, and, if it is the target of an on-load Derive
    /// function still waiting for them, that function's unloaded sources (the
    /// target counts as unloaded until its function has run). Empty when it can
    /// be read, and always outside lazy mode.
    pub fn nodegroups_to_fetch(&self, nodegroup_id: &str, model: &dyn ModelAccess) -> Vec<String> {
        if !self.lazy {
            return Vec::new();
        }
        self.unloaded_for(nodegroup_id, model)
    }

    /// [`nodegroups_to_fetch`](Self::nodegroups_to_fetch), whatever the mode.
    fn unloaded_for(&self, nodegroup_id: &str, model: &dyn ModelAccess) -> Vec<String> {
        if self.is_nodegroup_loaded(nodegroup_id) {
            return Vec::new();
        }
        let mut needed = vec![nodegroup_id.to_string()];
        let functions = model.on_load_functions();
        for func in functions.iter().filter(|f| f.nodegroup == nodegroup_id) {
            for source in &func.sources {
                if !self.is_nodegroup_loaded(source) && !needed.contains(source) {
                    needed.push(source.clone());
                }
            }
        }
        needed
    }

    /// Drop everything loaded for this resource: its tiles and their index,
    /// the pseudo cache and the nodegroup load states.
    pub fn release(&mut self) {
        self.tiles = None;
        self.nodegroup_index.clear();
        if let Ok(mut cache) = self.pseudo_cache.write() {
            cache.clear();
        }
        if let Ok(mut loaded) = self.loaded_nodegroups.write() {
            loaded.clear();
        }
    }

    /// Switch lazy mode (see [`Self::lazy`]).
    pub fn set_lazy(&mut self, lazy: bool) {
        self.lazy = lazy;
    }

    /// Whether this instance is in lazy mode.
    pub fn is_lazy(&self) -> bool {
        self.lazy
    }

    /// This resource's descriptors: the cached ones, unless `recompute` is set
    /// or none are cached, when they are built from its tiles with the graph's
    /// descriptor templates, and cached. `context` gives the graph and the
    /// extension registry, and is called only to build. Errors if no tiles
    /// are loaded. A missing `name` is the resource's stored name.
    pub fn descriptors(
        &mut self,
        recompute: bool,
        context: impl FnOnce() -> Result<(Arc<StaticGraph>, ExtensionTypeRegistry), String>,
    ) -> Result<StaticResourceDescriptors, String> {
        let mut descriptors = self.computed_descriptors(recompute, context)?;
        // No descriptor function gave a name: the resource's stored one, which
        // Arches keeps as the computed name. Only in what is returned; the
        // resource's own descriptors stay as computed. A template missing its
        // tile gives its placeholder (`<Name>`), which is kept: visibly stale.
        if descriptors.name.as_deref().is_none_or(str::is_empty) {
            if let Some(name) = self
                .resource_instance
                .as_ref()
                .map(|r| &r.name)
                .filter(|n| !n.is_empty())
            {
                descriptors.name = Some(name.clone());
            }
        }
        Ok(descriptors)
    }

    fn computed_descriptors(
        &mut self,
        recompute: bool,
        context: impl FnOnce() -> Result<(Arc<StaticGraph>, ExtensionTypeRegistry), String>,
    ) -> Result<StaticResourceDescriptors, String> {
        if !recompute {
            if let Some(cached) = self.resource_instance.as_ref().map(|r| &r.descriptors) {
                if !cached.is_empty() {
                    return Ok(cached.clone());
                }
            }
        }
        let tiles: Vec<StaticTile> = self
            .tiles
            .as_ref()
            .ok_or_else(|| "No tiles loaded".to_string())?
            .values()
            .cloned()
            .collect();
        let (graph, extension_registry) = context()?;
        let descriptors = graph.build_descriptors_with_context(
            &tiles,
            &mut Vec::new(),
            None,
            Some(&extension_registry),
        );
        if let Some(resource) = self.resource_instance.as_mut() {
            resource.descriptors = descriptors.clone();
        }
        Ok(descriptors)
    }

    /// This resource's name: its metadata's, else its descriptors', else
    /// `<Unnamed>`.
    pub fn display_name(&self) -> String {
        self.resource_instance
            .as_ref()
            .and_then(|r| {
                if r.name.is_empty() {
                    r.descriptors.name.clone()
                } else {
                    Some(r.name.clone())
                }
            })
            .unwrap_or_else(|| "<Unnamed>".to_string())
    }

    /// The pseudo for `alias` that hosts wrap when adding a child or making a
    /// root (TS `addPseudo`): a collector, unless `is_single`, is a list of its
    /// nodegroup's tiles in load order, empty if not permitted or no tiles are
    /// loaded; anything else is one value on `tile_id`'s tile (no tile: an
    /// independent value, filled later), or `Empty` (unavailable) if not
    /// permitted or no tiles are loaded.
    pub fn make_pseudo_value(
        &self,
        alias: &str,
        tile_id: Option<&str>,
        is_permitted: bool,
        is_single: bool,
        model: &dyn ModelAccess,
    ) -> Result<SemanticChildResult, String> {
        let node = model
            .get_node_by_alias(alias)
            .ok_or_else(|| format!("Could not find node by alias: {alias}"))?;
        let child_ids = model
            .get_edges()
            .ok_or("Model edges not initialized")?
            .get(&node.nodeid)
            .cloned()
            .unwrap_or_default();
        let tiles = self.tiles.as_ref().filter(|_| is_permitted);
        let value_on = |tile: Option<&StaticTile>| {
            PseudoValueCore::from_node_and_tile(
                Arc::clone(&node),
                tile.map(|t| Arc::new(t.clone())),
                tile.and_then(|t| t.data.get(&node.nodeid).cloned()),
                child_ids.clone(),
            )
        };

        if node.is_collector && !is_single {
            Ok(SemanticChildResult::List(match tiles {
                None => PseudoListCore::new(alias.to_string()),
                Some(tiles) => {
                    let nodegroup_id = node.nodegroup_id.as_deref().unwrap_or("");
                    let values = self
                        .nodegroup_index
                        .get(nodegroup_id)
                        .into_iter()
                        .flatten()
                        .filter_map(|id| tiles.get(id))
                        .map(|tile| value_on(Some(tile)))
                        .collect();
                    // In tile sortorder, as every list is.
                    PseudoListCore::from_values(alias.to_string(), values)
                }
            }))
        } else {
            Ok(match tiles {
                None => SemanticChildResult::Empty,
                // An id with no loaded tile is an independent value too.
                Some(tiles) => {
                    SemanticChildResult::Single(value_on(tile_id.and_then(|id| tiles.get(id))))
                }
            })
        }
    }

    /// The ids of `child_node`'s tiles that are semantic children of the given
    /// parent: only the child's nodegroup's tiles are looked at (every tile if
    /// it has none). Ordered by `sortorder`, tiles without one after the
    /// numbered ones; ties and unnumbered tiles keep their load order.
    fn semantic_child_tile_ids(
        &self,
        parent_tile_id: Option<&String>,
        parent_nodegroup_id: Option<&String>,
        child_node: &StaticNode,
    ) -> Result<Vec<String>, SemanticChildError> {
        let tiles = self
            .tiles
            .as_ref()
            .ok_or(SemanticChildError::TilesNotInitialized)?;
        let candidates: Vec<&String> = match &child_node.nodegroup_id {
            Some(nodegroup_id) => self
                .nodegroup_index
                .get(nodegroup_id)
                .map(|ids| ids.iter().collect())
                .unwrap_or_default(),
            None => tiles.keys().collect(),
        };
        let mut ids: Vec<String> = candidates
            .into_iter()
            .filter(|id| {
                tiles.get(*id).is_some_and(|tile| {
                    matches_semantic_child(parent_tile_id, parent_nodegroup_id, child_node, tile)
                })
            })
            .cloned()
            .collect();
        // Stable, so equal keys keep load order.
        ids.sort_by_key(|id| sortorder_key(tiles.get(id).and_then(|tile| tile.sortorder)));
        Ok(ids)
    }

    /// For every child of `parent_node_id`, by alias: the ids of its tiles that
    /// are semantic children of the given parent, ordered as
    /// [`get_semantic_child_value`](Self::get_semantic_child_value) orders
    /// values (`[]` if none).
    pub fn find_semantic_children(
        &self,
        parent_tile_id: Option<&String>,
        parent_node_id: &str,
        parent_nodegroup_id: Option<&String>,
        model: &dyn ModelAccess,
    ) -> Result<BTreeMap<String, Vec<String>>, SemanticChildError> {
        let children = model
            .get_child_nodes(parent_node_id)
            .map_err(SemanticChildError::ModelNotInitialized)?;
        children
            .iter()
            .map(|(alias, child)| {
                let ids =
                    self.semantic_child_tile_ids(parent_tile_id, parent_nodegroup_id, child)?;
                Ok((alias.clone(), ids))
            })
            .collect()
    }

    /// Every child of `parent_node_id`'s value, by alias, as
    /// [`get_semantic_child_value`](Self::get_semantic_child_value) gives it.
    /// Errors propagate: in lazy mode, `TilesNotLoaded` for a child whose
    /// nodegroup the host hasn't fetched yet.
    pub fn get_all_semantic_child_values(
        &self,
        parent_tile_id: Option<&String>,
        parent_node_id: &str,
        parent_nodegroup_id: Option<&String>,
        model: &dyn ModelAccess,
    ) -> Result<BTreeMap<String, SemanticChildResult>, SemanticChildError> {
        let children = model
            .get_child_nodes(parent_node_id)
            .map_err(SemanticChildError::ModelNotInitialized)?;
        children
            .keys()
            .map(|alias| {
                let value = self.get_semantic_child_value(
                    parent_tile_id,
                    parent_node_id,
                    parent_nodegroup_id,
                    alias,
                    model,
                )?;
                Ok((alias.clone(), value))
            })
            .collect()
    }

    /// Get a cached pseudo list by alias
    pub fn get_cached_pseudo(&self, alias: &str) -> Option<PseudoListCore> {
        if let Ok(cache) = self.pseudo_cache.read() {
            cache.get(alias).cloned()
        } else {
            None
        }
    }

    /// Store a pseudo list in the cache
    pub fn store_pseudo(&self, alias: String, pseudo_list: PseudoListCore) {
        if let Ok(mut cache) = self.pseudo_cache.write() {
            cache.insert(alias, pseudo_list);
        }
    }

    /// Build pseudo values from tiles for a nodegroup.
    ///
    /// Delegates to the standalone `values_from_resource_nodegroup` function,
    /// passing this wrapper's tile store.
    pub fn values_from_resource_nodegroup(
        &self,
        existing_values: &HashMap<String, Option<bool>>,
        nodegroup_tile_ids: &[String],
        nodegroup_id: &str,
        model: &dyn ModelAccess,
    ) -> Result<ValuesFromNodegroupResult, SemanticChildError> {
        let tiles_store = match &self.tiles {
            Some(t) => t,
            None => {
                return Ok(ValuesFromNodegroupResult {
                    values: HashMap::new(),
                    implied_nodegroups: Vec::new(),
                });
            }
        };
        values_from_resource_nodegroup(
            existing_values,
            nodegroup_tile_ids,
            nodegroup_id,
            model,
            tiles_store,
        )
    }

    /// Process a single nodegroup and return structured values.
    ///
    /// Delegates to the standalone `ensure_nodegroup` function,
    /// passing this wrapper's tile store.
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_nodegroup(
        &self,
        all_values_map: &HashMap<String, Option<bool>>,
        all_nodegroups: &mut HashMap<String, bool>,
        nodegroup_id: &str,
        add_if_missing: bool,
        nodegroup_permissions: &HashMap<String, PermissionRule>,
        do_implied_nodegroups: bool,
        model: &dyn ModelAccess,
    ) -> Result<EnsureNodegroupResult, SemanticChildError> {
        let tiles_store = match &self.tiles {
            Some(t) => t,
            None => {
                return Ok(EnsureNodegroupResult {
                    values: HashMap::new(),
                    implied_nodegroups: Vec::new(),
                    all_nodegroups_map: all_nodegroups.clone(),
                });
            }
        };
        ensure_nodegroup(
            all_values_map,
            all_nodegroups,
            nodegroup_id,
            add_if_missing,
            nodegroup_permissions,
            do_implied_nodegroups,
            model,
            tiles_store,
        )
    }

    /// Main populate implementation
    ///
    /// Orchestrates loading all nodegroups for a resource.
    pub fn populate(
        &self,
        lazy: bool,
        nodegroup_ids: &[String],
        root_node_alias: &str,
        model: &dyn ModelAccess,
    ) -> Result<PopulateResult, SemanticChildError> {
        let nodegroup_permissions = model.get_permitted_nodegroups();

        // Check if pseudo_cache has been populated
        let cache_len = if let Ok(cache) = self.pseudo_cache.read() {
            cache.len()
        } else {
            0
        };
        let cache_populated = cache_len > 1;

        // Get already-loaded nodegroups
        let already_loaded: HashSet<String> = if cache_populated {
            if let Ok(loaded) = self.loaded_nodegroups.read() {
                loaded
                    .iter()
                    .filter(|(_, state)| **state == LoadState::Loaded)
                    .map(|(id, _)| id.clone())
                    .collect()
            } else {
                HashSet::new()
            }
        } else {
            HashSet::new()
        };

        // Filter nodegroup_ids to only those not already loaded
        let nodegroups_to_process: Vec<String> = nodegroup_ids
            .iter()
            .filter(|id| !already_loaded.contains(*id))
            .cloned()
            .collect();

        // Initialize state maps
        let mut all_values: HashMap<String, Option<bool>> = HashMap::new();
        let mut all_nodegroups: HashMap<String, bool> = HashMap::new();

        // Initialize nodegroups
        for nodegroup_id in nodegroup_ids.iter() {
            let is_loaded = already_loaded.contains(nodegroup_id);
            all_nodegroups.insert(nodegroup_id.clone(), is_loaded);
        }

        // Set root node alias to false
        all_values.insert(root_node_alias.to_string(), Some(false));

        // Start with existing cache entries
        let mut all_structured_values: HashMap<String, PseudoListCore> =
            if !already_loaded.is_empty() {
                if let Ok(cache) = self.pseudo_cache.read() {
                    cache.clone()
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            };

        // Non-lazy loading: process all nodegroups
        if !lazy {
            let mut implied_nodegroups_set = HashSet::new();

            // Phase 1: Process all nodegroups with doImpliedNodegroups=false
            for nodegroup_id in nodegroups_to_process.iter() {
                let result = self.ensure_nodegroup(
                    &all_values,
                    &mut all_nodegroups,
                    nodegroup_id,
                    true,
                    &nodegroup_permissions,
                    false,
                    model,
                )?;

                for (alias, pseudo_list) in result.values {
                    all_structured_values.insert(alias, pseudo_list);
                }

                for implied_ng in result.implied_nodegroups.iter() {
                    if implied_ng != nodegroup_id {
                        implied_nodegroups_set.insert(implied_ng.clone());
                    }
                }
            }

            // Phase 2: Process implied nodegroups iteratively
            while !implied_nodegroups_set.is_empty() {
                let current_implied: Vec<String> = implied_nodegroups_set.iter().cloned().collect();
                implied_nodegroups_set.clear();

                for nodegroup_id in current_implied.iter() {
                    let current_value = all_nodegroups.get(nodegroup_id);
                    let should_process = matches!(current_value, Some(&false) | None);

                    if should_process {
                        let result = self.ensure_nodegroup(
                            &all_values,
                            &mut all_nodegroups,
                            nodegroup_id,
                            true,
                            &nodegroup_permissions,
                            true,
                            model,
                        )?;

                        for (alias, pseudo_list) in result.values {
                            all_structured_values.insert(alias, pseudo_list);
                        }

                        for implied_ng in result.implied_nodegroups.iter() {
                            implied_nodegroups_set.insert(implied_ng.clone());
                        }
                    }
                }
            }
        }

        // Create root pseudo value
        let root_node = model
            .get_root_node()
            .map_err(SemanticChildError::ModelNotInitialized)?;
        let edges = model.get_edges().ok_or_else(|| {
            SemanticChildError::ModelNotInitialized("Model edges not initialized".to_string())
        })?;

        let child_node_ids = edges.get(&root_node.nodeid).cloned().unwrap_or_default();

        let root_pseudo =
            PseudoValueCore::from_node_and_tile(root_node, None, None, child_node_ids);

        let root_list = PseudoListCore::from_values_with_cardinality(
            root_node_alias.to_string(),
            vec![root_pseudo],
            true,
        );

        all_structured_values.insert(root_node_alias.to_string(), root_list);

        // Store all values in pseudo_cache
        if let Ok(mut cache) = self.pseudo_cache.write() {
            for (alias, pseudo_list) in all_structured_values.iter() {
                cache.insert(alias.clone(), pseudo_list.clone());
            }
        }

        Ok(PopulateResult {
            values: all_structured_values,
            all_values_map: all_values,
            all_nodegroups_map: all_nodegroups,
        })
    }

    /// Get semantic child values for a given parent node and child alias
    pub fn get_semantic_child_value(
        &self,
        parent_tile_id: Option<&String>,
        parent_node_id: &str,
        parent_nodegroup_id: Option<&String>,
        child_alias: &str,
        model: &dyn ModelAccess,
    ) -> Result<SemanticChildResult, SemanticChildError> {
        // Get child nodes for this parent
        let child_nodes = model
            .get_child_nodes(parent_node_id)
            .map_err(SemanticChildError::ModelNotInitialized)?;

        // Find the child node by alias
        let child_node = child_nodes
            .values()
            .find(|n| n.alias.as_deref() == Some(child_alias))
            .ok_or_else(|| SemanticChildError::ChildNotFound {
                alias: child_alias.to_string(),
            })?;

        // Lazy mode: the host hasn't fetched this nodegroup's tiles yet, so
        // its absence would be a guess. Say so; the host fetches and asks again.
        if let Some(nodegroup_id) = &child_node.nodegroup_id {
            let nodegroup_ids = self.nodegroups_to_fetch(nodegroup_id, model);
            if !nodegroup_ids.is_empty() {
                return Err(SemanticChildError::TilesNotLoaded { nodegroup_ids });
            }
        }

        // Get tiles storage
        let tiles = self
            .tiles
            .as_ref()
            .ok_or(SemanticChildError::TilesNotInitialized)?;

        // The matching tiles, from the child's nodegroup only, in sortorder.
        let matching_tiles: Vec<Arc<StaticTile>> = self
            .semantic_child_tile_ids(parent_tile_id, parent_nodegroup_id, child_node)?
            .iter()
            .filter_map(|id| tiles.get(id))
            .map(|tile| Arc::new(tile.clone()))
            .collect();

        if matching_tiles.is_empty() {
            return Ok(SemanticChildResult::Empty);
        }

        // Get edges for child_node_ids
        let edges = model.get_edges().ok_or_else(|| {
            SemanticChildError::ModelNotInitialized("Model edges not initialized".to_string())
        })?;
        let child_node_ids = edges.get(&child_node.nodeid).cloned().unwrap_or_default();

        // Determine cardinality
        let nodegroups = model.get_nodegroups();
        let is_single = is_node_single_cardinality(child_node, nodegroups);

        // Create pseudo values
        let values: Vec<PseudoValueCore> = matching_tiles
            .into_iter()
            .map(|tile| {
                let tile_data = tile.data.get(&child_node.nodeid).cloned();
                PseudoValueCore::from_node_and_tile(
                    Arc::clone(child_node),
                    Some(tile),
                    tile_data,
                    child_node_ids.clone(),
                )
            })
            .collect();

        if is_single && values.len() == 1 {
            Ok(SemanticChildResult::Single(
                values.into_iter().next().unwrap(),
            ))
        } else {
            let list = PseudoListCore::from_values_with_cardinality(
                child_alias.to_string(),
                values,
                is_single,
            );
            Ok(SemanticChildResult::List(list))
        }
    }

    /// Resolve a dot-separated path through the graph model.
    ///
    /// Walks the graph edges matching node aliases at each segment,
    /// returning the target node's metadata and nodegroup ID.
    pub fn resolve_path(
        &self,
        path: &str,
        model: &dyn ModelAccess,
    ) -> Result<PathResolutionInfo, PathError> {
        let root_node = model
            .get_root_node()
            .map_err(PathError::ModelNotInitialized)?;
        let nodes = model
            .get_nodes()
            .ok_or_else(|| PathError::ModelNotInitialized("Nodes not initialized".into()))?;
        let edges = model
            .get_edges()
            .ok_or_else(|| PathError::ModelNotInitialized("Edges not initialized".into()))?;
        let nodegroups = model.get_nodegroups();

        resolve_path_segments(path, &root_node, nodes, edges, nodegroups)
    }

    /// Resolve a path and return the resolution info plus matching tiles.
    ///
    /// Delegates to the standalone `resolve_and_filter_tiles` function,
    /// passing this wrapper's tile store and nodegroup index.
    pub fn resolve_and_filter_tiles(
        &self,
        path: &str,
        model: &dyn ModelAccess,
        filter_tile_id: Option<&str>,
    ) -> Result<(PathResolutionInfo, Vec<StaticTile>), PathError> {
        let tiles_store = self.tiles.as_ref().ok_or(PathError::TilesNotInitialized)?;
        resolve_and_filter_tiles(
            path,
            model,
            tiles_store,
            &self.nodegroup_index,
            filter_tile_id,
        )
    }

    /// Resolve a dot-separated path and return a PseudoListCore for the target node.
    ///
    /// This is the main entry point for path-based value access. It walks the graph
    /// model to find the target node, then retrieves matching tiles from the store
    /// and builds a PseudoListCore — all without materializing the full tree.
    ///
    /// If `filter_tile_id` is provided, only tiles that match the parent-child
    /// relationship for that tile are included (using `matches_semantic_child`).
    pub fn get_values_at_path(
        &self,
        path: &str,
        model: &dyn ModelAccess,
        filter_tile_id: Option<&str>,
    ) -> Result<PseudoListCore, PathError> {
        let (info, tiles) = self.resolve_and_filter_tiles(path, model, filter_tile_id)?;

        let edges = model
            .get_edges()
            .ok_or_else(|| PathError::ModelNotInitialized("Edges not initialized".into()))?;

        let tiles_wrapped: Vec<Option<Arc<StaticTile>>> =
            tiles.into_iter().map(|t| Some(Arc::new(t))).collect();

        let pseudo_list =
            create_pseudo_list_from_tiles(info.target_node, tiles_wrapped, edges, info.is_single);

        Ok(pseudo_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_state_equality() {
        assert_eq!(LoadState::NotLoaded, LoadState::NotLoaded);
        assert_eq!(LoadState::Loading, LoadState::Loading);
        assert_eq!(LoadState::Loaded, LoadState::Loaded);
        assert_ne!(LoadState::NotLoaded, LoadState::Loaded);
    }

    #[test]
    fn test_is_node_single_cardinality_collector() {
        let node = Arc::new(StaticNode {
            nodeid: "test-node".to_string(),
            name: "Test Node".to_string(),
            alias: Some("test".to_string()),
            datatype: "string".to_string(),
            is_collector: true,
            nodegroup_id: Some("test-nodegroup".to_string()),
            graph_id: "test-graph".to_string(),
            isrequired: false,
            exportable: true,
            sortorder: None,
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: None,
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: false,
            istopnode: false,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        });

        // Collector nodes are multi-cardinality by default
        assert!(!is_node_single_cardinality(&node, None));
    }

    #[test]
    fn test_is_node_single_cardinality_non_collector() {
        let node = Arc::new(StaticNode {
            nodeid: "test-node".to_string(),
            name: "Test Node".to_string(),
            alias: Some("test".to_string()),
            datatype: "string".to_string(),
            is_collector: false,
            nodegroup_id: Some("test-nodegroup".to_string()),
            graph_id: "test-graph".to_string(),
            isrequired: false,
            exportable: true,
            sortorder: None,
            config: HashMap::new(),
            parentproperty: None,
            ontologyclass: None,
            description: None,
            fieldname: None,
            hascustomalias: false,
            issearchable: false,
            istopnode: false,
            sourcebranchpublication_id: None,
            source_identifier_id: None,
            is_immutable: None,
        });

        // Non-collector nodes are single-cardinality by default
        assert!(is_node_single_cardinality(&node, None));
    }
}
