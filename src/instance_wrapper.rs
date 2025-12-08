use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use crate::graph::{StaticResource, StaticResourceDescriptors};

// Use the new unified tracing infrastructure
use crate::tracing::record_timing;
// WASM wrapper type aliases for return types
use crate::graph::StaticTile as WasmStaticTile;
// Core types for internal storage (avoid WASM wrapper overhead)
use alizarin_core::StaticResourceMetadata as CoreStaticResourceMetadata;
use alizarin_core::StaticTile as CoreStaticTile;
use alizarin_core::StaticTile;
use alizarin_core::StaticNode;
use alizarin_core::StaticNodegroup;
use crate::pseudo_value::{RustPseudoValue, RustPseudoList, WasmPseudoList, WasmPseudoValue, VisitorContext};
use crate::model_wrapper::{WASMResourceModelWrapper};
use js_sys::{Array, Map as JsMap};
use wasm_bindgen::JsCast;
// ============================================================================
// Error types for semantic child value retrieval
// ============================================================================

/// Error type for get_semantic_child_value operations
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

/// Result of get_semantic_child_value - either values or an indication that none exist
#[derive(Debug)]
pub enum SemanticChildResult {
    /// A list of pseudo values (for collectors or multiple matches)
    List(RustPseudoList),
    /// A single pseudo value
    Single(RustPseudoValue),
    /// No matching values found (not an error, just empty)
    Empty,
}

/// Result from values_from_resource_nodegroup
/// Contains structured RustPseudoList values directly (no recipe intermediate)
#[derive(Clone)]
pub struct ValuesFromNodegroupResult {
    /// Map of node alias → RustPseudoList (structured hierarchy)
    pub values: HashMap<String, RustPseudoList>,
    pub implied_nodegroups: Vec<String>,
}

/// WASM wrapper for ValuesFromNodegroupResult
#[wasm_bindgen]
pub struct WasmValuesFromNodegroupResult {
    inner: ValuesFromNodegroupResult,
}

#[wasm_bindgen]
impl WasmValuesFromNodegroupResult {
    /// Get all values as a Map in a single boundary crossing
    #[wasm_bindgen(js_name = getAllValues)]
    pub fn get_all_values(&self) -> JsValue {
        let js_map = js_sys::Map::new();
        for (alias, rust_list) in &self.inner.values {
            let wasm_list = WasmPseudoList::from_rust(rust_list.clone());
            js_map.set(&JsValue::from_str(alias), &wasm_list.into());
        }
        js_map.into()
    }

    #[wasm_bindgen(getter = impliedNodegroups)]
    pub fn implied_nodegroups(&self) -> Vec<String> {
        self.inner.implied_nodegroups.clone()
    }
}

/// Result from ensure_nodegroup
/// PORT: Phase 4c - Now returns structured RustPseudoList values directly
pub struct EnsureNodegroupResult {
    /// Structured values by alias
    /// PORT: Map of alias → RustPseudoList (js/graphManager.ts:350 - newValues Map)
    pub values: HashMap<String, RustPseudoList>,
    pub implied_nodegroups: Vec<String>,
    pub all_nodegroups_map: HashMap<String, bool>,
}

// WASM wrapper for EnsureNodegroupResult
#[wasm_bindgen]
pub struct WasmEnsureNodegroupResult {
    inner: EnsureNodegroupResult,
}

#[wasm_bindgen]
impl WasmEnsureNodegroupResult {
    /// Get value aliases (keys)
    /// PORT: js/graphManager.ts:352 - iterating over result.recipes
    #[wasm_bindgen(js_name = getValueAliases)]
    pub fn get_value_aliases(&self) -> Vec<String> {
        self.inner.values.keys().cloned().collect()
    }

    /// Get a structured value by alias
    /// PORT: js/graphManager.ts:353 - recipe.nodeAlias lookup
    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(&self, alias: &str) -> Option<WasmPseudoList> {
        self.inner.values.get(alias).map(|v| WasmPseudoList::from_rust(v.clone()))
    }

    /// Get all values as a Map in a single boundary crossing
    /// Replaces getValueAliases/getValue loop pattern (N+1 crossings -> 1 crossing)
    #[wasm_bindgen(js_name = getAllValues)]
    pub fn get_all_values(&self) -> JsValue {
        let js_map = js_sys::Map::new();
        for (alias, rust_list) in &self.inner.values {
            let wasm_list = WasmPseudoList::from_rust(rust_list.clone());
            js_map.set(&JsValue::from_str(alias), &wasm_list.into());
        }
        js_map.into()
    }

    /// PORT: js/graphManager.ts - impliedNodegroups access
    #[wasm_bindgen(getter = impliedNodegroups)]
    pub fn implied_nodegroups(&self) -> Vec<String> {
        self.inner.implied_nodegroups.clone()
    }

    /// PORT: js/graphManager.ts - allNodegroupsMap access
    #[wasm_bindgen(getter = allNodegroupsMap)]
    pub fn all_nodegroups_map(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.all_nodegroups_map).unwrap_or(JsValue::NULL)
    }
}

/// Internal result from populate with structured values
/// PORT: Phase 4c - Matches js/graphManager.ts:724-729 (allValues, allNodegroups)
pub struct PopulateResult {
    /// Map of alias → RustPseudoList
    pub values: HashMap<String, RustPseudoList>,
    pub all_values_map: HashMap<String, Option<bool>>,
    pub all_nodegroups_map: HashMap<String, bool>,
}

/// WASM wrapper for PopulateResult
/// PORT: Phase 4c - Exposes structured values to JavaScript
#[wasm_bindgen]
pub struct WasmPopulateResult {
    inner: PopulateResult,
}

#[wasm_bindgen]
impl WasmPopulateResult {
    /// PORT: js/graphManager.ts:669 - iterating over result to get aliases
    #[wasm_bindgen(js_name = getValueAliases)]
    pub fn get_value_aliases(&self) -> Vec<String> {
        self.inner.values.keys().cloned().collect()
    }

    /// PORT: js/graphManager.ts:670 - accessing value by alias
    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(&self, alias: &str) -> Option<WasmPseudoList> {
        self.inner.values.get(alias).map(|v| WasmPseudoList::from_rust(v.clone()))
    }

    /// Get all values as a Map in a single boundary crossing
    /// Replaces getValueAliases/getValue loop pattern (N+1 crossings -> 1 crossing)
    #[wasm_bindgen(js_name = getAllValues)]
    pub fn get_all_values(&self) -> JsValue {
        let js_map = js_sys::Map::new();
        for (alias, rust_list) in &self.inner.values {
            let wasm_list = WasmPseudoList::from_rust(rust_list.clone());
            js_map.set(&JsValue::from_str(alias), &wasm_list.into());
        }
        js_map.into()
    }

    #[wasm_bindgen(getter = allValuesMap)]
    pub fn all_values_map(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.all_values_map).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter = allNodegroupsMap)]
    pub fn all_nodegroups_map(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.all_nodegroups_map).unwrap_or(JsValue::NULL)
    }
}

/// Core resource instance wrapper - WASM-independent business logic
/// Contains all tile storage, indexing, and business logic
/// Can be used from WASM, Python, or other bindings
pub struct ResourceInstanceWrapperCore {
    // Graph ID to look up model core from registry
    pub(crate) graph_id: String,

    // Resource ID
    pub(crate) resource_instance: Option<CoreStaticResourceMetadata>,

    // Core tile storage
    pub(crate) tiles: Option<HashMap<String, CoreStaticTile>>,

    // Index: nodegroup_id -> list of tile_ids
    pub(crate) nodegroup_index: HashMap<String, Vec<String>>,

    // Track which nodegroups have been loaded/loading
    // Phase 4g: Now uses Mutex for thread-safe parallel access
    pub(crate) loaded_nodegroups: Arc<Mutex<HashMap<String, LoadState>>>,

    // Phase 4g: Cache of PseudoValues (alias -> RustPseudoList)
    // This allows Rust to own the authoritative data and bindings to create lightweight wrappers
    pub(crate) pseudo_cache: Arc<Mutex<HashMap<String, RustPseudoList>>>,

    // Cached Arc references to model indices - avoids cloning on every nodegroup access
    // These are populated once on first access and shared across all calls
    pub(crate) cached_nodes: Option<Arc<HashMap<String, Arc<StaticNode>>>>,
    pub(crate) cached_edges: Option<Arc<HashMap<String, Vec<String>>>>,
    pub(crate) cached_reverse_edges: Option<Arc<HashMap<String, Vec<String>>>>,
    pub(crate) cached_nodes_by_nodegroup: Option<Arc<HashMap<String, Vec<Arc<StaticNode>>>>>,
    pub(crate) cached_nodegroups: Option<Arc<HashMap<String, Arc<StaticNodegroup>>>>,
}

/// Rust-side WASM resource instance wrapper
/// Thin wrapper around ResourceInstanceWrapperCore for WASM bindings
/// Phase 4g: Added PseudoValue cache for parallel access
/// Phase 4h: Using RefCell for interior mutability to allow &self in async functions
///           This prevents deadlocks when multiple async operations access the same wrapper
#[wasm_bindgen]
pub struct WASMResourceInstanceWrapper {
    /// Core implementation wrapped in RefCell for interior mutability
    /// This allows &self methods to mutate internal state, which is required
    /// for async wasm_bindgen functions that can't use &mut self without deadlocking
    core: RefCell<ResourceInstanceWrapperCore>,
    /// Optional callback for lazy-loading tiles per nodegroup
    /// Signature: (nodegroup_id: Option<string>) => Promise<StaticTile[]>
    /// None = load all tiles, Some(id) = load tiles for specific nodegroup
    tile_loader: RefCell<Option<js_sys::Function>>,
    /// Whether this instance uses lazy tile loading
    lazy: RefCell<bool>,
}

/// Phase 4g: Track loading state to prevent race conditions
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub(crate) enum LoadState {
    NotLoaded,
    Loading,
    Loaded,
}

impl ResourceInstanceWrapperCore {
    /// Create a new core from graph ID
    pub fn new_from_graph_id(graph_id: String) -> Self {
        // Ensure nodes are built and cache Arc refs to model indices
        let (cached_nodes, cached_edges, cached_reverse_edges, cached_nodes_by_nodegroup, cached_nodegroups) =
            crate::model_wrapper::MODEL_REGISTRY.with(|registry| {
                if let Some(core_arc) = registry.borrow().get(&graph_id) {
                    let mut core = core_arc.borrow_mut();
                    // Only build if not already built
                    if core.get_nodes_internal().is_none() {
                        core.build_nodes().ok();
                    }
                    // Get Arc refs to all indices (cheap refcount increment)
                    (
                        core.get_nodes_arc(),
                        core.get_edges_arc(),
                        core.get_reverse_edges_arc(),
                        core.get_nodes_by_nodegroup_arc(),
                        core.get_nodegroups_arc(),
                    )
                } else {
                    (None, None, None, None, None)
                }
            });

        ResourceInstanceWrapperCore {
            graph_id,
            resource_instance: None,
            tiles: None,
            nodegroup_index: HashMap::new(),
            loaded_nodegroups: Arc::new(Mutex::new(HashMap::new())),
            pseudo_cache: Arc::new(Mutex::new(HashMap::new())),
            cached_nodes,
            cached_edges,
            cached_reverse_edges,
            cached_nodes_by_nodegroup,
            cached_nodegroups,
        }
    }

    /// Create a new core from resource
    pub fn new_from_resource(resource: &StaticResource) -> Self {
        // Access the inner core type through Deref
        let mut core = Self::new_from_graph_id(resource.0.resourceinstance.graph_id.clone());
        core.resource_instance = Some(resource.0.resourceinstance.clone());
        core
    }

