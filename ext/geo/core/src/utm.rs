//! UTM projection as a load-time **derive function**.
//!
//! An [`ExtDeriveProvider`] that reads a resource's existing `geojson-feature-collection`
//! geometry and computes its UTM zone + easting/northing — tiles that are never
//! stored, only derived. This is the canonical "port an Arches derive function"
//! case: a pure, deterministic transform of existing node values, materialised at
//! load via `functions_x_graphs`.
//!
//! Registered under the provider key `"utm"`. The projection is delegated to
//! [`proj4rs`] (a pure-Rust proj4 port that also builds for WASM) — we only
//! compute the UTM zone and hand proj4rs the corresponding proj-string.

use std::collections::BTreeMap;
#[cfg(feature = "core")]
use std::sync::Arc;

use proj4rs::Proj;
use serde_json::{json, Value};

#[cfg(feature = "core")]
use alizarin_core::{FunctionsRegistry, RegisteredFunction};
use alizarin_extension_api::{
    DeriveConfig, DeriveLookup, ExtDeriveProvider, ResolvedNode, StaticTile,
};

/// The provider key a `functions_x_graphs` compute-tiles entry names to select
/// this derive function.
pub const PROVIDER_KEY: &str = "utm";

/// The result of projecting a lon/lat point into UTM.
#[derive(Debug, Clone, PartialEq)]
pub struct UtmCoord {
    pub zone: u8,
    /// `true` for the northern hemisphere.
    pub north: bool,
    pub easting: f64,
    pub northing: f64,
}

/// The UTM zone (1–60) for a WGS84 longitude.
pub fn utm_zone(lon_deg: f64) -> u8 {
    (((lon_deg + 180.0) / 6.0).floor() as i32).rem_euclid(60) as u8 + 1
}

/// Project a WGS84 lon/lat (degrees) to UTM via proj4rs. Deterministic.
pub fn project_utm(lon_deg: f64, lat_deg: f64) -> Result<UtmCoord, String> {
    let zone = utm_zone(lon_deg);
    let north = lat_deg >= 0.0;

    let from = Proj::from_proj_string("+proj=longlat +datum=WGS84 +no_defs")
        .map_err(|e| format!("longlat proj: {e:?}"))?;
    let utm_str = format!(
        "+proj=utm +zone={zone}{} +datum=WGS84 +units=m +no_defs",
        if north { "" } else { " +south" }
    );
    let to = Proj::from_proj_string(&utm_str).map_err(|e| format!("utm proj: {e:?}"))?;

    // proj4rs takes/returns radians for geographic CRS; longlat input is (lon, lat).
    let mut point = (lon_deg.to_radians(), lat_deg.to_radians(), 0.0_f64);
    proj4rs::transform::transform(&from, &to, &mut point)
        .map_err(|e| format!("transform: {e:?}"))?;

    Ok(UtmCoord {
        zone,
        north,
        easting: point.0,
        northing: point.1,
    })
}

/// A representative lon/lat for a geometry value (a feature collection's first
/// feature, or a bare geometry): the mean of the first polygon's exterior
/// ring, or a point/position. `None` if it holds no geometry.
fn lonlat_from_value(value: &Value) -> Option<(f64, f64)> {
    let obj = value.as_object()?;
    // FeatureCollection → first feature's geometry.
    if obj.get("type").and_then(Value::as_str) == Some("FeatureCollection") {
        let first = obj.get("features")?.as_array()?.first()?;
        let geom = first.as_object()?.get("geometry")?;
        return lonlat_from_geometry(geom);
    }
    // A bare geometry object.
    if obj.contains_key("coordinates") {
        return lonlat_from_geometry(value);
    }
    None
}

fn lonlat_from_geometry(geom: &Value) -> Option<(f64, f64)> {
    let coords = geom.as_object()?.get("coordinates")?;
    let ty = geom.as_object()?.get("type")?.as_str()?;
    match ty {
        "Point" => position(coords),
        "Polygon" => ring_centroid(coords.as_array()?.first()?),
        "MultiPolygon" => ring_centroid(coords.as_array()?.first()?.as_array()?.first()?),
        "LineString" | "MultiPoint" => ring_centroid(coords),
        _ => None,
    }
}

fn position(pos: &Value) -> Option<(f64, f64)> {
    let arr = pos.as_array()?;
    Some((arr.first()?.as_f64()?, arr.get(1)?.as_f64()?))
}

