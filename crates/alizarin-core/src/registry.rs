/// Graph Registry
///
/// A simple thread-local registry for storing graphs by graph_id.
/// Used by batch_merge_resources and other functions that need to look up
/// graphs without passing them explicitly.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::StaticGraph;

thread_local! {
    /// Registry mapping graph_id -> StaticGraph
    /// Uses Arc for cheap cloning when retrieving graphs
    static GRAPH_REGISTRY: RefCell<HashMap<String, Arc<StaticGraph>>> =
        RefCell::new(HashMap::new());
}

/// Register a graph in the registry
pub fn register_graph(graph_id: &str, graph: Arc<StaticGraph>) {
    GRAPH_REGISTRY.with(|r| {
        r.borrow_mut().insert(graph_id.to_string(), graph);
    });
}

/// Register a graph from an owned StaticGraph (wraps in Arc)
pub fn register_graph_owned(graph: StaticGraph) {
    let graph_id = graph.graph_id().to_string();
    register_graph(&graph_id, Arc::new(graph));
}

/// Get a graph from the registry by graph_id
pub fn get_graph(graph_id: &str) -> Option<Arc<StaticGraph>> {
    GRAPH_REGISTRY.with(|r| {
        r.borrow().get(graph_id).cloned()
    })
}

/// Check if a graph is registered
pub fn is_graph_registered(graph_id: &str) -> bool {
    GRAPH_REGISTRY.with(|r| {
        r.borrow().contains_key(graph_id)
    })
}

/// Unregister a graph from the registry
pub fn unregister_graph(graph_id: &str) -> Option<Arc<StaticGraph>> {
    GRAPH_REGISTRY.with(|r| {
        r.borrow_mut().remove(graph_id)
    })
}

/// Clear all graphs from the registry
pub fn clear_registry() {
    GRAPH_REGISTRY.with(|r| {
        r.borrow_mut().clear();
    });
}

/// Get the number of registered graphs
pub fn registry_size() -> usize {
    GRAPH_REGISTRY.with(|r| {
        r.borrow().len()
    })
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
