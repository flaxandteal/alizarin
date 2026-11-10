//! Aggregate functions: a provider computes node values on the calling resource
//! from a slice of OTHER resources, and `apply_aggregate_functions` writes them
//! into the resource's tiles. Covers the core contract (with stub providers) and
//! the built-in fold provider. Integration test (public API only).

use std::collections::HashMap;
use std::sync::Arc;

use alizarin_core::extension_type_registry::ExtensionTypeRegistry;
use alizarin_core::{
    aggregate_functions, apply_aggregate_functions, build_graph_from_model_csvs,
    default_functions_registry, AggregateConfig, AggregateContext, AggregateProvider,
    BusinessDataFileSlice, FunctionsRegistry, GraphLookup, NodeValueWrite, RegisteredFunction,
    SliceSource, StaticGraph, StaticResource, StaticResourceRegistry, StaticTile,
    AGGREGATE_FUNCTION_ID, FOLD_AGGREGATE_PROVIDER_ID,
};
use serde_json::{json, Value};

// ---- fixtures -----------------------------------------------------------------

const C: &str = "http://www.cidoc-crm.org/cidoc-crm/";
const LAYER: &str = "layer-1";
const NODES_HEADER: &str = "parent_alias,alias,name,datatype,cardinality,ontology_class,\
                            parent_property,description,collection_name,required,searchable,\
                            exportable,sortorder";

fn csv_graph(name: &str, rows: &[&str]) -> StaticGraph {
    let rows: Vec<String> = rows
        .iter()
        .map(|r| {
            let f: Vec<&str> = r.split(',').collect();
            // parent, alias, datatype, cardinality
            format!(
                "{},{},{},{},{},{C}E1,{C}P2,,,,,,",
                f[0], f[1], f[1], f[2], f[3]
            )
        })
        .collect();
    let (graph, _) = build_graph_from_model_csvs(
        &format!("name,ontology_class,author,description,is_resource\n{name},{C}E1,,,true"),
        &format!("{NODES_HEADER}\n{}", rows.join("\n")),
        None,
        "https://example.org/test",
        Default::default(),
    )
    .expect("graph");
    graph
}

/// A member graph: a number, a boolean and a date, each in its own nodegroup.
fn building_graph() -> StaticGraph {
    csv_graph(
        "Building",
        &[
            ",height,number,1",
            ",listed,boolean,1",
            ",built,date,1",
            ",note,non-localized-string,1",
        ],
    )
}

/// The Layer's graph: a top-level `span` nodegroup, and `inner` nested in `outer`.
fn layer_graph(fxgs: Vec<Value>) -> StaticGraph {
    let mut graph = csv_graph(
        "Layer",
        &[
            ",span,semantic,n",
            "span,span_value,non-localized-string,1",
            "span,span_text,non-localized-string,1",
            "span,span_values,number,1",
            "span,span_resources,number,1",
            "span,span_role,non-localized-string,1",
            ",outer,semantic,n",
            "outer,inner,semantic,n",
            "inner,inner_value,number,1",
        ],
    );
    graph.functions_x_graphs = Some(serde_json::from_value(Value::Array(fxgs)).unwrap());
    graph
}

fn node<'a>(graph: &'a StaticGraph, alias: &str) -> &'a alizarin_core::StaticNode {
    graph
        .nodes
        .iter()
        .find(|n| n.alias.as_deref() == Some(alias))
        .unwrap_or_else(|| panic!("no node {alias}"))
}

fn nodegroup(graph: &StaticGraph, alias: &str) -> String {
    node(graph, alias).nodegroup_id.clone().unwrap()
}

fn fxg(provider: &str, config: Value) -> Value {
    json!({
        "id": format!("fxg-{provider}"),
        "function_id": AGGREGATE_FUNCTION_ID,
        "graph_id": "layer",
        "config": { "provider": provider, "config": config }
    })
}

/// A Building resource with the given node values (by alias).
fn building(graph: &StaticGraph, id: &str, values: &[(&str, Value)]) -> StaticResource {
    let tiles: Vec<StaticTile> = values
        .iter()
        .map(|(alias, value)| {
            let n = node(graph, alias);
            let mut tile = StaticTile::new_empty(n.nodegroup_id.clone().unwrap());
            tile.tileid = Some(format!("{id}-{alias}"));
            tile.resourceinstance_id = id.to_string();
            tile.data.insert(n.nodeid.clone(), value.clone());
            tile
        })
        .collect();
    resource(&graph.graphid, id, tiles)
}

