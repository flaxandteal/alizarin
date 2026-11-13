/// NAPI bindings for ResourceInstanceWrapperCore
///
/// Provides the same public interface as WASMResourceInstanceWrapper so that
/// the TypeScript ViewModel layer (ResourceInstanceWrapper / PseudoValue / PseudoList)
/// can work with either backend.
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi_derive::napi;

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

fn ext_registry() -> ExtensionTypeRegistry {
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
    pub(crate) inner: RdmCache,
}

impl NapiRdmCache {
    /// Crate-internal accessor (field not directly accessible through #[napi] macro).
    pub(crate) fn inner(&self) -> &RdmCache {
        &self.inner
    }
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

    /// Check if a collection is already in the cache.
    #[napi]
    pub fn has_collection(&self, collection_id: String) -> bool {
        self.inner.has_collection(&collection_id)
    }

    /// Add a collection from a JSON string of concepts.
    #[napi]
    pub fn add_collection_from_json(
        &mut self,
        collection_id: String,
        json_str: String,
    ) -> Result<()> {
        self.inner
            .add_collection_from_json(&collection_id, &json_str)
            .map_err(|e| napi::Error::from_reason(format!("Failed to add collection: {e}")))
    }

    /// Get the parent concept ID for a concept within a collection.
    #[napi]
    pub fn get_parent_id(&self, collection_id: String, concept_id: String) -> Option<String> {
        self.inner.get_parent_id(&collection_id, &concept_id)
    }

    /// Remove a collection from the cache.
    #[napi]
    pub fn remove_collection(&mut self, collection_id: String) -> bool {
        self.inner.remove_collection(&collection_id)
    }

