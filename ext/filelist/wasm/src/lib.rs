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
    let json: serde_json::Value = serde_wasm_bindgen::from_value(value)
        .map_err(|e| JsError::new(&format!("Failed to parse value: {}", e)))?;

    let (tile_data, display_value) = alizarin_filelist_core::coerce_filelist_value(&json)
        .map_err(|e| JsError::new(&e))?;

    let result = serde_json::json!({
        "tileData": tile_data,
        "displayValue": display_value,
    });

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Render a file-list value to a display string.
///
/// @param tileData - The file-list tile data (array or single object)
/// @param language - Language code for display (e.g. "en")
/// @returns Display string or null
#[wasm_bindgen(js_name = renderFileListDisplay)]
pub fn render_file_list_display(
    tile_data: JsValue,
    language: &str,
) -> Result<Option<String>, JsError> {
    let json: serde_json::Value = serde_wasm_bindgen::from_value(tile_data)
        .map_err(|e| JsError::new(&format!("Failed to parse tile data: {}", e)))?;

    match alizarin_filelist_core::render_filelist_display_value(&json, Some(language)) {
        Ok(s) if s.is_empty() => Ok(None),
        Ok(s) => Ok(Some(s)),
        Err(e) => Err(JsError::new(&e)),
    }
}
