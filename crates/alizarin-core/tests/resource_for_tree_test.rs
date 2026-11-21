//! How the bindings' tiles-to-tree functions read a resource
//! (`resource_for_tree`): nested or flat, its graph, its id. One rule for
//! WASM, NAPI and Python, which each had their own (B-9).

use alizarin_core::{
    batch_tiles_to_trees_from_json, resource_for_tree, tiles_to_tree_from_json, GraphWrapper,
    StaticGraph,
};
use serde_json::{json, Value};
use std::path::PathBuf;

fn read(name: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/data/output-parity");
    std::fs::read_to_string(dir.join(name)).unwrap()
}

fn graph() -> StaticGraph {
    serde_json::from_str::<GraphWrapper>(&read("graph.json"))
        .unwrap()
        .graph
        .remove(0)
}

fn resource() -> Value {
    serde_json::from_str(&read("resource.json")).unwrap()
}

/// The resource as flat `{resourceinstanceid, graph_id, tiles}`.
fn flat() -> Value {
    let resource = resource();
    json!({
        "resourceinstanceid": resource["resourceinstance"]["resourceinstanceid"],
        "graph_id": resource["resourceinstance"]["graph_id"],
        "tiles": resource["tiles"],
    })
}

#[test]
fn flat_and_nested_give_the_same_tree_data() {
    let graph = graph();
    let nested = tiles_to_tree_from_json(&resource().to_string(), &graph).unwrap();
    let flat = tiles_to_tree_from_json(&flat().to_string(), &graph).unwrap();
    for key in ["resourceinstanceid", "graph_id", "name", "part"] {
        assert_eq!(nested[key], flat[key], "{key}");
    }
}

#[test]
fn a_flat_resource_without_an_id_takes_its_tiles() {
    let mut flat = flat();
    flat.as_object_mut().unwrap().remove("resourceinstanceid");
    let resource = resource_for_tree(&flat, &graph()).unwrap();
    assert_eq!(
        resource.resourceinstance.resourceinstanceid,
        "0b0b0b0b-0000-4000-8000-000000000001"
    );
}

#[test]
fn no_id_anywhere_is_an_error_not_a_random_one() {
    let err = resource_for_tree(&json!({"tiles": []}), &graph()).unwrap_err();
    assert!(err.contains("no resourceinstanceid"), "{err}");
}

#[test]
fn a_resource_of_another_graph_is_an_error() {
    let mut flat = flat();
    flat["graph_id"] = json!("not-this-graph");
    let err = resource_for_tree(&flat, &graph()).unwrap_err();
    assert!(err.contains("not-this-graph"), "{err}");
}

#[test]
fn malformed_tiles_are_an_error() {
    let mut flat = flat();
    flat["tiles"] = json!("not tiles");
    let err = resource_for_tree(&flat, &graph()).unwrap_err();
    assert!(err.contains("Failed to parse tiles"), "{err}");
}

#[test]
fn a_batch_collects_failures_unless_strict() {
    let graph = graph();
    let batch = format!("[{}, {}]", resource(), json!({"tiles": []}));

    let outcome = batch_tiles_to_trees_from_json(&batch, Some(&graph), false).unwrap();
    assert!(!outcome.success);
    assert_eq!((outcome.count, outcome.error_count), (1, 1));
    assert!(
        outcome.errors[0].starts_with("Resource 1: "),
        "{:?}",
        outcome.errors
    );

    let err = batch_tiles_to_trees_from_json(&batch, Some(&graph), true).unwrap_err();
    assert!(err.starts_with("Strict mode error: Resource 1: "), "{err}");
}
