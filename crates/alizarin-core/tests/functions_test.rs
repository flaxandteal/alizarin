//! Graph-attached function registry — thorough suite for BOTH kinds
//! (Derive + Descriptor). An integration test (compiles against the public API
//! only), so it runs independently of the crate's other unit-test modules.

use std::sync::Arc;

use alizarin_core::{
    apply_derive_functions, apply_stored_derive_functions, default_functions_registry,
    derived_tile_id, DeriveConfig, DeriveProvider, DescriptorProvider, FunctionsRegistry,
    GraphLookup, RegisteredFunction, StaticGraph, StaticTile,
};

// ---- helpers ---------------------------------------------------------------

fn tile(nodegroup: &str, resource: &str, data: serde_json::Value) -> StaticTile {
    let map = data
        .as_object()
        .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    StaticTile {
        data: map,
        nodegroup_id: nodegroup.to_string(),
        resourceinstance_id: resource.to_string(),
        tileid: None,
        parenttile_id: None,
        provisionaledits: None,
        sortorder: None,
    }
}

fn graph_with(fxgs: Vec<serde_json::Value>) -> StaticGraph {
    serde_json::from_value(serde_json::json!({
        "graphid": "g",
        "name": {"en": "G"},
        "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"},
        "nodes": [{"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"}],
        "edges": [], "nodegroups": [], "cards": [], "cards_x_nodes_x_widgets": [],
        "functions_x_graphs": fxgs,
    }))
    .expect("graph")
}

const COMPUTE_ID: &str = "60000000-0000-0000-0000-000000000002";

fn derive_fxg(provider: &str, nodegroup: &str, member_of: &str) -> serde_json::Value {
    serde_json::json!({
        "id": format!("fxg-{provider}"),
        "function_id": COMPUTE_ID,
        "graph_id": "g",
        "config": { "provider": provider, "nodegroup": nodegroup, "member_of": member_of, "cache": true }
    })
}

// ---- providers used across tests -------------------------------------------

/// A transform: latlng tile → utm tile. The canonical "port an Arches derive
/// function" case; proves the Derive provider reads EXISTING node values.
/// (The maths is an illustrative stand-in, not real UTM.)
struct LatLngToUtm;
impl DeriveProvider for LatLngToUtm {
    fn derive(
        &self,
        resource_id: &str,
        _graph: &dyn GraphLookup,
        tiles: &[StaticTile],
        config: &DeriveConfig,
    ) -> Result<Vec<StaticTile>, String> {
        let ll = tiles
            .iter()
            .find(|t| t.nodegroup_id == "latlng")
            .and_then(|t| t.data.get("latlng"))
            .ok_or("no latlng node value to convert")?;
        let lat = ll["lat"].as_f64().ok_or("bad lat")?;
        let lng = ll["lng"].as_f64().ok_or("bad lng")?;
        let easting = (lng + 180.0) * 1000.0;
        let northing = (lat + 90.0) * 1000.0;
        Ok(vec![tile(
            &config.nodegroup,
            resource_id,
            serde_json::json!({ "utm": { "easting": easting, "northing": northing } }),
        )])
    }
}

/// A generator that ignores existing tiles and emits one tile keyed by the
/// resource id (Gramadán-shaped: forms from the headword — here stubbed).
struct StubGen;
impl DeriveProvider for StubGen {
    fn derive(
        &self,
        resource_id: &str,
        _graph: &dyn GraphLookup,
        _tiles: &[StaticTile],
        config: &DeriveConfig,
    ) -> Result<Vec<StaticTile>, String> {
        Ok(vec![tile(
            &config.nodegroup,
            resource_id,
            serde_json::json!({}),
        )])
    }
}

/// A descriptor: build a display string from a tile value.
struct HeadwordDescriptor;
impl DescriptorProvider for HeadwordDescriptor {
    fn describe(
        &self,
        _resource_id: &str,
        tiles: &[StaticTile],
        _config: &serde_json::Value,
    ) -> Result<String, String> {
        tiles
            .iter()
            .find_map(|t| t.data.get("headword").and_then(|v| v.as_str()))
            .map(|s| s.to_string())
            .ok_or_else(|| "no headword".to_string())
    }
}

// ---- registry --------------------------------------------------------------

#[test]
fn default_registry_holds_only_the_builtin_fold_aggregate() {
    let reg = default_functions_registry();
    assert_eq!(reg.len(), 1);
    assert!(reg
        .aggregate(alizarin_core::FOLD_AGGREGATE_PROVIDER_ID)
        .is_some());
}

