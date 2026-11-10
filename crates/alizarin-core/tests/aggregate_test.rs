//! Aggregate functions: a provider folds a slice of OTHER resources into tiles on
//! the calling resource. Integration test (public API only).

use std::collections::HashMap;
use std::sync::Arc;

use alizarin_core::datatype_fold::{fold_identity, fold_values};
use alizarin_core::extension_type_registry::ExtensionTypeRegistry;
use alizarin_core::{
    aggregate_functions, apply_aggregate_functions, AggregateConfig, AggregateContext,
    AggregateProvider, BusinessDataFileSlice, FunctionsRegistry, GraphLookup, RegisteredFunction,
    SliceMember, SliceSource, StaticGraph, StaticResource, StaticResourceRegistry, StaticTile,
    AGGREGATE_FUNCTION_ID,
};
use serde_json::{json, Value};

// ---- fixtures -----------------------------------------------------------------

const LAYER: &str = "layer-1";

/// A member graph with one number node, `height`, in nodegroup `ng-height`.
fn building_graph() -> StaticGraph {
    let root =
        json!({"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "building"});
    serde_json::from_value(json!({
        "graphid": "building",
        "name": {"en": "Building"},
        "root": root,
        "nodes": [
            root,
            {"nodeid": "n-height", "alias": "height", "name": "height", "datatype": "number",
             "graph_id": "building", "nodegroup_id": "ng-height"}
        ],
        "edges": [], "nodegroups": [], "cards": [], "cards_x_nodes_x_widgets": [],
    }))
    .expect("building graph")
}

/// The Layer's graph, declaring the given Aggregate functions.
fn layer_graph(fxgs: Vec<Value>) -> StaticGraph {
    let root =
        json!({"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "layer"});
    serde_json::from_value(json!({
        "graphid": "layer",
        "name": {"en": "Layer"},
        "root": root,
        "nodes": [root],
        "edges": [], "nodegroups": [], "cards": [], "cards_x_nodes_x_widgets": [],
        "functions_x_graphs": fxgs,
    }))
    .expect("layer graph")
}

fn aggregate_fxg(provider: &str, nodegroup: &str, config: Value) -> Value {
    json!({
        "id": format!("fxg-{provider}-{nodegroup}"),
        "function_id": AGGREGATE_FUNCTION_ID,
        "graph_id": "layer",
        "config": { "provider": provider, "nodegroup": nodegroup, "config": config }
    })
}

fn tile(nodegroup: &str, resource: &str, tileid: &str, data: Value) -> StaticTile {
    StaticTile {
        data: data
            .as_object()
            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        nodegroup_id: nodegroup.to_string(),
        resourceinstance_id: resource.to_string(),
        tileid: Some(tileid.to_string()),
        parenttile_id: None,
        provisionaledits: None,
        sortorder: None,
    }
}

fn resource_json(graph_id: &str, id: &str, height: Option<f64>) -> Value {
    let tiles = match height {
        Some(h) => json!([{
            "tileid": format!("t-{id}"), "nodegroup_id": "ng-height",
            "resourceinstance_id": id, "parenttile_id": null, "sortorder": 0,
            "data": {"n-height": h}
        }]),
        None => json!([]),
    };
    json!({
        "resourceinstance": {"resourceinstanceid": id, "graph_id": graph_id, "name": id,
                             "descriptors": {}},
        "tiles": tiles
    })
}

/// Three buildings (heights 3, 12, 7), one with no height, one resource of
/// another graph, and the Layer resource itself (which must not be visited).
fn members() -> Vec<StaticResource> {
    [
        resource_json("building", "b1", Some(3.0)),
        resource_json("building", "b2", Some(12.0)),
        resource_json("building", "b3", Some(7.0)),
        resource_json("building", "b4", None),
        resource_json("other", "o1", Some(99.0)),
        resource_json("layer", LAYER, Some(1000.0)),
    ]
    .into_iter()
    .map(|r| serde_json::from_value(r).expect("resource"))
    .collect()
}

fn graphs() -> HashMap<String, Arc<StaticGraph>> {
    HashMap::from([("building".to_string(), Arc::new(building_graph()))])
}

// ---- providers -------------------------------------------------------------------

/// The intended shape of a real provider: resolve `{graph, alias}` through the
/// member's graph, fold that node's values with the datatype's fold, and write
/// one summary tile.
struct FoldNode;

impl AggregateProvider for FoldNode {
    fn aggregate(
        &self,
        target_id: &str,
        _graph: &dyn GraphLookup,
        slice: &dyn SliceSource,
        config: &AggregateConfig,
        context: &AggregateContext<'_>,
    ) -> Result<Vec<StaticTile>, String> {
        let graph_id = config.config["graph"].as_str().ok_or("no graph")?;
        let alias = config.config["alias"].as_str().ok_or("no alias")?;
        let graph = context.graphs.graph(graph_id).ok_or("unknown graph")?;
        let node = graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some(alias))
            .ok_or("unknown alias")?;
        let handlers = Some(context.handlers);
        let mut acc =
            fold_identity(&node.datatype, None, handlers).ok_or("datatype has no fold")?;
        let mut resources = 0usize;
        slice.visit(&mut |m: SliceMember<'_>| {
            if m.graph_id != graph_id {
                return Ok(());
            }
            let values: Vec<&Value> = m
                .tiles
                .iter()
                .filter_map(|t| t.data.get(&node.nodeid))
                .filter(|v| !v.is_null())
                .collect();
            if !values.is_empty() {
                resources += 1;
            }
            let step = fold_values(&node.datatype, acc.take(), &values, None, handlers)
                .map_err(|e| e.0)?;
            acc = step.summary;
            Ok(())
        })?;
        Ok(vec![tile(
            &config.nodegroup,
            target_id,
            "span-0",
            json!({ "summary": acc, "resources": resources }),
        )])
    }
}

/// Writes one tile per visited member, recording its id: shows exactly which
/// resources were in the slice.
struct ListMembers;

impl AggregateProvider for ListMembers {
    fn aggregate(
        &self,
        target_id: &str,
        _graph: &dyn GraphLookup,
        slice: &dyn SliceSource,
        config: &AggregateConfig,
        _context: &AggregateContext<'_>,
    ) -> Result<Vec<StaticTile>, String> {
        let mut ids = Vec::new();
        slice.visit(&mut |m| {
            ids.push(m.resource_id.to_string());
            Ok(())
        })?;
        ids.sort();
        Ok(vec![tile(
            &config.nodegroup,
            target_id,
            "members",
            json!({ "ids": ids }),
        )])
    }
}

/// Produces a tile for the wrong nodegroup.
struct Stray;

impl AggregateProvider for Stray {
    fn aggregate(
        &self,
        target_id: &str,
        _graph: &dyn GraphLookup,
        _slice: &dyn SliceSource,
        _config: &AggregateConfig,
        _context: &AggregateContext<'_>,
    ) -> Result<Vec<StaticTile>, String> {
        Ok(vec![tile("somewhere-else", target_id, "x", json!({}))])
    }
}

fn registry() -> FunctionsRegistry {
    let mut reg = FunctionsRegistry::new();
    reg.register(
        "fold-node",
        RegisteredFunction::Aggregate(Arc::new(FoldNode)),
    );
    reg.register(
        "list-members",
        RegisteredFunction::Aggregate(Arc::new(ListMembers)),
    );
    reg.register("stray", RegisteredFunction::Aggregate(Arc::new(Stray)));
    reg
}

fn run(
    fxgs: Vec<Value>,
    slice: &dyn SliceSource,
    tiles: &mut Vec<StaticTile>,
) -> Result<(), String> {
    let handlers = ExtensionTypeRegistry::new();
    let graphs = graphs();
    let context = AggregateContext::new(&handlers, &graphs);
    apply_aggregate_functions(
        tiles,
        &layer_graph(fxgs),
        LAYER,
        slice,
        &registry(),
        &context,
    )
}

fn fold_height() -> Value {
    aggregate_fxg(
        "fold-node",
        "ng-spans",
        json!({ "graph": "building", "alias": "height" }),
    )
}

// ---- tests -------------------------------------------------------------------------

#[test]
fn registry_resolves_aggregates_by_kind() {
    let reg = registry();
    assert_eq!(reg.get("fold-node").map(|f| f.kind()), Some("aggregate"));
    assert!(reg.aggregate("fold-node").is_some());
    assert!(reg.derive("fold-node").is_none(), "wrong-kind lookup");
    assert!(reg.aggregate("missing").is_none());
}

#[test]
fn folds_a_node_across_the_slice_into_the_target_nodegroup() {
    let mut tiles = Vec::new();
    run(vec![fold_height()], &members(), &mut tiles).unwrap();

    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].nodegroup_id, "ng-spans");
    assert_eq!(tiles[0].resourceinstance_id, LAYER);
    // b1..b3 only: b4 has no value, o1 is another graph, the Layer is excluded.
    assert_eq!(
        tiles[0].data["summary"],
        json!({ "min": 3.0, "max": 12.0, "count": 3 })
    );
    assert_eq!(tiles[0].data["resources"], json!(3));
}

