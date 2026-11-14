//! On-load derive functions run by core's load path: every binding loads
//! through `load_permitted_tiles` / `merge_permitted_tiles` /
//! `merge_fetched_tiles`, which add on-load tiles that `stored_tiles` (what
//! exports write) leaves out. Integration test (public API only); the provider
//! goes in the process-wide registry under an id unique to this file.

use std::sync::Arc;

use alizarin_core::graph_model_access::GraphModelAccess;
use alizarin_core::{
    build_graph_from_model_csvs, derived_tile_id, with_global_functions_registry_mut,
    ComputeTilesConfig, DeriveProvider, GraphLookup, PseudoListCore, RegisteredFunction,
    ResourceInstanceWrapperCore, StaticGraph, StaticResourceMetadata, StaticTile, TileSource,
    TileSourceError, COMPUTE_TILES_FUNCTION_ID,
};
use serde_json::{json, Value};

const C: &str = "http://www.cidoc-crm.org/cidoc-crm/";
const NODES_HEADER: &str = "parent_alias,alias,name,datatype,cardinality,ontology_class,\
                            parent_property,description,collection_name,required,searchable,\
                            exportable,sortorder";
const FORMS: &str = "on-load-derive-test-forms";

/// A graph with a `headword` and a `form` string, each in its own nodegroup,
/// declaring `form` as derived from `headword` in `mode`.
fn graph(mode: &str) -> StaticGraph {
    let rows = [
        ",headword,non-localized-string,1",
        ",form,non-localized-string,n",
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
        &format!("name,ontology_class,author,description,is_resource\nWord,{C}E1,,,true"),
        &format!("{NODES_HEADER}\n{}", rows.join("\n")),
        None,
        "https://example.org/test",
        Default::default(),
    )
    .expect("graph");
    let fxg = json!({
        "id": "fxg-forms",
        "function_id": COMPUTE_TILES_FUNCTION_ID,
        "graph_id": graph.graphid,
        "config": {
            "provider": FORMS,
            "nodegroup": nodegroup(&graph, "form"),
            "mode": mode,
            "sources": [nodegroup(&graph, "headword")],
        }
    });
    graph.functions_x_graphs = Some(serde_json::from_value(Value::Array(vec![fxg])).unwrap());
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

/// Two forms per headword: itself, and with "-anna" appended.
struct Forms;
impl DeriveProvider for Forms {
    fn derive(
        &self,
        resource_id: &str,
        graph: &dyn GraphLookup,
        tiles: &[StaticTile],
        config: &ComputeTilesConfig,
    ) -> Result<Vec<StaticTile>, String> {
        let headword = node_id(graph, "headword");
        let word = tiles
            .iter()
            .find_map(|t| t.data.get(&headword).and_then(Value::as_str))
            .ok_or("no headword")?;
        Ok([word.to_string(), format!("{word}-anna")]
            .into_iter()
            .map(|form| {
                let mut tile = StaticTile::new_empty(config.nodegroup.clone());
                tile.resourceinstance_id = resource_id.to_string();
                tile.data.insert(node_id(graph, "form"), json!(form));
                tile
            })
            .collect())
    }
}

fn register() {
    with_global_functions_registry_mut(|r| {
        r.register(FORMS, RegisteredFunction::Derive(Arc::new(Forms)))
    });
}

fn setup(mode: &str) -> (StaticGraph, GraphModelAccess, ResourceInstanceWrapperCore) {
    register();
    let graph = graph(mode);
    let model = GraphModelAccess::new_eager(Arc::new(graph.clone()), true);
    let mut core = ResourceInstanceWrapperCore::new(graph.graphid.clone());
    core.resource_instance = Some(StaticResourceMetadata {
        descriptors: Default::default(),
        graph_id: graph.graphid.clone(),
        name: "r1".to_string(),
        resourceinstanceid: "r1".to_string(),
        publication_id: None,
        principaluser_id: None,
        legacyid: None,
        graph_publication_id: None,
        createdtime: None,
        lastmodified: None,
    });
    (graph, model, core)
}

fn headword_tile(graph: &StaticGraph, word: &str) -> StaticTile {
    let mut tile = StaticTile::new_empty(nodegroup(graph, "headword"));
    tile.tileid = Some("hw-1".to_string());
    tile.resourceinstance_id = "r1".to_string();
    tile.data.insert(node_id(graph, "headword"), json!(word));
    tile
}

fn forms(graph: &StaticGraph, core: &ResourceInstanceWrapperCore) -> Vec<String> {
    let ng = nodegroup(graph, "form");
    let form = node_id(graph, "form");
    let mut forms: Vec<String> = core
        .tiles
        .as_ref()
        .map(|t| {
            t.values()
                .filter(|t| t.nodegroup_id == ng)
                .filter_map(|t| t.data.get(&form).and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    forms.sort();
    forms
}

#[test]
fn loading_adds_on_load_tiles_that_exports_leave_out() {
    let (graph, model, mut core) = setup("on_load");
    core.load_permitted_tiles(vec![headword_tile(&graph, "bád")], &model);

    assert_eq!(forms(&graph, &core), vec!["bád", "bád-anna"]);
    let ng = nodegroup(&graph, "form");
    for k in 0..2 {
        let id = derived_tile_id("r1", &ng, None, FORMS, k);
        assert!(
            core.on_load_derived.contains(&id),
            "slot {k} has its derived id"
        );
    }
    let stored = core.stored_tiles();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].tileid.as_deref(), Some("hw-1"));
}

#[test]
fn reloading_recomputes_rather_than_duplicating() {
    let (graph, model, mut core) = setup("on_load");
    core.load_permitted_tiles(vec![headword_tile(&graph, "bád")], &model);
    core.merge_permitted_tiles(vec![headword_tile(&graph, "cat")], &model);
    assert_eq!(forms(&graph, &core), vec!["cat", "cat-anna"]);
    assert_eq!(core.on_load_derived.len(), 2);
}

#[test]
fn an_attested_tile_in_the_target_wins() {
    let (graph, model, mut core) = setup("on_load");
    let mut attested = StaticTile::new_empty(nodegroup(&graph, "form"));
    attested.tileid = Some("form-1".to_string());
    attested.resourceinstance_id = "r1".to_string();
    attested
        .data
        .insert(node_id(&graph, "form"), json!("attested"));
    core.load_permitted_tiles(vec![headword_tile(&graph, "bád"), attested], &model);
    assert_eq!(forms(&graph, &core), vec!["attested"]);
    assert!(core.on_load_derived.is_empty());
}

#[test]
fn lazy_fetches_the_sources_before_the_target_counts_as_loaded() {
    let (graph, model, mut core) = setup("on_load");
    core.set_lazy(true);
    let forms_ng = nodegroup(&graph, "form");
    let headword_ng = nodegroup(&graph, "headword");

    // Reading the target asks for it and its source.
    assert_eq!(
        core.nodegroups_to_fetch(&forms_ng, &model),
        vec![forms_ng.clone(), headword_ng.clone()]
    );

    // The target fetched first, with no stored tiles: it stays unloaded, as
    // its function can't run yet, so it isn't read as empty.
    core.merge_fetched_tiles(vec![], Some(&forms_ng), &model);
    assert!(!core.is_nodegroup_loaded(&forms_ng));
    assert_eq!(
        core.nodegroups_to_fetch(&forms_ng, &model),
        vec![forms_ng.clone(), headword_ng.clone()]
    );

    core.merge_fetched_tiles(
        vec![headword_tile(&graph, "bád")],
        Some(&headword_ng),
        &model,
    );
    assert_eq!(forms(&graph, &core), vec!["bád", "bád-anna"]);
    assert!(core.is_nodegroup_loaded(&forms_ng));
    assert!(core.nodegroups_to_fetch(&forms_ng, &model).is_empty());
}

#[test]
fn a_lazy_lookup_names_every_nodegroup_to_fetch() {
    let (graph, model, mut core) = setup("on_load");
    core.set_lazy(true);
    let root = graph.root.nodeid.clone();
    let err = core
        .get_semantic_child_value(None, &root, None, "form", &model)
        .expect_err("not loaded");
    assert_eq!(
        err.to_string(),
        format!(
            "TILES_NOT_LOADED:{},{}",
            nodegroup(&graph, "form"),
            nodegroup(&graph, "headword")
        )
    );
    let mut missing = core.missing_nodegroups_for_children(&root, &model, true);
    missing.sort();
    let mut expected = vec![nodegroup(&graph, "form"), nodegroup(&graph, "headword")];
    expected.sort();
    assert_eq!(missing, expected);
}

#[test]
fn changed_on_load_tiles_drop_the_targets_cached_values() {
    let (graph, model, mut core) = setup("on_load");
    core.load_permitted_tiles(vec![headword_tile(&graph, "bád")], &model);
    core.store_pseudo(
        "form".to_string(),
        PseudoListCore::from_values_with_cardinality("form".to_string(), vec![], false),
    );
    core.store_pseudo(
        "headword".to_string(),
        PseudoListCore::from_values_with_cardinality("headword".to_string(), vec![], true),
    );

    // Recomputed unchanged: the cache stays.
    core.merge_permitted_tiles(vec![headword_tile(&graph, "bád")], &model);
    assert!(core.get_cached_pseudo("form").is_some());

    // Changed: the target's cached values go; others stay.
    core.merge_permitted_tiles(vec![headword_tile(&graph, "cat")], &model);
    assert!(core.get_cached_pseudo("form").is_none());
    assert!(core.get_cached_pseudo("headword").is_some());
}

/// Holds the headword nodegroup's tiles for `r1`, and nothing else.
struct HeadwordSource(StaticTile);
impl TileSource for HeadwordSource {
    fn load_tiles(
        &self,
        resource_id: &str,
        nodegroup_id: Option<&str>,
    ) -> Result<Vec<StaticTile>, TileSourceError> {
        if resource_id == "r1" && nodegroup_id == Some(self.0.nodegroup_id.as_str()) {
            Ok(vec![self.0.clone()])
        } else {
            Err(TileSourceError::ResourceNotFound {
                resource_id: resource_id.to_string(),
            })
        }
    }
}

#[test]
fn the_tile_source_serves_what_it_holds_and_returns_the_rest() {
    let (graph, model, mut core) = setup("on_load");
    core.set_lazy(true);
    core.tile_source = Some(Arc::new(HeadwordSource(headword_tile(&graph, "bád"))));
    let forms_ng = nodegroup(&graph, "form");
    let headword_ng = nodegroup(&graph, "headword");

    let remaining = core
        .fetch_from_tile_source(&[forms_ng.clone(), headword_ng.clone()], &model)
        .expect("fetch");
    assert_eq!(remaining, vec![forms_ng.clone()]);
    assert!(core.is_nodegroup_loaded(&headword_ng));
    // The source alone was enough for the function to run.
    assert_eq!(forms(&graph, &core), vec!["bád", "bád-anna"]);

    // Without a source, everything is the host's to fetch.
    core.tile_source = None;
    assert_eq!(
        core.fetch_from_tile_source(&[forms_ng.clone()], &model)
            .expect("no source"),
        vec![forms_ng]
    );
}

#[test]
fn stored_declarations_are_not_derived_on_load() {
    let (graph, model, mut core) = setup("stored");
    core.load_permitted_tiles(vec![headword_tile(&graph, "bád")], &model);
    assert!(forms(&graph, &core).is_empty());
    assert_eq!(core.stored_tiles().len(), 1);
}
