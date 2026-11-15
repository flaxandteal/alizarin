use crate::graph::{StaticResource, StaticResourceDescriptors, StaticResourceRegistry};
use std::cell::RefCell;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

// Use the new unified tracing infrastructure
use crate::tracing::{now_ms, record_timing};
// WASM wrapper type aliases for return types
use crate::graph::StaticTile as WasmStaticTile;
// Core types for internal storage (avoid WASM wrapper overhead)
use crate::node_config_wasm::WasmNodeConfigManager;
use crate::pseudo_value::{PseudoList, PseudoListInner, PseudoValue, PseudoValueInner};
use crate::rdm_cache_wasm::WasmRdmCache;
use alizarin_core::node_config::NodeConfigManager;
use alizarin_core::rdm_cache::RdmCache;
use alizarin_core::StaticTile as CoreStaticTile;
use alizarin_core::StaticTile;
use alizarin_core::TileSource;
use js_sys::Map as JsMap;
// Re-export from core to avoid duplication
pub use alizarin_core::SemanticChildError;
use alizarin_core::SemanticChildResult;

/// WASM wrapper for PopulateResult
#[wasm_bindgen]
pub struct WasmPopulateResult {
    inner: alizarin_core::PopulateResult,
}

#[wasm_bindgen]
impl WasmPopulateResult {
    /// PORT: js/graphManager.ts:669 - iterating over result to get aliases
    #[wasm_bindgen(js_name = getValueAliases)]
    pub fn get_value_aliases(&self) -> Vec<String> {
        alizarin_core::sorted_serde::sorted_keys(self.inner.values.keys())
    }

    /// PORT: js/graphManager.ts:670 - accessing value by alias
    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(&self, alias: Option<String>) -> Option<PseudoList> {
        let alias = alias?;
        self.inner
            .values
            .get(&alias)
            .map(|v| PseudoList::from_rust(PseudoListInner::from_core(v.clone())))
    }

