//! Shared geospatial type logic for the `geojson-feature-collection` datatype.
//!
//! Provides an [`ExtensionTypeHandler`] that **validates** geojson tile values
//! during `trees_to_tiles` (Rust-native — no FFI round-trip): GeoJSON structure,
//! coordinate ranges, and polygon-ring integrity. This is the geo analogue of the
//! CLM reference handler: checks the core datatype does not do today, moved to
//! generation-time so the fixes currently living in post-processing (e.g. the
//! polygon-nesting repair in Catalina's `fix_business_data.py`) surface at the
//! boundary instead.
//!
//! Validation-only for now — coercion/indexing stay with core. `register` this
//! handler for [`DATATYPE_NAME`] and the a129 validation pass will call it.

use std::sync::{Arc, RwLock};

use serde_json::Value;

use alizarin_core::extension_type_registry::{
    ExtensionError, ExtensionTypeHandler, HandlerCapabilities, ValidationResult,
};

/// The datatype name this handler registers for.
pub const DATATYPE_NAME: &str = "geojson-feature-collection";

/// WGS84 coordinate bounds.
const LON_RANGE: std::ops::RangeInclusive<f64> = -180.0..=180.0;
const LAT_RANGE: std::ops::RangeInclusive<f64> = -90.0..=90.0;

/// Default per-collection coordinate-count cap, matching Arches' Elasticsearch
/// limit in `GeojsonFeatureCollectionDataType`. Override or disable via
/// [`set_coord_limit`].
pub const DEFAULT_COORD_LIMIT: usize = 1500;

/// Process-wide coordinate-count limit: `Some(n)` caps at `n`, `None` disables
/// the check. Configured via [`set_coord_limit`] / [`reset_coord_limit`] — the
/// same global-setting pattern the CLM extension uses for its base URI.
static COORD_LIMIT: RwLock<Option<usize>> = RwLock::new(Some(DEFAULT_COORD_LIMIT));

/// Set the coordinate-count limit: `Some(n)` to cap, `None` to disable the check.
pub fn set_coord_limit(limit: Option<usize>) {
    if let Ok(mut guard) = COORD_LIMIT.write() {
        *guard = limit;
    }
}

/// The current coordinate-count limit (`None` = check disabled).
pub fn get_coord_limit() -> Option<usize> {
    COORD_LIMIT.read().ok().and_then(|g| *g)
}

/// Reset the coordinate-count limit to [`DEFAULT_COORD_LIMIT`].
pub fn reset_coord_limit() {
    set_coord_limit(Some(DEFAULT_COORD_LIMIT));
}

/// Handler for the `geojson-feature-collection` datatype (validation-only).
pub struct GeoTypeHandler;

impl ExtensionTypeHandler for GeoTypeHandler {
    fn capabilities(&self) -> HandlerCapabilities {
        HandlerCapabilities {
            can_coerce: false,
            can_render_display: false,
            can_render_search: false,
            can_resolve_markers: false,
            can_index: false,
            can_validate: true,
        }
    }

    fn validate(
        &self,
        value: &Value,
        _config: Option<&Value>,
    ) -> Result<ValidationResult, ExtensionError> {
        Ok(validate_geojson(value))
    }

    fn description(&self) -> &str {
        "Geospatial (geojson-feature-collection) type handler"
    }
}

