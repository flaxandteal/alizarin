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
#[wasm_bindgen(js_name = renderReferenceDisplay, unchecked_return_type = "string | null")]
pub fn render_reference_display(tile_data: JsValue, language: &str) -> Result<JsValue, JsError> {
    let json: serde_json::Value = alizarin_extension_api::js_json::from_value(tile_data)
        .map_err(|e| JsError::new(&format!("Failed to parse tile data: {}", e)))?;

    let display = alizarin_clm_core::render_reference_display_tile(&json, Some(language));
    Ok(alizarin_extension_api::js_json::or_null(
        Some(display).filter(|d| !d.is_empty()),
    ))
}

/// A reference node's config from its raw node config:
/// `{controlledList, rdmCollection, multiValue, collectionId}`. The same shape
/// the NAPI and Python handlers supply (one implementation, in
/// `alizarin-clm-core`).
#[wasm_bindgen(js_name = referenceNodeConfig)]
pub fn reference_node_config(config: JsValue) -> Result<JsValue, JsError> {
    let json: serde_json::Value = alizarin_extension_api::js_json::from_value(config)
        .map_err(|e| JsError::new(&format!("Failed to parse node config: {}", e)))?;
    let out = alizarin_clm_core::reference_node_config(&json);
    alizarin_extension_api::js_json::to_value(&out)
        .map_err(|e| JsError::new(&format!("Failed to serialize node config: {}", e)))
}

/// Whether a reference node displays as a list (it is multi-valued).
#[wasm_bindgen(js_name = referenceDisplayAsList)]
pub fn reference_display_as_list(config: JsValue) -> bool {
    alizarin_extension_api::js_json::from_value::<serde_json::Value>(config)
        .map(|c| alizarin_clm_core::reference_node_config(&c)["multiValue"] == true)
        .unwrap_or(false)
}
