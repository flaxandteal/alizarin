//! M0.4 sparse-hydration gate (Static-Assets-Implementation-Plan).
//!
//! Verifies that a resource populated from a SUBSET of its tiles behaves
//! correctly: enumeration does not assume completeness, absent nodegroups
//! (including cardinality-1) raise no spurious errors, tree building from a
//! subset yields absent keys rather than errors, and later backfill merges
//! without duplicating already-loaded values.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use alizarin_core::{
    resource_tiles_to_tree, GraphModelAccess, ResourceInstanceWrapperCore, StaticGraph,
    StaticResourceMetadata, StaticTile,
};

const RESOURCE_ID: &str = "sparse-test-resource-1";

fn load_group_graph() -> StaticGraph {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let test_file = workspace_root.join("tests/data/models/Group.json");
    let json_str = fs::read_to_string(&test_file).expect("Failed to read Group.json");
    let json: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse Group.json");
    let mut graph: StaticGraph =
        serde_json::from_value(json["graph"][0].clone()).expect("Failed to deserialize");
    graph.build_indices();
    graph
}

fn nodegroup_id_for_alias(graph: &StaticGraph, alias: &str) -> String {
    graph
        .nodes_slice()
        .iter()
        .find(|n| n.alias.as_deref() == Some(alias))
        .and_then(|n| n.nodegroup_id.clone())
        .unwrap_or_else(|| panic!("No nodegroup for alias {alias}"))
}

fn make_tile(graph: &StaticGraph, nodegroup_alias: &str, tile_id: &str) -> StaticTile {
    let ng_id = nodegroup_id_for_alias(graph, nodegroup_alias);
    let mut tile = StaticTile::new_empty(ng_id);
    tile.resourceinstance_id = RESOURCE_ID.to_string();
    tile.tileid = Some(tile_id.to_string());
    tile
}

/// Tile for the `basic_info` nodegroup carrying a `name` value.
fn basic_info_tile(graph: &StaticGraph) -> StaticTile {
    let mut tile = make_tile(graph, "basic_info", "tile-basic-info-1");
    let name_node = graph
        .nodes_slice()
        .iter()
        .find(|n| n.alias.as_deref() == Some("name"))
        .expect("name node");
    tile.data.insert(
        name_node.nodeid.clone(),
        serde_json::json!({"en": "Sparse Group"}),
    );
    tile
}

/// Tile for the `statement` nodegroup carrying a `description` value.
fn statement_tile(graph: &StaticGraph) -> StaticTile {
    let mut tile = make_tile(graph, "statement", "tile-statement-1");
    let desc_node = graph
        .nodes_slice()
        .iter()
        .find(|n| n.alias.as_deref() == Some("description"))
        .expect("description node");
    tile.data.insert(
        desc_node.nodeid.clone(),
        serde_json::json!("A partially hydrated group"),
    );
    tile
}

fn metadata(graph: &StaticGraph) -> StaticResourceMetadata {
    StaticResourceMetadata {
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
    }
}

fn make_core(graph: &StaticGraph, model: &GraphModelAccess) -> ResourceInstanceWrapperCore {
    let mut core = ResourceInstanceWrapperCore::new(graph.graphid.clone());
    core.set_cached_indices(model);
    core.resource_instance = Some(metadata(graph));
    core
}

/// Populating from a tile subset succeeds, yields values for the loaded
/// nodegroup, and does not invent values for (or error on) absent ones —
/// including an absent cardinality-1 nodegroup (`group_type`).
#[test]
fn populate_with_tile_subset_loads_only_present() {
    let graph = load_group_graph();
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let mut core = make_core(&graph, &model);

    // Load ONLY the basic_info tile; request three nodegroups.
    core.load_tiles(vec![basic_info_tile(&graph)]);

    let requested: Vec<String> = ["basic_info", "statement", "group_type"]
        .iter()
        .map(|a| nodegroup_id_for_alias(&graph, a))
        .collect();

    let result = core
        .populate(false, &requested, "group", &model)
        .expect("populate over a tile subset must not error");

    // Loaded nodegroup produced its value.
    let name_list = result
        .values
        .get("name")
        .expect("name should be populated from the loaded basic_info tile");
    assert_eq!(
        name_list.values.len(),
        1,
        "exactly one name value from one basic_info tile"
    );

    // Absent nodegroups: no truthy values, no phantom entries.
    let desc_truthy = result
        .all_values_map
        .get("description")
        .copied()
        .flatten()
        .unwrap_or(false);
    assert!(
        !desc_truthy,
        "description must not be truthy when its tile was never loaded"
    );
    let group_type_truthy = result
        .all_values_map
        .get("group_type")
        .copied()
        .flatten()
        .unwrap_or(false);
    assert!(
        !group_type_truthy,
        "absent cardinality-1 nodegroup (group_type) must not be truthy"
    );
}

