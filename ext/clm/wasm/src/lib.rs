//! WASM bindings for alizarin-clm-core.
//!
//! Exposes `reference` (CLM) display rendering and node config to JavaScript,
//! delegating to the shared Rust implementation. The `@alizarin/clm` JS package
//! uses these in a handler registered via `registerExtensionHandler`, so the
//! display logic lives once in `alizarin-clm-core` rather than reimplemented in
//! TypeScript. (Coercion for the browser still happens in the view-model layer;
//! marker resolution is lazy in JS against the runtime RDM cache.)

use wasm_bindgen::prelude::*;

/// Render a `reference` tile value to a display string.
///
/// Tolerates unresolved `__needs_rdm_label_lookup` markers (falls back to the
/// marker's `label`), matching the browser's lazy-resolution path.
///
/// @param tileData - The reference tile data (array of references / markers, a
///                   single reference object, or null)
/// @param language - Language code for display (e.g. "en")
/// @returns Display string, or null when nothing is renderable
#[wasm_bindgen(js_name = renderReferenceDisplay)]
pub fn render_reference_display(
    tile_data: JsValue,
    language: &str,
) -> Result<Option<String>, JsError> {
    let json: serde_json::Value = serde_wasm_bindgen::from_value(tile_data)
        .map_err(|e| JsError::new(&format!("Failed to parse tile data: {}", e)))?;

    let display = alizarin_clm_core::render_reference_display_tile(&json, Some(language));
    if display.is_empty() {
        Ok(None)
    } else {
        Ok(Some(display))
    }
}

/// A reference node's config from its raw node config:
/// `{controlledList, rdmCollection, multiValue, collectionId}`. The same shape
/// the NAPI and Python handlers supply (one implementation, in
/// `alizarin-clm-core`).
#[wasm_bindgen(js_name = referenceNodeConfig)]
pub fn reference_node_config(config: JsValue) -> Result<JsValue, JsError> {
    let json: serde_json::Value = serde_wasm_bindgen::from_value(config)
        .map_err(|e| JsError::new(&format!("Failed to parse node config: {}", e)))?;
    let out = alizarin_clm_core::reference_node_config(&json);
    // Nulls stay null (not undefined), so the config matches NAPI's and Python's.
    serde::Serialize::serialize(
        &out,
        &serde_wasm_bindgen::Serializer::new()
            .serialize_maps_as_objects(true)
            .serialize_missing_as_null(true),
    )
    .map_err(|e| JsError::new(&format!("Failed to serialize node config: {}", e)))
}

/// Whether a reference node displays as a list (it is multi-valued).
#[wasm_bindgen(js_name = referenceDisplayAsList)]
pub fn reference_display_as_list(config: JsValue) -> bool {
    serde_wasm_bindgen::from_value::<serde_json::Value>(config)
        .map(|c| alizarin_clm_core::reference_node_config(&c)["multiValue"] == true)
        .unwrap_or(false)
}