fn resource(graph_id: &str, id: &str, tiles: Vec<StaticTile>) -> StaticResource {
    serde_json::from_value(json!({
        "resourceinstance": {"resourceinstanceid": id, "graph_id": graph_id, "name": id,
                             "descriptors": {}},
        "tiles": tiles,
    }))
    .expect("resource")
}

/// b1..b3 have heights 3, 12, 7 (b2 and b3 listed); b4 has no height; o1 is of
/// another graph; and the Layer resource itself, which must never be visited.
fn members(b: &StaticGraph) -> Vec<StaticResource> {
    vec![
        building(b, "b1", &[("height", json!(3.0)), ("listed", json!(false))]),
        building(b, "b2", &[("height", json!(12.0)), ("listed", json!(true))]),
        building(b, "b3", &[("height", json!(7.0)), ("listed", json!(true))]),
        building(b, "b4", &[("listed", json!(true))]),
        resource("other-graph", "o1", vec![]),
        resource("layer", LAYER, vec![]),
    ]
}

struct Run {
    building: StaticGraph,
    layer: StaticGraph,
}

impl Run {
    fn new(fxgs: Vec<Value>) -> Self {
        Run {
            building: building_graph(),
            layer: layer_graph(fxgs),
        }
    }

    fn apply(&self, slice: &dyn SliceSource, tiles: &mut Vec<StaticTile>) -> Result<(), String> {
        let handlers = ExtensionTypeRegistry::new();
        let graphs: HashMap<String, Arc<StaticGraph>> = HashMap::from([(
            self.building.graphid.clone(),
            Arc::new(self.building.clone()),
        )]);
        let context = AggregateContext::new(&handlers, &graphs);
        apply_aggregate_functions(tiles, &self.layer, LAYER, slice, &registry(), &context)
    }

    fn value(&self, tiles: &[StaticTile], tileid: &str, alias: &str) -> Value {
        let tile = tiles
            .iter()
            .find(|t| t.tileid.as_deref() == Some(tileid))
            .unwrap_or_else(|| panic!("no tile {tileid}"));
        tile.data
            .get(&node(&self.layer, alias).nodeid)
            .cloned()
            .unwrap_or(Value::Null)
    }
}

// ---- stub providers (core contract) ------------------------------------------------

/// Writes exactly the `{tileid, alias, value}` triples in its config.
struct Fixed;

impl AggregateProvider for Fixed {
    fn aggregate(
        &self,
        _target_id: &str,
        graph: &dyn GraphLookup,
        _slice: &dyn SliceSource,
        config: &AggregateConfig,
        _context: &AggregateContext<'_>,
    ) -> Result<Vec<NodeValueWrite>, String> {
        config.config["writes"]
            .as_array()
            .ok_or("no writes")?
            .iter()
            .map(|w| {
                let alias = w["alias"].as_str().ok_or("no alias")?;
                let node_id = graph
                    .get_node_by_alias(alias)
                    .map(|n| n.nodeid.clone())
                    .unwrap_or_else(|| alias.to_string()); // unknown: pass through
                Ok(NodeValueWrite {
                    tileid: w["tileid"].as_str().ok_or("no tileid")?.to_string(),
                    node_id,
                    value: w["value"].clone(),
                })
            })
            .collect()
    }
}

/// Writes the sorted ids of every member it visits.
struct ListMembers;

impl AggregateProvider for ListMembers {
    fn aggregate(
        &self,
        _target_id: &str,
        graph: &dyn GraphLookup,
        slice: &dyn SliceSource,
        config: &AggregateConfig,
        _context: &AggregateContext<'_>,
    ) -> Result<Vec<NodeValueWrite>, String> {
        let mut ids = Vec::new();
        slice.visit(&mut |m| {
            ids.push(m.resource_id.to_string());
            Ok(())
        })?;
        ids.sort();
        Ok(vec![NodeValueWrite {
            tileid: config.config["tileid"].as_str().unwrap().to_string(),
            node_id: graph
                .get_node_by_alias("span_value")
                .unwrap()
                .nodeid
                .clone(),
            value: json!(ids),
        }])
    }
}

