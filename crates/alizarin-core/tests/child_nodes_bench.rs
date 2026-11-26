//! How much a full ORM traversal spends building child-node maps (sorted,
//! `ChildNodes`), against a hashed or a cached map. Not a correctness
//! test: ignored by default, timings printed.
//!
//!   cargo test --release -p alizarin-core --test child_nodes_bench -- --ignored --nocapture
//!
//! The traversal is the one every binding's ORM does: from the root, each
//! node's semantic children (`get_all_semantic_child_values`), recursing into
//! semantic nodes, over a resource with a tile in every nodegroup of the
//! largest model in tests/data (Heritage Item). `get_child_nodes` builds a
//! fresh alias map on each call, and `get_all_semantic_child_values` calls it
//! once, then once more per child (through `get_semantic_child_value`).

use std::collections::HashMap;
use std::hint::black_box;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use alizarin_core::{
    ChildNodes, GraphModelAccess, ModelAccess, ResourceInstanceWrapperCore, SemanticChildResult,
    StaticGraph, StaticNode, StaticResourceMetadata, StaticTile,
};

const RESOURCE_ID: &str = "bench-resource-1";
/// Traversals timed per measurement: as many resources of this model.
const RESOURCES: usize = 200;
/// Rounds of building every node's child map, per measurement.
const MAP_ROUNDS: usize = 2_000;

fn load_graph() -> StaticGraph {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data/models/Heritage Item.json");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut graph: StaticGraph = serde_json::from_value(json["graph"][0].clone()).unwrap();
    graph.build_indices();
    graph
}

/// A tile in every nodegroup, nested under its parent nodegroup's tile, with a
/// value in every node: the fullest resource the model admits with one tile
/// per nodegroup.
fn full_resource_tiles(graph: &StaticGraph) -> Vec<StaticTile> {
    let tile_id = |ng: &str| format!("tile-{ng}");
    graph
        .nodegroups_slice()
        .iter()
        .map(|ng| {
            let mut tile = StaticTile::new_empty(ng.nodegroupid.clone());
            tile.resourceinstance_id = RESOURCE_ID.to_string();
            tile.tileid = Some(tile_id(&ng.nodegroupid));
            tile.parenttile_id = ng.parentnodegroup_id.as_deref().map(tile_id);
            for node in graph.nodes_slice() {
                if node.nodegroup_id.as_deref() == Some(ng.nodegroupid.as_str())
                    && node.datatype != "semantic"
                {
                    tile.data
                        .insert(node.nodeid.clone(), serde_json::json!("value"));
                }
            }
            tile
        })
        .collect()
}

fn resource(graph: &StaticGraph) -> ResourceInstanceWrapperCore {
    let mut core = ResourceInstanceWrapperCore::new(graph.graphid.clone());
    core.resource_instance = Some(StaticResourceMetadata {
        descriptors: Default::default(),
        graph_id: graph.graphid.clone(),
        name: RESOURCE_ID.to_string(),
        resourceinstanceid: RESOURCE_ID.to_string(),
        publication_id: None,
        principaluser_id: None,
        legacyid: None,
        graph_publication_id: None,
        createdtime: None,
        lastmodified: None,
        resource_instance_lifecycle_state_id: None,
        lifecycle_state_conflict: Vec::new(),
    });
    core.load_tiles(full_resource_tiles(graph));
    core
}

/// What one traversal did: nodes visited, and child maps built.
#[derive(Default)]
struct Walk {
    visits: usize,
    map_builds: usize,
}

/// Walk `node`'s semantic children, recursing into semantic ones.
fn walk(
    core: &ResourceInstanceWrapperCore,
    model: &dyn ModelAccess,
    node: &StaticNode,
    tile: Option<&Arc<StaticTile>>,
    stats: &mut Walk,
) {
    let tile_id = tile.and_then(|t| t.tileid.clone());
    let children = core
        .get_all_semantic_child_values(
            tile_id.as_ref(),
            &node.nodeid,
            node.nodegroup_id.as_ref(),
            model,
        )
        .expect("traversal");
    // One build here, and one per child in get_semantic_child_value.
    stats.map_builds += 1 + children.len();
    for value in children.values() {
        let values = match value {
            SemanticChildResult::Single(v) => vec![v],
            SemanticChildResult::List(list) => list.values.iter().collect(),
            SemanticChildResult::Empty => vec![],
        };
        for v in values {
            stats.visits += 1;
            if v.node.datatype == "semantic" {
                walk(core, model, &v.node, v.tile.as_ref(), stats);
            }
        }
    }
}

