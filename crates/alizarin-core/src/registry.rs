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

    /// Global RDM (Reference Data Manager) cache for concept collections.
    /// Used by label resolution, display rendering, and other functions that
    /// need concept lookups without passing a cache explicitly.
    static ref GLOBAL_RDM_CACHE: RwLock<Option<RdmCache>> = RwLock::new(None);

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

    /// Model permissions registry.
    /// Mirrors WASM's MODEL_REGISTRY pattern for NAPI/Python backends, which
    /// construct instance wrappers independently of model wrappers.
    /// Stores (default_allow, permitted_nodegroups) per graph_id so instance
    /// wrappers can inherit the model's permission state at construction time.
    static ref MODEL_PERMISSIONS_REGISTRY: RwLock<HashMap<String, ModelPermissions>> =
        RwLock::new(HashMap::new());
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
// Model Permissions Registry
// ============================================================================

/// Cached model permission state for instance wrapper construction.
///
/// NAPI/Python backends construct instance wrappers independently of model
/// wrappers, so they need a way to look up the model's permission state.
/// WASM avoids this via a thread-local MODEL_REGISTRY that instance wrappers
/// reference directly.
#[derive(Clone, Debug)]
pub struct ModelPermissions {
    pub default_allow: bool,
    pub permitted_nodegroups: HashMap<String, PermissionRule>,
}

/// Register model permissions for a graph.
pub fn register_model_permissions(graph_id: &str, perms: ModelPermissions) {
    if let Ok(mut registry) = MODEL_PERMISSIONS_REGISTRY.write() {
        registry.insert(graph_id.to_string(), perms);
    }
}

/// Get model permissions for a graph.
pub fn get_model_permissions(graph_id: &str) -> Option<ModelPermissions> {
    MODEL_PERMISSIONS_REGISTRY
        .read()
        .ok()
        .and_then(|registry| registry.get(graph_id).cloned())
}

/// Update just the default_allow for a graph's model permissions.
pub fn update_model_default_allow(graph_id: &str, default_allow: bool) {
    if let Ok(mut registry) = MODEL_PERMISSIONS_REGISTRY.write() {
        if let Some(entry) = registry.get_mut(graph_id) {
            entry.default_allow = default_allow;
        }
    }
}

/// Update just the permitted_nodegroups for a graph's model permissions.
///
/// Keys may be aliases (e.g. "images") or nodegroup UUIDs. Any alias keys
/// are resolved to nodegroup IDs using the graph from GRAPH_REGISTRY so that
/// downstream tile filtering (which matches on `tile.nodegroup_id`) works
/// without per-tile alias resolution.
pub fn update_model_permitted_nodegroups(graph_id: &str, rules: HashMap<String, PermissionRule>) {
    let resolved = resolve_permission_keys(graph_id, rules);
    if let Ok(mut registry) = MODEL_PERMISSIONS_REGISTRY.write() {
        if let Some(entry) = registry.get_mut(graph_id) {
            entry.permitted_nodegroups = resolved;
        }
    }
}

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

/// Unregister model permissions for a graph.
pub fn unregister_model_permissions(graph_id: &str) {
    if let Ok(mut registry) = MODEL_PERMISSIONS_REGISTRY.write() {
        registry.remove(graph_id);
    }
}

/// Clear all model permissions from the registry.
pub fn clear_model_permissions_registry() {
    if let Ok(mut registry) = MODEL_PERMISSIONS_REGISTRY.write() {
        registry.clear();
    }
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
        *guard = Some(cache);
    }
}

/// Get a clone of the global RDM cache, if set.
pub fn get_global_rdm_cache() -> Option<RdmCache> {
    GLOBAL_RDM_CACHE.read().ok().and_then(|guard| guard.clone())
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
        .and_then(|guard| guard.as_ref().map(f))
}

/// Run a closure with a mutable reference to the global RDM cache.
/// Returns None if no cache is set or the lock is poisoned.
pub fn with_global_rdm_cache_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut RdmCache) -> R,
{
    GLOBAL_RDM_CACHE
        .write()
        .ok()
        .and_then(|mut guard| guard.as_mut().map(f))
}

/// Run a closure with a mutable reference to the global RDM cache,
/// creating it if it doesn't exist.
pub fn ensure_global_rdm_cache<F, R>(f: F) -> R
where
    F: FnOnce(&mut RdmCache) -> R,
{
    let mut guard = GLOBAL_RDM_CACHE.write().expect("RDM cache lock poisoned");
    if guard.is_none() {
        *guard = Some(RdmCache::default());
    }
    f(guard.as_mut().unwrap())
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

    #[test]
    fn test_model_permissions_register_and_get() {
        let perms = ModelPermissions {
            default_allow: false,
            permitted_nodegroups: {
                let mut m = HashMap::new();
                m.insert("ng1".to_string(), PermissionRule::Boolean(true));
                m.insert("ng2".to_string(), PermissionRule::Boolean(false));
                m
            },
        };
        register_model_permissions("test-perms-1", perms);

        let retrieved = get_model_permissions("test-perms-1");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert!(!retrieved.default_allow);
        assert_eq!(retrieved.permitted_nodegroups.len(), 2);

        unregister_model_permissions("test-perms-1");
        assert!(get_model_permissions("test-perms-1").is_none());
    }

    #[test]
    fn test_model_permissions_update_default_allow() {
        let perms = ModelPermissions {
            default_allow: true,
            permitted_nodegroups: HashMap::new(),
        };
        register_model_permissions("test-perms-2", perms);

        update_model_default_allow("test-perms-2", false);
        let retrieved = get_model_permissions("test-perms-2").unwrap();
        assert!(!retrieved.default_allow);

        unregister_model_permissions("test-perms-2");
    }

    #[test]
    fn test_model_permissions_update_nodegroups() {
        let perms = ModelPermissions {
            default_allow: false,
            permitted_nodegroups: HashMap::new(),
        };
        register_model_permissions("test-perms-3", perms);

        let mut rules = HashMap::new();
        rules.insert("ng-x".to_string(), PermissionRule::Boolean(true));
        update_model_permitted_nodegroups("test-perms-3", rules);

        let retrieved = get_model_permissions("test-perms-3").unwrap();
        assert_eq!(retrieved.permitted_nodegroups.len(), 1);

        unregister_model_permissions("test-perms-3");
    }
}