fn registry() -> FunctionsRegistry {
    let mut reg = default_functions_registry();
    reg.register("fixed", RegisteredFunction::Aggregate(Arc::new(Fixed)));
    reg.register(
        "list-members",
        RegisteredFunction::Aggregate(Arc::new(ListMembers)),
    );
    reg
}

fn fixed(writes: Value) -> Value {
    fxg("fixed", json!({ "writes": writes }))
}

fn layer_tile(run: &Run, tileid: &str, values: &[(&str, Value)]) -> StaticTile {
    let mut tile = StaticTile::new_empty(nodegroup(&run.layer, "span_value"));
    tile.tileid = Some(tileid.to_string());
    tile.resourceinstance_id = LAYER.to_string();
    for (alias, value) in values {
        tile.data
            .insert(node(&run.layer, alias).nodeid.clone(), value.clone());
    }
    tile
}

// ---- core contract -------------------------------------------------------------

#[test]
fn the_fold_provider_is_built_in() {
    let reg = default_functions_registry();
    assert_eq!(
        reg.get(FOLD_AGGREGATE_PROVIDER_ID).map(|f| f.kind()),
        Some("aggregate")
    );
    assert!(
        reg.derive(FOLD_AGGREGATE_PROVIDER_ID).is_none(),
        "wrong-kind lookup"
    );
}

#[test]
fn writes_one_node_value_and_leaves_the_rest_of_the_tile() {
    let run = Run::new(vec![fixed(json!([
        { "tileid": "t-span", "alias": "span_value", "value": "computed" }
    ]))]);
    let mut tiles = vec![layer_tile(
        &run,
        "t-span",
        &[
            ("span_role", json!("display-extent")),
            ("span_value", json!("old")),
        ],
    )];
    run.apply(&members(&run.building), &mut tiles).unwrap();

    assert_eq!(tiles.len(), 1);
    assert_eq!(run.value(&tiles, "t-span", "span_value"), json!("computed"));
    assert_eq!(
        run.value(&tiles, "t-span", "span_role"),
        json!("display-extent"),
        "the owner's data is untouched"
    );
}

#[test]
fn a_missing_top_level_tile_is_created_with_just_that_value() {
    let run = Run::new(vec![fixed(json!([
        { "tileid": "t-new", "alias": "span_value", "value": "a" },
        { "tileid": "t-new", "alias": "span_values", "value": 3 }
    ]))]);
    let mut tiles = Vec::new();
    run.apply(&members(&run.building), &mut tiles).unwrap();

    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].nodegroup_id, nodegroup(&run.layer, "span_value"));
    assert_eq!(tiles[0].resourceinstance_id, LAYER);
    assert_eq!(tiles[0].data.len(), 2);
    assert_eq!(run.value(&tiles, "t-new", "span_values"), json!(3));
}

#[test]
fn the_calling_resource_is_not_in_its_own_slice() {
    let run = Run::new(vec![fxg("list-members", json!({ "tileid": "t" }))]);
    let mut tiles = Vec::new();
    run.apply(&members(&run.building), &mut tiles).unwrap();
    assert_eq!(
        run.value(&tiles, "t", "span_value"),
        json!(["b1", "b2", "b3", "b4", "o1"])
    );
}

#[test]
fn failures_leave_tiles_untouched() {
    let cases: Vec<(&str, Vec<Value>, &str)> = vec![
        (
            "unregistered provider (after a good one)",
            vec![
                fixed(json!([{ "tileid": "t-span", "alias": "span_value", "value": 1 }])),
                fxg("missing", Value::Null),
            ],
            "no aggregate provider",
        ),
        (
            "two writes to one node of one tile",
            vec![fixed(json!([
                { "tileid": "t-span", "alias": "span_value", "value": 1 },
                { "tileid": "t-span", "alias": "span_value", "value": 2 }
            ]))],
            "two aggregate writes",
        ),
        (
            "an unknown node",
            vec![fixed(
                json!([{ "tileid": "t-span", "alias": "nope", "value": 1 }]),
            )],
            "unknown node",
        ),
        (
            "an existing tile in another nodegroup",
            vec![fixed(
                json!([{ "tileid": "t-span", "alias": "inner_value", "value": 1 }]),
            )],
            "the tile is in nodegroup",
        ),
        (
            "a new tile in a nested nodegroup",
            vec![fixed(
                json!([{ "tileid": "t-new", "alias": "inner_value", "value": 1 }]),
            )],
            "is nested",
        ),
        (
            "a new tile named for two nodegroups",
            vec![fixed(json!([
                { "tileid": "t-new", "alias": "span_value", "value": 1 },
                { "tileid": "t-new", "alias": "inner_value", "value": 2 }
            ]))],
            "name nodes in nodegroups",
        ),
    ];
    for (label, fxgs, expected) in cases {
        let run = Run::new(fxgs);
        let original = vec![layer_tile(&run, "t-span", &[("span_value", json!("keep"))])];
        let mut tiles = original.clone();
        let err = run
            .apply(&members(&run.building), &mut tiles)
            .expect_err(label);
        assert!(err.contains(expected), "{label}: {err}");
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
        "config": { "config": {} }
    });
    let err = aggregate_functions(&layer_graph(vec![bad])).unwrap_err();
    assert!(err.contains("fxg-bad"), "{err}");
}

