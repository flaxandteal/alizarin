//! UTM projection as a load-time **derive function**.
//!
//! A [`DeriveProvider`] that reads a resource's existing `geojson-feature-collection`
//! geometry and computes its UTM zone + easting/northing — tiles that are never
//! stored, only derived. This is the canonical "port an Arches derive function"
//! case: a pure, deterministic transform of existing node values, materialised at
//! load via `functions_x_graphs`.
//!
//! Registered under the provider key `"utm"`. The projection is delegated to
//! [`proj4rs`] (a pure-Rust proj4 port that also builds for WASM) — we only
//! compute the UTM zone and hand proj4rs the corresponding proj-string.

use std::collections::HashMap;
use std::sync::Arc;

use proj4rs::Proj;
use serde_json::{json, Value};

use alizarin_core::{
    ComputeTilesConfig, DeriveProvider, FunctionsRegistry, GraphLookup, RegisteredFunction,
    StaticTile,
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

/// Find the first `geojson-feature-collection`-shaped value in the resource's
/// tiles and return a representative lon/lat (the mean of the first polygon's
/// exterior ring, or a bare point/position). Returns `None` if no geometry is
/// present.
fn representative_lonlat(tiles: &[StaticTile]) -> Option<(f64, f64)> {
    for tile in tiles {
        for value in tile.data.values() {
            if let Some(ll) = lonlat_from_value(value) {
                return Some(ll);
            }
        }
    }
    None
}

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

/// The UTM derive provider. Reads the resource's geometry, projects it, and emits
/// one tile on the configured output nodegroup.
pub struct UtmDeriveProvider;

impl DeriveProvider for UtmDeriveProvider {
    fn derive(
        &self,
        resource_id: &str,
        _graph: &dyn GraphLookup,
        tiles: &[StaticTile],
        config: &ComputeTilesConfig,
    ) -> Result<Vec<StaticTile>, String> {
        let (lon, lat) = representative_lonlat(tiles).ok_or("no geojson geometry to project")?;
        let utm = project_utm(lon, lat)?;
        let value = json!({
            "zone": utm.zone,
            "hemisphere": if utm.north { "N" } else { "S" },
            "easting": (utm.easting * 100.0).round() / 100.0,
            "northing": (utm.northing * 100.0).round() / 100.0,
        });
        let mut data: HashMap<String, Value> = HashMap::new();
        data.insert(config.nodegroup.clone(), value);
        Ok(vec![StaticTile {
            data,
            nodegroup_id: config.nodegroup.clone(),
            resourceinstance_id: resource_id.to_string(),
            tileid: None,
            parenttile_id: None,
            provisionaledits: None,
            sortorder: None,
        }])
    }
}

/// Register the UTM derive function under [`PROVIDER_KEY`] into a given registry.
pub fn register_functions(registry: &mut FunctionsRegistry) {
    registry.register(
        PROVIDER_KEY,
        RegisteredFunction::Derive(Arc::new(UtmDeriveProvider)),
    );
}

/// Register the UTM derive function into alizarin-core's **global** function
/// registry — call once at binding init so the load-time derive hook can find it.
pub fn register_global() {
    alizarin_core::with_global_functions_registry_mut(register_functions);
}

#[cfg(test)]
mod tests {
    use super::*;
    use alizarin_core::{apply_derive_functions, default_functions_registry, StaticGraph};

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

    #[test]
    fn derive_from_geojson_via_registry() {
        let graph: StaticGraph = serde_json::from_value(serde_json::json!({
            "graphid": "g", "name": {"en": "G"},
            "root": {"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"},
            "nodes": [{"nodeid": "root", "name": "Root", "datatype": "semantic", "graph_id": "g"}],
            "edges": [], "nodegroups": [], "cards": [], "cards_x_nodes_x_widgets": [],
            "functions_x_graphs": [{
                "id": "fxg-utm", "function_id": "60000000-0000-0000-0000-000000000002", "graph_id": "g",
                "config": {"provider": "utm", "nodegroup": "utm-ng", "member_of": "base", "cache": true}
            }],
        })).expect("graph");

        let boundary = serde_json::json!({
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "properties": {}, "geometry": {
                "type": "Polygon",
                "coordinates": [[[80.24, 13.08], [80.26, 13.08], [80.26, 13.09], [80.24, 13.09], [80.24, 13.08]]]
            }}]
        });
        let mut data: HashMap<String, Value> = HashMap::new();
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
        let v = &utm_tile.data["utm-ng"];
        assert_eq!(v["zone"], 44);
        assert_eq!(v["hemisphere"], "N");
    }

    /// The plain-ORM path: `register_global()` populates alizarin-core's global
    /// registry, and the load-time hook reads it via `get_global_functions_registry()`.
    #[test]
    fn global_registry_round_trip() {
        alizarin_core::clear_global_functions_registry();
        // Empty global registry → derive is a no-op.
        assert!(alizarin_core::get_global_functions_registry()
            .derive(PROVIDER_KEY)
            .is_none());

        register_global();
        let reg = alizarin_core::get_global_functions_registry();
        assert!(
            reg.derive(PROVIDER_KEY).is_some(),
            "utm provider is globally registered"
        );

        alizarin_core::clear_global_functions_registry();
    }
}
