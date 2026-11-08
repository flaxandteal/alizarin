//! Fold: reduce the values at a path, across a set of resources, into a summary.
//!
//! A generic primitive. The caller decides which resources (the injected
//! `resource_ids`; alizarin does not own membership) and what a summary means;
//! each datatype decides how its values summarise ([`crate::datatype_fold`]).
//! Results are returned, not written: storing them (e.g. as tiles on another
//! resource) is the caller's schema, minted with
//! [`multi_tile_id`](crate::graph::multi_tile_id) and its layer namespace.
//!
//! One call folds one model. For a set spanning several models, fold each
//! (giving a per-model breakdown) and combine with
//! [`fold_merge`](crate::datatype_fold::fold_merge) for the union.

use serde_json::Value;

use crate::datatype_fold::{fold_identity, fold_values, FoldError};
use crate::extension_type_registry::ExtensionTypeRegistry;
use crate::instance_wrapper_core::ModelAccess;
use crate::path_resolution::resolve_path_segments;
use crate::tile_source::TileSource;

/// The summary of one path over the folded resources.
#[derive(Debug, Clone, PartialEq)]
pub struct FoldResult {
    pub path: String,
    pub datatype: String,
    /// `None` when no values were folded (so "no data" can't be mistaken for a
    /// summary).
    pub summary: Option<Value>,
    /// Values folded (nulls are not values).
    pub values: usize,
    /// Values skipped: only `edtf` values the date parser can't order yet.
    pub skipped: usize,
}

struct Target {
    path: String,
    node_id: String,
    datatype: String,
    config: Option<Value>,
    summary: Value,
    values: usize,
    skipped: usize,
}

