//! Alizarin CLM Extension
//!
//! This extension provides the "reference" datatype handler for Controlled List Manager
//! integration. It handles coercion of reference values to tile data format.
//!
//! ## Mutations
//!
//! When the `mutations` feature is enabled, this crate also provides:
//! - `ReferenceChangeCollectionHandler` - mutation to change a reference node's collection

// Re-export core types so downstream users don't need to depend on alizarin-clm-core directly
pub use alizarin_clm_core::{
    build_item_uri, build_static_reference_from_concept, clear_clm_base_uri,
    coerce_reference_value, create_reference_handler, get_clm_base_uri,
    render_reference_display_value, set_clm_base_uri, try_build_item_uri, ReferenceNodeConfig,
    ReferenceTypeHandler, StaticReference, StaticReferenceLabel, DATATYPE_NAME,
};

// Re-export mutation types when feature is enabled
#[cfg(feature = "mutations")]
pub mod mutations;

// =============================================================================
// Python Module (C ABI + pyo3)
// =============================================================================

#[cfg(feature = "pyo3-ext")]
mod python_module {
    use super::*;
    use serde_json::Value;
    use std::ffi::c_void;

    use alizarin_extension_api::{
        alizarin_free_coerce_result, alizarin_free_render_display_result,
        alizarin_free_resolve_markers_result, CoerceFn, CoerceResult, ConceptLookupByIdFn,
        ConceptLookupByLabelFn, FreeConceptJsonFn, FreeDisplayFn, FreeFn, FreeResolveMarkersFn,
        HasCollectionFn, RenderDisplayFn, RenderDisplayResult, ResolveMarkersFn,
        ResolveMarkersResult, TypeHandlerInfo,
    };

    use pyo3::prelude::*;
    use pyo3::types::PyCapsule;
    use std::ffi::CString;
    use std::sync::Once;

    // =========================================================================
    // C ABI Handler Functions
    // =========================================================================

    /// C ABI coercion function for reference type
    unsafe extern "C" fn coerce_reference(
        value_ptr: *const u8,
        value_len: usize,
        config_ptr: *const u8,
        config_len: usize,
    ) -> CoerceResult {
        let value_slice = std::slice::from_raw_parts(value_ptr, value_len);
        let value_str = match std::str::from_utf8(value_slice) {
            Ok(s) => s,
            Err(e) => return CoerceResult::error(format!("Invalid UTF-8 in value: {}", e)),
        };

        let value: Value = match serde_json::from_str(value_str) {
            Ok(v) => v,
            Err(e) => return CoerceResult::error(format!("Invalid JSON value: {}", e)),
        };

        let config: ReferenceNodeConfig = if config_len > 0 && !config_ptr.is_null() {
            let config_slice = std::slice::from_raw_parts(config_ptr, config_len);
            let config_str = match std::str::from_utf8(config_slice) {
                Ok(s) => s,
                Err(_) => return CoerceResult::error("Invalid UTF-8 in config".to_string()),
            };
            serde_json::from_str::<ReferenceNodeConfig>(config_str).unwrap_or_default()
        } else {
            ReferenceNodeConfig::default()
        };

        match coerce_reference_value(&value, &config) {
            Ok((tile_data, resolved)) => {
                match (
                    serde_json::to_vec(&tile_data),
                    serde_json::to_vec(&resolved),
                ) {
                    (Ok(tile_json), Ok(resolved_json)) => {
                        CoerceResult::success(tile_json, resolved_json)
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        CoerceResult::error(format!("Failed to serialize coerced value: {}", e))
                    }
                }
            }
            Err(e) => CoerceResult::error(e),
        }
    }

    /// C ABI display render function for reference type
    unsafe extern "C" fn render_reference_display(
        resolved_ptr: *const u8,
        resolved_len: usize,
        lang_ptr: *const u8,
        lang_len: usize,
    ) -> RenderDisplayResult {
        let resolved_slice = std::slice::from_raw_parts(resolved_ptr, resolved_len);
        let resolved_str = match std::str::from_utf8(resolved_slice) {
            Ok(s) => s,
            Err(e) => {
                return RenderDisplayResult::error(format!("Invalid UTF-8 in resolved: {}", e))
            }
        };

        let lang_slice = std::slice::from_raw_parts(lang_ptr, lang_len);
        let lang = std::str::from_utf8(lang_slice).ok();

        let resolved: Value = match serde_json::from_str(resolved_str) {
            Ok(v) => v,
            Err(e) => return RenderDisplayResult::error(format!("Invalid JSON: {}", e)),
        };

        match render_reference_display_value(&resolved, lang) {
            Ok(s) => RenderDisplayResult::success(s),
            Err(e) => RenderDisplayResult::error(e),
        }
    }

