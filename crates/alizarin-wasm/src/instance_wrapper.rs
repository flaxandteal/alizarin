use crate::graph::{StaticResource, StaticResourceDescriptors, StaticResourceRegistry};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

// Use the new unified tracing infrastructure
use crate::tracing::{now_ms, record_timing};
// WASM wrapper type aliases for return types
use crate::graph::StaticTile as WasmStaticTile;
// Core types for internal storage (avoid WASM wrapper overhead)
use crate::node_config_wasm::WasmNodeConfigManager;
use crate::pseudo_value::{
    PseudoList, PseudoListInner, PseudoValue, PseudoValueInner, VisitorContext,
};
use crate::rdm_cache_wasm::WasmRdmCache;
use alizarin_core::is_node_single_cardinality;
use alizarin_core::matches_semantic_child as matches_semantic_child_core;
use alizarin_core::node_config::NodeConfigManager;
use alizarin_core::rdm_cache::RdmCache;
use alizarin_core::StaticResourceMetadata as CoreStaticResourceMetadata;
use alizarin_core::StaticTile as CoreStaticTile;
use alizarin_core::StaticTile;
use alizarin_core::TileSource;
use js_sys::Map as JsMap;
// Re-export from core to avoid duplication
pub use alizarin_core::SemanticChildError;

/// Result of get_semantic_child_value - either values or an indication that none exist
#[derive(Debug)]
pub(crate) enum SemanticChildResult {
    /// A list of pseudo values (for collectors or multiple matches)
    List(PseudoListInner),
    /// A single pseudo value
    Single(PseudoValueInner),
    /// No matching values found (not an error, just empty)
    Empty,
}

/// Result from values_from_resource_nodegroup
/// Contains structured PseudoListInner values directly (no recipe intermediate)
#[derive(Clone)]
pub(crate) struct ValuesFromNodegroupResult {
    /// Map of node alias → PseudoListInner (structured hierarchy)
    pub values: HashMap<String, PseudoListInner>,
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
            let wasm_list = PseudoList::from_rust(rust_list.clone());
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
/// PORT: Phase 4c - Now returns structured PseudoListInner values directly
pub(crate) struct EnsureNodegroupResult {
    /// Structured values by alias
    /// PORT: Map of alias → PseudoListInner (js/graphManager.ts:350 - newValues Map)
    pub values: HashMap<String, PseudoListInner>,
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
    pub fn get_value(&self, alias: Option<String>) -> Option<PseudoList> {
        let alias = alias?;
        self.inner
            .values
            .get(&alias)
            .map(|v| PseudoList::from_rust(v.clone()))
    }

