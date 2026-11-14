//! Editing a loaded resource through `ResourceEdit`: what every binding's ORM
//! assignment sugar commits. Integration test (public API only). The current
//! language is process-wide, so every test here uses "ga".

use std::sync::Arc;

use alizarin_core::graph_model_access::GraphModelAccess;
use alizarin_core::type_coercion::set_current_language;
use alizarin_core::{
    build_graph_from_model_csvs, canonical_tile_id, ComputeTilesConfig, DeriveProvider,
    FunctionsRegistry, GraphLookup, RegisteredFunction, ResourceEdit, ResourceInstanceWrapperCore,
    StaticGraph, StaticResourceMetadata, StaticTile, COMPUTE_TILES_FUNCTION_ID,
};
use serde_json::{json, Map, Value};

const C: &str = "http://www.cidoc-crm.org/cidoc-crm/";
const NODES_HEADER: &str = "parent_alias,alias,name,datatype,cardinality,ontology_class,\
                            parent_property,description,collection_name,required,searchable,\
                            exportable,sortorder";

/// `title` (translatable, 1), `nickname` (n), `shout` (1), `label` (1): each
/// in its own top-level nodegroup. `shout` is a stored Derive target of
/// `nickname` when `derive` is set.
fn graph(derive: bool) -> StaticGraph {
    let rows = [
        ",title,string,1",
        ",nickname,non-localized-string,n",
        ",shout,non-localized-string,1",
        ",label,non-localized-string,1",
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
        &format!("name,ontology_class,author,description,is_resource\nPlace,{C}E1,,,true"),
        &format!("{NODES_HEADER}\n{}", rows.join("\n")),
        None,
        "https://example.org/test",
        Default::default(),
    )
    .expect("graph");
    if derive {
        let fxg = json!({
            "id": "fxg-shout",
            "function_id": COMPUTE_TILES_FUNCTION_ID,
            "graph_id": graph.graphid,
            "config": {
                "provider": "shout",
                "nodegroup": ng(&graph, "shout"),
                "mode": "stored",
                "sources": [ng(&graph, "nickname")],
            }
        });
        graph.functions_x_graphs = Some(serde_json::from_value(json!([fxg])).unwrap());
    }
    graph
}

fn node_id(graph: &dyn GraphLookup, alias: &str) -> String {
    graph.get_node_by_alias(alias).expect("node").nodeid.clone()
}

fn ng(graph: &dyn GraphLookup, alias: &str) -> String {
    graph
        .get_node_by_alias(alias)
        .and_then(|n| n.nodegroup_id.clone())
        .expect("nodegroup")
}

/// `shout` = every nickname upper-cased, joined.
struct Shout;
impl DeriveProvider for Shout {
    fn derive(
        &self,
        resource_id: &str,
        graph: &dyn GraphLookup,
        tiles: &[StaticTile],
        config: &ComputeTilesConfig,
    ) -> Result<Vec<StaticTile>, String> {
        let nickname = node_id(graph, "nickname");
        let mut all: Vec<String> = tiles
            .iter()
            .filter_map(|t| t.data.get(&nickname).and_then(Value::as_str))
            .map(str::to_uppercase)
            .collect();
        all.sort();
        let mut tile = StaticTile::new_empty(config.nodegroup.clone());
        tile.resourceinstance_id = resource_id.to_string();
        tile.data
            .insert(node_id(graph, "shout"), json!(all.join(" ")));
        Ok(vec![tile])
    }
}

/// A wrapper for resource `r1` holding `tiles`, and its model.
fn setup(
    derive: bool,
    tiles: Vec<StaticTile>,
) -> (StaticGraph, GraphModelAccess, ResourceInstanceWrapperCore) {
    set_current_language("ga");
    let graph = graph(derive);
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
    core.load_permitted_tiles(tiles, &model);
    (graph, model, core)
}

fn title_tile(graph: &StaticGraph) -> StaticTile {
    let ng = ng(graph, "title");
    let mut tile = StaticTile::new_empty(ng.clone());
    tile.tileid = Some(canonical_tile_id("r1", &ng, None));
    tile.resourceinstance_id = "r1".to_string();
    tile.data.insert(
        node_id(graph, "title"),
        json!({"en": {"value": "Town", "direction": "ltr"}}),
    );
    tile
}

fn data<'a>(
    core: &'a ResourceInstanceWrapperCore,
    graph: &StaticGraph,
    alias: &str,
) -> Vec<&'a Value> {
    let ng = ng(graph, alias);
    let node = node_id(graph, alias);
    core.tiles
        .as_ref()
        .unwrap()
        .values()
        .filter(|t| t.nodegroup_id == ng)
        .filter_map(|t| t.data.get(&node))
        .collect()
}

