//! Output parity: core's serialisation of every static type is the one
//! definition of alizarin's output, and `tests/data/output-parity/expected.json`
//! records it for the fixture in that directory. Every binding is held to the
//! same file, byte for byte:
//!
//! - WASM and NAPI: `tests/dual-backend/outputParity.test.ts`
//! - Python: `crates/alizarin-python/tests/test_output_parity.py`
//!
//! Each entry is the compact JSON of `serde_json::to_value(x)`, so keys are in
//! `serde_json`'s sorted order, as a JS object built from it (NAPI, and WASM
//! through `js_json::to_value`) stringifies.
//!
//! A deliberate change to the output: rerun with `ALIZARIN_UPDATE_GOLDEN=1` and
//! review the diff of `expected.json`.

use alizarin_core::graph::{
    compose_resource_layers_from_json, lifecycle_state_from_json, transition_from_json,
};
use alizarin_core::{
    batch_tiles_to_trees_from_json, cards_to_tree_from_json, tiles_to_tree_from_json, GraphWrapper,
    StaticPublication, StaticResource, StaticTile, StaticTranslatableString,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Arches' Standard lifecycle, as in `graph.json`.
const ACTIVE: &str = "f75bb034-36e3-4ab4-8167-f520cf0b4c58";
const RETIRED: &str = "d95d9c0e-0e2c-4450-93a3-d788b91abcc8";
/// Fixes the ids `tree_to_tiles` gives cardinality-n tiles.
const TILE_NAMESPACE: &str = "0d0d0d0d-0000-4000-8000-000000000001";

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/data/output-parity")
}

fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap()
}

fn canonical<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(&serde_json::to_value(value).unwrap()).unwrap()
}

