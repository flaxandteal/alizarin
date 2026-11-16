//! WASM bindings for alizarin-filelist-core.
//!
//! Exposes file-list coercion and display rendering to JavaScript,
//! delegating to the shared Rust implementation.

use wasm_bindgen::prelude::*;

/// Coerce a file-list value, sanitizing metadata fields.
///
/// Handles: null metadata → omitted, bare strings → i18n dicts,
/// missing index → auto-assigned.
///
/// @param value - Array of file objects, single file object, or null
/// @returns `{ tileData, displayValue }` with sanitized file-list data
#[wasm_bindgen(js_name = coerceFileList)]
pub fn coerce_file_list(value: JsValue) -> Result<JsValue, JsError> {
    let json: serde_json::Value = alizarin_extension_api::js_json::from_value(value)
        .map_err(|e| JsError::new(&format!("Failed to parse value: {}", e)))?;

    let (tile_data, display_value) =
        alizarin_filelist_core::coerce_filelist_value(&json).map_err(|e| JsError::new(&e))?;

    let result = serde_json::json!({
        "tileData": tile_data,
        "displayValue": display_value,
    });

    alizarin_extension_api::js_json::to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Render a file-list value to a display string.
///
/// @param tileData - The file-list tile data (array or single object)
/// @param language - Language code for display (e.g. "en")
/// @returns Display string or null
#[wasm_bindgen(js_name = renderFileListDisplay, unchecked_return_type = "string | null")]
pub fn render_file_list_display(tile_data: JsValue, language: &str) -> Result<JsValue, JsError> {
    let json: serde_json::Value = alizarin_extension_api::js_json::from_value(tile_data)
        .map_err(|e| JsError::new(&format!("Failed to parse tile data: {}", e)))?;

    let display = alizarin_filelist_core::render_filelist_display_value(&json, Some(language))
        .map_err(|e| JsError::new(&e))?;
    Ok(alizarin_extension_api::js_json::or_null(
        Some(display).filter(|d| !d.is_empty()),
    ))
}