    /// Helper to access the model core from registry (immutable)
    /// Generic error type for WASM-independent usage
    fn with_model_core<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&crate::model_wrapper::ResourceModelWrapperCore) -> Result<R, E>,
        E: From<String>,
    {
        crate::model_wrapper::MODEL_REGISTRY.with(|registry| {
            let registry_borrow = registry.borrow();
            let core_arc = registry_borrow.get(&self.graph_id)
                .ok_or_else(|| E::from(format!("Model not found in registry: {}", self.graph_id)))?;
            let core_borrow = core_arc.borrow();
            f(&*core_borrow)
        })
    }

    /// Helper to access the model core from registry (mutable)
    /// Generic error type for WASM-independent usage
    fn with_model_core_mut<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&mut crate::model_wrapper::ResourceModelWrapperCore) -> Result<R, E>,
        E: From<String>,
    {
        crate::model_wrapper::MODEL_REGISTRY.with(|registry| {
            let registry_borrow = registry.borrow();
            let core_arc = registry_borrow.get(&self.graph_id)
                .ok_or_else(|| E::from(format!("Model not found in registry: {}", self.graph_id)))?;
            let mut core_borrow = core_arc.borrow_mut();
            f(&mut *core_borrow)
        })
    }

    /// Check if a tile matches semantic parent-child relationship criteria
    /// PORT: js/semantic.ts lines 297-340 - the complex conditional logic
    ///
    /// This implements the exact logic from __getChildValues to determine if a value
    /// should be included as a child of a semantic node.
    ///
    /// We do not use edges, as this checks directly saving an access to the model.
    ///
    /// Note: This is public to allow test access
    pub fn matches_semantic_child(
        parent_tile_id: Option<&String>,
        parent_node_id: &str,
        child_node: &StaticNode,
        tile: &StaticTile,
    ) -> bool {
        // TODO: double check this addition
        if tile.nodegroup_id != *child_node.nodegroup_id.as_ref().unwrap_or(&"".into()) {
            return false;
        }
        // We do not have a child value, unless there is a value, or the whole tile is the
        // (semantic) value.
        // RMV: double check for any other case
        if !(Some(&child_node.nodeid) == child_node.nodegroup_id.as_ref() || tile.data.contains_key(&child_node.nodeid)) {
            return false;
        }

        // PORT: js/semantic.ts lines 311-315
        // Branch 1: Different nodegroup + correct parent tile relationship
        if tile.nodegroup_id != parent_node_id {
            if let (t, Some(parent_tid)) = (tile, parent_tile_id) {
                // Check if tile.parenttile_id is null or equals parent_tid
                // PORT: Line 311 - (!(value.tile.parenttile_id) || value.tile.parenttile_id == tile.tileid)
                let parent_matches = t.parenttile_id.is_none()
                    || t.parenttile_id.as_ref() == Some(parent_tid);

                if parent_matches {
                    return true;
                }
            }
        }

        // PORT: js/semantic.ts lines 312-315
        // Branch 2: Same nodegroup + shared tile + not collector
        if tile.nodegroup_id == parent_node_id {
            if let (t, Some(parent_tid)) = (tile, parent_tile_id) {
                // Check if this tile IS the parent tile and child is not a collector
                // PORT: Line 314 - value.tile == tile && !childNode.is_collector
                if t.tileid.as_ref() == Some(parent_tid) && !child_node.is_collector && t.data.contains_key(&child_node.nodeid) {
                    return true;
                }
            }
        }

        // PORT: js/semantic.ts lines 324-327
        // Branch 3: Different nodegroup + is_collector
        // This handles collector nodes that don't share tiles with their parent
        if tile.nodegroup_id != parent_node_id
            && child_node.is_collector {
            return true;
        }

        false
    }

    /// Pure Rust implementation of get_semantic_child_value
    /// Returns semantic child values for a given parent node and child alias.
    ///
    /// Parameters:
    /// - parent_tile_id: The tileid of the parent semantic node (or None)
    /// - parent_node_id: The nodeid of the parent semantic node
    /// - parent_nodegroup_id: The nodegroup_id of the parent node (or None)
    /// - child_alias: The alias of the specific child to retrieve
    /// - loaded_nodegroups: Set of nodegroups that have been loaded (None = non-lazy mode, all loaded)
    ///
    /// Returns: SemanticChildResult or SemanticChildError
    pub fn get_semantic_child_value(
        &self,
        parent_tile_id: Option<&String>,
        parent_node_id: &str,
        _parent_nodegroup_id: Option<&String>,
        child_alias: &str,
        loaded_nodegroups: Option<&HashSet<String>>,
    ) -> Result<SemanticChildResult, SemanticChildError> {
        // Get child nodes for this parent (needs mutable access for lazy init)
        let child_nodes = self.with_model_core_mut(|core| {
            core.get_child_nodes(parent_node_id)
                .map_err(|e| SemanticChildError::ModelNotInitialized(e))
                .map(|map| map.clone())
        })?;

        // Get edges for creating PseudoValues
        // let edges = self.with_model_core(|core| {
        //     core.get_edges_internal()
        //         .ok_or_else(|| SemanticChildError::ModelNotInitialized("Model edges not initialized".to_string()))
        //         .map(|map| map.clone())
        // })?;

        // Get the specific child node we're looking for (core type)
        let child_node = child_nodes.get(child_alias)
            .ok_or_else(|| SemanticChildError::ChildNotFound { alias: child_alias.to_string() })?
            .clone();

        // Check if tiles are loaded for this child's nodegroup (lazy mode check)
        if let Some(loaded_ngs) = loaded_nodegroups {
            if let Some(ref child_nodegroup_id) = child_node.nodegroup_id {
                if !loaded_ngs.contains(child_nodegroup_id) {
                    return Err(SemanticChildError::TilesNotLoaded {
                        nodegroup_id: child_nodegroup_id.clone()
                    });
                }
            }
        }

        // Get tiles (must be initialized)
        let tiles = self.tiles
            .as_ref()
            .ok_or(SemanticChildError::TilesNotInitialized)?;

        // Find all tiles that contain this child node with the correct semantic relationship
        // Use nodegroup_index for O(1) lookup instead of O(n) tile scan
        let mut matching_tile_ids: Vec<String> = Vec::new();

        // Get candidate tile IDs from the nodegroup index
        let candidate_tile_ids: Vec<String> = if let Some(ref child_nodegroup_id) = child_node.nodegroup_id {
            self.nodegroup_index
                .get(child_nodegroup_id)
                .cloned()
                .unwrap_or_default()
        } else {
            // No nodegroup_id on child node - fall back to scanning all tiles
            tiles.keys().cloned().collect()
        };

        for tile_id in candidate_tile_ids {
            if let Some(tile) = tiles.get(&tile_id) {
                // Check semantic parent-child relationship
                if Self::matches_semantic_child(
                    parent_tile_id,
                    parent_node_id,
                    &*child_node,
                    tile,
                ) {
                    matching_tile_ids.push(tile_id);
                }
            }
        }

        // If no matching tiles, return Empty (not an error)
        if matching_tile_ids.is_empty() {
            return Ok(SemanticChildResult::Empty);
        }

        // Get edges and nodegroups for creating PseudoValues
        let edges = self.with_model_core(|core| {
            core.get_edges_internal()
                .ok_or_else(|| SemanticChildError::ModelNotInitialized("Model edges not initialized".to_string()))
                .map(|map| map.clone())
        })?;

        // Determine if this is a cardinality-1 node (should return single value, not list)
        // A node is cardinality-1 if its nodegroup has cardinality "1"
        let is_cardinality_one = self.with_model_core::<_, _, SemanticChildError>(|core| {
            if let Some(nodegroup_id) = &child_node.nodegroup_id {
                if let Some(nodegroups) = core.get_nodegroups_internal() {
                    if let Some(nodegroup) = nodegroups.get(nodegroup_id) {
                        return Ok(nodegroup.cardinality.as_deref() == Some("1"));
                    }
                }
            }
            // Default to false (treat as cardinality-n) if we can't determine
            Ok(false)
        })?;

        // Create PseudoValues from the matching tiles
        let mut values = Vec::new();
        for tile_id in &matching_tile_ids {
            let tile = tiles.get(tile_id)
                .ok_or_else(|| SemanticChildError::Other("Tile not found".to_string()))?;

            let tile_data = tile.data.get(&child_node.nodeid);
            let child_ids = edges.get(&child_node.nodeid).cloned().unwrap_or_default();

            let pseudo_value = RustPseudoValue::from_node_and_tile(
                Arc::clone(&child_node),
                Some(Arc::new(tile.clone())),
                tile_data.cloned(),
                child_ids,
            );

            values.push(pseudo_value);
        }

        // Create PseudoList or single value based on is_collector and number of values
        if child_node.is_collector || values.len() > 1 {
            // Return as PseudoList, with is_single set based on cardinality
            let pseudo_list = RustPseudoList::from_values_with_cardinality(
                child_alias.to_string(),
                values,
                is_cardinality_one,
            );
            Ok(SemanticChildResult::List(pseudo_list))
        } else if values.len() == 1 {
            // Return single value
            Ok(SemanticChildResult::Single(values.into_iter().next().unwrap()))
        } else {
            Ok(SemanticChildResult::Empty)
        }
    }
}

impl WASMResourceInstanceWrapper {
    /// Helper to access the model core from registry (immutable) - WASM version
    fn with_model_core<F, R>(&self, f: F) -> Result<R, JsValue>
    where
        F: FnOnce(&crate::model_wrapper::ResourceModelWrapperCore) -> Result<R, JsValue>,
    {
        self.core.borrow().with_model_core(f)
    }

    /// Helper to access the model core from registry (mutable) - WASM version
    fn with_model_core_mut<F, R>(&self, f: F) -> Result<R, JsValue>
    where
        F: FnOnce(&mut crate::model_wrapper::ResourceModelWrapperCore) -> Result<R, JsValue>,
    {
        self.core.borrow().with_model_core_mut(f)
    }

    fn check_tiles(&self, tiles: &Vec<StaticTile>) -> Result<(), JsValue> {
        self.with_model_core_mut(|core| {
            let nodegroups = core.get_nodegroup_objects().map_err(|e| JsValue::from_str(&e))?;
            for tile in tiles {
                nodegroups.get(&tile.nodegroup_id)
                    .ok_or_else(|| JsValue::from_str(&format!("Tile {:?} has nodegroup not on model: nodegroup {}", tile.tileid, tile.nodegroup_id)))?;
            }
            Ok(())
        })
    }

    pub fn load_tiles(&self, tiles: Vec<StaticTile>, assume_tiles_comprehensive_for_nodegroup: bool) -> Result<(), JsValue> {
        self.load_tiles_internal(tiles, false, assume_tiles_comprehensive_for_nodegroup)
    }

    /// Append tiles without clearing existing ones
    /// Used by lazy loading callbacks to add tiles incrementally
    pub fn append_tiles(&self, tiles: Vec<StaticTile>, assume_tiles_comprehensive_for_nodegroup: bool) -> Result<(), JsValue> {
        self.load_tiles_internal(tiles, true, assume_tiles_comprehensive_for_nodegroup)
    }

    fn load_tiles_internal(&self, tiles: Vec<StaticTile>, append: bool, assume_tiles_comprehensive_for_nodegroup: bool) -> Result<(), JsValue> {
        self.check_tiles(&tiles)?;

        let lazy = *self.lazy.borrow();

        {
            let mut core = self.core.borrow_mut();
            if core.tiles.is_none() {
                core.tiles = Some(HashMap::new());
            }

            // In non-lazy mode and not appending, clear existing data (replacing all tiles)
            // In lazy mode or when appending, add to existing tiles (incremental loading)
            if !lazy && !append {
                core.tiles.as_mut().unwrap().clear();
                core.nodegroup_index.clear();
            }

            // Store tiles and build index
            for mut tile in tiles {
                // Ensure tile has an ID
                let tile_id = tile.ensure_id();
                let nodegroup_id = tile.nodegroup_id.clone();

                // Add to nodegroup index
                core.nodegroup_index
                    .entry(nodegroup_id.clone())
                    .or_insert_with(Vec::new)
                    .push(tile_id.clone());

                // Mark this nodegroup as loaded in loaded_nodegroups
                // This consolidates tile tracking with nodegroup processing state
                if let Ok(mut loaded) = core.loaded_nodegroups.lock() {
                    loaded.insert(nodegroup_id, LoadState::Loaded);
                }

                // Store tile
                core.tiles.as_mut().unwrap().insert(tile_id.clone(), tile);
            }
        }

        // If assume_tiles_comprehensive_for_nodegroup is true, mark ALL model nodegroups as loaded
        // This handles the case where some nodegroups have no tiles for this resource
        if assume_tiles_comprehensive_for_nodegroup {
            self.with_model_core_mut(|model_core| {
                if let Some(nodegroups) = model_core.get_nodegroups_internal() {
                    if let Ok(mut loaded) = self.core.borrow().loaded_nodegroups.lock() {
                        for nodegroup_id in nodegroups.keys() {
                            loaded.entry(nodegroup_id.clone()).or_insert(LoadState::Loaded);
                        }
                    }
                }
                Ok(())
            })?;
        }

        Ok(())
    }
}

