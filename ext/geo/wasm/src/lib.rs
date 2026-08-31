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

/// Set the max coordinate count per feature collection (or `null` to disable).
#[wasm_bindgen(js_name = setCoordLimit)]
pub fn set_coord_limit(limit: Option<usize>) {
    alizarin_geo_core::set_coord_limit(limit);
}