#[test]
fn registry_holds_both_kinds_and_typed_lookups_respect_kind() {
    let mut reg = FunctionsRegistry::new();
    reg.register("utm", RegisteredFunction::Derive(Arc::new(LatLngToUtm)));
    reg.register(
        "headword",
        RegisteredFunction::Descriptor(Arc::new(HeadwordDescriptor)),
    );
    assert_eq!(reg.len(), 2);

    assert!(reg.derive("utm").is_some());
    assert!(reg.descriptor("headword").is_some());
    // Wrong-kind lookups return None, not the other kind.
    assert!(reg.descriptor("utm").is_none());
    assert!(reg.derive("headword").is_none());
    // Unknown UUIDs return None.
    assert!(reg.derive("nope").is_none() && reg.get("nope").is_none());
    assert_eq!(reg.get("utm").map(|f| f.kind()), Some("derive"));
    assert_eq!(reg.get("headword").map(|f| f.kind()), Some("descriptor"));
}

#[test]
fn unregister_removes() {
    let mut reg = FunctionsRegistry::new();
    reg.register("utm", RegisteredFunction::Derive(Arc::new(LatLngToUtm)));
    assert!(reg.unregister("utm").is_some());
    assert!(reg.is_empty());
}

// ---- derive: the latlng → UTM port -----------------------------------------

#[test]
fn derive_reads_existing_tiles_and_produces_the_derived_nodegroup() {
    let mut reg = FunctionsRegistry::new();
    reg.register("utm", RegisteredFunction::Derive(Arc::new(LatLngToUtm)));
    let graph = graph_with(vec![derive_fxg("utm", "utm", "geo")]);

    // The resource already has its latlng node value (attested).
    let mut tiles = vec![tile(
        "latlng",
        "r1",
        serde_json::json!({ "latlng": { "lat": 10.0, "lng": 20.0 } }),
    )];
    apply_derive_functions(&mut tiles, &graph, "r1", &|_| true, &reg);

    let utm = tiles
        .iter()
        .find(|t| t.nodegroup_id == "utm")
        .expect("utm derived");
    assert_eq!(utm.resourceinstance_id, "r1");
    assert_eq!(utm.data["utm"]["easting"].as_f64(), Some(200_000.0));
    assert_eq!(utm.data["utm"]["northing"].as_f64(), Some(100_000.0));
}

#[test]
fn derive_errors_are_swallowed_not_panicked() {
    let mut reg = FunctionsRegistry::new();
    reg.register("utm", RegisteredFunction::Derive(Arc::new(LatLngToUtm)));
    let graph = graph_with(vec![derive_fxg("utm", "utm", "geo")]);
    // No latlng tile → the provider Errs; apply must swallow it.
    let mut tiles = Vec::new();
    apply_derive_functions(&mut tiles, &graph, "r1", &|_| true, &reg);
    assert!(tiles.is_empty());
}

// ---- derive: gating + precedence + missing provider ------------------------

#[test]
fn derive_skips_non_member() {
    let mut reg = FunctionsRegistry::new();
    reg.register("gen", RegisteredFunction::Derive(Arc::new(StubGen)));
    let graph = graph_with(vec![derive_fxg("gen", "forms", "bunamo")]);
    let mut tiles = Vec::new();
    apply_derive_functions(&mut tiles, &graph, "r1", &|_| false, &reg);
    assert!(tiles.is_empty());
}

#[test]
fn derive_generates_for_member() {
    let mut reg = FunctionsRegistry::new();
    reg.register("gen", RegisteredFunction::Derive(Arc::new(StubGen)));
    let graph = graph_with(vec![derive_fxg("gen", "forms", "bunamo")]);
    let mut tiles = Vec::new();
    apply_derive_functions(&mut tiles, &graph, "r1", &|id| id == "bunamo", &reg);
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].nodegroup_id, "forms");
}

#[test]
fn derive_does_not_overwrite_attested_nodegroup() {
    let mut reg = FunctionsRegistry::new();
    reg.register("gen", RegisteredFunction::Derive(Arc::new(StubGen)));
    let graph = graph_with(vec![derive_fxg("gen", "forms", "bunamo")]);
    // An attested `forms` tile is already present.
    let mut tiles = vec![tile("forms", "r1", serde_json::json!({ "attested": true }))];
    apply_derive_functions(&mut tiles, &graph, "r1", &|_| true, &reg);
    assert_eq!(
        tiles.len(),
        1,
        "attested forms must win — no computed tile added"
    );
    assert_eq!(tiles[0].data["attested"].as_bool(), Some(true));
}