/// Every output the bindings are checked against, by label.
fn outputs() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut put = |label: String, json: String| {
        assert!(out.insert(label.clone(), json).is_none(), "{label} twice");
    };

    let graph_json = read("graph.json");
    let graph = serde_json::from_str::<GraphWrapper>(&graph_json)
        .unwrap()
        .graph
        .remove(0);
    put("graph".into(), canonical(&graph));
    put("graph.name".into(), canonical(&graph.name));
    put("graph.root".into(), canonical(&graph.root));
    for node in &graph.nodes {
        put(
            format!("node:{}", node.alias.as_deref().unwrap()),
            canonical(node),
        );
    }
    for nodegroup in &graph.nodegroups {
        put(
            format!("nodegroup:{}", nodegroup.nodegroupid),
            canonical(nodegroup),
        );
    }
    for edge in &graph.edges {
        put(format!("edge:{}", edge.edgeid), canonical(edge));
    }
    for card in graph.cards.as_deref().unwrap_or_default() {
        put(format!("card:{}", card.cardid), canonical(card));
        for constraint in &card.constraints {
            put(
                format!("constraint:{}", constraint.constraintid),
                canonical(constraint),
            );
        }
    }
    for cxnxw in graph.cards_x_nodes_x_widgets.as_deref().unwrap_or_default() {
        put(format!("cxnxw:{}", cxnxw.id), canonical(cxnxw));
    }
    for fxg in graph.functions_x_graphs.as_deref().unwrap_or_default() {
        put(format!("fxg:{}", fxg.id), canonical(fxg));
    }
    let publication: StaticPublication =
        serde_json::from_value(graph.publication.clone().unwrap()).unwrap();
    put("publication".into(), canonical(&publication));

    let resource: StaticResource = serde_json::from_str(&read("resource.json")).unwrap();
    put("resource".into(), canonical(&resource));
    put("resource.summary".into(), canonical(&resource.to_summary()));
    put(
        "resource.summary.metadata".into(),
        canonical(&resource.to_summary().to_metadata()),
    );
    put(
        "resource.from_summary".into(),
        canonical(&StaticResource::from_summary(&resource.to_summary())),
    );
    for tile in resource.tiles.as_deref().unwrap_or_default() {
        put(
            format!("tile:{}", tile.tileid.as_deref().unwrap()),
            canonical(tile),
        );
    }

    // Tiles to trees: by node, by card, and as a batch.
    let resource_json = read("resource.json");
    put(
        "tree".into(),
        canonical(&tiles_to_tree_from_json(&resource_json, &graph).unwrap()),
    );
    put(
        "tree:cards".into(),
        canonical(&cards_to_tree_from_json(&resource_json, &graph).unwrap()),
    );
    put(
        "trees:batch".into(),
        canonical(
            &batch_tiles_to_trees_from_json(&format!("[{resource_json}]"), Some(&graph), true)
                .unwrap(),
        ),
    );

    // Trees back to tiles, with a tile namespace so the ids are fixed, and
    // resources merged: the fixture with its layer copy (the same id).
    let tree =
        serde_json::to_string(&tiles_to_tree_from_json(&resource_json, &graph).unwrap()).unwrap();
    let single = alizarin_core::TreeToTilesOptions {
        strict: true,
        tile_namespace: Some(TILE_NAMESPACE),
        ..Default::default()
    };
    let tiles = alizarin_core::tree_to_tiles_from_json(&tree, &graph, &single, None, None);
    put("tiles:tree".into(), canonical(&tiles.unwrap()));
    let batch = alizarin_core::BatchOptions {
        graph_id: &graph.graphid,
        from_camel: false,
        strict: true,
        random_ids: false,
        resolve_markers: false,
        id_keys: None,
        scopes: None,
        tile_namespace: Some(TILE_NAMESPACE),
    };
    let trees = format!("[{tree}]");
    let tiles = alizarin_core::batch_trees_to_tiles_from_json(&trees, &graph, &batch, None, None);
    put("tiles:batch".into(), canonical(&tiles.unwrap()));
    let layer_json = read("resource-layer.json");
    // Batch merging unifies single-valued tiles by the registered graph, as
    // each binding has it registered.
    alizarin_core::registry::register_graph(&graph.graphid, std::sync::Arc::new(graph.clone()));
    put(
        "merged".into(),
        canonical(
            &alizarin_core::merge_resources_from_json(&format!("[{resource_json},{layer_json}]"))
                .unwrap(),
        ),
    );
    // The registered graph exported as prebuild files.
    let ids = [graph.graphid.clone()];
    let export = alizarin_core::build_prebuild_export(Some(&ids), None, "http://localhost/");
    put("export:prebuild".into(), canonical(&export.unwrap()));

    let batches = [resource_json.clone(), layer_json];
    let merged = alizarin_core::batch_merge_resources_from_json(&batches, false, false);
    put("merged:batch".into(), canonical(&merged.unwrap()));

    // A business data file loaded into a registry: one ref per resource, and
    // whether its `__scopes` make it public.
    let refs = alizarin_core::StaticResourceRegistry::new()
        .load_from_business_data_bytes(read("business-data.json").as_bytes(), true, true)
        .unwrap();
    put("business_data:refs".into(), canonical(&refs));

    // populateCaches over the fixture resource (it references nothing), and a
    // path resolved through the fixture graph.
    let mut registry = alizarin_core::StaticResourceRegistry::new();
    registry.merge_from_resources(&[resource.clone()], true, true);
    put(
        "populate_caches".into(),
        canonical(
            &registry
                .populate_caches_from_json(
                    &format!("[{}]", read("resource.json")),
                    &graph,
                    true,
                    false,
                    false,
                )
                .unwrap(),
        ),
    );
    put(
        "resolve_path:part".into(),
        canonical(&alizarin_core::ResolvedPath::from(
            &alizarin_core::resolve_path_in_graph(&graph, "part").unwrap(),
        )),
    );

    // Resources built from input, as the bindings' constructors build them.
    put(
        "resource:constructed".into(),
        canonical(
            &alizarin_core::construct_static_resource(
                serde_json::from_str(&read("resource.json")).unwrap(),
            )
            .unwrap(),
        ),
    );
    put(
        "business_data:resources".into(),
        canonical(
            &alizarin_core::resources_from_business_data_json(&read("business-data.json")).unwrap(),
        ),
    );

    // A tile with every optional field empty, as `createStaticTile` makes one.
    let mut empty = StaticTile::new_empty("0a0a0a0a-0000-4000-8000-000000000011".into());
    empty.resourceinstance_id = "0b0b0b0b-0000-4000-8000-000000000001".into();
    put("tile:empty".into(), canonical(&empty));

    // composeResourceLayers, BRACE's route: the layer copy over the base.
    let layer: serde_json::Value = serde_json::from_str(&read("resource-layer.json")).unwrap();
    let base: serde_json::Value = serde_json::from_str(&read("resource.json")).unwrap();
    let base_graph = serde_json::to_string(&graph).unwrap();
    let composed = compose_resource_layers_from_json(
        &serde_json::json!([layer, base]).to_string(),
        &base_graph,
        "[]",
        false,
    )
    .unwrap();
    put("composed".into(), canonical(&composed));

    // Lifecycles: the base is Active and the layer copy Draft, so the
    // composed resource above is a Conflict.
    let lifecycle = serde_json::to_string(&graph.resource_instance_lifecycle).unwrap();
    let composed_json = serde_json::to_string(&composed.resource).unwrap();
    put(
        "lifecycle:state".into(),
        canonical(&lifecycle_state_from_json(&resource_json, &lifecycle).unwrap()),
    );
    put(
        "lifecycle:state:composed".into(),
        canonical(&lifecycle_state_from_json(&composed_json, &lifecycle).unwrap()),
    );
    put(
        "lifecycle:state:none".into(),
        canonical(&lifecycle_state_from_json(&resource_json, "null").unwrap()),
    );
    put(
        "lifecycle:transition".into(),
        canonical(&transition_from_json(&resource_json, &lifecycle, RETIRED, true).unwrap()),
    );
    put(
        "lifecycle:transition:resolve".into(),
        canonical(&transition_from_json(&composed_json, &lifecycle, ACTIVE, false).unwrap()),
    );

    // A translatable string with several languages: the key order must not
    // depend on the HashMap it is held in.
    let mut names = std::collections::HashMap::new();
    for lang in ["ga", "en", "fr", "de"] {
        names.insert(lang.to_string(), format!("name-{lang}"));
    }
    put(
        "translatable:many".into(),
        canonical(&StaticTranslatableString::from_translations(names, None)),
    );
    out
}

