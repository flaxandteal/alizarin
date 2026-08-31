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
    use std::ffi::c_void;

    use pyo3::prelude::*;
    use pyo3::types::PyCapsule;
    use std::ffi::CString;

    // =========================================================================
    // PyCapsule Registration
    // =========================================================================


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
        // The TypeHandlerInfo + its extern-"C" wrappers live in clm-core
        // (alizarin_clm_core::c_abi), shared with the napi binding.
        let ptr = alizarin_clm_core::c_abi::reference_handler_type_info();

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
