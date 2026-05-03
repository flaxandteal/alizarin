/// NAPI bindings for ResourceInstanceWrapperCore
///
/// Provides the same public interface as WASMResourceInstanceWrapper so that
/// the TypeScript ViewModel layer (ResourceInstanceWrapper / PseudoValue / PseudoList)
/// can work with either backend.
use std::collections::HashMap;
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use std::collections::HashSet;

use alizarin_core::extension_type_registry::ExtensionTypeRegistry;
use alizarin_core::graph::{
    StaticEdge, StaticGraph, StaticNode, StaticNodegroup, StaticResource, StaticTile,
};
use alizarin_core::instance_wrapper_core::ModelAccess;
use alizarin_core::instance_wrapper_core::{
    LoadState, ResourceInstanceWrapperCore, SemanticChildError, SemanticChildResult,
};
use alizarin_core::node_config::NodeConfigManager;
use alizarin_core::permissions::PermissionRule;
use alizarin_core::pseudo_value_core::{PseudoListCore, PseudoValueCore, VisitorContext};
use alizarin_core::rdm_cache::RdmCache;
use alizarin_core::type_serialization::{SerializationContext, SerializationOptions};
use alizarin_core::GraphModelAccess;

// =============================================================================
// Helpers
// =============================================================================

fn ext_registry() -> &'static ExtensionTypeRegistry {
    crate::extension_registry()
}

fn sc_err(e: SemanticChildError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

// =============================================================================
// NapiRdmCache
// =============================================================================

#[napi]
pub struct NapiRdmCache {
    inner: RdmCache,
}

#[napi]
impl NapiRdmCache {
    #[napi(constructor)]
    pub fn new() -> Self {
        NapiRdmCache {
            inner: RdmCache::new(),
        }
    }

    /// Load collections from a SKOS JSON string.
    #[napi]
    pub fn load_from_skos_json(&mut self, json_str: String) -> Result<()> {
        let collections: Vec<alizarin_core::skos::SkosCollection> = serde_json::from_str(&json_str)
            .map_err(|e| napi::Error::from_reason(format!("Invalid SKOS JSON: {e}")))?;
        for collection in collections {
            let rdm_col = alizarin_core::skos_to_rdm_collection(&collection);
            self.inner.add_collection(rdm_col);
        }
        Ok(())
    }

    /// Load a single collection from JSON.
    #[napi]
    pub fn load_collection_json(&mut self, json_str: String) -> Result<()> {
        let collection: alizarin_core::skos::SkosCollection = serde_json::from_str(&json_str)
            .map_err(|e| napi::Error::from_reason(format!("Invalid collection JSON: {e}")))?;
        let rdm_col = alizarin_core::skos_to_rdm_collection(&collection);
        self.inner.add_collection(rdm_col);
        Ok(())
    }

    #[napi(getter)]
    pub fn collection_count(&self) -> u32 {
        self.inner.len() as u32
    }
}

// =============================================================================
// NapiNodeConfigManager
// =============================================================================

#[napi]
pub struct NapiNodeConfigManager {
    inner: NodeConfigManager,
}

#[napi]
impl NapiNodeConfigManager {
    #[napi(constructor)]
    pub fn new() -> Self {
        NapiNodeConfigManager {
            inner: NodeConfigManager::new(),
        }
    }

    /// Build node configs from a graph JSON string.
    #[napi]
    pub fn build_from_graph_json(&mut self, graph_json: String) -> Result<()> {
        self.inner
            .from_graph_json(&graph_json)
            .map_err(napi::Error::from_reason)
    }

    /// Build node configs from a NapiStaticGraph.
    #[napi]
    pub fn build_from_graph(&mut self, graph: &crate::NapiStaticGraph) -> Result<()> {
        self.inner.build_from_graph(graph.inner_ref());
        Ok(())
    }
}

// =============================================================================
// NapiPseudoValue
// =============================================================================

#[napi]
pub struct NapiPseudoValue {
    inner: PseudoValueCore,
}

#[napi]
impl NapiPseudoValue {
    // -- Node metadata getters --

    #[napi(getter)]
    pub fn node(&self) -> serde_json::Value {
        serde_json::to_value(&*self.inner.node).unwrap_or(serde_json::Value::Null)
    }

    #[napi(getter)]
    pub fn node_id(&self) -> Option<String> {
        Some(self.inner.node.nodeid.clone())
    }

    #[napi(getter)]
    pub fn node_alias(&self) -> Option<String> {
        self.inner.node.alias.clone()
    }

    #[napi(getter)]
    pub fn datatype(&self) -> String {
        self.inner.node.datatype.clone()
    }

    #[napi(getter)]
    pub fn nodegroup_id(&self) -> Option<String> {
        self.inner.node.nodegroup_id.clone()
    }

    #[napi(getter)]
    pub fn is_collector(&self) -> bool {
        self.inner.is_collector
    }

    #[napi(getter)]
    pub fn independent(&self) -> bool {
        self.inner.independent
    }

    #[napi(getter)]
    pub fn exportable(&self) -> bool {
        self.inner.node.exportable
    }

    #[napi(getter)]
    pub fn isrequired(&self) -> bool {
        self.inner.node.isrequired
    }

    #[napi(getter)]
    pub fn issearchable(&self) -> bool {
        self.inner.node.issearchable
    }

    #[napi(getter)]
    pub fn ontologyclass(&self) -> serde_json::Value {
        serde_json::to_value(&self.inner.node.ontologyclass).unwrap_or(serde_json::Value::Null)
    }

    #[napi(getter)]
    pub fn hascustomalias(&self) -> bool {
        self.inner.node.hascustomalias
    }

    #[napi(getter)]
    pub fn parentproperty(&self) -> Option<String> {
        self.inner.node.parentproperty.clone()
    }

    #[napi(getter)]
    pub fn description(&self) -> serde_json::Value {
        serde_json::to_value(&self.inner.node.description).unwrap_or(serde_json::Value::Null)
    }

    #[napi(getter)]
    pub fn name(&self) -> serde_json::Value {
        serde_json::to_value(&self.inner.node.name).unwrap_or(serde_json::Value::Null)
    }

    // -- Tile data getters --

    #[napi(getter)]
    pub fn tile_id(&self) -> Option<String> {
        self.inner.tile.as_ref().and_then(|t| t.tileid.clone())
    }

    #[napi(getter)]
    pub fn tile_data(&self) -> serde_json::Value {
        self.inner
            .tile_data
            .clone()
            .unwrap_or(serde_json::Value::Null)
    }

    #[napi(getter)]
    pub fn value_loaded(&self) -> bool {
        self.inner.tile.is_some()
    }

    #[napi]
    pub fn has_tile_data(&self) -> bool {
        self.inner.tile_data.is_some()
    }

    #[napi]
    pub fn set_tile_data(&mut self, value: serde_json::Value) {
        self.inner.tile_data = Some(value);
    }

    // -- Relationship getters --

    #[napi(getter)]
    pub fn child_node_aliases(&self) -> serde_json::Value {
        serde_json::to_value(&self.inner.child_node_ids).unwrap_or(serde_json::Value::Null)
    }

    #[napi]
    pub fn get_child_node_id(&self, index: u32) -> Option<String> {
        self.inner.child_node_ids.get(index as usize).cloned()
    }

    #[napi]
    pub fn is_iterable(&self) -> bool {
        alizarin_core::is_iterable_datatype(&self.inner.node.datatype)
    }

    /// Get sortorder from the tile
    #[napi(getter)]
    pub fn sortorder(&self) -> serde_json::Value {
        self.inner
            .tile
            .as_ref()
            .and_then(|t| t.sortorder)
            .map(|s| serde_json::Value::Number(s.into()))
            .unwrap_or(serde_json::Value::Null)
    }

    /// Snapshot all properties in one call (avoids multiple N-API boundary crossings)
    #[napi]
    pub fn to_snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "nodeId": self.inner.node.nodeid,
            "alias": self.inner.node.alias,
            "datatype": &self.inner.node.datatype,
            "nodegroupId": self.inner.node.nodegroup_id,
            "isCollector": self.inner.is_collector,
            "independent": self.inner.independent,
            "exportable": self.inner.node.exportable,
            "isrequired": self.inner.node.isrequired,
            "issearchable": self.inner.node.issearchable,
            "hascustomalias": self.inner.node.hascustomalias,
            "tileId": self.inner.tile.as_ref().and_then(|t| t.tileid.clone()),
            "tileData": &self.inner.tile_data,
            "valueLoaded": self.inner.tile.is_some(),
            "childNodeIds": &self.inner.child_node_ids,
        })
    }

    /// Clear tile data
    #[napi]
    pub fn clear(&mut self) {
        self.inner.tile_data = None;
    }
}