#[wasm_bindgen]
impl WASMResourceInstanceWrapper {
    pub(crate) fn new_from_graph_id(graph_id: String) -> WASMResourceInstanceWrapper {
        WASMResourceInstanceWrapper {
            core: RefCell::new(ResourceInstanceWrapperCore::new_from_graph_id(graph_id)),
            tile_loader: RefCell::new(None),
            lazy: RefCell::new(false),
        }
    }

    // Keep old API for backward compatibility
    #[allow(dead_code)]
    pub(crate) fn new_from_model(model: Arc<RefCell<WASMResourceModelWrapper>>) -> WASMResourceInstanceWrapper {
        let graph_id = model.borrow().get_graph_id();
        Self::new_from_graph_id(graph_id)
    }

    pub(crate) fn new_from_resource(resource: &StaticResource) -> WASMResourceInstanceWrapper {
        WASMResourceInstanceWrapper {
            core: RefCell::new(ResourceInstanceWrapperCore::new_from_resource(resource)),
            tile_loader: RefCell::new(None),
            lazy: RefCell::new(false),
        }
    }

    /// Set whether this wrapper should use lazy tile loading
    #[wasm_bindgen(js_name = setLazy)]
    pub fn set_lazy(&self, lazy: bool) {
        *self.lazy.borrow_mut() = lazy;
        if lazy && self.core.borrow().tiles.is_none() {
            // Initialize empty tiles HashMap for lazy mode
            self.core.borrow_mut().tiles = Some(HashMap::new());
        }
    }

    #[wasm_bindgen(js_name = tilesLoaded)]
    pub fn tiles_loaded(&self) -> bool {
        // Check if tiles exist AND have actual content (not just initialized but empty)
        // In lazy mode, tiles HashMap is initialized empty, so we need to check for actual content
        self.core.borrow().tiles.as_ref().map_or(false, |t| !t.is_empty())
    }

    /// Set a callback function for lazy-loading tiles
    /// The callback should have signature: (nodegroup_id: Option<string>) => Promise<StaticTile[]>
    /// where None/null means load all tiles, Some(id) means load tiles for specific nodegroup
    #[wasm_bindgen(js_name = setTileLoader)]
    pub fn set_tile_loader(&self, loader: js_sys::Function) {
        *self.tile_loader.borrow_mut() = Some(loader);
    }

    /// Get the tile loader callback if set
    #[wasm_bindgen(js_name = getTileLoader)]
    pub fn get_tile_loader(&self) -> Option<js_sys::Function> {
        self.tile_loader.borrow().clone()
    }

    /// Request tiles for a specific nodegroup (or all tiles if nodegroup_id is None)
    /// Returns the tile loader callback with the nodegroup_id parameter
    /// JavaScript should await this and call loadTiles() with the result
    #[wasm_bindgen(js_name = requestTilesForNodegroup)]
    pub fn request_tiles_for_nodegroup(&self, _nodegroup_id: Option<String>) -> Result<js_sys::Function, JsValue> {
        let loader = self.tile_loader.borrow().clone()
            .ok_or_else(|| JsValue::from_str("No tile loader callback set"))?;
        Ok(loader)
    }

    /// Check if tiles have been loaded for a specific nodegroup
    /// In non-lazy mode, returns true if tiles exist
    /// In lazy mode, checks if nodegroup has been processed (which includes tile loading)
    #[wasm_bindgen(js_name = hasTilesForNodegroup)]
    pub fn has_tiles_for_nodegroup(&self, nodegroup_id: String) -> bool {
        if !*self.lazy.borrow() {
            // Non-lazy mode: if tiles exist, we have all tiles
            return self.core.borrow().tiles.is_some();
        }
        // Lazy mode: check if this specific nodegroup has been loaded via loaded_nodegroups
        if let Ok(loaded) = self.core.borrow().loaded_nodegroups.lock() {
            matches!(loaded.get(&nodegroup_id), Some(LoadState::Loaded))
        } else {
            false
        }
    }

