//! Computed tile layer support: JIT tile generation via `functions_x_graphs`.

use std::collections::BTreeMap;

use super::descriptors::COMPUTE_TILES_FUNCTION_ID;
use super::graph_lookup::GraphLookup;
#[cfg(test)]
use super::static_graph::StaticGraph;

/// Defined in the extension API, so an extension's derive provider needs no
/// core to read its declaration.
pub use alizarin_extension_api::{DeriveConfig, DeriveMode, ResolvedNode};

/// `config` with its [`nodes`](DeriveConfig::nodes) resolved against `graph`,
/// by alias or else by node id: an error naming the first that isn't there.
pub fn resolve_derive_nodes(
    config: &DeriveConfig,
    graph: &dyn GraphLookup,
) -> Result<DeriveConfig, String> {
    let mut resolved = BTreeMap::new();
    for (role, name) in &config.nodes {
        let node = graph
            .get_node_by_alias(name)
            .or_else(|| graph.get_node_by_id(name))
            .ok_or_else(|| {
                format!(
                    "derive function {}: role '{role}' names node '{name}', which the \
                     graph doesn't have",
                    config.provider
                )
            })?;
        resolved.insert(
            role.clone(),
            ResolvedNode {
                nodeid: node.nodeid.clone(),
                nodegroup_id: node.nodegroup_id.clone(),
                datatype: node.datatype.clone(),
                alias: node.alias.clone(),
            },
        );
    }
    Ok(DeriveConfig {
        resolved,
        ..config.clone()
    })
}

/// The former name of [`DeriveConfig`], from before the Derive interface
/// replaced compute-tiles providers.
#[deprecated(note = "renamed DeriveConfig")]
pub type ComputeTilesConfig = DeriveConfig;

/// The former name of [`derive_functions`].
#[deprecated(note = "renamed derive_functions")]
pub fn compute_tiles_functions(graph: &dyn GraphLookup) -> Vec<DeriveConfig> {
    derive_functions(graph)
}

/// The Derive declarations (compute-tiles functions) of every
/// `functions_x_graphs` entry visible through the lookup, with their [`nodes`]
/// (DeriveConfig::nodes) resolved. For a [`LayeredGraph`] this includes
/// an overlay (computed layer) whose fxg the base graph does not carry. A
/// declaration that doesn't parse, or names a node the graph lacks, is left out
/// (the stored-mode reader, `stored_derive_functions`, errors instead).
pub fn derive_functions(graph: &dyn GraphLookup) -> Vec<DeriveConfig> {
    graph
        .functions_x_graphs()
        .into_iter()
        .filter(|f| f.function_id == COMPUTE_TILES_FUNCTION_ID)
        .filter_map(|f| serde_json::from_value::<DeriveConfig>(f.config.clone()).ok())
        .filter_map(|config| resolve_derive_nodes(&config, graph).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_graph(functions_x_graphs: Vec<serde_json::Value>) -> StaticGraph {
        let json = serde_json::json!({
            "graphid": "test-graph",
            "name": {"en": "Test"},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "test-graph"},
            "nodes": [
                {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "test-graph"},
            ],
            "edges": [],
            "nodegroups": [],
            "cards": [],
            "cards_x_nodes_x_widgets": [],
            "functions_x_graphs": functions_x_graphs,
        });
        serde_json::from_value(json).expect("test graph should deserialize")
    }

    fn compute_fxg() -> serde_json::Value {
        serde_json::json!({
            "id": "fxg-001",
            "function_id": COMPUTE_TILES_FUNCTION_ID,
            "graph_id": "test-graph",
            "config": {
                "provider": "test",
                "nodegroup": "forms",
                "member_of": "test-layer",
                "cache": true,
            }
        })
    }

    #[test]
    fn extract_derive_functions() {
        let graph = test_graph(vec![compute_fxg()]);
        let funcs = derive_functions(&graph);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].provider, "test");
        assert_eq!(funcs[0].nodegroup, "forms");
        assert_eq!(funcs[0].member_of, "test-layer");
        assert!(funcs[0].cache);
    }

    #[test]
    fn skips_non_compute_functions() {
        let graph = test_graph(vec![serde_json::json!({
            "id": "fxg-desc",
            "function_id": "60000000-0000-0000-0000-000000000001",
            "graph_id": "test-graph",
            "config": { "descriptor_types": {} }
        })]);
        assert!(derive_functions(&graph).is_empty());
    }

    #[test]
    fn skips_bad_config() {
        let graph = test_graph(vec![serde_json::json!({
            "id": "fxg-bad",
            "function_id": COMPUTE_TILES_FUNCTION_ID,
            "graph_id": "test-graph",
            "config": { "not_a_valid_config": true }
        })]);
        assert!(derive_functions(&graph).is_empty());
    }
}
