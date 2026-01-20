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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StaticGraph;

    fn create_test_graph(graph_id: &str) -> StaticGraph {
        let json = format!(r#"{{
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
        }}"#, graph_id, graph_id, graph_id);
        StaticGraph::from_json_string(&json).expect("Failed to create test graph")
    }

    #[test]
    fn test_register_and_get() {
        clear_registry();

        let graph = create_test_graph("test-graph-1");
        register_graph_owned(graph);

        let retrieved = get_graph("test-graph-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().graphid, "test-graph-1");

        clear_registry();
    }

    #[test]
    fn test_is_registered() {
        clear_registry();

        assert!(!is_graph_registered("nonexistent"));

        let graph = create_test_graph("test-graph-2");
        register_graph_owned(graph);

        assert!(is_graph_registered("test-graph-2"));
        assert!(!is_graph_registered("nonexistent"));

        clear_registry();
    }

    #[test]
    fn test_unregister() {
        clear_registry();

        let graph = create_test_graph("test-graph-3");
        register_graph_owned(graph);

        assert!(is_graph_registered("test-graph-3"));

        let removed = unregister_graph("test-graph-3");
        assert!(removed.is_some());
        assert!(!is_graph_registered("test-graph-3"));

        clear_registry();
    }
}