// =============================================================================
// NapiPseudoList
// =============================================================================

#[napi]
pub struct NapiPseudoList {
    inner: PseudoListCore,
}

#[napi]
impl NapiPseudoList {
    #[napi(getter)]
    pub fn node_alias(&self) -> String {
        self.inner.node_alias.clone()
    }

    #[napi(getter)]
    pub fn total_values(&self) -> u32 {
        self.inner.values.len() as u32
    }

    #[napi(getter)]
    pub fn is_loaded(&self) -> bool {
        self.inner.is_loaded
    }

    #[napi(getter)]
    pub fn is_single(&self) -> bool {
        self.inner.is_single
    }

    #[napi]
    pub fn is_iterable(&self) -> bool {
        !self.inner.is_single
    }

    #[napi]
    pub fn get_value(&self, value_index: u32) -> Option<NapiPseudoValue> {
        self.inner
            .values
            .get(value_index as usize)
            .map(|v| NapiPseudoValue { inner: v.clone() })
    }

    #[napi]
    pub fn get_all_values(&self) -> Vec<NapiPseudoValue> {
        self.inner
            .values
            .iter()
            .map(|v| NapiPseudoValue { inner: v.clone() })
            .collect()
    }
}

// =============================================================================
// NapiPopulateResult
// =============================================================================

#[napi]
pub struct NapiPopulateResult {
    values: HashMap<String, PseudoListCore>,
    all_values_map: HashMap<String, Option<bool>>,
    all_nodegroups_map: HashMap<String, bool>,
}

#[napi]
impl NapiPopulateResult {
    #[napi]
    pub fn get_value_aliases(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    #[napi]
    pub fn get_value(&self, alias: String) -> Option<NapiPseudoList> {
        self.values
            .get(&alias)
            .map(|l| NapiPseudoList { inner: l.clone() })
    }

    /// Get all values as a JSON map (single boundary crossing).
    #[napi]
    pub fn get_all_values(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (alias, list) in &self.values {
            map.insert(
                alias.clone(),
                serde_json::json!({
                    "nodeAlias": list.node_alias,
                    "isSingle": list.is_single,
                    "isLoaded": list.is_loaded,
                    "totalValues": list.values.len(),
                }),
            );
        }
        serde_json::Value::Object(map)
    }

    #[napi(getter)]
    pub fn all_values_map(&self) -> serde_json::Value {
        serde_json::to_value(&self.all_values_map).unwrap_or(serde_json::Value::Null)
    }

    #[napi(getter)]
    pub fn all_nodegroups_map(&self) -> serde_json::Value {
        serde_json::to_value(&self.all_nodegroups_map).unwrap_or(serde_json::Value::Null)
    }
}

// =============================================================================
// NapiEnsureNodegroupResult
// =============================================================================

#[napi]
pub struct NapiEnsureNodegroupResult {
    values: HashMap<String, PseudoListCore>,
    implied_nodegroups: Vec<String>,
    all_nodegroups_map: HashMap<String, bool>,
}

#[napi]
impl NapiEnsureNodegroupResult {
    #[napi]
    pub fn get_value_aliases(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    #[napi]
    pub fn get_value(&self, alias: String) -> Option<NapiPseudoList> {
        self.values
            .get(&alias)
            .map(|l| NapiPseudoList { inner: l.clone() })
    }

    #[napi]
    pub fn get_all_values(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (alias, list) in &self.values {
            map.insert(
                alias.clone(),
                serde_json::json!({
                    "nodeAlias": list.node_alias,
                    "isSingle": list.is_single,
                    "isLoaded": list.is_loaded,
                    "totalValues": list.values.len(),
                }),
            );
        }
        serde_json::Value::Object(map)
    }

    #[napi(getter)]
    pub fn implied_nodegroups(&self) -> Vec<String> {
        self.implied_nodegroups.clone()
    }

    #[napi(getter)]
    pub fn all_nodegroups_map(&self) -> serde_json::Value {
        serde_json::to_value(&self.all_nodegroups_map).unwrap_or(serde_json::Value::Null)
    }
}

// =============================================================================
// NapiValuesFromNodegroupResult
// =============================================================================

#[napi]
pub struct NapiValuesFromNodegroupResult {
    values: HashMap<String, PseudoListCore>,
    implied_nodegroups: Vec<String>,
}

#[napi]
impl NapiValuesFromNodegroupResult {
    #[napi]
    pub fn get_all_values(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (alias, list) in &self.values {
            map.insert(
                alias.clone(),
                serde_json::json!({
                    "nodeAlias": list.node_alias,
                    "isSingle": list.is_single,
                    "isLoaded": list.is_loaded,
                    "totalValues": list.values.len(),
                }),
            );
        }
        serde_json::Value::Object(map)
    }

