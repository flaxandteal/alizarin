//! WASM bindings for RDM namespace management.
//!
//! Provides global namespace storage and deterministic UUID generation
//! for RDM collections and concepts.
//!
//! Usage from JavaScript:
//! ```javascript
//! import {
//!     setRdmNamespace,
//!     getRdmNamespace,
//!     hasRdmNamespace,
//!     clearRdmNamespace,
//!     generateCollectionUuid,
//!     generateConceptUuid
//! } from 'alizarin-wasm';
//!
//! // Set namespace from URL (same URL can be used as RDF namespace)
//! setRdmNamespace("http://example.org/rdm/");
//!
//! // Or use a UUID directly
//! setRdmNamespace("550e8400-e29b-41d4-a716-446655440000");
//!
//! // Generate deterministic UUIDs
//! const collectionId = generateCollectionUuid("MyCollection");
//! const conceptId = generateConceptUuid(collectionId, "ConceptA");
//! ```

use std::cell::RefCell;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use alizarin_core::rdm_namespace::{
    generate_collection_uuid as core_generate_collection,
    generate_concept_uuid_from_str as core_generate_concept,
    generate_value_uuid as core_generate_value,
    labels_to_deterministic_string as core_labels_to_string,
    parse_rdm_namespace as core_parse_namespace,
};

// =============================================================================
// Global Namespace Storage
// =============================================================================

