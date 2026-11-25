//! `clm.reference_change_collection`: point a reference node at another
//! controlled list. One implementation, over the graph as JSON, for every
//! binding: across the C ABI on Python and NAPI ([`crate::c_abi`]), and from
//! JS on WASM.

use alizarin_extension_api::ExtMutationHandler;
use serde_json::{Map, Value};

/// The mutation's registered name.
pub const REFERENCE_CHANGE_COLLECTION: &str = "clm.reference_change_collection";

/// Set the config of the reference node `params.subject` (alias: `node_id`;
/// a node id or alias) so `params.config_key` (default `controlledList`)
/// names the list `params.object` (alias: `collection_id`).
pub fn reference_change_collection(graph: &mut Value, params: &Value) -> Result<(), String> {
    let param = |keys: [&str; 2]| {
        keys.iter()
            .find_map(|k| params.get(*k).and_then(Value::as_str))
            .map(str::to_string)
    };
    let subject = param(["subject", "node_id"]).ok_or("Missing param: subject (node_id)")?;
    let object =
        param(["object", "collection_id"]).ok_or("Missing param: object (collection_id)")?;
    let config_key = params
        .get("config_key")
        .and_then(Value::as_str)
        .unwrap_or("controlledList")
        .to_string();

    let node = graph
        .get_mut("nodes")
        .and_then(Value::as_array_mut)
        .and_then(|nodes| {
            nodes.iter_mut().find(|n| {
                n.get("nodeid").and_then(Value::as_str) == Some(&subject)
                    || n.get("alias").and_then(Value::as_str) == Some(&subject)
            })
        })
        .ok_or_else(|| format!("Node not found: {subject}"))?;

    let datatype = node.get("datatype").and_then(Value::as_str).unwrap_or("");
    if datatype != "reference" && datatype != "reference-list" {
        return Err(format!(
            "Node {subject} is not a reference type: {datatype}"
        ));
    }

    if !node.get("config").is_some_and(Value::is_object) {
        node["config"] = Value::Object(Map::new());
    }
    node["config"][config_key] = Value::String(object);
    Ok(())
}

/// [`reference_change_collection`] as an extension mutation.
pub struct ReferenceChangeCollection;

impl ExtMutationHandler for ReferenceChangeCollection {
    fn apply(&self, graph: &mut Value, params: &Value) -> Result<(), String> {
        reference_change_collection(graph, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn graph() -> Value {
        json!({"nodes": [
            {"nodeid": "root-node-id", "alias": "root", "datatype": "semantic", "config": {}},
            {"nodeid": "ref-node-id", "alias": "type_ref", "datatype": "reference",
             "config": {"controlledList": "old-collection-uuid"}},
        ]})
    }

    fn config(graph: &Value) -> &Value {
        &graph["nodes"][1]["config"]
    }

    #[test]
    fn by_alias_or_id_under_either_param_name() {
        let mut g = graph();
        reference_change_collection(&mut g, &json!({"subject": "type_ref", "object": "new"}))
            .unwrap();
        assert_eq!(config(&g)["controlledList"], "new");
        reference_change_collection(
            &mut g,
            &json!({"node_id": "ref-node-id", "collection_id": "another"}),
        )
        .unwrap();
        assert_eq!(config(&g)["controlledList"], "another");
    }

    #[test]
    fn a_custom_config_key_is_added_beside_the_others() {
        let mut g = graph();
        reference_change_collection(
            &mut g,
            &json!({"subject": "type_ref", "object": "custom", "config_key": "rdmCollection"}),
        )
        .unwrap();
        assert_eq!(config(&g)["controlledList"], "old-collection-uuid");
        assert_eq!(config(&g)["rdmCollection"], "custom");
    }

    #[test]
    fn a_missing_node_or_a_non_reference_node_is_an_error() {
        let mut g = graph();
        let err = reference_change_collection(&mut g, &json!({"subject": "nope", "object": "x"}))
            .unwrap_err();
        assert!(err.contains("Node not found"), "{err}");
        let err = reference_change_collection(&mut g, &json!({"subject": "root", "object": "x"}))
            .unwrap_err();
        assert!(err.contains("not a reference type"), "{err}");
    }

    #[test]
    fn a_null_config_becomes_an_object() {
        let mut g = graph();
        g["nodes"][1]["config"] = Value::Null;
        reference_change_collection(&mut g, &json!({"subject": "type_ref", "object": "new"}))
            .unwrap();
        assert_eq!(config(&g), &json!({"controlledList": "new"}));
    }
}
