/// Graph Registry
///
/// A thread-safe global registry for storing graphs by graph_id.
/// Used by batch_merge_resources and other functions that need to look up
/// graphs without passing them explicitly.
///
/// Uses RwLock for thread-safe access, allowing multiple concurrent readers
/// or exclusive write access. This works correctly with rayon's parallel iterators.
///
/// SILENT: All `.ok()` calls on RwLock operations are deliberate — a poisoned
/// lock means another thread panicked while holding it. Propagating that panic
/// cross-thread isn't useful; returning a sensible default (None, false, empty)
/// lets the caller handle the "not found" case normally.
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::extension_type_registry::ExtensionTypeRegistry;
use crate::graph::FunctionsRegistry;
use crate::graph_model_access::GraphModelAccess;
use crate::permissions::PermissionRule;
use crate::rdm_cache::RdmCache;
use crate::skos::{parse_skos_to_collections, SkosCollection};
use crate::StaticGraph;

lazy_static::lazy_static! {
    /// Registry mapping graph_id -> StaticGraph
    /// Uses Arc for cheap cloning when retrieving graphs
    /// Uses RwLock for thread-safe access across parallel threads
    static ref GRAPH_REGISTRY: RwLock<HashMap<String, Arc<StaticGraph>>> =
        RwLock::new(HashMap::new());

    /// Every registered model by its id, and each graph's latest; see
    /// [`register_model_with_id`]. Always built: readers need only `&self`
    /// accessors.
    static ref MODEL_REGISTRY: RwLock<Models> = RwLock::new(Models::default());

    /// Global RDM (Reference Data Manager) cache for concept collections.
    /// Used by label resolution, display rendering, and other functions that
    /// need concept lookups without passing a cache explicitly.
    static ref GLOBAL_RDM_CACHE: RwLock<Option<Arc<RdmCache>>> = RwLock::new(None);

    /// Registry of datatypes where the array IS the value (list types).
    /// For these datatypes, arrays should NOT be iterated over during tree-to-tiles conversion.
    /// Extensions can register their list datatypes here.
    /// Core list types are registered at initialization.
    static ref LIST_DATATYPE_REGISTRY: RwLock<HashSet<String>> = {
        let mut set = HashSet::new();
        // Core list datatypes where array is the value
        set.insert("concept-list".to_string());
        set.insert("resource-instance-list".to_string());
        set.insert("domain-value-list".to_string());
        RwLock::new(set)
    };

    /// Registry mapping datatype -> widget_name for extension datatypes.
    /// Extensions can register their custom datatype-to-widget mappings here.
    /// Core mappings are handled in graph_mutator::get_default_widget_for_datatype.
    static ref WIDGET_MAPPING_REGISTRY: RwLock<HashMap<String, String>> =
        RwLock::new(HashMap::new());

    /// Registry for dynamically registered widgets from extensions.
    /// Maps widget_name -> Widget definition.
    static ref WIDGET_REGISTRY: RwLock<HashMap<String, RegisteredWidget>> =
        RwLock::new(HashMap::new());

    /// Global extension type registry.
    /// Set by FFI targets (Python, WASM, NAPI) after registering extension handlers.
    /// Read by extension crates (e.g. alizarin-pg) that need extension coercion
    /// without depending on a specific FFI target.
    static ref GLOBAL_EXTENSION_REGISTRY: RwLock<Option<ExtensionTypeRegistry>> =
        RwLock::new(None);

    /// Global graph-attached function registry (Derive / Descriptor / Aggregate
    /// providers). Seeded with core's built-ins (`default_functions_registry`,
    /// i.e. the fold Aggregate); provider crates add theirs at binding init (e.g.
    /// `alizarin_geo_core::register_functions`). Consulted by the load-time derive
    /// hook and by `apply_aggregate_functions_to_resource`.
    static ref GLOBAL_FUNCTIONS_REGISTRY: RwLock<FunctionsRegistry> =
        RwLock::new(crate::graph::default_functions_registry());
}

/// A dynamically registered widget definition.
/// Similar to graph_mutator::Widget but owned (not 'static).
#[derive(Debug, Clone)]
pub struct RegisteredWidget {
    pub id: String,
    pub name: String,
    pub datatype: String,
    pub default_config: serde_json::Value,
}