// Thread-local storage for global namespace (WASM is single-threaded)
thread_local! {
    static GLOBAL_RDM_NAMESPACE: RefCell<Option<Uuid>> = const { RefCell::new(None) };
    static GLOBAL_RDM_NAMESPACE_RAW: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Set the global RDM namespace for deterministic UUID generation.
///
/// This namespace is used when creating collections and concepts from labels
/// without explicit IDs.
///
/// @param namespace - Either a valid UUID string or a URL (http/https).
///                    If a URL is provided, it will be converted to a
///                    deterministic UUID using UUID5 with the standard URL namespace.
///
/// @example
/// // Using a URL (same URL can be used as RDF namespace)
/// setRdmNamespace("http://example.org/rdm/");
///
/// @example
/// // Using a UUID directly
/// setRdmNamespace("550e8400-e29b-41d4-a716-446655440000");
#[wasm_bindgen(js_name = setRdmNamespace)]
pub fn set_rdm_namespace(namespace: &str) -> Result<(), JsError> {
    let uuid = core_parse_namespace(namespace).map_err(|e| JsError::new(&e))?;

    GLOBAL_RDM_NAMESPACE.with(|ns| {
        *ns.borrow_mut() = Some(uuid);
    });
    GLOBAL_RDM_NAMESPACE_RAW.with(|ns| {
        *ns.borrow_mut() = Some(namespace.to_string());
    });

    Ok(())
}

/// Get the current global RDM namespace.
///
/// @returns The namespace UUID as a string, or undefined if not set
#[wasm_bindgen(js_name = getRdmNamespace)]
pub fn get_rdm_namespace() -> Option<String> {
    GLOBAL_RDM_NAMESPACE.with(|ns| ns.borrow().map(|u| u.to_string()))
}

/// Get the original namespace string passed to setRdmNamespace.
///
/// Unlike getRdmNamespace which returns the derived UUID, this returns
/// the raw input (URL or UUID string). Useful as an RDF base URI.
///
/// @returns The original namespace string, or undefined if not set
#[wasm_bindgen(js_name = getRdmNamespaceRaw)]
pub fn get_rdm_namespace_raw() -> Option<String> {
    GLOBAL_RDM_NAMESPACE_RAW.with(|ns| ns.borrow().clone())
}

/// Check if a global RDM namespace is set.
///
/// @returns true if a namespace is set
#[wasm_bindgen(js_name = hasRdmNamespace)]
pub fn has_rdm_namespace() -> bool {
    GLOBAL_RDM_NAMESPACE.with(|ns| ns.borrow().is_some())
}

/// Clear the global RDM namespace.
#[wasm_bindgen(js_name = clearRdmNamespace)]
pub fn clear_rdm_namespace() {
    GLOBAL_RDM_NAMESPACE.with(|ns| {
        *ns.borrow_mut() = None;
    });
    GLOBAL_RDM_NAMESPACE_RAW.with(|ns| {
        *ns.borrow_mut() = None;
    });
}

// =============================================================================
// UUID Generation
// =============================================================================

/// Generate a deterministic UUID for a collection from its name.
///
/// Requires the global namespace to be set via setRdmNamespace().
/// Uses: uuid5(namespace, "collection/" + name)
///
/// @param name - The collection name
/// @returns The collection UUID as a string
/// @throws Error if namespace is not set
#[wasm_bindgen(js_name = generateCollectionUuid)]
pub fn generate_collection_uuid(name: &str) -> Result<String, JsError> {
    let namespace = get_required_namespace()?;
    Ok(core_generate_collection(&namespace, name).to_string())
}

/// Generate a deterministic UUID for a concept from its collection and label.
///
/// Uses: uuid5(collection_id, label)
/// Does not require global namespace - uses collection_id as the namespace.
///
/// @param collectionId - The collection UUID
/// @param label - The concept label
/// @returns The concept UUID as a string
/// @throws Error if collectionId is not a valid UUID
#[wasm_bindgen(js_name = generateConceptUuid)]
pub fn generate_concept_uuid(collection_id: &str, label: &str) -> Result<String, JsError> {
    core_generate_concept(collection_id, label)
        .map(|u| u.to_string())
        .map_err(|e| JsError::new(&e))
}

/// Generate a deterministic UUID for a value (label in a specific language).
///
/// Uses: uuid5(uuid5(NAMESPACE_URL, "value"), "{concept_id}/prefLabel/{value}/{language}")
///
/// @param conceptId - The concept UUID
/// @param value - The label text
/// @param language - The language code (e.g., "en")
/// @returns The value UUID as a string
#[wasm_bindgen(js_name = generateValueUuid)]
pub fn generate_value_uuid(concept_id: &str, value: &str, language: &str) -> String {
    core_generate_value(concept_id, value, language).to_string()
}

/// Convert a multilingual label map to a deterministic string for UUID generation.
///
/// Sorts by language code and concatenates as "lang1:value1|lang2:value2".
/// This ensures deterministic UUID generation regardless of map ordering.
///
/// @param labels - Object mapping language code to label text
/// @returns A deterministic string representation
#[wasm_bindgen(js_name = labelsToDeterministicString)]
pub fn labels_to_deterministic_string(labels: JsValue) -> Result<String, JsError> {
    let labels_map: std::collections::HashMap<String, String> =
        serde_wasm_bindgen::from_value(labels)
            .map_err(|e| JsError::new(&format!("Invalid labels object: {}", e)))?;

    Ok(core_labels_to_string(&labels_map))
}

/// Parse a namespace string and return the resulting UUID.
///
/// Utility function for converting URLs to UUIDs without setting global state.
///
/// @param namespace - UUID string or URL (http/https)
/// @returns The namespace UUID as a string
/// @throws Error if parsing fails
#[wasm_bindgen(js_name = parseRdmNamespace)]
pub fn parse_rdm_namespace(namespace: &str) -> Result<String, JsError> {
    core_parse_namespace(namespace)
        .map(|u| u.to_string())
        .map_err(|e| JsError::new(&e))
}

// =============================================================================
// Internal Helpers
// =============================================================================

/// Get the global namespace or return an error if not set.
fn get_required_namespace() -> Result<Uuid, JsError> {
    GLOBAL_RDM_NAMESPACE.with(|ns| {
        ns.borrow().ok_or_else(|| {
            JsError::new(
                "RDM namespace not set. Call setRdmNamespace() before creating \
                 collections or concepts from labels without explicit IDs.",
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_namespace() {
        // Clear any existing state
        clear_rdm_namespace();
        assert!(!has_rdm_namespace());
        assert!(get_rdm_namespace().is_none());

        // Set from UUID
        set_rdm_namespace("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert!(has_rdm_namespace());
        assert_eq!(
            get_rdm_namespace(),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );

        // Clear
        clear_rdm_namespace();
        assert!(!has_rdm_namespace());
    }

    #[test]
    fn test_set_namespace_from_url() {
        clear_rdm_namespace();

        set_rdm_namespace("http://example.org/rdm/").unwrap();
        assert!(has_rdm_namespace());

        // URL should be converted to UUID
        let ns = get_rdm_namespace().unwrap();
        assert_eq!(ns.len(), 36); // UUID length
        assert_ne!(ns, "http://example.org/rdm/");
    }

    #[test]
    fn test_generate_collection_uuid() {
        clear_rdm_namespace();
        set_rdm_namespace("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let uuid1 = generate_collection_uuid("TestCollection").unwrap();
        let uuid2 = generate_collection_uuid("TestCollection").unwrap();
        assert_eq!(uuid1, uuid2); // Deterministic

        let uuid3 = generate_collection_uuid("OtherCollection").unwrap();
        assert_ne!(uuid1, uuid3); // Different names -> different UUIDs

        clear_rdm_namespace();
    }

    // Note: This test requires WASM target because JsError::new() panics on non-WASM
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_generate_collection_uuid_requires_namespace() {
        clear_rdm_namespace();

        let result = generate_collection_uuid("TestCollection");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_concept_uuid() {
        // Concept UUID doesn't require global namespace
        let uuid1 =
            generate_concept_uuid("550e8400-e29b-41d4-a716-446655440000", "ConceptA").unwrap();
        let uuid2 =
            generate_concept_uuid("550e8400-e29b-41d4-a716-446655440000", "ConceptA").unwrap();
        assert_eq!(uuid1, uuid2);

        let uuid3 =
            generate_concept_uuid("550e8400-e29b-41d4-a716-446655440000", "ConceptB").unwrap();
        assert_ne!(uuid1, uuid3);
    }

    #[test]
    fn test_generate_value_uuid() {
        let uuid1 = generate_value_uuid("concept-123", "Hello", "en");
        let uuid2 = generate_value_uuid("concept-123", "Hello", "en");
        assert_eq!(uuid1, uuid2);

        let uuid3 = generate_value_uuid("concept-123", "Hallo", "de");
        assert_ne!(uuid1, uuid3);
    }

    // Note: This test requires WASM target because JsError::new() panics on non-WASM
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_parse_rdm_namespace() {
        // UUID string
        let uuid = parse_rdm_namespace("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(uuid, "550e8400-e29b-41d4-a716-446655440000");

        // URL
        let uuid = parse_rdm_namespace("http://example.org/rdm/").unwrap();
        assert_eq!(uuid.len(), 36);

        // Invalid
        let result = parse_rdm_namespace("not-valid");
        assert!(result.is_err());
    }

    // Non-WASM version that tests the success cases only
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_parse_rdm_namespace_success_only() {
        // UUID string - use core function directly to avoid JsError
        let uuid = core_parse_namespace("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(uuid.to_string(), "550e8400-e29b-41d4-a716-446655440000");

        // URL
        let uuid = core_parse_namespace("http://example.org/rdm/").unwrap();
        assert_eq!(uuid.to_string().len(), 36);
    }
}
