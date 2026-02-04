/// Core ResourceInstanceWrapper types and business logic
///
/// This module contains platform-agnostic types and algorithms for:
/// - Tile storage and indexing
/// - Pseudo cache population (populate, ensure_nodegroup)
/// - Semantic child value retrieval
///
/// The WASM and Python bindings wrap these types with platform-specific
/// concerns (RefCell for WASM async, PyO3 for Python).
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::{StaticNode, StaticNodegroup, StaticTile, StaticResourceMetadata};
use crate::pseudo_value_core::{PseudoValueCore, PseudoListCore};

// =============================================================================
// Error types
// =============================================================================

/// Error type for semantic child value retrieval
#[derive(Debug, Clone)]
pub enum SemanticChildError {
    /// Tiles for the required nodegroup have not been loaded yet
    TilesNotLoaded { nodegroup_id: String },
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
            SemanticChildError::TilesNotLoaded { nodegroup_id } => {
                write!(f, "Tiles not loaded for nodegroup: {}", nodegroup_id)
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

    /// Get the root node of the graph
    fn get_root_node(&self) -> Result<Arc<StaticNode>, String>;

    /// Get child nodes for a parent node
    fn get_child_nodes(&self, node_id: &str) -> Result<HashMap<String, Arc<StaticNode>>, String>;

    /// Get permitted nodegroups (nodegroup_id -> is_permitted)
    fn get_permitted_nodegroups(&self) -> HashMap<String, bool>;
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
pub fn is_node_single_cardinality_with<F>(
    node: &StaticNode,
    get_cardinality: F,
) -> bool
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
        || is_semantic) {
        return false;
    }

    // Get the parent's nodegroup ID for comparisons
    let parent_ng = parent_nodegroup_id.map(|s| s.as_str()).unwrap_or("");

    // Branch 1: Different nodegroup + correct parent tile relationship
    // This handles child nodes in a NESTED nodegroup (different from parent's nodegroup)
    if tile.nodegroup_id != parent_ng {
        if let Some(parent_tid) = parent_tile_id {
            // Check if tile.parenttile_id is null or equals parent_tid
            let parent_matches = tile.parenttile_id.is_none()
                || tile.parenttile_id.as_ref() == Some(parent_tid);

            if parent_matches {
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
            let has_data_or_is_semantic = tile.data.contains_key(&child_node.nodeid)
                || child_node.datatype == "semantic";
            if tile.tileid.as_ref() == Some(parent_tid) && !child_node.is_collector && has_data_or_is_semantic {
                return true;
            }
        }
    }

    // Branch 3: Different nodegroup + is_collector
    // This handles collector nodes that don't share tiles with their parent
    if tile.nodegroup_id != parent_ng && child_node.is_collector {
        return true;
    }

    false
}

// =============================================================================
// Core resource instance wrapper
// =============================================================================

/// Type alias for alias -> (node, tiles) mapping used in values_from_resource_nodegroup
type AliasTilesMap = HashMap<String, (Arc<StaticNode>, Vec<Option<Arc<StaticTile>>>)>;

/// Core resource instance wrapper - platform-agnostic business logic
///
/// Contains all tile storage, indexing, and business logic.
/// Can be used from WASM, Python, or other bindings.
pub struct ResourceInstanceWrapperCore {
    /// Graph ID to look up model
    pub graph_id: String,

    /// Resource metadata
    pub resource_instance: Option<StaticResourceMetadata>,

    /// Tile storage (tileid -> tile)
    pub tiles: Option<HashMap<String, StaticTile>>,

    /// Index: nodegroup_id -> list of tile_ids
    pub nodegroup_index: HashMap<String, Vec<String>>,

    /// Track which nodegroups have been loaded/loading
    pub loaded_nodegroups: Arc<Mutex<HashMap<String, LoadState>>>,

    /// Cache of PseudoValues (alias -> PseudoListCore)
    pub pseudo_cache: Arc<Mutex<HashMap<String, PseudoListCore>>>,