impl RegisteredWidget {
    pub fn new(id: &str, name: &str, datatype: &str, default_config_json: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            datatype: datatype.to_string(),
            default_config: serde_json::from_str(default_config_json)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        }
    }

    /// Get a fresh copy of the default config
    pub fn get_default_config(&self) -> serde_json::Value {
        self.default_config.clone()
    }
}

/// Register a graph in the registry
pub fn register_graph(graph_id: &str, graph: Arc<StaticGraph>) {
    if let Ok(mut registry) = GRAPH_REGISTRY.write() {
        registry.insert(graph_id.to_string(), graph);
    }
}

/// Register a graph from an owned StaticGraph (wraps in Arc)
pub fn register_graph_owned(graph: StaticGraph) {
    let graph_id = graph.graph_id().to_string();
    register_graph(&graph_id, Arc::new(graph));
}

/// Get a graph from the registry by graph_id
pub fn get_graph(graph_id: &str) -> Option<Arc<StaticGraph>> {
    GRAPH_REGISTRY
        .read()
        .ok()
        .and_then(|registry| registry.get(graph_id).cloned())
}

/// Check if a graph is registered
pub fn is_graph_registered(graph_id: &str) -> bool {
    GRAPH_REGISTRY
        .read()
        .ok()
        .map(|registry| registry.contains_key(graph_id))
        .unwrap_or(false)
}

/// Unregister a graph from the registry
pub fn unregister_graph(graph_id: &str) -> Option<Arc<StaticGraph>> {
    GRAPH_REGISTRY
        .write()
        .ok()
        .and_then(|mut registry| registry.remove(graph_id))
}

/// Clear all graphs from the registry
pub fn clear_registry() {
    if let Ok(mut registry) = GRAPH_REGISTRY.write() {
        registry.clear();
    }
}

/// Get the number of registered graphs
pub fn registry_size() -> usize {
    GRAPH_REGISTRY
        .read()
        .ok()
        .map(|registry| registry.len())
        .unwrap_or(0)
}

/// Get all registered graph IDs
pub fn get_registered_graph_ids() -> Vec<String> {
    GRAPH_REGISTRY
        .read()
        .ok()
        .map(|registry| registry.keys().cloned().collect())
        .unwrap_or_default()
}

// ============================================================================
// Model Registry
// ============================================================================

/// The registered models: each by its id (`<graph_id>#<n>`), and each
/// graph's latest, which a graph id names.
#[derive(Default)]
struct Models {
    by_id: HashMap<String, Arc<GraphModelAccess>>,
    latest: HashMap<String, String>,
    next: u64,
}

impl Models {
    /// The model id `id` names: itself if a model id, else its graph's latest.
    fn resolve(&self, id: &str) -> Option<String> {
        if self.by_id.contains_key(id) {
            Some(id.to_string())
        } else {
            self.latest.get(id).cloned()
        }
    }
}

/// Register a model for `graph`: the graph itself (sharing this `Arc`, as
/// [`register_graph`] stores it) and a [`GraphModelAccess`] over it, built
/// now, with `default_allow` and no explicit permission rules. Each call makes
/// a new model with its own id, so two model wrappers over one graph (two
/// managers, a public and a private view) keep their own permissions and
/// prunes; the graph id names the latest. Models are not removed when their
/// wrappers go: each costs a model's indices, shared with its graph.
pub fn register_model_with_id(
    graph: Arc<StaticGraph>,
    default_allow: bool,
) -> Result<(String, Arc<GraphModelAccess>), String> {
    let graph_id = graph.graph_id().to_string();
    let mut model = GraphModelAccess::new(Arc::clone(&graph), default_allow);
    model.ensure_built()?;
    let model = Arc::new(model);
    register_graph(&graph_id, graph);
    let mut models = MODEL_REGISTRY
        .write()
        .map_err(|_| "model registry lock poisoned".to_string())?;
    let id = format!("{graph_id}#{}", models.next);
    models.next += 1;
    models.by_id.insert(id.clone(), Arc::clone(&model));
    models.latest.insert(graph_id, id.clone());
    Ok((id, model))
}

/// [`register_model_with_id`], without the id: for callers that name the
/// model by its graph id (the latest).
pub fn register_model(
    graph: Arc<StaticGraph>,
    default_allow: bool,
) -> Result<Arc<GraphModelAccess>, String> {
    register_model_with_id(graph, default_allow).map(|(_, model)| model)
}

