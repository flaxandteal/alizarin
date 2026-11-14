//! Stored derive functions run by core's write paths: tree-to-tiles and the CSV
//! business-data loader store a stored function's tiles with the resource, and
//! `apply_stored_derive_functions_to_resources` backfills them. Integration test
//! (public API only); providers go in the process-wide registry under ids unique
//! to this file, as the tests share it.

use std::sync::Arc;

use alizarin_core::{
    apply_stored_derive_functions_to_resources, build_graph_from_model_csvs,
    build_resources_from_business_csv, derived_tile_id, tree_to_tiles,
    with_global_functions_registry_mut, ComputeTilesConfig, DeriveProvider, GraphLookup,
    RegisteredFunction, StaticGraph, StaticResource, StaticTile, COMPUTE_TILES_FUNCTION_ID,
};
use serde_json::{json, Value};

const C: &str = "http://www.cidoc-crm.org/cidoc-crm/";
const NODES_HEADER: &str = "parent_alias,alias,name,datatype,cardinality,ontology_class,\
                            parent_property,description,collection_name,required,searchable,\
                            exportable,sortorder";
const UPPER: &str = "stored-derive-test-upper";

/// A graph with a `name` and a `name_upper` string, each in its own nodegroup,
/// declaring `fxgs` (built by `fxgs_for` from the graph's nodegroup ids).
fn thing_graph(fxgs_for: impl Fn(&StaticGraph) -> Vec<Value>) -> StaticGraph {
    let rows = [
        ",name,non-localized-string,1",
        ",name_upper,non-localized-string,1",
    ]
    .iter()
    .map(|r| {
        let f: Vec<&str> = r.split(',').collect();
        format!(
            "{},{},{},{},{},{C}E1,{C}P2,,,,,,",
            f[0], f[1], f[1], f[2], f[3]
        )
    })
    .collect::<Vec<_>>();
    let (mut graph, _) = build_graph_from_model_csvs(
        &format!("name,ontology_class,author,description,is_resource\nThing,{C}E1,,,true"),
        &format!("{NODES_HEADER}\n{}", rows.join("\n")),
        None,
        "https://example.org/test",
        Default::default(),
    )
    .expect("graph");
    graph.functions_x_graphs =
        Some(serde_json::from_value(Value::Array(fxgs_for(&graph))).unwrap());
    graph
}

fn node_id(graph: &dyn GraphLookup, alias: &str) -> String {
    graph.get_node_by_alias(alias).expect("node").nodeid.clone()
}

fn nodegroup(graph: &dyn GraphLookup, alias: &str) -> String {
    graph
        .get_node_by_alias(alias)
        .and_then(|n| n.nodegroup_id.clone())
        .expect("nodegroup")
}

/// A graph declaring `name_upper` as stored, derived from `name`.
fn upper_graph() -> StaticGraph {
    thing_graph(|g| {
        vec![json!({
            "id": "fxg-upper",
            "function_id": COMPUTE_TILES_FUNCTION_ID,
            "graph_id": g.graphid,
            "config": {
                "provider": UPPER,
                "nodegroup": nodegroup(g, "name_upper"),
                "mode": "stored",
                "sources": [nodegroup(g, "name")],
            }
        })]
    })
}

/// `name_upper` = `name` in upper case.
struct Upper;
impl DeriveProvider for Upper {
    fn derive(
        &self,
        resource_id: &str,
        graph: &dyn GraphLookup,
        tiles: &[StaticTile],
        config: &ComputeTilesConfig,
    ) -> Result<Vec<StaticTile>, String> {
        let name = node_id(graph, "name");
        let Some(value) = tiles
            .iter()
            .find_map(|t| t.data.get(&name).and_then(Value::as_str))
        else {
            return Ok(vec![]);
        };
        let mut tile = StaticTile::new_empty(config.nodegroup.clone());
        tile.resourceinstance_id = resource_id.to_string();
        tile.data
            .insert(node_id(graph, "name_upper"), json!(value.to_uppercase()));
        Ok(vec![tile])
    }
}

fn register() {
    with_global_functions_registry_mut(|r| {
        r.register(UPPER, RegisteredFunction::Derive(Arc::new(Upper)))
    });
}

fn upper_value(graph: &StaticGraph, resource: &StaticResource) -> Option<Value> {
    let upper = node_id(graph, "name_upper");
    resource
        .tiles
        .as_ref()?
        .iter()
        .find_map(|t| t.data.get(&upper).cloned())
}

#[test]
fn tree_to_tiles_stores_the_derived_tile() {
    register();
    let graph = upper_graph();
    let wrapper = tree_to_tiles(
        &json!({"resourceinstanceid": "r1", "name": "abc"}),
        &graph,
        true,
        None,
    )
    .expect("tree_to_tiles");
    let resource = &wrapper.business_data.resources[0];
    assert_eq!(upper_value(&graph, resource), Some(json!("ABC")));
    let id = resource.resourceinstance.resourceinstanceid.as_str();
    let ng = nodegroup(&graph, "name_upper");
    let derived = resource
        .tiles
        .as_ref()
        .unwrap()
        .iter()
        .find(|t| t.nodegroup_id == ng)
        .unwrap();
    assert_eq!(
        derived.tileid.as_deref(),
        Some(derived_tile_id(id, &ng, None, UPPER, 0).as_str())
    );
}

#[test]
fn csv_loader_stores_the_derived_tile() {
    register();
    let graph = upper_graph();
    let resources = build_resources_from_business_csv(
        "ResourceID,name\nthing-1,xyz",
        &graph,
        &[],
        Default::default(),
    )
    .expect("csv");
    assert_eq!(upper_value(&graph, &resources[0]), Some(json!("XYZ")));
}

#[test]
fn backfill_derives_for_resources_written_elsewhere() {
    register();
    let graph = upper_graph();
    // Built without the declaration, as data written outside alizarin would be.
    let plain = thing_graph(|_| vec![]);
    let mut resources = tree_to_tiles(
        &json!([{"resourceinstanceid": "r1", "name": "one"}, {"resourceinstanceid": "r2", "name": "two"}]),
        &plain,
        true,
        None,
    )
    .expect("tree_to_tiles")
    .business_data
    .resources;
    assert_eq!(upper_value(&graph, &resources[0]), None);

    let changes =
        apply_stored_derive_functions_to_resources(&mut resources, &graph, None).expect("backfill");
    assert_eq!(changes.len(), 2);
    assert!(changes.iter().all(|c| c.upserted.len() == 1));
    assert_eq!(upper_value(&graph, &resources[0]), Some(json!("ONE")));
    assert_eq!(upper_value(&graph, &resources[1]), Some(json!("TWO")));

    // Backfilling again finds nothing to change.
    let changes =
        apply_stored_derive_functions_to_resources(&mut resources, &graph, None).expect("rerun");
    assert!(changes.iter().all(|c| c.is_empty()));
}

#[test]
fn a_graph_with_no_stored_functions_is_untouched() {
    let plain = thing_graph(|_| vec![]);
    let mut resources = tree_to_tiles(
        &json!({"resourceinstanceid": "r1", "name": "abc"}),
        &plain,
        true,
        None,
    )
    .expect("tree_to_tiles")
    .business_data
    .resources;
    let before = serde_json::to_value(&resources).unwrap();
    apply_stored_derive_functions_to_resources(&mut resources, &plain, None).expect("no-op");
    assert_eq!(serde_json::to_value(&resources).unwrap(), before);
}
