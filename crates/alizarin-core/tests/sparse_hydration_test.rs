//! M0.4 sparse-hydration gate (Static-Assets-Implementation-Plan).
//!
//! Verifies that a resource populated from a SUBSET of its tiles behaves
//! correctly: enumeration does not assume completeness, absent nodegroups
//! (including cardinality-1) raise no spurious errors, tree building from a
//! subset yields absent keys rather than errors, and later backfill merges
//! without duplicating already-loaded values.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use alizarin_core::{
    resource_tiles_to_tree, ExtensionTypeRegistry, GraphModelAccess, PseudoListCore,
    PseudoValueCore, ResourceInstanceWrapperCore, SemanticChildResult, StaticGraph,
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

fn make_core(graph: &StaticGraph) -> ResourceInstanceWrapperCore {
    let mut core = ResourceInstanceWrapperCore::new(graph.graphid.clone());
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
    let mut core = make_core(&graph);

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
    let mut core = make_core(&graph);

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
    let mut core = make_core(&graph);

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
    let mut core = make_core(&graph);

    core.load_tiles(vec![basic_info_tile(&graph)]);

    let statement_ng = nodegroup_id_for_alias(&graph, "statement");
    assert!(
        !core.is_nodegroup_loaded(&statement_ng),
        "statement must report not-loaded before populate"
    );

    // Tiles map contains exactly what was loaded — no phantom tiles.
    let tiles: &alizarin_core::TileStore =
        core.tiles.as_ref().expect("tiles map exists after load");
    assert_eq!(tiles.len(), 1, "exactly the one loaded tile is present");
}

/// A lazy caller's missing nodegroups for a node's children are each listed
/// once, and drop out once loaded; eager mode has none missing; release
/// forgets what was loaded.
#[test]
fn missing_child_nodegroups_are_listed_once_and_release_forgets_loads() {
    let graph = load_group_graph();
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let basic_info = graph
        .get_node_by_alias("basic_info")
        .expect("basic_info node")
        .nodeid
        .clone();
    let mut core = make_core(&graph);

    // basic_info's three children share one nodegroup: listed once.
    let missing = core.missing_nodegroups_for_children(&basic_info, &model, true);
    assert_eq!(missing.len(), 1, "{missing:?}");
    assert!(
        core.missing_nodegroups_for_children(&basic_info, &model, false)
            .is_empty(),
        "eager: everything is loaded"
    );

    core.mark_nodegroup_loaded(&missing[0]);
    assert!(core
        .missing_nodegroups_for_children(&basic_info, &model, true)
        .is_empty());

    core.release();
    assert_eq!(
        core.missing_nodegroups_for_children(&basic_info, &model, true),
        missing,
        "release forgets the load"
    );
}

/// Loading through the model's permissions drops tiles it forbids, for a full
/// load and an incremental merge alike; once permitted, a merge accepts them.
#[test]
fn permitted_loads_drop_tiles_the_model_forbids() {
    let graph = load_group_graph();
    let tile = basic_info_tile(&graph);
    let mut model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    model.set_permitted_nodegroups_rules(std::collections::HashMap::from([(
        tile.nodegroup_id.clone(),
        alizarin_core::PermissionRule::Boolean(false),
    )]));
    let count = |core: &ResourceInstanceWrapperCore| core.tiles.as_ref().map_or(0, |t| t.len());

    let mut core = make_core(&graph);
    core.load_permitted_tiles(vec![tile.clone()], &model);
    assert_eq!(count(&core), 0, "load drops the forbidden tile");
    core.merge_permitted_tiles(vec![tile.clone()], &model);
    assert_eq!(count(&core), 0, "merge drops it too");

    model.set_default_allow(true);
    model.set_permitted_nodegroups_rules(std::collections::HashMap::new());
    core.merge_permitted_tiles(vec![tile], &model);
    assert_eq!(count(&core), 1, "permitted, so merged");
}

fn basic_info_tile_numbered(
    graph: &StaticGraph,
    tile_id: &str,
    sortorder: Option<i32>,
) -> StaticTile {
    let mut tile = basic_info_tile(graph);
    tile.tileid = Some(tile_id.to_string());
    tile.sortorder = sortorder;
    tile
}