/// The model `id` names (a model id, or a graph id for its latest), as a
/// cheap `Arc` snapshot. Instances look it up on each use, so they see
/// changes made after they were created.
pub fn get_model(id: &str) -> Option<Arc<GraphModelAccess>> {
    let models = MODEL_REGISTRY.read().ok()?;
    models
        .resolve(id)
        .and_then(|id| models.by_id.get(&id).cloned())
}

/// Change the model `id` names with `f`, rebuilding its indices if `f`
/// invalidated them (a new graph, a prune). Copy-on-write: anyone holding the
/// previous snapshot keeps it unchanged, and the copy is cheap, as the index
/// maps are `Arc`-shared. Errors if no model is registered, or with `f`'s
/// error, leaving the registered model as it was.
pub fn update_model<R>(
    id: &str,
    f: impl FnOnce(&mut GraphModelAccess) -> Result<R, String>,
) -> Result<R, String> {
    let mut models = MODEL_REGISTRY
        .write()
        .map_err(|_| "model registry lock poisoned".to_string())?;
    let model_id = models
        .resolve(id)
        .ok_or_else(|| format!("no model registered for graph {id}"))?;
    let entry = models
        .by_id
        .get_mut(&model_id)
        .ok_or_else(|| format!("no model registered for graph {id}"))?;
    let mut model = (**entry).clone();
    let result = f(&mut model)?;
    model.ensure_built()?;
    *entry = Arc::new(model);
    Ok(result)
}

/// Set the permission rules on the model `id` names. Keys may be node aliases
/// or nodegroup ids: aliases resolve to nodegroup ids here, once, so every
/// binding and every check (tile filtering, pruning) sees the same keys.
pub fn set_model_permissions(
    id: &str,
    rules: HashMap<String, PermissionRule>,
) -> Result<(), String> {
    let graph_id = get_model(id)
        .map(|model| model.get_graph_arc().graph_id().to_string())
        .unwrap_or_else(|| id.to_string());
    let resolved = resolve_permission_keys(&graph_id, rules);
    update_model(id, |model| {
        model.set_permitted_nodegroups_rules(resolved);
        Ok(())
    })
}

/// Set whether nodegroups without a rule are permitted on the model `id` names.
pub fn set_model_default_allow(id: &str, default_allow: bool) -> Result<(), String> {
    update_model(id, |model| {
        model.set_default_allow(default_allow);
        Ok(())
    })
}

/// Unregister models (their graph stays registered): a model id, that model;
/// a graph id, every model of the graph. Returns the one `id` named.
pub fn unregister_model(id: &str) -> Option<Arc<GraphModelAccess>> {
    let mut models = MODEL_REGISTRY.write().ok()?;
    let named = models.resolve(id)?;
    let removed = models.by_id.get(&named).cloned();
    if models.by_id.contains_key(id) {
        models.by_id.remove(id);
        models.latest.retain(|_, latest| latest != id);
    } else {
        let prefix = format!("{id}#");
        models
            .by_id
            .retain(|model_id, _| !model_id.starts_with(&prefix));
        models.latest.remove(id);
    }
    removed
}

/// Clear every registered model (graphs stay registered).
pub fn clear_model_registry() {
    if let Ok(mut models) = MODEL_REGISTRY.write() {
        models.by_id.clear();
        models.latest.clear();
    }
}

// ============================================================================
// Permission keys
// ============================================================================

/// Resolve permission keys from aliases to nodegroup IDs where possible.
///
/// Builds an alias→nodegroup_id map from the graph (O(N) on nodes, done once
/// at set-time). Keys that are already nodegroup IDs pass through unchanged.
fn resolve_permission_keys(
    graph_id: &str,
    rules: HashMap<String, PermissionRule>,
) -> HashMap<String, PermissionRule> {
    let graph = match get_graph(graph_id) {
        Some(g) => g,
        None => return rules, // No graph registered, pass through as-is
    };

    // Build alias → nodegroup_id mapping from graph nodes
    let alias_to_ng: HashMap<&str, &str> = graph
        .nodes
        .iter()
        .filter_map(|n| match (n.alias.as_deref(), n.nodegroup_id.as_deref()) {
            (Some(alias), Some(ng_id)) if !alias.is_empty() => Some((alias, ng_id)),
            _ => None,
        })
        .collect();

    rules
        .into_iter()
        .map(|(key, rule)| {
            if let Some(&ng_id) = alias_to_ng.get(key.as_str()) {
                (ng_id.to_string(), rule)
            } else {
                // Already a nodegroup ID, or unknown alias — pass through
                (key, rule)
            }
        })
        .collect()
}

