/// Graph Registry
///
/// A thread-safe global registry for storing graphs by graph_id.
/// Used by batch_merge_resources and other functions that need to look up
/// graphs without passing them explicitly.
///
/// Uses RwLock for thread-safe access, allowing multiple concurrent readers
/// or exclusive write access. This works correctly with rayon's parallel iterators.
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::extension_type_registry::{ExtensionError, ExtensionTypeHandler, ExtensionTypeRegistry};
use crate::StaticGraph;

lazy_static::lazy_static! {
    /// Registry mapping graph_id -> StaticGraph
    /// Uses Arc for cheap cloning when retrieving graphs
    /// Uses RwLock for thread-safe access across parallel threads
    static ref GRAPH_REGISTRY: RwLock<HashMap<String, Arc<StaticGraph>>> =
        RwLock::new(HashMap::new());

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

    /// Global extension type handler registry.
    /// Python/WASM register handlers here at extension init time.
    /// Core code (e.g., build_descriptors) reads from here to render display values.
    static ref EXTENSION_TYPE_REGISTRY: RwLock<ExtensionTypeRegistry> =
        RwLock::new(ExtensionTypeRegistry::new());
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
// Extension Type Handler Registry
// ============================================================================

/// Register an extension type handler globally.
///
/// Called by Python/WASM when extensions register their type handlers.
/// Core code (e.g., `build_descriptors`) uses `render_extension_display`
/// to get display strings for extension datatypes.
pub fn register_extension_type_handler(datatype: &str, handler: Arc<dyn ExtensionTypeHandler>) {
    if let Ok(mut registry) = EXTENSION_TYPE_REGISTRY.write() {
        registry.register(datatype, handler);
    }
}

/// Unregister an extension type handler.
pub fn unregister_extension_type_handler(datatype: &str) {
    if let Ok(mut registry) = EXTENSION_TYPE_REGISTRY.write() {
        registry.unregister(datatype);
    }
}

/// Render a display value using a registered extension handler.
///
/// Returns `Ok(Some(string))` if an extension handler produced a display string,
/// `Ok(None)` if no handler is registered for this datatype,
/// `Err` if the handler failed.
pub fn render_extension_display(
    datatype: &str,
    tile_data: &serde_json::Value,
    language: &str,
) -> Result<Option<String>, ExtensionError> {
    match EXTENSION_TYPE_REGISTRY.read() {
        Ok(registry) => registry.render_display(datatype, tile_data, language),
        Err(_) => Ok(None),
    }
}

/// Check if an extension type handler is registered for a datatype.
pub fn has_extension_type_handler(datatype: &str) -> bool {
    EXTENSION_TYPE_REGISTRY
        .read()
        .ok()
        .map(|registry| registry.has(datatype))
        .unwrap_or(false)
}

/// Get all registered extension type handler datatype names.
pub fn list_extension_type_handlers() -> Vec<String> {
    EXTENSION_TYPE_REGISTRY
        .read()
        .ok()
        .map(|registry| registry.list().into_iter().map(String::from).collect())
        .unwrap_or_default()
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
}