#[test]
fn the_golden_file_is_cores_output() {
    let path = dir().join("expected.json");
    let mut actual = serde_json::to_string_pretty(&outputs()).unwrap();
    actual.push('\n');
    if std::env::var_os("ALIZARIN_UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .expect("tests/data/output-parity/expected.json: run with ALIZARIN_UPDATE_GOLDEN=1");
    assert_eq!(
        expected, actual,
        "core's output changed: if deliberate, rerun with ALIZARIN_UPDATE_GOLDEN=1 and review"
    );
}

/// `resource.json` with its descriptors in Arches' per-language form: the
/// fixture's own under "en", and a decoy under "de", which sorts first. The
/// bindings read it through the same routes as `resource.json`.
fn resource_descriptors_by_language() -> String {
    let mut resource: serde_json::Value = serde_json::from_str(&read("resource.json")).unwrap();
    let flat = resource["resourceinstance"]["descriptors"].take();
    resource["resourceinstance"]["descriptors"] =
        serde_json::json!({"de": {"name": "Parität eins"}, "en": flat});
    resource.to_string()
}

/// Per-language descriptors are read as "en", so the resource serialises
/// exactly as `resource.json` does (the golden's "resource").
#[test]
fn per_language_descriptors_give_the_flat_resource() {
    let by_language: StaticResource =
        serde_json::from_str(&resource_descriptors_by_language()).unwrap();
    let flat: StaticResource = serde_json::from_str(&read("resource.json")).unwrap();
    assert_eq!(canonical(&by_language), canonical(&flat));
}

/// A struct serialised straight to a string (as `exportTilesJson` and the
/// exporters do) keeps its maps' iteration order, so a `HashMap` field would
/// make the output differ from run to run. Serialising the same values from
/// fresh maps many times must always give the same bytes.
#[test]
fn direct_serialisation_is_byte_stable() {
    let graph_json = read("graph.json");
    let resource_json = read("resource.json");
    // Parsed afresh each time, so each run fills new HashMaps.
    let direct = || {
        let graphs = serde_json::from_str::<GraphWrapper>(&graph_json).unwrap();
        let resource = serde_json::from_str::<StaticResource>(&resource_json).unwrap();
        (
            serde_json::to_string(&graphs.graph).unwrap(),
            serde_json::to_string(&resource).unwrap(),
        )
    };
    let first = direct();
    for _ in 0..32 {
        assert_eq!(first, direct(), "a HashMap field serialised in hash order");
    }
}