// ============================================================================
// List Datatype Registry
// ============================================================================

/// Register a datatype as a list type.
///
/// List types are datatypes where the array IS the value (not multiple items).
/// For these types, arrays should not be iterated during tree-to-tiles conversion.
///
/// Extensions should call this at initialization for their custom list datatypes.
pub fn register_list_datatype(datatype: &str) {
    if let Ok(mut registry) = LIST_DATATYPE_REGISTRY.write() {
        registry.insert(datatype.to_string());
    }
}

/// Check if a datatype is a registered list type.
///
/// Returns true if the datatype's array values should be treated as single values
/// rather than iterated over.
pub fn is_list_datatype(datatype: &str) -> bool {
    LIST_DATATYPE_REGISTRY
        .read()
        .ok()
        .map(|registry| registry.contains(datatype))
        .unwrap_or(false)
}

/// Unregister a list datatype.
pub fn unregister_list_datatype(datatype: &str) -> bool {
    LIST_DATATYPE_REGISTRY
        .write()
        .ok()
        .map(|mut registry| registry.remove(datatype))
        .unwrap_or(false)
}

/// Get all registered list datatypes.
pub fn list_datatypes() -> Vec<String> {
    LIST_DATATYPE_REGISTRY
        .read()
        .ok()
        .map(|registry| registry.iter().cloned().collect())
        .unwrap_or_default()
}

// ============================================================================
// Widget Mapping Registry
// ============================================================================

/// Register a widget mapping for a datatype.
///
/// Extensions should call this at initialization to register their custom
/// datatype-to-widget mappings. This allows `get_default_widget_for_datatype`
/// to find the correct widget for extension datatypes.
///
/// # Example
/// ```ignore
/// // In CLM extension initialization:
/// register_widget_for_datatype("reference", "reference-select-widget");
/// register_widget_for_datatype("reference-list", "reference-multiselect-widget");
/// ```
pub fn register_widget_for_datatype(datatype: &str, widget_name: &str) {
    if let Ok(mut registry) = WIDGET_MAPPING_REGISTRY.write() {
        registry.insert(datatype.to_string(), widget_name.to_string());
    }
}

/// Get the registered widget name for a datatype.
///
/// Returns None if no widget is registered for this datatype.
/// Used by `get_default_widget_for_datatype` to check extension mappings.
pub fn get_widget_for_datatype(datatype: &str) -> Option<String> {
    WIDGET_MAPPING_REGISTRY
        .read()
        .ok()
        .and_then(|registry| registry.get(datatype).cloned())
}

/// Unregister a widget mapping for a datatype.
pub fn unregister_widget_for_datatype(datatype: &str) -> Option<String> {
    WIDGET_MAPPING_REGISTRY
        .write()
        .ok()
        .and_then(|mut registry| registry.remove(datatype))
}

