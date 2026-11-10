//! The built-in **fold** Aggregate provider: folds a node's values across a
//! slice and writes the result as node values on the calling resource.
//!
//! It has no vocabulary of its own (it knows nothing of spans, roles or Layer
//! models) and no tile shape: the resource's owner creates the tiles and writes
//! everything else in them; this provider only writes the node values its config
//! names.
//! It depends only on fold dispatch (`crate::datatype_fold`), so any datatype
//! whose handler is registered (geometry with `alizarin-geo`, say) folds here.
//!
//! Config (the `config` of an Aggregate `functions_x_graphs` entry), one item
//! per folded value:
//!
//! ```jsonc
//! { "items": [
//!   { "tileid": "<tile on the calling resource>",
//!     "graph": "<member graph id>",
//!     "path": "<dot path to a value node in that graph>",
//!     "where": { "path": "<dot path>", "allowed": ["<value>", ...] },  // optional
//!     "group_by": "<dot path to a concept, domain or boolean node>",   // optional
//!     "into": { "<node alias on the calling graph>": "summary" | "summary_text"
//!                                                   | "values" | "resources" | "skipped" } } ] }
//! ```
//!
//! - **Outputs.** `summary` is the datatype's fold summary (null when nothing
//!   was folded); `summary_text` is the same as a JSON string, for a string
//!   node; `values` counts values folded, `resources` the members that
//!   contributed one, and `skipped` the values the datatype skipped (only
//!   unorderable `edtf`).
//! - **`where`** keeps a member only if one of its tiles has a value at that
//!   path in `allowed`: a string equal to an allowed value, or an array with any
//!   such element. Set membership only, with the semantics of
//!   [`PermissionRule::Conditional`](crate::permissions::PermissionRule), here
//!   applied across the member's tiles.
//! - **`group_by`** folds one summary per value of that node, and writes every
//!   output as an object keyed by value. A member with several values (a list
//!   node) counts towards each; one with none is left out.
//!
//! Unknown fields are rejected, and every path, node and output is checked
//! before the slice is visited. The slice is visited once for all items.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::datatype_fold::{fold_identity, fold_values};
use crate::graph::{
    AggregateConfig, AggregateContext, AggregateProvider, GraphLookup, NodeValueWrite, StaticGraph,
    StaticNode,
};
use crate::graph_model_access::GraphModelAccess;
use crate::instance_wrapper_core::ModelAccess;
use crate::path_resolution::resolve_path_segments;
use crate::slice_source::{SliceMember, SliceSource};

/// The fold provider's UUID in the functions registry. It is registered by
/// [`default_functions_registry`](crate::graph::default_functions_registry).
pub const FOLD_AGGREGATE_PROVIDER_ID: &str = "60000000-0000-0000-0000-000000000004";

/// Datatypes `group_by` accepts: fixed vocabularies, so the number of groups
/// stays bounded.
const GROUPABLE: &[&str] = &[
    "concept",
    "concept-list",
    "domain-value",
    "domain-value-list",
    "boolean",
];

