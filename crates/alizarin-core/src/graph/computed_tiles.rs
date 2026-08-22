//! Computed tile layer support: JIT tile generation via `functions_x_graphs`.

use serde::{Deserialize, Serialize};

use super::descriptors::COMPUTE_TILES_FUNCTION_ID;
use super::graph_lookup::GraphLookup;
use super::static_graph::StaticGraph;
use super::tile::StaticTile;

/// Configuration for a compute-tiles function, deserialized from the
/// `config` field of a `functions_x_graphs` entry whose `function_id`
/// is [`COMPUTE_TILES_FUNCTION_ID`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeTilesConfig {
    /// Provider dispatch key (e.g. "gramadan"). A JS-registered callback
    /// keyed by this name generates the tiles.
    pub provider: String,
    /// Which nodegroup this function fills.
    pub nodegroup: String,
    /// Layer ID for the fast first test: is the resource present in this
    /// layer? Usually the computed layer's own ID ("am I a member here?").
    pub member_of: String,
    #[serde(default)]
    pub cache: bool,
}

/// Extract compute-tiles function configs from every `functions_x_graphs`
/// declaration visible through the lookup. For a [`LayeredGraph`] this includes
/// an overlay (computed layer) whose fxg the base graph does not carry.
pub fn compute_tiles_functions(graph: &dyn GraphLookup) -> Vec<ComputeTilesConfig> {
    graph
        .functions_x_graphs()
        .into_iter()
        .filter(|f| f.function_id == COMPUTE_TILES_FUNCTION_ID)
        .filter_map(|f| serde_json::from_value(f.config.clone()).ok())
        .collect()
}

/// Substrate-agnostic interface for tile generation. Each provider
/// (e.g. gramadan-wasm) implements this to produce tiles on demand.
pub trait ComputeTilesProvider {
    fn generate_tiles(
        &self,
        resource_id: &str,
        config: &ComputeTilesConfig,
    ) -> Result<Vec<StaticTile>, String>;
}

/// Run compute-tiles functions declared by the graph, merging results
/// into `tiles`.
///
/// `is_member` checks whether `resource_id` is present in a layer
/// identified by `layer_id` — each substrate provides its own O(1)
/// implementation.
///
/// Computed tiles are only generated for nodegroups NOT already present
/// in `tiles` (attested data wins).
pub fn apply_compute_tiles(
    tiles: &mut Vec<StaticTile>,
    graph: &StaticGraph,
    resource_id: &str,
    is_member: &dyn Fn(&str) -> bool,
    provider: &dyn ComputeTilesProvider,
) {
    let functions = compute_tiles_functions(graph);
    for func in &functions {
        if !is_member(&func.member_of) {
            continue;
        }
        if tiles.iter().any(|t| t.nodegroup_id == func.nodegroup) {
            continue;
        }
        if let Ok(computed) = provider.generate_tiles(resource_id, func) {
            tiles.extend(computed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
    fn extract_compute_tiles_functions() {
        let graph = test_graph(vec![compute_fxg()]);
        let funcs = compute_tiles_functions(&graph);
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
        assert!(compute_tiles_functions(&graph).is_empty());
    }

    #[test]
    fn skips_bad_config() {
        let graph = test_graph(vec![serde_json::json!({
            "id": "fxg-bad",
            "function_id": COMPUTE_TILES_FUNCTION_ID,
            "graph_id": "test-graph",
            "config": { "not_a_valid_config": true }
        })]);
        assert!(compute_tiles_functions(&graph).is_empty());
    }

    struct StubProvider;
    impl ComputeTilesProvider for StubProvider {
        fn generate_tiles(
            &self,
            resource_id: &str,
            config: &ComputeTilesConfig,
        ) -> Result<Vec<StaticTile>, String> {
            Ok(vec![StaticTile {
                data: HashMap::new(),
                nodegroup_id: config.nodegroup.clone(),
                resourceinstance_id: resource_id.to_string(),
                tileid: None,
                parenttile_id: None,
                provisionaledits: None,
                sortorder: None,
            }])
        }
    }

    #[test]
    fn apply_skips_non_member() {
        let graph = test_graph(vec![compute_fxg()]);
        let mut tiles = Vec::new();
        apply_compute_tiles(&mut tiles, &graph, "r-001", &|_| false, &StubProvider);
        assert!(tiles.is_empty());
    }

    #[test]
    fn apply_generates_for_member() {
        let graph = test_graph(vec![compute_fxg()]);
        let mut tiles = Vec::new();
        apply_compute_tiles(&mut tiles, &graph, "r-001", &|_| true, &StubProvider);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].nodegroup_id, "forms");
        assert_eq!(tiles[0].resourceinstance_id, "r-001");
    }

    #[test]
    fn apply_skips_when_nodegroup_attested() {
        let graph = test_graph(vec![compute_fxg()]);
        let mut tiles = vec![StaticTile {
            data: HashMap::new(),
            nodegroup_id: "forms".to_string(),
            resourceinstance_id: "r-001".to_string(),
            tileid: None,
            parenttile_id: None,
            provisionaledits: None,
            sortorder: None,
        }];
        apply_compute_tiles(&mut tiles, &graph, "r-001", &|_| true, &StubProvider);
        assert_eq!(tiles.len(), 1, "attested tile should prevent computed tile");
    }
}