/// Semantic children come back in sortorder, unnumbered tiles after the
/// numbered ones in load order, the same for ids and values; every child
/// alias is listed, `[]` when it has none.
#[test]
fn semantic_children_follow_sortorder_with_unnumbered_last_in_load_order() {
    let graph = load_group_graph();
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let root = graph.root.nodeid.clone();
    let mut core = make_core(&graph);
    core.load_tiles(vec![
        basic_info_tile_numbered(&graph, "a", None),
        basic_info_tile_numbered(&graph, "b", Some(2)),
        basic_info_tile_numbered(&graph, "c", Some(1)),
        basic_info_tile_numbered(&graph, "d", None),
        statement_tile(&graph),
    ]);
    let expected = vec!["c", "b", "a", "d"];

    let children = core
        .find_semantic_children(None, &root, None, &model)
        .expect("children");
    assert_eq!(children["basic_info"], expected);
    assert_eq!(children["statement"], vec!["tile-statement-1"]);
    assert_eq!(
        children["identifier"],
        Vec::<String>::new(),
        "listed, with none"
    );

    match core
        .get_semantic_child_value(None, &root, None, "basic_info", &model)
        .expect("value")
    {
        SemanticChildResult::List(list) => {
            let ids: Vec<String> = list
                .values
                .iter()
                .map(|v| {
                    v.tile
                        .as_ref()
                        .and_then(|t| t.tileid.clone())
                        .unwrap_or_default()
                })
                .collect();
            assert_eq!(ids, expected);
        }
        other => panic!("expected a list, got {other:?}"),
    }
}

/// In lazy mode a child whose nodegroup the host hasn't fetched is an error
/// (`TILES_NOT_LOADED:<nodegroup>`), not an absence, for one child or all;
/// merging its tiles, or marking an empty fetch loaded, answers it. Eager
/// mode never asks.
#[test]
fn lazy_semantic_children_error_until_their_nodegroup_is_loaded() {
    let graph = load_group_graph();
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let root = graph.root.nodeid.clone();
    let statement_ng = nodegroup_id_for_alias(&graph, "statement");
    let mut core = make_core(&graph);
    core.set_lazy(true);
    core.load_tiles(vec![basic_info_tile(&graph)]);

    assert!(core
        .get_semantic_child_value(None, &root, None, "basic_info", &model)
        .is_ok());
    let err = core
        .get_semantic_child_value(None, &root, None, "statement", &model)
        .expect_err("not fetched yet");
    assert_eq!(err.to_string(), format!("TILES_NOT_LOADED:{statement_ng}"));
    assert!(core
        .get_all_semantic_child_values(None, &root, None, &model)
        .is_err());

    core.merge_tiles(vec![statement_tile(&graph)]);
    assert!(core
        .get_semantic_child_value(None, &root, None, "statement", &model)
        .is_ok());

    // The host fetched the rest and found nothing.
    for nodegroup_id in core.missing_nodegroups_for_children(&root, &model, true) {
        core.mark_nodegroup_loaded(&nodegroup_id);
    }
    assert!(core
        .get_all_semantic_child_values(None, &root, None, &model)
        .is_ok());

    let mut eager = make_core(&graph);
    eager.load_tiles(vec![basic_info_tile(&graph)]);
    assert!(eager
        .get_semantic_child_value(None, &root, None, "statement", &model)
        .is_ok());
}

/// Tiles the host says are the resource's complete set make every nodegroup
/// loaded, so in lazy mode a child with no tiles is empty, not unfetched.
#[test]
fn comprehensive_tiles_mark_every_nodegroup_loaded() {
    let graph = load_group_graph();
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let root = graph.root.nodeid.clone();
    let mut core = make_core(&graph);
    core.set_lazy(true);
    core.load_tiles(vec![basic_info_tile(&graph)]);
    assert!(core
        .get_semantic_child_value(None, &root, None, "statement", &model)
        .is_err());

    core.mark_all_nodegroups_loaded(&model);
    assert!(matches!(
        core.get_semantic_child_value(None, &root, None, "statement", &model),
        Ok(SemanticChildResult::Empty)
    ));
}