#[test]
fn derive_skips_when_provider_not_registered() {
    let reg = FunctionsRegistry::new(); // empty — provider crate absent
    let graph = graph_with(vec![derive_fxg("gramadan", "forms", "gramadan")]);
    let mut tiles = Vec::new();
    apply_derive_functions(&mut tiles, &graph, "r1", &|_| true, &reg);
    assert!(tiles.is_empty(), "unknown provider → skip, no panic");
}

#[test]
fn derive_functions_chain_second_sees_first() {
    // A → nodegroup "a"; B reads "a" and → nodegroup "b". fxg order [A, B].
    struct A;
    impl DeriveProvider for A {
        fn derive(
            &self,
            r: &str,
            _g: &dyn GraphLookup,
            _t: &[StaticTile],
            c: &DeriveConfig,
        ) -> Result<Vec<StaticTile>, String> {
            Ok(vec![tile(&c.nodegroup, r, serde_json::json!({ "v": 1 }))])
        }
    }
    struct B;
    impl DeriveProvider for B {
        fn derive(
            &self,
            r: &str,
            _g: &dyn GraphLookup,
            t: &[StaticTile],
            c: &DeriveConfig,
        ) -> Result<Vec<StaticTile>, String> {
            let a = t
                .iter()
                .find(|x| x.nodegroup_id == "a")
                .ok_or("a not yet present")?;
            let v = a.data["v"].as_i64().unwrap_or(0) + 1;
            Ok(vec![tile(&c.nodegroup, r, serde_json::json!({ "v": v }))])
        }
    }
    let mut reg = FunctionsRegistry::new();
    reg.register("pa", RegisteredFunction::Derive(Arc::new(A)));
    reg.register("pb", RegisteredFunction::Derive(Arc::new(B)));
    let graph = graph_with(vec![derive_fxg("pa", "a", "m"), derive_fxg("pb", "b", "m")]);
    let mut tiles = Vec::new();
    apply_derive_functions(&mut tiles, &graph, "r1", &|_| true, &reg);
    let b = tiles
        .iter()
        .find(|t| t.nodegroup_id == "b")
        .expect("b derived from a");
    assert_eq!(
        b.data["v"].as_i64(),
        Some(2),
        "B saw A's output in the same pass"
    );
}

// ---- derive: stored mode ---------------------------------------------------

/// A graph with nodegroups `latlng`, `utm` (cardinality `utm_card`) and `b`
/// (cardinality 1), declaring `fxgs`.
fn stored_graph(utm_card: &str, fxgs: Vec<serde_json::Value>) -> StaticGraph {
    serde_json::from_value(serde_json::json!({
        "graphid": "g",
        "name": {"en": "G"},
        "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"},
        "nodes": [{"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"}],
        "edges": [],
        "nodegroups": [
            {"nodegroupid": "latlng", "cardinality": "1"},
            {"nodegroupid": "utm", "cardinality": utm_card},
            {"nodegroupid": "b", "cardinality": "1"},
        ],
        "cards": [], "cards_x_nodes_x_widgets": [],
        "functions_x_graphs": fxgs,
    }))
    .expect("graph")
}

fn stored_fxg(provider: &str, nodegroup: &str, sources: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "id": format!("fxg-{provider}"),
        "function_id": COMPUTE_ID,
        "graph_id": "g",
        "config": { "provider": provider, "nodegroup": nodegroup, "mode": "stored", "sources": sources }
    })
}

fn utm_registry() -> FunctionsRegistry {
    let mut reg = FunctionsRegistry::new();
    reg.register("utm", RegisteredFunction::Derive(Arc::new(LatLngToUtm)));
    reg
}

fn latlng(lat: f64, lng: f64) -> StaticTile {
    let mut t = tile(
        "latlng",
        "r1",
        serde_json::json!({ "latlng": { "lat": lat, "lng": lng } }),
    );
    t.tileid = Some("ll-1".to_string());
    t
}

fn with_id(mut t: StaticTile, id: &str) -> StaticTile {
    t.tileid = Some(id.to_string());
    t
}

#[test]
fn stored_declarations_do_not_run_on_load() {
    let graph = stored_graph("1", vec![stored_fxg("utm", "utm", &["latlng"])]);
    let mut tiles = vec![latlng(10.0, 20.0)];
    apply_derive_functions(&mut tiles, &graph, "r1", &|_| true, &utm_registry());
    assert_eq!(
        tiles.len(),
        1,
        "a stored declaration must not derive on load"
    );
}