    // =========================================================================
    // Marker Resolution (delegates to core's resolve_reference_markers)
    // =========================================================================

    /// Call an FFI concept lookup callback and parse the returned JSON.
    ///
    /// # Safety
    /// All pointer arguments must be valid for the lifetime of this call.
    unsafe fn call_concept_lookup(
        lookup_fn: unsafe extern "C" fn(
            *mut c_void,
            *const u8,
            usize,
            *const u8,
            usize,
            *mut *mut u8,
            *mut usize,
        ) -> bool,
        free_fn: FreeConceptJsonFn,
        user_data: *mut c_void,
        collection_id: &str,
        key: &str,
    ) -> Option<Value> {
        let mut json_ptr: *mut u8 = std::ptr::null_mut();
        let mut json_len: usize = 0;

        let found = lookup_fn(
            user_data,
            collection_id.as_ptr(),
            collection_id.len(),
            key.as_ptr(),
            key.len(),
            &mut json_ptr,
            &mut json_len,
        );

        if !found || json_ptr.is_null() || json_len == 0 {
            return None;
        }

        let json_slice = std::slice::from_raw_parts(json_ptr, json_len);
        let result = std::str::from_utf8(json_slice)
            .ok()
            .and_then(|s| serde_json::from_str(s).ok());

        free_fn(json_ptr, json_len);
        result
    }

    /// C ABI marker resolution function for reference type.
    ///
    /// Uses the provided RDM lookup callbacks to resolve markers, bridging
    /// the shared library boundary (the CLM .so has its own copy of the global
    /// RDM cache, so we must use the callbacks from the alizarin Python binding
    /// which has the populated cache).
    unsafe extern "C" fn resolve_reference_markers(
        value_ptr: *const u8,
        value_len: usize,
        lang_ptr: *const u8,
        lang_len: usize,
        _has_collection: HasCollectionFn,
        lookup_by_id: ConceptLookupByIdFn,
        lookup_by_label: ConceptLookupByLabelFn,
        free_concept_json: FreeConceptJsonFn,
        lookup_user_data: *mut c_void,
    ) -> ResolveMarkersResult {
        let value_slice = std::slice::from_raw_parts(value_ptr, value_len);
        let value_str = match std::str::from_utf8(value_slice) {
            Ok(s) => s,
            Err(e) => return ResolveMarkersResult::error(format!("Invalid UTF-8 in value: {}", e)),
        };

        let value: Value = match serde_json::from_str(value_str) {
            Ok(v) => v,
            Err(e) => return ResolveMarkersResult::error(format!("Invalid JSON value: {}", e)),
        };

        let language = if lang_len > 0 && !lang_ptr.is_null() {
            std::str::from_utf8(std::slice::from_raw_parts(lang_ptr, lang_len)).unwrap_or("en")
        } else {
            "en"
        };

        let resolved = match alizarin_clm_core::resolve_reference_markers_with_lookups(
            &value,
            language,
            |collection_id, concept_id| unsafe {
                call_concept_lookup(
                    lookup_by_id,
                    free_concept_json,
                    lookup_user_data,
                    collection_id,
                    concept_id,
                )
            },
            |collection_id, label| unsafe {
                call_concept_lookup(
                    lookup_by_label,
                    free_concept_json,
                    lookup_user_data,
                    collection_id,
                    label,
                )
            },
        ) {
            Ok(v) => v,
            // Malformed reference data (e.g. a non-UUID concept id) is an input
            // problem — surface it rather than emitting/keeping bad data.
            Err(e) => return ResolveMarkersResult::error(e),
        };

        if resolved == value {
            ResolveMarkersResult::unchanged()
        } else {
            match serde_json::to_vec(&resolved) {
                Ok(json) => ResolveMarkersResult::success(json),
                Err(e) => ResolveMarkersResult::error(format!(
                    "Failed to serialize resolved value: {}",
                    e
                )),
            }
        }
    }