/// Descriptors come from the cache unless asked to recompute or none are
/// cached, when they are built from the tiles (the graph is asked for only
/// then) and cached; the name falls back from the metadata to them.
#[test]
fn descriptors_build_from_tiles_once_and_name_falls_back_to_them() {
    let graph = Arc::new(load_group_graph());
    let mut core = make_core(&graph);
    core.load_tiles(vec![basic_info_tile(&graph)]);
    core.resource_instance.as_mut().expect("metadata").name = String::new();
    assert_eq!(core.display_name(), "<Unnamed>");

    let calls = std::cell::Cell::new(0);
    let context = || {
        calls.set(calls.get() + 1);
        Ok::<_, String>((graph.clone(), ExtensionTypeRegistry::new()))
    };
    let built = core.descriptors(false, context).expect("built");
    assert_eq!(built.name.as_deref(), Some("Sparse Group"));
    assert_eq!(core.display_name(), "Sparse Group");

    core.descriptors(false, context).expect("cached");
    assert_eq!(calls.get(), 1, "cached, so the graph isn't asked for");
    core.descriptors(true, context).expect("rebuilt");
    assert_eq!(calls.get(), 2, "recompute builds again");
}

/// A collector's made pseudo lists its nodegroup's tiles (none if not
/// permitted); anything else is one value on the given tile, or unavailable
/// if not permitted; an unknown alias is an error.
#[test]
fn made_pseudos_list_a_collectors_tiles_and_respect_permission() {
    let graph = load_group_graph();
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let mut core = make_core(&graph);
    core.load_tiles(vec![
        basic_info_tile_numbered(&graph, "a", None),
        basic_info_tile_numbered(&graph, "b", Some(1)),
    ]);
    let made = |alias: &str, tile_id: Option<&str>, permitted: bool| {
        core.make_pseudo_value(alias, tile_id, permitted, false, &model)
    };

    match made("basic_info", None, true).expect("made") {
        SemanticChildResult::List(list) => {
            let ids: Vec<Option<&str>> = list
                .values
                .iter()
                .map(|v| v.tile.as_ref().and_then(|t| t.tileid.as_deref()))
                .collect();
            assert_eq!(ids, [Some("b"), Some("a")], "numbered first");
        }
        other => panic!("expected a list, got {other:?}"),
    }
    match made("basic_info", None, false).expect("made") {
        SemanticChildResult::List(list) => assert!(list.values.is_empty()),
        other => panic!("expected an empty list, got {other:?}"),
    }
    match made("name", Some("a"), true).expect("made") {
        SemanticChildResult::Single(value) => assert_eq!(
            value.tile.as_ref().and_then(|t| t.tileid.as_deref()),
            Some("a")
        ),
        other => panic!("expected a value, got {other:?}"),
    }
    assert!(matches!(
        made("name", Some("a"), false),
        Ok(SemanticChildResult::Empty)
    ));
    assert!(made("no_such_alias", None, true).is_err());
}

/// A list keeps its values in tile sortorder (unnumbered last, ties in load
/// order) however it is built: from values, merged, or pushed to.
#[test]
fn pseudo_lists_keep_values_in_sortorder() {
    let graph = load_group_graph();
    let node = Arc::new(
        graph
            .nodes_slice()
            .iter()
            .find(|n| n.alias.as_deref() == Some("basic_info"))
            .expect("basic_info node")
            .clone(),
    );
    let value = |id: &str, sortorder: Option<i32>| {
        PseudoValueCore::from_node_and_tile(
            Arc::clone(&node),
            Some(Arc::new(basic_info_tile_numbered(&graph, id, sortorder))),
            None,
            vec![],
        )
    };
    let ids = |list: &PseudoListCore| -> Vec<String> {
        list.values
            .iter()
            .map(|v| {
                v.tile
                    .as_ref()
                    .and_then(|t| t.tileid.clone())
                    .unwrap_or_default()
            })
            .collect()
    };

    let mut list = PseudoListCore::from_values(
        "basic_info".into(),
        vec![value("a", None), value("b", Some(2))],
    );
    assert_eq!(ids(&list), ["b", "a"]);
    list.merge(PseudoListCore::from_values(
        "basic_info".into(),
        vec![value("c", Some(1)), value("d", None)],
    ));
    assert_eq!(ids(&list), ["c", "b", "a", "d"]);
    list.push(value("e", Some(2)));
    assert_eq!(ids(&list), ["c", "b", "e", "a", "d"]);
}