/// The best of three timings of `f`.
fn best_of_three(mut f: impl FnMut()) -> Duration {
    (0..3)
        .map(|_| {
            let start = Instant::now();
            f();
            start.elapsed()
        })
        .min()
        .unwrap()
}

/// `get_child_nodes`'s own body, into a map of the caller's choosing.
fn child_map<M: Default + Extend<(String, Arc<StaticNode>)>>(
    model: &dyn ModelAccess,
    node_id: &str,
) -> M {
    let edges = model.get_edges().unwrap();
    let nodes = model.get_nodes().unwrap();
    let mut map = M::default();
    map.extend(
        edges
            .get(node_id)
            .into_iter()
            .flatten()
            .filter_map(|id| nodes.get(id))
            .filter_map(|n| match n.alias.as_deref() {
                Some(alias) if !alias.is_empty() => Some((alias.to_string(), Arc::clone(n))),
                _ => None,
            }),
    );
    map
}

#[test]
#[ignore = "benchmark: run with --release -- --ignored --nocapture"]
fn child_node_maps_in_a_full_traversal() {
    let graph = load_graph();
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let core = resource(&graph);
    let root = graph.root.clone();
    // The nodes a traversal builds maps for: those with children.
    let node_ids: Vec<String> = graph
        .nodes_slice()
        .iter()
        .map(|n| n.nodeid.clone())
        .filter(|id| !child_map::<Vec<_>>(&model, id).is_empty())
        .collect();

    let mut stats = Walk::default();
    walk(&core, &model, &root, None, &mut stats);
    assert!(stats.visits > 0, "the traversal reached no values");

    let traversal = best_of_three(|| {
        for _ in 0..RESOURCES {
            let mut s = Walk::default();
            walk(&core, &model, &root, None, &mut s);
            black_box(s.visits);
        }
    }) / RESOURCES as u32;

    let per_round = |d: Duration| d / (MAP_ROUNDS * node_ids.len()) as u32;
    // The same build into each map type, then core's own.
    let hash_build = per_round(best_of_three(|| {
        for _ in 0..MAP_ROUNDS {
            for id in &node_ids {
                black_box(child_map::<HashMap<_, _>>(&model, id));
            }
        }
    }));
    let btree_build = per_round(best_of_three(|| {
        for _ in 0..MAP_ROUNDS {
            for id in &node_ids {
                black_box(child_map::<ChildNodes>(&model, id));
            }
        }
    }));
    let core_build = per_round(best_of_three(|| {
        for _ in 0..MAP_ROUNDS {
            for id in &node_ids {
                black_box(ModelAccess::get_child_nodes(&model, id).unwrap());
            }
        }
    }));
    let cache: HashMap<String, Arc<ChildNodes>> = node_ids
        .iter()
        .map(|id| (id.clone(), Arc::new(child_map(&model, id))))
        .collect();
    let cached = per_round(best_of_three(|| {
        for _ in 0..MAP_ROUNDS {
            for id in &node_ids {
                black_box(Arc::clone(&cache[id]));
            }
        }
    }));

    let in_maps = core_build * stats.map_builds as u32;
    println!(
        "\nHeritage Item: {} nodes, {} nodegroups; one resource, a tile per nodegroup",
        graph.nodes_slice().len(),
        graph.nodegroups_slice().len()
    );
    println!(
        "full traversal:            {traversal:>10.2?} per resource ({} values, {} child-map builds)",
        stats.visits, stats.map_builds
    );
    println!(
        "child maps (over the {} nodes with children):",
        node_ids.len()
    );
    println!("child map, HashMap build:  {hash_build:>10.2?} per call");
    println!("child map, BTreeMap build: {btree_build:>10.2?} per call");
    println!("get_child_nodes (core):    {core_build:>10.2?} per call");
    println!("child map, cached Arc:     {cached:>10.2?} per call");
    println!(
        "child-map builds in a traversal: ~{in_maps:.2?} of {traversal:.2?} ({:.0}%)",
        100.0 * in_maps.as_secs_f64() / traversal.as_secs_f64()
    );
    println!(
        "at a million resources: traversal ~{:.0?}, of which child maps ~{:.0?}",
        traversal * 1_000_000,
        in_maps * 1_000_000
    );
}
