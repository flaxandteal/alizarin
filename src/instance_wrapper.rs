use wasm_bindgen::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use serde_json::Value as JsonValue;
use crate::graph::{StaticTile, StaticNode};
use crate::pseudo_value::{RustPseudoValue, RustPseudoList, RustPseudoGroup, WasmPseudoList, WasmPseudoValue};
use js_sys::{Array, Map as JsMap};
use wasm_bindgen::JsCast;

/// Recipe for creating a PseudoValue in JavaScript
/// Contains all information needed to instantiate a Pseudo without holding full tile data
#[wasm_bindgen]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PseudoRecipe {
    node_alias: String,
    tile_id: Option<String>,
    node_id: String,
    is_collector: bool,
    sentinel_undefined: bool,
}

#[wasm_bindgen]
impl PseudoRecipe {
    #[wasm_bindgen(getter = nodeAlias)]
    pub fn node_alias(&self) -> String {
        self.node_alias.clone()
    }

    #[wasm_bindgen(getter = tileId)]
    pub fn tile_id(&self) -> Option<String> {
        self.tile_id.clone()
    }

    #[wasm_bindgen(getter = nodeId)]
    pub fn node_id(&self) -> String {
        self.node_id.clone()
    }

    #[wasm_bindgen(getter = isCollector)]
    pub fn is_collector(&self) -> bool {
        self.is_collector
    }

    #[wasm_bindgen(getter = sentinelUndefined)]
    pub fn sentinel_undefined(&self) -> bool {
        self.sentinel_undefined
    }
}

/// Result from values_from_resource_nodegroup
/// PORT: Phase 3 - Now internally uses structured RustPseudoList
/// Still exposes recipes to WASM for backward compatibility until Phase 4
#[derive(Clone)]
pub struct ValuesFromNodegroupResult {
    /// Map of node alias → RustPseudoList (structured hierarchy - internal use)
    pub values: HashMap<String, RustPseudoList>,
    pub implied_nodegroups: Vec<String>,
}

impl ValuesFromNodegroupResult {
    /// Convert structured values back to recipes for WASM compatibility
    /// TODO: Remove after Phase 4 (JS wrapper layer complete)
    fn to_recipes(&self) -> Vec<PseudoRecipe> {
        let mut recipes = Vec::new();

        for (alias, pseudo_list) in &self.values {
            // Each group in the list represents tiles
            for group in &pseudo_list.groups {
                for value in &group.values {
                    recipes.push(PseudoRecipe {
                        node_alias: alias.clone(),
                        tile_id: value.tile.as_ref().and_then(|t| t.tileid.clone()),
                        node_id: value.node.nodeid.clone(),
                        is_collector: value.is_collector,
                        sentinel_undefined: false,
                    });
                }
            }
        }

        recipes
    }
}

/// Result from ensure_nodegroup
/// PORT: Phase 4c - Now returns structured RustPseudoList values directly
pub struct EnsureNodegroupResult {
    /// Structured values by alias
    /// PORT: Map of alias → RustPseudoList (js/graphManager.ts:350 - newValues Map)
    pub values: HashMap<String, RustPseudoList>,
    pub implied_nodegroups: Vec<String>,
    pub all_nodegroups_map: HashMap<String, serde_json::Value>,
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
    pub all_values_map: HashMap<String, serde_json::Value>,
    pub all_nodegroups_map: HashMap<String, serde_json::Value>,
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

    #[wasm_bindgen(getter = allValuesMap)]
    pub fn all_values_map(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.all_values_map).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter = allNodegroupsMap)]
    pub fn all_nodegroups_map(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.all_nodegroups_map).unwrap_or(JsValue::NULL)
    }
}

/// Rust-side resource instance wrapper that owns tile data
/// Manages tile storage, indexing, and provides query methods
/// Phase 4g: Added PseudoValue cache for parallel access
#[wasm_bindgen]
pub struct WASMResourceInstanceWrapper {
    // Core tile storage
    tiles: HashMap<String, StaticTile>,

    // Index: nodegroup_id -> list of tile_ids
    nodegroup_index: HashMap<String, Vec<String>>,

    // Track which nodegroups have been loaded/loading
    // Phase 4g: Now uses Mutex for thread-safe parallel access
    loaded_nodegroups: Arc<Mutex<HashMap<String, LoadState>>>,

    // Phase 4g: Cache of PseudoValues (alias -> RustPseudoList)
    // This allows Rust to own the authoritative data and TS to create lightweight wrappers
    pseudo_cache: Arc<Mutex<HashMap<String, RustPseudoList>>>,
}

/// Phase 4g: Track loading state to prevent race conditions
#[derive(Clone, Debug, PartialEq)]
enum LoadState {
    NotLoaded,
    Loading,
    Loaded,
}