/// Get all registered widget mappings.
pub fn widget_mappings() -> Vec<(String, String)> {
    WIDGET_MAPPING_REGISTRY
        .read()
        .ok()
        .map(|registry| {
            registry
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default()
}

// ============================================================================
// Widget Registry
// ============================================================================

/// Register a widget definition.
///
/// Extensions should call this at initialization to register their custom widgets.
/// This allows `get_default_widget_for_datatype` to find extension widgets.
///
/// # Example
/// ```ignore
/// // In CLM extension initialization:
/// register_widget(RegisteredWidget::new(
///     "10000000-0000-0000-0000-000000000017",
///     "reference-select-widget",
///     "reference",
///     r#"{ "placeholder": "Select a reference" }"#
/// ));
/// ```
pub fn register_widget(widget: RegisteredWidget) {
    if let Ok(mut registry) = WIDGET_REGISTRY.write() {
        registry.insert(widget.name.clone(), widget);
    }
}

/// Get a registered widget by name.
///
/// Returns None if no widget is registered with this name.
pub fn get_registered_widget(name: &str) -> Option<RegisteredWidget> {
    WIDGET_REGISTRY
        .read()
        .ok()
        .and_then(|registry| registry.get(name).cloned())
}

/// Unregister a widget.
pub fn unregister_widget(name: &str) -> Option<RegisteredWidget> {
    WIDGET_REGISTRY
        .write()
        .ok()
        .and_then(|mut registry| registry.remove(name))
}

/// Get all registered widget names.
pub fn registered_widgets() -> Vec<String> {
    WIDGET_REGISTRY
        .read()
        .ok()
        .map(|registry| registry.keys().cloned().collect())
        .unwrap_or_default()
}

// ============================================================================
// Global RDM Cache
// ============================================================================

/// Replace the global RDM cache with the given cache.
pub fn set_global_rdm_cache(cache: RdmCache) {
    if let Ok(mut guard) = GLOBAL_RDM_CACHE.write() {
        *guard = Some(Arc::new(cache));
    }
}

/// Get a clone of the global RDM cache, if set.
///
/// This deep-copies every collection/concept. Prefer [`get_global_rdm_cache_arc`]
/// on hot paths (e.g. per-value marker resolution) — a full copy per call is what
/// made batch reference resolution O(values × cache size).
pub fn get_global_rdm_cache() -> Option<RdmCache> {
    GLOBAL_RDM_CACHE
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().map(|arc| (**arc).clone()))
}

/// Get a cheap `Arc` handle to the global RDM cache, if set — a refcount bump,
/// not a deep copy. Use this wherever the cache is only read (lookups).
pub fn get_global_rdm_cache_arc() -> Option<Arc<RdmCache>> {
    GLOBAL_RDM_CACHE
        .read()
        .ok()
        .and_then(|guard| (*guard).clone())
}

/// Check if a global RDM cache has been set.
pub fn has_global_rdm_cache() -> bool {
    GLOBAL_RDM_CACHE
        .read()
        .ok()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

/// Clear the global RDM cache.
pub fn clear_global_rdm_cache() {
    if let Ok(mut guard) = GLOBAL_RDM_CACHE.write() {
        *guard = None;
    }
}

/// Run a closure with a read reference to the global RDM cache.
/// Returns None if no cache is set or the lock is poisoned.
pub fn with_global_rdm_cache<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&RdmCache) -> R,
{
    GLOBAL_RDM_CACHE
        .read()
        .ok()
        .and_then(|guard| guard.as_deref().map(f))
}

/// Run a closure with a mutable reference to the global RDM cache.
/// Returns None if no cache is set or the lock is poisoned.
///
/// If the cache is shared (an `Arc` handle is outstanding), this copy-on-writes.
pub fn with_global_rdm_cache_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut RdmCache) -> R,
{
    GLOBAL_RDM_CACHE
        .write()
        .ok()
        .and_then(|mut guard| guard.as_mut().map(|arc| f(Arc::make_mut(arc))))
}

/// Run a closure with a mutable reference to the global RDM cache,
/// creating it if it doesn't exist.
pub fn ensure_global_rdm_cache<F, R>(f: F) -> R
where
    F: FnOnce(&mut RdmCache) -> R,
{
    let mut guard = GLOBAL_RDM_CACHE.write().expect("RDM cache lock poisoned");
    if guard.is_none() {
        *guard = Some(Arc::new(RdmCache::default()));
    }
    f(Arc::make_mut(guard.as_mut().unwrap()))
}

/// Add parsed SKOS collections to the global RDM cache (auto-creates if needed).
/// Returns the list of collection IDs added.
pub fn add_to_global_rdm_cache_from_skos(collections: &[SkosCollection]) -> Vec<String> {
    ensure_global_rdm_cache(|cache| cache.add_from_skos_collections(collections))
}

/// Parse SKOS XML and add to the global RDM cache (auto-creates if needed).
/// Returns the list of collection IDs added.
pub fn add_to_global_rdm_cache_from_skos_xml(
    xml_content: &str,
    base_uri: &str,
) -> Result<Vec<String>, String> {
    let collections = parse_skos_to_collections(xml_content, base_uri)?;
    Ok(add_to_global_rdm_cache_from_skos(&collections))
}

/// Add a collection, given as its concepts' JSON, to the global RDM cache
/// (auto-creates it if needed). Every binding's `addCollectionToGlobalRdmCache`.
pub fn add_collection_to_global_rdm_cache(
    collection_id: &str,
    concepts_json: &str,
) -> Result<(), String> {
    ensure_global_rdm_cache(|cache| cache.add_collection_from_json(collection_id, concepts_json))
}

/// Remove a collection from the global RDM cache: whether it was there.
pub fn remove_from_global_rdm_cache(collection_id: &str) -> bool {
    with_global_rdm_cache_mut(|cache| cache.remove_collection(collection_id)).unwrap_or(false)
}