// ---- the fold provider ------------------------------------------------------------

fn fold(run_building: &StaticGraph, item: Value) -> Value {
    let mut item = item;
    item["graph"] = json!(run_building.graphid);
    fxg(FOLD_AGGREGATE_PROVIDER_ID, json!({ "items": [item] }))
}

fn fold_run(item: Value) -> Run {
    let building = building_graph();
    let fxgs = vec![fold(&building, item)];
    Run {
        building,
        layer: layer_graph(fxgs),
    }
}

#[test]
fn folds_a_node_into_the_named_node_values() {
    let run = fold_run(json!({
        "tileid": "t-height", "path": "height",
        "into": { "span_value": "summary", "span_text": "summary_text",
                  "span_values": "values", "span_resources": "resources" }
    }));
    let mut tiles = Vec::new();
    run.apply(&members(&run.building), &mut tiles).unwrap();

    let summary = json!({ "count": 3, "min": 3.0, "max": 12.0 });
    assert_eq!(run.value(&tiles, "t-height", "span_value"), summary);
    assert_eq!(
        run.value(&tiles, "t-height", "span_text"),
        json!(summary.to_string())
    );
    assert_eq!(run.value(&tiles, "t-height", "span_values"), json!(3));
    assert_eq!(
        run.value(&tiles, "t-height", "span_resources"),
        json!(3),
        "b4 has no height; o1 and the Layer aren't Buildings"
    );
}

#[test]
fn where_keeps_only_members_with_an_allowed_value() {
    let run = fold_run(json!({
        "tileid": "t", "path": "height",
        "where": { "path": "listed", "allowed": ["true"] },
        "into": { "span_value": "summary", "span_resources": "resources" }
    }));
    let mut tiles = Vec::new();
    run.apply(&members(&run.building), &mut tiles).unwrap();
    assert_eq!(
        run.value(&tiles, "t", "span_value"),
        json!({ "count": 2, "min": 7.0, "max": 12.0 })
    );
    assert_eq!(run.value(&tiles, "t", "span_resources"), json!(2));
}

#[test]
fn group_by_writes_one_entry_per_value() {
    let run = fold_run(json!({
        "tileid": "t", "path": "height", "group_by": "listed",
        "into": { "span_value": "summary", "span_resources": "resources" }
    }));
    let mut tiles = Vec::new();
    run.apply(&members(&run.building), &mut tiles).unwrap();
    assert_eq!(
        run.value(&tiles, "t", "span_value"),
        json!({
            "false": { "count": 1, "min": 3.0, "max": 3.0 },
            "true": { "count": 2, "min": 7.0, "max": 12.0 }
        })
    );
    assert_eq!(
        run.value(&tiles, "t", "span_resources"),
        json!({ "false": 1, "true": 2 }),
        "b4 is listed but has no height, so it contributes nothing"
    );
}

#[test]
fn nothing_folded_writes_a_null_summary() {
    let run = fold_run(json!({
        "tileid": "t", "path": "built",
        "into": { "span_value": "summary", "span_text": "summary_text", "span_values": "values" }
    }));
    let mut tiles = Vec::new();
    run.apply(&members(&run.building), &mut tiles).unwrap();
    assert_eq!(run.value(&tiles, "t", "span_value"), Value::Null);
    assert_eq!(run.value(&tiles, "t", "span_text"), Value::Null);
    assert_eq!(run.value(&tiles, "t", "span_values"), json!(0));
}