    /// Cached model indices - avoids repeated lookups
    pub cached_nodes: Option<Arc<HashMap<String, Arc<StaticNode>>>>,
    pub cached_edges: Option<Arc<HashMap<String, Vec<String>>>>,
    pub cached_reverse_edges: Option<Arc<HashMap<String, Vec<String>>>>,
    pub cached_nodes_by_nodegroup: Option<Arc<HashMap<String, Vec<Arc<StaticNode>>>>>,
    pub cached_nodegroups: Option<Arc<HashMap<String, Arc<StaticNodegroup>>>>,
}

impl ResourceInstanceWrapperCore {
    /// Create a new empty core with just a graph ID
    pub fn new(graph_id: String) -> Self {
        ResourceInstanceWrapperCore {
            graph_id,
            resource_instance: None,
            tiles: None,
            nodegroup_index: HashMap::new(),
            loaded_nodegroups: Arc::new(Mutex::new(HashMap::new())),
            pseudo_cache: Arc::new(Mutex::new(HashMap::new())),
            cached_nodes: None,
            cached_edges: None,
            cached_reverse_edges: None,
            cached_nodes_by_nodegroup: None,
            cached_nodegroups: None,
        }
    }

    /// Set cached model indices from a ModelAccess implementation
    pub fn set_cached_indices(&mut self, model: &dyn ModelAccess) {
        self.cached_nodes = model.get_nodes().map(|m| Arc::new(m.clone()));
        self.cached_edges = model.get_edges().map(|m| Arc::new(m.clone()));
        self.cached_reverse_edges = model.get_reverse_edges().map(|m| Arc::new(m.clone()));
        self.cached_nodes_by_nodegroup = model.get_nodes_by_nodegroup().map(|m| Arc::new(m.clone()));
        self.cached_nodegroups = model.get_nodegroups().map(|m| Arc::new(m.clone()));
    }

    /// Load tiles into the wrapper
    pub fn load_tiles(&mut self, tiles: Vec<StaticTile>) {
        let mut tiles_map = HashMap::new();
        let mut nodegroup_index: HashMap<String, Vec<String>> = HashMap::new();

        for tile in tiles {
            let tile_id = tile.tileid.clone().unwrap_or_else(|| {
                format!("synthetic_{}", tiles_map.len())
            });

            // Index by nodegroup
            nodegroup_index
                .entry(tile.nodegroup_id.clone())
                .or_default()
                .push(tile_id.clone());

            tiles_map.insert(tile_id, tile);
        }

        self.tiles = Some(tiles_map);
        self.nodegroup_index = nodegroup_index;
    }

    /// Get a tile by ID
    pub fn get_tile(&self, tile_id: &str) -> Option<&StaticTile> {
        self.tiles.as_ref().and_then(|t| t.get(tile_id))
    }

