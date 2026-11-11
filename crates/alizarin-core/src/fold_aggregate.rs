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
    StaticNode, StaticTile,
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
    /// Whether any value has been folded (so `summary` means something).
    seen: bool,
    values: usize,
    resources: usize,
    skipped: usize,
}

impl Acc {
    fn empty(p: &Plan) -> Acc {
        Acc {
            summary: p.identity.clone(),
            seen: false,
            values: 0,
            resources: 0,
            skipped: 0,
        }
    }

    /// An accumulator resumed from the node values an earlier run stored for it
    /// (`stored(output)`): the summary from `summary`, else `summary_text`, and
    /// whichever counts the item writes. Outputs it doesn't write start empty.
    fn resume(p: &Plan, stored: impl Fn(Output) -> Option<Value>) -> Result<Acc, String> {
        let summary = match stored(Output::Summary).filter(|v| !v.is_null()) {
            Some(summary) => Some(summary),
            None => match stored(Output::SummaryText) {
                Some(Value::String(text)) => Some(
                    serde_json::from_str(&text)
                        .map_err(|e| format!("stored summary text is not JSON: {e}"))?,
                ),
                _ => None,
            },
        };
        let count = |output| {
            stored(output)
                .and_then(|v| v.as_u64())
                .map(|n| n as usize)
                .unwrap_or(0)
        };
        let (values, resources, skipped) = (
            count(Output::Values),
            count(Output::Resources),
            count(Output::Skipped),
        );
        Ok(Acc {
            seen: summary.is_some() || values > 0 || resources > 0,
            summary: summary.unwrap_or_else(|| p.identity.clone()),
            values,
            resources,
            skipped,
        })
    }
}

/// One accumulator per item, or per group value for a `group_by` item.
struct Totals {
    singles: Vec<Acc>,
    groups: Vec<BTreeMap<String, Acc>>,
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
        let plans = parse_plans(config, graph, context)?;
        let mut totals = Totals {
            singles: plans.iter().map(Acc::empty).collect(),
            groups: plans.iter().map(|_| BTreeMap::new()).collect(),
        };
        fold_slice(&plans, &mut totals, slice, context)?;
        Ok(writes(&plans, &totals))
    }

    /// Resume each item from the node values stored in `tiles`, fold only the
    /// `added` members, and write the expanded totals. Earlier members are never
    /// revisited; a loose stored summary stays loose.
    fn add(
        &self,
        _target_id: &str,
        graph: &dyn GraphLookup,
        added: &dyn SliceSource,
        tiles: &[StaticTile],
        config: &AggregateConfig,
        context: &AggregateContext<'_>,
    ) -> Option<Result<Vec<NodeValueWrite>, String>> {
        Some((|| {
            let plans = parse_plans(config, graph, context)?;
            let mut totals = resume(&plans, tiles)?;
            fold_slice(&plans, &mut totals, added, context)?;
            Ok(writes(&plans, &totals))
        })())
    }
}

fn parse_plans(
    config: &AggregateConfig,
    graph: &dyn GraphLookup,
    context: &AggregateContext<'_>,
) -> Result<Vec<Plan>, String> {
    let parsed: FoldConfig = serde_json::from_value(config.config.clone())
        .map_err(|e| format!("invalid fold config: {e}"))?;
    parsed
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| plan(item, graph, context).map_err(|e| format!("item {i}: {e}")))
        .collect()
}

/// Each item's totals as an earlier run stored them in `tiles`.
fn resume(plans: &[Plan], tiles: &[StaticTile]) -> Result<Totals, String> {
    let mut totals = Totals {
        singles: Vec::with_capacity(plans.len()),
        groups: Vec::with_capacity(plans.len()),
    };
    for (i, p) in plans.iter().enumerate() {
        let tile = tiles
            .iter()
            .find(|t| t.tileid.as_deref() == Some(p.tileid.as_str()));
        // The value stored for `output`, if the item writes it and it's set.
        let stored = |output: Output| -> Option<&Value> {
            let (node_id, _) = p
                .into
                .iter()
                .find(|(_, o)| std::mem::discriminant(o) == std::mem::discriminant(&output))?;
            tile?.data.get(node_id).filter(|v| !v.is_null())
        };
        let item = |e: String| format!("item {i}: {e}");
        match &p.group_node {
            None => {
                totals
                    .singles
                    .push(Acc::resume(p, |o| stored(o).cloned()).map_err(item)?);
                totals.groups.push(BTreeMap::new());
            }
            Some(_) => {
                let outputs = [
                    Output::Summary,
                    Output::SummaryText,
                    Output::Values,
                    Output::Resources,
                    Output::Skipped,
                ];
                let keys: BTreeSet<String> = outputs
                    .iter()
                    .filter_map(|o| stored(*o)?.as_object())
                    .flat_map(|by_group| by_group.keys().cloned())
                    .collect();
                let mut groups = BTreeMap::new();
                for key in keys {
                    let acc = Acc::resume(p, |o| stored(o)?.get(&key).cloned()).map_err(item)?;
                    groups.insert(key, acc);
                }
                totals.singles.push(Acc::empty(p));
                totals.groups.push(groups);
            }
        }
    }
    Ok(totals)
}

/// Fold `slice`'s members into `totals`, item by item, in one visit.
fn fold_slice(
    plans: &[Plan],
    totals: &mut Totals,
    slice: &dyn SliceSource,
    context: &AggregateContext<'_>,
) -> Result<(), String> {
    let Totals { singles, groups } = totals;
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
                    acc.seen = true;
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
                        let acc = groups[i].entry(key).or_insert_with(|| Acc::empty(p));
                        fold_into(acc)?;
                    }
                }
            }
        }
        Ok(())
    })
}

/// The node values to write for `totals`.
fn writes(plans: &[Plan], totals: &Totals) -> Vec<NodeValueWrite> {
    let mut writes = Vec::new();
    for (i, p) in plans.iter().enumerate() {
        for (node_id, output) in &p.into {
            let value = match &p.group_node {
                None => output_value(&totals.singles[i], *output),
                Some(_) => Value::Object(
                    totals.groups[i]
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
    writes
}

fn output_value(acc: &Acc, output: Output) -> Value {
    match output {
        Output::Summary if !acc.seen => Value::Null,
        Output::Summary => acc.summary.clone(),
        Output::SummaryText if !acc.seen => Value::Null,
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
