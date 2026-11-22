//! Core's one reader of business data (`parse_business_data_bytes`), behind
//! the prebuild loader, `loadFromBusinessDataBytes` and `parseStaticResources`
//! on every binding (B-12: there were two, which disagreed).

use alizarin_core::{parse_business_data_bytes, resources_from_business_data_json};
use serde_json::json;

fn file(resources: serde_json::Value) -> Vec<u8> {
    json!({"business_data": {"resources": resources}})
        .to_string()
        .into_bytes()
}

fn resource(id: &str) -> serde_json::Value {
    json!({
        "resourceinstance": {
            "graph_id": "0a0a0a0a-0000-4000-8000-000000000001",
            "name": id,
            "resourceinstanceid": id,
        },
    })
}

#[test]
fn descriptors_and_metadata_may_be_absent_or_null() {
    let mut with_null = resource("b");
    with_null["metadata"] = json!(null);
    let read = parse_business_data_bytes(&file(json!([resource("a"), with_null]))).unwrap();
    assert_eq!(read.len(), 2);
    assert!(read[0].resourceinstance.descriptors.is_empty());
    assert!(read[1].metadata.is_empty());
}

#[test]
fn tiles_are_loaded_unless_declared_otherwise() {
    let mut empty = resource("a");
    empty["tiles"] = json!([]);
    let mut declared = resource("b");
    declared["tiles_loaded"] = json!(false);
    let read = parse_business_data_bytes(&file(json!([empty, resource("c"), declared]))).unwrap();
    let loaded: Vec<_> = read.iter().map(|r| r.tiles_loaded).collect();
    assert_eq!(loaded, [Some(true), Some(true), Some(false)]);
}

#[test]
fn a_bare_resource_is_read_as_one() {
    let read = parse_business_data_bytes(resource("a").to_string().as_bytes()).unwrap();
    assert_eq!(read[0].resourceinstance.resourceinstanceid, "a");
    assert_eq!(read[0].tiles_loaded, Some(true));
}

#[test]
fn a_malformed_wrapper_is_reported_as_the_wrapper() {
    let err = parse_business_data_bytes(&file(json!([{"tiles": []}]))).unwrap_err();
    assert!(err.to_string().contains("resourceinstance"), "{err}");
    let err =
        resources_from_business_data_json(r#"{"business_data": {"resources": 5}}"#).unwrap_err();
    assert!(err.contains("invalid type: integer"), "{err}");
}

#[test]
fn a_prebuilds_business_data_loads_recursively_in_path_order() {
    let dir = std::env::temp_dir().join(format!("alizarin-prebuild-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("business_data/nested")).unwrap();
    let write = |path: &str, id: &str| {
        std::fs::write(dir.join(path), file(json!([resource(id)]))).unwrap();
    };
    write("business_data/b.json", "b");
    write("business_data/nested/a.json", "a");
    std::fs::write(dir.join("business_data/not-json.txt"), "ignored").unwrap();

    let mut registry = alizarin_core::StaticResourceRegistry::new();
    let refs = registry
        .load_prebuild_business_data(dir.to_str().unwrap(), true, true)
        .unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    let ids: Vec<_> = refs.iter().map(|r| r.resourceinstanceid.as_str()).collect();
    assert_eq!(ids, ["b", "a"]); // b.json sorts before nested/a.json
    assert!(registry.get_full("a").is_some());
}

#[test]
fn every_entry_point_reads_alike() {
    let bytes = file(json!([resource("a")]));
    let by_bytes = parse_business_data_bytes(&bytes).unwrap();
    let by_json = resources_from_business_data_json(std::str::from_utf8(&bytes).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(&by_bytes).unwrap(),
        serde_json::to_value(&by_json).unwrap()
    );
}