    /// Get tiles for a nodegroup
    pub fn get_tiles_for_nodegroup(&self, nodegroup_id: &str) -> Vec<&StaticTile> {
        let tile_ids = self.nodegroup_index.get(nodegroup_id);
        match (tile_ids, &self.tiles) {
            (Some(ids), Some(tiles)) => {
                ids.iter()
                    .filter_map(|id| tiles.get(id))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// Check if a nodegroup is loaded
    pub fn is_nodegroup_loaded(&self, nodegroup_id: &str) -> bool {
        if let Ok(loaded) = self.loaded_nodegroups.lock() {
            matches!(loaded.get(nodegroup_id), Some(LoadState::Loaded))
        } else {
            false
        }
    }

    /// Mark a nodegroup as loaded
    pub fn mark_nodegroup_loaded(&self, nodegroup_id: &str) {
        if let Ok(mut loaded) = self.loaded_nodegroups.lock() {
            loaded.insert(nodegroup_id.to_string(), LoadState::Loaded);
        }
    }

    /// Get a cached pseudo list by alias
    pub fn get_cached_pseudo(&self, alias: &str) -> Option<PseudoListCore> {
        if let Ok(cache) = self.pseudo_cache.lock() {
            cache.get(alias).cloned()
        } else {
            None
        }
    }

    /// Store a pseudo list in the cache
    pub fn store_pseudo(&self, alias: String, pseudo_list: PseudoListCore) {
        if let Ok(mut cache) = self.pseudo_cache.lock() {
            cache.insert(alias, pseudo_list);
        }
    }

    /// Build pseudo values from tiles for a nodegroup
    ///
    /// This is the core algorithm that processes tiles and creates PseudoListCore
    /// entries for each node alias in the nodegroup.
    pub fn values_from_resource_nodegroup(
        &self,
        existing_values: &HashMap<String, Option<bool>>,
        nodegroup_tile_ids: &[String],
        nodegroup_id: &str,
        model: &dyn ModelAccess,
    ) -> Result<ValuesFromNodegroupResult, SemanticChildError> {
        let node_objs = model.get_nodes()
            .ok_or_else(|| SemanticChildError::ModelNotInitialized("Model nodes not initialized".to_string()))?;
        let edges = model.get_edges()
            .ok_or_else(|| SemanticChildError::ModelNotInitialized("Model edges not initialized".to_string()))?;
        let reverse_edges = model.get_reverse_edges()
            .ok_or_else(|| SemanticChildError::ModelNotInitialized("Model reverse edges not initialized".to_string()))?;
        let nodes_by_nodegroup = model.get_nodes_by_nodegroup()
            .ok_or_else(|| SemanticChildError::ModelNotInitialized("Model nodes-by-nodegroup not initialized".to_string()))?;
        let nodegroups = model.get_nodegroups();

        let mut values: HashMap<String, PseudoListCore> = HashMap::new();
        let mut implied_nodegroups: HashSet<String> = HashSet::new();

        // Track implied nodes (parent nodes in same nodegroup that need pseudo values)
        let mut implied_nodes: HashMap<(String, String), (Arc<StaticNode>, Arc<StaticTile>)> = HashMap::new();
        let mut tile_nodes_seen: HashSet<(String, String)> = HashSet::new();

        let tiles_store = match &self.tiles {
            Some(t) => t,
            None => {
                return Ok(ValuesFromNodegroupResult {
                    values: HashMap::new(),
                    implied_nodegroups: Vec::new(),
                });
            }
        };

        // Build a map of alias -> (node, tiles)
        let mut alias_tiles: AliasTilesMap = HashMap::new();

        // Get nodes for this nodegroup
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

                    // Track seen (nodeid, tileid) combinations
                    if !tile_id.is_empty() {
                        tile_nodes_seen.insert((node.nodeid.clone(), tile_id.clone()));
                    }

                    // Skip if already exists as truthy
                    if let Some(Some(true)) = existing_values.get(&alias) {
                        continue;
                    }

                    // Add to alias_tiles
                    let entry = alias_tiles.entry(alias.clone()).or_insert_with(|| {
                        (Arc::clone(node), Vec::new())
                    });
                    entry.1.push(tile.clone());

                    // Check for implied nodegroups (parent in different nodegroup)
                    if let Some(parent_ids) = reverse_edges.get(&node.nodeid) {
                        if let Some(parent_id) = parent_ids.first() {
                            if let Some(domain_node) = node_objs.get(parent_id) {
                                if let Some(ref domain_ng_id) = domain_node.nodegroup_id {
                                    if !domain_ng_id.is_empty() && domain_ng_id != nodegroup_id {
                                        implied_nodegroups.insert(domain_ng_id.clone());
                                    }

                                    // Check for implied nodes (same nodegroup, parent node)
                                    if let Some(ref tile_ng_id) = tile_nodegroup_id {
                                        if domain_ng_id == tile_ng_id
                                            && domain_ng_id != &domain_node.nodeid
                                            && !tile_id.is_empty()
                                        {
                                            let key = (domain_node.nodeid.clone(), tile_id.clone());
                                            if let Some(t) = tile.as_ref() {
                                                implied_nodes.entry(key).or_insert_with(|| (Arc::clone(domain_node), Arc::clone(t)));
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
        for (_key, (node, tile)) in implied_nodes.iter() {
            if let Some(tid) = tile.tileid.as_ref() {
                let key = (node.nodeid.clone(), tid.clone());
                if !tile_nodes_seen.contains(&key) {
                    let alias = match &node.alias {
                        Some(a) if !a.is_empty() => a.clone(),
                        _ => continue,
                    };

                    tile_nodes_seen.insert(key);

                    if existing_values.get(&alias) != Some(&Some(true)) {
                        let entry = alias_tiles.entry(alias.clone()).or_insert_with(|| {
                            (Arc::clone(node), Vec::new())
                        });
                        entry.1.push(Some(Arc::clone(tile)));
                    }
                }
            }
        }

        // Convert to PseudoListCore
        for (alias, (node, tiles)) in alias_tiles {
            let is_single = is_node_single_cardinality(&node, nodegroups);
            let pseudo_list = Self::create_pseudo_list_from_tiles(node, tiles, edges, is_single);
            values.insert(alias, pseudo_list);
        }

        Ok(ValuesFromNodegroupResult {
            values,
            implied_nodegroups: implied_nodegroups.into_iter().collect(),
        })
    }

    /// Create a PseudoListCore from a node and its tiles
    fn create_pseudo_list_from_tiles(
        node: Arc<StaticNode>,
        tiles: Vec<Option<Arc<StaticTile>>>,
        edges: &HashMap<String, Vec<String>>,
        is_single: bool,
    ) -> PseudoListCore {
        let alias = node.alias.clone().unwrap_or_default();
        let child_node_ids = edges.get(&node.nodeid)
            .cloned()
            .unwrap_or_default();

        let values: Vec<PseudoValueCore> = tiles.into_iter()
            .map(|tile| {
                let tile_data = tile.as_ref().and_then(|t| {
                    t.data.get(&node.nodeid).cloned()
                });

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

    /// Process a single nodegroup and return structured values
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_nodegroup(
        &self,
        all_values_map: &HashMap<String, Option<bool>>,
        all_nodegroups: &mut HashMap<String, bool>,
        nodegroup_id: &str,
        add_if_missing: bool,
        nodegroup_permissions: &HashMap<String, bool>,
        do_implied_nodegroups: bool,
        model: &dyn ModelAccess,
    ) -> Result<EnsureNodegroupResult, SemanticChildError> {
        // Check sentinel state
        let sentinel = all_nodegroups.get(nodegroup_id);
        let should_process = match sentinel {
            Some(&false) => true,        // force reload
            Some(&true) => false,        // already loaded
            None => add_if_missing,      // key doesn't exist
        };

        let mut all_values: HashMap<String, PseudoListCore> = HashMap::new();
        let mut implied_nodegroups_set: HashSet<String> = HashSet::new();

        if should_process {
            // Filter tiles by nodegroup_id and permissions
            let mut nodegroup_tiles: Vec<String> = Vec::new();

            if let Some(tiles) = &self.tiles {
                for (tile_id, tile) in tiles.iter() {
                    if tile.nodegroup_id == nodegroup_id {
                        let permitted = nodegroup_permissions.get(&tile.nodegroup_id).copied().unwrap_or(true);
                        if permitted {
                            nodegroup_tiles.push(tile_id.clone());
                        }
                    }
                }
            }

            // If no tiles and addIfMissing, use empty string to indicate null tile
            if nodegroup_tiles.is_empty() && add_if_missing {
                nodegroup_tiles.push(String::new());
            }

            // Call values_from_resource_nodegroup
            let values_result = self.values_from_resource_nodegroup(
                all_values_map,
                &nodegroup_tiles,
                nodegroup_id,
                model,
            )?;

            // Merge structured values
            for (alias, pseudo_list) in values_result.values {
                all_values.insert(alias, pseudo_list);
            }

            // Collect implied nodegroups
            for ng in values_result.implied_nodegroups.iter() {
                implied_nodegroups_set.insert(ng.clone());
            }

            // Mark nodegroup as loaded
            all_nodegroups.insert(nodegroup_id.to_string(), true);

            // Recursive processing of implied nodegroups
            if do_implied_nodegroups && !implied_nodegroups_set.is_empty() {
                let implied_list: Vec<String> = implied_nodegroups_set.iter().cloned().collect();

                for implied_ng in implied_list.iter() {
                    let implied_result = self.ensure_nodegroup(
                        all_values_map,
                        all_nodegroups,
                        implied_ng,
                        true,
                        nodegroup_permissions,
                        true,
                        model,
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
        let cache_len = if let Ok(cache) = self.pseudo_cache.lock() {
            cache.len()
        } else {
            0
        };
        let cache_populated = cache_len > 1;

        // Get already-loaded nodegroups
        let already_loaded: HashSet<String> = if cache_populated {
            if let Ok(loaded) = self.loaded_nodegroups.lock() {
                loaded.iter()
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
        let nodegroups_to_process: Vec<String> = nodegroup_ids.iter()
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
        let mut all_structured_values: HashMap<String, PseudoListCore> = if !already_loaded.is_empty() {
            if let Ok(cache) = self.pseudo_cache.lock() {
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
        let root_node = model.get_root_node()
            .map_err(SemanticChildError::ModelNotInitialized)?;
        let edges = model.get_edges()
            .ok_or_else(|| SemanticChildError::ModelNotInitialized("Model edges not initialized".to_string()))?;

        let child_node_ids = edges.get(&root_node.nodeid)
            .cloned()
            .unwrap_or_default();

        let root_pseudo = PseudoValueCore::from_node_and_tile(
            root_node,
            None,
            None,
            child_node_ids,
        );

        let root_list = PseudoListCore::from_values_with_cardinality(
            root_node_alias.to_string(),
            vec![root_pseudo],
            true,
        );

        all_structured_values.insert(root_node_alias.to_string(), root_list);

        // Store all values in pseudo_cache
        if let Ok(mut cache) = self.pseudo_cache.lock() {
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
        let child_nodes = model.get_child_nodes(parent_node_id)
            .map_err(SemanticChildError::ModelNotInitialized)?;

        // Find the child node by alias
        let child_node = child_nodes.values()
            .find(|n| n.alias.as_deref() == Some(child_alias))
            .ok_or_else(|| SemanticChildError::ChildNotFound { alias: child_alias.to_string() })?;

        // Get tiles storage
        let tiles = self.tiles.as_ref()
            .ok_or(SemanticChildError::TilesNotInitialized)?;

        // Filter tiles that match the semantic child relationship
        let matching_tiles: Vec<Arc<StaticTile>> = tiles.values()
            .filter(|tile| {
                matches_semantic_child(
                    parent_tile_id,
                    parent_nodegroup_id,
                    child_node,
                    tile,
                )
            })
            .map(|t| Arc::new(t.clone()))
            .collect();

        if matching_tiles.is_empty() {
            return Ok(SemanticChildResult::Empty);
        }

        // Get edges for child_node_ids
        let edges = model.get_edges()
            .ok_or_else(|| SemanticChildError::ModelNotInitialized("Model edges not initialized".to_string()))?;
        let child_node_ids = edges.get(&child_node.nodeid)
            .cloned()
            .unwrap_or_default();

        // Determine cardinality
        let nodegroups = model.get_nodegroups();
        let is_single = is_node_single_cardinality(child_node, nodegroups);

        // Create pseudo values
        let values: Vec<PseudoValueCore> = matching_tiles.into_iter()
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
            Ok(SemanticChildResult::Single(values.into_iter().next().unwrap()))
        } else {
            let list = PseudoListCore::from_values_with_cardinality(
                child_alias.to_string(),
                values,
                is_single,
            );
            Ok(SemanticChildResult::List(list))
        }
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