    /// Get all values as a Map in a single boundary crossing
    /// Replaces getValueAliases/getValue loop pattern (N+1 crossings -> 1 crossing)
    #[wasm_bindgen(js_name = getAllValues)]
    pub fn get_all_values(&self) -> JsValue {
        let js_map = js_sys::Map::new();
        for (alias, core_list) in &self.inner.values {
            let wasm_list = PseudoList::from_rust(PseudoListInner::from_core(core_list.clone()));
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

// The resource-instance STATE now lives in `alizarin_core::ResourceInstanceWrapperCore`
// (shared with NAPI/Python; its pseudo_cache/loaded_nodegroups use the feature-gated
// `SharedMut`, which is Rc<RefCell> on wasm32). The former WASM copy is gone. What
// stays WASM-specific is the MODEL_REGISTRY binding and the async fetch-and-retry
// around core's (lazy-aware, synchronous) semantic-child lookup.

/// Access the model for `graph_id` from the WASM MODEL_REGISTRY (immutable). A free
/// function (not a method) so a caller already holding a borrow of the resource core
/// can pass its `graph_id` without re-borrowing it — avoiding a RefCell double-borrow.
pub(crate) fn with_registry_model<F, R, E>(graph_id: &str, f: F) -> Result<R, E>
where
    F: FnOnce(&crate::model_wrapper::ResourceModelWrapperCore) -> Result<R, E>,
    E: From<String>,
{
    let core =
        crate::model_wrapper::ResourceModelWrapperCore::for_graph(graph_id).map_err(E::from)?;
    f(&core)
}

/// Build core's resource-instance state for `graph_id` (core keeps the shared
/// model built, so there is nothing to prepare).
fn new_core(graph_id: String) -> alizarin_core::ResourceInstanceWrapperCore {
    alizarin_core::ResourceInstanceWrapperCore::new(graph_id)
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
    core: RefCell<alizarin_core::ResourceInstanceWrapperCore>,
    /// Optional callback for lazy-loading tiles per nodegroup
    /// Signature: (nodegroup_id: Option<string>) => Promise<StaticTile[]>
    /// None = load all tiles, Some(id) = load tiles for specific nodegroup
    tile_loader: RefCell<Option<js_sys::Function>>,
}

// Re-export from core to avoid duplication
pub(crate) use alizarin_core::LoadState;

/// Core's semantic child lookup for `core`, against its graph's shared model.
/// In lazy mode, a child whose nodegroup isn't loaded yet is `TilesNotLoaded`.
fn semantic_child_value(
    core: &alizarin_core::ResourceInstanceWrapperCore,
    parent_tile_id: Option<&String>,
    parent_node_id: &str,
    parent_nodegroup_id: Option<&String>,
    child_alias: &str,
) -> Result<SemanticChildResult, SemanticChildError> {
    with_registry_model(&core.graph_id, |model| {
        core.get_semantic_child_value(
            parent_tile_id,
            parent_node_id,
            parent_nodegroup_id,
            child_alias,
            model,
        )
    })
}

/// A semantic child as JS sees it: a PseudoList, a PseudoValue, or null.
fn semantic_child_to_js(result: SemanticChildResult) -> JsValue {
    match result {
        SemanticChildResult::List(list) => {
            PseudoList::from_rust(PseudoListInner::from_core(list)).into()
        }
        SemanticChildResult::Single(value) => {
            PseudoValue::from_rust(PseudoValueInner::from_core(value, None)).into()
        }
        SemanticChildResult::Empty => JsValue::NULL,
    }
}

impl WASMResourceInstanceWrapper {
    /// Set a compiled-in tile source, tried before the host's loader
    /// (`fetchFromTileSource`). Rust-only, as on NAPI and Python: for a binary
    /// that compiles alizarin and a tile backend together.
    pub fn set_tile_source(&self, source: Arc<dyn TileSource>) {
        self.core.borrow_mut().tile_source = Some(source);
    }

    /// Remove the compiled-in tile source.
    pub fn clear_tile_source(&self) {
        self.core.borrow_mut().tile_source = None;
    }

    /// Return the graph for this wrapper's model (Rust-only, not exposed to JS).
    pub fn graph(&self) -> Option<Arc<alizarin_core::StaticGraph>> {
        let graph_id = self.core.borrow().graph_id.clone();
        with_registry_model::<_, _, String>(&graph_id, |core| Ok(core.model_access.get_graph_arc()))
            .ok()
    }

    /// Check whether the wrapper has any tiles for the given nodegroup.
    pub fn has_nodegroup_tiles(&self, nodegroup_id: &str) -> bool {
        self.core
            .borrow()
            .nodegroup_index
            .contains_key(nodegroup_id)
    }

    /// Helper to access the model core from registry (immutable) - WASM version
    fn with_model_core<F, R>(&self, f: F) -> Result<R, JsValue>
    where
        F: FnOnce(&crate::model_wrapper::ResourceModelWrapperCore) -> Result<R, JsValue>,
    {
        let graph_id = self.core.borrow().graph_id.clone();
        with_registry_model(&graph_id, f)
    }

    fn check_tiles(&self, tiles: &Vec<StaticTile>) -> Result<(), JsValue> {
        self.with_model_core(|core| {
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

    /// Merge the tiles a lazy fetch returned for `nodegroup_id` (`None`: the
    /// whole resource), through core: that nodegroup counts as loaded even
    /// with no tiles, as on NAPI and Python.
    pub fn append_fetched_tiles(
        &self,
        tiles: Vec<StaticTile>,
        nodegroup_id: Option<&str>,
    ) -> Result<(), JsValue> {
        self.check_tiles(&tiles)?;
        let mut core = self.core.borrow_mut();
        let graph_id = core.graph_id.clone();
        with_registry_model::<_, _, JsValue>(&graph_id, |model| {
            core.merge_fetched_tiles(tiles, nodegroup_id, &model.model_access);
            Ok(())
        })
    }

    fn load_tiles_internal(
        &self,
        tiles: Vec<StaticTile>,
        append: bool,
        assume_tiles_comprehensive_for_nodegroup: bool,
    ) -> Result<(), JsValue> {
        self.check_tiles(&tiles)?;

        // Core's, as NAPI and Python load: permissions (failing closed without
        // the model), on-load derives, then, when comprehensive, every
        // nodegroup marked loaded.
        let mut core = self.core.borrow_mut();
        let graph_id = core.graph_id.clone();
        with_registry_model::<_, _, JsValue>(&graph_id, |model| {
            let model = &model.model_access;
            if append {
                core.merge_permitted_tiles(tiles, model);
            } else {
                core.load_permitted_tiles(tiles, model);
            }
            if assume_tiles_comprehensive_for_nodegroup {
                core.mark_loaded_comprehensively(model);
            }
            Ok(())
        })?;

        Ok(())
    }
}

#[wasm_bindgen]
impl WASMResourceInstanceWrapper {
    pub(crate) fn new_from_graph_id(graph_id: String) -> WASMResourceInstanceWrapper {
        WASMResourceInstanceWrapper {
            core: RefCell::new(new_core(graph_id)),
            tile_loader: RefCell::new(None),
        }
    }

    pub(crate) fn new_from_resource(resource: &StaticResource) -> WASMResourceInstanceWrapper {
        WASMResourceInstanceWrapper {
            core: RefCell::new({
                let mut c = new_core(resource.0.resourceinstance.graph_id.clone());
                c.resource_instance = Some(resource.0.resourceinstance.clone());
                c
            }),
            tile_loader: RefCell::new(None),
        }
    }

    /// Set whether this wrapper should use lazy tile loading
    #[wasm_bindgen(js_name = setLazy)]
    pub fn set_lazy(&self, lazy: bool) {
        let mut core = self.core.borrow_mut();
        core.set_lazy(lazy);
        if lazy && core.tiles.is_none() {
            // Initialize empty tiles HashMap for lazy mode
            core.tiles = Some(alizarin_core::TileStore::new());
        }
    }

    #[wasm_bindgen(js_name = tilesLoaded)]
    pub fn tiles_loaded(&self) -> bool {
        // Core's, as on NAPI: loaded even with no tiles, except in lazy mode.
        self.core.borrow().tiles_loaded()
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

    /// Append the tiles a lazy fetch returned for `nodegroupId` (null: the
    /// whole resource), marking just that nodegroup loaded, even if it had no
    /// tiles. Use this, not appendTiles (which marks every nodegroup), after
    /// fetching one nodegroup.
    #[wasm_bindgen(js_name = appendFetchedTiles)]
    pub fn append_fetched_tiles_js(
        &self,
        tiles_js: JsValue,
        nodegroup_id: Option<String>,
    ) -> Result<(), JsValue> {
        let tiles: Vec<StaticTile> = serde_wasm_bindgen::from_value(tiles_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize tiles: {:?}", e)))?;
        self.append_fetched_tiles(tiles, nodegroup_id.as_deref())
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
        self.core.borrow().display_name()
    }

    /// Get resource descriptors, optionally recomputing from tiles.
    /// PORT: js/graphManager.ts:127-186 (getDescriptors method)
    ///
    /// If `recompute` is false, returns cached descriptors.
    /// If `recompute` is true, computes fresh descriptors from tiles using
    /// graph configuration templates, updates the cache, and returns them.
    #[wasm_bindgen(js_name = getDescriptors)]
    pub fn get_descriptors(&self, recompute: bool) -> Result<StaticResourceDescriptors, JsValue> {
        // Core's, as NAPI uses; the graph and registry only when building.
        let graph_id = self.core.borrow().graph_id.clone();
        self.core
            .borrow_mut()
            .descriptors(recompute, || {
                let graph = with_registry_model::<_, _, String>(&graph_id, |model| {
                    Ok(model.model_access.get_graph_arc())
                })?;
                Ok((graph, crate::extension_registry::build_extension_registry()))
            })
            .map(StaticResourceDescriptors)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Export all tiles as a JSON string.
    /// Returns the wrapper's stored tiles as they now stand (edits committed by
    /// `applyEdits` included; on-load derived tiles left out).
    /// Returned as a string for fast boundary crossing — call JSON.parse() on the JS side.
    #[wasm_bindgen(js_name = exportTilesJson)]
    pub fn export_tiles_json(&self) -> Result<String, JsValue> {
        let core = self.core.borrow();
        let tiles: Vec<&CoreStaticTile> = core.stored_tiles();
        serde_json::to_string(&tiles).map_err(|e| JsValue::from_str(&e.to_string()))
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

    /// Phase 4g: Get cached PseudoValue from Rust cache
    /// Returns PseudoList if found, null otherwise
    /// Handles null/undefined alias gracefully by returning None
    #[wasm_bindgen(js_name = getCachedPseudo)]
    pub fn get_cached_pseudo(&self, alias: Option<String>) -> Option<PseudoList> {
        let alias = alias?;
        {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.read().unwrap();
            if let Some(pseudo_list) = cache.get(&alias) {
                return Some(PseudoList::from_rust(PseudoListInner::from_core(
                    pseudo_list.clone(),
                )));
            }
        }
        None
    }

    #[wasm_bindgen(js_name = pruneResourceTiles)]
    pub fn prune_resource_tiles(&mut self) -> Result<(), JsValue> {
        let tiles = self
            .core
            .borrow()
            .tiles
            .to_owned()
            .ok_or_else(|| JsValue::from_str("Tiles not initialized"))?;

        let pruned_tiles: alizarin_core::TileStore = self.with_model_core(|core| {
            Ok(tiles
                .into_iter()
                .filter(|(_tile_id, tile)| core.is_tile_permitted(tile))
                .collect())
        })?;

        // Use borrow_mut() for assignment
        self.core.borrow_mut().tiles = Some(pruned_tiles);
        Ok(())
    }

    /// Release all memory held by this instance.
    /// Call this when you're done with an instance to free WASM memory.
    /// After calling this, the instance should not be used.
    #[wasm_bindgen(js_name = release)]
    pub fn release(&self) {
        // Core's release (tiles, index, pseudo cache, load states), as NAPI uses.
        self.core.borrow_mut().release();

        // Clear tile loader callback (WASM's own)
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
        let core_ref = self.core.borrow();
        let root_alias = with_registry_model::<_, _, String>(&core_ref.graph_id, |model| {
            Ok(model.get_root_node()?.alias.clone().unwrap_or_default())
        })
        .ok()?;

        // Core's cached root list (PseudoListCore): wrap only the value returned.
        let cache = core_ref.pseudo_cache.read().unwrap();
        let root_list = cache.get(&root_alias)?;

        if root_list.values.len() > 1 {
            web_sys::console::warn_1(&"Multiple root tiles found - returning first".into());
        }

        root_list
            .values
            .first()
            .map(|v| PseudoValue::from_rust(PseudoValueInner::from_core(v.clone(), None)))
    }

    /// Convert the entire resource to JSON using Rust-side traversal.
    ///
    /// Check that all tiles have been loaded before serialization.
    /// Returns an error if any nodegroups are not in Loaded state.
    fn check_tiles_loaded(&self, method_name: &str) -> Result<(), JsValue> {
        let core_ref = self.core.borrow();
        let loaded_nodegroups = core_ref.loaded_nodegroups.read().unwrap();

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
        let graph = self.with_model_core(|core| Ok(core.model_access.get_graph_arc()))?;

        let card_index = graph.card_index().ok_or_else(|| {
            JsValue::from_str("Graph has no card index — cards may not be loaded")
        })?;

        // Convert WASM pseudo_cache to core types
        let core_cache: std::collections::HashMap<String, alizarin_core::PseudoListCore> = {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.read().unwrap();
            cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
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
        let graph = self.with_model_core(|core| Ok(core.model_access.get_graph_arc()))?;

        let card_index = graph.card_index().ok_or_else(|| {
            JsValue::from_str("Graph has no card index — cards may not be loaded")
        })?;

        let core_cache: std::collections::HashMap<String, alizarin_core::PseudoListCore> = {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.read().unwrap();
            cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
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
        let graph = self.with_model_core(|core| Ok(core.model_access.get_graph_arc()))?;

        let card_index = graph.card_index().ok_or_else(|| {
            JsValue::from_str("Graph has no card index — cards may not be loaded")
        })?;

        let core_cache: std::collections::HashMap<String, alizarin_core::PseudoListCore> = {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.read().unwrap();
            cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
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
            concept_lookup: None,
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
        let graph = self.with_model_core(|core| Ok(core.model_access.get_graph_arc()))?;

        let card_index = graph.card_index().ok_or_else(|| {
            JsValue::from_str("Graph has no card index — cards may not be loaded")
        })?;

        let core_cache: std::collections::HashMap<String, alizarin_core::PseudoListCore> = {
            let core_ref = self.core.borrow();
            let cache = core_ref.pseudo_cache.read().unwrap();
            cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
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
            concept_lookup: None,
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
        let graph = self.with_model_core(|core| Ok(core.model_access.get_graph_arc()))?;

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

        // Check that all tiles have been loaded - fail fast if not
        self.check_tiles_loaded("toJson")?;

        // Core's serialisation, as on NAPI: core's pseudo_cache (populated by
        // populate()) and the model's own node and edge maps, borrowed, not copied.
        let core_ref = self.core.borrow();
        let json = with_registry_model::<_, _, JsValue>(&core_ref.graph_id, |model| {
            let root_alias = model
                .get_root_node()
                .map_err(|e| JsValue::from_str(&e))?
                .alias
                .clone()
                .unwrap_or_default();
            let nodes_by_alias = model
                .get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("nodes_by_alias not built"))?;
            let edges = model
                .get_edges_internal()
                .ok_or_else(|| JsValue::from_str("edges not built"))?;
            let cache = core_ref.pseudo_cache.read().unwrap();
            let ctx = alizarin_core::pseudo_value_core::VisitorContext::new(
                &cache,
                nodes_by_alias,
                edges,
            );
            let root_list = cache.get(&root_alias).ok_or_else(|| {
                JsValue::from_str(&format!(
                    "Root pseudo not found in cache for alias '{}' - was populate() called?",
                    root_alias
                ))
            })?;
            Ok(root_list.to_json(&ctx))
        })?;

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

        // Check that all tiles have been loaded - fail fast if not
        self.check_tiles_loaded("toDisplayJson")?;

        let lang = language.unwrap_or_else(|| "en".to_string());
        let ext_registry = crate::extension_registry::build_extension_registry();

        // Core's serialisation in display mode: core's pseudo_cache (populated
        // by populate()) and the model's own maps, borrowed, not copied.
        let core_ref = self.core.borrow();
        let json = with_registry_model::<_, _, JsValue>(&core_ref.graph_id, |model| {
            let root_alias = model
                .get_root_node()
                .map_err(|e| JsValue::from_str(&e))?
                .alias
                .clone()
                .unwrap_or_default();
            let nodes_by_alias = model
                .get_nodes_by_alias_internal()
                .ok_or_else(|| JsValue::from_str("nodes_by_alias not built"))?;
            let edges = model
                .get_edges_internal()
                .ok_or_else(|| JsValue::from_str("edges not built"))?;
            let cache = core_ref.pseudo_cache.read().unwrap();

            let ser_ctx = alizarin_core::type_serialization::SerializationContext {
                node_config: None,
                external_resolver: rdm_cache
                    .map(|r| r as &dyn alizarin_core::type_serialization::ExternalResolver),
                concept_lookup: None,
                resource_resolver: resource_registry
                    .map(|r| r as &dyn alizarin_core::type_serialization::ResourceDisplayResolver),
                extension_registry: Some(&ext_registry),
            };
            let ctx = alizarin_core::pseudo_value_core::VisitorContext {
                pseudo_cache: &cache,
                nodes_by_alias,
                edges,
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

            // Display resolution happens at leaf level via serialize_leaf_value
            // in to_json_generic.
            Ok(root_list.to_json(&ctx))
        })?;

        // Convert serde_json::Value to JSON string, then parse to JS object
        let json_string = serde_json::to_string(&json)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize JSON: {}", e)))?;

        let result = js_sys::JSON::parse(&json_string)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {:?}", e)))?;

        let end = now_ms();
        record_timing("toDisplayJson (Rust)", end - start);

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
    /// - is_single: Force single value (not a list)
    ///
    /// Core's, as NAPI uses: null (TS: PseudoUnavailable) when not permitted.
    #[wasm_bindgen(js_name = makePseudoValue)]
    pub fn make_pseudo_value(
        &self,
        alias: &str,
        tile_id: Option<String>,
        is_permitted: bool,
        is_single: bool,
    ) -> Result<JsValue, JsValue> {
        let core_ref = self.core.borrow();
        let result = with_registry_model::<_, _, String>(&core_ref.graph_id, |model| {
            core_ref.make_pseudo_value(alias, tile_id.as_deref(), is_permitted, is_single, model)
        })
        .map_err(|e| JsValue::from_str(&e))?;
        Ok(semantic_child_to_js(result))
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
        let loaded = core_ref.loaded_nodegroups.read().unwrap();
        matches!(loaded.get(&nodegroup_id), Some(LoadState::Loaded))
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
        // Core's populate, the one NAPI and Python use; the model view supplies
        // the permissions and graph indices.
        let core = self.core.borrow();
        let result = with_registry_model(&core.graph_id, |model| {
            core.populate(lazy, &nodegroup_ids, &root_node_alias, model)
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
        })?;
        record_timing("populate: total (Rust)", now_ms() - populate_start);
        Ok(WasmPopulateResult { inner: result })
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
        // In lazy mode, an unfetched nodegroup throws `TILES_NOT_LOADED:<id>`.
        semantic_child_value(
            &self.core.borrow(),
            parent_tile_id.as_ref(),
            &parent_node_id,
            parent_nodegroup_id.as_ref(),
            &child_alias,
        )
        .map(semantic_child_to_js)
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Resolve a dot-separated path to its target node metadata without needing tiles.
    ///
    /// Returns { nodegroupId, isSingle, targetNodeId } — enough for the JS layer to
    /// lazy-load just that nodegroup's tiles before calling getValuesAtPath.
    #[wasm_bindgen(js_name = resolvePath)]
    pub fn resolve_path(&self, path: &str) -> Result<JsValue, JsValue> {
        // Core's resolve_path, as NAPI and Python use.
        let core_ref = self.core.borrow();
        let info = with_registry_model::<_, _, JsValue>(&core_ref.graph_id, |model| {
            core_ref
                .resolve_path(path, model)
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
        // Core's, as NAPI and Python use.
        let core_ref = self.core.borrow();
        let list = with_registry_model(&core_ref.graph_id, |model| {
            core_ref
                .get_values_at_path(path, model, filter_tile_id.as_deref())
                .map_err(|e| JsValue::from_str(&e.to_string()))
        })?;
        Ok(PseudoList::from_rust(PseudoListInner::from_core(list)))
    }

    /// The nodegroups to fetch before `nodegroupId` can be read (in lazy mode;
    /// none otherwise): itself and, for an on-load Derive target, its unloaded
    /// sources. Core's, as on NAPI and Python.
    #[wasm_bindgen(js_name = nodegroupsToFetch)]
    pub fn nodegroups_to_fetch(&self, nodegroup_id: String) -> Result<Vec<String>, JsValue> {
        let core_ref = self.core.borrow();
        with_registry_model::<_, _, JsValue>(&core_ref.graph_id, |model| {
            Ok(core_ref.nodegroups_to_fetch(&nodegroup_id, model))
        })
    }

    /// Throws if node `alias` can't be given a value here (unknown, semantic,
    /// derived, or a lazy nodegroup not loaded): what the ORM checks on
    /// assignment. Core's, as on NAPI and Python.
    #[wasm_bindgen(js_name = checkEditable)]
    pub fn check_editable(&self, alias: String) -> Result<(), JsValue> {
        let core_ref = self.core.borrow();
        with_registry_model::<_, _, JsValue>(&core_ref.graph_id, |model| {
            core_ref
                .check_editable(&alias, &model.model_access)
                .map_err(|e| JsValue::from_str(&e))
        })
    }

    /// Apply and commit a batch of edits as the ORM recorded them
    /// (`[{op: "set" | "add" | "remove", ...}]`), all or nothing, returning
    /// `{upserted, removed}`. Core's, as on NAPI and Python.
    #[wasm_bindgen(js_name = applyEdits)]
    pub fn apply_edits(&self, ops: JsValue) -> Result<JsValue, JsValue> {
        let ops: Vec<alizarin_core::EditOp> = serde_wasm_bindgen::from_value(ops)
            .map_err(|e| JsValue::from_str(&format!("invalid edits: {e}")))?;
        let mut core = self.core.borrow_mut();
        let graph_id = core.graph_id.clone();
        let delta = with_registry_model::<_, _, JsValue>(&graph_id, |model| {
            core.apply_edits(&ops, &model.model_access)
                .map_err(|e| JsValue::from_str(&e))
        })?;
        serde::Serialize::serialize(
            &delta,
            &serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true),
        )
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Fetch `nodegroupIds` from the compiled-in tile source, if there is one,
    /// returning those still for the host's loader. Core's, as on NAPI and
    /// Python.
    #[wasm_bindgen(js_name = fetchFromTileSource)]
    pub fn fetch_from_tile_source(
        &self,
        nodegroup_ids: Vec<String>,
    ) -> Result<Vec<String>, JsValue> {
        let mut core = self.core.borrow_mut();
        let graph_id = core.graph_id.clone();
        with_registry_model::<_, _, JsValue>(&graph_id, |model| {
            core.fetch_from_tile_source(&nodegroup_ids, &model.model_access)
                .map_err(|e| JsValue::from_str(&format!("TileSource error: {e}")))
        })
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
        // Core's, as NAPI uses (none in eager mode; each nodegroup once).
        let core_ref = self.core.borrow();
        with_registry_model::<_, _, JsValue>(&core_ref.graph_id, |model| {
            Ok(
                core_ref.missing_nodegroups_for_children(
                    &parent_node_id,
                    model,
                    core_ref.is_lazy(),
                ),
            )
        })
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
        let core_ref = self.core.borrow();
        let values = with_registry_model::<_, _, SemanticChildError>(&core_ref.graph_id, |model| {
            core_ref.get_all_semantic_child_values(
                parent_tile_id.as_ref(),
                &parent_node_id,
                parent_nodegroup_id.as_ref(),
                model,
            )
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result_map = JsMap::new();
        for (alias, value) in values {
            result_map.set(&JsValue::from_str(&alias), &semantic_child_to_js(value));
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
    resource_id: Option<String>,
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
    let wrapper = WASMResourceInstanceWrapper::new_from_graph_id(graph_id.to_string());

    // If a resource_id is provided, set minimal resource metadata so the
    // tile source can resolve this instance's tiles.
    if let Some(rid) = resource_id {
        use alizarin_core::graph::{StaticResourceDescriptors, StaticResourceMetadata};
        wrapper.core.borrow_mut().resource_instance = Some(StaticResourceMetadata {
            resourceinstanceid: rid,
            graph_id: graph_id.to_string(),
            name: String::new(),
            descriptors: StaticResourceDescriptors::default(),
            publication_id: None,
            principaluser_id: None,
            legacyid: None,
            graph_publication_id: None,
            createdtime: None,
            lastmodified: None,
        });
    }

    Ok(wrapper)
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

#[cfg(test)]
mod adoption_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn pseudo_cache_round_trips_through_shared_mut() {
        let w = WASMResourceInstanceWrapper::new_from_graph_id("test-graph".to_string());

        let list =
            alizarin_core::PseudoListCore::new_with_cardinality("greeting".to_string(), true);
        w.core.borrow().store_pseudo("greeting".to_string(), list);

        assert!(
            w.get_cached_pseudo(Some("greeting".to_string())).is_some(),
            "a cached list must survive the SharedMut write→read round-trip"
        );
        assert!(
            w.get_cached_pseudo(Some("absent".to_string())).is_none(),
            "an uncached alias reads back as None, not a spurious hit"
        );
    }

    #[wasm_bindgen_test]
    fn semantic_child_of_an_unregistered_model_is_a_clean_error() {
        let core =
            alizarin_core::ResourceInstanceWrapperCore::new("unregistered-graph".to_string());
        let result = semantic_child_value(&core, None, "parent-node", None, "child");
        assert!(result.is_err(), "an unregistered model is a clean error");
    }
}
