//! Alizarin CLM Extension
//!
//! This extension provides the "reference" datatype handler for Controlled List Manager
//! integration. It handles coercion of reference values to tile data format.
//!
//! ## Mutations
//!
//! When the `mutations` feature is enabled, this crate also provides:
//! - `ReferenceChangeCollectionHandler` - mutation to change a reference node's collection

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::ffi::c_void;

use alizarin_extension_api::{
    alizarin_free_coerce_result, alizarin_free_render_display_result,
    alizarin_free_resolve_markers_result,
    CoerceFn, CoerceResult, FreeFn, TypeHandlerInfo,
    RenderDisplayFn, RenderDisplayResult, FreeDisplayFn,
    ResolveMarkersFn, ResolveMarkersResult, FreeResolveMarkersFn,
    HasCollectionFn, ConceptLookupByIdFn, ConceptLookupByLabelFn, FreeConceptJsonFn,
};

// Re-export mutation types when feature is enabled
#[cfg(feature = "mutations")]
pub mod mutations;

// =============================================================================
// Display Serializer (requires alizarin-core)
// =============================================================================

#[cfg(feature = "mutations")]
mod display_serializer {
    use super::{StaticReference, Value};
    use alizarin_core::{
        ExtensionDisplaySerializer, SerializationOptions, SerializationResult,
        DisplaySerializerRegistry,
    };
    use std::sync::Arc;

    /// Display serializer for CLM reference type.
    ///
    /// This implements `ExtensionDisplaySerializer` from alizarin-core
    /// to provide display mode serialization for the `reference` datatype.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use alizarin_clm::display_serializer::create_clm_display_registry;
    /// use alizarin_core::SerializationContext;
    ///
    /// let registry = create_clm_display_registry();
    /// let ctx = SerializationContext::with_registry(&registry);
    /// ```
    pub struct ReferenceDisplaySerializer;

    impl ExtensionDisplaySerializer for ReferenceDisplaySerializer {
        fn serialize_display(
            &self,
            tile_data: &Value,
            options: &SerializationOptions,
        ) -> SerializationResult {
            let lang = Some(options.language.as_str());

            match tile_data {
                Value::Null => SerializationResult::success(Value::Null),

                // Array of references - join with ", "
                Value::Array(arr) => {
                    let displays: Vec<String> = arr
                        .iter()
                        .filter_map(|item| {
                            serde_json::from_value::<StaticReference>(item.clone())
                                .ok()
                                .map(|r| r.to_display_string(lang))
                        })
                        .collect();

                    if displays.is_empty() {
                        return SerializationResult::success(Value::Null);
                    }
                    SerializationResult::success(Value::String(displays.join(", ")))
                }

                // Single reference object
                Value::Object(_) => {
                    match serde_json::from_value::<StaticReference>(tile_data.clone()) {
                        Ok(reference) => {
                            SerializationResult::success(Value::String(reference.to_display_string(lang)))
                        }
                        Err(_) => {
                            // Not a StaticReference format - pass through
                            SerializationResult::success(tile_data.clone())
                        }
                    }
                }

                // Pass through other types
                _ => SerializationResult::success(tile_data.clone()),
            }
        }

        fn description(&self) -> &str {
            "Display serializer for CLM reference type (StaticReference format)"
        }
    }

    /// Create a DisplaySerializerRegistry with CLM serializers.
    ///
    /// Registers display serializers for:
    /// - `reference` - Single CLM reference
    /// - `reference-list` - Array of CLM references
    pub fn create_clm_display_registry() -> DisplaySerializerRegistry {
        let mut registry = DisplaySerializerRegistry::new();
        registry.register("reference", Arc::new(ReferenceDisplaySerializer));
        registry.register("reference-list", Arc::new(ReferenceDisplaySerializer));
        registry
    }
}

#[cfg(feature = "mutations")]
pub use display_serializer::{ReferenceDisplaySerializer, create_clm_display_registry};