#[test]
fn the_calling_resource_is_not_in_its_own_slice() {
    let mut tiles = Vec::new();
    run(
        vec![aggregate_fxg("list-members", "ng-members", Value::Null)],
        &members(),
        &mut tiles,
    )
    .unwrap();
    assert_eq!(tiles[0].data["ids"], json!(["b1", "b2", "b3", "b4", "o1"]));
}

#[test]
fn rerunning_replaces_the_target_nodegroup_and_keeps_others() {
    let mut tiles = vec![
        tile("ng-spans", LAYER, "stale", json!({ "summary": "old" })),
        tile("ng-title", LAYER, "title", json!({ "title": "Layer one" })),
    ];
    run(vec![fold_height()], &members(), &mut tiles).unwrap();
    let first = tiles.clone();
    run(vec![fold_height()], &members(), &mut tiles).unwrap();

    assert_eq!(tiles.len(), 2);
    assert!(tiles.iter().any(|t| t.tileid.as_deref() == Some("title")));
    assert!(!tiles.iter().any(|t| t.tileid.as_deref() == Some("stale")));
    assert_eq!(
        serde_json::to_value(&tiles).unwrap(),
        serde_json::to_value(&first).unwrap(),
        "re-running on unchanged members is idempotent"
    );
}

#[test]
fn visit_order_does_not_change_the_output() {
    let mut forward = Vec::new();
    run(vec![fold_height()], &members(), &mut forward).unwrap();
    let mut reversed_members = members();
    reversed_members.reverse();
    let mut reversed = Vec::new();
    run(vec![fold_height()], &reversed_members, &mut reversed).unwrap();
    assert_eq!(forward[0].data, reversed[0].data);
}