    #[napi(getter)]
    pub fn implied_nodegroups(&self) -> Vec<String> {
        self.implied_nodegroups.clone()
    }
}

// =============================================================================
// NapiResourceInstanceWrapper
// =============================================================================

#[napi]
pub struct NapiResourceInstanceWrapper {
    inner: ResourceInstanceWrapperCore,
    model_access: GraphModelAccess,
    lazy: bool,
}

#[napi]
impl NapiResourceInstanceWrapper {
    // =========================================================================
    // Construction
    // =========================================================================

    /// Create a wrapper for a given graph (must be registered).
    #[napi(constructor)]
    pub fn new(graph_id: String) -> Result<Self> {
        let graph = alizarin_core::get_graph(&graph_id).ok_or_else(|| {
            napi::Error::from_reason(format!(
                "Graph '{}' not registered. Call registerGraph() first.",
                graph_id
            ))
        })?;

        let model_access = GraphModelAccess::from_graph(&graph);
        let mut core = ResourceInstanceWrapperCore::new(graph_id);
        core.set_cached_indices(&model_access);

        Ok(NapiResourceInstanceWrapper {
            inner: core,
            model_access,
            lazy: false,
        })
    }

    // =========================================================================
    // Tile loading
    // =========================================================================

    /// Load tiles from a JSON array.
    #[napi]
    pub fn load_tiles(&mut self, tiles_js: serde_json::Value) -> Result<()> {
        let tiles: Vec<StaticTile> = serde_json::from_value(tiles_js)
            .map_err(|e| napi::Error::from_reason(format!("Invalid tiles JSON: {e}")))?;
        self.inner.load_tiles(tiles);
        Ok(())
    }

    /// Load tiles directly from a StaticResource JSON.
    #[napi]
    pub fn load_tiles_from_resource(&mut self, resource_js: serde_json::Value) -> Result<()> {
        let resource: StaticResource = serde_json::from_value(resource_js)
            .map_err(|e| napi::Error::from_reason(format!("Invalid resource JSON: {e}")))?;

        self.inner.resource_instance = Some(resource.resourceinstance.clone());

        if let Some(tiles_vec) = resource.tiles {
            self.inner.load_tiles(tiles_vec);
        }
        Ok(())
    }

    /// Load tiles directly from a NapiStaticResourceRegistry by resource ID.
    /// This avoids the JS round-trip of serializing tiles to JS and back.
    #[napi]
    pub fn load_from_registry(
        &mut self,
        resource_id: String,
        registry: &crate::NapiStaticResourceRegistry,
    ) -> Result<bool> {
        let resource = match registry.inner.get_full(&resource_id) {
            Some(r) => r,
            None => return Ok(false),
        };

        self.inner.resource_instance = Some(resource.resourceinstance.clone());

        if let Some(tiles_vec) = &resource.tiles {
            self.inner.load_tiles(tiles_vec.clone());
        }
        Ok(true)
    }

    /// Append tiles incrementally (for lazy loading).
    #[napi]
    pub fn append_tiles(&mut self, tiles_js: serde_json::Value) -> Result<()> {
        let tiles: Vec<StaticTile> = serde_json::from_value(tiles_js)
            .map_err(|e| napi::Error::from_reason(format!("Invalid tiles JSON: {e}")))?;

        if self.inner.tiles.is_none() {
            self.inner.tiles = Some(HashMap::new());
        }

        let tiles_map = self.inner.tiles.as_mut().unwrap();
        for tile in tiles {
            let tile_id = tile
                .tileid
                .clone()
                .unwrap_or_else(|| format!("synthetic_{}", tiles_map.len()));
            self.inner
                .nodegroup_index
                .entry(tile.nodegroup_id.clone())
                .or_default()
                .push(tile_id.clone());
            tiles_map.insert(tile_id, tile);
        }
        Ok(())
    }

    #[napi]
    pub fn get_tile_count(&self) -> u32 {
        self.inner.tiles.as_ref().map(|t| t.len()).unwrap_or(0) as u32
    }