/// Mean of an array of `[lon, lat]` positions.
fn ring_centroid(ring: &Value) -> Option<(f64, f64)> {
    let arr = ring.as_array()?;
    let mut n = 0.0;
    let (mut sx, mut sy) = (0.0, 0.0);
    for p in arr {
        if let Some((x, y)) = position(p) {
            sx += x;
            sy += y;
            n += 1.0;
        }
    }
    if n == 0.0 {
        return None;
    }
    Some((sx / n, sy / n))
}

/// The roles a UTM declaration's `nodes` may name for its outputs; it must
/// name at least one, each in the target nodegroup.
pub const OUTPUT_ROLES: [&str; 4] = ["zone", "hemisphere", "easting", "northing"];

/// The datatypes an output node may have.
pub const OUTPUT_DATATYPES: [&str; 3] = ["number", "non-localized-string", "string"];

/// The UTM derive provider. Projects the declared `geometry` node's value and
/// writes each declared output (`zone`, `hemisphere`, `easting`, `northing`)
/// into its own node of one tile in the target nodegroup, as a raw value the
/// host coerces for that node's datatype. As arches-her's GeoJSON-to-BNG
/// function: its config names its input and output nodes.
///
/// A resource without a geometry value gets no tile (so a stored rerun drops
/// an earlier one); a value with no projectable geometry is an error.
pub struct UtmDeriveProvider;

impl ExtDeriveProvider for UtmDeriveProvider {
    fn derive(
        &self,
        resource_id: &str,
        graph: &dyn DeriveLookup,
        tiles: &[StaticTile],
        config: &DeriveConfig,
    ) -> Result<Vec<StaticTile>, String> {
        let geometry = config.node("geometry")?;
        let outputs: Vec<(&str, &ResolvedNode)> = OUTPUT_ROLES
            .iter()
            .filter_map(|role| config.resolved.get(*role).map(|node| (*role, node)))
            .collect();
        if outputs.is_empty() {
            return Err(format!(
                "utm: declare at least one output node ({})",
                OUTPUT_ROLES.join(", ")
            ));
        }
        if let Some((role, _)) = outputs
            .iter()
            .find(|(_, node)| node.nodegroup_id.as_deref() != Some(config.nodegroup.as_str()))
        {
            return Err(format!(
                "utm: output '{role}' is not in the target nodegroup {}",
                config.nodegroup
            ));
        }

        let Some(source) = tiles
            .iter()
            .find(|t| t.data.get(&geometry.nodeid).is_some_and(|v| !v.is_null()))
        else {
            return Ok(vec![]);
        };
        let (lon, lat) = lonlat_from_value(&source.data[&geometry.nodeid])
            .ok_or("utm: the geometry value has no geometry to project")?;
        let utm = project_utm(lon, lat)?;

        let mut data: BTreeMap<String, Value> = BTreeMap::new();
        for (role, node) in outputs {
            let value = match role {
                "zone" => json!(utm.zone),
                "hemisphere" => json!(if utm.north { "N" } else { "S" }),
                "easting" => json!((utm.easting * 100.0).round() / 100.0),
                _ => json!((utm.northing * 100.0).round() / 100.0),
            };
            // Raw: the host coerces it for the node's datatype, in its own
            // language. These are the datatypes a UTM value can be written as.
            if !OUTPUT_DATATYPES.contains(&node.datatype.as_str()) {
                return Err(format!(
                    "utm: output '{role}' is a {} node; UTM writes {}",
                    node.datatype,
                    OUTPUT_DATATYPES.join(", ")
                ));
            }
            data.insert(node.nodeid.clone(), value);
        }

        // A nested target hangs off the same parent as the geometry, as
        // arches-her's function does.
        let nested = graph
            .nodegroup(&config.nodegroup)
            .is_some_and(|ng| ng.parentnodegroup_id.is_some());
        Ok(vec![StaticTile {
            data,
            nodegroup_id: config.nodegroup.clone(),
            resourceinstance_id: resource_id.to_string(),
            tileid: None,
            parenttile_id: if nested {
                source.parenttile_id.clone()
            } else {
                None
            },
            provisionaledits: None,
            sortorder: None,
        }])
    }
}

