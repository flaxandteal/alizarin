//! Whether two nodes are the same node apart from their ids: the per-node
//! equality a graph diff needs, and how a mutator's output is checked against
//! an expected node whose ids it generated. One rule for every binding (WASM's
//! `StaticNode.compare`, NAPI's `compareNodes`, Python's `compare_nodes`).

use super::StaticNode;
use serde::Serialize;
use serde_json::Value;

/// The ids, compared last and only where both sides have them.
const IDS: [&str; 3] = ["graph_id", "nodegroup_id", "nodeid"];

/// How two nodes compare. Fields other than the ids are compared first; then
/// the ids, coarsest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeComparison {
    /// A field other than the ids differs.
    Different,
    /// Both have a graph id, and they differ.
    GraphIdConflict,
    /// Both have a nodegroup id, and they differ.
    NodegroupIdConflict,
    /// Both have a node id, and they differ.
    NodeidConflict,
    /// The same apart from ids, with no conflict, but some id is set on one
    /// side only.
    SameIdsPartial,
    /// The same, each id set and equal on both sides, or absent on both.
    Same,
}

impl NodeComparison {
    /// Its name, as it serialises (`"same"`, `"nodeid_conflict"`, ...).
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeComparison::Different => "different",
            NodeComparison::GraphIdConflict => "graph_id_conflict",
            NodeComparison::NodegroupIdConflict => "nodegroup_id_conflict",
            NodeComparison::NodeidConflict => "nodeid_conflict",
            NodeComparison::SameIdsPartial => "same_ids_partial",
            NodeComparison::Same => "same",
        }
    }
}

/// An id as set: a non-empty string (absent, `null` and `""` are unset).
fn id<'a>(node: &'a Value, key: &str) -> Option<&'a str> {
    node.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

/// Two field values alike: absent is `null`, and for `config` also `{}`.
fn alike(key: &str, a: Option<&Value>, b: Option<&Value>) -> bool {
    let unset = |v: Option<&Value>| match v {
        None | Some(Value::Null) => true,
        Some(Value::Object(o)) => key == "config" && o.is_empty(),
        _ => false,
    };
    if unset(a) && unset(b) {
        return true;
    }
    a == b
}

/// Compare two nodes given as JSON objects, either of which may be partial
/// (fields it lacks count as `null`).
pub fn compare_nodes(a: &Value, b: &Value) -> NodeComparison {
    let keys = a
        .as_object()
        .into_iter()
        .chain(b.as_object())
        .flat_map(|o| o.keys())
        .filter(|k| !IDS.contains(&k.as_str()));
    for key in keys {
        if !alike(key, a.get(key), b.get(key)) {
            return NodeComparison::Different;
        }
    }
    let ids = IDS.map(|key| (id(a, key), id(b, key)));
    let conflicts = [
        NodeComparison::GraphIdConflict,
        NodeComparison::NodegroupIdConflict,
        NodeComparison::NodeidConflict,
    ];
    for ((a, b), conflict) in ids.iter().zip(conflicts) {
        if let (Some(a), Some(b)) = (a, b) {
            if a != b {
                return conflict;
            }
        }
    }
    if ids.iter().all(|(a, b)| a.is_some() == b.is_some()) {
        NodeComparison::Same
    } else {
        NodeComparison::SameIdsPartial
    }
}

/// [`compare_nodes`] over JSON strings: the bindings' entry point.
pub fn compare_nodes_from_json(a: &str, b: &str) -> Result<NodeComparison, String> {
    let parse = |s: &str| serde_json::from_str::<Value>(s);
    match (parse(a), parse(b)) {
        (Ok(a), Ok(b)) => Ok(compare_nodes(&a, &b)),
        (Err(e), _) | (_, Err(e)) => Err(format!("Invalid node: {e}")),
    }
}

impl StaticNode {
    /// [`compare_nodes`] on two whole nodes.
    pub fn compare(&self, other: &StaticNode) -> NodeComparison {
        let json = |n: &StaticNode| serde_json::to_value(n).unwrap_or(Value::Null);
        compare_nodes(&json(self), &json(other))
    }
}

#[cfg(test)]
mod tests {
    use super::NodeComparison as N;
    use super::*;
    use serde_json::json;

    fn node() -> Value {
        json!({
            "nodeid": "n1", "graph_id": "g1", "nodegroup_id": "ng1",
            "alias": "name", "datatype": "string", "config": {}, "name": "Name",
        })
    }

    #[test]
    fn a_node_is_the_same_as_itself() {
        assert_eq!(compare_nodes(&node(), &node()), N::Same);
    }

    #[test]
    fn a_field_other_than_the_ids_makes_it_different() {
        let mut other = node();
        other["datatype"] = json!("number");
        assert_eq!(compare_nodes(&node(), &other), N::Different);
    }

    #[test]
    fn ids_conflict_coarsest_first() {
        let mut other = node();
        other["nodeid"] = json!("n2");
        assert_eq!(compare_nodes(&node(), &other), N::NodeidConflict);
        other["nodegroup_id"] = json!("ng2");
        assert_eq!(compare_nodes(&node(), &other), N::NodegroupIdConflict);
        other["graph_id"] = json!("g2");
        assert_eq!(compare_nodes(&node(), &other), N::GraphIdConflict);
    }

    #[test]
    fn a_partial_expected_node_matches_without_its_ids() {
        let expected = json!({"alias": "name", "datatype": "string", "config": null});
        let mut node = node();
        node.as_object_mut().unwrap().remove("name");
        assert_eq!(compare_nodes(&node, &expected), N::SameIdsPartial);
        let mut no_config = expected.clone();
        no_config.as_object_mut().unwrap().remove("config");
        assert_eq!(compare_nodes(&node, &no_config), N::SameIdsPartial);
    }

    #[test]
    fn its_name_is_its_serialisation() {
        for c in [N::Different, N::GraphIdConflict, N::NodegroupIdConflict] {
            assert_eq!(serde_json::to_value(c).unwrap(), json!(c.as_str()));
        }
        for c in [N::NodeidConflict, N::SameIdsPartial, N::Same] {
            assert_eq!(serde_json::to_value(c).unwrap(), json!(c.as_str()));
        }
    }

    #[test]
    fn absent_null_and_empty_ids_are_unset() {
        let mut a = node();
        a["nodegroup_id"] = json!(null);
        let mut b = node();
        b["nodegroup_id"] = json!("");
        assert_eq!(compare_nodes(&a, &b), N::Same);
    }
}
