//! WASM bindings for PrebuildExporter.
//!
//! Returns export data as JavaScript objects. The caller is responsible
//! for writing to whatever storage is appropriate (IndexedDB, download, etc).

use wasm_bindgen::prelude::*;

/// Convert an ExportFile to a JS object `{relativePath, content}`.
fn export_file_to_js(file: &alizarin_core::ExportFile) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("relativePath"),
        &JsValue::from_str(&file.relative_path),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("content"),
        &JsValue::from_str(&file.content),
    );
    obj.into()
}

/// Export registered graphs as data (no filesystem).
///
/// Returns an array of `{relativePath, content}` objects.
#[wasm_bindgen(js_name = exportGraphs)]
pub fn export_graphs(graph_ids: Vec<String>) -> Result<JsValue, JsError> {
    let files =
        alizarin_core::export_graphs(&graph_ids).map_err(|e| JsError::new(&e.to_string()))?;

    let arr = js_sys::Array::new();
    for file in &files {
        arr.push(&export_file_to_js(file));
    }
    Ok(arr.into())
}

/// Export all registered graphs as data.
///
/// Returns an array of `{relativePath, content}` objects.
#[wasm_bindgen(js_name = exportAllGraphs)]
pub fn export_all_graphs() -> Result<JsValue, JsError> {
    let files = alizarin_core::export_all_graphs().map_err(|e| JsError::new(&e.to_string()))?;

    let arr = js_sys::Array::new();
    for file in &files {
        arr.push(&export_file_to_js(file));
    }
    Ok(arr.into())
}

/// Build complete prebuild export data (graphs only, no RDM from WASM).
///
/// Returns `{graphFiles: [{relativePath, content}], referenceDataFiles: [...]}`.
#[wasm_bindgen(js_name = buildPrebuildExport)]
pub fn build_prebuild_export(base_uri: &str) -> Result<JsValue, JsError> {
    let data = alizarin_core::build_prebuild_export(None, None, base_uri)
        .map_err(|e| JsError::new(&e.to_string()))?;

    let result = js_sys::Object::new();

    let graph_arr = js_sys::Array::new();
    for file in &data.graph_files {
        graph_arr.push(&export_file_to_js(file));
    }
    let _ = js_sys::Reflect::set(&result, &JsValue::from_str("graphFiles"), &graph_arr);

    let ref_arr = js_sys::Array::new();
    for file in &data.reference_data_files {
        ref_arr.push(&export_file_to_js(file));
    }
    let _ = js_sys::Reflect::set(&result, &JsValue::from_str("referenceDataFiles"), &ref_arr);

    Ok(result.into())
}

/// Get IDs of all registered graphs.
#[wasm_bindgen(js_name = getRegisteredGraphIds)]
pub fn get_registered_graph_ids() -> Vec<String> {
    alizarin_core::get_registered_graph_ids()
}