/// The built-in fold Aggregate. See the module docs for its config.
pub struct FoldAggregate;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FoldConfig {
    items: Vec<FoldItem>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FoldItem {
    tileid: String,
    graph: String,
    path: String,
    #[serde(default, rename = "where")]
    where_: Option<Where>,
    #[serde(default)]
    group_by: Option<String>,
    into: BTreeMap<String, Output>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Where {
    path: String,
    allowed: BTreeSet<String>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum Output {
    Summary,
    SummaryText,
    Values,
    Resources,
    Skipped,
}

/// One item, resolved against its member graph and the calling graph.
struct Plan {
    graph_id: String,
    datatype: String,
    node_id: String,
    node_config: Option<Value>,
    identity: Value,
    where_: Option<(String, BTreeSet<String>)>,
    group_node: Option<String>,
    tileid: String,
    into: Vec<(String, Output)>, // (target node id, output)
}

#[derive(Clone)]
struct Acc {
    summary: Value,
    values: usize,
    resources: usize,
    skipped: usize,
}

impl AggregateProvider for FoldAggregate {
    fn aggregate(
        &self,
        _target_id: &str,
        graph: &dyn GraphLookup,
        slice: &dyn SliceSource,
        config: &AggregateConfig,
        context: &AggregateContext<'_>,
    ) -> Result<Vec<NodeValueWrite>, String> {
        let parsed: FoldConfig = serde_json::from_value(config.config.clone())
            .map_err(|e| format!("invalid fold config: {e}"))?;
        let plans = parsed
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| plan(item, graph, context).map_err(|e| format!("item {i}: {e}")))
            .collect::<Result<Vec<_>, _>>()?;

        let empty = |p: &Plan| Acc {
            summary: p.identity.clone(),
            values: 0,
            resources: 0,
            skipped: 0,
        };
        // One accumulator per item, or per group value for a group_by item.
        let mut singles: Vec<Acc> = plans.iter().map(empty).collect();
        let mut groups: Vec<BTreeMap<String, Acc>> =
            plans.iter().map(|_| BTreeMap::new()).collect();

        slice.visit(&mut |member: SliceMember<'_>| {
            for (i, p) in plans.iter().enumerate() {
                if member.graph_id != p.graph_id {
                    continue;
                }
                if let Some((node, allowed)) = &p.where_ {
                    let passes = member
                        .tiles
                        .iter()
                        .filter_map(|t| t.data.get(node))
                        .any(|v| keys(v).iter().any(|k| allowed.contains(k)));
                    if !passes {
                        continue;
                    }
                }
                let values: Vec<&Value> = member
                    .tiles
                    .iter()
                    .filter_map(|t| t.data.get(&p.node_id))
                    .filter(|v| !v.is_null())
                    .collect();
                let fold_into = |acc: &mut Acc| -> Result<(), String> {
                    let step = fold_values(
                        &p.datatype,
                        acc.summary.take(),
                        &values,
                        p.node_config.as_ref(),
                        Some(context.handlers),
                    )
                    .map_err(|e| format!("item {i}, resource {}: {e}", member.resource_id))?;
                    let folded = values.len() - step.skipped;
                    acc.summary = step.summary;
                    acc.values += folded;
                    acc.skipped += step.skipped;
                    if folded > 0 {
                        acc.resources += 1;
                    }
                    Ok(())
                };
                match &p.group_node {
                    None => fold_into(&mut singles[i])?,
                    Some(group_node) => {
                        let group_keys: BTreeSet<String> = member
                            .tiles
                            .iter()
                            .filter_map(|t| t.data.get(group_node))
                            .flat_map(keys)
                            .collect();
                        for key in group_keys {
                            let acc = groups[i].entry(key).or_insert_with(|| empty(p));
                            fold_into(acc)?;
                        }
                    }
                }
            }
            Ok(())
        })?;

        let mut writes = Vec::new();
        for (i, p) in plans.iter().enumerate() {
            for (node_id, output) in &p.into {
                let value = match &p.group_node {
                    None => output_value(&singles[i], *output),
                    Some(_) => Value::Object(
                        groups[i]
                            .iter()
                            .map(|(key, acc)| (key.clone(), output_value(acc, *output)))
                            .collect(),
                    ),
                };
                writes.push(NodeValueWrite {
                    tileid: p.tileid.clone(),
                    node_id: node_id.clone(),
                    value,
                });
            }
        }
        Ok(writes)
    }
}

fn output_value(acc: &Acc, output: Output) -> Value {
    match output {
        Output::Summary if acc.values == 0 => Value::Null,
        Output::Summary => acc.summary.clone(),
        Output::SummaryText if acc.values == 0 => Value::Null,
        Output::SummaryText => Value::String(acc.summary.to_string()),
        Output::Values => acc.values.into(),
        Output::Resources => acc.resources.into(),
        Output::Skipped => acc.skipped.into(),
    }
}

/// A value's keys for `where` and `group_by`: a string, a boolean as
/// `"true"`/`"false"`, or each such element of an array.
fn keys(value: &Value) -> Vec<String> {
    match value {
        Value::String(s) => vec![s.clone()],
        Value::Bool(b) => vec![b.to_string()],
        Value::Array(items) => items.iter().flat_map(keys).collect(),
        _ => Vec::new(),
    }
}

fn plan(
    item: &FoldItem,
    graph: &dyn GraphLookup,
    context: &AggregateContext<'_>,
) -> Result<Plan, String> {
    let member_graph = context
        .graphs
        .graph(&item.graph)
        .ok_or_else(|| format!("graph {} is not available", item.graph))?;
    let model = GraphModelAccess::from_graph(&member_graph);

    let node = resolve(&model, &member_graph, &item.path)?;
    if node.datatype == "semantic" {
        return Err(format!(
            "{}: is a semantic node; only value nodes fold",
            item.path
        ));
    }
    let node_config = (!node.config.is_empty()).then(|| {
        Value::Object(
            node.config
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        )
    });
    let identity = fold_identity(&node.datatype, node_config.as_ref(), Some(context.handlers))
        .ok_or_else(|| {
            format!(
                "{}: datatype {} has no fold (is its handler registered?)",
                item.path, node.datatype
            )
        })?;

    let where_ = match &item.where_ {
        None => None,
        Some(w) => {
            if w.allowed.is_empty() {
                return Err(format!("where {}: 'allowed' is empty", w.path));
            }
            let n = resolve(&model, &member_graph, &w.path)?;
            Some((n.nodeid.clone(), w.allowed.clone()))
        }
    };
    let group_node = match &item.group_by {
        None => None,
        Some(path) => {
            let n = resolve(&model, &member_graph, path)?;
            if !GROUPABLE.contains(&n.datatype.as_str()) {
                return Err(format!(
                    "group_by {path}: datatype {} is not a fixed vocabulary ({})",
                    n.datatype,
                    GROUPABLE.join(", ")
                ));
            }
            Some(n.nodeid.clone())
        }
    };

    if item.into.is_empty() {
        return Err("'into' names no nodes".to_string());
    }
    let into = item
        .into
        .iter()
        .map(|(alias, output)| {
            graph
                .get_node_by_alias(alias)
                .map(|n| (n.nodeid.clone(), *output))
                .ok_or_else(|| format!("into: no node with alias {alias} on the calling graph"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Plan {
        graph_id: item.graph.clone(),
        datatype: node.datatype.clone(),
        node_id: node.nodeid.clone(),
        node_config,
        identity,
        where_,
        group_node,
        tileid: item.tileid.clone(),
        into,
    })
}

fn resolve(
    model: &GraphModelAccess,
    graph: &StaticGraph,
    path: &str,
) -> Result<Arc<StaticNode>, String> {
    let root = model.get_root_node()?;
    let nodes = model.get_nodes().ok_or("graph nodes not built")?;
    let edges = model.get_edges().ok_or("graph edges not built")?;
    resolve_path_segments(path, &root, nodes, edges, model.get_nodegroups())
        .map(|info| info.target_node)
        .map_err(|e| format!("{path}: {e} (in graph {})", graph.graphid))
}