/// Validate a `geojson-feature-collection` tile value.
///
/// An absent geometry (null or `{}`) is valid — there is nothing to check.
/// Otherwise the value must parse as GeoJSON, and every geometry is checked for
/// coordinate ranges and polygon-ring integrity. A malformed polygon nesting
/// (coordinates one level too shallow) fails to parse and is reported as invalid
/// GeoJSON — exactly the case the post-processing step used to repair.
pub fn validate_geojson(value: &Value) -> ValidationResult {
    if value.is_null() || value.as_object().is_some_and(|o| o.is_empty()) {
        return ValidationResult::valid();
    }

    let gj = match geojson::GeoJson::from_json_value(value.clone()) {
        Ok(g) => g,
        Err(e) => return ValidationResult::invalid(format!("invalid GeoJSON: {e}")),
    };

    // Arches stores this datatype as a FeatureCollection
    // (`{"type":"FeatureCollection","features":[…]}` — see
    // GeojsonFeatureCollectionDataType). A bare Feature or Geometry is the wrong
    // shape for a tile value: `transform_value_for_tile` would reject it.
    let fc = match gj {
        geojson::GeoJson::FeatureCollection(fc) => fc,
        geojson::GeoJson::Feature(_) => {
            return ValidationResult::invalid("expected a FeatureCollection, got a bare Feature")
        }
        geojson::GeoJson::Geometry(_) => {
            return ValidationResult::invalid("expected a FeatureCollection, got a bare Geometry")
        }
    };

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut coord_count = 0usize;
    for (i, feature) in fc.features.iter().enumerate() {
        if let Some(geom) = &feature.geometry {
            validate_geometry(&geom.value, i, &mut errors);
            coord_count += count_positions(&geom.value);
        }
        check_property_names(feature.properties.as_ref(), i, &mut warnings);
    }

    // Arches rejects a collection with too many coordinates for Elasticsearch.
    // The cap is configurable (`None` disables it); defaults to Arches' 1500.
    if let Some(limit) = get_coord_limit() {
        if coord_count > limit {
            errors.push(format!(
                "feature collection has {coord_count} coordinates (limit is {limit})"
            ));
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Count positions (coordinate pairs) in a geometry, recursively.
fn count_positions(geom: &geojson::Value) -> usize {
    use geojson::Value::*;
    match geom {
        Point(_) => 1,
        MultiPoint(p) | LineString(p) => p.len(),
        MultiLineString(lines) => lines.iter().map(Vec::len).sum(),
        Polygon(rings) => rings.iter().map(Vec::len).sum(),
        MultiPolygon(polys) => polys.iter().flatten().map(Vec::len).sum(),
        GeometryCollection(geoms) => geoms.iter().map(|g| count_positions(&g.value)).sum(),
    }
}

/// Warn about feature property names containing a dot. Arches indexes feature
/// properties into Elasticsearch, which treats `.` as object nesting — so a
/// feature with both `"x"` and `"x.2"` triggers a mapping conflict. The GeoJSON
/// is otherwise valid, hence a warning (the post-processing step rewrote `.`→`_`).
fn check_property_names(
    properties: Option<&serde_json::Map<String, Value>>,
    feature_idx: usize,
    warnings: &mut Vec<String>,
) {
    if let Some(props) = properties {
        for key in props.keys() {
            if key.contains('.') {
                warnings.push(format!(
                    "feature {feature_idx}: property name '{key}' contains '.' — collides in Elasticsearch mapping"
                ));
            }
        }
    }
}

fn validate_geometry(geom: &geojson::Value, feature_idx: usize, errors: &mut Vec<String>) {
    use geojson::Value::*;
    match geom {
        Point(pos) => check_position(pos, feature_idx, errors),
        MultiPoint(points) => points
            .iter()
            .for_each(|p| check_position(p, feature_idx, errors)),
        LineString(line) => line
            .iter()
            .for_each(|p| check_position(p, feature_idx, errors)),
        MultiLineString(lines) => lines
            .iter()
            .flatten()
            .for_each(|p| check_position(p, feature_idx, errors)),
        Polygon(rings) => check_polygon(rings, feature_idx, errors),
        MultiPolygon(polys) => polys
            .iter()
            .for_each(|rings| check_polygon(rings, feature_idx, errors)),
        GeometryCollection(geoms) => geoms
            .iter()
            .for_each(|g| validate_geometry(&g.value, feature_idx, errors)),
    }
}

fn check_polygon(rings: &[Vec<Vec<f64>>], feature_idx: usize, errors: &mut Vec<String>) {
    for (r, ring) in rings.iter().enumerate() {
        // A valid GeoJSON linear ring has >= 4 positions and is closed.
        if ring.len() < 4 {
            errors.push(format!(
                "feature {feature_idx}: polygon ring {r} has {} positions (need >= 4)",
                ring.len()
            ));
        } else if ring.first() != ring.last() {
            errors.push(format!(
                "feature {feature_idx}: polygon ring {r} is not closed"
            ));
        }
        ring.iter()
            .for_each(|p| check_position(p, feature_idx, errors));
    }
}

fn check_position(pos: &[f64], feature_idx: usize, errors: &mut Vec<String>) {
    if pos.len() < 2 {
        errors.push(format!(
            "feature {feature_idx}: position has {} coordinates (need >= 2)",
            pos.len()
        ));
        return;
    }
    let (lon, lat) = (pos[0], pos[1]);
    if !LON_RANGE.contains(&lon) {
        errors.push(format!(
            "feature {feature_idx}: longitude {lon} out of range [-180, 180]"
        ));
    }
    if !LAT_RANGE.contains(&lat) {
        errors.push(format!(
            "feature {feature_idx}: latitude {lat} out of range [-90, 90]"
        ));
    }
}

/// Create an `Arc<dyn ExtensionTypeHandler>` for the geospatial type.
pub fn create_geo_handler() -> Arc<dyn ExtensionTypeHandler> {
    Arc::new(GeoTypeHandler)
}

/// Register the geospatial handler for [`DATATYPE_NAME`] into `registry`.
pub fn register(registry: &mut alizarin_core::extension_type_registry::ExtensionTypeRegistry) {
    registry.register(DATATYPE_NAME.to_string(), Arc::new(GeoTypeHandler));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Serialises tests that mutate the process-wide coord limit.
    static LIMIT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    fn lock_limit() -> std::sync::MutexGuard<'static, ()> {
        LIMIT_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// FeatureCollection wrapping a single geometry with empty properties.
    fn fc(geometry: Value) -> Value {
        json!({
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": geometry, "properties": {}}]
        })
    }

    /// FeatureCollection wrapping a single geometry with the given properties.
    fn fc_props(geometry: Value, properties: Value) -> Value {
        json!({
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": geometry, "properties": properties}]
        })
    }

    fn point(lon: f64, lat: f64) -> Value {
        json!({"type": "Point", "coordinates": [lon, lat]})
    }

    // A closed square ring [(0,0),(0,1),(1,1),(1,0),(0,0)].
    fn closed_ring() -> Value {
        json!([[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]])
    }

    #[test]
    fn capability_is_validate_only() {
        let caps = GeoTypeHandler.capabilities();
        assert!(caps.can_validate);
        assert!(!caps.can_coerce && !caps.can_index && !caps.can_resolve_markers);
        assert!(!caps.can_render_display && !caps.can_render_search);
    }

    // ---- succeeding geometries ----

    #[test]
    fn valid_geometries_of_every_type_pass() {
        let cases = vec![
            point(151.84, -26.54),
            json!({"type": "MultiPoint", "coordinates": [[0.0, 0.0], [10.0, 10.0]]}),
            json!({"type": "LineString", "coordinates": [[0.0, 0.0], [1.0, 1.0], [2.0, 0.5]]}),
            json!({"type": "MultiLineString", "coordinates": [[[0.0, 0.0], [1.0, 1.0]]]}),
            json!({"type": "Polygon", "coordinates": [closed_ring()]}),
            json!({"type": "MultiPolygon", "coordinates": [[closed_ring()]]}),
            json!({"type": "GeometryCollection", "geometries": [point(1.0, 2.0)]}),
        ];
        for g in cases {
            let r = validate_geojson(&fc(g.clone()));
            assert!(r.valid, "should be valid: {g}  errors: {:?}", r.errors);
            assert!(r.warnings.is_empty());
        }
    }

    #[test]
    fn boundary_coordinates_are_valid() {
        for (lon, lat) in [(180.0, 90.0), (-180.0, -90.0), (0.0, 0.0)] {
            let r = validate_geojson(&fc(point(lon, lat)));
            assert!(r.valid, "({lon},{lat}) should be valid: {:?}", r.errors);
        }
    }

    #[test]
    fn absent_geometry_is_valid() {
        assert!(validate_geojson(&Value::Null).valid);
        assert!(validate_geojson(&json!({})).valid);
        // A feature with an explicit null geometry is fine too.
        assert!(validate_geojson(&fc(Value::Null)).valid);
    }

    #[test]
    fn multiple_features_all_valid() {
        let v = json!({
            "type": "FeatureCollection",
            "features": [
                {"type": "Feature", "geometry": point(1.0, 2.0), "properties": {}},
                {"type": "Feature", "geometry": json!({"type": "Polygon", "coordinates": [closed_ring()]}), "properties": {}},
            ]
        });
        assert!(validate_geojson(&v).valid);
    }

    // ---- failing geometries (errors) ----

    #[test]
    fn longitude_out_of_range_is_invalid() {
        for lon in [200.0, -181.0] {
            let r = validate_geojson(&fc(point(lon, 0.0)));
            assert!(!r.valid);
            assert!(
                r.errors.iter().any(|e| e.contains("longitude")),
                "{:?}",
                r.errors
            );
        }
    }

    #[test]
    fn latitude_out_of_range_is_invalid() {
        for lat in [100.0, -90.5] {
            let r = validate_geojson(&fc(point(0.0, lat)));
            assert!(!r.valid);
            assert!(
                r.errors.iter().any(|e| e.contains("latitude")),
                "{:?}",
                r.errors
            );
        }
    }

    #[test]
    fn unclosed_polygon_ring_is_invalid() {
        // 4 positions but first != last.
        let r = validate_geojson(&fc(json!({
            "type": "Polygon",
            "coordinates": [[[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]]
        })));
        assert!(!r.valid);
        assert!(
            r.errors.iter().any(|e| e.contains("not closed")),
            "{:?}",
            r.errors
        );
    }

    #[test]
    fn polygon_ring_with_too_few_positions_is_invalid() {
        // A triangle needs 4 positions (closed); 3 is too few.
        let r = validate_geojson(&fc(json!({
            "type": "Polygon",
            "coordinates": [[[0.0, 0.0], [1.0, 1.0], [0.0, 0.0]]]
        })));
        assert!(!r.valid);
        assert!(
            r.errors.iter().any(|e| e.contains("need >= 4")),
            "{:?}",
            r.errors
        );
    }

    #[test]
    fn multipolygon_with_unclosed_ring_is_invalid() {
        let r = validate_geojson(&fc(json!({
            "type": "MultiPolygon",
            "coordinates": [[[[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]]]
        })));
        assert!(!r.valid);
        assert!(
            r.errors.iter().any(|e| e.contains("not closed")),
            "{:?}",
            r.errors
        );
    }

    #[test]
    fn shallow_polygon_nesting_is_reported_as_invalid_geojson() {
        // fix_business_data.py::fix_polygon_nesting — Polygon coordinates one
        // level short ([[lon,lat],…] instead of [[[lon,lat],…]]). Cannot parse as
        // a Polygon; surfaced here instead of needing a downstream repair.
        let r = validate_geojson(&fc(json!({
            "type": "Polygon",
            "coordinates": [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [0.0, 0.0]]
        })));
        assert!(!r.valid);
        assert!(
            r.errors.iter().any(|e| e.contains("invalid GeoJSON")),
            "{:?}",
            r.errors
        );
    }

    #[test]
    fn error_identifies_the_offending_feature_index() {
        let v = json!({
            "type": "FeatureCollection",
            "features": [
                {"type": "Feature", "geometry": point(1.0, 2.0), "properties": {}},
                {"type": "Feature", "geometry": point(999.0, 2.0), "properties": {}},
            ]
        });
        let r = validate_geojson(&v);
        assert!(!r.valid);
        assert!(
            r.errors.iter().any(|e| e.contains("feature 1")),
            "{:?}",
            r.errors
        );
    }

    // ---- data-quality warnings (valid geometry) ----

    #[test]
    fn dotted_property_name_is_a_warning_not_an_error() {
        // fix_business_data.py::fix_dotted_property_names — "caids_matched.2"
        // collides in Elasticsearch. The GeoJSON is valid, so it's a warning.
        let r = validate_geojson(&fc_props(
            point(1.0, 2.0),
            json!({"caids_matched": "a", "caids_matched.2": "b"}),
        ));
        assert!(r.valid, "geometry is valid: {:?}", r.errors);
        assert_eq!(r.warnings.len(), 1, "{:?}", r.warnings);
        assert!(
            r.warnings[0].contains("caids_matched.2"),
            "{:?}",
            r.warnings
        );
    }

    #[test]
    fn clean_property_names_produce_no_warning() {
        let r = validate_geojson(&fc_props(
            point(1.0, 2.0),
            json!({"caids_matched": "a", "caids_matched_2": "b"}),
        ));
        assert!(
            r.valid && r.warnings.is_empty(),
            "{:?} {:?}",
            r.errors,
            r.warnings
        );
    }

    // ---- shape: Arches expects a FeatureCollection ----

    #[test]
    fn bare_geometry_or_feature_is_rejected() {
        // A raw Point — the geometry, not a FeatureCollection.
        let r = validate_geojson(&point(1.0, 2.0));
        assert!(!r.valid);
        assert!(
            r.errors
                .iter()
                .any(|e| e.contains("expected a FeatureCollection")),
            "{:?}",
            r.errors
        );

        // A bare Feature (not wrapped in a collection).
        let r = validate_geojson(
            &json!({"type": "Feature", "geometry": point(1.0, 2.0), "properties": {}}),
        );
        assert!(!r.valid);
        assert!(
            r.errors
                .iter()
                .any(|e| e.contains("expected a FeatureCollection")),
            "{:?}",
            r.errors
        );
    }

    #[test]
    fn too_many_coordinates_is_invalid_at_default_limit() {
        let _g = lock_limit();
        reset_coord_limit(); // default 1500
                             // A LineString with > 1500 in-range positions — only the coord-limit
                             // error should fire (coords are valid, so no range errors).
        let coords: Vec<Value> = (0..1600).map(|i| json!([(i % 180) as f64, 0.0])).collect();
        let r = validate_geojson(&fc(json!({"type": "LineString", "coordinates": coords})));
        assert!(!r.valid);
        assert!(
            r.errors.iter().any(|e| e.contains("limit")),
            "{:?}",
            r.errors
        );
    }

    #[test]
    fn coord_limit_can_be_disabled() {
        let _g = lock_limit();
        set_coord_limit(None); // off
        let coords: Vec<Value> = (0..1600).map(|i| json!([(i % 180) as f64, 0.0])).collect();
        let r = validate_geojson(&fc(json!({"type": "LineString", "coordinates": coords})));
        assert!(r.valid, "limit off → valid: {:?}", r.errors);
        reset_coord_limit();
    }

    #[test]
    fn coord_limit_can_be_lowered() {
        let _g = lock_limit();
        set_coord_limit(Some(2));
        // 3 positions > 2.
        let r = validate_geojson(&fc(json!({
            "type": "LineString", "coordinates": [[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]]
        })));
        assert!(!r.valid);
        assert!(
            r.errors.iter().any(|e| e.contains("limit is 2")),
            "{:?}",
            r.errors
        );
        reset_coord_limit();
    }

    #[test]
    fn default_coord_limit_matches_arches() {
        let _g = lock_limit();
        reset_coord_limit();
        assert_eq!(DEFAULT_COORD_LIMIT, 1500);
        assert_eq!(get_coord_limit(), Some(DEFAULT_COORD_LIMIT));
    }
}