#[test]
fn a_plain_string_sets_the_current_language_and_keeps_the_others() {
    let (graph, model, mut core) = setup(false, vec![]);
    core.load_permitted_tiles(vec![title_tile(&graph)], &model);

    let mut edit = ResourceEdit::new(&core).unwrap();
    edit.set_value(None, None, "title", &json!("Baile"), &model, None)
        .unwrap();
    let delta = edit
        .commit(&mut core, &model, &FunctionsRegistry::new())
        .unwrap();

    assert_eq!(delta.upserted.len(), 1);
    assert!(delta.removed.is_empty());
    assert_eq!(
        data(&core, &graph, "title"),
        vec![&json!({
            "en": {"value": "Town", "direction": "ltr"},
            "ga": {"value": "Baile", "direction": "ltr"},
        })]
    );

    // An object replaces every language.
    let mut edit = ResourceEdit::new(&core).unwrap();
    edit.set_value(None, None, "title", &json!({"en": "City"}), &model, None)
        .unwrap();
    edit.commit(&mut core, &model, &FunctionsRegistry::new())
        .unwrap();
    assert_eq!(
        data(&core, &graph, "title"),
        vec![&json!({"en": {"value": "City", "direction": "ltr"}})]
    );
}

#[test]
fn a_missing_single_valued_tile_is_created_with_its_canonical_id() {
    let (graph, model, mut core) = setup(false, vec![]);
    let mut edit = ResourceEdit::new(&core).unwrap();
    let id = edit
        .set_value(None, None, "label", &json!("x"), &model, None)
        .unwrap();
    assert_eq!(id, canonical_tile_id("r1", &ng(&graph, "label"), None));
    let delta = edit
        .commit(&mut core, &model, &FunctionsRegistry::new())
        .unwrap();
    assert_eq!(delta.upserted[0].tileid.as_deref(), Some(id.as_str()));
    assert_eq!(data(&core, &graph, "label"), vec![&json!("x")]);
}

#[test]
fn multi_valued_tiles_are_added_and_removed() {
    let (graph, model, mut core) = setup(false, vec![]);
    let mut edit = ResourceEdit::new(&core).unwrap();
    let mut values = Map::new();
    values.insert("nickname".into(), json!("an baile"));
    let first = edit
        .add_tile("nickname", None, &values, &model, None)
        .unwrap();
    values.insert("nickname".into(), json!("an chathair"));
    let second = edit
        .add_tile("nickname", None, &values, &model, None)
        .unwrap();
    assert_ne!(first, second);
    edit.commit(&mut core, &model, &FunctionsRegistry::new())
        .unwrap();
    assert_eq!(core.get_tile(&first).unwrap().sortorder, Some(0));
    assert_eq!(core.get_tile(&second).unwrap().sortorder, Some(1));

    let mut edit = ResourceEdit::new(&core).unwrap();
    edit.remove_tile(&first, &model).unwrap();
    let delta = edit
        .commit(&mut core, &model, &FunctionsRegistry::new())
        .unwrap();
    assert_eq!(delta.removed, vec![first.clone()]);
    assert!(delta.upserted.is_empty());
    assert!(core.get_tile(&first).is_none());
    assert_eq!(data(&core, &graph, "nickname"), vec![&json!("an chathair")]);
}

#[test]
fn committing_reruns_stored_derives_for_the_touched_nodegroups_only() {
    let (graph, model, mut core) = setup(true, vec![]);
    let mut functions = FunctionsRegistry::new();
    functions.register("shout", RegisteredFunction::Derive(Arc::new(Shout)));

    // Touching only `label`: the derive doesn't run.
    let mut edit = ResourceEdit::new(&core).unwrap();
    edit.set_value(None, None, "label", &json!("x"), &model, None)
        .unwrap();
    edit.commit(&mut core, &model, &functions).unwrap();
    assert!(data(&core, &graph, "shout").is_empty());

    // Adding a nickname: it does, in the same delta.
    let mut edit = ResourceEdit::new(&core).unwrap();
    let mut values = Map::new();
    values.insert("nickname".into(), json!("baile"));
    edit.add_tile("nickname", None, &values, &model, None)
        .unwrap();
    let delta = edit.commit(&mut core, &model, &functions).unwrap();
    assert_eq!(delta.upserted.len(), 2);
    assert_eq!(data(&core, &graph, "shout"), vec![&json!("BAILE")]);
}

#[test]
fn edits_the_graph_rules_out_fail_when_made() {
    let (_graph, model, core) = setup(true, vec![]);
    let mut edit = ResourceEdit::new(&core).unwrap();

    let err = |r: Result<String, String>| r.expect_err("should fail");
    assert!(err(edit.set_value(None, None, "nope", &json!(1), &model, None)).contains("no node"));
    assert!(
        err(edit.set_value(None, None, "nickname", &json!("x"), &model, None))
            .contains("multi-valued")
    );
    assert!(
        err(edit.set_value(None, None, "shout", &json!("X"), &model, None)).contains("derived")
    );
    assert!(err(edit.set_value(
        None,
        None,
        "title",
        &json!(["not", "a", "string"]),
        &model,
        None
    ))
    .contains("title"));
    assert!(edit.is_empty(), "failed edits change nothing");
}

#[test]
fn a_lazy_wrapper_can_only_edit_loaded_nodegroups() {
    let (graph, model, mut core) = setup(false, vec![]);
    core.set_lazy(true);
    core.mark_nodegroup_loaded(&ng(&graph, "label"));
    let mut edit = ResourceEdit::new(&core).unwrap();
    edit.set_value(None, None, "label", &json!("x"), &model, None)
        .unwrap();
    let err = edit
        .set_value(None, None, "title", &json!("Baile"), &model, None)
        .unwrap_err();
    assert_eq!(err, format!("TILES_NOT_LOADED:{}", ng(&graph, "title")));
}