#[test]
fn failures_leave_tiles_untouched() {
    let original = vec![tile("ng-spans", LAYER, "keep", json!({ "summary": "old" }))];
    let cases: Vec<(&str, Vec<Value>)> = vec![
        (
            "unregistered provider (after a good one)",
            vec![
                fold_height(),
                aggregate_fxg("missing", "ng-other", Value::Null),
            ],
        ),
        (
            "tile for another nodegroup",
            vec![aggregate_fxg("stray", "ng-spans", Value::Null)],
        ),
        (
            "two functions fill one nodegroup",
            vec![
                fold_height(),
                aggregate_fxg("list-members", "ng-spans", Value::Null),
            ],
        ),
        (
            "provider error",
            vec![aggregate_fxg(
                "fold-node",
                "ng-spans",
                json!({ "graph": "building" }),
            )],
        ),
    ];
    for (label, fxgs) in cases {
        let mut tiles = original.clone();
        let err = run(fxgs, &members(), &mut tiles);
        assert!(err.is_err(), "{label}: expected an error");
        assert_eq!(
            serde_json::to_value(&tiles).unwrap(),
            serde_json::to_value(&original).unwrap(),
            "{label}: tiles changed"
        );
    }
}

#[test]
fn a_malformed_declaration_is_an_error() {
    let bad = json!({
        "id": "fxg-bad", "function_id": AGGREGATE_FUNCTION_ID, "graph_id": "layer",
        "config": { "provider": "fold-node" }
    });
    let err = aggregate_functions(&layer_graph(vec![bad])).unwrap_err();
    assert!(err.contains("fxg-bad"), "{err}");
}

#[test]
fn registry_and_streamed_file_slices_match_the_in_memory_one() {
    let mut expected = Vec::new();
    run(vec![fold_height()], &members(), &mut expected).unwrap();

    let mut registry_slice = StaticResourceRegistry::new();
    for r in members() {
        registry_slice.insert_full(r);
    }
    let mut from_registry = Vec::new();
    run(vec![fold_height()], &registry_slice, &mut from_registry).unwrap();
    assert_eq!(from_registry[0].data, expected[0].data);

    let path = std::env::temp_dir().join(format!(
        "alizarin-aggregate-test-{}.json",
        std::process::id()
    ));
    let resources: Vec<Value> = members()
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    std::fs::write(
        &path,
        serde_json::to_vec(&json!({ "business_data": { "resources": resources } })).unwrap(),
    )
    .unwrap();
    let mut from_file = Vec::new();
    let result = run(
        vec![fold_height()],
        &BusinessDataFileSlice::new(&path),
        &mut from_file,
    );
    std::fs::remove_file(&path).ok();
    result.unwrap();
    assert_eq!(from_file[0].data, expected[0].data);
}

#[test]
fn a_visitor_error_stops_the_streamed_visit() {
    let path = std::env::temp_dir().join(format!(
        "alizarin-aggregate-stop-{}.json",
        std::process::id()
    ));
    let resources: Vec<Value> = members()
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    std::fs::write(
        &path,
        serde_json::to_vec(&json!({ "business_data": { "resources": resources } })).unwrap(),
    )
    .unwrap();
    let mut seen = 0;
    let result = BusinessDataFileSlice::new(&path).visit(&mut |_| {
        seen += 1;
        if seen == 2 {
            Err("stop".to_string())
        } else {
            Ok(())
        }
    });
    std::fs::remove_file(&path).ok();
    assert!(result.unwrap_err().contains("stop"));
    assert_eq!(seen, 2);
}
