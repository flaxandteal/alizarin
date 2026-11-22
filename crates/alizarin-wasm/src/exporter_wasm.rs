//! WASM bindings for PrebuildExporter.
//!
//! Returns export data as JavaScript objects. The caller is responsible
//! for writing to whatever storage is appropriate (IndexedDB, download, etc).

use wasm_bindgen::prelude::*;

/// Core's export output as JS: `{relativePath, content}` files.
fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, JsError> {
    crate::js_json::to_value(value).map_err(|e| JsError::new(&e.to_string()))
}

/// Export registered graphs as data (no filesystem).
///
/// Returns an array of `{relativePath, content}` objects.
#[wasm_bindgen(js_name = exportGraphs)]
pub fn export_graphs(graph_ids: Vec<String>) -> Result<JsValue, JsError> {
    to_js(&alizarin_core::export_graphs(&graph_ids).map_err(|e| JsError::new(&e.to_string()))?)
}

/// Export all registered graphs as data.
///
/// Returns an array of `{relativePath, content}` objects.
#[wasm_bindgen(js_name = exportAllGraphs)]
pub fn export_all_graphs() -> Result<JsValue, JsError> {
    to_js(&alizarin_core::export_all_graphs().map_err(|e| JsError::new(&e.to_string()))?)
}

/// Build complete prebuild export data (no RDM from WASM): core's
/// `{graphFiles, referenceDataFiles, businessDataFiles}`, each a list of
/// `{relativePath, content}`, as NAPI's `buildExportData`. `graphIds`
/// limits it to those graphs (all registered ones without).
#[wasm_bindgen(js_name = buildPrebuildExport)]
pub fn build_prebuild_export(
    base_uri: &str,
    graph_ids: Option<Vec<String>>,
) -> Result<JsValue, JsError> {
    let data = alizarin_core::build_prebuild_export(graph_ids.as_deref(), None, base_uri)
        .map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&data)
}

/// Get IDs of all registered graphs.
#[wasm_bindgen(js_name = getRegisteredGraphIds)]
pub fn get_registered_graph_ids() -> Vec<String> {
    alizarin_core::get_registered_graph_ids()
}