// Manual Clone implementation since Arc<Mutex<>> is Clone but wasm_bindgen doesn't auto-derive
impl Clone for WASMResourceInstanceWrapper {
    fn clone(&self) -> Self {
        WASMResourceInstanceWrapper {
            tiles: self.tiles.clone(),
            nodegroup_index: self.nodegroup_index.clone(),
            loaded_nodegroups: Arc::clone(&self.loaded_nodegroups),
            pseudo_cache: Arc::clone(&self.pseudo_cache),
        }
    }
}

#[wasm_bindgen]
impl WASMResourceInstanceWrapper {
    /// Create a new empty resource instance wrapper
    #[wasm_bindgen(constructor)]
    pub fn new() -> WASMResourceInstanceWrapper {
        WASMResourceInstanceWrapper {
            tiles: HashMap::new(),
            nodegroup_index: HashMap::new(),
            loaded_nodegroups: Arc::new(Mutex::new(HashMap::new())),
            pseudo_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Load tiles from JavaScript into Rust storage
    /// This is called during ResourceInstanceWrapper construction
    #[wasm_bindgen(js_name = loadTiles)]
    pub fn load_tiles(&mut self, tiles_js: JsValue) -> Result<(), JsValue> {
        // Deserialize JS array of tiles
        let tiles: Vec<StaticTile> = serde_wasm_bindgen::from_value(tiles_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize tiles: {:?}", e)))?;

        // Clear existing data
        self.tiles.clear();
        self.nodegroup_index.clear();

        // Store tiles and build index
        for mut tile in tiles {
            // Ensure tile has an ID
            let tile_id = tile.ensure_id();
            let nodegroup_id = tile.nodegroup_id.clone();

            // Add to nodegroup index
            self.nodegroup_index
                .entry(nodegroup_id)
                .or_insert_with(Vec::new)
                .push(tile_id.clone());

            // Store tile
            self.tiles.insert(tile_id, tile);
        }

        Ok(())
    }

    /// Get tile IDs for a specific nodegroup
    /// Returns array of tile ID strings
    #[wasm_bindgen(js_name = getTileIdsByNodegroup)]
    pub fn get_tile_ids_by_nodegroup(&self, nodegroup_id: &str) -> Vec<String> {
        self.nodegroup_index
            .get(nodegroup_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get full tile data by tile ID
    /// Returns StaticTile WASM object or error if not found
    #[wasm_bindgen(js_name = getTile)]
    pub fn get_tile(&self, tile_id: &str) -> Result<StaticTile, JsValue> {
        self.tiles
            .get(tile_id)
            .cloned()
            .ok_or_else(|| JsValue::from_str(&format!("Tile not found: {}", tile_id)))
    }

    /// Get specific node data from a tile
    /// Returns the data value for the given node_id within the tile
    #[wasm_bindgen(js_name = getTileData)]
    pub fn get_tile_data(&self, tile_id: &str, node_id: &str) -> Result<JsValue, JsValue> {
        let tile = self.tiles
            .get(tile_id)
            .ok_or_else(|| JsValue::from_str(&format!("Tile not found: {}", tile_id)))?;

        // Get data for specific node from tile.data HashMap
        match tile.data.get(node_id) {
            Some(value) => serde_wasm_bindgen::to_value(value)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize data: {:?}", e))),
            None => Ok(JsValue::NULL),
        }
    }

    /// Mark a nodegroup as loaded to prevent re-loading
    /// Phase 4g: Now uses Mutex for thread-safe access
    #[wasm_bindgen(js_name = markNodegroupLoaded)]
    pub fn mark_nodegroup_loaded(&self, nodegroup_id: String) {
        if let Ok(mut loaded) = self.loaded_nodegroups.lock() {
            loaded.insert(nodegroup_id, LoadState::Loaded);
        }
    }

    /// Phase 4g: Check if a nodegroup is being loaded or already loaded
    #[wasm_bindgen(js_name = isNodegroupLoadedOrLoading)]
    pub fn is_nodegroup_loaded_or_loading(&self, nodegroup_id: &str) -> bool {
        if let Ok(loaded) = self.loaded_nodegroups.lock() {
            matches!(loaded.get(nodegroup_id), Some(LoadState::Loading) | Some(LoadState::Loaded))
        } else {
            false
        }
    }

    /// Phase 4g: Try to atomically acquire loading lock for a nodegroup
    /// Returns true if caller should proceed with loading, false if already being loaded
    #[wasm_bindgen(js_name = tryAcquireNodegroupLock)]
    pub fn try_acquire_nodegroup_lock(&self, nodegroup_id: String) -> bool {
        if let Ok(mut loaded) = self.loaded_nodegroups.lock() {
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
        if let Ok(cache) = self.pseudo_cache.lock() {
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
        if let Ok(mut cache) = self.pseudo_cache.lock() {
            cache.insert(alias, rust_list);
        }
    }

    /// Phase 4g: Store single WasmPseudoValue as a list in Rust cache
    #[wasm_bindgen(js_name = cachePseudoValue)]
    pub fn cache_pseudo_value(&self, alias: String, wasm_value: WasmPseudoValue) {
        let rust_value = wasm_value.into_inner();
        let tile_id = rust_value.tile.as_ref().and_then(|t| t.tileid.clone());
        let node_alias = alias.clone();  // Use the provided alias
        let rust_list = RustPseudoList {
            node_alias,
            groups: vec![RustPseudoGroup {
                tile_id,
                values: vec![rust_value],
            }],
            is_loaded: true,
        };
        if let Ok(mut cache) = self.pseudo_cache.lock() {
            cache.insert(alias, rust_list);
        }
    }

    /// Phase 4g: Clear all cached PseudoValues
    #[wasm_bindgen(js_name = clearPseudoCache)]
    pub fn clear_pseudo_cache(&self) {
        if let Ok(mut cache) = self.pseudo_cache.lock() {
            cache.clear();
        }
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
        node_objs_js: JsValue,
        edges_js: JsValue,
        is_single: bool,
    ) -> Result<JsValue, JsValue> {
        // PORT: js/pseudos.ts:506-510 - Get node by alias
        let node_objs = self.deserialize_node_map(node_objs_js)?;
        let node = node_objs.get(alias)
            .ok_or_else(|| JsValue::from_str(&format!("Could not find node by alias: {}", alias)))?;

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

            // Deserialize edges for child lookup
            let edges = self.deserialize_edges_map(edges_js)?;

            // Create list of RustPseudoValues from each tile
            let mut values = Vec::new();

            for tid in tile_ids {
                let tile = self.tiles.get(&tid);
                if let Some(tile) = tile {
                    // Get tile data for this node
                    let tile_data = tile.data.get(&node.nodeid);

                    // Get child node IDs from edges
                    let child_ids = edges.get(&node.nodeid)
                        .cloned()
                        .unwrap_or_default();

                    // Create RustPseudoValue for this tile
                    let pseudo_value = RustPseudoValue::from_node_and_tile(
                        Arc::new(node.clone()),
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
                self.tiles.get(&tid).cloned()
            } else {
                None
            };

            // Extract tile data if we have a tile
            let tile_data = if let Some(ref t) = tile {
                t.data.get(&node.nodeid).cloned()
            } else {
                None
            };

            // Get child node IDs from edges
            let edges = self.deserialize_edges_map(edges_js)?;
            let child_ids = edges.get(&node.nodeid)
                .cloned()
                .unwrap_or_default();

            // Create RustPseudoValue
            let pseudo_value = RustPseudoValue::from_node_and_tile(
                Arc::new(node.clone()),
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
        if let Ok(loaded) = self.loaded_nodegroups.lock() {
            matches!(loaded.get(nodegroup_id), Some(LoadState::Loaded))
        } else {
            false
        }
    }

    /// Get count of tiles stored
    #[wasm_bindgen(js_name = getTileCount)]
    pub fn get_tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Get count of nodegroups indexed
    #[wasm_bindgen(js_name = getNodegroupCount)]
    pub fn get_nodegroup_count(&self) -> usize {
        self.nodegroup_index.len()
    }

    // ========================================================================
    // Phase 2: Graph Traversal Helpers
    // ========================================================================

    /// Complete ensureNodegroup implementation in Rust
    /// PORT: graphManager.ts lines 302-377 (full ensureNodegroup function)
    /// PORT: Phase 4c - Now returns structured WasmEnsureNodegroupResult instead of recipes
    #[wasm_bindgen(js_name = ensureNodegroup)]
    pub fn ensure_nodegroup(
        &self,
        all_values_js: JsValue,        // Map<string, any> - mutable
        all_nodegroups_js: JsValue,    // Map<string, boolean | Promise> - mutable
        nodegroup_id: &str,
        node_objs_js: JsValue,         // Map<string, StaticNode>
        nodegroup_objs_js: JsValue,    // Map<string, StaticNodegroup>
        edges_js: JsValue,             // Map<string, string[]>
        add_if_missing: bool,
        all_tiles: Vec<String>,        // All tile IDs available
        nodegroup_permissions_js: JsValue,  // Phase 4h: Map<nodegroupId, boolean>
        do_implied_nodegroups: bool,
    ) -> Result<WasmEnsureNodegroupResult, JsValue> {
        use std::collections::{HashMap, HashSet};

        // Deserialize all_nodegroups to check sentinel
        let mut all_nodegroups: HashMap<String, serde_json::Value> = serde_wasm_bindgen::from_value(all_nodegroups_js.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize all_nodegroups: {:?}", e)))?;

        // Check sentinel state (line 314)
        let sentinel = all_nodegroups.get(nodegroup_id);
        let should_process = match sentinel {
            Some(serde_json::Value::Bool(false)) => true,  // sentinel === false (force reload)
            Some(serde_json::Value::Bool(true)) => false,  // sentinel === true (already loaded)
            None => add_if_missing,                         // sentinel === undefined
            _ => false,  // Promise or other - skip
        };

        // PORT: Phase 4c - Changed from Vec<PseudoRecipe> to HashMap<String, RustPseudoList>
        // PORT: js/graphManager.ts:350 - newValues is a Map<string, PseudoValue | PseudoList>
        let mut all_values: HashMap<String, RustPseudoList> = HashMap::new();
        let mut implied_nodegroups_set: HashSet<String> = HashSet::new();

        if should_process {
            // Delete old node values (lines 318-320)
            let node_objs: HashMap<String, serde_json::Value> = serde_wasm_bindgen::from_value(node_objs_js.clone())
                .map_err(|e| JsValue::from_str(&format!("Failed to deserialize node_objs: {:?}", e)))?;

            // Phase 4h: Deserialize nodegroup permissions and compute tile permissions internally
            let nodegroup_permissions: HashMap<String, bool> = serde_wasm_bindgen::from_value(nodegroup_permissions_js.clone())
                .map_err(|e| JsValue::from_str(&format!("Failed to deserialize nodegroup_permissions: {:?}", e)))?;

            // Filter tiles by nodegroup_id and permissions (lines 326-328)
            // Phase 4h: Compute tile permission from nodegroup permission
            let mut nodegroup_tiles: Vec<String> = Vec::new();
            for tile_id in all_tiles.iter() {
                if let Some(tile) = self.tiles.get(tile_id) {
                    if tile.nodegroup_id == nodegroup_id {
                        // Phase 4h: Look up permission by tile's nodegroup_id
                        let permitted = nodegroup_permissions.get(&tile.nodegroup_id).copied().unwrap_or(true);
                        if permitted {
                            nodegroup_tiles.push(tile_id.clone());
                        }
                    }
                }
            }

            // If no tiles and addIfMissing, use empty string to indicate null tile (lines 329-330)
            if nodegroup_tiles.is_empty() && add_if_missing {
                nodegroup_tiles.push(String::new());
            }

            // Call values_from_resource_nodegroup_internal (line 332)
            // PORT: Phase 4c - Use structured values directly (no recipe conversion)
            // PORT: js/graphManager.ts:352 - iterating over result and adding to newValues
            let values_result = self.values_from_resource_nodegroup_internal(
                all_values_js.clone(),
                nodegroup_tiles,
                nodegroup_id,
                node_objs_js.clone(),
                edges_js.clone(),
            )?;

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
            all_nodegroups.insert(nodegroup_id.to_string(), serde_json::Value::Bool(true));

            // Recursive processing of implied nodegroups (lines 355-373)
            if do_implied_nodegroups && !implied_nodegroups_set.is_empty() {
                let implied_list: Vec<String> = implied_nodegroups_set.iter().cloned().collect();

                for implied_ng in implied_list.iter() {
                    // Recursive call (lines 358-368)
                    let implied_result = self.ensure_nodegroup(
                        all_values_js.clone(),
                        serde_wasm_bindgen::to_value(&all_nodegroups)?,
                        implied_ng,
                        node_objs_js.clone(),
                        nodegroup_objs_js.clone(),
                        edges_js.clone(),
                        true,  // addIfMissing = true for implied
                        all_tiles.clone(),
                        nodegroup_permissions_js.clone(),  // Phase 4h
                        true,  // doImpliedNodegroups = true
                    )?;

                    // Merge implied values (lines 369-371)
                    // PORT: Phase 4c - Merge RustPseudoList values instead of recipes
                    // PORT: js/graphManager.ts:369-371 - merging newValues from recursive call
                    for alias in implied_result.get_value_aliases() {
                        if let Some(pseudo_list) = implied_result.get_value(&alias) {
                            all_values.insert(alias, pseudo_list.into_inner());
                        }
                    }

                    // Update all_nodegroups from recursive call
                    let implied_nodegroups_map: HashMap<String, serde_json::Value> =
                        serde_wasm_bindgen::from_value(implied_result.all_nodegroups_map())?;
                    for (k, v) in implied_nodegroups_map.iter() {
                        all_nodegroups.insert(k.clone(), v.clone());
                    }
                }

                // Clear implied set after processing (line 373)
                implied_nodegroups_set.clear();
            }
        }

        // PORT: Phase 4c - Return structured values wrapped in WasmEnsureNodegroupResult
        // PORT: js/graphManager.ts:377 - return { newValues, impliedNodegroups, ... }
        Ok(WasmEnsureNodegroupResult {
            inner: EnsureNodegroupResult {
                values: all_values,
                implied_nodegroups: implied_nodegroups_set.into_iter().collect(),
                all_nodegroups_map: all_nodegroups,
            }
        })
    }

    /// Complete populate implementation in Rust
    /// PORT: graphManager.ts lines 600-688 (populate function)
    /// PORT: Phase 4c - Now returns WasmPopulateResult with structured values
    /// Orchestrates loading all nodegroups for a resource
    #[wasm_bindgen(js_name = populate)]
    pub fn populate(
        &self,
        lazy: bool,
        nodegroup_ids: Vec<String>,
        root_node_alias: String,
        node_objs_js: JsValue,
        nodegroup_objs_js: JsValue,
        edges_js: JsValue,
        all_tiles: Vec<String>,
        nodegroup_permissions_js: JsValue,  // Phase 4h: Map<nodegroupId, boolean>
    ) -> Result<WasmPopulateResult, JsValue> {
        // Initialize state maps
        let mut all_values: HashMap<String, serde_json::Value> = HashMap::new();
        let mut all_nodegroups: HashMap<String, serde_json::Value> = HashMap::new();

        // Initialize all nodegroups to false (line 610-612)
        for nodegroup_id in nodegroup_ids.iter() {
            all_nodegroups.insert(nodegroup_id.clone(), serde_json::Value::Bool(false));
        }

        // Set root node alias to false (line 626)
        all_values.insert(root_node_alias.clone(), serde_json::Value::Bool(false));

        // PORT: Phase 4c - Collect structured RustPseudoList values instead of recipes
        // PORT: js/graphManager.ts:668 - newValues is a Map<string, PseudoValue | PseudoList>
        let mut all_structured_values: HashMap<String, RustPseudoList> = HashMap::new();

        // Non-lazy loading: process all nodegroups
        if !lazy {
            // Phase 1: Process all nodegroups with doImpliedNodegroups=false (lines 636-653)
            let mut implied_nodegroups_set = HashSet::new();

            for nodegroup_id in nodegroup_ids.iter() {
                let result = self.ensure_nodegroup(
                    serde_wasm_bindgen::to_value(&all_values)?,
                    serde_wasm_bindgen::to_value(&all_nodegroups)?,
                    nodegroup_id,
                    node_objs_js.clone(),
                    nodegroup_objs_js.clone(),
                    edges_js.clone(),
                    true,  // addIfMissing
                    all_tiles.clone(),
                    nodegroup_permissions_js.clone(),  // Phase 4h
                    false, // doImpliedNodegroups = false for phase 1
                )?;

                // Collect structured values
                // PORT: Phase 4c - Merge RustPseudoList values from ensure_nodegroup result
                // PORT: js/graphManager.ts:669-720 - Processing result.recipes becomes iterating values
                for alias in result.get_value_aliases() {
                    if let Some(pseudo_list) = result.get_value(&alias) {
                        all_structured_values.insert(alias, pseudo_list.into_inner());
                    }
                }

                // Update all_values and all_nodegroups from result
                let result_values: HashMap<String, serde_json::Value> =
                    serde_wasm_bindgen::from_value(result.all_nodegroups_map())?;
                for (k, v) in result_values.iter() {
                    all_nodegroups.insert(k.clone(), v.clone());
                }

                // Collect implied nodegroups (lines 649-652)
                for implied_ng in result.implied_nodegroups().iter() {
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
                        Some(serde_json::Value::Bool(false)) => true,
                        None => true,
                        _ => false,
                    };

                    if should_process {
                        let result = self.ensure_nodegroup(
                            serde_wasm_bindgen::to_value(&all_values)?,
                            serde_wasm_bindgen::to_value(&all_nodegroups)?,
                            nodegroup_id,
                            node_objs_js.clone(),
                            nodegroup_objs_js.clone(),
                            edges_js.clone(),
                            true,  // addIfMissing
                            all_tiles.clone(),
                            nodegroup_permissions_js.clone(),  // Phase 4h
                            true,  // doImpliedNodegroups = true for phase 2
                        )?;

                        // Collect structured values
                        // PORT: Phase 4c - Merge structured values from implied nodegroup
                        for alias in result.get_value_aliases() {
                            if let Some(pseudo_list) = result.get_value(&alias) {
                                all_structured_values.insert(alias, pseudo_list.into_inner());
                            }
                        }

                        // Update all_nodegroups from result
                        let result_values: HashMap<String, serde_json::Value> =
                            serde_wasm_bindgen::from_value(result.all_nodegroups_map())?;
                        for (k, v) in result_values.iter() {
                            all_nodegroups.insert(k.clone(), v.clone());
                        }

                        // Collect new implied nodegroups (lines 671-673)
                        for implied_ng in result.implied_nodegroups().iter() {
                            implied_nodegroups_set.insert(implied_ng.clone());
                        }
                    }
                }
            }
        }
        // If lazy: just skip loading (lines 678-680 in JS - stripTiles happens in JS)

        // PORT: Phase 4c - Return structured values wrapped in WasmPopulateResult
        // PORT: js/graphManager.ts:724-729 - returning allValues and allNodegroups
        Ok(WasmPopulateResult {
            inner: PopulateResult {
                values: all_structured_values,
                all_values_map: all_values,
                all_nodegroups_map: all_nodegroups,
            }
        })
    }

    /// PORT: graphManager.ts lines 505-643
    /// Internal implementation - returns structured RustPseudoList (Phase 3)
    fn values_from_resource_nodegroup_internal(
        &self,
        existing_values_js: JsValue,  // Map<string, any>
        nodegroup_tile_ids: Vec<String>,  // Tile IDs for this nodegroup (nulls become None)
        nodegroup_id: &str,
        node_objs_js: JsValue,  // Map<string, StaticNode>
        edges_js: JsValue,  // Map<string, string[]>
    ) -> Result<ValuesFromNodegroupResult, JsValue> {
        // Deserialize JS Maps to Rust HashMaps
        let node_objs = self.deserialize_node_map(node_objs_js)?;
        let edges = self.deserialize_edges_map(edges_js)?;
        let existing_values = self.deserialize_existing_values(existing_values_js)?;

        // PORT: Phase 3 - Convert nodes to Arc for sharing in RustPseudoValue structures
        let node_objs_arc: HashMap<String, Arc<StaticNode>> = node_objs
            .iter()
            .map(|(k, v)| (k.clone(), Arc::new(v.clone())))
            .collect();

        // Get tiles for this nodegroup
        let mut nodegroup_tiles: Vec<Option<&StaticTile>> = Vec::new();
        for tile_id in &nodegroup_tile_ids {
            if tile_id.is_empty() {
                nodegroup_tiles.push(None); // null tile
            } else {
                let tile = self.tiles.get(tile_id);
                nodegroup_tiles.push(tile);
            }
        }

        // PORT: Line 514
        let mut implied_nodegroups: HashSet<String> = HashSet::new();
        // PORT: Line 515
        let mut implied_nodes: HashMap<String, (StaticNode, String)> = HashMap::new(); // key = nodeid+tileid, value = (node, tileid)

        // PORT: Lines 517-521 - Track unseen nodes
        let mut nodes_unseen: HashSet<String> = node_objs
            .values()
            .filter(|node| node.nodegroup_id.as_deref() == Some(nodegroup_id))
            .filter_map(|node| node.alias.clone())
            .collect();

        // PORT: Line 522 - Track seen tile-node pairs
        let mut tile_nodes_seen: HashSet<(String, String)> = HashSet::new(); // (nodeid, tileid)

        let mut recipes: Vec<PseudoRecipe> = Vec::new();

        // Helper to add a pseudo recipe
        // PORT: Lines 523-582
        let mut add_pseudo = |node: &StaticNode, tile: Option<&StaticTile>| {
            let key = node.alias.clone().unwrap_or_default();

            // PORT: Line 525
            nodes_unseen.remove(&key);

            // PORT: Lines 526-528
            let tileid: Option<String> = tile.and_then(|t| t.tileid.clone());
            if let Some(ref tid) = tileid {
                tile_nodes_seen.insert((node.nodeid.clone(), tid.clone()));
            }

            // PORT: Lines 530-537 - Check if already exists
            if let Some(existing) = existing_values.get(&key) {
                if !existing.is_false && !existing.is_undefined {
                    // Already loaded, skip
                    return;
                }
            }

            // PORT: Lines 546-563 - Discover implied nodegroups and nodes
            for (domain, ranges) in &edges {
                if ranges.contains(&node.nodeid) {
                    if let Some(domain_node) = node_objs.get(domain) {
                        // PORT: Lines 552-557 - Implied nodegroup
                        if let Some(ref ng_id) = domain_node.nodegroup_id {
                            if !ng_id.is_empty() && ng_id != nodegroup_id {
                                implied_nodegroups.insert(ng_id.clone());
                            }
                        }

                        // PORT: Lines 558-560 - Implied node (shares tile)
                        if let (Some(ref domain_ng), Some(tile), Some(ref tid)) =
                            (&domain_node.nodegroup_id, tile, &tileid) {
                            if domain_ng == &tile.nodegroup_id
                                && domain_ng != &domain_node.nodeid
                                && !implied_nodes.contains_key(&format!("{}{}", domain_node.nodeid, tid)) {
                                implied_nodes.insert(
                                    format!("{}{}", domain_node.nodeid, tid),
                                    (domain_node.clone(), tid.clone())
                                );
                            }
                        }
                    }
                    break;
                }
            }

            // PORT: Lines 542, 581 - Add the recipe
            recipes.push(PseudoRecipe {
                node_alias: key,
                tile_id: tileid,
                node_id: node.nodeid.clone(),
                is_collector: node.is_collector,
                sentinel_undefined: false,
            });
        };

        // PORT: Lines 584-621 - Main loop over tiles
        for tile_opt in nodegroup_tiles {
            // PORT: Lines 585-591 - Add parent node
            if let Some(parent_node) = node_objs.get(nodegroup_id) {
                if parent_node.nodegroup_id.as_deref().unwrap_or("") == ""
                    || parent_node.nodegroup_id.as_deref() == Some(nodegroup_id) {
                    add_pseudo(parent_node, tile_opt);
                }
            }

            // PORT: Lines 593-620 - Process tile data
            if let Some(tile) = tile_opt {
                // PORT: Lines 594-597 - Get tile nodes
                let mut tile_nodes: HashMap<String, JsonValue> = HashMap::new();
                for (key, value) in &tile.data {
                    tile_nodes.insert(key.clone(), value.clone());
                }

                // PORT: Lines 599-602 - Add semantic nodes without data
                for node in node_objs.values() {
                    if node.nodegroup_id.as_deref() == Some(nodegroup_id)
                        && !tile_nodes.contains_key(&node.nodeid)
                        && node.datatype == "semantic" {
                        tile_nodes.insert(node.nodeid.clone(), JsonValue::Object(serde_json::Map::new()));
                    }
                }

                // PORT: Lines 604-606 - Ensure nodegroup_id is in tile_nodes
                if !tile_nodes.contains_key(&tile.nodegroup_id) {
                    tile_nodes.insert(tile.nodegroup_id.clone(), JsonValue::Object(serde_json::Map::new()));
                }

                // PORT: Lines 607-620 - Add pseudo for each tile node
                for (nodeid, node_value) in tile_nodes {
                    // PORT: Lines 608-611 - Skip nodegroup node
                    if nodeid == nodegroup_id {
                        continue;
                    }

                    if let Some(node) = node_objs.get(&nodeid) {
                        // PORT: Line 616 - Only add if value is not null
                        if !node_value.is_null() {
                            add_pseudo(node, Some(tile));
                        }
                    }
                }
            }
        }

        // PORT: Lines 622-632 - Process implied nodes
        // Note: Can't use add_pseudo closure due to borrow checker, inline the logic
        while !implied_nodes.is_empty() {
            let key = implied_nodes.keys().next().unwrap().clone();
            let (node, tileid) = implied_nodes.remove(&key).unwrap();

            // PORT: Line 627 - Only add if not seen
            if !tile_nodes_seen.contains(&(node.nodeid.clone(), tileid.clone())) {
                let tile = self.tiles.get(&tileid);

                // Inline add_pseudo logic
                let key = node.alias.clone().unwrap_or_default();
                nodes_unseen.remove(&key);

                // tileid is already a String here (from the tuple), not Option
                tile_nodes_seen.insert((node.nodeid.clone(), tileid.clone()));

                // Check if already exists
                if let Some(existing) = existing_values.get(&key) {
                    if existing.is_false || existing.is_undefined {
                        // Can add
                        recipes.push(PseudoRecipe {
                            node_alias: key.clone(),
                            tile_id: Some(tileid.clone()),
                            node_id: node.nodeid.clone(),
                            is_collector: node.is_collector,
                            sentinel_undefined: false,
                        });
                    }
                } else {
                    // Not exists, add
                    recipes.push(PseudoRecipe {
                        node_alias: key,
                        tile_id: Some(tileid),
                        node_id: node.nodeid.clone(),
                        is_collector: node.is_collector,
                        sentinel_undefined: false,
                    });
                }
            }
        }

        // PORT: Lines 635-641 - Mark unseen nodes as undefined (sentinel)
        for node_unseen in nodes_unseen {
            if !node_unseen.is_empty() {
                recipes.push(PseudoRecipe {
                    node_alias: node_unseen,
                    tile_id: None,
                    node_id: String::new(),
                    is_collector: false,
                    sentinel_undefined: true,  // Special flag for undefined sentinel
                });
            }
        }

        // PORT: Phase 3 - Convert recipes to structured RustPseudoList hierarchy
        // This conversion happens AFTER the TS algorithm (preserving all PORT line numbers above)
        // The recipes are generated following js/graphManager.ts:1099-1230 exactly
        // Now we convert those recipes into RustPseudoList structures
        let values = self.recipes_to_pseudo_lists(
            &recipes,
            &node_objs_arc,
            &edges,
        );

        Ok(ValuesFromNodegroupResult {
            values,
            implied_nodegroups: implied_nodegroups.into_iter().collect(),
        })
    }

    /// Convert flat recipes to structured RustPseudoList hierarchy
    /// PORT: Phase 3 new conversion layer - not in original TS (TS builds hierarchy in makePseudoCls)
    /// This bridges the gap: Rust algorithm produces recipes (matching TS), then converts to hierarchy
    fn recipes_to_pseudo_lists(
        &self,
        recipes: &[PseudoRecipe],
        node_objs: &HashMap<String, Arc<StaticNode>>,
        edges: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, RustPseudoList> {
        // Group recipes by node alias
        let mut recipes_by_alias: HashMap<String, Vec<&PseudoRecipe>> = HashMap::new();
        for recipe in recipes {
            recipes_by_alias
                .entry(recipe.node_alias.clone())
                .or_insert_with(Vec::new)
                .push(recipe);
        }

        let mut values: HashMap<String, RustPseudoList> = HashMap::new();

        for (alias, alias_recipes) in recipes_by_alias {
            // Skip sentinel undefined entries
            if alias_recipes.iter().all(|r| r.sentinel_undefined) {
                continue;
            }

            // Find the node for this alias
            if let Some(node) = node_objs.get(&alias_recipes[0].node_id) {
                // Collect tiles for this node
                let mut tiles: Vec<Option<Arc<StaticTile>>> = Vec::new();

                for recipe in &alias_recipes {
                    if recipe.sentinel_undefined {
                        continue;
                    }

                    let tile = if let Some(ref tile_id) = recipe.tile_id {
                        self.tiles.get(tile_id).map(|t| Arc::new(t.clone()))
                    } else {
                        None
                    };
                    tiles.push(tile);
                }

                // Create RustPseudoList from node and tiles
                // PORT: Equivalent to js/pseudos.ts:452-461 makePseudoCls creating PseudoList
                let pseudo_list = RustPseudoList::from_node_tiles(
                    Arc::clone(node),
                    tiles,
                    edges,
                );

                values.insert(alias, pseudo_list);
            }
        }

        values
    }

    /// WASM-exposed wrapper - returns recipes for backward compatibility
    /// TODO: Remove after Phase 4 (JS uses RustPseudoList directly)
    #[wasm_bindgen(js_name = valuesFromResourceNodegroup)]
    pub fn values_from_resource_nodegroup(
        &self,
        existing_values_js: JsValue,
        nodegroup_tile_ids: Vec<String>,
        nodegroup_id: &str,
        node_objs_js: JsValue,
        edges_js: JsValue,
    ) -> Result<JsValue, JsValue> {
        // Call internal version that returns structured values
        let result = self.values_from_resource_nodegroup_internal(
            existing_values_js,
            nodegroup_tile_ids,
            nodegroup_id,
            node_objs_js,
            edges_js,
        )?;

        // Convert back to recipes for backward compatibility
        let recipes = result.to_recipes();

        // Return as JS object with recipes and impliedNodegroups
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("recipes"),
            &serde_wasm_bindgen::to_value(&recipes)?,
        )?;
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("impliedNodegroups"),
            &serde_wasm_bindgen::to_value(&result.implied_nodegroups)?,
        )?;

        Ok(obj.into())
    }
}

// ============================================================================
// Helper implementations
// ============================================================================

/// Helper struct to represent existing values from JS
#[derive(Debug)]
struct ExistingValue {
    is_false: bool,
    is_undefined: bool,
}

impl WASMResourceInstanceWrapper {
    /// Deserialize JS Map<string, StaticNode> to Rust HashMap
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

    /// Deserialize existing values map (JS Map<string, any>)
    fn deserialize_existing_values(&self, js_map: JsValue) -> Result<HashMap<String, ExistingValue>, JsValue> {
        let map = JsMap::from(js_map);
        let mut result = HashMap::new();

        map.for_each(&mut |value, key| {
            if let Some(key_str) = key.as_string() {
                let existing = ExistingValue {
                    is_false: value.is_falsy() && !value.is_undefined(),
                    is_undefined: value.is_undefined(),
                };
                result.insert(key_str, existing);
            }
        });

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_storage() {
        let mut wrapper = WASMResourceInstanceWrapper::new();
        assert_eq!(wrapper.get_tile_count(), 0);

        // Test will be expanded when we have tile data
    }
}