// =============================================================================
// Static Reference Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticReferenceLabel {
    pub id: String,
    pub language_id: String,
    pub list_item_id: String,
    pub value: String,
    pub valuetype_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticReference {
    pub labels: Vec<StaticReferenceLabel>,
    pub list_id: String,
    pub uri: String,
}

impl StaticReference {
    /// Get the display string for this reference
    pub fn to_display_string(&self, lang: Option<&str>) -> String {
        if self.labels.len() == 1 {
            return self.labels[0].value.clone();
        }

        let target_lang = lang.unwrap_or("en");
        let mut pref_label: Option<&str> = None;

        for label in &self.labels {
            if label.valuetype_id == "prefLabel" {
                pref_label = Some(&label.value);
                if label.language_id == target_lang {
                    return label.value.clone();
                }
            }
        }

        pref_label.unwrap_or("(undefined)").to_string()
    }
}

// =============================================================================
// Node Config for Reference Type
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReferenceNodeConfig {
    #[serde(rename = "controlledList")]
    pub controlled_list: Option<String>,
    #[serde(rename = "rdmCollection")]
    pub rdm_collection: Option<String>,
    #[serde(rename = "multiValue")]
    pub multi_value: Option<bool>,
}

impl ReferenceNodeConfig {
    pub fn get_collection_id(&self) -> Option<&str> {
        self.controlled_list
            .as_deref()
            .or(self.rdm_collection.as_deref())
    }

    pub fn is_multi_value(&self) -> bool {
        self.multi_value.unwrap_or(false)
    }
}

// =============================================================================
// Coercion Logic
// =============================================================================

/// Coerce a value to reference tile data format
fn coerce_reference_value(value: &Value, config: &ReferenceNodeConfig) -> Result<(Value, Value), String> {
    // Helper to coerce single item (without multiValue wrapping)
    fn coerce_single(value: &Value, config: &ReferenceNodeConfig) -> Result<(Value, Value), String> {
        match value {
            // Pre-formed StaticReference objects are not valid coercion input.
            // Use a label string or UUID instead.
            Value::Object(obj) if obj.contains_key("labels") || obj.contains_key("list_id") => {
                Err(format!(
                    "Pre-formed reference objects are not valid input. \
                     Use a label string or UUID instead. Got: {:?}",
                    value
                ))
            }

            // String - could be UUID or label value, needs lookup
            Value::String(s) => {
                // TODO: Consider caching this regex in a static for performance
                let uuid_regex = regex::Regex::new(
                    r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
                ).expect("hardcoded UUID regex is valid");

                if uuid_regex.is_match(s) {
                    // UUID - needs RDM lookup by ID
                    Ok((
                        json!({"__needs_rdm_lookup": true, "uuid": s}),
                        json!({"__needs_rdm_lookup": true, "uuid": s}),
                    ))
                } else {
                    // Non-UUID string - could be a label value, needs RDM label lookup
                    // Return a marker for label-based lookup (Python will resolve against collection)
                    Ok((
                        json!({"__needs_rdm_label_lookup": true, "label": s, "controlledList": config.controlled_list}),
                        json!({"__needs_rdm_label_lookup": true, "label": s, "controlledList": config.controlled_list}),
                    ))
                }
            }

            // Array of references (for list type)
            Value::Array(arr) => {
                let mut tile_data = Vec::new();
                let mut resolved = Vec::new();

                for item in arr {
                    let (item_tile, item_resolved) = coerce_single(item, config)?;
                    tile_data.push(item_tile);
                    resolved.push(item_resolved);
                }

                Ok((json!(tile_data), json!(resolved)))
            }

            Value::Null => Ok((Value::Null, Value::Null)),

            _ => Err(format!("Could not coerce value to reference: {:?}", value)),
        }
    }

    let (tile_data, resolved) = coerce_single(value, config)?;

    // If multiValue is true and result is not already an array, wrap in array
    if config.multi_value == Some(true) && !matches!(tile_data, Value::Array(_) | Value::Null) {
        Ok((json!([tile_data]), json!([resolved])))
    } else {
        Ok((tile_data, resolved))
    }
}