    /// Clear all collections from the cache.
    #[napi]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Resolve labels to UUIDs using this cache for lookups.
    #[napi]
    pub fn resolve_labels(
        &self,
        tree_json: String,
        alias_to_collection: HashMap<String, String>,
        strict: bool,
    ) -> Result<serde_json::Value> {
        let tree: serde_json::Value = serde_json::from_str(&tree_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid tree JSON: {e}")))?;

        let resolved = alizarin_core::label_resolution::resolve_labels(
            tree,
            &alias_to_collection,
            &self.inner,
            strict,
        )
        .map_err(|e| napi::Error::from_reason(e.message))?;

        Ok(resolved)
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

    // The readers below mirror `WasmNodeConfigManager` name for name (and its
    // null-id handling), so the shared TypeScript `NodeConfigManager` works on
    // both backends.

    /// Boolean config for a node, or null.
    #[napi(js_name = "getBoolean")]
    pub fn get_boolean(
        &self,
        nodeid: Option<String>,
    ) -> Option<crate::node_config_napi::NapiNodeConfigBoolean> {
        self.inner.get_boolean(&nodeid?).map(Into::into)
    }

    /// Concept config for a node, or null.
    #[napi(js_name = "getConcept")]
    pub fn get_concept(
        &self,
        nodeid: Option<String>,
    ) -> Option<crate::node_config_napi::NapiNodeConfigConcept> {
        self.inner.get_concept(&nodeid?).map(Into::into)
    }

    /// An extension datatype's node config, as its registered handler supplies
    /// it (a plain object; its shape is the extension's), or null if the node
    /// has none or no handler for its datatype is registered.
    #[napi(js_name = "getExtensionConfig")]
    pub fn get_extension_config(&self, nodeid: Option<String>) -> Option<serde_json::Value> {
        let registry = alizarin_core::build_extension_registry_from_registered();
        self.inner.get_extension_config(&nodeid?, Some(&registry))
    }

    /// Domain config for a node, or null.
    #[napi(js_name = "getDomain")]
    pub fn get_domain(
        &self,
        nodeid: Option<String>,
    ) -> Option<crate::node_config_napi::NapiNodeConfigDomain> {
        self.inner.get_domain(&nodeid?).map(Into::into)
    }

    /// A domain value by id, or null.
    #[napi(js_name = "lookupDomainValue")]
    pub fn lookup_domain_value(
        &self,
        nodeid: Option<String>,
        value_id: Option<String>,
    ) -> Option<crate::node_config_napi::NapiStaticDomainValue> {
        self.inner
            .lookup_domain_value(&nodeid?, &value_id?)
            .map(Into::into)
    }

    /// Whether a node has config (false for a null id).
    #[napi(js_name = "hasConfig")]
    pub fn has_config(&self, nodeid: Option<String>) -> bool {
        nodeid.is_some_and(|id| self.inner.has_config(&id))
    }

    /// The config type for a node, or null.
    #[napi(js_name = "getConfigType")]
    pub fn get_config_type(&self, nodeid: Option<String>) -> Option<String> {
        self.inner.get_config_type(&nodeid?).map(str::to_string)
    }

    /// Clear all cached configs.
    #[napi]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Number of cached configs.
    #[napi(getter)]
    pub fn length(&self) -> u32 {
        self.inner.len() as u32
    }
}

// =============================================================================
// NapiTileData — Map-like interface over a tile's BTreeMap<String, Value>
// =============================================================================

#[napi]
pub struct NapiTileData {
    entries: Arc<Mutex<BTreeMap<String, serde_json::Value>>>,
}

#[napi]
impl NapiTileData {
    #[napi]
    pub fn has(&self, key: String) -> bool {
        self.entries.lock().unwrap().contains_key(&key)
    }

    #[napi]
    pub fn get(&self, key: String) -> serde_json::Value {
        self.entries
            .lock()
            .unwrap()
            .get(&key)
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    }

    #[napi]
    pub fn set(&mut self, key: String, value: serde_json::Value) {
        self.entries.lock().unwrap().insert(key, value);
    }

    #[napi]
    pub fn delete(&mut self, key: String) -> bool {
        self.entries.lock().unwrap().remove(&key).is_some()
    }

    /// Keys in sorted order, matching the WASM `data` Map and serialized tiles.
    #[napi]
    pub fn keys(&self) -> Vec<String> {
        self.entries.lock().unwrap().keys().cloned().collect()
    }
}

// =============================================================================
// NapiStaticTile — Rust-owned tile with Map-like .data access
// =============================================================================

#[napi]
pub struct NapiStaticTile {
    tileid: Option<String>,
    nodegroup_id: String,
    sortorder: Option<i32>,
    resourceinstance_id: String,
    parenttile_id: Option<String>,
    provisionaledits: Option<Vec<serde_json::Value>>,
    data_store: Arc<Mutex<BTreeMap<String, serde_json::Value>>>,
}

#[napi]
impl NapiStaticTile {
    #[napi(constructor)]
    pub fn new(
        nodegroup_id: String,
        tileid: Option<String>,
        sortorder: Option<i32>,
        resourceinstance_id: Option<String>,
        parenttile_id: Option<String>,
    ) -> Self {
        NapiStaticTile {
            tileid,
            nodegroup_id,
            sortorder,
            resourceinstance_id: resourceinstance_id.unwrap_or_default(),
            parenttile_id,
            provisionaledits: None,
            data_store: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    #[napi(getter)]
    pub fn data(&self) -> NapiTileData {
        NapiTileData {
            entries: self.data_store.clone(),
        }
    }

    /// Setter is a no-op — data is always accessed via the NapiTileData getter.
    /// Exists to prevent TypeError in strict mode when JS does
    /// `tile.data = tile.data || new Map()`.
    #[napi(setter)]
    pub fn set_data(&mut self, _value: serde_json::Value) {
        // No-op: mutations go through NapiTileData.set/delete
    }

    #[napi(getter)]
    pub fn tileid(&self) -> Option<String> {
        self.tileid.clone()
    }

    #[napi(setter)]
    pub fn set_tileid(&mut self, value: Option<String>) {
        self.tileid = value;
    }

    #[napi(getter)]
    pub fn nodegroup_id(&self) -> String {
        self.nodegroup_id.clone()
    }

    #[napi(getter)]
    pub fn sortorder(&self) -> Option<i32> {
        self.sortorder
    }

    #[napi(getter)]
    pub fn resourceinstance_id(&self) -> String {
        self.resourceinstance_id.clone()
    }

    #[napi(getter)]
    pub fn parenttile_id(&self) -> Option<String> {
        self.parenttile_id.clone()
    }

    #[napi(setter)]
    pub fn set_parenttile_id(&mut self, value: Option<String>) {
        self.parenttile_id = value;
    }

    #[napi(getter)]
    pub fn provisionaledits(&self) -> serde_json::Value {
        match &self.provisionaledits {
            Some(edits) => serde_json::to_value(edits).unwrap_or(serde_json::Value::Null),
            None => serde_json::Value::Null,
        }
    }

    /// Generate a tile ID if not already set.
    #[napi(js_name = "ensureId")]
    pub fn ensure_id(&mut self) -> String {
        if self.tileid.is_none() {
            self.tileid = Some(uuid::Uuid::new_v4().to_string());
        }
        self.tileid.clone().unwrap()
    }
}

impl NapiStaticTile {
    /// Convert to core StaticTile for storage in PseudoValueCore.
    pub(crate) fn to_static_tile(&self) -> StaticTile {
        let data = self.data_store.lock().unwrap().clone();
        StaticTile {
            tileid: self.tileid.clone(),
            nodegroup_id: self.nodegroup_id.clone(),
            sortorder: self.sortorder,
            resourceinstance_id: self.resourceinstance_id.clone(),
            parenttile_id: self.parenttile_id.clone(),
            provisionaledits: self.provisionaledits.clone(),
            data,
        }
    }

    /// Create from a core StaticTile.
    pub(crate) fn from_static_tile(tile: &StaticTile) -> Self {
        NapiStaticTile {
            tileid: tile.tileid.clone(),
            nodegroup_id: tile.nodegroup_id.clone(),
            sortorder: tile.sortorder,
            resourceinstance_id: tile.resourceinstance_id.clone(),
            parenttile_id: tile.parenttile_id.clone(),
            provisionaledits: tile.provisionaledits.clone(),
            data_store: Arc::new(Mutex::new(tile.data.clone())),
        }
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
            "name": &self.inner.node.name,
            "alias": self.inner.node.alias,
            "datatype": &self.inner.node.datatype,
            "nodegroupId": self.inner.node.nodegroup_id,
            "sortorder": self.inner.node.sortorder,
            "isOuter": self.inner.is_outer(),
            "isInner": self.inner.is_inner,
            "isCollector": self.inner.is_collector,
            "accessed": false,
            "independent": self.inner.independent,
            "hasTile": self.inner.tile.is_some(),
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

    // -- Tile getter/setter (NapiStaticTile, Rust-owned) --

    #[napi(getter, js_name = "tile")]
    pub fn get_tile(&self) -> Option<NapiStaticTile> {
        self.inner
            .tile
            .as_ref()
            .map(|t| NapiStaticTile::from_static_tile(t.as_ref()))
    }

    #[napi(setter, js_name = "tile")]
    pub fn set_tile(&mut self, tile: Option<&NapiStaticTile>) {
        match tile {
            Some(t) => {
                self.inner.tile = Some(Arc::new(t.to_static_tile()));
            }
            None => {
                self.inner.tile = None;
            }
        }
    }

    // -- Inner (outer/inner pattern) --

    /// Get the inner PseudoValueCore wrapped as NapiPseudoValue.
    #[napi(getter)]
    pub fn inner(&self) -> Option<NapiPseudoValue> {
        self.inner
            .inner
            .as_ref()
            .map(|i| NapiPseudoValue { inner: *i.clone() })
    }

    /// Returns null — value is managed JS-side via _cachedValue.
    #[napi(getter)]
    pub fn value(&self) -> serde_json::Value {
        serde_json::Value::Null
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
        alizarin_core::sorted_serde::sorted_keys(self.values.keys())
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
        alizarin_core::sorted_serde::sorted_keys(self.values.keys())
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

/// The shared model core holds for `graph_id`, registered by its model wrapper.
fn model_for(graph_id: &str) -> Result<Arc<GraphModelAccess>> {
    alizarin_core::get_model(graph_id).ok_or_else(|| {
        napi::Error::from_reason(format!(
            "No model registered for graph '{graph_id}'. Create its model wrapper first."
        ))
    })
}

/// A resource instance. It holds no model of its own: it looks up its graph's
/// shared model on each use, so it sees permission changes and prunes made
/// after it was created.
#[napi]
pub struct NapiResourceInstanceWrapper {
    inner: ResourceInstanceWrapperCore,
}

#[napi]
impl NapiResourceInstanceWrapper {
    // =========================================================================
    // Construction
    // =========================================================================

    /// Create a wrapper for a given graph (must be registered).
    ///
    /// If `resource_id` is provided, a minimal resource metadata is created so
    /// the tile-source fast path can look up tiles by resource.
    #[napi(constructor)]
    pub fn new(graph_id: String, resource_id: Option<String>) -> Result<Self> {
        // Its graph's model must be registered; the instance builds nothing.
        model_for(&graph_id)?;

        let mut core = ResourceInstanceWrapperCore::new(graph_id.clone());

        // Set minimal resource metadata so tile source can resolve this instance.
        if let Some(rid) = resource_id {
            use alizarin_core::graph::{StaticResourceDescriptors, StaticResourceMetadata};
            core.resource_instance = Some(StaticResourceMetadata {
                resourceinstanceid: rid,
                graph_id,
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

        Ok(NapiResourceInstanceWrapper { inner: core })
    }

    // =========================================================================
    // Tile loading
    // =========================================================================

    /// Load tiles from a JSON string (single-pass deserialization). Unless
    /// `assumeTilesComprehensiveForNodegroup` is false, they are the resource's
    /// complete set, so every nodegroup counts as loaded (as on WASM).
    #[napi]
    pub fn load_tiles(
        &mut self,
        tiles_json: String,
        assume_tiles_comprehensive_for_nodegroup: Option<bool>,
    ) -> Result<()> {
        let tiles: Vec<StaticTile> = serde_json::from_str(&tiles_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid tiles JSON: {e}")))?;
        let model = self.model()?;
        self.inner.load_permitted_tiles(tiles, &model);
        if assume_tiles_comprehensive_for_nodegroup.unwrap_or(true) {
            self.inner.mark_all_nodegroups_loaded(&*model);
        }
        Ok(())
    }

    /// Load tiles directly from a StaticResource JSON string (single-pass
    /// deserialization); comprehensive unless told otherwise, as `loadTiles`.
    #[napi]
    pub fn load_tiles_from_resource(
        &mut self,
        resource_json: String,
        assume_tiles_comprehensive_for_nodegroup: Option<bool>,
    ) -> Result<()> {
        let resource: StaticResource = serde_json::from_str(&resource_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid resource JSON: {e}")))?;

        let model = self.model()?;
        self.inner.resource_instance = Some(resource.resourceinstance.clone());

        if let Some(tiles_vec) = resource.tiles {
            self.inner.load_permitted_tiles(tiles_vec, &model);
            if assume_tiles_comprehensive_for_nodegroup.unwrap_or(true) {
                self.inner.mark_all_nodegroups_loaded(&*model);
            }
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
        assume_tiles_comprehensive_for_nodegroup: Option<bool>,
    ) -> Result<bool> {
        let resource = match registry.inner.get_full(&resource_id) {
            Some(r) => r,
            None => return Ok(false),
        };

        let model = self.model()?;
        self.inner.resource_instance = Some(resource.resourceinstance.clone());

        if let Some(tiles_vec) = &resource.tiles {
            self.inner
                .load_permitted_tiles(tiles_vec.iter().cloned(), &model);
            if assume_tiles_comprehensive_for_nodegroup.unwrap_or(true) {
                self.inner.mark_all_nodegroups_loaded(&*model);
            }
        }
        Ok(true)
    }

    /// Append tiles incrementally (for lazy loading); comprehensive unless
    /// told otherwise, as WASM's `appendTiles` is.
    #[napi]
    pub fn append_tiles(
        &mut self,
        tiles_js: serde_json::Value,
        assume_tiles_comprehensive_for_nodegroup: Option<bool>,
    ) -> Result<()> {
        let tiles: Vec<StaticTile> = serde_json::from_value(tiles_js)
            .map_err(|e| napi::Error::from_reason(format!("Invalid tiles JSON: {e}")))?;
        let model = self.model()?;
        self.inner.merge_permitted_tiles(tiles, &model);
        if assume_tiles_comprehensive_for_nodegroup.unwrap_or(true) {
            self.inner.mark_all_nodegroups_loaded(&*model);
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
        let model = self.model()?;
        let tiles = self
            .inner
            .tiles
            .take()
            .ok_or_else(|| napi::Error::from_reason("Tiles not initialized".to_string()))?;
        let pruned: alizarin_core::TileStore = tiles
            .into_iter()
            .filter(|(_id, tile)| model.is_tile_permitted(tile))
            .collect();
        self.inner.tiles = Some(pruned);
        Ok(())
    }

    #[napi]
    pub fn tiles_loaded(&self) -> bool {
        self.inner
            .tiles
            .as_ref()
            .map(|t| !self.inner.is_lazy() || !t.is_empty())
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
        if self.inner.is_lazy() {
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
        self.inner.set_lazy(lazy);
        if lazy && self.inner.tiles.is_none() {
            self.inner.tiles = Some(alizarin_core::TileStore::new());
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
                if let Ok(loaded) = self.inner.loaded_nodegroups.read() {
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
        if let Ok(mut loaded) = self.inner.loaded_nodegroups.write() {
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
        let Ok(model) = self.model() else {
            return Vec::new();
        };
        self.inner
            .missing_nodegroups_for_children(&parent_node_id, &*model, self.inner.is_lazy())
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
    pub fn get_name(&self) -> String {
        self.inner.display_name()
    }

    /// The cached descriptors, built from the tiles (and cached) if `recompute`
    /// or none are cached: core's, as WASM uses.
    #[napi]
    pub fn get_descriptors(&mut self, recompute: bool) -> Result<serde_json::Value> {
        let graph_id = self.inner.graph_id.clone();
        let descriptors = self
            .inner
            .descriptors(recompute, || {
                let graph = model_for(&graph_id).map_err(|e| e.reason)?.get_graph_arc();
                Ok((graph, ext_registry()))
            })
            .map_err(napi::Error::from_reason)?;
        serde_json::to_value(&descriptors)
            .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e)))
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
        let model = self.model()?;
        let result = self
            .inner
            .populate(lazy, &nodegroup_ids, &root_node_alias, &*model)
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
        let model = self.model()?;
        let nodegroup_permissions = model.get_permitted_nodegroups();

        let result = self
            .inner
            .ensure_nodegroup(
                &all_values,
                &mut all_nodegroups,
                &nodegroup_id,
                add_if_missing,
                &nodegroup_permissions,
                do_implied_nodegroups,
                &*model,
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
        let model = self.model()?;
        let existing_values: HashMap<String, Option<bool>> =
            serde_json::from_value(existing_values_js)
                .map_err(|e| napi::Error::from_reason(format!("Invalid existing_values: {e}")))?;

        let result = self
            .inner
            .values_from_resource_nodegroup(
                &existing_values,
                &nodegroup_tile_ids,
                &nodegroup_id,
                &*model,
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

    /// Resolve a dot-separated path to its target node metadata without needing tiles.
    ///
    /// Returns { nodegroupId, isSingle, targetNodeId } — enough for the JS layer to
    /// lazy-load just that nodegroup's tiles before calling getValuesAtPath.
    #[napi(js_name = "resolvePath")]
    pub fn resolve_path(&self, path: String) -> Result<serde_json::Value> {
        let model = self.model()?;
        let info = self
            .inner
            .resolve_path(&path, &*model)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(serde_json::json!({
            "nodegroupId": info.nodegroup_id,
            "isSingle": info.is_single,
            "targetNodeId": info.target_node.nodeid,
        }))
    }

    /// Resolve a dot-separated path and return a PseudoList.
    #[napi]
    pub fn get_values_at_path(
        &self,
        path: String,
        filter_tile_id: Option<String>,
    ) -> Result<NapiPseudoList> {
        let model = self.model()?;
        let result = self
            .inner
            .get_values_at_path(&path, &*model, filter_tile_id.as_deref())
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
        let model = self.model()?;
        let result = self
            .inner
            .get_semantic_child_value(
                parent_tile_id.as_ref(),
                &parent_node_id,
                parent_nodegroup_id.as_ref(),
                &child_alias,
                &*model,
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

    /// Get all semantic child values for a parent node, by alias (`null` for
    /// none). Errors propagate: in lazy mode, `TILES_NOT_LOADED:<nodegroup>`.
    #[napi]
    pub fn get_all_semantic_child_values(
        &self,
        parent_tile_id: Option<String>,
        parent_node_id: String,
        parent_nodegroup_id: Option<String>,
    ) -> Result<serde_json::Value> {
        let model = self.model()?;
        let values = self
            .inner
            .get_all_semantic_child_values(
                parent_tile_id.as_ref(),
                &parent_node_id,
                parent_nodegroup_id.as_ref(),
                &*model,
            )
            .map_err(sc_err)?;
        let results = values
            .into_iter()
            .map(|(alias, value)| {
                let json = match value {
                    SemanticChildResult::List(list) => {
                        serde_json::to_value(NapiPseudoListJson::from(&list))
                    }
                    SemanticChildResult::Single(val) => {
                        serde_json::to_value(NapiPseudoValueJson::from(&val))
                    }
                    SemanticChildResult::Empty => Ok(serde_json::Value::Null),
                }
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;
                Ok((alias, json))
            })
            .collect::<Result<serde_json::Map<_, _>>>()?;
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
        let model = self.model()?;
        let children = self
            .inner
            .find_semantic_children(
                parent_tile_id.as_ref(),
                &parent_node_id,
                parent_nodegroup_id.as_ref(),
                &*model,
            )
            .map_err(sc_err)?;
        serde_json::to_value(children).map_err(|e| napi::Error::from_reason(e.to_string()))
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
        let model = self.model().ok()?;
        let root = model.get_root_node().ok()?;
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
        if let Ok(mut cache) = self.inner.pseudo_cache.write() {
            cache.clear();
        }
    }

    /// Construct the pseudo for `alias` (for the TS wrapper): core's, as WASM
    /// uses — a list, a value, or null (TS: PseudoUnavailable) when not
    /// permitted.
    #[napi]
    pub fn make_pseudo_value(
        &self,
        alias: String,
        tile_id: Option<String>,
        is_permitted: bool,
        is_single: bool,
    ) -> Result<Option<Either<NapiPseudoList, NapiPseudoValue>>> {
        let model = self.model()?;
        let result = self
            .inner
            .make_pseudo_value(&alias, tile_id.as_deref(), is_permitted, is_single, &*model)
            .map_err(napi::Error::from_reason)?;
        Ok(match result {
            SemanticChildResult::List(list) => Some(Either::A(NapiPseudoList { inner: list })),
            SemanticChildResult::Single(value) => Some(Either::B(NapiPseudoValue { inner: value })),
            SemanticChildResult::Empty => None,
        })
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
        resource_registry: Option<&crate::NapiStaticResourceRegistry>,
    ) -> Result<serde_json::Value> {
        let lang = language.unwrap_or_else(|| "en".to_string());
        self.serialize_with_options(
            SerializationOptions::display(&lang),
            Some(&rdm_cache.inner),
            Some(&node_config_manager.inner),
            resource_registry.map(|r| &r.inner),
        )
    }

    /// Serialize to display JSON without RDM/config (basic labels only).
    #[napi]
    pub fn to_display_json_simple(&self, language: Option<String>) -> Result<serde_json::Value> {
        let lang = language.unwrap_or_else(|| "en".to_string());
        self.serialize_with_options(SerializationOptions::display(&lang), None, None, None)
    }

    /// Populate (if needed) and serialize to display JSON in a single synchronous call.
    ///
    /// Combines `populate()` + `toDisplayJson()` to avoid the async microtask
    /// overhead of calling them separately from JS.
    #[napi]
    pub fn to_display_json_full(
        &self,
        rdm_cache: &NapiRdmCache,
        node_config_manager: &NapiNodeConfigManager,
        language: Option<String>,
        resource_registry: Option<&crate::NapiStaticResourceRegistry>,
    ) -> Result<serde_json::Value> {
        self.ensure_populated()?;
        let lang = language.unwrap_or_else(|| "en".to_string());
        self.serialize_with_options(
            SerializationOptions::display(&lang),
            Some(&rdm_cache.inner),
            Some(&node_config_manager.inner),
            resource_registry.map(|r| &r.inner),
        )
    }

    /// Populate (if needed) and serialize to raw JSON in a single synchronous call.
    #[napi]
    pub fn to_json_full(&self) -> Result<serde_json::Value> {
        self.ensure_populated()?;
        self.serialize_with_options(SerializationOptions::tile_data(), None, None, None)
    }

    // =========================================================================
    // Cleanup
    // =========================================================================

    #[napi]
    pub fn release(&mut self) {
        self.inner.release();
    }
}

// =============================================================================
// Private helpers
// =============================================================================

impl NapiResourceInstanceWrapper {
    /// This instance's graph's shared model, as it is now.
    fn model(&self) -> Result<Arc<GraphModelAccess>> {
        model_for(&self.inner.graph_id)
    }

    fn ensure_populated(&self) -> Result<()> {
        let needs_populate = {
            let cache = self
                .inner
                .pseudo_cache
                .read()
                .map_err(|_| napi::Error::from_reason("Failed to read pseudo cache"))?;
            cache.len() <= 1
        };

        if needs_populate {
            let model = self.model()?;
            let root_node = model.get_root_node().map_err(napi::Error::from_reason)?;
            let root_alias = root_node
                .alias
                .clone()
                .ok_or_else(|| napi::Error::from_reason("Root node has no alias"))?;

            let nodegroup_ids: Vec<String> = model
                .get_nodegroups()
                .map(|ngs| alizarin_core::sorted_serde::sorted_keys(ngs.keys()))
                .unwrap_or_default();

            self.inner
                .populate(false, &nodegroup_ids, &root_alias, &*model)
                .map_err(sc_err)?;
        }

        Ok(())
    }

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
            .read()
            .map_err(|_| napi::Error::from_reason("Failed to lock pseudo cache"))?;

        let model = self.model()?;
        // Get root alias
        let root_node = model.get_root_node().map_err(napi::Error::from_reason)?;
        let root_alias = root_node
            .alias
            .as_deref()
            .ok_or_else(|| napi::Error::from_reason("Root node has no alias"))?;

        let root_list = cache.get(root_alias).ok_or_else(|| {
            napi::Error::from_reason("Root pseudo not found — call populate() first")
        })?;

        // Build graph indices for VisitorContext
        let nodes_by_alias = model
            .get_nodes_by_alias_arc()
            .ok_or_else(|| napi::Error::from_reason("Nodes by alias not built"))?;
        let edges = model
            .get_edges_arc()
            .ok_or_else(|| napi::Error::from_reason("Edges not built"))?;

        let ext_reg = ext_registry();

        let ser_ctx = SerializationContext {
            node_config: None,
            external_resolver: rdm
                .map(|r| r as &dyn alizarin_core::type_serialization::ExternalResolver),
            concept_lookup: None,
            resource_resolver: resource_registry
                .map(|r| r as &dyn alizarin_core::type_serialization::ResourceDisplayResolver),
            extension_registry: Some(&ext_reg),
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
///
/// A handle on its graph's model in core's registry, which instance wrappers
/// share: it reads that model and changes it only through core.
#[napi]
pub struct NapiResourceModelWrapper {
    graph_id: String,
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
        Self::register(Arc::new(graph), default_allow)
    }

    /// Create from an already-loaded NapiStaticGraph.
    #[napi(factory)]
    pub fn from_graph(graph: &crate::NapiStaticGraph, default_allow: bool) -> Result<Self> {
        let mut cloned = graph.inner_ref().clone();
        cloned.build_indices();
        Self::register(Arc::new(cloned), default_allow)
    }

    // =========================================================================
    // Graph ID
    // =========================================================================

    #[napi(js_name = "getGraphId")]
    pub fn get_graph_id(&self) -> String {
        self.graph_id.clone()
    }

    // =========================================================================
    // Graph getter/setter
    // =========================================================================

    #[napi(getter)]
    pub fn graph(&self) -> Result<serde_json::Value> {
        let model = self.model()?;
        serde_json::to_value(model.get_graph()).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    #[napi(setter)]
    pub fn set_graph(&mut self, graph_json: String) -> Result<()> {
        let graph: StaticGraph = serde_json::from_str(&graph_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid graph JSON: {}", e)))?;
        self.update(|model| {
            model.rebuild_from_graph(&graph);
            Ok(())
        })
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
        self.update(|model| {
            model.rebuild_from_graph(&graph);
            Ok(())
        })
    }

    // =========================================================================
    // Node accessors
    // =========================================================================

    /// Get the root node of the graph.
    #[napi(js_name = "getRootNode")]
    pub fn get_root_node(&self) -> Result<serde_json::Value> {
        let model = self.model()?;
        let root = model.get_root_node().map_err(napi::Error::from_reason)?;
        serde_json::to_value(&*root).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Get all nodes as a Map<string, StaticNode>.
    #[napi(js_name = "getNodeObjects")]
    pub fn get_node_objects(&self) -> Result<HashMap<String, serde_json::Value>> {
        let model = self.model()?;
        let nodes = model
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
        let model = self.model()?;
        let nodes = model
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
        let model = self.model()?;
        let nodes = model
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
        let model = self.model()?;
        let nodes = model
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
        let model = self.model()?;
        let children = model.get_child_nodes(&node_id);
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
        self.model()
            .map(|model| model.get_child_nodes(&node_id).keys().cloned().collect())
            .unwrap_or_default()
    }

    // =========================================================================
    // Edge accessors
    // =========================================================================

    /// Get edges as Map<parent_id, [child_ids]>.
    #[napi(js_name = "getEdges")]
    pub fn get_edges(&self) -> Result<HashMap<String, Vec<String>>> {
        let model = self.model()?;
        let edges = model
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
        let model = self.model()?;
        let nodegroups = model
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
        self.model()
            .ok()
            .and_then(|model| {
                model
                    .get_nodegroups()
                    .map(|ng| alizarin_core::sorted_serde::sorted_keys(ng.keys()))
            })
            .unwrap_or_default()
    }

    /// Get nodegroup name (actually the name of the root node for that nodegroup).
    #[napi(js_name = "getNodegroupName")]
    pub fn get_nodegroup_name(&self, nodegroup_id: String) -> Result<String> {
        let model = self.model()?;
        let nodes = model
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
        let model = self.model()?;
        let nodes = model
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
        self.model()
            .map(|model| model.get_permitted_nodegroups_bool())
            .unwrap_or_default()
    }

    /// Check if a nodegroup is permitted.
    #[napi(js_name = "isNodegroupPermitted")]
    pub fn is_nodegroup_permitted(&self, nodegroup_id: String) -> bool {
        self.model()
            .is_ok_and(|model| model.is_nodegroup_permitted(&nodegroup_id))
    }

    /// Set permitted nodegroups. Accepts an object with boolean values or
    /// conditional rule objects { path, allowed }.
    #[napi(js_name = "setPermittedNodegroups")]
    pub fn set_permitted_nodegroups(&mut self, permissions: serde_json::Value) -> Result<()> {
        let perms =
            PermissionRule::rules_from_json(&permissions).map_err(napi::Error::from_reason)?;
        alizarin_core::set_model_permissions(&self.graph_id, perms)
            .map_err(napi::Error::from_reason)
    }

    #[napi(js_name = "setDefaultAllowAllNodegroups")]
    pub fn set_default_allow_all_nodegroups(&mut self, default_allow: bool) -> Result<()> {
        alizarin_core::set_model_default_allow(&self.graph_id, default_allow)
            .map_err(napi::Error::from_reason)
    }

    // =========================================================================
    // Graph modification
    // =========================================================================

    #[napi(js_name = "setGraphNodes")]
    pub fn set_graph_nodes(&mut self, nodes_json: String) -> Result<()> {
        let nodes: Vec<StaticNode> = serde_json::from_str(&nodes_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid nodes JSON: {}", e)))?;
        self.update(|model| {
            model.set_graph_nodes(nodes);
            Ok(())
        })
    }

    #[napi(js_name = "setGraphEdges")]
    pub fn set_graph_edges(&mut self, edges_json: String) -> Result<()> {
        let edges: Vec<StaticEdge> = serde_json::from_str(&edges_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid edges JSON: {}", e)))?;
        self.update(|model| {
            model.set_graph_edges(edges);
            Ok(())
        })
    }

    #[napi(js_name = "setGraphNodegroups")]
    pub fn set_graph_nodegroups(&mut self, nodegroups_json: String) -> Result<()> {
        let nodegroups: Vec<StaticNodegroup> = serde_json::from_str(&nodegroups_json)
            .map_err(|e| napi::Error::from_reason(format!("Invalid nodegroups JSON: {}", e)))?;
        self.update(|model| {
            model.set_graph_nodegroups(nodegroups);
            Ok(())
        })
    }

    // =========================================================================
    // Graph pruning
    // =========================================================================

    /// Prune graph to only include permitted nodegroups and their dependencies.
    #[napi(js_name = "pruneGraph")]
    pub fn prune_graph(&mut self, keep_functions: Option<Vec<String>>) -> Result<()> {
        self.update(|model| model.prune_graph(keep_functions.as_deref()))
    }
}

// Private helpers for NapiResourceModelWrapper
impl NapiResourceModelWrapper {
    /// Register `graph` and its shared model with core (replacing any earlier one).
    fn register(graph: Arc<StaticGraph>, default_allow: bool) -> Result<Self> {
        let graph_id = graph.graphid.clone();
        alizarin_core::register_model(graph, default_allow).map_err(napi::Error::from_reason)?;
        Ok(NapiResourceModelWrapper { graph_id })
    }

    /// The graph's shared model, as it is now.
    fn model(&self) -> Result<Arc<GraphModelAccess>> {
        model_for(&self.graph_id)
    }

    /// Change the graph's shared model through core (copy-on-write).
    fn update<R>(
        &self,
        f: impl FnOnce(&mut GraphModelAccess) -> std::result::Result<R, String>,
    ) -> Result<R> {
        alizarin_core::update_model(&self.graph_id, f).map_err(napi::Error::from_reason)
    }
}
