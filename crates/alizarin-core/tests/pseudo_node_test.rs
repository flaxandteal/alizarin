//! Pseudo nodes: navigation, the outer/inner split and placeholders, as every
//! binding exposes them.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use alizarin_core::{GraphModelAccess, PseudoNodeCore, StaticGraph};

fn model(file: &str) -> GraphModelAccess {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data/models")
        .join(file);
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read graph")).expect("parse");
    let mut graph: StaticGraph =
        serde_json::from_value(json["graph"][0].clone()).expect("deserialize graph");
    graph.build_indices();
    GraphModelAccess::new_eager(Arc::new(graph), true)
}

/// Navigating from the root records the path: each child's placeholder is its
/// parent's plus its alias, with `[*]` after a list node.
#[test]
fn navigation_records_the_path_from_the_root() {
    let model = model("Group.json");
    let root = PseudoNodeCore::root(&model).expect("root");
    assert_eq!(root.placeholder(), ".group");
    assert_eq!(root.datatype(), "semantic");
    assert!(!root.is_outer());

    let basic_info = root.child("basic_info", &model).expect("basic_info");
    assert_eq!(basic_info.placeholder(), ".group.basic_info");
    let mut children: Vec<&String> = basic_info.child_nodes.keys().collect();
    children.sort();
    assert_eq!(children, ["image", "name", "source"]);
    let name = basic_info.child("name", &model).expect("name");
    assert_eq!(name.placeholder(), ".group.basic_info.name");

    let members = root.child("members", &model).expect("members");
    assert!(members.is_iterable());
    assert_eq!(members.placeholder(), ".group.members[*]");

    assert!(root.child("no_such_child", &model).is_err());
    let start = PseudoNodeCore::by_alias("name", &model).expect("by alias");
    assert_eq!(
        start.placeholder(),
        ".name",
        "no parent, so the path starts here"
    );
}

/// A non-semantic node with children is an outer node whose inner, read as
/// semantic, holds the children.
#[test]
fn a_non_semantic_node_with_children_splits_into_outer_and_inner() {
    let model = model("Person.json");
    let outer = PseudoNodeCore::by_alias("contact_point_type", &model).expect("node");
    assert!(outer.is_outer());
    assert_eq!(outer.datatype(), "concept");
    assert!(outer.child_nodes.is_empty(), "the inner holds the children");

    let inner = outer.inner.as_ref().expect("inner");
    assert!(inner.is_inner);
    assert_eq!(inner.datatype(), "semantic");
    assert!(inner.child_nodes.contains_key("contact_point_metatype"));
    assert_eq!(inner.placeholder(), outer.placeholder());
}