// =============================================================================
// C ABI Handler
// =============================================================================

/// C ABI coercion function for reference type
unsafe extern "C" fn coerce_reference(
    value_ptr: *const u8,
    value_len: usize,
    config_ptr: *const u8,
    config_len: usize,
) -> CoerceResult {
    // Parse value JSON
    let value_slice = std::slice::from_raw_parts(value_ptr, value_len);
    let value_str = match std::str::from_utf8(value_slice) {
        Ok(s) => s,
        Err(e) => return CoerceResult::error(format!("Invalid UTF-8 in value: {}", e)),
    };

    let value: Value = match serde_json::from_str(value_str) {
        Ok(v) => v,
        Err(e) => return CoerceResult::error(format!("Invalid JSON value: {}", e)),
    };

    // Parse config JSON
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

    // Perform coercion
    match coerce_reference_value(&value, &config) {
        Ok((tile_data, resolved)) => {
            match (serde_json::to_vec(&tile_data), serde_json::to_vec(&resolved)) {
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
///
/// Takes a resolved StaticReference JSON and returns the display string
/// for the specified language.
unsafe extern "C" fn render_reference_display(
    resolved_ptr: *const u8,
    resolved_len: usize,
    lang_ptr: *const u8,
    lang_len: usize,
) -> RenderDisplayResult {
    // Parse resolved JSON
    let resolved_slice = std::slice::from_raw_parts(resolved_ptr, resolved_len);
    let resolved_str = match std::str::from_utf8(resolved_slice) {
        Ok(s) => s,
        Err(e) => return RenderDisplayResult::error(format!("Invalid UTF-8 in resolved: {}", e)),
    };

    // Parse language
    let lang_slice = std::slice::from_raw_parts(lang_ptr, lang_len);
    let lang = std::str::from_utf8(lang_slice).ok();

    // Handle arrays of references
    let resolved: Value = match serde_json::from_str(resolved_str) {
        Ok(v) => v,
        Err(e) => return RenderDisplayResult::error(format!("Invalid JSON: {}", e)),
    };

    match &resolved {
        // Single reference object
        Value::Object(_) => {
            let reference: StaticReference = match serde_json::from_value(resolved.clone()) {
                Ok(r) => r,
                Err(e) => return RenderDisplayResult::error(format!("Invalid reference: {}", e)),
            };
            RenderDisplayResult::success(reference.to_display_string(lang))
        }

        // Array of references - join with ", "
        Value::Array(arr) => {
            let mut displays = Vec::new();
            for item in arr {
                match serde_json::from_value::<StaticReference>(item.clone()) {
                    Ok(reference) => displays.push(reference.to_display_string(lang)),
                    Err(_) => continue, // Skip invalid items
                }
            }
            RenderDisplayResult::success(displays.join(", "))
        }

        Value::Null => RenderDisplayResult::success(String::new()),

        _ => RenderDisplayResult::error(format!("Unexpected resolved type: {:?}", resolved)),
    }
}

// =============================================================================
// Marker Resolution
// =============================================================================

/// Resolve a single reference value that may contain markers
fn resolve_single_reference(
    value: &Value,
    config: &ReferenceNodeConfig,
    has_collection: HasCollectionFn,
    lookup_by_id: ConceptLookupByIdFn,
    lookup_by_label: ConceptLookupByLabelFn,
    free_concept_json: FreeConceptJsonFn,
    lookup_user_data: *mut c_void,
) -> Result<Option<Value>, String> {
    match value {
        Value::Object(obj) => {
            // Check for __needs_rdm_lookup marker (UUID-based lookup)
            if obj.contains_key("__needs_rdm_lookup") {
                let uuid = obj.get("uuid")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing uuid in __needs_rdm_lookup marker")?;

                let collection_id = config.get_collection_id()
                    .ok_or("Missing collection ID for RDM lookup")?;

                // Check if collection is in cache first
                let collection_exists = unsafe {
                    has_collection(
                        lookup_user_data,
                        collection_id.as_ptr(),
                        collection_id.len(),
                    )
                };

                if !collection_exists {
                    return Err(format!(
                        "Collection '{}' not found in cache. Load the collection before resolving references.",
                        collection_id
                    ));
                }

                // Call lookup callback
                let mut concept_json_ptr: *mut u8 = std::ptr::null_mut();
                let mut concept_json_len: usize = 0;

                let found = unsafe {
                    lookup_by_id(
                        lookup_user_data,
                        collection_id.as_ptr(),
                        collection_id.len(),
                        uuid.as_ptr(),
                        uuid.len(),
                        &mut concept_json_ptr,
                        &mut concept_json_len,
                    )
                };

                if !found || concept_json_ptr.is_null() {
                    return Err(format!("Concept not found: {} in {}", uuid, collection_id));
                }

                // Parse the concept JSON returned by callback
                let concept_json = unsafe {
                    let slice = std::slice::from_raw_parts(concept_json_ptr, concept_json_len);
                    let result = std::str::from_utf8(slice)
                        .map_err(|e| format!("Invalid UTF-8 in concept JSON: {}", e))
                        .and_then(|s| serde_json::from_str::<Value>(s)
                            .map_err(|e| format!("Invalid concept JSON: {}", e)));
                    // Free the concept JSON
                    free_concept_json(concept_json_ptr, concept_json_len);
                    result
                }?;

                // Build StaticReference from concept
                let reference = build_static_reference_from_concept(&concept_json, collection_id)?;
                return Ok(Some(serde_json::to_value(reference)
                    .map_err(|e| format!("Failed to serialize reference: {}", e))?));
            }

            // Check for __needs_rdm_label_lookup marker (label-based lookup)
            if obj.contains_key("__needs_rdm_label_lookup") {
                let label = obj.get("label")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing label in __needs_rdm_label_lookup marker")?;

                let collection_id = config.get_collection_id()
                    .ok_or("Missing collection ID for RDM label lookup")?;

                // Check if collection is in cache first
                let collection_exists = unsafe {
                    has_collection(
                        lookup_user_data,
                        collection_id.as_ptr(),
                        collection_id.len(),
                    )
                };

                if !collection_exists {
                    return Err(format!(
                        "Collection '{}' not found in cache. Load the collection before resolving references.",
                        collection_id
                    ));
                }

                // Call lookup callback
                let mut concept_json_ptr: *mut u8 = std::ptr::null_mut();
                let mut concept_json_len: usize = 0;

                let found = unsafe {
                    lookup_by_label(
                        lookup_user_data,
                        collection_id.as_ptr(),
                        collection_id.len(),
                        label.as_ptr(),
                        label.len(),
                        &mut concept_json_ptr,
                        &mut concept_json_len,
                    )
                };

                if !found || concept_json_ptr.is_null() {
                    return Err(format!("Concept not found by label: {} in {}", label, collection_id));
                }

                // Parse the concept JSON returned by callback
                let concept_json = unsafe {
                    let slice = std::slice::from_raw_parts(concept_json_ptr, concept_json_len);
                    let result = std::str::from_utf8(slice)
                        .map_err(|e| format!("Invalid UTF-8 in concept JSON: {}", e))
                        .and_then(|s| serde_json::from_str::<Value>(s)
                            .map_err(|e| format!("Invalid concept JSON: {}", e)));
                    // Free the concept JSON
                    free_concept_json(concept_json_ptr, concept_json_len);
                    result
                }?;

                // Build StaticReference from concept
                let reference = build_static_reference_from_concept(&concept_json, collection_id)?;
                return Ok(Some(serde_json::to_value(reference)
                    .map_err(|e| format!("Failed to serialize reference: {}", e))?));
            }

            // Already a resolved reference or unknown object - no change
            Ok(None)
        }

        Value::Array(arr) => {
            let mut modified = false;
            let mut resolved_arr = Vec::with_capacity(arr.len());

            for item in arr {
                match resolve_single_reference(
                    item, config, has_collection, lookup_by_id, lookup_by_label, free_concept_json, lookup_user_data
                )? {
                    Some(resolved) => {
                        modified = true;
                        resolved_arr.push(resolved);
                    }
                    None => {
                        resolved_arr.push(item.clone());
                    }
                }
            }

            if modified {
                Ok(Some(Value::Array(resolved_arr)))
            } else {
                Ok(None)
            }
        }

        _ => Ok(None),
    }
}

/// Build a StaticReference from RDM concept JSON
fn build_static_reference_from_concept(concept: &Value, collection_id: &str) -> Result<StaticReference, String> {
    let concept_id = concept.get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing id in concept")?;

    // Build URI from collection and concept ID
    let uri = format!("urn:uuid:{}", concept_id);

    // Extract labels from pref_label
    // Handles both formats:
    // - Simple: { "en": "Label" }
    // - WithId: { "en": { "id": "...", "value": "Label" } }
    let mut labels = Vec::new();
    if let Some(pref_label) = concept.get("pref_label").and_then(|v| v.as_object()) {
        for (lang_id, value) in pref_label {
            // Try as plain string first
            let label_value = if let Some(s) = value.as_str() {
                Some(s.to_string())
            } else if let Some(obj) = value.as_object() {
                // Try as RdmValue object with "value" field
                obj.get("value").and_then(|v| v.as_str()).map(|s| s.to_string())
            } else {
                None
            };

            if let Some(label_text) = label_value {
                // Use the label's own ID if available, otherwise generate one
                let label_id = value.as_object()
                    .and_then(|obj| obj.get("id"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{}-{}", concept_id, lang_id));

                labels.push(StaticReferenceLabel {
                    id: label_id,
                    language_id: lang_id.clone(),
                    list_item_id: concept_id.to_string(),
                    value: label_text,
                    valuetype_id: "prefLabel".to_string(),
                });
            }
        }
    }

    // If no pref_label, try to get label from other fields
    if labels.is_empty() {
        if let Some(label) = concept.get("label").and_then(|v| v.as_str()) {
            labels.push(StaticReferenceLabel {
                id: format!("{}-en", concept_id),
                language_id: "en".to_string(),
                list_item_id: concept_id.to_string(),
                value: label.to_string(),
                valuetype_id: "prefLabel".to_string(),
            });
        }
    }

    Ok(StaticReference {
        labels,
        list_id: collection_id.to_string(),
        uri,
    })
}

/// C ABI marker resolution function for reference type
///
/// Resolves `__needs_rdm_lookup` and `__needs_rdm_label_lookup` markers
/// to full StaticReference objects with embedded labels.
unsafe extern "C" fn resolve_reference_markers(
    value_ptr: *const u8,
    value_len: usize,
    config_ptr: *const u8,
    config_len: usize,
    has_collection: HasCollectionFn,
    lookup_by_id: ConceptLookupByIdFn,
    lookup_by_label: ConceptLookupByLabelFn,
    free_concept_json: FreeConceptJsonFn,
    lookup_user_data: *mut c_void,
) -> ResolveMarkersResult {
    // Parse value JSON
    let value_slice = std::slice::from_raw_parts(value_ptr, value_len);
    let value_str = match std::str::from_utf8(value_slice) {
        Ok(s) => s,
        Err(e) => return ResolveMarkersResult::error(format!("Invalid UTF-8 in value: {}", e)),
    };

    let value: Value = match serde_json::from_str(value_str) {
        Ok(v) => v,
        Err(e) => return ResolveMarkersResult::error(format!("Invalid JSON value: {}", e)),
    };

    // Parse config JSON
    let config: ReferenceNodeConfig = if config_len > 0 && !config_ptr.is_null() {
        let config_slice = std::slice::from_raw_parts(config_ptr, config_len);
        let config_str = match std::str::from_utf8(config_slice) {
            Ok(s) => s,
            Err(_) => return ResolveMarkersResult::error("Invalid UTF-8 in config".to_string()),
        };

        serde_json::from_str::<ReferenceNodeConfig>(config_str).unwrap_or_default()
    } else {
        ReferenceNodeConfig::default()
    };

    // Resolve markers
    match resolve_single_reference(
        &value,
        &config,
        has_collection,
        lookup_by_id,
        lookup_by_label,
        free_concept_json,
        lookup_user_data,
    ) {
        Ok(Some(resolved)) => {
            match serde_json::to_vec(&resolved) {
                Ok(json) => ResolveMarkersResult::success(json),
                Err(e) => ResolveMarkersResult::error(format!("Failed to serialize resolved value: {}", e)),
            }
        }
        Ok(None) => ResolveMarkersResult::unchanged(),
        Err(e) => ResolveMarkersResult::error(e),
    }
}

// =============================================================================
// Python Module
// =============================================================================

#[cfg(feature = "pyo3-ext")]
mod python_module {
    use super::*;
    use pyo3::prelude::*;
    use pyo3::types::PyCapsule;
    use std::ffi::{c_void, CString};
    use std::sync::Once;

    /// Static storage for the TypeHandlerInfo
    /// This must be static because the capsule needs a stable pointer
    static mut HANDLER_INFO: Option<TypeHandlerInfo> = None;
    static INIT: Once = Once::new();

    /// Get the type handler capsule for registration with alizarin
    ///
    /// This handler includes display rendering support for toDisplayJson().
    #[pyfunction]
    pub fn get_reference_handler_capsule(py: Python<'_>) -> PyResult<Py<PyCapsule>> {
        static TYPE_NAME: &[u8] = b"reference";

        // Initialize the static handler info once
        INIT.call_once(|| {
            unsafe {
                HANDLER_INFO = Some(TypeHandlerInfo {
                    type_name_ptr: TYPE_NAME.as_ptr(),
                    type_name_len: TYPE_NAME.len(),
                    coerce_fn: coerce_reference as CoerceFn,
                    free_fn: alizarin_free_coerce_result as FreeFn,
                    render_display_fn: Some(render_reference_display as RenderDisplayFn),
                    free_display_fn: Some(alizarin_free_render_display_result as FreeDisplayFn),
                    resolve_markers_fn: Some(resolve_reference_markers as ResolveMarkersFn),
                    free_resolve_markers_fn: Some(alizarin_free_resolve_markers_result as FreeResolveMarkersFn),
                    user_data: std::ptr::null_mut(),
                });
            }
        });

        // Get pointer to the static handler info
        // SAFETY: HANDLER_INFO is initialized unconditionally in Once::call_once above
        #[allow(static_mut_refs)]
        let ptr = unsafe {
            HANDLER_INFO.as_ref().expect("HANDLER_INFO initialized in Once::call_once above")
                as *const TypeHandlerInfo
        };

        // SAFETY: Hardcoded string with no null bytes (capsule name must be null-terminated)
        let name = CString::new("alizarin_clm.reference_handler")
            .expect("handler name contains no null bytes");

        // Create capsule using unsafe raw pointer approach
        // PyCapsule::new expects something that implements Send, so we use the raw FFI
        unsafe {
            let capsule = pyo3::ffi::PyCapsule_New(
                ptr as *mut c_void,
                name.as_ptr(),
                None,
            );

            if capsule.is_null() {
                return Err(PyErr::fetch(py));
            }

            Ok(Py::from_owned_ptr(py, capsule))
        }
    }

    /// Python module definition
    #[pymodule]
    pub fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(get_reference_handler_capsule, m)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(result.unwrap_err().contains("Pre-formed reference objects are not valid input"));
    }

    #[test]
    fn test_coerce_uuid_string() {
        let value = json!("550e8400-e29b-41d4-a716-446655440000");
        let config = ReferenceNodeConfig::default();
        let result = coerce_reference_value(&value, &config);

        assert!(result.is_ok());
        let (tile_data, _) = result.unwrap();
        assert_eq!(tile_data.get("__needs_rdm_lookup").and_then(|v| v.as_bool()), Some(true));
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
        assert_eq!(reference.to_display_string(None), "English Label"); // default to en
    }

    #[test]
    fn test_render_reference_display() {
        let reference_json = r#"{
            "labels": [{
                "id": "1",
                "language_id": "en",
                "list_item_id": "item-1",
                "value": "Test Label",
                "valuetype_id": "prefLabel"
            }],
            "list_id": "list-1",
            "uri": "http://example.com"
        }"#;

        unsafe {
            let result = render_reference_display(
                reference_json.as_ptr(),
                reference_json.len(),
                "en".as_ptr(),
                2,
            );

            assert!(!result.is_error());
            let display = std::str::from_utf8(
                std::slice::from_raw_parts(result.display_ptr, result.display_len)
            ).unwrap();
            assert_eq!(display, "Test Label");

            alizarin_free_render_display_result(result);
        }
    }

    #[test]
    fn test_render_reference_display_array() {
        let references_json = r#"[
            {
                "labels": [{"id": "1", "language_id": "en", "list_item_id": "item-1", "value": "Label A", "valuetype_id": "prefLabel"}],
                "list_id": "list-1",
                "uri": "http://example.com/a"
            },
            {
                "labels": [{"id": "2", "language_id": "en", "list_item_id": "item-2", "value": "Label B", "valuetype_id": "prefLabel"}],
                "list_id": "list-1",
                "uri": "http://example.com/b"
            }
        ]"#;

        unsafe {
            let result = render_reference_display(
                references_json.as_ptr(),
                references_json.len(),
                "en".as_ptr(),
                2,
            );

            assert!(!result.is_error());
            let display = std::str::from_utf8(
                std::slice::from_raw_parts(result.display_ptr, result.display_len)
            ).unwrap();
            assert_eq!(display, "Label A, Label B");

            alizarin_free_render_display_result(result);
        }
    }

    #[test]
    fn test_coerce_multivalue_wraps_single_in_array() {
        // Use a UUID string (valid input)
        let value = json!("550e8400-e29b-41d4-a716-446655440000");

        // Without multiValue - should return single object (marker)
        let config_single = ReferenceNodeConfig {
            controlled_list: Some("list-1".to_string()),
            rdm_collection: None,
            multi_value: Some(false),
        };
        let (tile_data, _) = coerce_reference_value(&value, &config_single).unwrap();
        assert!(tile_data.is_object(), "Without multiValue, should return object");

        // With multiValue=true - should wrap in array
        let config_multi = ReferenceNodeConfig {
            controlled_list: Some("list-1".to_string()),
            rdm_collection: None,
            multi_value: Some(true),
        };
        let (tile_data, _) = coerce_reference_value(&value, &config_multi).unwrap();
        assert!(tile_data.is_array(), "With multiValue=true, should return array");
        assert_eq!(tile_data.as_array().unwrap().len(), 1, "Array should contain one element");
    }

    #[test]
    fn test_coerce_multivalue_preserves_existing_array() {
        // Use an array of label strings (valid input)
        let value = json!(["Hotel/Inn", "Cinema"]);

        // With multiValue=true and already an array - should not double-wrap
        let config = ReferenceNodeConfig {
            controlled_list: Some("list-1".to_string()),
            rdm_collection: None,
            multi_value: Some(true),
        };
        let (tile_data, _) = coerce_reference_value(&value, &config).unwrap();
        assert!(tile_data.is_array(), "Should remain an array");
        assert_eq!(tile_data.as_array().unwrap().len(), 2, "Should have 2 elements, not double-wrapped");
    }
}