/// A concept's label in the global RDM cache, if it and the cache exist.
pub fn global_rdm_lookup_label(
    collection_id: &str,
    concept_id: &str,
    language: &str,
) -> Option<String> {
    with_global_rdm_cache(|cache| cache.lookup_label(collection_id, concept_id, language)).flatten()
}

/// Whether the global RDM cache has a collection.
pub fn global_rdm_has_collection(collection_id: &str) -> bool {
    with_global_rdm_cache(|cache| cache.has_collection(collection_id)).unwrap_or(false)
}

/// A concept's parent in the global RDM cache, if any.
pub fn global_rdm_get_parent_id(collection_id: &str, concept_id: &str) -> Option<String> {
    with_global_rdm_cache(|cache| cache.get_parent_id(collection_id, concept_id)).flatten()
}

/// The global RDM cache's collection ids (none without a cache).
pub fn global_rdm_get_collection_ids() -> Vec<String> {
    with_global_rdm_cache(|cache| cache.get_collection_ids()).unwrap_or_default()
}

// ============================================================================
// Global Extension Type Registry
// ============================================================================

/// Replace the global extension type registry.
pub fn set_global_extension_registry(registry: ExtensionTypeRegistry) {
    if let Ok(mut guard) = GLOBAL_EXTENSION_REGISTRY.write() {
        *guard = Some(registry);
    }
}

/// Get a reference-counted clone of the global extension type registry, if set.
pub fn get_global_extension_registry() -> Option<ExtensionTypeRegistry> {
    GLOBAL_EXTENSION_REGISTRY
        .read()
        .ok()
        .and_then(|guard| guard.clone())
}