    /// Export all tiles as a JSON string.
    /// Returns the wrapper's current tile state (including any mutations from setTileDataForNode).
    /// Returned as a string for fast boundary crossing — call JSON.parse() on the JS side.
    #[napi]
    pub fn export_tiles_json(&self) -> Result<String> {
        let tiles: Vec<&StaticTile> = self
            .inner
            .tiles
            .as_ref()
            .map(|t| t.values().collect())
            .unwrap_or_default();
        serde_json::to_string(&tiles).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Prune tiles to only keep those in permitted nodegroups.
    #[napi(js_name = "pruneResourceTiles")]
    pub fn prune_resource_tiles(&mut self) -> Result<()> {
        let tiles = self
            .inner
            .tiles
            .take()
            .ok_or_else(|| napi::Error::from_reason("Tiles not initialized".to_string()))?;
        let pruned: HashMap<String, StaticTile> = tiles
            .into_iter()
            .filter(|(_id, tile)| self.model_access.is_nodegroup_permitted(&tile.nodegroup_id))
            .collect();
        self.inner.tiles = Some(pruned);
        Ok(())
    }

    #[napi]
    pub fn tiles_loaded(&self) -> bool {
        self.inner
            .tiles
            .as_ref()
            .map(|t| !self.lazy || !t.is_empty())
            .unwrap_or(false)
    }

    #[napi]
    pub fn get_all_tile_ids(&self) -> Vec<String> {
        self.inner
            .tiles
            .as_ref()
            .map(|t| t.keys().cloned().collect())
            .unwrap_or_default()
    }

    #[napi]
    pub fn get_tile_ids_by_nodegroup(&self, nodegroup_id: Option<String>) -> Vec<String> {
        match nodegroup_id {
            Some(ng_id) => self
                .inner
                .nodegroup_index
                .get(&ng_id)
                .cloned()
                .unwrap_or_default(),
            None => Vec::new(),
        }
    }

    #[napi]
    pub fn get_tile(&self, tile_id: String) -> Result<serde_json::Value> {
        let tile = self
            .inner
            .get_tile(&tile_id)
            .ok_or_else(|| napi::Error::from_reason(format!("Tile '{}' not found", tile_id)))?;
        serde_json::to_value(tile).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    #[napi]
    pub fn get_tile_data(&self, tile_id: String, node_id: String) -> serde_json::Value {
        self.inner
            .get_tile(&tile_id)
            .and_then(|t| t.data.get(&node_id).cloned())
            .unwrap_or(serde_json::Value::Null)
    }

    /// Set a single node's data in a tile, mutating in place.
    /// Returns true if the tile was found and updated.
    #[napi]
    pub fn set_tile_data_for_node(
        &mut self,
        tile_id: String,
        node_id: String,
        value: serde_json::Value,
    ) -> bool {
        self.inner.set_tile_data_for_node(&tile_id, &node_id, value)
    }

    #[napi]
    pub fn has_tiles_for_nodegroup(&self, nodegroup_id: String) -> bool {
        if self.lazy {
            self.inner.is_nodegroup_loaded(&nodegroup_id)
        } else {
            self.inner
                .tiles
                .as_ref()
                .map(|t| !t.is_empty())
                .unwrap_or(false)
        }
    }

    #[napi]
    pub fn get_nodegroup_count(&self) -> u32 {
        self.inner.nodegroup_index.len() as u32
    }

    // =========================================================================
    // Lazy loading state
    // =========================================================================

    #[napi]
    pub fn set_lazy(&mut self, lazy: bool) {
        self.lazy = lazy;
        if lazy && self.inner.tiles.is_none() {
            self.inner.tiles = Some(HashMap::new());
        }
    }

    #[napi]
    pub fn is_nodegroup_loaded(&self, nodegroup_id: Option<String>) -> bool {
        match nodegroup_id {
            Some(id) => self.inner.is_nodegroup_loaded(&id),
            None => false,
        }
    }

    #[napi]
    pub fn is_nodegroup_loaded_or_loading(&self, nodegroup_id: Option<String>) -> bool {
        match nodegroup_id {
            Some(id) => {
                if let Ok(loaded) = self.inner.loaded_nodegroups.lock() {
                    matches!(
                        loaded.get(&id),
                        Some(LoadState::Loaded) | Some(LoadState::Loading)
                    )
                } else {
                    false
                }
            }
            None => false,
        }
    }

    #[napi]
    pub fn try_acquire_nodegroup_lock(&self, nodegroup_id: String) -> bool {
        if let Ok(mut loaded) = self.inner.loaded_nodegroups.lock() {
            let state = loaded.entry(nodegroup_id).or_insert(LoadState::NotLoaded);
            if *state == LoadState::NotLoaded {
                *state = LoadState::Loading;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    #[napi]
    pub fn get_missing_nodegroups_for_children(&self, parent_node_id: String) -> Vec<String> {
        let child_nodes = self.model_access.get_child_nodes(&parent_node_id);

        if !self.lazy {
            return Vec::new();
        }

        let mut missing = Vec::new();
        for child in child_nodes.values() {
            if let Some(ng_id) = &child.nodegroup_id {
                if !self.inner.is_nodegroup_loaded(ng_id) {
                    missing.push(ng_id.to_string());
                }
            }
        }
        missing
    }

    // =========================================================================
    // Resource metadata
    // =========================================================================

    #[napi]
    pub fn get_resource_id(&self) -> Option<String> {
        self.inner
            .resource_instance
            .as_ref()
            .map(|ri| ri.resourceinstanceid.clone())
    }

    #[napi]
    pub fn get_name(&self) -> Result<serde_json::Value> {
        match self.inner.resource_instance.as_ref() {
            Some(ri) => serde_json::to_value(&ri.name)
                .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e))),
            None => Ok(serde_json::Value::Null),
        }
    }

    #[napi]
    pub fn get_descriptors(&self, _recompute: bool) -> Result<serde_json::Value> {
        // For now return cached descriptors; recompute=true could be supported later
        match self.inner.resource_instance.as_ref() {
            Some(ri) => serde_json::to_value(&ri.descriptors)
                .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e))),
            None => Ok(serde_json::Value::Null),
        }
    }

    // =========================================================================
    // Populate & tree building
    // =========================================================================

    /// Populate the pseudo cache for the given nodegroups.
    #[napi]
    pub fn populate(
        &self,
        lazy: bool,
        nodegroup_ids: Vec<String>,
        root_node_alias: String,
    ) -> Result<NapiPopulateResult> {
        let result = self
            .inner
            .populate(lazy, &nodegroup_ids, &root_node_alias, &self.model_access)
            .map_err(sc_err)?;

        Ok(NapiPopulateResult {
            values: result.values,
            all_values_map: result.all_values_map,
            all_nodegroups_map: result.all_nodegroups_map,
        })
    }

    /// Ensure a single nodegroup is loaded and return structured values.
    #[napi]
    pub fn ensure_nodegroup(
        &self,
        all_values_js: serde_json::Value,
        all_nodegroups_js: serde_json::Value,
        nodegroup_id: String,
        add_if_missing: bool,
        nodegroup_permissions_js: serde_json::Value,
        do_implied_nodegroups: bool,
    ) -> Result<NapiEnsureNodegroupResult> {
        let all_values: HashMap<String, Option<bool>> = serde_json::from_value(all_values_js)
            .map_err(|e| napi::Error::from_reason(format!("Invalid all_values: {e}")))?;
        let mut all_nodegroups: HashMap<String, bool> =
            serde_json::from_value(all_nodegroups_js)
                .map_err(|e| napi::Error::from_reason(format!("Invalid all_nodegroups: {e}")))?;
        // PermissionRule is currently used only for tile filtering, and may not be
        // JSON-serializable. Use the model's permissions if available.
        let _ = nodegroup_permissions_js;
        let nodegroup_permissions = self.model_access.get_permitted_nodegroups();

        let result = self
            .inner
            .ensure_nodegroup(
                &all_values,
                &mut all_nodegroups,
                &nodegroup_id,
                add_if_missing,
                &nodegroup_permissions,
                do_implied_nodegroups,
                &self.model_access,
            )
            .map_err(sc_err)?;

        Ok(NapiEnsureNodegroupResult {
            values: result.values,
            implied_nodegroups: result.implied_nodegroups,
            all_nodegroups_map: result.all_nodegroups_map,
        })
    }

    /// Build pseudo values from tiles for a specific nodegroup.
    #[napi]
    pub fn values_from_resource_nodegroup(
        &self,
        existing_values_js: serde_json::Value,
        nodegroup_tile_ids: Vec<String>,
        nodegroup_id: String,
    ) -> Result<NapiValuesFromNodegroupResult> {
        let existing_values: HashMap<String, Option<bool>> =
            serde_json::from_value(existing_values_js)
                .map_err(|e| napi::Error::from_reason(format!("Invalid existing_values: {e}")))?;

        let result = self
            .inner
            .values_from_resource_nodegroup(
                &existing_values,
                &nodegroup_tile_ids,
                &nodegroup_id,
                &self.model_access,
            )
            .map_err(sc_err)?;

        Ok(NapiValuesFromNodegroupResult {
            values: result.values,
            implied_nodegroups: result.implied_nodegroups,
        })
    }