    /// Load tiles from plain JS objects (JSON deserialization)
    /// Use this for initial loading from JSON files
    #[wasm_bindgen(js_name = loadTiles)]
    pub fn load_tiles_js(&self, tiles_js: JsValue) -> Result<(), JsValue> {
        let tiles: Vec<StaticTile> = serde_wasm_bindgen::from_value(tiles_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize tiles: {:?}", e)))?;
        self.load_tiles(tiles, true)
    }

    /// Load tiles from WASM StaticTile objects directly (no serialization overhead)
    /// Use this when passing tiles between Rust/WASM contexts
    #[wasm_bindgen(js_name = loadTilesWasm)]
    pub fn load_tiles_wasm(&self, tiles: Vec<WasmStaticTile>, assume_tiles_comprehensive_for_nodegroup: Option<bool>) -> Result<(), JsValue> {
        // Extract inner CoreStaticTile from each WASM wrapper
        let core_tiles: Vec<StaticTile> = tiles.into_iter()
            .map(|wasm_tile| wasm_tile.0)
            .collect();

        self.load_tiles(core_tiles, assume_tiles_comprehensive_for_nodegroup.unwrap_or(true))
    }

    #[wasm_bindgen(js_name = getResourceId)]
    pub fn get_resource_id(&self) -> Option<String> {
        self.core.borrow().resource_instance.as_ref().map(|r| r.resourceinstanceid.clone())
    }

    /// Get resource name from metadata
    /// PORT: js/graphManager.ts:115-125 (getName method)
    #[wasm_bindgen(js_name = getName)]
    pub fn get_name(&self) -> String {
        self.core.borrow().resource_instance
            .as_ref()
            .map(|r| {
                // Get name from metadata, or from descriptors
                if !r.name.is_empty() {
                    r.name.clone()
                } else if let Some(descriptor_name) = &r.descriptors.name {
                    descriptor_name.clone()
                } else {
                    "<Unnamed>".to_string()
                }
            })
            .unwrap_or_else(|| "<Unnamed>".to_string())
    }

    /// Get resource descriptors from metadata
    /// PORT: js/graphManager.ts:127-186 (getDescriptors method)
    ///
    /// Note: This returns the cached descriptors. For computed descriptors from
    /// templates, use computeDescriptors() instead.
    #[wasm_bindgen(js_name = getDescriptors)]
    pub fn get_descriptors(&self) -> Option<StaticResourceDescriptors> {
        self.core.borrow().resource_instance
            .as_ref()
            .map(|r| StaticResourceDescriptors(r.descriptors.clone()))
    }

    /// Compute resource descriptors from tiles using graph configuration
    /// PORT: js/graphManager.ts:127-186 + js/utils.ts:149-274 (buildResourceDescriptors)
    ///
    /// This computes descriptors by processing descriptor templates in the graph configuration
    /// and replacing placeholders with actual values from the resource's tiles.
    ///
    /// Returns computed StaticResourceDescriptors, or empty descriptors if:
    /// - No tiles are loaded
    /// - No descriptor configuration exists in the graph
    /// - Required template values are missing
    #[wasm_bindgen(js_name = computeDescriptors)]
    pub fn compute_descriptors(&self) -> Result<StaticResourceDescriptors, JsValue> {
        use alizarin_core::IndexedGraph;

        // Get tiles - return empty descriptors if no tiles loaded
        // Store the borrow so it lives long enough
        let core_ref = self.core.borrow();
        let tiles = core_ref.tiles
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No tiles loaded"))?;

        // Get the graph from model core
        let graph = self.with_model_core(|core| {
            Ok(core.get_graph().clone())
        })?;

        // Create IndexedGraph for efficient descriptor building
        let indexed_graph = IndexedGraph::new(graph);

        // Convert tiles HashMap to Vec for the build_descriptors call
        let tiles_vec: Vec<_> = tiles.values().cloned().collect();

        // Compute descriptors using the Rust implementation
        let descriptors = indexed_graph.build_descriptors(&tiles_vec);

        // Wrap in WASM type and return
        Ok(StaticResourceDescriptors(descriptors))
    }

    /// Get tile IDs for a specific nodegroup
    /// Returns array of tile ID strings
    #[wasm_bindgen(js_name = getTileIdsByNodegroup)]
    pub fn get_tile_ids_by_nodegroup(&self, nodegroup_id: &str) -> Vec<String> {
        self.core.borrow().nodegroup_index
            .get(nodegroup_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get full tile data by tile ID
    /// Returns StaticTile WASM object or error if not found
    #[wasm_bindgen(js_name = getTile)]
    pub fn get_tile(&self, tile_id: &str) -> Result<WasmStaticTile, JsValue> {
        self.core.borrow().tiles
            .as_ref()
            .ok_or_else(|| JsValue::from_str(&format!("No tiles loaded: {}", tile_id)))?
            .get(tile_id)
            .cloned()
            .map(WasmStaticTile)
            .ok_or_else(|| JsValue::from_str(&format!("Tile not found: {}", tile_id)))
    }

    /// Get specific node data from a tile
    /// Returns the data value for the given node_id within the tile
    #[wasm_bindgen(js_name = getTileData)]
    pub fn get_tile_data(&self, tile_id: &str, node_id: &str) -> Result<JsValue, JsValue> {
        // Store the borrow so it lives long enough
        let core_ref = self.core.borrow();
        let tile = core_ref.tiles
            .as_ref()
            .ok_or_else(|| JsValue::from_str(&format!("No tiles loaded: {}", tile_id)))?
            .get(tile_id)
            .ok_or_else(|| JsValue::from_str(&format!("Tile not found: {}", tile_id)))?;

        // Get data for specific node from tile.data HashMap
        match tile.data.get(node_id) {
            Some(value) => serde_wasm_bindgen::to_value(value)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize data: {:?}", e))),
            None => Ok(JsValue::NULL),
        }
    }

    /// Phase 4g: Check if a nodegroup is being loaded or already loaded
    #[wasm_bindgen(js_name = isNodegroupLoadedOrLoading)]
    pub fn is_nodegroup_loaded_or_loading(&self, nodegroup_id: &str) -> bool {
        if let Ok(loaded) = self.core.borrow().loaded_nodegroups.lock() {
            matches!(loaded.get(nodegroup_id), Some(LoadState::Loading) | Some(LoadState::Loaded))
        } else {
            false
        }
    }

    /// Phase 4g: Try to atomically acquire loading lock for a nodegroup
    /// Returns true if caller should proceed with loading, false if already being loaded
    #[wasm_bindgen(js_name = tryAcquireNodegroupLock)]
    pub fn try_acquire_nodegroup_lock(&self, nodegroup_id: String) -> bool {
        if let Ok(mut loaded) = self.core.borrow().loaded_nodegroups.lock() {
            match loaded.get(&nodegroup_id) {
                Some(LoadState::Loading) | Some(LoadState::Loaded) => false,
                _ => {
                    loaded.insert(nodegroup_id, LoadState::Loading);
                    true
                }
            }
        } else {
            false
        }
    }

    /// Phase 4g: Get cached PseudoValue from Rust cache
    /// Returns WasmPseudoList if found, null otherwise
    #[wasm_bindgen(js_name = getCachedPseudo)]
    pub fn get_cached_pseudo(&self, alias: &str) -> Option<WasmPseudoList> {
        if let Ok(cache) = self.core.borrow().pseudo_cache.lock() {
            if let Some(pseudo_list) = cache.get(alias) {
                // Convert RustPseudoList to WasmPseudoList
                return Some(WasmPseudoList::from_rust(pseudo_list.clone()));
            }
        }
        None
    }

    /// Phase 4g: Store WasmPseudoList in Rust cache
    #[wasm_bindgen(js_name = cachePseudoList)]
    pub fn cache_pseudo_list(&self, alias: String, wasm_list: WasmPseudoList) {
        let rust_list = wasm_list.into_inner();
        if let Ok(mut cache) = self.core.borrow().pseudo_cache.lock() {
            cache.insert(alias, rust_list);
        }
    }

    #[wasm_bindgen(js_name = pruneResourceTiles)]
    pub fn prune_resource_tiles(&mut self) -> Result<(), JsValue> {
        let tiles = self.core.borrow().tiles
            .to_owned()
            .ok_or_else(|| JsValue::from_str("Tiles not initialized"))?;

        let pruned_tiles: HashMap<String, StaticTile> = self.with_model_core(|core| {
            Ok(tiles.into_iter().filter(|(_tile_id, tile)| {
                core.is_nodegroup_permitted(&tile.nodegroup_id)
            }).collect())
        })?;

        // Use borrow_mut() for assignment
        self.core.borrow_mut().tiles = Some(pruned_tiles);
        Ok(())
    }

    /// Phase 4g: Store single WasmPseudoValue as a list in Rust cache
    #[wasm_bindgen(js_name = cachePseudoValue)]
    pub fn cache_pseudo_value(&self, alias: String, wasm_value: WasmPseudoValue) {
        let rust_value = wasm_value.into_inner();
        let node_alias = alias.clone();  // Use the provided alias
        let rust_list = RustPseudoList {
            node_alias,
            values: vec![rust_value],
            is_loaded: true,
            is_single: true, // Single value being cached
        };
        if let Ok(mut cache) = self.core.borrow().pseudo_cache.lock() {
            cache.insert(alias, rust_list);
        }
    }

    /// Phase 4g: Clear all cached PseudoValues
    #[wasm_bindgen(js_name = clearPseudoCache)]
    pub fn clear_pseudo_cache(&self) {
        if let Ok(mut cache) = self.core.borrow().pseudo_cache.lock() {
            cache.clear();
        }
    }

    /// Get the root pseudo value from the cache.
    ///
    /// Returns the first (and should be only) value from the root pseudo list.
    /// The root is stored in cache with its alias from the graph model.
    ///
    /// PORT: js/graphManager.ts:330-364 getRoot()
    #[wasm_bindgen(js_name = getRootPseudo)]
    pub fn get_root_pseudo(&self) -> Option<WasmPseudoValue> {
        // Get root node to find its alias
        let root_node = self.with_model_core_mut(|core| {
            Ok(core.get_root_node().ok())
        }).ok()??;

        let root_alias = root_node.alias.clone().unwrap_or_default();

        // Need to hold borrow long enough for the lock
        let core_ref = self.core.borrow();
        let cache = core_ref.pseudo_cache.lock().ok()?;

        // Look up root by its actual alias
        let root_list = cache.get(&root_alias)?;

        // Return the first value (root should have exactly one)
        // PORT: js/graphManager.ts:346-350 - if list, get first element
        if root_list.values.len() > 1 {
            web_sys::console::warn_1(&"Multiple root tiles found - returning first".into());
        }

        root_list.values.first().map(|v| WasmPseudoValue::from_rust(v.clone()))
    }

    /// Convert the entire resource to JSON using Rust-side traversal.
    ///
    /// Check that all tiles have been loaded before serialization.
    /// Returns an error if any nodegroups are not in Loaded state.
    fn check_tiles_loaded(&self, method_name: &str) -> Result<(), JsValue> {
        let core_ref = self.core.borrow();
        let loaded_nodegroups = core_ref.loaded_nodegroups.lock()
            .map_err(|e| JsValue::from_str(&format!("Failed to lock loaded_nodegroups: {}", e)))?;

        if loaded_nodegroups.is_empty() {
            return Err(JsValue::from_str(&format!(
                "{}() called but no tiles have been loaded. \
                 Call populate() or ensure tiles are loaded before serialization.",
                method_name
            )));
        }

        // Check for nodegroups that have tiles but aren't loaded
        let unloaded: Vec<String> = loaded_nodegroups.iter()
            .filter(|(_, state)| **state != LoadState::Loaded)
            .map(|(id, _)| id.clone())
            .collect();

        if !unloaded.is_empty() {
            return Err(JsValue::from_str(&format!(
                "{}() called but {} nodegroup(s) have not been loaded: {}. \
                 Ensure all tiles are loaded before serialization.",
                method_name,
                unloaded.len(),
                unloaded.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
            )));
        }

        Ok(())
    }

    /// This replaces the JS `forJson()` method that makes 98+ WASM boundary
    /// crossings per resource. By doing the traversal entirely in Rust,
    /// we only cross the boundary once.
    ///
    /// Prerequisites: `populate()` must have been called to fill the pseudo_cache.
    /// Fails with an error if tiles have not been loaded.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        let start = js_sys::Date::now();

        // Get root node from model to find its alias
        let root_node = self.with_model_core_mut(|core| {
            core.get_root_node().map_err(|e| JsValue::from_str(&e))
        })?;

        let root_alias = root_node.alias.clone().unwrap_or_default();

        // Ensure nodes_by_alias and edges are built
        self.with_model_core_mut(|core| {
            if core.get_nodes_by_alias_internal().is_none() {
                core.build_nodes().map_err(|e| JsValue::from_str(&e))?;
            }
            Ok(())
        })?;

        let (nodes_by_alias, edges) = self.with_model_core(|core| {
            let nodes = core.get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("nodes_by_alias not built"))?
                .clone();
            let edges = core.get_edges_internal()
                .ok_or_else(|| JsValue::from_str("edges not built"))?
                .clone();
            Ok((nodes, edges))
        })?;

        // Check that all tiles have been loaded - fail fast if not
        self.check_tiles_loaded("toJson")?;

        // Get the pseudo_cache (populated by populate())
        let core_ref = self.core.borrow();
        let cache = core_ref.pseudo_cache.lock()
            .map_err(|e| JsValue::from_str(&format!("Failed to lock pseudo_cache: {}", e)))?;

        // Build visitor context
        let ctx = VisitorContext {
            pseudo_cache: &cache,
            nodes_by_alias: &nodes_by_alias,
            edges: &edges,
            depth: 0,
            max_depth: 50, // Reasonable limit to prevent infinite recursion
        };

        // Look up the root pseudo from cache (created in populate())
        let root_list = cache.get(&root_alias)
            .ok_or_else(|| JsValue::from_str(&format!(
                "Root pseudo not found in cache for alias '{}' - was populate() called?",
                root_alias
            )))?;

        // Use the root list's to_json - the root is a semantic node, so it traverses children
        let json = root_list.to_json(&ctx);

        // Convert serde_json::Value to JSON string, then parse to JS object
        // Note: Using JSON.parse is more reliable than serde_wasm_bindgen::to_value
        // which can return empty objects in some cases
        let json_string = serde_json::to_string(&json)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize JSON: {}", e)))?;

        let result = js_sys::JSON::parse(&json_string)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {:?}", e)))?;

        let end = js_sys::Date::now();
        record_timing("toJson (Rust)", end - start);

        Ok(result)
    }

    /// Convert the pseudo cache to a list of StaticTiles.
    ///
    /// Traverses the cache using the same visitor pattern as toJson(),
    /// but collects tile data into StaticTile structures.
    ///
    /// Returns an array of StaticTile objects ready for serialization.
    #[wasm_bindgen(js_name = toTiles)]
    pub fn to_tiles(&self) -> Result<JsValue, JsValue> {
        use crate::pseudo_value::{TileBuilder, TileBuilderContext};

        let start = js_sys::Date::now();

        // Get root node from model
        let root_node = self.with_model_core_mut(|core| {
            core.get_root_node().map_err(|e| JsValue::from_str(&e))
        })?;

        let root_alias = root_node.alias.clone().unwrap_or_default();

        // Check that all tiles have been loaded - fail fast if not
        self.check_tiles_loaded("toTiles")?;

        // Get resource instance ID
        let resource_id = self.core.borrow().resource_instance
            .as_ref()
            .map(|r| r.resourceinstanceid.clone())
            .unwrap_or_default();

        // Get nodes and edges
        let nodes_by_alias = self.with_model_core(|core| {
            core.get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("Model nodes not initialized"))
                .map(|map| map.clone())
        })?;

        let edges = self.with_model_core(|core| {
            core.get_edges_internal()
                .ok_or_else(|| JsValue::from_str("Model edges not initialized"))
                .map(|map| map.clone())
        })?;

        // Get the pseudo cache
        let core_ref = self.core.borrow();
        let cache = core_ref.pseudo_cache.lock()
            .map_err(|e| JsValue::from_str(&format!("Failed to lock pseudo_cache: {}", e)))?;

        // Build context for tile collection
        let ctx = TileBuilderContext {
            pseudo_cache: &cache,
            nodes_by_alias: &nodes_by_alias,
            edges: &edges,
            resourceinstance_id: resource_id,
            depth: 0,
            max_depth: 50,
        };

        // Get root from cache
        let root_list = cache.get(&root_alias)
            .ok_or_else(|| JsValue::from_str(&format!(
                "Root pseudo not found in cache for alias '{}' - was populate() called?",
                root_alias
            )))?;

        // Collect tiles from the tree
        let mut tiles: std::collections::HashMap<String, TileBuilder> = std::collections::HashMap::new();
        root_list.collect_tiles(&ctx, &mut tiles);

        // Convert to StaticTiles and serialize
        let static_tiles: Vec<alizarin_core::StaticTile> = tiles.values()
            .map(|builder| builder.to_static_tile())
            .collect();

        // Convert to JSON string then parse to JS (same pattern as toJson)
        let json_string = serde_json::to_string(&static_tiles)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize tiles: {}", e)))?;

        let result = js_sys::JSON::parse(&json_string)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse tiles JSON: {:?}", e)))?;

        let end = js_sys::Date::now();
        record_timing("toTiles (Rust)", end - start);

        Ok(result)
    }

    /// Convert the instance to a complete StaticResource.
    ///
    /// Combines the resource metadata with tiles built from the pseudo cache.
    /// This is the inverse of loading a resource - it produces a serializable
    /// StaticResource that can be sent to an API for saving.
    ///
    /// Returns a StaticResource object ready for serialization.
    #[wasm_bindgen(js_name = toResource)]
    pub fn to_resource(&self) -> Result<JsValue, JsValue> {
        use crate::pseudo_value::{TileBuilder, TileBuilderContext};

        let start = js_sys::Date::now();

        // Check that all tiles have been loaded - fail fast if not
        self.check_tiles_loaded("toResource")?;

        // Get the original resource metadata
        let resource_metadata = self.core.borrow().resource_instance.clone()
            .ok_or_else(|| JsValue::from_str("No resource metadata available"))?;

        // Get root node from model
        let root_node = self.with_model_core_mut(|core| {
            core.get_root_node().map_err(|e| JsValue::from_str(&e))
        })?;

        let root_alias = root_node.alias.clone().unwrap_or_default();

        // Get nodes and edges
        let nodes_by_alias = self.with_model_core(|core| {
            core.get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("Model nodes not initialized"))
                .map(|map| map.clone())
        })?;

        let edges = self.with_model_core(|core| {
            core.get_edges_internal()
                .ok_or_else(|| JsValue::from_str("Model edges not initialized"))
                .map(|map| map.clone())
        })?;

        // Get the pseudo cache
        let core_ref = self.core.borrow();
        let cache = core_ref.pseudo_cache.lock()
            .map_err(|e| JsValue::from_str(&format!("Failed to lock pseudo_cache: {}", e)))?;

        // Build context for tile collection
        let ctx = TileBuilderContext {
            pseudo_cache: &cache,
            nodes_by_alias: &nodes_by_alias,
            edges: &edges,
            resourceinstance_id: resource_metadata.resourceinstanceid.clone(),
            depth: 0,
            max_depth: 50,
        };

        // Get root from cache
        let root_list = cache.get(&root_alias)
            .ok_or_else(|| JsValue::from_str(&format!(
                "Root pseudo not found in cache for alias '{}' - was populate() called?",
                root_alias
            )))?;

        // Collect tiles from the tree
        let mut tiles_map: std::collections::HashMap<String, TileBuilder> = std::collections::HashMap::new();
        root_list.collect_tiles(&ctx, &mut tiles_map);

        // Convert to StaticTiles
        let tiles: Vec<alizarin_core::StaticTile> = tiles_map.values()
            .map(|builder| builder.to_static_tile())
            .collect();

        // Build the complete resource
        let output_resource = alizarin_core::StaticResource {
            resourceinstance: resource_metadata.clone(),
            tiles: Some(tiles),
            metadata: std::collections::HashMap::new(), // No metadata stored in wrapper
            cache: None, // Don't include internal cache in output
            scopes: None, // Don't include scopes in output
            tiles_loaded: Some(true),
        };

        // Convert to JSON string then parse to JS
        let json_string = serde_json::to_string(&output_resource)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize resource: {}", e)))?;

        let result = js_sys::JSON::parse(&json_string)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse resource JSON: {:?}", e)))?;

        let end = js_sys::Date::now();
        record_timing("toResource (Rust)", end - start);