/// Check if a global extension type registry has been set.
pub fn has_global_extension_registry() -> bool {
    GLOBAL_EXTENSION_REGISTRY
        .read()
        .ok()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

/// Clear the global extension type registry.
pub fn clear_global_extension_registry() {
    if let Ok(mut guard) = GLOBAL_EXTENSION_REGISTRY.write() {
        *guard = None;
    }
}

// ============================================================================

/// Mutate the global graph-attached function registry in place — used by
/// provider crates at init (e.g. `alizarin_geo_core::register_functions`).
pub fn with_global_functions_registry_mut<R>(
    f: impl FnOnce(&mut FunctionsRegistry) -> R,
) -> Option<R> {
    GLOBAL_FUNCTIONS_REGISTRY
        .write()
        .ok()
        .map(|mut guard| f(&mut guard))
}

/// Get a clone of the global graph-attached function registry (cheap — `Arc`
/// providers). Returns an empty registry if the lock is poisoned.
pub fn get_global_functions_registry() -> FunctionsRegistry {
    GLOBAL_FUNCTIONS_REGISTRY
        .read()
        .ok()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

/// Reset the global function registry to core's built-ins, dropping any
/// registered providers (mainly for tests).
pub fn clear_global_functions_registry() {
    if let Ok(mut guard) = GLOBAL_FUNCTIONS_REGISTRY.write() {
        *guard = crate::graph::default_functions_registry();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StaticGraph;

    fn create_test_graph(graph_id: &str) -> StaticGraph {
        let json = format!(
            r#"{{
            "graphid": "{}",
            "name": {{"en": "Test Graph"}},
            "nodes": [{{
                "nodeid": "root",
                "name": "Root",
                "datatype": "semantic",
                "graph_id": "{}"
            }}],
            "root": {{
                "nodeid": "root",
                "name": "Root",
                "datatype": "semantic",
                "graph_id": "{}"
            }}
        }}"#,
            graph_id, graph_id, graph_id
        );
        StaticGraph::from_json_string(&json).expect("Failed to create test graph")
    }

    #[test]
    fn test_register_and_get() {
        let graph = create_test_graph("test-graph-1");
        register_graph_owned(graph);

        let retrieved = get_graph("test-graph-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().graphid, "test-graph-1");

        unregister_graph("test-graph-1");
    }

    #[test]
    fn test_is_registered() {
        assert!(!is_graph_registered("nonexistent"));

        let graph = create_test_graph("test-graph-2");
        register_graph_owned(graph);

        assert!(is_graph_registered("test-graph-2"));
        assert!(!is_graph_registered("nonexistent"));

        unregister_graph("test-graph-2");
    }

    #[test]
    fn test_unregister() {
        let graph = create_test_graph("test-graph-3");
        register_graph_owned(graph);

        assert!(is_graph_registered("test-graph-3"));

        let removed = unregister_graph("test-graph-3");
        assert!(removed.is_some());
        assert!(!is_graph_registered("test-graph-3"));
    }

    /// A graph with one aliased value node (`name`) in nodegroup `ng1`.
    fn create_aliased_graph(graph_id: &str) -> StaticGraph {
        let json = serde_json::json!({
            "graphid": graph_id,
            "name": {"en": "Test Graph"},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": graph_id},
            "nodes": [
                {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": graph_id},
                {"nodeid": "ng1", "name": "Name", "alias": "name", "datatype": "string",
                 "nodegroup_id": "ng1", "graph_id": graph_id}
            ],
            "edges": [{"edgeid": "e1", "domainnode_id": "root", "rangenode_id": "ng1",
                       "graph_id": graph_id}],
            "nodegroups": [{"nodegroupid": "ng1", "cardinality": "1"}]
        });
        StaticGraph::from_json_string(&json.to_string()).expect("Failed to create test graph")
    }

    #[test]
    fn a_registered_model_is_shared_and_changed_copy_on_write() {
        let graph_id = "test-model-1";
        let model = register_model(Arc::new(create_aliased_graph(graph_id)), false).unwrap();
        assert!(model.is_built());
        assert!(Arc::ptr_eq(&get_model(graph_id).unwrap(), &model));
        assert!(
            Arc::ptr_eq(&get_graph(graph_id).unwrap(), &model.get_graph_arc()),
            "the registered graph is the model's own"
        );
        assert!(!model.is_nodegroup_permitted("ng1"));

        // An alias key resolves to its nodegroup id, once, for every reader.
        let rules = HashMap::from([("name".to_string(), PermissionRule::Boolean(true))]);
        set_model_permissions(graph_id, rules).unwrap();
        let updated = get_model(graph_id).unwrap();
        assert!(updated.is_nodegroup_permitted("ng1"));
        assert!(updated.get_permission_rule("name").is_none());
        assert!(updated.is_built());
        assert!(
            !model.is_nodegroup_permitted("ng1"),
            "a snapshot taken before the change keeps it"
        );

        unregister_model(graph_id);
        assert!(get_model(graph_id).is_none());
        unregister_graph(graph_id);
    }

    #[test]
    fn two_models_of_one_graph_keep_their_own_permissions() {
        // A-9: a public and a private view of one graph.
        let graph_id = "test-model-3";
        let graph = Arc::new(create_aliased_graph(graph_id));
        let (private, _) = register_model_with_id(Arc::clone(&graph), true).unwrap();
        let (public, _) = register_model_with_id(graph, false).unwrap();
        assert_ne!(private, public);

        set_model_permissions(
            &public,
            HashMap::from([("name".to_string(), PermissionRule::Boolean(false))]),
        )
        .unwrap();
        assert!(get_model(&private).unwrap().is_nodegroup_permitted("ng1"));
        assert!(!get_model(&public).unwrap().is_nodegroup_permitted("ng1"));
        // The graph id names the latest.
        assert!(Arc::ptr_eq(
            &get_model(graph_id).unwrap(),
            &get_model(&public).unwrap()
        ));

        // Unregistering one model leaves the other; the graph id, then, none.
        unregister_model(&public);
        assert!(get_model(&private).is_some());
        assert!(get_model(graph_id).is_none());
        unregister_model(graph_id);
        unregister_model(&private);
        assert!(get_model(&private).is_none());
        unregister_graph(graph_id);
    }

    #[test]
    fn a_failed_or_impossible_update_changes_nothing() {
        let err = update_model("test-model-none", |_| Ok(())).unwrap_err();
        assert!(err.contains("no model registered"), "{err}");

        let graph_id = "test-model-2";
        register_model(Arc::new(create_aliased_graph(graph_id)), true).unwrap();
        let before = get_model(graph_id).unwrap();
        let err = update_model(graph_id, |model| {
            model.set_default_allow(false);
            Err::<(), _>("nope".to_string())
        })
        .unwrap_err();
        assert_eq!(err, "nope");
        assert!(Arc::ptr_eq(&before, &get_model(graph_id).unwrap()));

        unregister_model(graph_id);
        unregister_graph(graph_id);
    }
}