/// A resource with NO tiles at all populates without error (fully sparse).
#[test]
fn populate_with_no_tiles_is_not_an_error() {
    let graph = load_group_graph();
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let mut core = make_core(&graph, &model);

    core.load_tiles(vec![]);

    let requested = vec![nodegroup_id_for_alias(&graph, "basic_info")];
    let result = core
        .populate(false, &requested, "group", &model)
        .expect("populate with zero tiles must not error");

    let name_truthy = result
        .all_values_map
        .get("name")
        .copied()
        .flatten()
        .unwrap_or(false);
    assert!(!name_truthy, "no tiles → no truthy name value");
}

/// Backfill: populate a subset, then load the remaining tiles and populate
/// the remaining nodegroup. Previously loaded values must not duplicate and
/// the new values must appear.
#[test]
fn backfill_merges_without_duplication() {
    let graph = load_group_graph();
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let mut core = make_core(&graph, &model);

    let basic_info_ng = nodegroup_id_for_alias(&graph, "basic_info");
    let statement_ng = nodegroup_id_for_alias(&graph, "statement");

    // Phase 1: only basic_info.
    core.load_tiles(vec![basic_info_tile(&graph)]);
    let first = core
        .populate(false, &[basic_info_ng.clone()], "group", &model)
        .expect("first populate");
    assert_eq!(first.values.get("name").map(|l| l.values.len()), Some(1));

    // CONTRACT (finding of this gate): core populate() READS
    // loaded_nodegroups for dedup but never WRITES it — marking is the
    // caller's responsibility (the WASM wrapper does this at
    // instance_wrapper.rs:624/646). A hydration layer that skips this gets
    // idempotent-but-recomputed populates, silently.
    core.mark_nodegroup_loaded(&basic_info_ng);
    assert!(
        core.is_nodegroup_loaded(&basic_info_ng),
        "caller-side marking is the load-state contract"
    );

    // Phase 2: backfill — full tile set now available, request statement.
    // NOTE (finding): load_tiles() REPLACES the tile map (no append in
    // core); incremental accumulation is the tile-source/caller's job, so
    // the union must be passed here.
    core.load_tiles(vec![basic_info_tile(&graph), statement_tile(&graph)]);
    let second = core
        .populate(
            false,
            &[basic_info_ng.clone(), statement_ng.clone()],
            "group",
            &model,
        )
        .expect("backfill populate");

    let name_list = second
        .values
        .get("name")
        .expect("name survives the backfill");
    assert_eq!(
        name_list.values.len(),
        1,
        "backfill must not duplicate the already-loaded name value"
    );
    let desc_list = second
        .values
        .get("description")
        .expect("description appears after backfill");
    assert_eq!(desc_list.values.len(), 1, "exactly one description value");

    assert!(
        core.is_nodegroup_loaded(&basic_info_ng),
        "basic_info remains marked loaded across backfill"
    );
}

/// Tree building from a tile subset: loaded nodegroups appear, absent
/// nodegroups are absent keys (not nulls, not errors).
#[test]
fn tree_from_tile_subset_has_absent_not_error() {
    let graph = load_group_graph();
    let subset = vec![basic_info_tile(&graph)];
    let meta = metadata(&graph);

    let tree = resource_tiles_to_tree(&subset, &meta, &graph)
        .expect("tree from a tile subset must not error");

    let obj = tree.as_object().expect("tree is an object");
    assert!(
        obj.contains_key("basic_info"),
        "loaded nodegroup present in tree; keys: {:?}",
        obj.keys().collect::<Vec<_>>()
    );
    assert!(
        !obj.contains_key("statement"),
        "unloaded nodegroup must be an ABSENT key, found: {:?}",
        obj.get("statement")
    );
    // Sanity: the name value made it through.
    let basic_info = &obj["basic_info"];
    let rendered = serde_json::to_string(basic_info).unwrap();
    assert!(
        rendered.contains("Sparse Group"),
        "name value should appear in the subtree: {rendered}"
    );
}

/// The nodegroup index only tracks nodegroups that actually have tiles —
/// enumeration over a sparse resource must not assume completeness.
#[test]
fn enumeration_does_not_assume_completeness() {
    let graph = load_group_graph();
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let mut core = make_core(&graph, &model);

    core.load_tiles(vec![basic_info_tile(&graph)]);

    let statement_ng = nodegroup_id_for_alias(&graph, "statement");
    assert!(
        !core.is_nodegroup_loaded(&statement_ng),
        "statement must report not-loaded before populate"
    );

    // Tiles map contains exactly what was loaded — no phantom tiles.
    let tiles: &HashMap<String, StaticTile> =
        core.tiles.as_ref().expect("tiles map exists after load");
    assert_eq!(tiles.len(), 1, "exactly the one loaded tile is present");
}