    // =========================================================================
    // Path resolution
    // =========================================================================

    /// Resolve a dot-separated path and return a PseudoList.
    #[napi]
    pub fn get_values_at_path(&self, path: String) -> Result<NapiPseudoList> {
        let result = self
            .inner
            .get_values_at_path(&path, &self.model_access, None)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(NapiPseudoList { inner: result })
    }

    // =========================================================================
    // Semantic child traversal
    // =========================================================================

    /// Get semantic child value for a parent/child relationship.
    /// Returns a NapiPseudoList (or null for empty). Singles are wrapped in a
    /// single-element list so the JS `wrapRustPseudo` path works uniformly.
    #[napi]
    pub fn get_semantic_child_value(
        &self,
        parent_tile_id: Option<String>,
        parent_node_id: String,
        parent_nodegroup_id: Option<String>,
        child_alias: String,
    ) -> Result<Option<NapiPseudoList>> {
        let result = self
            .inner
            .get_semantic_child_value(
                parent_tile_id.as_ref(),
                &parent_node_id,
                parent_nodegroup_id.as_ref(),
                &child_alias,
                &self.model_access,
            )
            .map_err(sc_err)?;

        match result {
            SemanticChildResult::List(list) => Ok(Some(NapiPseudoList { inner: list })),
            SemanticChildResult::Single(val) => {
                // Wrap single value in a PseudoListCore so JS sees a uniform interface
                let list = alizarin_core::PseudoListCore {
                    node_alias: val.node.alias.clone().unwrap_or_default(),
                    is_single: true,
                    is_loaded: true,
                    values: vec![val],
                };
                Ok(Some(NapiPseudoList { inner: list }))
            }
            SemanticChildResult::Empty => Ok(None),
        }
    }

    /// Get all semantic child values for a parent node.
    #[napi]
    pub fn get_all_semantic_child_values(
        &self,
        parent_tile_id: Option<String>,
        parent_node_id: String,
        parent_nodegroup_id: Option<String>,
    ) -> Result<serde_json::Value> {
        // Get child nodes
        let child_nodes = self.model_access.get_child_nodes(&parent_node_id);

        let mut results = serde_json::Map::new();
        for child in child_nodes.values() {
            if let Some(ref alias) = child.alias {
                let result = self.inner.get_semantic_child_value(
                    parent_tile_id.as_ref(),
                    &parent_node_id,
                    parent_nodegroup_id.as_ref(),
                    alias,
                    &self.model_access,
                );
                match result {
                    Ok(SemanticChildResult::List(list)) => {
                        results.insert(
                            alias.clone(),
                            serde_json::to_value(NapiPseudoListJson::from(&list))
                                .unwrap_or(serde_json::Value::Null),
                        );
                    }
                    Ok(SemanticChildResult::Single(val)) => {
                        results.insert(
                            alias.clone(),
                            serde_json::to_value(NapiPseudoValueJson::from(&val))
                                .unwrap_or(serde_json::Value::Null),
                        );
                    }
                    Ok(SemanticChildResult::Empty) => {
                        results.insert(alias.clone(), serde_json::Value::Null);
                    }
                    Err(_) => {
                        results.insert(alias.clone(), serde_json::Value::Null);
                    }
                }
            }
        }
        Ok(serde_json::Value::Object(results))
    }

    /// Find all semantic children of a parent node (returns map of alias → tile IDs).
    #[napi]
    pub fn find_semantic_children(
        &self,
        parent_tile_id: Option<String>,
        parent_node_id: String,
        parent_nodegroup_id: Option<String>,
    ) -> Result<serde_json::Value> {
        let child_nodes = self.model_access.get_child_nodes(&parent_node_id);

        let tiles = self
            .inner
            .tiles
            .as_ref()
            .ok_or_else(|| napi::Error::from_reason("Tiles not initialized"))?;

        let mut results = serde_json::Map::new();
        for child in child_nodes.values() {
            if let Some(ref alias) = child.alias {
                let matching_tile_ids: Vec<String> = tiles
                    .iter()
                    .filter(|(_, tile)| {
                        alizarin_core::matches_semantic_child(
                            parent_tile_id.as_ref(),
                            parent_nodegroup_id.as_ref(),
                            child,
                            tile,
                        )
                    })
                    .filter_map(|(_, tile)| tile.tileid.clone())
                    .collect();
                results.insert(
                    alias.clone(),
                    serde_json::to_value(&matching_tile_ids).unwrap(),
                );
            }
        }
        Ok(serde_json::Value::Object(results))
    }

    // =========================================================================
    // Pseudo cache management
    // =========================================================================

    #[napi]
    pub fn get_cached_pseudo(&self, alias: Option<String>) -> Option<NapiPseudoList> {
        let alias = alias?;
        self.inner
            .get_cached_pseudo(&alias)
            .map(|l| NapiPseudoList { inner: l })
    }

    #[napi]
    pub fn get_root_pseudo(&self) -> Option<NapiPseudoValue> {
        let root = self.model_access.get_root_node().ok()?;
        let alias = root.alias.as_deref()?;
        let cached = self.inner.get_cached_pseudo(alias)?;
        cached
            .values
            .first()
            .map(|v| NapiPseudoValue { inner: v.clone() })
    }

    #[napi]
    pub fn cache_pseudo_list(&self, alias: String, list: &NapiPseudoList) {
        self.inner.store_pseudo(alias, list.inner.clone());
    }

    #[napi]
    pub fn cache_pseudo_value(&self, alias: String, value: &NapiPseudoValue) {
        let list = PseudoListCore::from_values_with_cardinality(
            alias.clone(),
            vec![value.inner.clone()],
            true,
        );
        self.inner.store_pseudo(alias, list);
    }

    #[napi]
    pub fn clear_pseudo_cache(&self) {
        if let Ok(mut cache) = self.inner.pseudo_cache.lock() {
            cache.clear();
        }
    }

    /// Construct a PseudoValue from node metadata (for TS wrapper).
    #[napi]
    pub fn make_pseudo_value(
        &self,
        alias: String,
        tile_id: Option<String>,
        is_permitted: bool,
        is_single: bool,
    ) -> Result<serde_json::Value> {
        // Find node by alias
        let nodes = self
            .model_access
            .get_nodes()
            .ok_or_else(|| napi::Error::from_reason("Nodes not initialized"))?;

        let node = nodes
            .values()
            .find(|n| n.alias.as_deref() == Some(&alias))
            .ok_or_else(|| {
                napi::Error::from_reason(format!("Node with alias '{}' not found", alias))
            })?;

        let edges = self
            .model_access
            .get_edges()
            .ok_or_else(|| napi::Error::from_reason("Edges not initialized"))?;
        let child_node_ids = edges.get(&node.nodeid).cloned().unwrap_or_default();

        // Get tile if tile_id provided
        let tile = tile_id
            .as_ref()
            .and_then(|tid| self.inner.get_tile(tid))
            .map(|t| Arc::new(t.clone()));

        let tile_data = tile
            .as_ref()
            .and_then(|t| t.data.get(&node.nodeid).cloned());

        let value =
            PseudoValueCore::from_node_and_tile(Arc::clone(node), tile, tile_data, child_node_ids);

        // Return as JSON with metadata about how to wrap it
        Ok(serde_json::json!({
            "alias": alias,
            "isSingle": is_single,
            "isPermitted": is_permitted,
            "value": NapiPseudoValueJson::from(&value),
        }))
    }