#[test]
fn stored_writes_a_predictable_id_and_replaces_it_on_rerun() {
    let graph = stored_graph("1", vec![stored_fxg("utm", "utm", &["latlng"])]);
    let reg = utm_registry();
    let expected = derived_tile_id("r1", "utm", None, "utm", 0);

    let mut tiles = vec![latlng(10.0, 20.0)];
    let changes =
        apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &reg).expect("derive");
    assert_eq!(changes.upserted.len(), 1);
    assert_eq!(
        changes.upserted[0].tileid.as_deref(),
        Some(expected.as_str())
    );
    assert!(changes.removed.is_empty());

    // The source changes: the same tile is rewritten, not duplicated.
    tiles[0] = latlng(30.0, 40.0);
    let changed = ["latlng".to_string()];
    let changes = apply_stored_derive_functions(&mut tiles, &graph, "r1", Some(&changed), &reg)
        .expect("rerun");
    assert_eq!(changes.upserted.len(), 1);
    assert!(changes.removed.is_empty(), "replaced in place, same id");
    let utm: Vec<_> = tiles.iter().filter(|t| t.nodegroup_id == "utm").collect();
    assert_eq!(utm.len(), 1);
    assert_eq!(utm[0].tileid.as_deref(), Some(expected.as_str()));
    assert_eq!(utm[0].data["utm"]["easting"].as_f64(), Some(220_000.0));

    // Nothing changed: rerunning reports no changes.
    let changes = apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &reg)
        .expect("rerun unchanged");
    assert!(changes.is_empty());
}

#[test]
fn stored_runs_only_when_a_source_changed() {
    let graph = stored_graph("1", vec![stored_fxg("utm", "utm", &["latlng"])]);
    let mut tiles = vec![latlng(10.0, 20.0)];
    let changed = ["b".to_string()];
    let changes =
        apply_stored_derive_functions(&mut tiles, &graph, "r1", Some(&changed), &utm_registry())
            .expect("derive");
    assert!(changes.is_empty());
    assert_eq!(tiles.len(), 1);
}

#[test]
fn stored_overwrites_another_tile_in_a_cardinality_1_target_by_default() {
    let graph = stored_graph("1", vec![stored_fxg("utm", "utm", &["latlng"])]);
    let mut tiles = vec![
        latlng(10.0, 20.0),
        with_id(
            tile("utm", "r1", serde_json::json!({ "manual": true })),
            "mine",
        ),
    ];
    let changes = apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &utm_registry())
        .expect("derive");
    assert_eq!(changes.removed, vec!["mine".to_string()]);
    let utm: Vec<_> = tiles.iter().filter(|t| t.nodegroup_id == "utm").collect();
    assert_eq!(utm.len(), 1);
    assert!(utm[0].data.contains_key("utm"));
}

#[test]
fn stored_gives_way_in_a_cardinality_1_target_without_overwrite() {
    let mut fxg = stored_fxg("utm", "utm", &["latlng"]);
    fxg["config"]["overwrite"] = serde_json::json!(false);
    let graph = stored_graph("1", vec![fxg]);
    let reg = utm_registry();

    // Its own earlier tile, written before someone added theirs, is dropped.
    let mut tiles = vec![latlng(10.0, 20.0)];
    apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &reg).expect("derive");
    let own = derived_tile_id("r1", "utm", None, "utm", 0);
    tiles.retain(|t| t.nodegroup_id != "utm");
    tiles.push(with_id(
        tile("utm", "r1", serde_json::json!({ "manual": true })),
        "mine",
    ));
    tiles.push(with_id(
        tile("utm", "r1", serde_json::json!({ "stale": true })),
        &own,
    ));

    let changes =
        apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &reg).expect("derive");
    assert_eq!(changes.removed, vec![own]);
    assert!(changes.upserted.is_empty());
    let utm: Vec<_> = tiles.iter().filter(|t| t.nodegroup_id == "utm").collect();
    assert_eq!(utm.len(), 1);
    assert_eq!(utm[0].tileid.as_deref(), Some("mine"));
}