    // =========================================================================
    // PyCapsule Registration
    // =========================================================================

    static mut HANDLER_INFO: Option<TypeHandlerInfo> = None;
    static INIT: Once = Once::new();

    #[pyfunction]
    #[pyo3(name = "set_clm_base_uri")]
    pub fn py_set_clm_base_uri(uri: &str) {
        alizarin_clm_core::set_clm_base_uri(uri);
    }

    #[pyfunction]
    #[pyo3(name = "get_clm_base_uri")]
    pub fn py_get_clm_base_uri() -> Option<String> {
        alizarin_clm_core::get_clm_base_uri()
    }

    #[pyfunction]
    #[pyo3(name = "clear_clm_base_uri")]
    pub fn py_clear_clm_base_uri() {
        alizarin_clm_core::clear_clm_base_uri();
    }

    /// Build a reference item URI from a UUID, using the configured base or the
    /// process-wide default. Single source of truth for the default and the UUID
    /// check — Python delegates here rather than reimplementing either.
    #[pyfunction]
    #[pyo3(name = "build_item_uri")]
    pub fn py_build_item_uri(item_id: &str) -> PyResult<String> {
        alizarin_clm_core::try_build_item_uri(item_id)
            .map_err(pyo3::exceptions::PyValueError::new_err)
    }

    #[pyfunction]
    pub fn get_reference_handler_capsule(py: Python<'_>) -> PyResult<Py<PyCapsule>> {
        static TYPE_NAME: &[u8] = b"reference";

        INIT.call_once(|| unsafe {
            HANDLER_INFO = Some(TypeHandlerInfo {
                type_name_ptr: TYPE_NAME.as_ptr(),
                type_name_len: TYPE_NAME.len(),
                coerce_fn: Some(coerce_reference as CoerceFn),
                free_fn: Some(alizarin_free_coerce_result as FreeFn),
                render_display_fn: Some(render_reference_display as RenderDisplayFn),
                free_display_fn: Some(alizarin_free_render_display_result as FreeDisplayFn),
                resolve_markers_fn: Some(resolve_reference_markers as ResolveMarkersFn),
                free_resolve_markers_fn: Some(
                    alizarin_free_resolve_markers_result as FreeResolveMarkersFn,
                ),
                validate_fn: None,
                free_validate_fn: None,
                render_search_fn: None,
                free_render_search_fn: None,
                index_spec_fn: None,
                free_index_spec_fn: None,
                abi: alizarin_extension_api::abi_fingerprint(),
                user_data: std::ptr::null_mut(),
            });
        });

        #[allow(static_mut_refs)]
        let ptr = unsafe {
            HANDLER_INFO
                .as_ref()
                .expect("HANDLER_INFO initialized in Once::call_once above")
                as *const TypeHandlerInfo
        };

        let name = CString::new("alizarin_clm.reference_handler")
            .expect("handler name contains no null bytes");

        unsafe {
            let capsule = pyo3::ffi::PyCapsule_New(ptr as *mut c_void, name.as_ptr(), None);

            if capsule.is_null() {
                return Err(PyErr::fetch(py));
            }

            Ok(Py::from_owned_ptr(py, capsule))
        }
    }

