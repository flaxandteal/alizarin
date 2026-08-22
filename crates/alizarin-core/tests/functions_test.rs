//! Graph-attached function registry — thorough suite for BOTH kinds
//! (Derive + Descriptor). An integration test (compiles against the public API
//! only), so it runs independently of the crate's other unit-test modules.

use std::sync::Arc;

use alizarin_core::{
    apply_derive_functions, default_functions_registry, ComputeTilesConfig, DeriveProvider,
    DescriptorProvider, FunctionsRegistry, GraphLookup, RegisteredFunction, StaticGraph,
    StaticTile,
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
        config: &ComputeTilesConfig,
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
        config: &ComputeTilesConfig,
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
fn default_registry_is_empty() {
    assert!(default_functions_registry().is_empty());
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
            c: &ComputeTilesConfig,
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
            c: &ComputeTilesConfig,
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