    /// Get all values as a Map in a single boundary crossing
    /// Replaces getValueAliases/getValue loop pattern (N+1 crossings -> 1 crossing)
    #[wasm_bindgen(js_name = getAllValues)]
    pub fn get_all_values(&self) -> JsValue {
        let js_map = js_sys::Map::new();
        for (alias, rust_list) in &self.inner.values {
            let wasm_list = PseudoList::from_rust(rust_list.clone());
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
pub(crate) struct PopulateResult {
    /// Map of alias → PseudoListInner
    pub values: HashMap<String, PseudoListInner>,
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
    pub fn get_value(&self, alias: Option<String>) -> Option<PseudoList> {
        let alias = alias?;
        self.inner
            .values
            .get(&alias)
            .map(|v| PseudoList::from_rust(v.clone()))
    }

    /// Get all values as a Map in a single boundary crossing
    /// Replaces getValueAliases/getValue loop pattern (N+1 crossings -> 1 crossing)
    #[wasm_bindgen(js_name = getAllValues)]
    pub fn get_all_values(&self) -> JsValue {
        let js_map = js_sys::Map::new();
        for (alias, rust_list) in &self.inner.values {
            let wasm_list = PseudoList::from_rust(rust_list.clone());
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
///
/// TODO(priority): This duplicates alizarin-core::ResourceInstanceWrapperCore.
/// The only reason they diverged is Rc<RefCell> (WASM, single-threaded) vs
/// Arc<Mutex> (core, thread-safe for NAPI/Python). Unify them ASAP — either
/// via a generic concurrency parameter or feature-gated type aliases — so
/// methods like set_tile_data_for_node don't need to be added in two places.
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
    // WASM is single-threaded, so we use Rc<RefCell<...>> instead of Arc<Mutex<...>>
    pub(crate) loaded_nodegroups: Rc<RefCell<HashMap<String, LoadState>>>,

    // Cache of PseudoValues (alias -> PseudoListInner)
    // This allows Rust to own the authoritative data and bindings to create lightweight wrappers
    pub(crate) pseudo_cache: Rc<RefCell<HashMap<String, PseudoListInner>>>,
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
    /// Optional compiled-in tile source (checked before tile_loader JS callback).
    /// Set via Rust-only `set_tile_source()` when both alizarin and a tile
    /// backend (e.g. Rós Madair) are compiled into the same binary.
    tile_source: RefCell<Option<Arc<dyn TileSource>>>,
}

// Re-export from core to avoid duplication
pub(crate) use alizarin_core::LoadState;

impl ResourceInstanceWrapperCore {
    /// Create a new core from graph ID
    pub fn new_from_graph_id(graph_id: String) -> Self {
        // Ensure nodes are built in the model (needed for ModelAccess trait)
        crate::model_wrapper::MODEL_REGISTRY.with(|registry| {
            if let Some(core_arc) = registry.borrow().get(&graph_id) {
                let mut core = core_arc.borrow_mut();
                core.ensure_built().ok();
            }
        });

        ResourceInstanceWrapperCore {
            graph_id,
            resource_instance: None,
            tiles: None,
            nodegroup_index: HashMap::new(),
            loaded_nodegroups: Rc::new(RefCell::new(HashMap::new())),
            pseudo_cache: Rc::new(RefCell::new(HashMap::new())),
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
            let core_arc = registry_borrow.get(&self.graph_id).ok_or_else(|| {
                E::from(format!("Model not found in registry: {}", self.graph_id))
            })?;
            let core_borrow = core_arc.borrow();
            f(&core_borrow)
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
            let core_arc = registry_borrow.get(&self.graph_id).ok_or_else(|| {
                E::from(format!("Model not found in registry: {}", self.graph_id))
            })?;
            let mut core_borrow = core_arc.borrow_mut();
            f(&mut core_borrow)
        })
    }

    /// Set a single node's data in a tile, mutating in place.
    /// Returns true if the tile was found and updated, false otherwise.
    pub(crate) fn set_tile_data_for_node(
        &mut self,
        tile_id: &str,
        node_id: &str,
        value: serde_json::Value,
    ) -> bool {
        if let Some(tiles) = &mut self.tiles {
            if let Some(tile) = tiles.get_mut(tile_id) {
                tile.data.insert(node_id.to_string(), value);
                return true;
            }
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
    pub(crate) fn get_semantic_child_value(
        &self,
        parent_tile_id: Option<&String>,
        parent_node_id: &str,
        parent_nodegroup_id: Option<&String>,
        child_alias: &str,
        loaded_nodegroups: Option<&HashSet<String>>,
    ) -> Result<SemanticChildResult, SemanticChildError> {
        // Get child nodes for this parent (needs mutable access for lazy init)
        #[allow(clippy::map_clone)]
        let child_nodes = self.with_model_core_mut(|core| {
            core.get_child_nodes(parent_node_id)
                .map_err(SemanticChildError::ModelNotInitialized)
                .map(|map| map.clone())
        })?;

        // Get edges for creating PseudoValues
        // let edges = self.with_model_core(|core| {
        //     core.get_edges_internal()
        //         .ok_or_else(|| SemanticChildError::ModelNotInitialized("Model edges not initialized".to_string()))
        //         .map(|map| map.clone())
        // })?;

        // Get the specific child node we're looking for (core type)
        let child_node = child_nodes
            .get(child_alias)
            .ok_or_else(|| SemanticChildError::ChildNotFound {
                alias: child_alias.to_string(),
            })?
            .clone();

        // Check if tiles are loaded for this child's nodegroup (lazy mode check)
        if let Some(loaded_ngs) = loaded_nodegroups {
            if let Some(ref child_nodegroup_id) = child_node.nodegroup_id {
                if !loaded_ngs.contains(child_nodegroup_id) {
                    return Err(SemanticChildError::TilesNotLoaded {
                        nodegroup_id: child_nodegroup_id.clone(),
                    });
                }
            }
        }

        // Get tiles (must be initialized)
        let tiles = self
            .tiles
            .as_ref()
            .ok_or(SemanticChildError::TilesNotInitialized)?;

        // Find all tiles that contain this child node with the correct semantic relationship
        // Use nodegroup_index for O(1) lookup instead of O(n) tile scan
        let mut matching_tile_ids: Vec<String> = Vec::new();

        // Get candidate tile IDs from the nodegroup index
        let candidate_tile_ids: Vec<String> =
            if let Some(ref child_nodegroup_id) = child_node.nodegroup_id {
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
                if matches_semantic_child_core(
                    parent_tile_id,
                    parent_nodegroup_id,
                    &child_node,
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

        // Sort tile IDs for deterministic ordering (important when is_single=true
        // and there are multiple tiles for a cardinality-1 nodegroup)
        matching_tile_ids.sort();

        #[allow(clippy::map_clone)]
        // Get edges and nodegroups for creating PseudoValues
        let edges = self.with_model_core(|core| {
            core.get_edges_internal()
                .ok_or_else(|| {
                    SemanticChildError::ModelNotInitialized(
                        "Model edges not initialized".to_string(),
                    )
                })
                .map(|map| map.clone())
        })?;

        // Determine cardinality using the shared helper
        let is_single = self.with_model_core::<_, _, SemanticChildError>(|core| {
            Ok(is_node_single_cardinality(
                &child_node,
                core.get_nodegroups_internal(),
            ))
        })?;

        // Create PseudoValues from the matching tiles
        let mut values = Vec::new();
        for tile_id in &matching_tile_ids {
            let tile = tiles
                .get(tile_id)
                .ok_or_else(|| SemanticChildError::Other("Tile not found".to_string()))?;

            let tile_data = tile.data.get(&child_node.nodeid);
            let child_ids = edges.get(&child_node.nodeid).cloned().unwrap_or_default();

            let pseudo_value = PseudoValueInner::from_node_and_tile(
                Arc::clone(&child_node),
                Some(Arc::new(tile.clone())),
                tile_data.cloned(),
                child_ids,
            );

            values.push(pseudo_value);
        }

        // Create PseudoList or single value based on cardinality and number of values
        if !is_single || values.len() > 1 {
            // Return as PseudoList for multi-cardinality nodes or when multiple tiles match
            let pseudo_list = PseudoListInner::from_values_with_cardinality(
                child_alias.to_string(),
                values,
                is_single,
            );
            Ok(SemanticChildResult::List(pseudo_list))
        } else {
            // Return single value if present, otherwise Empty
            match values.into_iter().next() {
                Some(value) => Ok(SemanticChildResult::Single(value)),
                None => Ok(SemanticChildResult::Empty),
            }
        }
    }
}

impl WASMResourceInstanceWrapper {
    /// Set a compiled-in tile source. Checked before the JS tile_loader callback.
    ///
    /// This is a Rust-only method (not exposed to JS). Use it when both alizarin
    /// and a tile backend are compiled into the same WASM/NAPI binary.
    pub fn set_tile_source(&self, source: Arc<dyn TileSource>) {
        *self.tile_source.borrow_mut() = Some(source);
    }

    /// Remove the compiled-in tile source.
    pub fn clear_tile_source(&self) {
        *self.tile_source.borrow_mut() = None;
    }

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
            let nodegroups = core
                .get_nodegroup_objects()
                .map_err(|e| JsValue::from_str(&e))?;
            for tile in tiles {
                nodegroups.get(&tile.nodegroup_id).ok_or_else(|| {
                    JsValue::from_str(&format!(
                        "Tile {:?} has nodegroup not on model: nodegroup {}",
                        tile.tileid, tile.nodegroup_id
                    ))
                })?;
            }
            Ok(())
        })
    }

    pub fn load_tiles(
        &self,
        tiles: Vec<StaticTile>,
        assume_tiles_comprehensive_for_nodegroup: bool,
    ) -> Result<(), JsValue> {
        self.load_tiles_internal(tiles, false, assume_tiles_comprehensive_for_nodegroup)
    }

    /// Append tiles without clearing existing ones
    /// Used by lazy loading callbacks to add tiles incrementally
    pub fn append_tiles(
        &self,
        tiles: Vec<StaticTile>,
        assume_tiles_comprehensive_for_nodegroup: bool,
    ) -> Result<(), JsValue> {
        self.load_tiles_internal(tiles, true, assume_tiles_comprehensive_for_nodegroup)
    }

    fn load_tiles_internal(
        &self,
        tiles: Vec<StaticTile>,
        append: bool,
        assume_tiles_comprehensive_for_nodegroup: bool,
    ) -> Result<(), JsValue> {
        use alizarin_core::PermissionRule;

        self.check_tiles(&tiles)?;

        let lazy = *self.lazy.borrow();

        // Get permission rules from model (cloned for use in filter)
        // This allows conditional filtering based on tile data values
        let permission_rules: Option<HashMap<String, PermissionRule>> = self
            .with_model_core(|model_core| {
                let rules = model_core.model_access.get_permitted_nodegroups_rules();
                if rules.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(rules.clone()))
                }
            })
            .ok()
            .flatten();

        // Get default_allow from model
        let default_allow: bool = self
            .with_model_core(|model_core| Ok(model_core.model_access.get_default_allow()))
            .unwrap_or(true);

        {
            let mut core = self.core.borrow_mut();
            if core.tiles.is_none() {
                core.tiles = Some(HashMap::new());
            }

            // In non-lazy mode and not appending, clear existing data (replacing all tiles)
            // In lazy mode or when appending, add to existing tiles (incremental loading)
            if !lazy && !append {
                // SAFETY: tiles is guaranteed Some by the initialization above
                core.tiles
                    .as_mut()
                    .expect("tiles initialized above")
                    .clear();
                core.nodegroup_index.clear();
            }

            // Store tiles and build index
            for mut tile in tiles {
                // Apply permission filter - skip tiles that don't pass conditional rules
                if let Some(ref rules) = permission_rules {
                    if let Some(rule) = rules.get(&tile.nodegroup_id) {
                        if !rule.permits_tile(&tile) {
                            continue; // Skip this tile - doesn't match permission criteria
                        }
                    } else if !default_allow {
                        continue; // No rule and default is deny
                    }
                }

                // Ensure tile has an ID
                let tile_id = tile.ensure_id();
                let nodegroup_id = tile.nodegroup_id.clone();

                // Add to nodegroup index
                core.nodegroup_index
                    .entry(nodegroup_id.clone())
                    .or_default()
                    .push(tile_id.clone());

                // Mark this nodegroup as loaded in loaded_nodegroups
                // This consolidates tile tracking with nodegroup processing state
                {
                    let mut loaded = core.loaded_nodegroups.borrow_mut();
                    loaded.insert(nodegroup_id, LoadState::Loaded);
                }

                // Store tile
                // SAFETY: tiles is guaranteed Some by the initialization above
                core.tiles
                    .as_mut()
                    .expect("tiles initialized above")
                    .insert(tile_id.clone(), tile);
            }
        }

        // If assume_tiles_comprehensive_for_nodegroup is true, mark ALL model nodegroups as loaded
        // This handles the case where some nodegroups have no tiles for this resource
        if assume_tiles_comprehensive_for_nodegroup {
            self.with_model_core_mut(|model_core| {
                if let Some(nodegroups) = model_core.get_nodegroups_internal() {
                    let core_ref = self.core.borrow();
                    let mut loaded = core_ref.loaded_nodegroups.borrow_mut();
                    for nodegroup_id in nodegroups.keys() {
                        loaded
                            .entry(nodegroup_id.clone())
                            .or_insert(LoadState::Loaded);
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
            tile_source: RefCell::new(None),
            lazy: RefCell::new(false),
        }
    }

    pub(crate) fn new_from_resource(resource: &StaticResource) -> WASMResourceInstanceWrapper {
        WASMResourceInstanceWrapper {
            core: RefCell::new(ResourceInstanceWrapperCore::new_from_resource(resource)),
            tile_loader: RefCell::new(None),
            tile_source: RefCell::new(None),
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
        self.core
            .borrow()
            .tiles
            .as_ref()
            .is_some_and(|t| !t.is_empty())
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
    pub fn request_tiles_for_nodegroup(
        &self,
        _nodegroup_id: Option<String>,
    ) -> Result<js_sys::Function, JsValue> {
        let loader = self
            .tile_loader
            .borrow()
            .clone()
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
        let core_ref = self.core.borrow();
        let loaded = core_ref.loaded_nodegroups.borrow();
        matches!(loaded.get(&nodegroup_id), Some(LoadState::Loaded))
    }

    /// Load tiles from plain JS objects (JSON deserialization)
    /// Use this for initial loading from JSON files
    #[wasm_bindgen(js_name = loadTiles)]
    pub fn load_tiles_js(&self, tiles_js: JsValue) -> Result<(), JsValue> {
        let tiles: Vec<StaticTile> = serde_wasm_bindgen::from_value(tiles_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize tiles: {:?}", e)))?;
        self.load_tiles(tiles, true)
    }

    /// Append tiles from plain JS objects (JSON deserialization)
    /// Use this for incremental loading (e.g., loading tiles for specific nodegroups)
    /// Unlike loadTiles, this adds to existing tiles without clearing them
    #[wasm_bindgen(js_name = appendTiles)]
    pub fn append_tiles_js(&self, tiles_js: JsValue) -> Result<(), JsValue> {
        let tiles: Vec<StaticTile> = serde_wasm_bindgen::from_value(tiles_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize tiles: {:?}", e)))?;
        self.append_tiles(tiles, true)
    }

    /// Load tiles from WASM StaticTile objects directly (no serialization overhead)
    /// Use this when passing tiles between Rust/WASM contexts
    #[wasm_bindgen(js_name = loadTilesWasm)]
    pub fn load_tiles_wasm(
        &self,
        tiles: Vec<WasmStaticTile>,
        assume_tiles_comprehensive_for_nodegroup: Option<bool>,
    ) -> Result<(), JsValue> {
        // Extract inner CoreStaticTile from each WASM wrapper
        let core_tiles: Vec<StaticTile> = tiles.into_iter().map(|wasm_tile| wasm_tile.0).collect();

        self.load_tiles(
            core_tiles,
            assume_tiles_comprehensive_for_nodegroup.unwrap_or(true),
        )
    }

    /// Load tiles directly from a StaticResource without going through JS
    /// This avoids the expensive tiles getter that creates N WASM wrapper objects
    #[wasm_bindgen(js_name = loadTilesFromResource)]
    pub fn load_tiles_from_resource(
        &self,
        resource: &crate::graph::StaticResource,
        assume_tiles_comprehensive_for_nodegroup: Option<bool>,
    ) -> Result<(), JsValue> {
        // Access tiles directly from the Rust struct without creating wrappers
        if let Some(tiles) = &resource.0.tiles {
            let core_tiles: Vec<StaticTile> = tiles.clone();
            self.load_tiles(
                core_tiles,
                assume_tiles_comprehensive_for_nodegroup.unwrap_or(true),
            )
        } else {
            Ok(()) // No tiles to load
        }
    }

    /// Set a single node's data in a tile on the wrapper's own tile store.
    /// Returns true if the tile was found and updated, false otherwise.
    #[wasm_bindgen(js_name = setTileDataForNode)]
    pub fn set_tile_data_for_node(&self, tile_id: &str, node_id: &str, value: JsValue) -> bool {
        match serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
            Ok(json_val) => self
                .core
                .borrow_mut()
                .set_tile_data_for_node(tile_id, node_id, json_val),
            Err(_) => false,
        }
    }

    #[wasm_bindgen(js_name = getResourceId)]
    pub fn get_resource_id(&self) -> Option<String> {
        self.core
            .borrow()
            .resource_instance
            .as_ref()
            .map(|r| r.resourceinstanceid.clone())
    }

    /// Get resource name from metadata
    /// PORT: js/graphManager.ts:115-125 (getName method)
    #[wasm_bindgen(js_name = getName)]
    pub fn get_name(&self) -> String {
        self.core
            .borrow()
            .resource_instance
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

    /// Get resource descriptors, optionally recomputing from tiles.
    /// PORT: js/graphManager.ts:127-186 (getDescriptors method)
    ///
    /// If `recompute` is false, returns cached descriptors.
    /// If `recompute` is true, computes fresh descriptors from tiles using
    /// graph configuration templates, updates the cache, and returns them.
    #[wasm_bindgen(js_name = getDescriptors)]
    pub fn get_descriptors(&self, recompute: bool) -> Result<StaticResourceDescriptors, JsValue> {
        if !recompute {
            // Return cached descriptors
            let cached = self
                .core
                .borrow()
                .resource_instance
                .as_ref()
                .map(|r| StaticResourceDescriptors(r.descriptors.clone()));

            if let Some(descriptors) = cached {
                if !descriptors.is_empty() {
                    return Ok(descriptors);
                }
            }
            // Fall through to compute if cache is empty
        }

        // Compute fresh descriptors
        let descriptors = self.compute_descriptors_internal()?;

        // Update the cache
        if let Some(ref mut r) = self.core.borrow_mut().resource_instance {
            r.descriptors = descriptors.0.clone();
        }

        Ok(descriptors)
    }

    /// Internal helper to compute descriptors without updating cache.
    fn compute_descriptors_internal(&self) -> Result<StaticResourceDescriptors, JsValue> {
        use alizarin_core::IndexedGraph;

        // Get tiles - clone to release borrow
        let tiles_vec: Vec<_> = {
            let core_ref = self.core.borrow();
            let tiles = core_ref
                .tiles
                .as_ref()
                .ok_or_else(|| JsValue::from_str("No tiles loaded"))?;
            tiles.values().cloned().collect()
        };

        // Get the graph from model core
        let graph = self.with_model_core(|core| Ok(core.get_graph().clone()))?;

        // Create IndexedGraph for efficient descriptor building
        let indexed_graph = IndexedGraph::new(graph);

        // Compute descriptors using the Rust implementation
        let ext_registry = crate::extension_registry::build_extension_registry();
        let descriptors = indexed_graph.build_descriptors_with_context(
            &tiles_vec,
            &mut Vec::new(),
            None,
            Some(&ext_registry),
        );

        Ok(StaticResourceDescriptors(descriptors))
    }

    /// Get tile IDs for a specific nodegroup
    /// Returns array of tile ID strings, or empty array if nodegroup_id is null/undefined
    #[wasm_bindgen(js_name = getTileIdsByNodegroup)]
    pub fn get_tile_ids_by_nodegroup(&self, nodegroup_id: Option<String>) -> Vec<String> {
        match nodegroup_id {
            Some(id) => self
                .core
                .borrow()
                .nodegroup_index
                .get(&id)
                .cloned()
                .unwrap_or_default(),
            None => Vec::new(),
        }
    }

    /// Export all tiles as a JSON string.
    /// Returns the wrapper's current tile state (including any mutations from setTileDataForNode).
    /// Returned as a string for fast boundary crossing — call JSON.parse() on the JS side.
    #[wasm_bindgen(js_name = exportTilesJson)]
    pub fn export_tiles_json(&self) -> Result<String, JsValue> {
        let core = self.core.borrow();
        let tiles: Vec<&CoreStaticTile> = core
            .tiles
            .as_ref()
            .map(|t| t.values().collect())
            .unwrap_or_default();
        serde_json::to_string(&tiles).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get all tile IDs that have been loaded (and passed filtering)
    /// Returns array of all tile ID strings
    #[wasm_bindgen(js_name = getAllTileIds)]
    pub fn get_all_tile_ids(&self) -> Vec<String> {
        self.core
            .borrow()
            .tiles
            .as_ref()
            .map(|tiles| tiles.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get full tile data by tile ID
    /// Returns StaticTile WASM object or error if not found
    /// Returns error with clear message if tile_id is null/undefined
    #[wasm_bindgen(js_name = getTile)]
    pub fn get_tile(&self, tile_id: Option<String>) -> Result<WasmStaticTile, JsValue> {
        let tile_id = tile_id.ok_or_else(|| JsValue::from_str("tile_id is null or undefined"))?;
        self.core
            .borrow()
            .tiles
            .as_ref()
            .ok_or_else(|| JsValue::from_str(&format!("No tiles loaded: {}", tile_id)))?
            .get(&tile_id)
            .cloned()
            .map(WasmStaticTile)
            .ok_or_else(|| JsValue::from_str(&format!("Tile not found: {}", tile_id)))
    }

    /// Get specific node data from a tile
    /// Returns the data value for the given node_id within the tile
    /// Returns error with clear message if tile_id or node_id is null/undefined
    #[wasm_bindgen(js_name = getTileData)]
    pub fn get_tile_data(
        &self,
        tile_id: Option<String>,
        node_id: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let tile_id = tile_id.ok_or_else(|| JsValue::from_str("tile_id is null or undefined"))?;
        let node_id = node_id.ok_or_else(|| JsValue::from_str("node_id is null or undefined"))?;
        // Store the borrow so it lives long enough
        let core_ref = self.core.borrow();
        let tile = core_ref
            .tiles
            .as_ref()
            .ok_or_else(|| JsValue::from_str(&format!("No tiles loaded: {}", tile_id)))?
            .get(&tile_id)
            .ok_or_else(|| JsValue::from_str(&format!("Tile not found: {}", tile_id)))?;

        // Get data for specific node from tile.data HashMap
        match tile.data.get(&node_id) {
            Some(value) => serde_wasm_bindgen::to_value(value)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize data: {:?}", e))),
            None => Ok(JsValue::NULL),
        }
    }

    /// Phase 4g: Check if a nodegroup is being loaded or already loaded
    /// Returns false if nodegroup_id is null/undefined
    #[wasm_bindgen(js_name = isNodegroupLoadedOrLoading)]
    pub fn is_nodegroup_loaded_or_loading(&self, nodegroup_id: Option<String>) -> bool {
        let nodegroup_id = match nodegroup_id {
            Some(id) => id,
            None => return false,
        };
        let core_ref = self.core.borrow();
        let loaded = core_ref.loaded_nodegroups.borrow();
        matches!(
            loaded.get(&nodegroup_id),
            Some(LoadState::Loading) | Some(LoadState::Loaded)
        )
    }

    /// Phase 4g: Try to atomically acquire loading lock for a nodegroup
    /// Returns true if caller should proceed with loading, false if already being loaded
    #[wasm_bindgen(js_name = tryAcquireNodegroupLock)]
    pub fn try_acquire_nodegroup_lock(&self, nodegroup_id: String) -> bool {
        let core_ref = self.core.borrow();
        let mut loaded = core_ref.loaded_nodegroups.borrow_mut();
        match loaded.get(&nodegroup_id) {
            Some(LoadState::Loading) | Some(LoadState::Loaded) => false,
            _ => {
                loaded.insert(nodegroup_id, LoadState::Loading);
                true
            }
        }
    }

    /// Phase 4g: Get cached PseudoValue from Rust cache
    /// Returns PseudoList if found, null otherwise
    /// Handles null/undefined alias gracefully by returning None
    #[wasm_bindgen(js_name = getCachedPseudo)]
    pub fn get_cached_pseudo(&self, alias: Option<String>) -> Option<PseudoList> {
        let alias = alias?;
        {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.borrow();
            if let Some(pseudo_list) = cache.get(&alias) {
                // Convert PseudoListInner to PseudoList
                return Some(PseudoList::from_rust(pseudo_list.clone()));
            }
        }
        None
    }

    /// Phase 4g: Store PseudoList in Rust cache
    #[wasm_bindgen(js_name = cachePseudoList)]
    pub fn cache_pseudo_list(&self, alias: String, wasm_list: PseudoList) {
        let rust_list = wasm_list.into_inner();
        {
            let core_ref = self.core.borrow();
            let mut cache = core_ref.pseudo_cache.borrow_mut();
            cache.insert(alias, rust_list);
        }
    }

    #[wasm_bindgen(js_name = pruneResourceTiles)]
    pub fn prune_resource_tiles(&mut self) -> Result<(), JsValue> {
        let tiles = self
            .core
            .borrow()
            .tiles
            .to_owned()
            .ok_or_else(|| JsValue::from_str("Tiles not initialized"))?;

        let pruned_tiles: HashMap<String, StaticTile> = self.with_model_core(|core| {
            Ok(tiles
                .into_iter()
                .filter(|(_tile_id, tile)| core.is_nodegroup_permitted(&tile.nodegroup_id))
                .collect())
        })?;

        // Use borrow_mut() for assignment
        self.core.borrow_mut().tiles = Some(pruned_tiles);
        Ok(())
    }

    /// Phase 4g: Store single PseudoValue as a list in Rust cache
    #[wasm_bindgen(js_name = cachePseudoValue)]
    pub fn cache_pseudo_value(&self, alias: String, wasm_value: PseudoValue) {
        let rust_value = wasm_value.into_inner();
        let node_alias = alias.clone(); // Use the provided alias
        let rust_list = PseudoListInner {
            node_alias,
            values: vec![rust_value],
            is_loaded: true,
            is_single: true, // Single value being cached
        };
        {
            let core_ref = self.core.borrow();
            let mut cache = core_ref.pseudo_cache.borrow_mut();
            cache.insert(alias, rust_list);
        }
    }

    /// Phase 4g: Clear all cached PseudoValues
    #[wasm_bindgen(js_name = clearPseudoCache)]
    pub fn clear_pseudo_cache(&self) {
        {
            let core_ref = self.core.borrow();
            let mut cache = core_ref.pseudo_cache.borrow_mut();
            cache.clear();
        }
    }

    /// Release all memory held by this instance.
    /// Call this when you're done with an instance to free WASM memory.
    /// After calling this, the instance should not be used.
    #[wasm_bindgen(js_name = release)]
    pub fn release(&self) {
        // Clear tiles
        {
            let mut core = self.core.borrow_mut();
            if let Some(ref mut tiles) = core.tiles {
                tiles.clear();
            }
            core.tiles = None;
            core.nodegroup_index.clear();
        }

        // Clear pseudo cache
        {
            let core_ref = self.core.borrow();
            let mut cache = core_ref.pseudo_cache.borrow_mut();
            cache.clear();
        }

        // Clear loaded nodegroups tracking
        {
            let core_ref = self.core.borrow();
            let mut loaded = core_ref.loaded_nodegroups.borrow_mut();
            loaded.clear();
        }

        // Clear tile loader callback
        *self.tile_loader.borrow_mut() = None;
    }

    /// Get the root pseudo value from the cache.
    ///
    /// Returns the first (and should be only) value from the root pseudo list.
    /// The root is stored in cache with its alias from the graph model.
    ///
    /// PORT: js/graphManager.ts:330-364 getRoot()
    #[wasm_bindgen(js_name = getRootPseudo)]
    pub fn get_root_pseudo(&self) -> Option<PseudoValue> {
        // Get root node to find its alias
        let root_node = self
            .with_model_core_mut(|core| Ok(core.get_root_node().ok()))
            .ok()??;

        let root_alias = root_node.alias.clone().unwrap_or_default();

        // Need to hold borrow long enough for the lock
        let core_ref = self.core.borrow();
        let cache = core_ref.pseudo_cache.borrow();

        // Look up root by its actual alias
        let root_list = cache.get(&root_alias)?;

        // Return the first value (root should have exactly one)
        // PORT: js/graphManager.ts:346-350 - if list, get first element
        if root_list.values.len() > 1 {
            web_sys::console::warn_1(&"Multiple root tiles found - returning first".into());
        }

        root_list
            .values
            .first()
            .map(|v| PseudoValue::from_rust(v.clone()))
    }

    /// Convert the entire resource to JSON using Rust-side traversal.
    ///
    /// Check that all tiles have been loaded before serialization.
    /// Returns an error if any nodegroups are not in Loaded state.
    fn check_tiles_loaded(&self, method_name: &str) -> Result<(), JsValue> {
        let core_ref = self.core.borrow();
        let loaded_nodegroups = core_ref.loaded_nodegroups.borrow();

        if loaded_nodegroups.is_empty() {
            return Err(JsValue::from_str(&format!(
                "{}() called but no tiles have been loaded. \
                 Call populate() or ensure tiles are loaded before serialization.",
                method_name
            )));
        }

        // Check for nodegroups that have tiles but aren't loaded
        let unloaded: Vec<String> = loaded_nodegroups
            .iter()
            .filter(|(_, state)| **state != LoadState::Loaded)
            .map(|(id, _)| id.clone())
            .collect();

        if !unloaded.is_empty() {
            return Err(JsValue::from_str(&format!(
                "{}() called but {} nodegroup(s) have not been loaded: {}. \
                 Ensure all tiles are loaded before serialization.",
                method_name,
                unloaded.len(),
                unloaded
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }

        Ok(())
    }

    /// Serialize a single card from the populated pseudo_cache.
    ///
    /// Prerequisites: `populate()` must have been called for the relevant nodegroups.
    /// Use `nodegroupIdsForCard` to discover which nodegroups are needed.
    ///
    /// @param cardId - The card UUID to serialize
    /// @param parentTileId - Parent tile ID for nested cards (null for root cards)
    /// @param parentNodegroupId - Parent nodegroup ID for nested cards (null for root cards)
    /// @param maxDepth - Max recursion depth (null = unlimited, 0 = this card only)
    #[wasm_bindgen(js_name = serializeCard)]
    pub fn serialize_card(
        &self,
        card_id: String,
        parent_tile_id: Option<String>,
        parent_nodegroup_id: Option<String>,
        max_depth: Option<usize>,
    ) -> Result<JsValue, JsValue> {
        let graph = self.with_model_core(|core| Ok(core.get_graph().clone()))?;

        let card_index = graph.card_index().ok_or_else(|| {
            JsValue::from_str("Graph has no card index — cards may not be loaded")
        })?;

        // Convert WASM pseudo_cache to core types
        let core_cache: std::collections::HashMap<String, alizarin_core::PseudoListCore> = {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.borrow();
            cache
                .iter()
                .map(|(k, v)| (k.clone(), v.to_core()))
                .collect()
        };

        let result = alizarin_core::serialize_card(
            &card_id,
            card_index,
            &core_cache,
            parent_tile_id.as_deref(),
            parent_nodegroup_id.as_deref(),
            &graph,
            max_depth,
            None,
        )
        .map_err(|e| JsValue::from_str(&e))?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
    }

    /// Serialize all root cards from the populated pseudo_cache.
    ///
    /// Prerequisites: `populate()` must have been called.
    ///
    /// @param maxDepth - Max recursion depth (null = unlimited, 0 = root cards only)
    #[wasm_bindgen(js_name = serializeRootCards)]
    pub fn serialize_root_cards(&self, max_depth: Option<usize>) -> Result<JsValue, JsValue> {
        let graph = self.with_model_core(|core| Ok(core.get_graph().clone()))?;

        let card_index = graph.card_index().ok_or_else(|| {
            JsValue::from_str("Graph has no card index — cards may not be loaded")
        })?;

        let core_cache: std::collections::HashMap<String, alizarin_core::PseudoListCore> = {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.borrow();
            cache
                .iter()
                .map(|(k, v)| (k.clone(), v.to_core()))
                .collect()
        };

        let result =
            alizarin_core::serialize_root_cards(card_index, &core_cache, &graph, max_depth, None);

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
    }

    /// Serialize a single card in display mode (resolves UUIDs to labels
    /// using the provided RDM cache and node config manager).
    ///
    /// @param cardId - The card UUID to serialize
    /// @param rdmCache - WasmRdmCache for concept label lookups
    /// @param nodeConfigManager - WasmNodeConfigManager for domain value / boolean label lookups
    /// @param parentTileId - Parent tile ID for nested cards (null for root cards)
    /// @param parentNodegroupId - Parent nodegroup ID for nested cards (null for root cards)
    /// @param maxDepth - Max recursion depth (null = unlimited, 0 = this card only)
    /// @param language - Language code for display labels (defaults to "en")
    #[allow(clippy::too_many_arguments)]
    #[wasm_bindgen(js_name = serializeCardDisplay)]
    pub fn serialize_card_display(
        &self,
        card_id: String,
        rdm_cache: &WasmRdmCache,
        node_config_manager: &WasmNodeConfigManager,
        parent_tile_id: Option<String>,
        parent_nodegroup_id: Option<String>,
        max_depth: Option<usize>,
        language: Option<String>,
        resource_registry: &StaticResourceRegistry,
    ) -> Result<JsValue, JsValue> {
        let graph = self.with_model_core(|core| Ok(core.get_graph().clone()))?;

        let card_index = graph.card_index().ok_or_else(|| {
            JsValue::from_str("Graph has no card index — cards may not be loaded")
        })?;

        let core_cache: std::collections::HashMap<String, alizarin_core::PseudoListCore> = {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.borrow();
            cache
                .iter()
                .map(|(k, v)| (k.clone(), v.to_core()))
                .collect()
        };

        let lang = language.unwrap_or_else(|| "en".to_string());
        let opts = alizarin_core::SerializationOptions::display(&lang);
        let rcache = rdm_cache.inner();
        let ncm = node_config_manager.inner();
        let core_registry = resource_registry.inner();
        let ext_registry = crate::extension_registry::build_extension_registry();
        let ser_ctx = alizarin_core::type_serialization::SerializationContext {
            node_config: None,
            external_resolver: Some(
                rcache as &dyn alizarin_core::type_serialization::ExternalResolver,
            ),
            resource_resolver: Some(
                core_registry as &dyn alizarin_core::type_serialization::ResourceDisplayResolver,
            ),
            extension_registry: Some(&ext_registry),
        };

        let result = alizarin_core::serialize_card(
            &card_id,
            card_index,
            &core_cache,
            parent_tile_id.as_deref(),
            parent_nodegroup_id.as_deref(),
            &graph,
            max_depth,
            Some(alizarin_core::card_traversal::CardSerializationParams {
                opts: &opts,
                ser_ctx: &ser_ctx,
                node_config_manager: Some(ncm),
            }),
        )
        .map_err(|e| JsValue::from_str(&e))?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
    }

    /// Serialize all root cards in display mode.
    ///
    /// @param rdmCache - WasmRdmCache for concept label lookups
    /// @param nodeConfigManager - WasmNodeConfigManager for domain value / boolean label lookups
    /// @param maxDepth - Max recursion depth (null = unlimited, 0 = root cards only)
    /// @param language - Language code for display labels (defaults to "en")
    #[wasm_bindgen(js_name = serializeRootCardsDisplay)]
    pub fn serialize_root_cards_display(
        &self,
        rdm_cache: &WasmRdmCache,
        node_config_manager: &WasmNodeConfigManager,
        max_depth: Option<usize>,
        language: Option<String>,
        resource_registry: &StaticResourceRegistry,
    ) -> Result<JsValue, JsValue> {
        let graph = self.with_model_core(|core| Ok(core.get_graph().clone()))?;

        let card_index = graph.card_index().ok_or_else(|| {
            JsValue::from_str("Graph has no card index — cards may not be loaded")
        })?;

        let core_cache: std::collections::HashMap<String, alizarin_core::PseudoListCore> = {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.borrow();
            cache
                .iter()
                .map(|(k, v)| (k.clone(), v.to_core()))
                .collect()
        };

        let lang = language.unwrap_or_else(|| "en".to_string());
        let opts = alizarin_core::SerializationOptions::display(&lang);
        let rcache = rdm_cache.inner();
        let ncm = node_config_manager.inner();
        let core_registry = resource_registry.inner();
        let ext_registry = crate::extension_registry::build_extension_registry();
        let ser_ctx = alizarin_core::type_serialization::SerializationContext {
            node_config: None,
            external_resolver: Some(
                rcache as &dyn alizarin_core::type_serialization::ExternalResolver,
            ),
            resource_resolver: Some(
                core_registry as &dyn alizarin_core::type_serialization::ResourceDisplayResolver,
            ),
            extension_registry: Some(&ext_registry),
        };

        let result = alizarin_core::serialize_root_cards(
            card_index,
            &core_cache,
            &graph,
            max_depth,
            Some(alizarin_core::card_traversal::CardSerializationParams {
                opts: &opts,
                ser_ctx: &ser_ctx,
                node_config_manager: Some(ncm),
            }),
        );

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
    }

    /// Get nodegroup IDs needed to serialize a card (and its descendants).
    ///
    /// Use this to know which nodegroups to load via `populate()` or
    /// `ensureNodegroup()` before calling `serializeCard()`.
    ///
    /// @param cardId - The card UUID
    /// @param maxDepth - Max depth (null = all descendants, 0 = this card only)
    #[wasm_bindgen(js_name = nodegroupIdsForCard)]
    pub fn nodegroup_ids_for_card(
        &self,
        card_id: String,
        max_depth: Option<usize>,
    ) -> Result<Vec<String>, JsValue> {
        let graph = self.with_model_core(|core| Ok(core.get_graph().clone()))?;

        let card_index = graph.card_index().ok_or_else(|| {
            JsValue::from_str("Graph has no card index — cards may not be loaded")
        })?;

        Ok(card_index.nodegroup_ids_for_card(&card_id, max_depth))
    }

    /// This replaces the JS `forJson()` method that makes 98+ WASM boundary
    /// crossings per resource. By doing the traversal entirely in Rust,
    /// we only cross the boundary once.
    ///
    /// Prerequisites: `populate()` must have been called to fill the pseudo_cache.
    /// Fails with an error if tiles have not been loaded.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        let start = now_ms();

        // Get root node from model to find its alias
        let root_node = self
            .with_model_core_mut(|core| core.get_root_node().map_err(|e| JsValue::from_str(&e)))?;

        let root_alias = root_node.alias.clone().unwrap_or_default();

        // Ensure nodes_by_alias and edges are built
        self.with_model_core_mut(|core| {
            if core.get_nodes_by_alias_internal().is_none() {
                core.build_nodes().map_err(|e| JsValue::from_str(&e))?;
            }
            Ok(())
        })?;

        let (nodes_by_alias, edges) = self.with_model_core(|core| {
            let nodes = core
                .get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("nodes_by_alias not built"))?
                .clone();
            let edges = core
                .get_edges_internal()
                .ok_or_else(|| JsValue::from_str("edges not built"))?
                .clone();
            Ok((nodes, edges))
        })?;

        // Check that all tiles have been loaded - fail fast if not
        self.check_tiles_loaded("toJson")?;

        // Get the pseudo_cache (populated by populate())
        let core_ref = self.core.borrow();
        let cache = core_ref.pseudo_cache.borrow();

        // Build visitor context with default tile_data serialization
        let ctx = VisitorContext::new(&cache, &nodes_by_alias, &edges);

        // Look up the root pseudo from cache (created in populate())
        let root_list = cache.get(&root_alias).ok_or_else(|| {
            JsValue::from_str(&format!(
                "Root pseudo not found in cache for alias '{}' - was populate() called?",
                root_alias
            ))
        })?;

        // Use the root list's to_json - the root is a semantic node, so it traverses children
        let json = root_list.to_json(&ctx);

        // Convert serde_json::Value to JSON string, then parse to JS object
        // Note: Using JSON.parse is more reliable than serde_wasm_bindgen::to_value
        // which can return empty objects in some cases
        let json_string = serde_json::to_string(&json)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize JSON: {}", e)))?;

        let result = js_sys::JSON::parse(&json_string)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {:?}", e)))?;

        let end = now_ms();
        record_timing("toJson (Rust)", end - start);

        Ok(result)
    }

    /// Convert the pseudo cache to display JSON with resolved labels.
    ///
    /// Unlike `toJson()` which returns tile_data (UUIDs) for domain values and concepts,
    /// this method resolves UUIDs to display labels:
    /// - domain-value: looks up label from node config
    /// - concept/concept-list: looks up label from RDM cache
    /// - reference (CLM): extracts display string from StaticReference
    /// - boolean: looks up true/false labels from node config
    /// - Other types: returns tile_data as-is
    ///
    /// Prerequisites: `populate()` must have been called to fill the pseudo_cache.
    ///
    /// @param rdmCache - WasmRdmCache for concept label lookups
    /// @param nodeConfigManager - WasmNodeConfigManager for domain value lookups
    /// @param language - Language code for labels (defaults to "en")
    #[wasm_bindgen(js_name = toDisplayJson)]
    pub fn to_display_json(
        &self,
        rdm_cache: &WasmRdmCache,
        node_config_manager: &WasmNodeConfigManager,
        language: Option<String>,
        resource_registry: &StaticResourceRegistry,
    ) -> Result<JsValue, JsValue> {
        self.to_display_json_impl(
            Some(rdm_cache.inner()),
            Some(node_config_manager.inner()),
            Some(resource_registry.inner()),
            language,
        )
    }

    /// Convert the pseudo cache to display JSON without RDM/config lookups.
    ///
    /// Same as toDisplayJson but without the RDM cache and node config manager.
    /// This is useful when you don't have reference data loaded and just want
    /// the basic JSON output (UUIDs won't be resolved to labels).
    #[wasm_bindgen(js_name = toDisplayJsonSimple)]
    pub fn to_display_json_simple(&self, language: Option<String>) -> Result<JsValue, JsValue> {
        self.to_display_json_impl(None, None, None, language)
    }

    /// Internal implementation for toDisplayJson variants.
    ///
    /// Uses the same core `VisitorContext` + `to_json_generic` path as `toJson()`,
    /// but with display-mode serialization options and a populated `SerializationContext`
    /// carrying `node_config_manager` and `external_resolver` for label resolution.
    fn to_display_json_impl(
        &self,
        rdm_cache: Option<&RdmCache>,
        node_config_manager: Option<&NodeConfigManager>,
        resource_registry: Option<&alizarin_core::StaticResourceRegistry>,
        language: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let start = now_ms();

        // Get root node from model to find its alias
        let root_node = self
            .with_model_core_mut(|core| core.get_root_node().map_err(|e| JsValue::from_str(&e)))?;

        let root_alias = root_node.alias.clone().unwrap_or_default();

        // Ensure nodes_by_alias and edges are built
        self.with_model_core_mut(|core| {
            if core.get_nodes_by_alias_internal().is_none() {
                core.build_nodes().map_err(|e| JsValue::from_str(&e))?;
            }
            Ok(())
        })?;

        let (nodes_by_alias, edges) = self.with_model_core(|core| {
            let nodes = core
                .get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("nodes_by_alias not built"))?
                .clone();
            let edges = core
                .get_edges_internal()
                .ok_or_else(|| JsValue::from_str("edges not built"))?
                .clone();
            Ok((nodes, edges))
        })?;

        // Check that all tiles have been loaded - fail fast if not
        self.check_tiles_loaded("toDisplayJson")?;

        // Get the pseudo_cache (populated by populate())
        let core_ref = self.core.borrow();
        let cache = core_ref.pseudo_cache.borrow();

        let lang = language.unwrap_or_else(|| "en".to_string());

        // Build SerializationContext with resolvers for concept and resource lookups
        let ext_registry = crate::extension_registry::build_extension_registry();
        let ser_ctx = alizarin_core::type_serialization::SerializationContext {
            node_config: None, // Set per-node at leaf serialization
            external_resolver: rdm_cache
                .map(|r| r as &dyn alizarin_core::type_serialization::ExternalResolver),
            resource_resolver: resource_registry
                .map(|r| r as &dyn alizarin_core::type_serialization::ResourceDisplayResolver),
            extension_registry: Some(&ext_registry),
        };

        // Build display-mode VisitorContext — same path as toJson() but with display options
        let ctx = VisitorContext {
            pseudo_cache: &cache,
            nodes_by_alias: &nodes_by_alias,
            edges: &edges,
            depth: 0,
            max_depth: 50,
            serialization_options: alizarin_core::SerializationOptions::display(&lang),
            serialization_context: ser_ctx,
            node_config_manager,
        };

        // Look up the root pseudo from cache (created in populate())
        let root_list = cache.get(&root_alias).ok_or_else(|| {
            JsValue::from_str(&format!(
                "Root pseudo not found in cache for alias '{}' - was populate() called?",
                root_alias
            ))
        })?;

        // Use the same to_json path — display resolution happens at leaf level
        // via serialize_leaf_value in to_json_generic
        let json = root_list.to_json(&ctx);

        // Convert serde_json::Value to JSON string, then parse to JS object
        let json_string = serde_json::to_string(&json)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize JSON: {}", e)))?;

        let result = js_sys::JSON::parse(&json_string)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {:?}", e)))?;

        let end = now_ms();
        record_timing("toDisplayJson (Rust)", end - start);

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

        let start = now_ms();

        // Get root node from model
        let root_node = self
            .with_model_core_mut(|core| core.get_root_node().map_err(|e| JsValue::from_str(&e)))?;

        let root_alias = root_node.alias.clone().unwrap_or_default();

        // Check that all tiles have been loaded - fail fast if not
        self.check_tiles_loaded("toTiles")?;

        // Get resource instance ID
        let resource_id = self
            .core
            .borrow()
            .resource_instance
            .as_ref()
            .map(|r| r.resourceinstanceid.clone())
            .unwrap_or_default();

        // Get nodes and edges
        #[allow(clippy::map_clone)]
        let nodes_by_alias = self.with_model_core(|core| {
            core.get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("Model nodes not initialized"))
                .map(|map| map.clone())
        })?;

        #[allow(clippy::map_clone)]
        let edges = self.with_model_core(|core| {
            core.get_edges_internal()
                .ok_or_else(|| JsValue::from_str("Model edges not initialized"))
                .map(|map| map.clone())
        })?;

        // Get the pseudo cache
        let core_ref = self.core.borrow();
        let cache = core_ref.pseudo_cache.borrow();

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
        let root_list = cache.get(&root_alias).ok_or_else(|| {
            JsValue::from_str(&format!(
                "Root pseudo not found in cache for alias '{}' - was populate() called?",
                root_alias
            ))
        })?;

        // Collect tiles from the tree
        let mut tiles: std::collections::HashMap<String, TileBuilder> =
            std::collections::HashMap::new();
        root_list.collect_tiles(&ctx, &mut tiles);

        // Convert to StaticTiles and serialize
        let static_tiles: Vec<alizarin_core::StaticTile> = tiles
            .values()
            .map(|builder| builder.to_static_tile())
            .collect();

        // Convert to JSON string then parse to JS (same pattern as toJson)
        let json_string = serde_json::to_string(&static_tiles)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize tiles: {}", e)))?;

        let result = js_sys::JSON::parse(&json_string)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse tiles JSON: {:?}", e)))?;

        let end = now_ms();
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

        let start = now_ms();

        // Check that all tiles have been loaded - fail fast if not
        self.check_tiles_loaded("toResource")?;

        // Get the original resource metadata
        let resource_metadata = self
            .core
            .borrow()
            .resource_instance
            .clone()
            .ok_or_else(|| JsValue::from_str("No resource metadata available"))?;

        // Get root node from model
        let root_node = self
            .with_model_core_mut(|core| core.get_root_node().map_err(|e| JsValue::from_str(&e)))?;

        let root_alias = root_node.alias.clone().unwrap_or_default();

        // Get nodes and edges
        #[allow(clippy::map_clone)]
        let nodes_by_alias = self.with_model_core(|core| {
            core.get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("Model nodes not initialized"))
                .map(|map| map.clone())
        })?;

        #[allow(clippy::map_clone)]
        let edges = self.with_model_core(|core| {
            core.get_edges_internal()
                .ok_or_else(|| JsValue::from_str("Model edges not initialized"))
                .map(|map| map.clone())
        })?;

        // Get the pseudo cache
        let core_ref = self.core.borrow();
        let cache = core_ref.pseudo_cache.borrow();

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
        let root_list = cache.get(&root_alias).ok_or_else(|| {
            JsValue::from_str(&format!(
                "Root pseudo not found in cache for alias '{}' - was populate() called?",
                root_alias
            ))
        })?;

        // Collect tiles from the tree
        let mut tiles_map: std::collections::HashMap<String, TileBuilder> =
            std::collections::HashMap::new();
        root_list.collect_tiles(&ctx, &mut tiles_map);

        // Convert to StaticTiles
        let tiles: Vec<alizarin_core::StaticTile> = tiles_map
            .values()
            .map(|builder| builder.to_static_tile())
            .collect();

        // Build the complete resource
        let output_resource = alizarin_core::StaticResource {
            resourceinstance: resource_metadata.clone(),
            tiles: Some(tiles),
            metadata: std::collections::HashMap::new(), // No metadata stored in wrapper
            cache: None,                                // Don't include internal cache in output
            scopes: None,                               // Don't include scopes in output
            tiles_loaded: Some(true),
        };

        // Convert to JSON string then parse to JS
        let json_string = serde_json::to_string(&output_resource)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize resource: {}", e)))?;

        let result = js_sys::JSON::parse(&json_string)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse resource JSON: {:?}", e)))?;

        let end = now_ms();
        record_timing("toResource (Rust)", end - start);

        Ok(result)
    }

    /// Create a PseudoValue or PseudoList from node metadata
    /// PORT: js/pseudos.ts:497-594 makePseudoCls()
    ///
    /// This replaces the TS makePseudoCls logic, returning PseudoValue or PseudoList
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

        #[allow(clippy::map_clone)]
        let node_objs = self.with_model_core(|core| {
            core.get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("Model nodes not initialized"))
                .map(|map| map.clone())
        })?;
        let node = node_objs
            .get(alias)
            .ok_or_else(|| JsValue::from_str(&format!("Could not find node by alias: {}", alias)))?
            .clone();

        // PORT: js/pseudos.ts:518-532 - Check if this should be a PseudoList
        let is_collector = node.is_collector;
        // PORT: js/pseudos.ts:421 - handle missing nodegroup_id (root node case)
        // If nodegroup_id is None or empty string, use empty string (root nodegroup)
        let nodegroup_id = node
            .nodegroup_id
            .as_ref()
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
                let empty_list = PseudoListInner::new(alias.to_string());
                let wasm_list = PseudoList::from_rust(empty_list);
                return Ok(wasm_list.into());
            } else {
                return Ok(JsValue::null());
            }
        };

        if should_be_list {
            // PORT: js/pseudos.ts:536-562 - Create PseudoList with values from tiles
            if !is_permitted {
                // Return empty list for unpermitted nodegroup
                let empty_list = PseudoListInner::new(alias.to_string());
                let wasm_list = PseudoList::from_rust(empty_list);
                return Ok(wasm_list.into());
            }

            // Get all tiles for this nodegroup
            let tile_ids = self.get_tile_ids_by_nodegroup(Some(nodegroup_id.to_string()));

            // Get edges from model directly (no deserialization needed)
            #[allow(clippy::map_clone)]
            let edges = self.with_model_core(|core| {
                core.get_edges_internal()
                    .ok_or_else(|| JsValue::from_str("Model edges not initialized"))
                    .map(|map| map.clone())
            })?;

            // Create list of PseudoValueInners from each tile
            let mut values = Vec::new();

            for tid in tile_ids {
                let tile = tiles.get(&tid);
                if let Some(tile) = tile {
                    // Get tile data for this node
                    let tile_data = tile.data.get(&node.nodeid);

                    // Get child node IDs from edges
                    let child_ids = edges.get(&node.nodeid).cloned().unwrap_or_default();

                    // Create PseudoValueInner for this tile
                    let pseudo_value = PseudoValueInner::from_node_and_tile(
                        Arc::clone(&node),
                        Some(Arc::new(tile.clone())),
                        tile_data.cloned(),
                        child_ids,
                    );

                    values.push(pseudo_value);
                }
            }

            let pseudo_list = PseudoListInner::from_values(alias.to_string(), values);
            let wasm_list = PseudoList::from_rust(pseudo_list);
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
            #[allow(clippy::map_clone)]
            let edges = self.with_model_core(|core| {
                core.get_edges_internal()
                    .ok_or_else(|| JsValue::from_str("Model edges not initialized"))
                    .map(|map| map.clone())
            })?;
            let child_ids = edges.get(&node.nodeid).cloned().unwrap_or_default();

            // Create PseudoValueInner
            let pseudo_value = PseudoValueInner::from_node_and_tile(
                Arc::clone(&node),
                tile.map(Arc::new),
                tile_data,
                child_ids,
            );

            // Convert to WASM wrapper
            let wasm_value = PseudoValue::from_rust(pseudo_value);
            Ok(wasm_value.into())
        }
    }

    /// Check if a nodegroup has been loaded
    /// Phase 4g: Updated to use Mutex
    #[wasm_bindgen(js_name = isNodegroupLoaded)]
    pub fn is_nodegroup_loaded(&self, nodegroup_id: Option<String>) -> bool {
        let nodegroup_id = match nodegroup_id {
            Some(id) => id,
            None => return false,
        };
        let core_ref = self.core.borrow();
        let loaded = core_ref.loaded_nodegroups.borrow();
        matches!(loaded.get(&nodegroup_id), Some(LoadState::Loaded))
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
    /// Delegates to core's standalone `ensure_nodegroup`, then wraps
    /// the PseudoListCore results as PseudoListInner for WASM consumption.
    fn ensure_nodegroup_internal(
        &self,
        all_values_map: &HashMap<String, Option<bool>>,
        all_nodegroups: &mut HashMap<String, bool>,
        nodegroup_id: &str,
        add_if_missing: bool,
        nodegroup_permissions: &HashMap<String, alizarin_core::PermissionRule>,
        do_implied_nodegroups: bool,
    ) -> Result<EnsureNodegroupResult, JsValue> {
        let fn_start = now_ms();

        let core_ref = self.core.borrow();
        let tiles_store = match core_ref.tiles.as_ref() {
            Some(t) => t,
            None => {
                return Ok(EnsureNodegroupResult {
                    values: std::collections::HashMap::new(),
                    implied_nodegroups: Vec::new(),
                    all_nodegroups_map: all_nodegroups.clone(),
                });
            }
        };

        let core_result: alizarin_core::EnsureNodegroupResult =
            core_ref.with_model_core(|model| {
                alizarin_core::ensure_nodegroup(
                    all_values_map,
                    all_nodegroups,
                    nodegroup_id,
                    add_if_missing,
                    nodegroup_permissions,
                    do_implied_nodegroups,
                    model,
                    tiles_store,
                )
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
            })?;

        // Wrap PseudoListCore → PseudoListInner
        let values = core_result
            .values
            .into_iter()
            .map(|(alias, list_core)| (alias, PseudoListInner::from_core(list_core)))
            .collect();

        record_timing("eng: total (delegated to core)", now_ms() - fn_start);
        Ok(EnsureNodegroupResult {
            values,
            implied_nodegroups: core_result.implied_nodegroups,
            all_nodegroups_map: core_result.all_nodegroups_map,
        })
    }

    /// Complete ensureNodegroup implementation in Rust
    /// PORT: graphManager.ts lines 302-377 (full ensureNodegroup function)
    /// PORT: Phase 4c - Now returns structured WasmEnsureNodegroupResult instead of recipes
    /// Now gets node_objs, nodegroup_objs, and edges from model core directly
    #[wasm_bindgen(js_name = ensureNodegroup)]
    pub fn ensure_nodegroup(
        &self,
        all_values_js: JsValue, // HashMap<string, Option<bool>> - pre-serialized by TS
        all_nodegroups_js: JsValue, // Map<string, boolean | Promise> - mutable
        nodegroup_id: &str,
        add_if_missing: bool,
        nodegroup_permissions_js: JsValue, // Phase 4h: Map<nodegroupId, boolean>
        do_implied_nodegroups: bool,
    ) -> Result<WasmEnsureNodegroupResult, JsValue> {
        use std::collections::HashMap;

        // Deserialize all_values from pre-serialized HashMap
        let all_values_map: HashMap<String, Option<bool>> =
            serde_wasm_bindgen::from_value(all_values_js).map_err(|e| {
                JsValue::from_str(&format!("Failed to deserialize all_values: {:?}", e))
            })?;

        // Deserialize all_nodegroups from pre-serialized HashMap
        // true = loaded, false = needs reload, missing key = undefined
        let mut all_nodegroups: HashMap<String, bool> =
            serde_wasm_bindgen::from_value(all_nodegroups_js).map_err(|e| {
                JsValue::from_str(&format!("Failed to deserialize all_nodegroups: {:?}", e))
            })?;

        // Phase 4h: Deserialize nodegroup permissions
        // JS sends HashMap<String, bool> — convert to PermissionRule at the boundary.
        // Non-boolean values (if any slip through) are flattened to false with a warning.
        let bool_permissions: HashMap<String, bool> =
            serde_wasm_bindgen::from_value(nodegroup_permissions_js).map_err(|e| {
                JsValue::from_str(&format!(
                    "Failed to deserialize nodegroup_permissions: {:?}",
                    e
                ))
            })?;
        let nodegroup_permissions: HashMap<String, alizarin_core::PermissionRule> =
            bool_permissions
                .into_iter()
                .map(|(k, v)| (k, alizarin_core::PermissionRule::Boolean(v)))
                .collect();

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
        Ok(WasmEnsureNodegroupResult { inner: result })
    }

    /// Complete populate implementation in Rust
    /// PORT: graphManager.ts lines 600-688 (populate function)
    /// PORT: Phase 4c - Now returns WasmPopulateResult with structured values
    /// Orchestrates loading all nodegroups for a resource
    ///
    /// Optimization: Skips nodegroups that are already loaded (tracked in loaded_nodegroups).
    /// Preserves existing pseudo_cache entries for already-loaded nodegroups.
    /// Permissions are read directly from the model (set via setPermittedNodegroups).
    #[wasm_bindgen(js_name = populate)]
    pub fn populate(
        &self,
        lazy: bool,
        nodegroup_ids: Vec<String>,
        root_node_alias: String,
    ) -> Result<WasmPopulateResult, JsValue> {
        let populate_start = now_ms();

        // Get nodegroup permissions from model via ModelAccess trait (returns PermissionRule,
        // not flattened bools, so per-tile conditional filtering works correctly)
        let t0 = now_ms();
        let nodegroup_permissions: HashMap<String, alizarin_core::PermissionRule> = self
            .with_model_core(|core| {
                use alizarin_core::ModelAccess;
                Ok(core.get_permitted_nodegroups())
            })?;
        record_timing("populate: get permissions from model", now_ms() - t0);

        // Check if pseudo_cache has been populated (not just tiles loaded).
        // This is the true indicator that populate() has run before.
        // loaded_nodegroups tracks tile loading, but pseudo_cache tracks populate().
        let cache_len = {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.borrow();
            cache.len()
        };
        let cache_populated = cache_len > 1;

        // Only use optimization if cache was actually populated previously
        let already_loaded: HashSet<String> = if cache_populated {
            let core_ref = self.core.borrow();
            let loaded = core_ref.loaded_nodegroups.borrow();
            loaded
                .iter()
                .filter(|(_, state)| **state == LoadState::Loaded)
                .map(|(id, _)| id.clone())
                .collect()
        } else {
            // First time populate() is called - process all nodegroups
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

        // Initialize all nodegroups - already loaded ones start as true, others as false
        for nodegroup_id in nodegroup_ids.iter() {
            let is_loaded = already_loaded.contains(nodegroup_id);
            all_nodegroups.insert(nodegroup_id.clone(), is_loaded);
        }

        // Set root node alias to false (line 626)
        all_values.insert(root_node_alias.clone(), Some(false));

        // PORT: Phase 4c - Collect structured PseudoListInner values instead of recipes
        // PORT: js/graphManager.ts:668 - newValues is a Map<string, PseudoValue | PseudoList>
        // Start with existing cache entries for already-loaded nodegroups
        let mut all_structured_values: HashMap<String, PseudoListInner> =
            if !already_loaded.is_empty() {
                let core_ref = self.core.borrow();
                let cache = core_ref.pseudo_cache.borrow();
                cache.clone()
            } else {
                HashMap::new()
            };

        // Non-lazy loading: process all nodegroups (only those not already loaded)
        let t1 = now_ms();
        if !lazy {
            // Phase 1: Process all nodegroups with doImpliedNodegroups=false (lines 636-653)
            let mut implied_nodegroups_set = HashSet::new();

            for nodegroup_id in nodegroups_to_process.iter() {
                let result = self.ensure_nodegroup_internal(
                    &all_values,
                    &mut all_nodegroups,
                    nodegroup_id,
                    true, // addIfMissing
                    &nodegroup_permissions,
                    false, // TODO RMV: doImpliedNodegroups = false for phase 1
                )?;

                // Collect structured values
                // PORT: Phase 4c - Merge PseudoListInner values from ensure_nodegroup result
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
                        Some(&false) => true, // false = needs reload
                        None => true,         // undefined = needs load
                        _ => false,           // true = already loaded
                    };

                    if should_process {
                        let result = self.ensure_nodegroup_internal(
                            &all_values,
                            &mut all_nodegroups,
                            nodegroup_id,
                            true, // addIfMissing
                            &nodegroup_permissions,
                            true, // doImpliedNodegroups = true for phase 2
                        )?;

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
        record_timing("populate: ensure_nodegroup_internal loop", now_ms() - t1);
        // If lazy: just skip loading (lines 678-680 in JS - stripTiles happens in JS)

        // Create root pseudo value (root node has no tile, but we need it in cache for toJson)
        // Get root node and create a semantic pseudo value for it
        let root_node = self
            .with_model_core_mut(|core| core.get_root_node().map_err(|e| JsValue::from_str(&e)))?;

        #[allow(clippy::map_clone)]
        let edges = self.with_model_core(|core| {
            core.get_edges_internal()
                .ok_or_else(|| JsValue::from_str("Model edges not initialized"))
                .map(|map| map.clone())
        })?;

        // Get child node IDs for the root
        let child_node_ids = edges.get(&root_node.nodeid).cloned().unwrap_or_default();

        // Create root pseudo value with no tile (using from_node_and_tile for proper initialization)
        let root_pseudo = PseudoValueInner::from_node_and_tile(
            root_node,
            None, // root has no tile
            None, // root has no tile_data
            child_node_ids,
        );

        // Create root pseudo list (single value, semantic node)
        let root_list = PseudoListInner {
            node_alias: root_node_alias.clone(),
            values: vec![root_pseudo],
            is_loaded: true,
            is_single: true,
        };

        // Add root to structured values
        all_structured_values.insert(root_node_alias.clone(), root_list);

        // Store all values in Rust's pseudo_cache so TS can query them later
        let t2 = now_ms();
        {
            let core_ref = self.core.borrow();
            let mut cache = core_ref.pseudo_cache.borrow_mut();
            for (alias, pseudo_list) in all_structured_values.iter() {
                cache.insert(alias.clone(), pseudo_list.clone());
            }
        }
        record_timing("populate: store in pseudo_cache", now_ms() - t2);

        // PORT: Phase 4c - Return structured values wrapped in WasmPopulateResult
        // PORT: js/graphManager.ts:724-729 - returning allValues and allNodegroups
        record_timing("populate: total (Rust)", now_ms() - populate_start);
        Ok(WasmPopulateResult {
            inner: PopulateResult {
                values: all_structured_values,
                all_values_map: all_values,
                all_nodegroups_map: all_nodegroups,
            },
        })
    }

    /// Delegates to core's standalone `values_from_resource_nodegroup`, then wraps
    /// the PseudoListCore results as PseudoListInner for WASM consumption.
    fn values_from_resource_nodegroup_internal(
        &self,
        existing_values: HashMap<String, Option<bool>>,
        nodegroup_tile_ids: Vec<String>,
        nodegroup_id: &str,
    ) -> Result<ValuesFromNodegroupResult, JsValue> {
        let fn_start = now_ms();

        let core_ref = self.core.borrow();
        let tiles_store = match core_ref.tiles.as_ref() {
            Some(t) => t,
            None => {
                return Ok(ValuesFromNodegroupResult {
                    values: HashMap::new(),
                    implied_nodegroups: Vec::new(),
                })
            }
        };

        let core_result = core_ref.with_model_core(|model| {
            alizarin_core::values_from_resource_nodegroup(
                &existing_values,
                &nodegroup_tile_ids,
                nodegroup_id,
                model,
                tiles_store,
            )
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
        })?;

        // Wrap PseudoListCore → PseudoListInner
        let values = core_result
            .values
            .into_iter()
            .map(|(alias, list_core)| (alias, PseudoListInner::from_core(list_core)))
            .collect();

        record_timing("vfrn: total (delegated to core)", now_ms() - fn_start);
        Ok(ValuesFromNodegroupResult {
            values,
            implied_nodegroups: core_result.implied_nodegroups,
        })
    }

    /// WASM-exposed wrapper - returns structured values
    #[wasm_bindgen(js_name = valuesFromResourceNodegroup)]
    pub fn values_from_resource_nodegroup(
        &self,
        existing_values_js: JsValue,
        nodegroup_tile_ids: Vec<String>,
        nodegroup_id: &str,
        _node_objs_js: JsValue, // No longer needed - we use model directly
        _edges_js: JsValue,     // No longer needed - we use model directly
    ) -> Result<WasmValuesFromNodegroupResult, JsValue> {
        // Deserialize existing_values from JS
        let existing_values: HashMap<String, Option<bool>> =
            serde_wasm_bindgen::from_value(existing_values_js).map_err(|e| {
                JsValue::from_str(&format!("Failed to deserialize existing_values: {:?}", e))
            })?;

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
        parent_nodegroup_id: Option<String>,
    ) -> Result<JsValue, JsValue> {
        // Get nodes from model
        self.with_model_core_mut(|core| {
            if core.get_nodes_internal().is_none() {
                core.build_nodes().ok();
            }
            Ok(())
        })?;

        #[allow(clippy::map_clone)]
        let child_nodes = self.with_model_core_mut(|core| {
            core.get_child_nodes(parent_node_id.as_str())
                .map_err(|e| JsValue::from_str(&e))
                .map(|map| map.clone())
        })?;

        // Result map: alias -> Vec<tileid>
        let mut results: HashMap<String, Vec<String>> = HashMap::new();

        // Store the borrow so it lives long enough for the loop
        let core_ref = self.core.borrow();

        // Iterate through all tiles
        let tiles = core_ref
            .tiles
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Tiles not initialized"))?;
        for (tile_id, tile) in tiles.iter() {
            // Check each child alias we're looking for
            for (child_alias, child_node) in &child_nodes {
                // Now check semantic parent-child relationship
                // PORT: js/semantic.ts lines 296-340
                if matches_semantic_child_core(
                    parent_tile_id.as_ref(),
                    parent_nodegroup_id.as_ref(),
                    child_node,
                    tile,
                ) {
                    results
                        .entry(child_alias.clone())
                        .or_default()
                        .push(tile_id.clone());
                }
            }
        }

        // Sort each alias's tile IDs for deterministic ordering
        for tile_ids in results.values_mut() {
            tile_ids.sort();
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
    /// Returns: PseudoValue or PseudoList, or null if not found
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
            let core_ref = self.core.borrow();
            let loaded = core_ref.loaded_nodegroups.borrow();
            Some(
                loaded
                    .iter()
                    .filter(|(_, state)| **state == LoadState::Loaded)
                    .map(|(id, _)| id.clone())
                    .collect(),
            )
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
                let wasm_list = PseudoList::from_rust(pseudo_list);
                Ok(wasm_list.into())
            }
            Ok(SemanticChildResult::Single(pseudo_value)) => {
                let wasm_value = PseudoValue::from_rust(pseudo_value);
                Ok(wasm_value.into())
            }
            Ok(SemanticChildResult::Empty) => Ok(JsValue::NULL),
            Err(SemanticChildError::TilesNotLoaded { nodegroup_id }) => {
                // Return a specific error that JS can catch and handle
                Err(JsValue::from_str(&format!(
                    "TILES_NOT_LOADED:{}",
                    nodegroup_id
                )))
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Resolve a dot-separated path to its target node metadata without needing tiles.
    ///
    /// Returns { nodegroupId, isSingle, targetNodeId } — enough for the JS layer to
    /// lazy-load just that nodegroup's tiles before calling getValuesAtPath.
    #[wasm_bindgen(js_name = resolvePath)]
    pub fn resolve_path(&self, path: &str) -> Result<JsValue, JsValue> {
        // Ensure model caches are built
        self.with_model_core_mut(|model_core| {
            model_core.ensure_built().map_err(|e| JsValue::from_str(&e))
        })?;

        let info = self.with_model_core(|model| {
            use alizarin_core::ModelAccess;
            let root_node = model.get_root_node().map_err(|e| JsValue::from_str(&e))?;
            let nodes = model
                .get_nodes()
                .ok_or_else(|| JsValue::from_str("Nodes not initialized"))?;
            let edges = model
                .get_edges()
                .ok_or_else(|| JsValue::from_str("Edges not initialized"))?;
            let nodegroups = model.get_nodegroups();
            alizarin_core::resolve_path_segments(path, &root_node, nodes, edges, nodegroups)
                .map_err(|e| JsValue::from_str(&e.to_string()))
        })?;

        Ok(serde_wasm_bindgen::to_value(&serde_json::json!({
            "nodegroupId": info.nodegroup_id,
            "isSingle": info.is_single,
            "targetNodeId": info.target_node.nodeid,
        }))
        .unwrap())
    }

    /// Resolve a dot-separated path through the graph model and return a PseudoList.
    ///
    /// Walks the graph edges matching node aliases at each path segment (e.g. "building.name"),
    /// then retrieves matching tiles for the target node's nodegroup. This avoids
    /// full tree materialization — it goes straight from path to nodegroup to tiles.
    ///
    /// Parameters:
    /// - path: Dot-separated path of node aliases (e.g. "building.name" or ".building.name")
    /// - filter_tile_id: Optional parent tile ID to filter results by parent-child relationship
    ///
    /// Returns: PseudoList for the target node, or throws on invalid path
    #[wasm_bindgen(js_name = getValuesAtPath)]
    pub fn get_values_at_path(
        &self,
        path: &str,
        filter_tile_id: Option<String>,
    ) -> Result<PseudoList, JsValue> {
        // Borrow core once — avoid double-borrow of the RefCell
        let core_ref = self.core.borrow();

        // Ensure model caches are built
        core_ref.with_model_core_mut(|model_core| {
            model_core.ensure_built().map_err(|e| JsValue::from_str(&e))
        })?;

        let tiles_store = core_ref
            .tiles
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Tiles not initialized"))?;

        // Delegate path resolution and tile filtering to core's standalone function
        let (info, tiles) = core_ref.with_model_core(|model| {
            alizarin_core::resolve_and_filter_tiles(
                path,
                model,
                tiles_store,
                &core_ref.nodegroup_index,
                filter_tile_id.as_deref(),
            )
            .map_err(|e| JsValue::from_str(&e.to_string()))
        })?;

        drop(core_ref);

        // Build PseudoValueInner from core results — inner/outer handled by PseudoValueCore
        let values: Vec<PseudoValueInner> = tiles
            .into_iter()
            .map(|tile| {
                let tile_data = tile.data.get(&info.target_node.nodeid).cloned();
                let tile_arc = Arc::new(tile);
                PseudoValueInner::from_node_and_tile(
                    Arc::clone(&info.target_node),
                    Some(tile_arc),
                    tile_data,
                    info.child_node_ids.clone(),
                )
            })
            .collect();

        let alias = info.target_node.alias.clone().unwrap_or_default();
        let pseudo_list =
            PseudoListInner::from_values_with_cardinality(alias, values, info.is_single);
        Ok(PseudoList::from_rust(pseudo_list))
    }

    /// Async version of getSemanticChildValue that handles lazy tile loading
    ///
    /// If tiles are not loaded for the child's nodegroup, this method will:
    /// 1. Call the tile loader to fetch tiles for that nodegroup
    /// 2. Load the tiles into Rust storage
    /// 3. Mark the nodegroup as loaded
    /// 4. Retry getting the semantic child value
    ///
    /// Returns: PseudoValue or PseudoList, or null if not found
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
        {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.borrow();
            if let Some(pseudo_list) = cache.get(&child_alias) {
                // TODO: inefficient and cacheable on PseudoListInner
                let nodegroup_id = self.with_model_core_mut(|core| {
                    Ok(core
                        .get_nodes_by_alias_internal()
                        .ok_or_else(|| {
                            JsValue::from_str(&format!("Failed to get nodes {:?}", child_alias))
                        })?
                        .get(&child_alias)
                        .ok_or_else(|| {
                            JsValue::from_str(&format!("Failed to get children {:?}", child_alias))
                        })?
                        .nodegroup_id
                        .clone())
                })?;

                // Sanity check: is_single should only have 0 or 1 values
                let matching_entries =
                    pseudo_list.matching_entries(parent_tile_id, nodegroup_id, parent_nodegroup_id);

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
                        let wasm_value = PseudoValue::from_rust((*first_value).clone());
                        return Ok(wasm_value.into());
                    } else {
                        return Ok(JsValue::NULL);
                    }
                } else {
                    // Return as list - convert Vec<&PseudoValueInner> to PseudoListInner
                    let cloned_values: Vec<PseudoValueInner> =
                        matching_entries.iter().map(|v| (*v).clone()).collect();
                    let result_list = PseudoListInner::from_values_with_cardinality(
                        child_alias.clone(),
                        cloned_values,
                        pseudo_list.is_single,
                    );
                    let wasm_list = PseudoList::from_rust(result_list);
                    return Ok(wasm_list.into());
                }
            }
        }

        // Determine loaded nodegroups for lazy mode check
        // Extract loaded nodegroup IDs from loaded_nodegroups state
        let is_lazy = *self.lazy.borrow();
        let loaded_nodegroups_snapshot: Option<HashSet<String>> = if is_lazy {
            let core_ref = self.core.borrow();
            let loaded = core_ref.loaded_nodegroups.borrow();
            Some(
                loaded
                    .iter()
                    .filter(|(_, state)| **state == LoadState::Loaded)
                    .map(|(id, _)| id.clone())
                    .collect(),
            )
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
                let block_start = now_ms();

                // Fast path: try compiled-in TileSource before JS callback
                let tile_source_result = {
                    let source_ref = self.tile_source.borrow();
                    if let Some(source) = source_ref.as_ref() {
                        let resource_id = self
                            .core
                            .borrow()
                            .resource_instance
                            .as_ref()
                            .map(|r| r.resourceinstanceid.clone());
                        resource_id
                            .as_ref()
                            .map(|rid| source.load_tiles(rid, Some(&nodegroup_id)))
                    } else {
                        None
                    }
                };

                let used_tile_source = if let Some(source_result) = tile_source_result {
                    match source_result {
                        Ok(core_tiles) => {
                            record_timing("tile_source_load", now_ms() - block_start);
                            self.append_tiles(core_tiles, true)?;
                            true
                        }
                        Err(alizarin_core::TileSourceError::ResourceNotFound { .. }) => {
                            // Resource not in this source — fall through to JS callback
                            false
                        }
                        Err(e) => {
                            // Hard error — propagate rather than masking
                            return Err(JsValue::from_str(&format!("TileSource error: {}", e)));
                        }
                    }
                } else {
                    false
                };

                if !used_tile_source {
                    // Fallback: JS tile_loader callback (existing async path)
                    let t0 = now_ms();
                    let loader =
                        self.tile_loader.borrow().clone().ok_or_else(|| {
                            JsValue::from_str("No tile loader set for lazy loading")
                        })?;

                    let nodegroup_js = if is_lazy {
                        JsValue::from_str(&nodegroup_id)
                    } else {
                        JsValue::NULL
                    };

                    let promise = loader.call1(&JsValue::NULL, &nodegroup_js).map_err(|e| {
                        JsValue::from_str(&format!("Failed to call tile loader: {:?}", e))
                    })?;
                    record_timing("loader_call", now_ms() - t0);

                    let t1 = now_ms();
                    let tiles_js = JsFuture::from(js_sys::Promise::from(promise))
                        .await
                        .map_err(|e| JsValue::from_str(&format!("Tile loader failed: {:?}", e)))?;
                    record_timing("loader_await", now_ms() - t1);

                    // Convert JS array to Vec<CoreStaticTile>
                    let t2 = now_ms();
                    let tiles_array = js_sys::Array::from(&tiles_js);
                    let mut core_tiles: Vec<StaticTile> = Vec::new();

                    for i in 0..tiles_array.length() {
                        let tile_js = tiles_array.get(i);

                        // Fast path disabled - FromWasmAbi::from_abi takes ownership of the pointer
                        // which causes double-free when the JS object is also dropped.
                        // TODO: Implement proper cloning or use RefFromWasmAbi for borrowing.

                        if let Ok(tile) =
                            serde_wasm_bindgen::from_value::<StaticTile>(tile_js.clone())
                        {
                            core_tiles.push(tile);
                        } else {
                            let to_json_fn =
                                js_sys::Reflect::get(&tile_js, &JsValue::from_str("toJSON"))
                                    .ok()
                                    .filter(|v| v.is_function());

                            if let Some(func) = to_json_fn {
                                if let Ok(func) = func.dyn_into::<js_sys::Function>() {
                                    if let Ok(json_value) = func.call0(&tile_js) {
                                        if let Ok(tile) =
                                            serde_wasm_bindgen::from_value::<StaticTile>(json_value)
                                        {
                                            core_tiles.push(tile);
                                            continue;
                                        }
                                    }
                                }
                            }
                            web_sys::console::log_1(&format!("[retrieve_semantic_children] Warning: could not convert tile {} to CoreStaticTile", i).into());
                        }
                    }
                    record_timing("tile_conversion", now_ms() - t2);

                    let t3 = now_ms();
                    self.append_tiles(core_tiles, true)?;
                    record_timing("append_tiles", now_ms() - t3);
                }

                // Mark nodegroup as loaded
                let t4 = now_ms();
                {
                    let core_ref = self.core.borrow();
                    let mut loaded = core_ref.loaded_nodegroups.borrow_mut();
                    loaded.insert(nodegroup_id.clone(), LoadState::Loaded);
                }
                record_timing("mark_loaded", now_ms() - t4);

                // Retry with updated loaded_nodegroups
                let t5 = now_ms();
                let is_lazy = *self.lazy.borrow();
                let loaded_nodegroups_snapshot: Option<HashSet<String>> = if is_lazy {
                    let core_ref = self.core.borrow();
                    let loaded = core_ref.loaded_nodegroups.borrow();
                    Some(
                        loaded
                            .iter()
                            .filter(|(_, state)| **state == LoadState::Loaded)
                            .map(|(id, _)| id.clone())
                            .collect(),
                    )
                } else {
                    None
                };
                record_timing("snapshot_rebuild", now_ms() - t5);

                let t6 = now_ms();
                let retry_result = self.core.borrow().get_semantic_child_value(
                    parent_tile_id.as_ref(),
                    &parent_node_id,
                    parent_nodegroup_id.as_ref(),
                    &child_alias,
                    loaded_nodegroups_snapshot.as_ref(),
                );
                record_timing("retry_get_semantic_child", now_ms() - t6);

                record_timing("tiles_not_loaded_block_total", now_ms() - block_start);

                // Convert retry result to JsValue
                match retry_result {
                    Ok(SemanticChildResult::List(pseudo_list)) => {
                        let wasm_list = PseudoList::from_rust(pseudo_list);
                        Ok(wasm_list.into())
                    }
                    Ok(SemanticChildResult::Single(pseudo_value)) => {
                        let wasm_value = PseudoValue::from_rust(pseudo_value);
                        Ok(wasm_value.into())
                    }
                    Ok(SemanticChildResult::Empty) => Ok(JsValue::NULL),
                    Err(e) => Err(JsValue::from_str(&e.to_string())),
                }
            }
            Ok(SemanticChildResult::List(pseudo_list)) => {
                let wasm_list = PseudoList::from_rust(pseudo_list);
                Ok(wasm_list.into())
            }
            Ok(SemanticChildResult::Single(pseudo_value)) => {
                let wasm_value = PseudoValue::from_rust(pseudo_value);
                Ok(wasm_value.into())
            }
            Ok(SemanticChildResult::Empty) => {
                // Return empty PseudoList instead of null - TS should never need to create pseudos
                Ok(JsValue::NULL)
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Check which nodegroups need tiles loaded for all children of a parent node.
    /// Returns an array of nodegroup IDs that need loading, or empty array if all loaded.
    ///
    /// This enables JS to proactively load tiles before calling the sync batch endpoint,
    /// eliminating async boundary crossings in the hot path.
    ///
    /// Parameters:
    /// - parent_node_id: The nodeid of the parent semantic node
    ///
    /// Returns: Array of nodegroup_id strings that need tiles loaded
    #[wasm_bindgen(js_name = getMissingNodegroupsForChildren)]
    pub fn get_missing_nodegroups_for_children(
        &self,
        parent_node_id: String,
    ) -> Result<Vec<String>, JsValue> {
        // In non-lazy mode, all tiles are assumed loaded
        if !*self.lazy.borrow() {
            return Ok(Vec::new());
        }

        // Get loaded nodegroups snapshot
        let loaded_nodegroups: HashSet<String> = {
            let core_ref = self.core.borrow();
            let loaded = core_ref.loaded_nodegroups.borrow();
            loaded
                .iter()
                .filter(|(_, state)| **state == LoadState::Loaded)
                .map(|(id, _)| id.clone())
                .collect()
        };

        // Get child nodes for this parent
        #[allow(clippy::map_clone)]
        let child_nodes = self.with_model_core_mut(|core| {
            core.get_child_nodes(&parent_node_id)
                .map_err(|e| JsValue::from_str(&e))
                .map(|map| map.clone())
        })?;

        // Collect missing nodegroups
        let mut missing: Vec<String> = Vec::new();
        for child_node in child_nodes.values() {
            if let Some(ref nodegroup_id) = child_node.nodegroup_id {
                if !loaded_nodegroups.contains(nodegroup_id) && !missing.contains(nodegroup_id) {
                    missing.push(nodegroup_id.clone());
                }
            }
        }

        Ok(missing)
    }

    /// Sync batch endpoint: get all semantic child values for a parent node at once.
    ///
    /// IMPORTANT: This is a SYNC method. JS must ensure all required tiles are loaded
    /// before calling this (use getMissingNodegroupsForChildren to check first).
    ///
    /// Returns a JS Map<string, PseudoValue | PseudoList | null> keyed by child alias.
    ///
    /// Parameters:
    /// - parent_tile_id: The tileid of the parent semantic node (or null)
    /// - parent_node_id: The nodeid of the parent semantic node
    /// - parent_nodegroup_id: The nodegroup_id of the parent node (or null)
    ///
    /// Returns: JS Map of alias -> value/list/null
    /// Throws an error if tiles are not loaded for any child's nodegroup
    #[wasm_bindgen(js_name = getAllSemanticChildValues)]
    pub fn get_all_semantic_child_values(
        &self,
        parent_tile_id: Option<String>,
        parent_node_id: String,
        parent_nodegroup_id: Option<String>,
    ) -> Result<JsValue, JsValue> {
        // Determine loaded nodegroups for lazy mode check
        let is_lazy = *self.lazy.borrow();
        let loaded_nodegroups_snapshot: Option<HashSet<String>> = if is_lazy {
            let core_ref = self.core.borrow();
            let loaded = core_ref.loaded_nodegroups.borrow();
            Some(
                loaded
                    .iter()
                    .filter(|(_, state)| **state == LoadState::Loaded)
                    .map(|(id, _)| id.clone())
                    .collect(),
            )
        } else {
            None
        };

        // Get all child aliases for this parent
        #[allow(clippy::map_clone)]
        let child_nodes = self.with_model_core_mut(|core| {
            core.get_child_nodes(&parent_node_id)
                .map_err(|e| JsValue::from_str(&e))
                .map(|map| map.clone())
        })?;

        // Create JS Map for results
        let result_map = JsMap::new();

        // Get each child value and add to map
        for child_alias in child_nodes.keys() {
            let result = self.core.borrow().get_semantic_child_value(
                parent_tile_id.as_ref(),
                &parent_node_id,
                parent_nodegroup_id.as_ref(),
                child_alias,
                loaded_nodegroups_snapshot.as_ref(),
            );

            let js_value = match result {
                Ok(SemanticChildResult::List(pseudo_list)) => {
                    let wasm_list = PseudoList::from_rust(pseudo_list);
                    wasm_list.into()
                }
                Ok(SemanticChildResult::Single(pseudo_value)) => {
                    let wasm_value = PseudoValue::from_rust(pseudo_value);
                    wasm_value.into()
                }
                Ok(SemanticChildResult::Empty) => JsValue::NULL,
                Err(SemanticChildError::TilesNotLoaded { nodegroup_id }) => {
                    return Err(JsValue::from_str(&format!(
                        "TILES_NOT_LOADED:{} (for alias {}). Call getMissingNodegroupsForChildren first.",
                        nodegroup_id, child_alias
                    )));
                }
                Err(e) => {
                    return Err(JsValue::from_str(&e.to_string()));
                }
            };

            result_map.set(&JsValue::from_str(child_alias), &js_value);
        }

        Ok(result_map.into())
    }
}

// ============================================================================
// Helper implementations
// ============================================================================

#[wasm_bindgen(js_name = newWASMResourceInstanceWrapperForModel)]
pub fn new_wasm_resource_instance_wrapper_for_model(
    graph_id: &str,
) -> Result<WASMResourceInstanceWrapper, JsValue> {
    // Verify the model exists in the registry
    crate::model_wrapper::MODEL_REGISTRY.with(|registry| {
        if !registry.borrow().contains_key(graph_id) {
            return Err(JsValue::from_str(&format!(
                "Model not found in registry for graph_id: {}",
                graph_id
            )));
        }
        Ok(())
    })?;

    // Create instance wrapper with the graph_id
    Ok(WASMResourceInstanceWrapper::new_from_graph_id(
        graph_id.to_string(),
    ))
}

#[wasm_bindgen(js_name = newWASMResourceInstanceWrapperForResource)]
pub fn new_wasm_resource_instance_wrapper_for_resource(
    resource: &StaticResource,
) -> Result<WASMResourceInstanceWrapper, JsValue> {
    let model_id = resource.resourceinstance.graph_id.clone();
    // Verify the model exists in the registry
    crate::model_wrapper::MODEL_REGISTRY.with(|registry| {
        if !registry.borrow().contains_key(&model_id) {
            return Err(JsValue::from_str(&format!(
                "Model not found in registry for graph_id: {}",
                model_id
            )));
        }
        Ok(())
    })?;

    // Create instance wrapper with the graph_id
    Ok(WASMResourceInstanceWrapper::new_from_resource(resource))
}

// Tests for this module are in tests/ directory
// See tests/semantic_children_test.rs for semantic relationship testing