#[test]
fn config_mistakes_are_caught_before_folding() {
    let cases = [
        (
            "an unknown field",
            json!({ "tileid": "t", "path": "height", "into": { "span_value": "summary" }, "typo": 1 }),
            "unknown field",
        ),
        (
            "an unknown output",
            json!({ "tileid": "t", "path": "height", "into": { "span_value": "average" } }),
            "unknown variant",
        ),
        (
            "a path that doesn't resolve",
            json!({ "tileid": "t", "path": "nope", "into": { "span_value": "summary" } }),
            "nope",
        ),
        (
            "a datatype with no fold",
            json!({ "tileid": "t", "path": "note", "into": { "span_value": "summary" } }),
            "has no fold",
        ),
        (
            "group_by on an open vocabulary",
            json!({ "tileid": "t", "path": "height", "group_by": "note",
                    "into": { "span_value": "summary" } }),
            "not a fixed vocabulary",
        ),
        (
            "an empty allowed set",
            json!({ "tileid": "t", "path": "height", "where": { "path": "listed", "allowed": [] },
                    "into": { "span_value": "summary" } }),
            "'allowed' is empty",
        ),
        (
            "an into alias the Layer graph doesn't have",
            json!({ "tileid": "t", "path": "height", "into": { "nope": "summary" } }),
            "no node with alias nope",
        ),
        (
            "nothing to write",
            json!({ "tileid": "t", "path": "height", "into": {} }),
            "names no nodes",
        ),
    ];
    for (label, item, expected) in cases {
        let run = fold_run(item);
        let mut tiles = Vec::new();
        let err = run
            .apply(&members(&run.building), &mut tiles)
            .expect_err(label);
        assert!(err.contains(expected), "{label}: {err}");
        assert!(tiles.is_empty(), "{label}: tiles changed");
    }
}

#[test]
fn reruns_and_visit_order_give_identical_tiles() {
    let item = json!({
        "tileid": "t", "path": "height", "group_by": "listed",
        "into": { "span_value": "summary", "span_values": "values" }
    });
    let run = fold_run(item);
    let mut first = Vec::new();
    run.apply(&members(&run.building), &mut first).unwrap();

    let mut again = first.clone();
    run.apply(&members(&run.building), &mut again).unwrap();

    let mut reversed_members = members(&run.building);
    reversed_members.reverse();
    let mut reversed = Vec::new();
    run.apply(&reversed_members, &mut reversed).unwrap();

    let json = |t: &Vec<StaticTile>| serde_json::to_value(t).unwrap();
    assert_eq!(json(&again), json(&first), "re-running is idempotent");
    assert_eq!(json(&reversed), json(&first), "visit order doesn't matter");
}

// ---- slices -----------------------------------------------------------------------

fn write_business_data(name: &str, resources: &[StaticResource]) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "alizarin-aggregate-{name}-{}.json",
        std::process::id()
    ));
    let resources: Vec<Value> = resources
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    std::fs::write(
        &path,
        serde_json::to_vec(&json!({ "business_data": { "resources": resources } })).unwrap(),
    )
    .unwrap();
    path
}

#[test]
fn registry_and_streamed_file_slices_match_the_in_memory_one() {
    let run = fold_run(json!({
        "tileid": "t", "path": "height", "into": { "span_value": "summary" }
    }));
    let mut expected = Vec::new();
    run.apply(&members(&run.building), &mut expected).unwrap();

    let mut registry_slice = StaticResourceRegistry::new();
    for r in members(&run.building) {
        registry_slice.insert_full(r);
    }
    let mut from_registry = Vec::new();
    run.apply(&registry_slice, &mut from_registry).unwrap();
    assert_eq!(from_registry[0].data, expected[0].data);

    let path = write_business_data("slice", &members(&run.building));
    let mut from_file = Vec::new();
    let result = run.apply(&BusinessDataFileSlice::new(&path), &mut from_file);
    std::fs::remove_file(&path).ok();
    result.unwrap();
    assert_eq!(from_file[0].data, expected[0].data);
}

#[test]
fn a_visitor_error_stops_the_streamed_visit() {
    let path = write_business_data("stop", &members(&building_graph()));
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