/// Fold the value at each of `paths` over `resource_ids`, reading tiles from
/// `tiles`.
///
/// Each path must resolve to a value node (not a semantic one) whose datatype
/// has a summary. Fails on the first problem (an unresolvable path, a resource
/// the source doesn't have, a value its datatype can't summarise), naming the
/// path and resource. A resource of another model simply has no tiles in these
/// nodegroups and contributes nothing.
pub fn fold_resources(
    tiles: &dyn TileSource,
    resource_ids: &[String],
    model: &dyn ModelAccess,
    paths: &[String],
    registry: Option<&ExtensionTypeRegistry>,
) -> Result<Vec<FoldResult>, FoldError> {
    let root = model.get_root_node().map_err(FoldError)?;
    let nodes = model
        .get_nodes()
        .ok_or_else(|| FoldError("model nodes not built".into()))?;
    let edges = model
        .get_edges()
        .ok_or_else(|| FoldError("model edges not built".into()))?;

    let mut targets = Vec::with_capacity(paths.len());
    // Nodegroup -> indices into `targets`, so each resource's tiles for a
    // nodegroup are loaded once however many paths share it.
    let mut by_nodegroup: indexmap::IndexMap<String, Vec<usize>> = indexmap::IndexMap::new();
    for path in paths {
        let info = resolve_path_segments(path, &root, nodes, edges, model.get_nodegroups())
            .map_err(|e| FoldError(format!("{path}: {e}")))?;
        let node = &info.target_node;
        if node.datatype == "semantic" {
            return Err(FoldError(format!(
                "{path}: is a semantic node; only value nodes fold"
            )));
        }
        let config = (!node.config.is_empty()).then(|| {
            Value::Object(
                node.config
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            )
        });
        let summary =
            fold_identity(&node.datatype, config.as_ref(), registry).ok_or_else(|| {
                FoldError(format!("{path}: datatype {} has no summary", node.datatype))
            })?;
        by_nodegroup
            .entry(info.nodegroup_id.clone())
            .or_default()
            .push(targets.len());
        targets.push(Target {
            path: path.clone(),
            node_id: node.nodeid.clone(),
            datatype: node.datatype.clone(),
            config,
            summary,
            values: 0,
            skipped: 0,
        });
    }

    for resource_id in resource_ids {
        for (nodegroup_id, indices) in &by_nodegroup {
            let resource_tiles = tiles
                .load_tiles(resource_id, Some(nodegroup_id))
                .map_err(|e| FoldError(format!("resource {resource_id}: {e}")))?;
            for &i in indices {
                let target = &mut targets[i];
                let values: Vec<&Value> = resource_tiles
                    .iter()
                    .filter(|t| &t.nodegroup_id == nodegroup_id)
                    .filter_map(|t| t.data.get(&target.node_id))
                    .filter(|v| !v.is_null())
                    .collect();
                if values.is_empty() {
                    continue;
                }
                let acc = std::mem::take(&mut target.summary);
                let step = fold_values(
                    &target.datatype,
                    acc,
                    &values,
                    target.config.as_ref(),
                    registry,
                )
                .map_err(|e| {
                    FoldError(format!("{} in resource {resource_id}: {e}", target.path))
                })?;
                target.summary = step.summary;
                target.values += values.len() - step.skipped;
                target.skipped += step.skipped;
            }
        }
    }

    Ok(targets
        .into_iter()
        .map(|t| FoldResult {
            summary: (t.values > 0).then_some(t.summary),
            path: t.path,
            datatype: t.datatype,
            values: t.values,
            skipped: t.skipped,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_business_data_loader::{
        build_resources_from_business_csv, BusinessDataCsvOptions,
    };
    use crate::csv_model_loader::build_graph_from_model_csvs;
    use crate::graph::StaticResourceRegistry;
    use crate::graph_model_access::GraphModelAccess;
    use crate::graph_mutator::MutatorOptions;
    use serde_json::json;

    const C: &str = "http://www.cidoc-crm.org/cidoc-crm/";

    fn fixture() -> (GraphModelAccess, StaticResourceRegistry, Vec<String>) {
        let row = |parent: &str, alias: &str, datatype: &str, card: &str| {
            format!("{parent},{alias},{alias},{datatype},{card},{C}E1,{C}P1,,,,,,")
        };
        let nodes = [
            "parent_alias,alias,name,datatype,cardinality,ontology_class,parent_property,description,collection_name,required,searchable,exportable,sortorder".to_string(),
            row("", "measurements", "semantic", "n"),
            row("measurements", "height", "number", "1"),
            row("", "built", "date", "1"),
            row("", "label", "string", "1"),
        ]
        .join("\n");
        let (graph, collections) = build_graph_from_model_csvs(
            &format!("name,ontology_class,author,description,is_resource\nTower,{C}E22,,,true"),
            &nodes,
            None,
            "https://example.org/test",
            MutatorOptions::default(),
        )
        .unwrap();
        let csv = "ResourceID,height,built,label\n\
                   a,12,1850-03-01,A\n\
                   a,30.5,,\n\
                   b,-2,1902,B\n\
                   c,,,C";
        let resources = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        )
        .unwrap();
        let ids = resources
            .iter()
            .map(|r| r.resourceinstance.resourceinstanceid.clone())
            .collect();
        let mut registry = StaticResourceRegistry::new();
        registry.merge_from_resources(&resources, true, false);
        (GraphModelAccess::from_graph(&graph), registry, ids)
    }

    fn paths(p: &[&str]) -> Vec<String> {
        p.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn folds_each_path_across_the_resources() {
        let (model, registry, ids) = fixture();
        let results = fold_resources(
            &registry,
            &ids,
            &model,
            &paths(&["measurements.height", "built"]),
            None,
        )
        .unwrap();

        let height = &results[0];
        assert_eq!(height.datatype, "number");
        assert_eq!(height.values, 3, "two heights on a, one on b, none on c");
        assert_eq!(
            height.summary,
            Some(json!({ "count": 3, "min": -2.0, "max": 30.5 })),
            "summaries keep the raw values (CSV numbers load as floats)"
        );

        let built = &results[1];
        assert_eq!(built.values, 2);
        assert_eq!(built.summary.as_ref().unwrap()["min"], json!("1850-03-01"));
        assert_eq!(built.summary.as_ref().unwrap()["max"], json!("1902"));
    }

    #[test]
    fn resource_order_does_not_change_the_summary() {
        let (model, registry, ids) = fixture();
        let p = paths(&["measurements.height", "built"]);
        let forward = fold_resources(&registry, &ids, &model, &p, None).unwrap();
        let mut reversed_ids = ids.clone();
        reversed_ids.reverse();
        let reversed = fold_resources(&registry, &reversed_ids, &model, &p, None).unwrap();
        assert_eq!(forward, reversed);
    }

    #[test]
    fn only_the_given_resources_are_folded_and_none_means_no_summary() {
        let (model, registry, ids) = fixture();
        let only_c = fold_resources(
            &registry,
            &ids[2..],
            &model,
            &paths(&["measurements.height"]),
            None,
        )
        .unwrap();
        assert_eq!(only_c[0].values, 0);
        assert_eq!(only_c[0].summary, None);
    }

    #[test]
    fn problems_fail_the_fold() {
        let (model, registry, ids) = fixture();
        let fold =
            |p: &[&str], ids: &[String]| fold_resources(&registry, ids, &model, &paths(p), None);
        assert!(fold(&["measurements"], &ids).is_err(), "semantic node");
        assert!(fold(&["label"], &ids).is_err(), "string has no summary");
        assert!(fold(&["nope"], &ids).is_err(), "unknown path");
        assert!(
            fold(&["built"], &["not-a-resource".to_string()]).is_err(),
            "resource missing from the source"
        );
    }
}
