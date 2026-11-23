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
use alizarin_core::graph::{StaticGraph, StaticResource, StaticTile};
use alizarin_core::instance_wrapper_core::ModelAccess;
use alizarin_core::instance_wrapper_core::{
    ResourceInstanceWrapperCore, SemanticChildError, SemanticChildResult,
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

/// The resolvers and language a display-mode card serialization uses.
struct CardDisplay<'a> {
    rdm: &'a RdmCache,
    ncm: &'a NodeConfigManager,
    registry: Option<&'a alizarin_core::StaticResourceRegistry>,
    language: String,
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

    // As WASM's WasmRdmCache and Python's RustRdmCache: lookups, all core's.

    /// Every loaded collection's id.
    #[napi]
    pub fn get_collection_ids(&self) -> Vec<String> {
        self.inner.get_collection_ids()
    }

    /// A concept's label in a language, if any.
    #[napi]
    pub fn lookup_label(
        &self,
        collection_id: String,
        concept_id: String,
        language: String,
    ) -> Option<String> {
        self.inner
            .lookup_label(&collection_id, &concept_id, &language)
    }

    /// A concept as JSON, if any.
    #[napi]
    pub fn lookup_concept(
        &self,
        collection_id: String,
        concept_id: String,
    ) -> Result<Option<serde_json::Value>> {
        self.inner
            .lookup_concept(&collection_id, &concept_id)
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// A label value by its value id: `{id, value, conceptId, language}`.
    #[napi]
    pub fn lookup_value(
        &self,
        collection_id: String,
        value_id: String,
    ) -> Result<Option<serde_json::Value>> {
        self.inner
            .lookup_value_info(&collection_id, &value_id)
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// The concept a value id belongs to, if any.
    #[napi]
    pub fn get_concept_id_for_value(
        &self,
        collection_id: String,
        value_id: String,
    ) -> Option<String> {
        self.inner
            .get_concept_id_for_value(&collection_id, &value_id)
            .map(|s| s.to_string())
    }

    /// Whether a value id is in a collection.
    #[napi]
    pub fn validate_value(&self, collection_id: String, value_id: String) -> bool {
        self.inner.validate_value(&collection_id, &value_id)
    }

    /// Add the collections in SKOS RDF/XML: their ids.
    #[napi]
    pub fn add_collections_from_skos_xml(
        &mut self,
        xml_content: String,
        base_uri: String,
    ) -> Result<Vec<String>> {
        let collections = alizarin_core::skos::parse_skos_to_collections(&xml_content, &base_uri)
            .map_err(napi::Error::from_reason)?;
        Ok(self.inner.add_from_skos_collections(&collections))
    }
}

// =============================================================================
// Global RDM cache: core's, as WASM's and Python's. Tree conversion resolves
// concept labels against it.
// =============================================================================

/// Set the global RDM cache to a copy of `cache`.
#[napi]
pub fn set_global_rdm_cache(cache: &NapiRdmCache) {
    alizarin_core::set_global_rdm_cache(cache.inner.clone());
}

/// Whether a global RDM cache is set.
#[napi]
pub fn has_global_rdm_cache() -> bool {
    alizarin_core::has_global_rdm_cache()
}

/// Clear the global RDM cache.
#[napi]
pub fn clear_global_rdm_cache() {
    alizarin_core::clear_global_rdm_cache();
}

/// Add SKOS XML collections to the global RDM cache: their ids.
#[napi]
pub fn add_to_global_rdm_cache_from_skos_xml(
    xml_content: String,
    base_uri: String,
) -> Result<Vec<String>> {
    alizarin_core::add_to_global_rdm_cache_from_skos_xml(&xml_content, &base_uri)
        .map_err(napi::Error::from_reason)
}

/// Add a collection (its concepts' JSON) to the global RDM cache.
#[napi]
pub fn add_collection_to_global_rdm_cache(
    collection_id: String,
    concepts_json: String,
) -> Result<()> {
    alizarin_core::add_collection_to_global_rdm_cache(&collection_id, &concepts_json)
        .map_err(napi::Error::from_reason)
}

/// Remove a collection from the global RDM cache: whether it was there.
#[napi]
pub fn remove_from_global_rdm_cache(collection_id: String) -> bool {
    alizarin_core::remove_from_global_rdm_cache(&collection_id)
}

/// A concept's label in the global RDM cache, if any.
#[napi]
pub fn global_rdm_lookup_label(
    collection_id: String,
    concept_id: String,
    language: String,
) -> Option<String> {
    alizarin_core::global_rdm_lookup_label(&collection_id, &concept_id, &language)
}

/// Whether the global RDM cache has a collection.
#[napi]
pub fn global_rdm_has_collection(collection_id: String) -> bool {
    alizarin_core::global_rdm_has_collection(&collection_id)
}

/// A concept's parent in the global RDM cache, if any.
#[napi]
pub fn global_rdm_get_parent_id(collection_id: String, concept_id: String) -> Option<String> {
    alizarin_core::global_rdm_get_parent_id(&collection_id, &concept_id)
}

/// The global RDM cache's collection ids.
#[napi]
pub fn global_rdm_get_collection_ids() -> Vec<String> {
    alizarin_core::global_rdm_get_collection_ids()
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
    pub fn set(&mut self, key: String, #[napi(ts_arg_type = "any")] value: crate::JsonArg) {
        self.entries.lock().unwrap().insert(key, value.0);
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
        #[napi(ts_arg_type = "any[] | null")] provisionaledits: Option<crate::JsonArg>,
    ) -> Self {
        NapiStaticTile {
            tileid,
            nodegroup_id,
            sortorder,
            resourceinstance_id: resourceinstance_id.unwrap_or_default(),
            parenttile_id,
            provisionaledits: match provisionaledits {
                Some(crate::JsonArg(serde_json::Value::Array(edits))) => Some(edits),
                _ => None,
            },
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
    pub fn set_data(&mut self, #[napi(ts_arg_type = "any")] _value: napi::JsUnknown) {
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

    // Tile fields keep their snake_case names, as on WASM's StaticTile and in
    // the TS that reads them.
    #[napi(getter, js_name = "nodegroup_id")]
    pub fn nodegroup_id(&self) -> String {
        self.nodegroup_id.clone()
    }

    #[napi(getter)]
    pub fn sortorder(&self) -> Option<i32> {
        self.sortorder
    }

    #[napi(getter, js_name = "resourceinstance_id")]
    pub fn resourceinstance_id(&self) -> String {
        self.resourceinstance_id.clone()
    }

    #[napi(getter, js_name = "parenttile_id")]
    pub fn parenttile_id(&self) -> Option<String> {
        self.parenttile_id.clone()
    }

    #[napi(setter, js_name = "parenttile_id")]
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

    /// Core's serialisation, as WASM's tile uses, so empty fields follow the
    /// one rule (see `StaticTile` in core). Without it a tile stringified as `{}`.
    #[napi(js_name = "toJSON")]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self.to_static_tile()).unwrap_or(serde_json::Value::Null)
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

    // The node's own fields are on `node`, not repeated here.

    /// Core's structural flags, as on WASM and Python.
    #[napi(getter)]
    pub fn has_tile(&self) -> bool {
        self.inner.has_tile()
    }

    #[napi(getter)]
    pub fn is_inner(&self) -> bool {
        self.inner.is_inner
    }

    #[napi(getter)]
    pub fn is_outer(&self) -> bool {
        self.inner.is_outer()
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

    #[napi]
    pub fn is_iterable(&self) -> bool {
        alizarin_core::is_iterable_datatype(&self.inner.node.datatype)
    }

    /// Snapshot all properties in one call (avoids multiple N-API boundary crossings)
    #[napi]
    pub fn to_snapshot(&self) -> serde_json::Value {
        // Core's (its datatype an inner's "semantic"), as on WASM, with
        // NAPI's runtime state added.
        let mut map = self.inner.snapshot();
        map.insert("accessed".to_string(), serde_json::Value::Bool(false));
        map.insert(
            "valueLoaded".to_string(),
            serde_json::Value::Bool(self.inner.has_tile()),
        );
        serde_json::Value::Object(map)
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
        // Core's, keeping tileData in step, as on WASM.
        self.inner
            .set_tile(tile.map(|t| Arc::new(t.to_static_tile())));
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

    /// Every value list by alias, in one boundary crossing: an object of
    /// `NapiPseudoList`s, as WASM's `getAllValues` gives its PseudoLists.
    #[napi]
    pub fn get_all_values(&self) -> HashMap<String, NapiPseudoList> {
        self.values
            .iter()
            .map(|(alias, list)| {
                let list = NapiPseudoList {
                    inner: list.clone(),
                };
                (alias.clone(), list)
            })
            .collect()
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
                resource_instance_lifecycle_state_id: None,
                lifecycle_state_conflict: Vec::new(),
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
            self.inner.mark_loaded_comprehensively(&model);
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
                self.inner.mark_loaded_comprehensively(&model);
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
                self.inner.mark_loaded_comprehensively(&model);
            }
        }
        Ok(true)
    }

    /// Append tiles incrementally (for lazy loading); comprehensive unless
    /// told otherwise, as WASM's `appendTiles` is.
    #[napi]
    pub fn append_tiles(
        &mut self,
        #[napi(ts_arg_type = "any")] tiles_js: crate::JsonArg,
        assume_tiles_comprehensive_for_nodegroup: Option<bool>,
    ) -> Result<()> {
        let tiles: Vec<StaticTile> = serde_json::from_value(tiles_js.0)
            .map_err(|e| napi::Error::from_reason(format!("Invalid tiles JSON: {e}")))?;
        let model = self.model()?;
        self.inner.merge_permitted_tiles(tiles, &model);
        if assume_tiles_comprehensive_for_nodegroup.unwrap_or(true) {
            self.inner.mark_loaded_comprehensively(&model);
        }
        Ok(())
    }

    /// Append the tiles a lazy fetch returned for `nodegroupId` (null: the
    /// whole resource), marking just that nodegroup loaded, even if it had no
    /// tiles. Core's, as WASM and Python use.
    #[napi]
    pub fn append_fetched_tiles(
        &mut self,
        #[napi(ts_arg_type = "any")] tiles_js: crate::JsonArg,
        nodegroup_id: Option<String>,
    ) -> Result<()> {
        let tiles: Vec<StaticTile> = serde_json::from_value(tiles_js.0)
            .map_err(|e| napi::Error::from_reason(format!("Invalid tiles JSON: {e}")))?;
        let model = self.model()?;
        self.inner
            .merge_fetched_tiles(tiles, nodegroup_id.as_deref(), &model);
        Ok(())
    }

    /// Export all tiles as a JSON string.
    /// Returns the wrapper's stored tiles as they now stand (edits committed by
    /// `applyEdits` included; on-load derived tiles left out).
    /// Returned as a string for fast boundary crossing — call JSON.parse() on the JS side.
    #[napi]
    pub fn export_tiles_json(&self) -> Result<String> {
        let tiles: Vec<&StaticTile> = self.inner.stored_tiles();
        serde_json::to_string(&tiles).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Prune tiles to only keep those in permitted nodegroups.
    #[napi(js_name = "pruneResourceTiles")]
    pub fn prune_resource_tiles(&mut self) -> Result<()> {
        // Core's, as on WASM and Python: the tiles and their index entries.
        let model = self.model()?;
        self.inner
            .prune_to_permitted(&model)
            .map_err(napi::Error::from_reason)
    }

    #[napi]
    pub fn tiles_loaded(&self) -> bool {
        // Core's, as on WASM.
        self.inner.tiles_loaded()
    }

    #[napi]
    pub fn get_tile(&self, tile_id: String) -> Result<serde_json::Value> {
        let tile = self
            .inner
            .get_tile(&tile_id)
            .ok_or_else(|| napi::Error::from_reason(format!("Tile '{}' not found", tile_id)))?;
        serde_json::to_value(tile).map_err(|e| napi::Error::from_reason(e.to_string()))
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

    /// Whether tiles arrive per nodegroup as lookups need them. Core's, as on
    /// WASM and Python.
    #[napi]
    pub fn is_lazy(&self) -> bool {
        self.inner.is_lazy()
    }

    #[napi]
    pub fn is_nodegroup_loaded(&self, nodegroup_id: Option<String>) -> bool {
        match nodegroup_id {
            Some(id) => self.inner.is_nodegroup_loaded(&id),
            None => false,
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

    /// The nodegroups to fetch before `nodegroupId` can be read (in lazy mode;
    /// none otherwise): itself and, for an on-load Derive target, its unloaded
    /// sources. Core's, as on WASM and Python.
    #[napi]
    pub fn nodegroups_to_fetch(&self, nodegroup_id: String) -> Result<Vec<String>> {
        let model = self.model()?;
        Ok(self.inner.nodegroups_to_fetch(&nodegroup_id, &*model))
    }

    /// Throws if node `alias` can't be given a value here (unknown, semantic,
    /// derived, or a lazy nodegroup not loaded): what the ORM checks on
    /// assignment. Core's, as on WASM and Python.
    #[napi]
    pub fn check_editable(&self, alias: String) -> Result<()> {
        let model = self.model()?;
        self.inner
            .check_editable(&alias, &model)
            .map_err(napi::Error::from_reason)
    }

    /// Apply and commit a batch of edits as the ORM recorded them
    /// (`[{op: "set" | "add" | "remove", ...}]`), all or nothing, returning
    /// `{upserted, removed}`. Core's, as on WASM and Python.
    #[napi]
    pub fn apply_edits(
        &mut self,
        #[napi(ts_arg_type = "any")] ops: crate::JsonArg,
    ) -> Result<serde_json::Value> {
        let ops: Vec<alizarin_core::EditOp> = serde_json::from_value(ops.0)
            .map_err(|e| napi::Error::from_reason(format!("invalid edits: {e}")))?;
        let model = self.model()?;
        let delta = self
            .inner
            .apply_edits(&ops, &model)
            .map_err(napi::Error::from_reason)?;
        serde_json::to_value(delta).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Fetch `nodegroupIds` from the compiled-in tile source, if there is one,
    /// returning those still for the host's loader. Core's, as on WASM and
    /// Python.
    #[napi]
    pub fn fetch_from_tile_source(&mut self, nodegroup_ids: Vec<String>) -> Result<Vec<String>> {
        let model = self.model()?;
        self.inner
            .fetch_from_tile_source(&nodegroup_ids, &model)
            .map_err(|e| napi::Error::from_reason(format!("TileSource error: {e}")))
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
        serde_json::to_value(alizarin_core::ResolvedPath::from(&info))
            .map_err(|e| napi::Error::from_reason(e.to_string()))
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

    /// Serialize to JSON (tile_data mode — raw values), populating from the
    /// loaded tiles first if needed. As on WASM and Python.
    #[napi]
    pub fn to_json(&self) -> Result<serde_json::Value> {
        self.ensure_populated()?;
        self.serialize_with_options(SerializationOptions::tile_data(), None, None, None)
    }

    /// Serialize to display JSON (resolved labels), populating from the loaded
    /// tiles first if needed. As on WASM and Python.
    #[napi]
    pub fn to_display_json(
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

    // =========================================================================
    // Card serialization (core's, as on WASM and Python)
    // =========================================================================

    /// Serialize card `cardId` from the pseudo cache (populate the nodegroups
    /// `nodegroupIdsForCard` lists first). `maxDepth`: null for unlimited, 0
    /// for this card only.
    #[napi]
    pub fn serialize_card(
        &self,
        card_id: String,
        parent_tile_id: Option<String>,
        parent_nodegroup_id: Option<String>,
        max_depth: Option<u32>,
    ) -> Result<serde_json::Value> {
        self.serialize_cards_with(
            Some(&card_id),
            parent_tile_id.as_deref(),
            parent_nodegroup_id.as_deref(),
            max_depth,
            None,
        )
    }

    /// Serialize every root card from the pseudo cache.
    #[napi]
    pub fn serialize_root_cards(&self, max_depth: Option<u32>) -> Result<serde_json::Value> {
        self.serialize_cards_with(None, None, None, max_depth, None)
    }

    /// Serialize card `cardId` in display mode (labels for UUIDs).
    #[allow(clippy::too_many_arguments)]
    #[napi]
    pub fn serialize_card_display(
        &self,
        card_id: String,
        rdm_cache: &NapiRdmCache,
        node_config_manager: &NapiNodeConfigManager,
        parent_tile_id: Option<String>,
        parent_nodegroup_id: Option<String>,
        max_depth: Option<u32>,
        language: Option<String>,
        resource_registry: Option<&crate::NapiStaticResourceRegistry>,
    ) -> Result<serde_json::Value> {
        self.serialize_cards_with(
            Some(&card_id),
            parent_tile_id.as_deref(),
            parent_nodegroup_id.as_deref(),
            max_depth,
            Some(CardDisplay {
                rdm: &rdm_cache.inner,
                ncm: &node_config_manager.inner,
                registry: resource_registry.map(|r| &r.inner),
                language: language.unwrap_or_else(|| "en".to_string()),
            }),
        )
    }

    /// Serialize every root card in display mode.
    #[napi]
    pub fn serialize_root_cards_display(
        &self,
        rdm_cache: &NapiRdmCache,
        node_config_manager: &NapiNodeConfigManager,
        max_depth: Option<u32>,
        language: Option<String>,
        resource_registry: Option<&crate::NapiStaticResourceRegistry>,
    ) -> Result<serde_json::Value> {
        self.serialize_cards_with(
            None,
            None,
            None,
            max_depth,
            Some(CardDisplay {
                rdm: &rdm_cache.inner,
                ncm: &node_config_manager.inner,
                registry: resource_registry.map(|r| &r.inner),
                language: language.unwrap_or_else(|| "en".to_string()),
            }),
        )
    }

    /// The nodegroups card `cardId` and its descendants (to `maxDepth`) need
    /// populated before `serializeCard`.
    #[napi]
    pub fn nodegroup_ids_for_card(
        &self,
        card_id: String,
        max_depth: Option<u32>,
    ) -> Result<Vec<String>> {
        let model = self.model()?;
        ResourceInstanceWrapperCore::nodegroup_ids_for_card(
            &card_id,
            max_depth.map(|d| d as usize),
            &model,
        )
        .map_err(napi::Error::from_reason)
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

    /// Set a compiled-in tile source, tried before the host's loader
    /// (`fetchFromTileSource`). Rust-only, as on WASM and Python: for a binary
    /// that compiles alizarin and a tile backend together.
    pub fn set_tile_source(&mut self, source: Arc<dyn alizarin_core::TileSource>) {
        self.inner.tile_source = Some(source);
    }

    /// Remove the compiled-in tile source.
    pub fn clear_tile_source(&mut self) {
        self.inner.tile_source = None;
    }

    /// Core's, as WASM and Python serialise.
    fn ensure_populated(&self) -> Result<()> {
        let model = self.model()?;
        self.inner.ensure_populated(&*model).map_err(sc_err)
    }

    /// Core's `serialize_cards`, in display mode with `display`'s resolvers.
    fn serialize_cards_with(
        &self,
        card_id: Option<&str>,
        parent_tile_id: Option<&str>,
        parent_nodegroup_id: Option<&str>,
        max_depth: Option<u32>,
        display: Option<CardDisplay>,
    ) -> Result<serde_json::Value> {
        let model = self.model()?;
        let max_depth = max_depth.map(|d| d as usize);
        let Some(display) = display else {
            return self
                .inner
                .serialize_cards(
                    card_id,
                    parent_tile_id,
                    parent_nodegroup_id,
                    max_depth,
                    &model,
                    None,
                )
                .map_err(napi::Error::from_reason);
        };
        let opts = SerializationOptions::display(&display.language);
        let ext_reg = ext_registry();
        let ser_ctx = SerializationContext {
            node_config: None,
            external_resolver: Some(
                display.rdm as &dyn alizarin_core::type_serialization::ExternalResolver,
            ),
            concept_lookup: None,
            resource_resolver: display
                .registry
                .map(|r| r as &dyn alizarin_core::type_serialization::ResourceDisplayResolver),
            extension_registry: Some(&ext_reg),
        };
        self.inner
            .serialize_cards(
                card_id,
                parent_tile_id,
                parent_nodegroup_id,
                max_depth,
                &model,
                Some(alizarin_core::card_traversal::CardSerializationParams {
                    opts: &opts,
                    ser_ctx: &ser_ctx,
                    node_config_manager: Some(display.ncm),
                }),
            )
            .map_err(napi::Error::from_reason)
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
// NapiPseudoNode
// =============================================================================

/// A graph node for navigating a model's structure (`NodeViewModel`): core's
/// [`alizarin_core::PseudoNodeCore`], with the same names as WASM's
/// `PseudoNode`.
#[napi]
pub struct NapiPseudoNode {
    core: alizarin_core::PseudoNodeCore,
}

#[napi]
impl NapiPseudoNode {
    #[napi(js_name = "isIterable")]
    pub fn is_iterable(&self) -> bool {
        self.core.is_iterable()
    }

    /// The path from where navigation started (`.group.basic_info`), with
    /// `[*]` after a list node.
    #[napi(js_name = "getNodePlaceholder")]
    pub fn get_node_placeholder(&self) -> String {
        self.core.placeholder()
    }

    #[napi(getter)]
    pub fn node(&self) -> serde_json::Value {
        serde_json::to_value(&*self.core.node).unwrap_or(serde_json::Value::Null)
    }

    #[napi(getter)]
    pub fn size(&self) -> u32 {
        self.core.child_nodes.len() as u32
    }

    #[napi(getter, js_name = "childNodeAliases")]
    pub fn child_node_aliases(&self) -> Vec<String> {
        self.core.child_nodes.keys().cloned().collect()
    }

    /// Children by alias, as a `Map` (as on WASM).
    #[napi(getter, js_name = "childNodes", ts_return_type = "Map<string, any>")]
    pub fn child_nodes(&self, env: Env) -> Result<napi::JsObject> {
        let map_ctor: napi::JsFunction = env.get_global()?.get_named_property("Map")?;
        let map = map_ctor.new_instance::<napi::JsUnknown>(&[])?;
        let set: napi::JsFunction = map.get_named_property("set")?;
        for (alias, node) in &self.core.child_nodes {
            let key = env.create_string(alias)?.into_unknown();
            let value = crate::json_arg::to_js(&env, &**node)?;
            set.call(Some(&map), &[key, value])?;
        }
        Ok(map)
    }

    #[napi(getter)]
    pub fn nodeid(&self) -> String {
        self.core.node.nodeid.clone()
    }

    #[napi(getter)]
    pub fn alias(&self) -> String {
        self.core.node.alias.clone().unwrap_or_default()
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.core.node.name.clone()
    }

    #[napi(getter)]
    pub fn description(&self) -> serde_json::Value {
        self.core
            .node
            .description
            .as_ref()
            .and_then(|d| serde_json::to_value(d.to_json()).ok())
            .unwrap_or(serde_json::Value::Null)
    }

    #[napi(getter, js_name = "graphId")]
    pub fn graph_id(&self) -> String {
        self.core.node.graph_id.clone()
    }

    #[napi(getter)]
    pub fn ontologyclass(&self) -> serde_json::Value {
        match self.core.node.ontologyclass.as_deref() {
            None | Some([]) => serde_json::Value::Null,
            Some([class]) => serde_json::Value::String(class.clone()),
            Some(classes) => serde_json::json!(classes),
        }
    }

    #[napi(getter, js_name = "nodegroup_id")]
    pub fn nodegroup_id(&self) -> Option<String> {
        self.core.node.nodegroup_id.clone()
    }

    #[napi(getter)]
    pub fn fieldname(&self) -> Option<String> {
        self.core.node.fieldname.clone()
    }

    #[napi(getter)]
    pub fn parentproperty(&self) -> Option<String> {
        self.core.node.parentproperty.clone()
    }

    #[napi(getter, js_name = "sourcebranchpublicationId")]
    pub fn sourcebranchpublication_id(&self) -> Option<String> {
        self.core.node.sourcebranchpublication_id.clone()
    }

    #[napi(getter)]
    pub fn exportable(&self) -> bool {
        self.core.node.exportable
    }

    #[napi(getter)]
    pub fn hascustomalias(&self) -> bool {
        self.core.node.hascustomalias
    }

    #[napi(getter, js_name = "isCollector")]
    pub fn is_collector(&self) -> bool {
        self.core.node.is_collector
    }

    #[napi(getter)]
    pub fn isrequired(&self) -> bool {
        self.core.node.isrequired
    }

    #[napi(getter)]
    pub fn issearchable(&self) -> bool {
        self.core.node.issearchable
    }

    #[napi(getter)]
    pub fn istopnode(&self) -> bool {
        self.core.node.istopnode
    }

    #[napi(getter)]
    pub fn sortorder(&self) -> Option<i32> {
        self.core.node.sortorder
    }

    #[napi(getter)]
    pub fn config(&self) -> serde_json::Value {
        serde_json::to_value(&self.core.node.config).unwrap_or(serde_json::Value::Null)
    }

    /// The datatype, read as `semantic` on an inner node.
    #[napi(getter)]
    pub fn datatype(&self) -> String {
        self.core.datatype().to_string()
    }

    #[napi(getter, js_name = "isOuter")]
    pub fn is_outer(&self) -> bool {
        self.core.is_outer()
    }

    #[napi(getter)]
    pub fn inner(&self) -> Result<NapiPseudoNode> {
        self.core
            .inner
            .as_ref()
            .map(|inner| NapiPseudoNode {
                core: (**inner).clone(),
            })
            .ok_or_else(|| napi::Error::from_reason("No inner node"))
    }

    #[napi(getter, js_name = "isInner")]
    pub fn is_inner(&self) -> bool {
        self.core.is_inner
    }

    #[napi(js_name = "toJSON")]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self.core.node.to_json()).unwrap_or(serde_json::Value::Null)
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
    // Graph getter/setter
    // =========================================================================

    /// The model's graph, as a `NapiStaticGraph` (as WASM gives its StaticGraph).
    #[napi(getter)]
    pub fn graph(&self) -> Result<crate::NapiStaticGraph> {
        let model = self.model()?;
        Ok(crate::NapiStaticGraph::from_inner(
            model.get_graph().clone(),
        ))
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
    // Node accessors
    // =========================================================================

    /// Get the root node of the graph.
    #[napi(js_name = "getRootNode")]
    pub fn get_root_node(&self) -> Result<serde_json::Value> {
        let model = self.model()?;
        let root = model.get_root_node().map_err(napi::Error::from_reason)?;
        serde_json::to_value(&*root).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// A pseudo node to navigate from: the root, or the node with `alias`.
    /// Core's, as WASM and Python use.
    #[napi(js_name = "createPseudoNode")]
    pub fn create_pseudo_node(&self, alias: Option<String>) -> Result<NapiPseudoNode> {
        let model = self.model()?;
        match alias {
            Some(alias) => alizarin_core::PseudoNodeCore::by_alias(&alias, &*model),
            None => alizarin_core::PseudoNodeCore::root(&*model),
        }
        .map(|core| NapiPseudoNode { core })
        .map_err(napi::Error::from_reason)
    }

    /// `parent`'s child `child`, which records `parent` as its parent.
    #[napi(js_name = "createPseudoNodeChild")]
    pub fn create_pseudo_node_child(
        &self,
        child: String,
        parent: &NapiPseudoNode,
    ) -> Result<NapiPseudoNode> {
        let model = self.model()?;
        parent
            .core
            .child(&child, &*model)
            .map(|core| NapiPseudoNode { core })
            .map_err(napi::Error::from_reason)
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
        self.model()?
            .nodegroup_name(&nodegroup_id)
            .map_err(napi::Error::from_reason)
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
    pub fn set_permitted_nodegroups(
        &mut self,
        #[napi(ts_arg_type = "any")] permissions: crate::JsonArg,
    ) -> Result<()> {
        let perms =
            PermissionRule::rules_from_json(&permissions.0).map_err(napi::Error::from_reason)?;
        alizarin_core::set_model_permissions(&self.graph_id, perms)
            .map_err(napi::Error::from_reason)
    }

    #[napi(js_name = "setDefaultAllowAllNodegroups")]
    pub fn set_default_allow_all_nodegroups(&mut self, default_allow: bool) -> Result<()> {
        alizarin_core::set_model_default_allow(&self.graph_id, default_allow)
            .map_err(napi::Error::from_reason)
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