#[test]
fn stored_leaves_other_tiles_in_a_cardinality_n_target_and_drops_its_surplus() {
    // Emits one utm tile per latlng tile.
    struct PerLatLng;
    impl DeriveProvider for PerLatLng {
        fn derive(
            &self,
            r: &str,
            _g: &dyn GraphLookup,
            t: &[StaticTile],
            c: &DeriveConfig,
        ) -> Result<Vec<StaticTile>, String> {
            Ok(t.iter()
                .filter(|x| x.nodegroup_id == "latlng")
                .map(|x| tile(&c.nodegroup, r, serde_json::json!({ "from": x.tileid })))
                .collect())
        }
    }
    let mut reg = FunctionsRegistry::new();
    reg.register("per", RegisteredFunction::Derive(Arc::new(PerLatLng)));
    let graph = stored_graph("n", vec![stored_fxg("per", "utm", &["latlng"])]);

    let mut tiles = vec![
        with_id(latlng(1.0, 2.0), "ll-1"),
        with_id(latlng(3.0, 4.0), "ll-2"),
        with_id(
            tile("utm", "r1", serde_json::json!({ "manual": true })),
            "mine",
        ),
    ];
    let changes =
        apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &reg).expect("derive");
    assert_eq!(changes.upserted.len(), 2);
    assert_eq!(tiles.iter().filter(|t| t.nodegroup_id == "utm").count(), 3);

    // One source tile goes: the function's second tile goes with it, the
    // manual tile stays.
    tiles.retain(|t| t.tileid.as_deref() != Some("ll-2"));
    let changes =
        apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &reg).expect("rerun");
    assert_eq!(
        changes.removed,
        vec![derived_tile_id("r1", "utm", None, "per", 1)]
    );
    let utm: Vec<_> = tiles.iter().filter(|t| t.nodegroup_id == "utm").collect();
    assert_eq!(utm.len(), 2);
    assert!(utm.iter().any(|t| t.tileid.as_deref() == Some("mine")));
}

#[test]
fn stored_functions_chain_on_their_targets() {
    // utm from latlng, then `b` from utm: a latlng change reruns both.
    struct FromUtm;
    impl DeriveProvider for FromUtm {
        fn derive(
            &self,
            r: &str,
            _g: &dyn GraphLookup,
            t: &[StaticTile],
            c: &DeriveConfig,
        ) -> Result<Vec<StaticTile>, String> {
            let utm = t
                .iter()
                .find(|x| x.nodegroup_id == "utm")
                .ok_or("no utm yet")?;
            Ok(vec![tile(
                &c.nodegroup,
                r,
                serde_json::json!({ "e": utm.data["utm"]["easting"] }),
            )])
        }
    }
    let mut reg = utm_registry();
    reg.register("from-utm", RegisteredFunction::Derive(Arc::new(FromUtm)));
    let graph = stored_graph(
        "1",
        vec![
            stored_fxg("utm", "utm", &["latlng"]),
            stored_fxg("from-utm", "b", &["utm"]),
        ],
    );
    let mut tiles = vec![latlng(10.0, 20.0)];
    let changed = ["latlng".to_string()];
    let changes = apply_stored_derive_functions(&mut tiles, &graph, "r1", Some(&changed), &reg)
        .expect("derive");
    assert_eq!(changes.upserted.len(), 2);
    let b = tiles.iter().find(|t| t.nodegroup_id == "b").expect("b");
    assert_eq!(b.data["e"].as_f64(), Some(200_000.0));
}

#[test]
fn stored_errors_leave_tiles_untouched() {
    let reg = utm_registry();
    let original = vec![latlng(10.0, 20.0)];

    // No sources declared.
    let graph = stored_graph("1", vec![stored_fxg("utm", "utm", &[])]);
    let mut tiles = original.clone();
    assert!(apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &reg).is_err());

    // Provider not registered: an error, not a silent skip, for stored mode.
    let graph = stored_graph("1", vec![stored_fxg("nope", "utm", &["latlng"])]);
    assert!(apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &reg).is_err());

    // A later function fails: the earlier one's tile is not kept.
    let graph = stored_graph(
        "1",
        vec![
            stored_fxg("utm", "utm", &["latlng"]),
            stored_fxg("nope", "b", &["utm"]),
        ],
    );
    assert!(apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &reg).is_err());

    // A target nodegroup the graph doesn't have.
    let graph = stored_graph("1", vec![stored_fxg("utm", "missing", &["latlng"])]);
    assert!(apply_stored_derive_functions(&mut tiles, &graph, "r1", None, &reg).is_err());

    assert_eq!(tiles.len(), original.len());
    assert!(tiles.iter().all(|t| t.nodegroup_id == "latlng"));
}

// ---- descriptor kind -------------------------------------------------------

#[test]
fn descriptor_provider_builds_a_string_from_tiles() {
    let mut reg = FunctionsRegistry::new();
    reg.register(
        "hw",
        RegisteredFunction::Descriptor(Arc::new(HeadwordDescriptor)),
    );
    let desc = reg.descriptor("hw").expect("descriptor registered");
    let tiles = vec![tile(
        "entry",
        "r1",
        serde_json::json!({ "headword": "fear" }),
    )];
    assert_eq!(
        desc.describe("r1", &tiles, &serde_json::Value::Null)
            .unwrap(),
        "fear"
    );
    // Missing input → Err, surfaced to the caller.
    assert!(desc.describe("r1", &[], &serde_json::Value::Null).is_err());
}