/// Register the UTM derive function under [`PROVIDER_KEY`] into a given
/// registry, in-process (`core` feature). The bindings register it with the
/// host through [`crate::c_abi::utm_derive_provider_info`] instead.
#[cfg(feature = "core")]
pub fn register_functions(registry: &mut FunctionsRegistry) {
    registry.register(
        PROVIDER_KEY,
        RegisteredFunction::Derive(Arc::new(alizarin_core::derive_abi::InProcess(
            UtmDeriveProvider,
        ))),
    );
}

/// Run [`UtmDeriveProvider`] over JSON (the resource's tiles, its resolved
/// declaration), with `lookup` answering graph lookups (kinds as core's
/// `derive_abi::LOOKUP_*`): for a binding without core's types, e.g. the WASM
/// one, whose JS package registers it as a JS derive provider.
pub fn derive_json(
    resource_id: &str,
    tiles: Value,
    config: Value,
    lookup: impl Fn(u32, &str) -> Option<Value>,
) -> Result<Value, String> {
    let tiles: Vec<StaticTile> =
        serde_json::from_value(tiles).map_err(|e| format!("utm: tiles: {e}"))?;
    let config: DeriveConfig =
        serde_json::from_value(config).map_err(|e| format!("utm: config: {e}"))?;
    let derived = UtmDeriveProvider.derive(resource_id, &lookup, &tiles, &config)?;
    serde_json::to_value(derived).map_err(|e| format!("utm: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alizarin_core::derive_abi::GraphLookupView;
    use alizarin_core::{
        apply_derive_functions, default_functions_registry, resolve_derive_nodes, StaticGraph,
    };

    /// Definitional reference: a point ON a zone's central meridian AT the equator
    /// projects to the false easting (500 000 m) exactly, with zero northing. Zone
    /// 31N's central meridian is 3°E. This is exact by the UTM definition, so it
    /// pins the projection without relying on any memorised constant.
    #[test]
    fn central_meridian_equator_is_exact() {
        let utm = project_utm(3.0, 0.0).expect("project");
        assert_eq!(utm.zone, 31);
        assert!(utm.north);
        assert!(
            (utm.easting - 500_000.0).abs() < 1e-3,
            "easting {}",
            utm.easting
        );
        assert!(utm.northing.abs() < 1e-3, "northing {}", utm.northing);
    }

    /// A published reference: London (51.5074°N, 0.1278°W) is in zone 30N with
    /// easting ≈ 699 320 m and northing ≈ 5 710 158 m (per standard UTM tables).
    /// Loose tolerance guards against an outright-wrong projection, not rounding.
    #[test]
    fn london_matches_published_utm() {
        let utm = project_utm(-0.1278, 51.5074).expect("project");
        assert_eq!(utm.zone, 30);
        assert!(utm.north);
        assert!(
            (utm.easting - 699_320.0).abs() < 50.0,
            "easting {}",
            utm.easting
        );
        assert!(
            (utm.northing - 5_710_158.0).abs() < 50.0,
            "northing {}",
            utm.northing
        );
    }

    /// Round-trip: project to UTM and back, recovering the original lon/lat.
    #[test]
    fn round_trip_recovers_input() {
        let (lon, lat) = (80.25_f64, 13.08_f64); // Chennai-ish → zone 44N
        let utm = project_utm(lon, lat).expect("project");
        assert_eq!(utm.zone, 44);
        let from = Proj::from_proj_string(&format!(
            "+proj=utm +zone={} +datum=WGS84 +units=m +no_defs",
            utm.zone
        ))
        .unwrap();
        let to = Proj::from_proj_string("+proj=longlat +datum=WGS84 +no_defs").unwrap();
        let mut p = (utm.easting, utm.northing, 0.0_f64);
        proj4rs::transform::transform(&from, &to, &mut p).unwrap();
        assert!(
            (p.0.to_degrees() - lon).abs() < 1e-6,
            "lon {}",
            p.0.to_degrees()
        );
        assert!(
            (p.1.to_degrees() - lat).abs() < 1e-6,
            "lat {}",
            p.1.to_degrees()
        );
    }

    #[test]
    fn zone_boundaries() {
        assert_eq!(utm_zone(-177.0), 1);
        assert_eq!(utm_zone(177.0), 60);
        assert_eq!(utm_zone(-0.5), 30); // London
    }

    /// A graph with a `boundary` geometry, and `zone`/`hemisphere`/`easting`
    /// in a `utm-ng` nodegroup, declaring UTM over them with `outputs`.
    fn utm_graph(outputs: serde_json::Value) -> StaticGraph {
        let node = |id: &str, ng: &str, datatype: &str| {
            serde_json::json!({
                "nodeid": id, "name": id, "alias": id, "datatype": datatype,
                "nodegroup_id": ng, "graph_id": "g",
            })
        };
        let mut nodes = serde_json::json!({ "geometry": "boundary" });
        for (role, alias) in outputs.as_object().unwrap() {
            nodes[role] = alias.clone();
        }
        serde_json::from_value(serde_json::json!({
            "graphid": "g", "name": {"en": "G"},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"},
            "nodes": [
                {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"},
                node("boundary", "boundary-ng", "geojson-feature-collection"),
                node("zone", "utm-ng", "number"),
                node("hemisphere", "utm-ng", "non-localized-string"),
                node("easting", "utm-ng", "number"),
            ],
            "edges": [],
            "nodegroups": [
                {"nodegroupid": "boundary-ng", "cardinality": "1"},
                {"nodegroupid": "utm-ng", "cardinality": "1"},
            ],
            "cards": [], "cards_x_nodes_x_widgets": [],
            "functions_x_graphs": [{
                "id": "fxg-utm", "function_id": "60000000-0000-0000-0000-000000000002", "graph_id": "g",
                "config": {"provider": "utm", "nodegroup": "utm-ng", "member_of": "base", "nodes": nodes}
            }],
        }))
        .expect("graph")
    }

    /// A resolved UTM declaration into `utm-ng`, with `resolved` given directly:
    /// role → (node id, nodegroup, datatype).
    fn resolved_config(nodes: &[(&str, &str, &str, &str)]) -> DeriveConfig {
        let mut resolved = serde_json::Map::new();
        for (role, nodeid, nodegroup, datatype) in nodes {
            resolved.insert(
                role.to_string(),
                serde_json::json!({
                    "nodeid": nodeid, "nodegroup_id": nodegroup,
                    "datatype": datatype, "alias": nodeid,
                }),
            );
        }
        serde_json::from_value(serde_json::json!({
            "provider": "utm", "nodegroup": "utm-ng", "resolved": resolved,
        }))
        .unwrap()
    }

    fn london_tile(parent: Option<&str>) -> StaticTile {
        let mut data: BTreeMap<String, Value> = BTreeMap::new();
        data.insert(
            "boundary".to_string(),
            serde_json::json!({"type": "FeatureCollection", "features": [{
                "type": "Feature", "properties": {},
                "geometry": {"type": "Point", "coordinates": [-0.1278, 51.5074]},
            }]}),
        );
        StaticTile {
            data,
            nodegroup_id: "boundary-ng".to_string(),
            resourceinstance_id: "r1".to_string(),
            tileid: Some("geo-1".to_string()),
            parenttile_id: parent.map(str::to_string),
            provisionaledits: None,
            sortorder: None,
        }
    }

    #[test]
    fn a_nested_target_hangs_off_the_geometry_tiles_parent() {
        let graph: StaticGraph = serde_json::from_value(serde_json::json!({
            "graphid": "g", "name": {"en": "G"},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"},
            "nodes": [{"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"}],
            "edges": [],
            "nodegroups": [
                {"nodegroupid": "site-ng", "cardinality": "n"},
                {"nodegroupid": "utm-ng", "cardinality": "1", "parentnodegroup_id": "site-ng"},
            ],
            "cards": [], "cards_x_nodes_x_widgets": [],
        }))
        .unwrap();
        let config = resolved_config(&[
            (
                "geometry",
                "boundary",
                "boundary-ng",
                "geojson-feature-collection",
            ),
            ("zone", "zone", "utm-ng", "number"),
        ]);
        let tiles = UtmDeriveProvider
            .derive(
                "r1",
                &GraphLookupView(&graph),
                &[london_tile(Some("site-1"))],
                &config,
            )
            .unwrap();
        assert_eq!(tiles[0].parenttile_id.as_deref(), Some("site-1"));
        assert_eq!(tiles[0].data["zone"], 30);

        // A top-level target has no parent, wherever the geometry is.
        let graph = utm_graph(serde_json::json!({ "zone": "zone" }));
        let tiles = UtmDeriveProvider
            .derive(
                "r1",
                &GraphLookupView(&graph),
                &[london_tile(Some("site-1"))],
                &config,
            )
            .unwrap();
        assert_eq!(tiles[0].parenttile_id, None);
    }

    #[test]
    fn an_output_outside_the_target_nodegroup_is_an_error() {
        let graph = utm_graph(serde_json::json!({ "zone": "zone" }));
        let config = resolved_config(&[
            (
                "geometry",
                "boundary",
                "boundary-ng",
                "geojson-feature-collection",
            ),
            ("easting", "elsewhere", "other-ng", "number"),
        ]);
        let err = UtmDeriveProvider
            .derive(
                "r1",
                &GraphLookupView(&graph),
                &[london_tile(None)],
                &config,
            )
            .unwrap_err();
        assert!(
            err.contains("'easting' is not in the target nodegroup"),
            "{err}"
        );
    }

    #[test]
    fn outputs_are_raw_values_and_only_for_datatypes_utm_can_write() {
        let graph = utm_graph(serde_json::json!({ "zone": "zone" }));
        let config = resolved_config(&[
            (
                "geometry",
                "boundary",
                "boundary-ng",
                "geojson-feature-collection",
            ),
            ("hemisphere", "hemisphere", "utm-ng", "string"),
            ("zone", "zone", "utm-ng", "number"),
        ]);
        let tiles = UtmDeriveProvider
            .derive(
                "r1",
                &GraphLookupView(&graph),
                &[london_tile(None)],
                &config,
            )
            .unwrap();
        // Raw: the host makes the string translatable, in its own language.
        assert_eq!(tiles[0].data["hemisphere"], "N");
        assert_eq!(tiles[0].data["zone"], 30);

        let config = resolved_config(&[
            (
                "geometry",
                "boundary",
                "boundary-ng",
                "geojson-feature-collection",
            ),
            ("zone", "zone", "utm-ng", "concept"),
        ]);
        let err = UtmDeriveProvider
            .derive(
                "r1",
                &GraphLookupView(&graph),
                &[london_tile(None)],
                &config,
            )
            .unwrap_err();
        assert!(err.contains("'zone' is a concept node"), "{err}");
    }

    #[test]
    fn a_resource_without_geometry_gets_no_utm_tile_and_outputs_are_required() {
        let mut reg = default_functions_registry();
        register_functions(&mut reg);

        let graph = utm_graph(serde_json::json!({ "zone": "zone" }));
        let mut tiles = Vec::new();
        apply_derive_functions(&mut tiles, &graph, "r1", &|_| true, &reg);
        assert!(tiles.is_empty());

        // No output declared: the provider refuses (on load: no tile).
        let config: DeriveConfig = serde_json::from_value(serde_json::json!({
            "provider": "utm", "nodegroup": "utm-ng", "nodes": {"geometry": "boundary"}
        }))
        .unwrap();
        let config = resolve_derive_nodes(&config, &utm_graph(serde_json::json!({}))).unwrap();
        let err = UtmDeriveProvider
            .derive("r1", &GraphLookupView(&graph), &[], &config)
            .unwrap_err();
        assert!(err.contains("at least one output"), "{err}");
    }

    #[test]
    fn derive_from_geojson_via_registry() {
        let graph = utm_graph(serde_json::json!({
            "zone": "zone", "hemisphere": "hemisphere", "easting": "easting",
        }));

        let boundary = serde_json::json!({
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "properties": {}, "geometry": {
                "type": "Polygon",
                "coordinates": [[[80.24, 13.08], [80.26, 13.08], [80.26, 13.09], [80.24, 13.09], [80.24, 13.08]]]
            }}]
        });
        let mut data: BTreeMap<String, Value> = BTreeMap::new();
        data.insert("boundary".to_string(), boundary);
        let mut tiles = vec![StaticTile {
            data,
            nodegroup_id: "boundary-ng".to_string(),
            resourceinstance_id: "r1".to_string(),
            tileid: None,
            parenttile_id: None,
            provisionaledits: None,
            sortorder: None,
        }];

        let mut reg = default_functions_registry();
        register_functions(&mut reg);
        apply_derive_functions(&mut tiles, &graph, "r1", &|_| true, &reg);

        let utm_tile = tiles
            .iter()
            .find(|t| t.nodegroup_id == "utm-ng")
            .expect("utm tile was derived");
        // Each declared output in its own node; nothing for the undeclared northing.
        assert_eq!(utm_tile.data["zone"], 44);
        assert_eq!(utm_tile.data["hemisphere"], "N");
        assert!(utm_tile.data["easting"].as_f64().unwrap() > 0.0);
        assert_eq!(utm_tile.data.len(), 3);
    }
}