    #[pymodule]
    pub fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(get_reference_handler_capsule, m)?)?;
        m.add_function(wrap_pyfunction!(py_set_clm_base_uri, m)?)?;
        m.add_function(wrap_pyfunction!(py_get_clm_base_uri, m)?)?;
        m.add_function(wrap_pyfunction!(py_clear_clm_base_uri, m)?)?;
        m.add_function(wrap_pyfunction!(py_build_item_uri, m)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_coerce_rejects_preformed_reference_object() {
        let value = json!({
            "labels": [{
                "id": "label-1",
                "language_id": "en",
                "list_item_id": "item-1",
                "value": "Test Item",
                "valuetype_id": "prefLabel"
            }],
            "list_id": "list-1",
            "uri": "http://example.com/item/1"
        });

        let config = ReferenceNodeConfig::default();
        let result = coerce_reference_value(&value, &config);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Pre-formed reference objects are not valid input"));
    }

    #[test]
    fn test_coerce_uuid_string() {
        let value = json!("550e8400-e29b-41d4-a716-446655440000");
        let config = ReferenceNodeConfig::default();
        let result = coerce_reference_value(&value, &config);

        assert!(result.is_ok());
        let (tile_data, _) = result.unwrap();
        assert!(
            tile_data.is_array(),
            "Single value should be wrapped in array"
        );
        let first = &tile_data.as_array().unwrap()[0];
        assert_eq!(
            first.get("__needs_rdm_lookup").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_static_reference_display() {
        let reference = StaticReference {
            labels: vec![
                StaticReferenceLabel {
                    id: "1".to_string(),
                    language_id: "en".to_string(),
                    list_item_id: "item-1".to_string(),
                    value: "English Label".to_string(),
                    valuetype_id: "prefLabel".to_string(),
                },
                StaticReferenceLabel {
                    id: "2".to_string(),
                    language_id: "es".to_string(),
                    list_item_id: "item-1".to_string(),
                    value: "Etiqueta Española".to_string(),
                    valuetype_id: "prefLabel".to_string(),
                },
            ],
            list_id: "list-1".to_string(),
            uri: "http://example.com".to_string(),
        };

        assert_eq!(reference.to_display_string(Some("en")), "English Label");
        assert_eq!(reference.to_display_string(Some("es")), "Etiqueta Española");
        assert_eq!(reference.to_display_string(None), "English Label");
    }

    #[test]
    fn test_render_reference_display() {
        let resolved = json!({
            "labels": [{"id": "1", "language_id": "en", "list_item_id": "item-1", "value": "Test Label", "valuetype_id": "prefLabel"}],
            "list_id": "list-1",
            "uri": "http://example.com"
        });
        assert_eq!(
            render_reference_display_value(&resolved, Some("en")).unwrap(),
            "Test Label"
        );
    }

    #[test]
    fn test_render_reference_display_array() {
        let resolved = json!([
            {"labels": [{"id": "1", "language_id": "en", "list_item_id": "item-1", "value": "Label A", "valuetype_id": "prefLabel"}], "list_id": "list-1", "uri": "http://example.com/a"},
            {"labels": [{"id": "2", "language_id": "en", "list_item_id": "item-2", "value": "Label B", "valuetype_id": "prefLabel"}], "list_id": "list-1", "uri": "http://example.com/b"}
        ]);
        assert_eq!(
            render_reference_display_value(&resolved, Some("en")).unwrap(),
            "Label A, Label B"
        );
    }

    #[test]
    fn test_coerce_multivalue_wraps_single_in_array() {
        let value = json!("550e8400-e29b-41d4-a716-446655440000");

        let config_single = ReferenceNodeConfig {
            controlled_list: Some("list-1".to_string()),
            rdm_collection: None,
            multi_value: Some(false),
        };
        let (tile_data, _) = coerce_reference_value(&value, &config_single).unwrap();
        assert!(tile_data.is_array(), "Single value always wrapped in array");
        assert_eq!(tile_data.as_array().unwrap().len(), 1);

        let config_multi = ReferenceNodeConfig {
            controlled_list: Some("list-1".to_string()),
            rdm_collection: None,
            multi_value: Some(true),
        };
        let (tile_data, _) = coerce_reference_value(&value, &config_multi).unwrap();
        assert!(
            tile_data.is_array(),
            "With multiValue=true, should return array"
        );
        assert_eq!(
            tile_data.as_array().unwrap().len(),
            1,
            "Array should contain one element"
        );
    }

    #[test]
    fn test_coerce_multivalue_preserves_existing_array() {
        let value = json!(["Hotel/Inn", "Cinema"]);

        let config = ReferenceNodeConfig {
            controlled_list: Some("list-1".to_string()),
            rdm_collection: None,
            multi_value: Some(true),
        };
        let (tile_data, _) = coerce_reference_value(&value, &config).unwrap();
        assert!(tile_data.is_array(), "Should remain an array");
        assert_eq!(
            tile_data.as_array().unwrap().len(),
            2,
            "Should have 2 elements, not double-wrapped"
        );
    }
}
