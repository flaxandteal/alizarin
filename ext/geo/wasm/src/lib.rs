//! WASM bindings for alizarin-geo-core.
//!
//! Exposes geojson-feature-collection validation to JavaScript, delegating to the
//! shared Rust implementation. The `@alizarin/geo` JS package uses this in a
//! `validate` handler registered via `registerExtensionHandler`.

use wasm_bindgen::prelude::*;

/// Validate a `geojson-feature-collection` value.
///
/// @param value - The tile value (a GeoJSON FeatureCollection)
/// @returns a `ValidationResult` — `{ valid, errors, warnings }`
#[wasm_bindgen(js_name = validateGeojson)]
pub fn validate_geojson(value: JsValue) -> Result<JsValue, JsError> {
    let json: serde_json::Value = serde_wasm_bindgen::from_value(value)
        .map_err(|e| JsError::new(&format!("Failed to parse value: {}", e)))?;

    let result = alizarin_geo_core::validate_geojson(&json);

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

fn from_js(value: JsValue, what: &str) -> Result<serde_json::Value, JsError> {
    serde_wasm_bindgen::from_value(value)
        .map_err(|e| JsError::new(&format!("Failed to parse {what}: {e}")))
}

fn to_js(value: &impl serde::Serialize) -> Result<JsValue, JsError> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {e}")))
}

fn batch(values: JsValue) -> Result<Vec<serde_json::Value>, JsError> {
    match from_js(values, "values")? {
        serde_json::Value::Array(items) => Ok(items),
        _ => Err(JsError::new("values must be an array")),
    }
}

/// The empty geometry summary. Registered as the handler's `foldIdentity`.
#[wasm_bindgen(js_name = geoFoldIdentity)]
pub fn geo_fold_identity() -> Result<JsValue, JsError> {
    to_js(&alizarin_geo_core::fold::identity())
}

/// Fold a batch of feature-collection values into a summary
/// (`{count, bbox, geometry_types}`). Registered as the handler's `fold`.
#[wasm_bindgen(js_name = geoFold)]
pub fn geo_fold(acc: JsValue, values: JsValue) -> Result<JsValue, JsError> {
    let values = batch(values)?;
    let refs: Vec<&serde_json::Value> = values.iter().collect();
    let summary = alizarin_geo_core::fold::fold(from_js(acc, "summary")?, &refs)
        .map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&summary)
}

/// Merge two geometry summaries. Registered as the handler's `foldMerge`.
#[wasm_bindgen(js_name = geoFoldMerge)]
pub fn geo_fold_merge(a: JsValue, b: JsValue) -> Result<JsValue, JsError> {
    let merged = alizarin_geo_core::fold::merge(from_js(a, "summary")?, from_js(b, "summary")?)
        .map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&merged)
}

/// Remove previously folded values: `{ summary, exact }`, where `exact` is false
/// if a removed geometry touched an edge of the bbox (kept, so a superset).
/// Registered as the handler's `foldRemove`.
#[wasm_bindgen(js_name = geoFoldRemove)]
pub fn geo_fold_remove(acc: JsValue, values: JsValue) -> Result<JsValue, JsError> {
    let values = batch(values)?;
    let refs: Vec<&serde_json::Value> = values.iter().collect();
    let (summary, exact) = alizarin_geo_core::fold::remove(from_js(acc, "summary")?, &refs)
        .map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&serde_json::json!({ "summary": summary, "exact": exact }))
}

/// Whether any geometry folded into `summary` could intersect `predicate`'s
/// `{bbox: [minx, miny, maxx, maxy]}`: `false` only when certain, `true` for
/// anything undecidable. Registered as the handler's `foldMayMatch`.
#[wasm_bindgen(js_name = geoFoldMayMatch)]
pub fn geo_fold_may_match(summary: JsValue, predicate: JsValue) -> bool {
    match (from_js(summary, "summary"), from_js(predicate, "predicate")) {
        (Ok(summary), Ok(predicate)) => alizarin_geo_core::fold::may_match(&summary, &predicate),
        _ => true,
    }
}

/// Set the max coordinate count per feature collection (or `null` to disable).
#[wasm_bindgen(js_name = setCoordLimit)]
pub fn set_coord_limit(limit: Option<usize>) {
    alizarin_geo_core::set_coord_limit(limit);
}