    // =========================================================================
    // JSON serialization
    // =========================================================================

    /// Serialize to JSON (tile_data mode — raw values).
    #[napi]
    pub fn to_json(&self) -> Result<serde_json::Value> {
        self.serialize_with_options(SerializationOptions::tile_data(), None, None, None)
    }

    /// Serialize to display JSON (resolved labels).
    #[napi]
    pub fn to_display_json(
        &self,
        rdm_cache: &NapiRdmCache,
        node_config_manager: &NapiNodeConfigManager,
        language: Option<String>,
    ) -> Result<serde_json::Value> {
        let lang = language.unwrap_or_else(|| "en".to_string());
        self.serialize_with_options(
            SerializationOptions::display(&lang),
            Some(&rdm_cache.inner),
            Some(&node_config_manager.inner),
            None,
        )
    }

    /// Serialize to display JSON without RDM/config (basic labels only).
    #[napi]
    pub fn to_display_json_simple(&self, language: Option<String>) -> Result<serde_json::Value> {
        let lang = language.unwrap_or_else(|| "en".to_string());
        self.serialize_with_options(SerializationOptions::display(&lang), None, None, None)
    }

    // =========================================================================
    // Cleanup
    // =========================================================================

    #[napi]
    pub fn release(&mut self) {
        self.inner.tiles = None;
        self.inner.nodegroup_index.clear();
        self.clear_pseudo_cache();
        if let Ok(mut loaded) = self.inner.loaded_nodegroups.lock() {
            loaded.clear();
        }
    }
}

// =============================================================================
// Private helpers
// =============================================================================

impl NapiResourceInstanceWrapper {
    fn serialize_with_options(
        &self,
        options: SerializationOptions,
        rdm: Option<&RdmCache>,
        ncm: Option<&NodeConfigManager>,
        resource_registry: Option<&alizarin_core::StaticResourceRegistry>,
    ) -> Result<serde_json::Value> {
        let cache = self
            .inner
            .pseudo_cache
            .lock()
            .map_err(|_| napi::Error::from_reason("Failed to lock pseudo cache"))?;

        // Get root alias
        let root_node = self
            .model_access
            .get_root_node()
            .map_err(napi::Error::from_reason)?;
        let root_alias = root_node
            .alias
            .as_deref()
            .ok_or_else(|| napi::Error::from_reason("Root node has no alias"))?;

        let root_list = cache.get(root_alias).ok_or_else(|| {
            napi::Error::from_reason("Root pseudo not found — call populate() first")
        })?;

        // Build graph indices for VisitorContext
        let nodes_by_alias = self
            .model_access
            .get_nodes_by_alias_arc()
            .ok_or_else(|| napi::Error::from_reason("Nodes by alias not built"))?;
        let edges = self
            .model_access
            .get_edges_arc()
            .ok_or_else(|| napi::Error::from_reason("Edges not built"))?;

        let ext_reg = ext_registry();

        let ser_ctx = SerializationContext {
            node_config: None,
            external_resolver: rdm
                .map(|r| r as &dyn alizarin_core::type_serialization::ExternalResolver),
            resource_resolver: resource_registry
                .map(|r| r as &dyn alizarin_core::type_serialization::ResourceDisplayResolver),
            extension_registry: Some(ext_reg),
        };

        let ctx = VisitorContext {
            pseudo_cache: &*cache,
            nodes_by_alias: &nodes_by_alias,
            edges: &edges,
            depth: 0,
            max_depth: 50,
            serialization_options: options,
            serialization_context: ser_ctx,
            node_config_manager: ncm,
        };

        Ok(root_list.to_json(&ctx))
    }
}

// =============================================================================
// JSON serialization helpers for SemanticChildResult
// =============================================================================

#[derive(serde::Serialize)]
struct NapiPseudoValueJson {
    node_id: String,
    alias: Option<String>,
    datatype: String,
    nodegroup_id: Option<String>,
    tile_id: Option<String>,
    tile_data: Option<serde_json::Value>,
    is_collector: bool,
    independent: bool,
    child_node_ids: Vec<String>,
}

impl From<&PseudoValueCore> for NapiPseudoValueJson {
    fn from(v: &PseudoValueCore) -> Self {
        NapiPseudoValueJson {
            node_id: v.node.nodeid.clone(),
            alias: v.node.alias.clone(),
            datatype: v.node.datatype.clone(),
            nodegroup_id: v.node.nodegroup_id.clone(),
            tile_id: v.tile.as_ref().and_then(|t| t.tileid.clone()),
            tile_data: v.tile_data.clone(),
            is_collector: v.is_collector,
            independent: v.independent,
            child_node_ids: v.child_node_ids.clone(),
        }
    }
}

#[derive(serde::Serialize)]
struct NapiPseudoListJson {
    node_alias: String,
    is_single: bool,
    is_loaded: bool,
    values: Vec<NapiPseudoValueJson>,
}

impl From<&PseudoListCore> for NapiPseudoListJson {
    fn from(l: &PseudoListCore) -> Self {
        NapiPseudoListJson {
            node_alias: l.node_alias.clone(),
            is_single: l.is_single,
            is_loaded: l.is_loaded,
            values: l.values.iter().map(NapiPseudoValueJson::from).collect(),
        }
    }
}

// =============================================================================
// NapiResourceModelWrapper
// =============================================================================

/// NAPI equivalent of WASMResourceModelWrapper.
///
/// Provides graph schema access (nodes, edges, nodegroups, permissions, pruning)
/// so that the TS ResourceModelWrapper can delegate to either this or the WASM
/// model wrapper via the backend abstraction.
#[napi]
pub struct NapiResourceModelWrapper {
    model_access: GraphModelAccess,
}

#[napi]
impl NapiResourceModelWrapper {
    /// Create a model wrapper from a graph JSON string.
    ///
    /// The graph is also registered in the core GRAPH_REGISTRY so that
    /// NapiResourceInstanceWrapper can look it up by graph_id.
    #[napi(constructor)]
    pub fn new(graph_json: String, default_allow: bool) -> Result<Self> {
        let mut graph: StaticGraph = serde_json::from_str(&graph_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid graph JSON: {}", e)))?;

        graph.build_indices();
        let graph_id = graph.graphid.clone();
        let graph_arc = Arc::new(graph);

        // Register in core graph registry
        alizarin_core::register_graph(&graph_id, Arc::clone(&graph_arc));

        let model_access = GraphModelAccess::new_eager(graph_arc, default_allow);

        Ok(NapiResourceModelWrapper { model_access })
    }

    /// Create from an already-loaded NapiStaticGraph.
    #[napi(factory)]
    pub fn from_graph(graph: &crate::NapiStaticGraph, default_allow: bool) -> Result<Self> {
        let inner = graph.inner_ref();
        let graph_id = inner.graphid.clone();
        let mut cloned = inner.clone();
        cloned.build_indices();
        let graph_arc = Arc::new(cloned);

        alizarin_core::register_graph(&graph_id, Arc::clone(&graph_arc));

        let model_access = GraphModelAccess::new_eager(graph_arc, default_allow);

        Ok(NapiResourceModelWrapper { model_access })
    }

    // =========================================================================
    // Graph ID
    // =========================================================================

    #[napi(js_name = "getGraphId")]
    pub fn get_graph_id(&self) -> String {
        self.model_access.get_graph().graphid.clone()
    }

    // =========================================================================
    // Graph getter/setter
    // =========================================================================

    #[napi(getter)]
    pub fn graph(&self) -> Result<serde_json::Value> {
        serde_json::to_value(self.model_access.get_graph())
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    #[napi(setter)]
    pub fn set_graph(&mut self, graph_json: String) -> Result<()> {
        let graph: StaticGraph = serde_json::from_str(&graph_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid graph JSON: {}", e)))?;
        self.model_access.rebuild_from_graph(&graph);
        Ok(())
    }

    // =========================================================================
    // Build nodes (no-op — caches are built eagerly in constructor)
    // =========================================================================

    #[napi(js_name = "buildNodes")]
    pub fn build_nodes(&self) -> Result<()> {
        // GraphModelAccess builds caches in from_graph(), so this is a no-op.
        Ok(())
    }

    #[napi(js_name = "buildNodesForGraph")]
    pub fn build_nodes_for_graph(&mut self, graph_json: String) -> Result<()> {
        let graph: StaticGraph = serde_json::from_str(&graph_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid graph JSON: {}", e)))?;
        self.model_access.rebuild_from_graph(&graph);
        Ok(())
    }

    // =========================================================================
    // Node accessors
    // =========================================================================

    /// Get the root node of the graph.
    #[napi(js_name = "getRootNode")]
    pub fn get_root_node(&self) -> Result<serde_json::Value> {
        let root = self
            .model_access
            .get_root_node()
            .map_err(napi::Error::from_reason)?;
        serde_json::to_value(&*root).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Get all nodes as a Map<string, StaticNode>.
    #[napi(js_name = "getNodeObjects")]
    pub fn get_node_objects(&self) -> Result<HashMap<String, serde_json::Value>> {
        let nodes = self
            .model_access
            .get_nodes()
            .ok_or_else(|| napi::Error::from_reason("Nodes not available"))?;
        let mut result = HashMap::new();
        for (k, v) in nodes {
            result.insert(
                k.clone(),
                serde_json::to_value(&**v).map_err(|e| napi::Error::from_reason(e.to_string()))?,
            );
        }
        Ok(result)
    }

    /// Get all nodes indexed by alias.
    #[napi(js_name = "getNodeObjectsByAlias")]
    pub fn get_node_objects_by_alias(&self) -> Result<HashMap<String, serde_json::Value>> {
        let nodes = self
            .model_access
            .get_nodes_by_alias()
            .ok_or_else(|| napi::Error::from_reason("Nodes by alias not available"))?;
        let mut result = HashMap::new();
        for (k, v) in nodes {
            result.insert(
                k.clone(),
                serde_json::to_value(&**v).map_err(|e| napi::Error::from_reason(e.to_string()))?,
            );
        }
        Ok(result)
    }

    /// Get a single node by alias.
    #[napi(js_name = "getNodeObjectFromAlias")]
    pub fn get_node_object_from_alias(&self, alias: String) -> Result<serde_json::Value> {
        let nodes = self
            .model_access
            .get_nodes_by_alias()
            .ok_or_else(|| napi::Error::from_reason("Nodes by alias not available"))?;
        let node = nodes.get(&alias).ok_or_else(|| {
            napi::Error::from_reason(format!("Node not found for alias: {}", alias))
        })?;
        serde_json::to_value(&**node).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Get a single node by node ID.
    #[napi(js_name = "getNodeObjectFromId")]
    pub fn get_node_object_from_id(&self, id: String) -> Result<serde_json::Value> {
        let nodes = self
            .model_access
            .get_nodes()
            .ok_or_else(|| napi::Error::from_reason("Nodes not available"))?;
        let node = nodes
            .get(&id)
            .ok_or_else(|| napi::Error::from_reason(format!("Node not found: {}", id)))?;
        serde_json::to_value(&**node).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Get child nodes for a parent node ID.
    #[napi(js_name = "getChildNodes")]
    pub fn get_child_nodes(&self, node_id: String) -> Result<HashMap<String, serde_json::Value>> {
        let children = self.model_access.get_child_nodes(&node_id);
        let mut result = HashMap::new();
        for (alias, node) in &children {
            result.insert(
                alias.clone(),
                serde_json::to_value(&**node)
                    .map_err(|e| napi::Error::from_reason(e.to_string()))?,
            );
        }
        Ok(result)
    }

    /// Get child node aliases (just strings, not full objects).
    #[napi(js_name = "getChildNodeAliases")]
    pub fn get_child_node_aliases(&self, node_id: String) -> Vec<String> {
        self.model_access
            .get_child_nodes(&node_id)
            .keys()
            .cloned()
            .collect()
    }

    // =========================================================================
    // Edge accessors
    // =========================================================================

    /// Get edges as Map<parent_id, [child_ids]>.
    #[napi(js_name = "getEdges")]
    pub fn get_edges(&self) -> Result<HashMap<String, Vec<String>>> {
        let edges = self
            .model_access
            .get_edges()
            .ok_or_else(|| napi::Error::from_reason("Edges not available"))?;
        Ok(edges.clone())
    }

    // =========================================================================
    // Nodegroup accessors
    // =========================================================================

    /// Get all nodegroups.
    #[napi(js_name = "getNodegroupObjects")]
    pub fn get_nodegroup_objects(&self) -> Result<HashMap<String, serde_json::Value>> {
        let nodegroups = self
            .model_access
            .get_nodegroups()
            .ok_or_else(|| napi::Error::from_reason("Nodegroups not available"))?;
        let mut result = HashMap::new();
        for (k, v) in nodegroups {
            result.insert(
                k.clone(),
                serde_json::to_value(&**v).map_err(|e| napi::Error::from_reason(e.to_string()))?,
            );
        }
        Ok(result)
    }

    /// Get nodegroup IDs.
    #[napi(js_name = "getNodegroupIds")]
    pub fn get_nodegroup_ids(&self) -> Vec<String> {
        self.model_access
            .get_nodegroups()
            .map(|ng| ng.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get nodegroup name (actually the name of the root node for that nodegroup).
    #[napi(js_name = "getNodegroupName")]
    pub fn get_nodegroup_name(&self, nodegroup_id: String) -> Result<String> {
        let nodes = self
            .model_access
            .get_nodes()
            .ok_or_else(|| napi::Error::from_reason("Nodes not available"))?;
        let node = nodes
            .get(&nodegroup_id)
            .ok_or_else(|| napi::Error::from_reason(format!("Node not found: {}", nodegroup_id)))?;
        Ok(node.name.clone())
    }

    /// Get node ID from alias.
    #[napi(js_name = "getNodeIdFromAlias")]
    pub fn get_node_id_from_alias(&self, alias: String) -> Result<String> {
        let nodes = self
            .model_access
            .get_nodes_by_alias()
            .ok_or_else(|| napi::Error::from_reason("Nodes by alias not available"))?;
        let node = nodes.get(&alias).ok_or_else(|| {
            napi::Error::from_reason(format!("Node not found for alias: {}", alias))
        })?;
        Ok(node.nodeid.clone())
    }

    // =========================================================================
    // Property getters (matching WASM's getter properties)
    // =========================================================================

    #[napi(getter)]
    pub fn nodes(&self) -> Result<HashMap<String, serde_json::Value>> {
        self.get_node_objects()
    }

    #[napi(getter, js_name = "nodesByAlias")]
    pub fn nodes_by_alias(&self) -> Result<HashMap<String, serde_json::Value>> {
        self.get_node_objects_by_alias()
    }

    #[napi(getter)]
    pub fn edges(&self) -> Result<HashMap<String, Vec<String>>> {
        self.get_edges()
    }

    #[napi(getter)]
    pub fn nodegroups(&self) -> Result<HashMap<String, serde_json::Value>> {
        self.get_nodegroup_objects()
    }

    // =========================================================================
    // Permission management
    // =========================================================================

    /// Get permitted nodegroups as boolean map.
    #[napi(js_name = "getPermittedNodegroups")]
    pub fn get_permitted_nodegroups(&self) -> HashMap<String, bool> {
        self.model_access.get_permitted_nodegroups_bool()
    }

    /// Check if a nodegroup is permitted.
    #[napi(js_name = "isNodegroupPermitted")]
    pub fn is_nodegroup_permitted(&self, nodegroup_id: String) -> bool {
        self.model_access.is_nodegroup_permitted(&nodegroup_id)
    }

    /// Set permitted nodegroups. Accepts an object with boolean values or
    /// conditional rule objects { path, allowed }.
    #[napi(js_name = "setPermittedNodegroups")]
    pub fn set_permitted_nodegroups(&mut self, permissions: serde_json::Value) -> Result<()> {
        let perms = Self::parse_permission_value(&permissions)?;
        self.model_access.set_permitted_nodegroups_rules(perms);
        Ok(())
    }

    #[napi(js_name = "setDefaultAllowAllNodegroups")]
    pub fn set_default_allow_all_nodegroups(&mut self, default_allow: bool) {
        self.model_access.set_default_allow(default_allow);
    }

    // =========================================================================
    // Graph modification
    // =========================================================================

    #[napi(js_name = "setGraphNodes")]
    pub fn set_graph_nodes(&mut self, nodes_json: String) -> Result<()> {
        let nodes: Vec<StaticNode> = serde_json::from_str(&nodes_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid nodes JSON: {}", e)))?;
        self.model_access.set_graph_nodes(nodes);
        Ok(())
    }

    #[napi(js_name = "setGraphEdges")]
    pub fn set_graph_edges(&mut self, edges_json: String) -> Result<()> {
        let edges: Vec<StaticEdge> = serde_json::from_str(&edges_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid edges JSON: {}", e)))?;
        self.model_access.set_graph_edges(edges);
        Ok(())
    }

    #[napi(js_name = "setGraphNodegroups")]
    pub fn set_graph_nodegroups(&mut self, nodegroups_json: String) -> Result<()> {
        let nodegroups: Vec<StaticNodegroup> = serde_json::from_str(&nodegroups_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid nodegroups JSON: {}", e)))?;
        self.model_access.set_graph_nodegroups(nodegroups);
        Ok(())
    }

    // =========================================================================
    // Graph pruning
    // =========================================================================

    /// Prune graph to only include permitted nodegroups and their dependencies.
    #[napi(js_name = "pruneGraph")]
    pub fn prune_graph(&mut self, keep_functions: Option<Vec<String>>) -> Result<()> {
        let keep_fns_ref = keep_functions.as_deref();
        self.model_access
            .prune_graph(keep_fns_ref)
            .map_err(napi::Error::from_reason)
    }
}

// Private helpers for NapiResourceModelWrapper
impl NapiResourceModelWrapper {
    /// Parse permission values from a JSON object.
    /// Supports booleans and conditional rules: { path: "...", allowed: [...] }
    fn parse_permission_value(val: &serde_json::Value) -> Result<HashMap<String, PermissionRule>> {
        let obj = val
            .as_object()
            .ok_or_else(|| napi::Error::from_reason("Permissions must be an object"))?;
        let mut perms = HashMap::new();
        for (key, value) in obj {
            if let Some(b) = value.as_bool() {
                perms.insert(key.clone(), PermissionRule::Boolean(b));
            } else if let Some(obj) = value.as_object() {
                let path = obj
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_default();
                let allowed = obj
                    .get("allowed")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<HashSet<String>>()
                    })
                    .unwrap_or_default();
                let rule = PermissionRule::conditional(path, allowed).map_err(|e| {
                    napi::Error::from_reason(format!(
                        "Invalid conditional rule for '{}': {}",
                        key, e
                    ))
                })?;
                perms.insert(key.clone(), rule);
            } else {
                return Err(napi::Error::from_reason(format!(
                    "Invalid permission value for '{}': expected boolean or {{path, allowed}}",
                    key
                )));
            }
        }
        Ok(perms)
    }
}