        Ok(result)
    }

    /// Create a PseudoValue or PseudoList from node metadata
    /// PORT: js/pseudos.ts:497-594 makePseudoCls()
    ///
    /// This replaces the TS makePseudoCls logic, returning WasmPseudoValue or WasmPseudoList
    /// that can be wrapped in TS PseudoValue/PseudoList wrappers.
    ///
    /// Parameters:
    /// - alias: Node alias to create pseudo for
    /// - tile_id: Optional tile ID (if node shares parent's tile)
    /// - is_permitted: Whether nodegroup is permitted (precalculated in TS)
    /// - node_objs_js: Map<string, StaticNode> from TS
    /// - edges_js: Map<string, string[]> from TS
    /// - is_single: Force single value (not a list)
    #[wasm_bindgen(js_name = makePseudoValue)]
    pub fn make_pseudo_value(
        &self,
        alias: &str,
        tile_id: Option<String>,
        is_permitted: bool,
        is_single: bool,
    ) -> Result<JsValue, JsValue> {
        // PORT: js/pseudos.ts:506-510 - Get node by alias from model
        // Ensure nodes are built (only if not already built)
        self.with_model_core_mut(|core| {
            if core.get_nodes_internal().is_none() {
                core.build_nodes().map_err(|e| JsValue::from_str(&e))?;
            }
            Ok(())
        })?;

        let node_objs = self.with_model_core(|core| {
            core.get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("Model nodes not initialized"))
                .map(|map| map.clone())
        })?;
        let node = node_objs.get(alias)
            .ok_or_else(|| JsValue::from_str(&format!("Could not find node by alias: {}", alias)))?
            .clone();

        // PORT: js/pseudos.ts:518-532 - Check if this should be a PseudoList
        let is_collector = node.is_collector;
        // PORT: js/pseudos.ts:421 - handle missing nodegroup_id (root node case)
        // If nodegroup_id is None or empty string, use empty string (root nodegroup)
        let nodegroup_id = node.nodegroup_id.as_ref()
            .map(|s| if s.is_empty() { "" } else { s.as_str() })
            .unwrap_or("");

        // Get nodegroup to check cardinality
        // Note: In TS this comes from model.getNodegroupObjects()
        // For now, assume is_collector + !is_single means it's a list
        let should_be_list = is_collector && !is_single;

        // Store the borrow so it lives long enough for the entire function
        let core_ref = self.core.borrow();

        // If tiles aren't loaded, return empty pseudo
        let Some(tiles) = core_ref.tiles.as_ref() else {
            if should_be_list {
                let empty_list = RustPseudoList::new(alias.to_string());
                let wasm_list = WasmPseudoList::from_rust(empty_list);
                return Ok(wasm_list.into());
            } else {
                return Ok(JsValue::null());
            }
        };

        if should_be_list {
            // PORT: js/pseudos.ts:536-562 - Create PseudoList with values from tiles
            if !is_permitted {
                // Return empty list for unpermitted nodegroup
                let empty_list = RustPseudoList::new(alias.to_string());
                let wasm_list = WasmPseudoList::from_rust(empty_list);
                return Ok(wasm_list.into());
            }

            // Get all tiles for this nodegroup
            let tile_ids = self.get_tile_ids_by_nodegroup(nodegroup_id);

            // Get edges from model directly (no deserialization needed)
            let edges = self.with_model_core(|core| {
                core.get_edges_internal()
                    .ok_or_else(|| JsValue::from_str("Model edges not initialized"))
                    .map(|map| map.clone())
            })?;

            // Create list of RustPseudoValues from each tile
            let mut values = Vec::new();

            for tid in tile_ids {
                let tile = tiles.get(&tid);
                if let Some(tile) = tile {
                    // Get tile data for this node
                    let tile_data = tile.data.get(&node.nodeid);

                    // Get child node IDs from edges
                    let child_ids = edges.get(&node.nodeid)
                        .cloned()
                        .unwrap_or_default();

                    // Create RustPseudoValue for this tile
                    let pseudo_value = RustPseudoValue::from_node_and_tile(
                        Arc::clone(&node),
                        Some(Arc::new(tile.clone())),
                        tile_data.cloned(),
                        child_ids,
                    );

                    values.push(pseudo_value);
                }
            }

            let pseudo_list = RustPseudoList::from_values(alias.to_string(), values);
            let wasm_list = WasmPseudoList::from_rust(pseudo_list);
            Ok(wasm_list.into())
        } else {
            // PORT: js/pseudos.ts:563-590 - Create single PseudoValue
            if !is_permitted {
                // Return null/unavailable for unpermitted nodegroup
                // TS will wrap this in PseudoUnavailable
                return Ok(JsValue::NULL);
            }

            // Get the tile if provided
            // NOTE: We do NOT search for tiles here - we trust the tile_id passed from TS
            // The tile may be null, which means the value is "independent" and will be populated later
            let tile = if let Some(tid) = tile_id {
                tiles.get(&tid).cloned()
            } else {
                None
            };

            // Extract tile data if we have a tile
            let tile_data = if let Some(ref t) = tile {
                t.data.get(&node.nodeid).cloned()
            } else {
                None
            };

            // Get child node IDs from edges from model directly
            let edges = self.with_model_core(|core| {
                core.get_edges_internal()
                    .ok_or_else(|| JsValue::from_str("Model edges not initialized"))
                    .map(|map| map.clone())
            })?;
            let child_ids = edges.get(&node.nodeid)
                .cloned()
                .unwrap_or_default();

            // Create RustPseudoValue
            let pseudo_value = RustPseudoValue::from_node_and_tile(
                Arc::clone(&node),
                tile.map(Arc::new),
                tile_data,
                child_ids,
            );

            // Convert to WASM wrapper
            let wasm_value = WasmPseudoValue::from_rust(pseudo_value);
            Ok(wasm_value.into())
        }
    }

    /// Check if a nodegroup has been loaded
    /// Phase 4g: Updated to use Mutex
    #[wasm_bindgen(js_name = isNodegroupLoaded)]
    pub fn is_nodegroup_loaded(&self, nodegroup_id: &str) -> bool {
        if let Ok(loaded) = self.core.borrow().loaded_nodegroups.lock() {
            matches!(loaded.get(nodegroup_id), Some(LoadState::Loaded))
        } else {
            false
        }
    }

    /// Get count of tiles stored
    #[wasm_bindgen(js_name = getTileCount)]
    pub fn get_tile_count(&self) -> usize {
        if let Some(tiles) = self.core.borrow().tiles.as_ref() {
            tiles.len()
        } else {
            0
        }
    }

    /// Get count of nodegroups indexed
    #[wasm_bindgen(js_name = getNodegroupCount)]
    pub fn get_nodegroup_count(&self) -> usize {
        self.core.borrow().nodegroup_index.len()
    }

    // ========================================================================
    // Phase 2: Graph Traversal Helpers
    // ========================================================================

    /// Internal ensureNodegroup implementation with Rust-native types
    /// This function works entirely with Rust types and can recursively call itself
    /// without serialization overhead. We cannot sensibly store the all_nodegroups
    /// or all_values_map, as these represent the state of the dependent language's
    /// viewModels (which may not exist or be ready, for example).
    fn ensure_nodegroup_internal(
        &self,
        all_values_map: &HashMap<String, Option<bool>>,
        all_nodegroups: &mut HashMap<String, bool>,
        nodegroup_id: &str,
        add_if_missing: bool,
        nodegroup_permissions: &HashMap<String, bool>,
        do_implied_nodegroups: bool,
    ) -> Result<EnsureNodegroupResult, JsValue> {
        use std::collections::{HashMap, HashSet};
        let fn_start = js_sys::Date::now();

        // Check sentinel state (line 314)
        let sentinel = all_nodegroups.get(nodegroup_id);
        let should_process = match sentinel {
            Some(&false) => true,        // sentinel === false (force reload)
            Some(&true) => false,        // sentinel === true (already loaded)
            None => add_if_missing,      // sentinel === undefined (key doesn't exist)
        };

        // PORT: Phase 4c - Changed from Vec<PseudoRecipe> to HashMap<String, RustPseudoList>
        // PORT: js/graphManager.ts:350 - newValues is a Map<string, PseudoValue | PseudoList>
        let mut all_values: HashMap<String, RustPseudoList> = HashMap::new();
        let mut implied_nodegroups_set: HashSet<String> = HashSet::new();

        if should_process {
            // Filter tiles by nodegroup_id and permissions (lines 326-328)
            // Phase 4h: Compute tile permission from nodegroup permission
            let t0 = js_sys::Date::now();
            let mut nodegroup_tiles: Vec<String> = Vec::new();

            // If tiles aren't loaded, treat as empty (will trigger tile loading callback if set)
            if let Some(tiles) = self.core.borrow().tiles.as_ref() {
                for (tile_id, tile) in tiles.iter() {
                    if tile.nodegroup_id == nodegroup_id {
                        // Phase 4h: Look up permission by tile's nodegroup_id
                        let permitted = nodegroup_permissions.get(&tile.nodegroup_id).copied().unwrap_or(true);
                        if permitted {
                            nodegroup_tiles.push(tile_id.clone());
                        }
                    }
                }
            }
            record_timing("eng: filter tiles", js_sys::Date::now() - t0);

            // If no tiles and addIfMissing, use empty string to indicate null tile (lines 329-330)
            if nodegroup_tiles.is_empty() && add_if_missing {
                nodegroup_tiles.push(String::new());
            }

            // Call values_from_resource_nodegroup_internal (line 332)
            // PORT: Phase 4c - Use structured values directly (no recipe conversion)
            // PORT: js/graphManager.ts:352 - iterating over result and adding to newValues
            let t1 = js_sys::Date::now();
            let values_result = self.values_from_resource_nodegroup_internal(
                all_values_map.clone(),
                nodegroup_tiles,
                nodegroup_id,
            )?;
            record_timing("eng: values_from_resource_nodegroup_internal", js_sys::Date::now() - t1);

            // Merge structured values into all_values
            // PORT: js/graphManager.ts:353-355 - newValues.set(recipe.nodeAlias, makePseudoCls(...))
            for (alias, pseudo_list) in values_result.values {
                all_values.insert(alias, pseudo_list);
            }

            // Collect implied nodegroups (lines 347-349)
            for ng in values_result.implied_nodegroups.iter() {
                implied_nodegroups_set.insert(ng.clone());
            }

            // Mark nodegroup as loaded (line 350)
            all_nodegroups.insert(nodegroup_id.to_string(), true);

            // Recursive processing of implied nodegroups (lines 355-373)
            if do_implied_nodegroups && !implied_nodegroups_set.is_empty() {
                let implied_list: Vec<String> = implied_nodegroups_set.iter().cloned().collect();

                for implied_ng in implied_list.iter() {
                    // Recursive call to internal version - NO serialization!
                    let implied_result = self.ensure_nodegroup_internal(
                        all_values_map,
                        all_nodegroups,
                        implied_ng,
                        true,  // addIfMissing = true for implied
                        nodegroup_permissions,
                        true,  // doImpliedNodegroups = true
                    )?;

                    // Merge implied values (lines 369-371)
                    // PORT: Phase 4c - Merge RustPseudoList values instead of recipes
                    // PORT: js/graphManager.ts:369-371 - merging newValues from recursive call
                    for (alias, pseudo_list) in implied_result.values {
                        all_values.insert(alias, pseudo_list);
                    }

                    // Update all_nodegroups from recursive call (already mutated in place!)
                    // No need to deserialize and merge - we passed &mut all_nodegroups
                }

                // Clear implied set after processing (line 373)
                implied_nodegroups_set.clear();
            }
        }

        // Return structured result
        record_timing("eng: total", js_sys::Date::now() - fn_start);
        Ok(EnsureNodegroupResult {
            values: all_values,
            implied_nodegroups: implied_nodegroups_set.into_iter().collect(),
            all_nodegroups_map: all_nodegroups.clone(),
        })
    }

    /// Complete ensureNodegroup implementation in Rust
    /// PORT: graphManager.ts lines 302-377 (full ensureNodegroup function)
    /// PORT: Phase 4c - Now returns structured WasmEnsureNodegroupResult instead of recipes
    /// Now gets node_objs, nodegroup_objs, and edges from model core directly
    #[wasm_bindgen(js_name = ensureNodegroup)]
    pub fn ensure_nodegroup(
        &self,
        all_values_js: JsValue,        // HashMap<string, Option<bool>> - pre-serialized by TS
        all_nodegroups_js: JsValue,    // Map<string, boolean | Promise> - mutable
        nodegroup_id: &str,
        add_if_missing: bool,
        nodegroup_permissions_js: JsValue,  // Phase 4h: Map<nodegroupId, boolean>
        do_implied_nodegroups: bool,
    ) -> Result<WasmEnsureNodegroupResult, JsValue> {
        use std::collections::HashMap;

        // Deserialize all_values from pre-serialized HashMap
        let all_values_map: HashMap<String, Option<bool>> = serde_wasm_bindgen::from_value(all_values_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize all_values: {:?}", e)))?;

        // Deserialize all_nodegroups from pre-serialized HashMap
        // true = loaded, false = needs reload, missing key = undefined
        let mut all_nodegroups: HashMap<String, bool> = serde_wasm_bindgen::from_value(all_nodegroups_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize all_nodegroups: {:?}", e)))?;

        // Phase 4h: Deserialize nodegroup permissions
        let nodegroup_permissions: HashMap<String, bool> = serde_wasm_bindgen::from_value(nodegroup_permissions_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize nodegroup_permissions: {:?}", e)))?;

        // Call internal implementation with Rust-native types
        let result = self.ensure_nodegroup_internal(
            &all_values_map,
            &mut all_nodegroups,
            nodegroup_id,
            add_if_missing,
            &nodegroup_permissions,
            do_implied_nodegroups,
        )?;

        // PORT: Phase 4c - Return structured values wrapped in WasmEnsureNodegroupResult
        // PORT: js/graphManager.ts:377 - return { newValues, impliedNodegroups, ... }
        Ok(WasmEnsureNodegroupResult {
            inner: result
        })
    }

    /// Complete populate implementation in Rust
    /// PORT: graphManager.ts lines 600-688 (populate function)
    /// PORT: Phase 4c - Now returns WasmPopulateResult with structured values
    /// Orchestrates loading all nodegroups for a resource
    ///
    /// Optimization: Skips nodegroups that are already loaded (tracked in loaded_nodegroups).
    /// Preserves existing pseudo_cache entries for already-loaded nodegroups.
    #[wasm_bindgen(js_name = populate)]
    pub fn populate(
        &self,
        lazy: bool,
        nodegroup_ids: Vec<String>,
        root_node_alias: String,
        nodegroup_permissions_js: JsValue,  // Phase 4h: Map<nodegroupId, boolean>
    ) -> Result<WasmPopulateResult, JsValue> {
        let populate_start = js_sys::Date::now();

        // Phase 4h: Deserialize nodegroup permissions once
        let t0 = js_sys::Date::now();
        let nodegroup_permissions: HashMap<String, bool> = serde_wasm_bindgen::from_value(nodegroup_permissions_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize nodegroup_permissions: {:?}", e)))?;
        record_timing("populate: deserialize permissions", js_sys::Date::now() - t0);

        // Check if pseudo_cache has been populated (not just tiles loaded).
        // This is the true indicator that populate() has run before.
        // loaded_nodegroups tracks tile loading, but pseudo_cache tracks populate().
        let cache_populated = if let Ok(cache) = self.core.borrow().pseudo_cache.lock() {
            // Cache is "populated" if it has more than just the root node
            // (The root is added at the end of populate, so 2+ entries means real data)
            cache.len() > 1
        } else {
            false
        };

        // Only use optimization if cache was actually populated previously
        let already_loaded: HashSet<String> = if cache_populated {
            if let Ok(loaded) = self.core.borrow().loaded_nodegroups.lock() {
                loaded.iter()
                    .filter(|(_, state)| **state == LoadState::Loaded)
                    .map(|(id, _)| id.clone())
                    .collect()
            } else {
                HashSet::new()
            }
        } else {
            // First time populate() is called - process all nodegroups
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

        // Initialize all nodegroups - already loaded ones start as true, others as false
        for nodegroup_id in nodegroup_ids.iter() {
            let is_loaded = already_loaded.contains(nodegroup_id);
            all_nodegroups.insert(nodegroup_id.clone(), is_loaded);
        }

        // Set root node alias to false (line 626)
        all_values.insert(root_node_alias.clone(), Some(false));

        // PORT: Phase 4c - Collect structured RustPseudoList values instead of recipes
        // PORT: js/graphManager.ts:668 - newValues is a Map<string, PseudoValue | PseudoList>
        // Start with existing cache entries for already-loaded nodegroups
        let mut all_structured_values: HashMap<String, RustPseudoList> = if !already_loaded.is_empty() {
            if let Ok(cache) = self.core.borrow().pseudo_cache.lock() {
                cache.clone()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        // Non-lazy loading: process all nodegroups (only those not already loaded)
        let t1 = js_sys::Date::now();
        if !lazy {
            // Phase 1: Process all nodegroups with doImpliedNodegroups=false (lines 636-653)
            let mut implied_nodegroups_set = HashSet::new();

            for nodegroup_id in nodegroups_to_process.iter() {
                let result = self.ensure_nodegroup_internal(
                    &all_values,
                    &mut all_nodegroups,
                    nodegroup_id,
                    true,  // addIfMissing
                    &nodegroup_permissions,
                    false, // TODO RMV: doImpliedNodegroups = false for phase 1
                )?;
                ensure_nodegroup_count += 1;

                // Collect structured values
                // PORT: Phase 4c - Merge RustPseudoList values from ensure_nodegroup result
                // PORT: js/graphManager.ts:669-720 - Processing result.recipes becomes iterating values
                for (alias, pseudo_list) in result.values {
                    all_structured_values.insert(alias, pseudo_list);
                }

                // Note: all_nodegroups already updated in place via &mut reference

                // Collect implied nodegroups (lines 649-652)
                for implied_ng in result.implied_nodegroups.iter() {
                    if implied_ng != nodegroup_id {
                        implied_nodegroups_set.insert(implied_ng.clone());
                    }
                }
            }

            // Phase 2: Process implied nodegroups iteratively (lines 655-677)
            while !implied_nodegroups_set.is_empty() {
                let current_implied: Vec<String> = implied_nodegroups_set.iter().cloned().collect();
                implied_nodegroups_set.clear();

                for nodegroup_id in current_implied.iter() {
                    // Check sentinel state (lines 658-659)
                    let current_value = all_nodegroups.get(nodegroup_id);
                    let should_process = match current_value {
                        Some(&false) => true,  // false = needs reload
                        None => true,          // undefined = needs load
                        _ => false,            // true = already loaded
                    };

                    if should_process {
                        let result = self.ensure_nodegroup_internal(
                            &all_values,
                            &mut all_nodegroups,
                            nodegroup_id,
                            true,  // addIfMissing
                            &nodegroup_permissions,
                            true,  // doImpliedNodegroups = true for phase 2
                        )?;
                        ensure_nodegroup_count += 1;

                        // Collect structured values
                        // PORT: Phase 4c - Merge structured values from implied nodegroup
                        for (alias, pseudo_list) in result.values {
                            all_structured_values.insert(alias, pseudo_list);
                        }

                        // Note: all_nodegroups already updated in place via &mut reference

                        // Collect new implied nodegroups (lines 671-673)
                        for implied_ng in result.implied_nodegroups.iter() {
                            implied_nodegroups_set.insert(implied_ng.clone());
                        }
                    }
                }
            }
        }
        record_timing("populate: ensure_nodegroup_internal loop", js_sys::Date::now() - t1);
        // If lazy: just skip loading (lines 678-680 in JS - stripTiles happens in JS)

        // Create root pseudo value (root node has no tile, but we need it in cache for toJson)
        // Get root node and create a semantic pseudo value for it
        let root_node = self.with_model_core_mut(|core| {
            core.get_root_node().map_err(|e| JsValue::from_str(&e))
        })?;

        let edges = self.with_model_core(|core| {
            core.get_edges_internal()
                .ok_or_else(|| JsValue::from_str("Model edges not initialized"))
                .map(|map| map.clone())
        })?;

        // Get child node IDs for the root
        let child_node_ids = edges.get(&root_node.nodeid)
            .map(|ids| ids.clone())
            .unwrap_or_default();

        // Create root pseudo value with no tile (using from_node_and_tile for proper initialization)
        let root_pseudo = RustPseudoValue::from_node_and_tile(
            root_node,
            None,  // root has no tile
            None,  // root has no tile_data
            child_node_ids,
        );

        // Create root pseudo list (single value, semantic node)
        let root_list = RustPseudoList {
            node_alias: root_node_alias.clone(),
            values: vec![root_pseudo],
            is_loaded: true,
            is_single: true,
        };

        // Add root to structured values
        all_structured_values.insert(root_node_alias.clone(), root_list);

        // Store all values in Rust's pseudo_cache so TS can query them later
        let t2 = js_sys::Date::now();
        if let Ok(mut cache) = self.core.borrow().pseudo_cache.lock() {
            for (alias, pseudo_list) in all_structured_values.iter() {
                cache.insert(alias.clone(), pseudo_list.clone());
            }
        }
        record_timing("populate: store in pseudo_cache", js_sys::Date::now() - t2);

        // PORT: Phase 4c - Return structured values wrapped in WasmPopulateResult
        // PORT: js/graphManager.ts:724-729 - returning allValues and allNodegroups
        record_timing("populate: total (Rust)", js_sys::Date::now() - populate_start);
        Ok(WasmPopulateResult {
            inner: PopulateResult {
                values: all_structured_values,
                all_values_map: all_values,
                all_nodegroups_map: all_nodegroups,
            }
        })
    }

    /// PORT: graphManager.ts lines 505-643
    /// Simplified implementation - builds RustPseudoList directly without recipe intermediate
    fn values_from_resource_nodegroup_internal(
        &self,
        existing_values: HashMap<String, Option<bool>>,
        nodegroup_tile_ids: Vec<String>,
        nodegroup_id: &str,
    ) -> Result<ValuesFromNodegroupResult, JsValue> {
        let fn_start = js_sys::Date::now();

        // Use cached Arc refs from instance (no cloning needed - just Arc::clone which is cheap)
        let t0 = js_sys::Date::now();
        let core_ref = self.core.borrow();
        let node_objs_wasm = core_ref.cached_nodes.as_ref()
            .ok_or_else(|| JsValue::from_str("Model nodes not cached"))?;
        let edges = core_ref.cached_edges.as_ref()
            .ok_or_else(|| JsValue::from_str("Model edges not cached"))?;
        let reverse_edges = core_ref.cached_reverse_edges.as_ref()
            .ok_or_else(|| JsValue::from_str("Model reverse edges not cached"))?;
        let nodes_by_nodegroup = core_ref.cached_nodes_by_nodegroup.as_ref()
            .ok_or_else(|| JsValue::from_str("Model nodes-by-nodegroup not cached"))?;
        record_timing("vfrn: get cached indices", js_sys::Date::now() - t0);

        let mut values: HashMap<String, RustPseudoList> = HashMap::new();
        let mut implied_nodegroups: HashSet<String> = HashSet::new();

        // PORT: impliedNodes - parent nodes in same nodegroup that need pseudo values
        // Key: nodeid + tileid, Value: (node, tile)
        let mut implied_nodes: HashMap<String, (Arc<StaticNode>, Arc<StaticTile>)> = HashMap::new();
        // Track which (nodeid, tileid) combinations we've already processed
        let mut tile_nodes_seen: HashSet<(String, String)> = HashSet::new();

        // Collect tiles for this nodegroup (core_ref already borrowed above)
        let tiles_store = match core_ref.tiles.as_ref() {
            Some(t) => t,
            None => {
                return Ok(ValuesFromNodegroupResult {
                    values: HashMap::new(),
                    implied_nodegroups: Vec::new(),
                })
            },
        };

        // Build a map of alias -> (core node, tiles)
        let mut alias_tiles: HashMap<String, (Arc<StaticNode>, Vec<Option<Arc<StaticTile>>>)> = HashMap::new();

        // Helper to add a pseudo for a node
        let mut add_to_alias_tiles = |node: &Arc<StaticNode>, tile: Option<Arc<StaticTile>>, tile_id: Option<&String>| {
            let alias = node.alias.clone().unwrap_or_default();
            if alias.is_empty() {
                return;
            }

            // Track that we've seen this (nodeid, tileid) combination
            if let Some(tid) = tile_id {
                tile_nodes_seen.insert((node.nodeid.clone(), tid.clone()));
            }

            // Skip if already exists as truthy
            if let Some(Some(true)) = existing_values.get(&alias) {
                return;
            }

            let entry = alias_tiles.entry(alias.clone()).or_insert_with(|| {
                (Arc::clone(node), Vec::new())
            });
            entry.1.push(tile.clone());
        };

        let t1 = js_sys::Date::now();
        // Get nodes for this nodegroup directly from index (O(1) vs O(all_nodes))
        let nodegroup_nodes = nodes_by_nodegroup.get(nodegroup_id);

        for tile_id in &nodegroup_tile_ids {
            let tile = if tile_id.is_empty() {
                None
            } else {
                tiles_store.get(tile_id).map(|t| Arc::new(t.clone()))
            };
            let tile_nodegroup_id = tile.as_ref().map(|t| t.nodegroup_id.clone());

            // Iterate only nodes in this nodegroup (using pre-built index)
            if let Some(nodes_in_ng) = nodegroup_nodes {
                for node_wasm in nodes_in_ng.iter() {
                    // Add pseudo for this node
                    let t1b = js_sys::Date::now();
                    add_to_alias_tiles(node_wasm, tile.clone(), if tile_id.is_empty() { None } else { Some(tile_id) });
                    record_timing("vfrn: add_to_alias_tiles", js_sys::Date::now() - t1b);

                    // Use reverse_edges for O(1) parent lookup instead of O(edges) scan
                    // PORT: lines 523-540 of old TS
                    if let Some(parent_ids) = reverse_edges.get(&node_wasm.nodeid) {
                        // Only process first parent (like TS "break" behavior)
                        if let Some(parent_id) = parent_ids.first() {
                            if let Some(domain_node) = node_objs_wasm.get(parent_id) {
                                if let Some(ref domain_ng_id) = domain_node.nodegroup_id {
                                    // Check for implied nodegroups (different nodegroup)
                                    if !domain_ng_id.is_empty() && domain_ng_id != nodegroup_id {
                                        implied_nodegroups.insert(domain_ng_id.clone());
                                    }

                                    // PORT: Check for implied nodes (same nodegroup, parent node)
                                    // Condition: domain node is in same nodegroup as tile,
                                    // domain node is NOT the nodegroup root (nodegroup_id != nodeid),
                                    // and we haven't already processed this (nodeid, tileid) combo
                                    if let Some(ref tile_ng_id) = tile_nodegroup_id {
                                        if domain_ng_id == tile_ng_id &&
                                           domain_ng_id != &domain_node.nodeid &&
                                           !tile_id.is_empty() {
                                            let key = format!("{}{}", domain_node.nodeid, tile_id);
                                            if !implied_nodes.contains_key(&key) {
                                                if let Some(t) = tile.as_ref() {
                                                    implied_nodes.insert(key, (Arc::clone(domain_node), Arc::clone(t)));
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
        }
        record_timing("vfrn: tile+node loop (O(tiles*nodes_in_ng))", js_sys::Date::now() - t1);

        // PORT: Process implied nodes (lines 599-609)
        // These are parent semantic nodes that share the same tile
        // Note: We inline the add_to_alias_tiles logic here to avoid borrow checker issues
        let t2 = js_sys::Date::now();
        for (_key, (node, tile)) in implied_nodes.iter() {
            let tile_id = tile.tileid.as_ref();
            // Only add if we haven't seen this (nodeid, tileid) combination yet
            if let Some(tid) = tile_id {
                if !tile_nodes_seen.contains(&(node.nodeid.clone(), tid.clone())) {
                    // Inline add_to_alias_tiles logic
                    let alias = node.alias.clone().unwrap_or_default();
                    if !alias.is_empty() {
                        // Track that we've seen this (nodeid, tileid) combination
                        tile_nodes_seen.insert((node.nodeid.clone(), tid.clone()));

                        // Skip if already exists as truthy
                        if existing_values.get(&alias) != Some(&Some(true)) {
                            let entry = alias_tiles.entry(alias.clone()).or_insert_with(|| {
                                (Arc::clone(node), Vec::new())
                            });
                            entry.1.push(Some(Arc::clone(tile)));
                        }
                    }
                }
            }
        }
        record_timing("vfrn: implied nodes loop", js_sys::Date::now() - t2);

        // Get nodegroups for cardinality check (use cached ref - no cloning)
        let t3 = js_sys::Date::now();
        let nodegroups = core_ref.cached_nodegroups.as_ref();
        record_timing("vfrn: get nodegroups (cached)", js_sys::Date::now() - t3);

        // Convert to RustPseudoList
        let t4 = js_sys::Date::now();
        for (alias, (node, tiles)) in alias_tiles {

            // Determine if this should be single (cardinality-1) based on nodegroup
            let is_single = if node.is_collector {
                // Collector nodes are lists only if nodegroup has cardinality 'n'
                if let Some(ref ng_id) = node.nodegroup_id {
                    if let Some(ngs) = nodegroups {
                        if let Some(ng) = ngs.get(ng_id) {
                            ng.cardinality.as_deref() != Some("n")
                        } else {
                            true // Default to single if nodegroup not found
                        }
                    } else {
                        true // Default to single if no nodegroups
                    }
                } else {
                    true // No nodegroup_id means single
                }
            } else {
                true // Non-collector nodes are always single
            };

            let t4b = js_sys::Date::now();
            let pseudo_list = RustPseudoList::from_node_tiles(node, tiles, &edges, None, is_single);
            record_timing("vfrn: from_node_tiles", js_sys::Date::now() - t4b);
            values.insert(alias, pseudo_list);
        }
        record_timing("vfrn: convert to RustPseudoList", js_sys::Date::now() - t4);

        record_timing("vfrn: total", js_sys::Date::now() - fn_start);
        Ok(ValuesFromNodegroupResult {
            values,
            implied_nodegroups: implied_nodegroups.into_iter().collect(),
        })
    }

    /// WASM-exposed wrapper - returns structured values
    #[wasm_bindgen(js_name = valuesFromResourceNodegroup)]
    pub fn values_from_resource_nodegroup(
        &self,
        existing_values_js: JsValue,
        nodegroup_tile_ids: Vec<String>,
        nodegroup_id: &str,
        _node_objs_js: JsValue,  // No longer needed - we use model directly
        _edges_js: JsValue,      // No longer needed - we use model directly
    ) -> Result<WasmValuesFromNodegroupResult, JsValue> {
        // Deserialize existing_values from JS
        let existing_values: HashMap<String, Option<bool>> = serde_wasm_bindgen::from_value(existing_values_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize existing_values: {:?}", e)))?;

        // Call internal version that returns structured values
        let result = self.values_from_resource_nodegroup_internal(
            existing_values,
            nodegroup_tile_ids,
            nodegroup_id,
        )?;

        Ok(WasmValuesFromNodegroupResult { inner: result })
    }

    /// Find semantic children for a parent semantic node
    /// PORT: js/semantic.ts lines 269-340 (__getChildValues)
    ///
    /// This implements the exact filtering logic from SemanticViewModel.__getChildValues
    /// to determine which tiles contain data for a semantic node's children.
    ///
    /// Parameters:
    /// - parent_tile_id: The tileid of the parent semantic node (or null)
    /// - parent_node_id: The nodeid of the parent semantic node
    /// - parent_nodegroup_id: The nodegroup_id of the parent node (or null)
    /// - child_aliases: List of child node aliases to search for
    ///
    /// Returns: Map<alias, Vec<tileid>> - for each child alias, which tiles contain it
    #[wasm_bindgen(js_name = findSemanticChildren)]
    pub fn find_semantic_children(
        &self,
        parent_tile_id: Option<String>,
        parent_node_id: String,
        _parent_nodegroup_id: Option<String>,
    ) -> Result<JsValue, JsValue> {
        // Get nodes from model
        self.with_model_core_mut(|core| {
            if core.get_nodes_internal().is_none() {
                core.build_nodes().ok();
            }
            Ok(())
        })?;

        let child_nodes = self.with_model_core_mut(|core| {
            core.get_child_nodes(&parent_node_id.as_str())
                .map_err(|e| JsValue::from_str(&e))
                .map(|map| map.clone())
        })?;

        // Result map: alias -> Vec<tileid>
        let mut results: HashMap<String, Vec<String>> = HashMap::new();

        // Store the borrow so it lives long enough for the loop
        let core_ref = self.core.borrow();

        // Iterate through all tiles
        let tiles = core_ref.tiles
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Tiles not initialized"))?;
        for (tile_id, tile) in tiles.iter() {
            // Check each child alias we're looking for
            for (child_alias, child_node) in &child_nodes {
                // Now check semantic parent-child relationship
                // PORT: js/semantic.ts lines 296-340
                if ResourceInstanceWrapperCore::matches_semantic_child(
                    parent_tile_id.as_ref(),
                    parent_node_id.as_ref(),
                    &**child_node,
                    tile,
                ) {
                    results
                        .entry(child_alias.clone())
                        .or_insert_with(Vec::new)
                        .push(tile_id.clone());
                }
            }
        }

        serde_wasm_bindgen::to_value(&results)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {:?}", e)))
    }

    /// Get a single semantic child value
    /// PORT: js/semantic.ts lines 177-204 (__getChildValue)
    ///
    /// More efficient than findSemanticChildren when you only need one child.
    ///
    /// Parameters:
    /// - parent_tile_id: The tileid of the parent semantic node (or null)
    /// - parent_node_id: The nodeid of the parent semantic node
    /// - parent_nodegroup_id: The nodegroup_id of the parent node (or null)
    /// - child_alias: The alias of the specific child to retrieve
    ///
    /// Returns: WasmPseudoValue or WasmPseudoList, or null if not found
    /// Throws an error if tiles are not loaded for the child's nodegroup (in lazy mode)
    #[wasm_bindgen(js_name = getSemanticChildValue)]
    pub fn get_semantic_child_value_wasm(
        &self,
        parent_tile_id: Option<String>,
        parent_node_id: String,
        parent_nodegroup_id: Option<String>,
        child_alias: String,
    ) -> Result<JsValue, JsValue> {
        // Determine loaded nodegroups for lazy mode check
        // Extract loaded nodegroup IDs from loaded_nodegroups state
        let is_lazy = *self.lazy.borrow();
        let loaded_nodegroups_snapshot: Option<HashSet<String>> = if is_lazy {
            if let Ok(loaded) = self.core.borrow().loaded_nodegroups.lock() {
                Some(loaded.iter()
                    .filter(|(_, state)| **state == LoadState::Loaded)
                    .map(|(id, _)| id.clone())
                    .collect())
            } else {
                Some(HashSet::new())
            }
        } else {
            None
        };

        // Call core implementation
        let result = self.core.borrow().get_semantic_child_value(
            parent_tile_id.as_ref(),
            &parent_node_id,
            parent_nodegroup_id.as_ref(),
            &child_alias,
            loaded_nodegroups_snapshot.as_ref(),
        );

        // Convert result to JsValue
        match result {
            Ok(SemanticChildResult::List(pseudo_list)) => {
                let wasm_list = WasmPseudoList::from_rust(pseudo_list);
                Ok(wasm_list.into())
            }
            Ok(SemanticChildResult::Single(pseudo_value)) => {
                let wasm_value = WasmPseudoValue::from_rust(pseudo_value);
                Ok(wasm_value.into())
            }
            Ok(SemanticChildResult::Empty) => {
                Ok(JsValue::NULL)
            }
            Err(SemanticChildError::TilesNotLoaded { nodegroup_id }) => {
                // Return a specific error that JS can catch and handle
                Err(JsValue::from_str(&format!("TILES_NOT_LOADED:{}", nodegroup_id)))
            }
            Err(e) => {
                Err(JsValue::from_str(&e.to_string()))
            }
        }
    }

    /// Async version of getSemanticChildValue that handles lazy tile loading
    ///
    /// If tiles are not loaded for the child's nodegroup, this method will:
    /// 1. Call the tile loader to fetch tiles for that nodegroup
    /// 2. Load the tiles into Rust storage
    /// 3. Mark the nodegroup as loaded
    /// 4. Retry getting the semantic child value
    ///
    /// Returns: WasmPseudoValue or WasmPseudoList, or null if not found
    ///
    /// Phase 4h: Changed from &mut self to &self using RefCell interior mutability.
    /// This prevents deadlocks when multiple async operations run concurrently on
    /// the same wrapper instance. The previous &mut self caused WASM async operations
    /// to block each other since only one mutable borrow can exist at a time.
    #[wasm_bindgen(js_name = retrieveSemanticChildValue)]
    pub async fn retrieve_semantic_child_value(
        &self,
        parent_tile_id: Option<String>,
        parent_node_id: String,
        parent_nodegroup_id: Option<String>,
        child_alias: String,
    ) -> Result<JsValue, JsValue> {
        // First check pseudo_cache - after populate() values are stored there
        if let Ok(cache) = self.core.borrow().pseudo_cache.lock() {
            if let Some(pseudo_list) = cache.get(&child_alias) {
                // TODO: inefficient and cacheable on RustPseudoList
                let nodegroup_id = self.with_model_core_mut(|core| {
                    Ok(core.get_nodes_by_alias_internal()
                        .ok_or_else(|| JsValue::from_str(&format!("Failed to get nodes {:?}", child_alias)))?
                        .get(&child_alias)
                        .ok_or_else(|| JsValue::from_str(&format!("Failed to get children {:?}", child_alias)))?
                        .nodegroup_id
                        .clone())
                })?;

                // Sanity check: is_single should only have 0 or 1 values
                let matching_entries = pseudo_list.matching_entries(parent_tile_id, nodegroup_id);

                // Check if this should be returned as single or list based on is_single flag
                if pseudo_list.is_single {
                    if matching_entries.len() > 1 {
                        return Err(JsValue::from_str(&format!(
                            "is_single=true but pseudo_list for '{}' has {} values (expected 0 or 1)",
                            child_alias, matching_entries.len()
                        )));
                    }
                    // Return single value (first element) if present
                    if let Some(first_value) = matching_entries.first() {
                        let wasm_value = WasmPseudoValue::from_rust((*first_value).clone());
                        return Ok(wasm_value.into());
                    } else {
                        return Ok(JsValue::NULL);
                    }
                } else {
                    // Return as list - convert Vec<&RustPseudoValue> to RustPseudoList
                    let cloned_values: Vec<RustPseudoValue> = matching_entries.iter().map(|v| (*v).clone()).collect();
                    let result_list = RustPseudoList::from_values_with_cardinality(
                        child_alias.clone(),
                        cloned_values,
                        pseudo_list.is_single
                    );
                    let wasm_list = WasmPseudoList::from_rust(result_list);
                    return Ok(wasm_list.into());
                }
            }
        }

        // Determine loaded nodegroups for lazy mode check
        // Extract loaded nodegroup IDs from loaded_nodegroups state
        let is_lazy = *self.lazy.borrow();
        let loaded_nodegroups_snapshot: Option<HashSet<String>> = if is_lazy {
            if let Ok(loaded) = self.core.borrow().loaded_nodegroups.lock() {
                Some(loaded.iter()
                    .filter(|(_, state)| **state == LoadState::Loaded)
                    .map(|(id, _)| id.clone())
                    .collect())
            } else {
                Some(HashSet::new())
            }
        } else {
            None
        };

        // First attempt - call core implementation
        let result = self.core.borrow().get_semantic_child_value(
            parent_tile_id.as_ref(),
            &parent_node_id,
            parent_nodegroup_id.as_ref(),
            &child_alias,
            loaded_nodegroups_snapshot.as_ref(),
        );

        // Check if we need to load tiles
        match result {
            Err(SemanticChildError::TilesNotLoaded { nodegroup_id }) => {
                let block_start = js_sys::Date::now();

                // Need to load tiles for this nodegroup
                let t0 = js_sys::Date::now();
                let loader = self.tile_loader.borrow().clone()
                    .ok_or_else(|| JsValue::from_str("No tile loader set for lazy loading"))?;

                // Call the tile loader with the nodegroup ID (or null for all tiles)
                let nodegroup_js = if is_lazy {
                    JsValue::from_str(&nodegroup_id)
                } else {
                    JsValue::NULL
                };

                let promise = loader.call1(&JsValue::NULL, &nodegroup_js)
                    .map_err(|e| JsValue::from_str(&format!("Failed to call tile loader: {:?}", e)))?;
                record_timing("loader_call", js_sys::Date::now() - t0);

                // Await the promise
                let t1 = js_sys::Date::now();
                let tiles_js = JsFuture::from(js_sys::Promise::from(promise)).await
                    .map_err(|e| JsValue::from_str(&format!("Tile loader failed: {:?}", e)))?;
                record_timing("loader_await", js_sys::Date::now() - t1);

                // Convert JS array of WASM StaticTile wrappers to Vec<CoreStaticTile>
                // Fast path: if tiles are WASM StaticTile objects, extract inner directly
                // Slow path: deserialize from JSON for plain objects
                let t2 = js_sys::Date::now();
                let tiles_array = js_sys::Array::from(&tiles_js);
                let mut core_tiles: Vec<StaticTile> = Vec::new();

                for i in 0..tiles_array.length() {
                    let tile_js = tiles_array.get(i);

                    // Fast path disabled - FromWasmAbi::from_abi takes ownership of the pointer
                    // which causes double-free when the JS object is also dropped.
                    // TODO: Implement proper cloning or use RefFromWasmAbi for borrowing.
                    // if let Some(wasm_tile) = WasmStaticTile::try_from_js(tile_js.clone()) {
                    //     core_tiles.push(wasm_tile.into_inner());
                    //     continue;
                    // }

                    // Slow path: try direct deserialization
                    if let Ok(tile) = serde_wasm_bindgen::from_value::<StaticTile>(tile_js.clone()) {
                        core_tiles.push(tile);
                    } else {
                        // Fallback: try toJSON() if direct deserialization fails
                        let to_json_fn = js_sys::Reflect::get(&tile_js, &JsValue::from_str("toJSON"))
                            .ok()
                            .filter(|v| v.is_function());

                        if let Some(func) = to_json_fn {
                            let func: js_sys::Function = func.dyn_into().unwrap();
                            if let Ok(json_value) = func.call0(&tile_js) {
                                if let Ok(tile) = serde_wasm_bindgen::from_value::<StaticTile>(json_value) {
                                    core_tiles.push(tile);
                                    continue;
                                }
                            }
                        }
                        web_sys::console::log_1(&format!("[retrieve_semantic_children] Warning: could not convert tile {} to CoreStaticTile", i).into());
                    }
                }
                record_timing("tile_conversion", js_sys::Date::now() - t2);

                // Use append_tiles to add these tiles without clearing existing ones
                let t3 = js_sys::Date::now();
                self.append_tiles(core_tiles, true)?;
                record_timing("append_tiles", js_sys::Date::now() - t3);

                // Mark this nodegroup as loaded even if no tiles were returned
                let t4 = js_sys::Date::now();
                if let Ok(mut loaded) = self.core.borrow().loaded_nodegroups.lock() {
                    loaded.insert(nodegroup_id.clone(), LoadState::Loaded);
                }
                record_timing("mark_loaded", js_sys::Date::now() - t4);

                // Retry with updated loaded_nodegroups
                let t5 = js_sys::Date::now();
                let is_lazy = *self.lazy.borrow();
                let loaded_nodegroups_snapshot: Option<HashSet<String>> = if is_lazy {
                    if let Ok(loaded) = self.core.borrow().loaded_nodegroups.lock() {
                        Some(loaded.iter()
                            .filter(|(_, state)| **state == LoadState::Loaded)
                            .map(|(id, _)| id.clone())
                            .collect())
                    } else {
                        Some(HashSet::new())
                    }
                } else {
                    None
                };
                record_timing("snapshot_rebuild", js_sys::Date::now() - t5);

                let t6 = js_sys::Date::now();
                let retry_result = self.core.borrow().get_semantic_child_value(
                    parent_tile_id.as_ref(),
                    &parent_node_id,
                    parent_nodegroup_id.as_ref(),
                    &child_alias,
                    loaded_nodegroups_snapshot.as_ref(),
                );
                record_timing("retry_get_semantic_child", js_sys::Date::now() - t6);

                record_timing("tiles_not_loaded_block_total", js_sys::Date::now() - block_start);

                // Convert retry result to JsValue
                match retry_result {
                    Ok(SemanticChildResult::List(pseudo_list)) => {
                        let wasm_list = WasmPseudoList::from_rust(pseudo_list);
                        Ok(wasm_list.into())
                    }
                    Ok(SemanticChildResult::Single(pseudo_value)) => {
                        let wasm_value = WasmPseudoValue::from_rust(pseudo_value);
                        Ok(wasm_value.into())
                    }
                    Ok(SemanticChildResult::Empty) => {
                        // Return null for empty - consistent with non-retry path
                        Ok(JsValue::NULL)
                    }
                    Err(e) => {
                        Err(JsValue::from_str(&e.to_string()))
                    }
                }
            }
            Ok(SemanticChildResult::List(pseudo_list)) => {
                let wasm_list = WasmPseudoList::from_rust(pseudo_list);
                Ok(wasm_list.into())
            }
            Ok(SemanticChildResult::Single(pseudo_value)) => {
                let wasm_value = WasmPseudoValue::from_rust(pseudo_value);
                Ok(wasm_value.into())
            }
            Ok(SemanticChildResult::Empty) => {
                // Return empty PseudoList instead of null - TS should never need to create pseudos
                Ok(JsValue::NULL)
            }
            Err(e) => {
                Err(JsValue::from_str(&e.to_string()))
            }
        }
    }
}

// ============================================================================
// Helper implementations
// ============================================================================

/// Helper: ExistingValue representation
// Removed: ExistingValue struct - now using Option<bool> directly
// Some(true) = truthy value exists
// Some(false) = false value exists
// None = undefined

impl WASMResourceInstanceWrapper {
    /// Deserialize JS Map<string, StaticNode> to Rust HashMap
    #[allow(dead_code)]
    fn deserialize_node_map(&self, js_map: JsValue) -> Result<HashMap<String, StaticNode>, JsValue> {
        let map = JsMap::from(js_map);
        let mut result = HashMap::new();

        map.for_each(&mut |value, key| {
            if let Some(key_str) = key.as_string() {
                if let Ok(node) = serde_wasm_bindgen::from_value::<StaticNode>(value) {
                    result.insert(key_str, node);
                }
            }
        });

        Ok(result)
    }

    /// Deserialize JS Map<string, string[]> to Rust HashMap
    #[allow(dead_code)]
    fn deserialize_edges_map(&self, js_map: JsValue) -> Result<HashMap<String, Vec<String>>, JsValue> {
        let map = JsMap::from(js_map);
        let mut result = HashMap::new();

        map.for_each(&mut |value, key| {
            if let Some(key_str) = key.as_string() {
                if let Ok(arr) = value.dyn_into::<Array>() {
                    let vec: Vec<String> = (0..arr.length())
                        .filter_map(|i| arr.get(i).as_string())
                        .collect();
                    result.insert(key_str, vec);
                }
            }
        });

        Ok(result)
    }

    // Removed: deserialize_existing_values - now accepting pre-serialized HashMap<String, Option<bool>> from TS
}

#[wasm_bindgen(js_name = newWASMResourceInstanceWrapperForModel)]
pub fn new_wasm_resource_instance_wrapper_for_model(graph_id: &str) -> Result<WASMResourceInstanceWrapper, JsValue> {
    // Verify the model exists in the registry
    crate::model_wrapper::MODEL_REGISTRY.with(|registry| {
        if !registry.borrow().contains_key(graph_id) {
            return Err(JsValue::from_str(&format!("Model not found in registry for graph_id: {}", graph_id)));
        }
        Ok(())
    })?;

    // Create instance wrapper with the graph_id
    Ok(WASMResourceInstanceWrapper::new_from_graph_id(graph_id.to_string()))
}

#[wasm_bindgen(js_name = newWASMResourceInstanceWrapperForResource)]
pub fn new_wasm_resource_instance_wrapper_for_resource(resource: &StaticResource) -> Result<WASMResourceInstanceWrapper, JsValue> {
    let model_id = resource.resourceinstance.graph_id.clone();
    // Verify the model exists in the registry
    crate::model_wrapper::MODEL_REGISTRY.with(|registry| {
        if !registry.borrow().contains_key(&model_id) {
            return Err(JsValue::from_str(&format!("Model not found in registry for graph_id: {}", model_id)));
        }
        Ok(())
    })?;

    // Create instance wrapper with the graph_id
    Ok(WASMResourceInstanceWrapper::new_from_resource(resource))
}

// Tests for this module are in tests/ directory
// See tests/semantic_children_test.rs for semantic relationship testing
