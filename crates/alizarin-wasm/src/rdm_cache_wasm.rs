//! WASM RDM Cache for concept lookups
//!
//! This module wraps the platform-agnostic RdmCache from alizarin-core
//! with wasm-bindgen bindings for JavaScript interop.
//!
//! Usage from JavaScript:
//! ```javascript
//! const cache = new WasmRdmCache();
//! cache.addCollectionFromJson(collectionId, conceptsJson);
//! const label = cache.lookupLabel(collectionId, conceptId, "en");
//! ```

use wasm_bindgen::prelude::*;
use alizarin_core::rdm_cache::RdmCache;

// =============================================================================
// RDM Cache (WASM-exported)
// =============================================================================

/// Cache for RDM collections, enabling concept UUID -> label lookups
#[wasm_bindgen]
pub struct WasmRdmCache {
    inner: RdmCache,
}

#[wasm_bindgen]
impl WasmRdmCache {
    /// Create a new empty cache
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RdmCache::new(),
        }
    }

    /// Add a collection from JSON
    ///
    /// @param collectionId - The collection identifier
    /// @param conceptsJson - JSON array of concepts with {id, prefLabel: {lang: label}}
    #[wasm_bindgen(js_name = addCollectionFromJson)]
    pub fn add_collection_from_json(
        &mut self,
        collection_id: &str,
        concepts_json: &str,
    ) -> Result<(), JsError> {
        self.inner.add_collection_from_json(collection_id, concepts_json)
            .map_err(|e| JsError::new(&e))
    }

    /// Check if a collection is loaded
    #[wasm_bindgen(js_name = hasCollection)]
    pub fn has_collection(&self, collection_id: &str) -> bool {
        self.inner.has_collection(collection_id)
    }

    /// Get all loaded collection IDs
    #[wasm_bindgen(js_name = getCollectionIds)]
    pub fn get_collection_ids(&self) -> Vec<String> {
        self.inner.get_collection_ids()
    }

    /// Look up the label for a concept
    ///
    /// @param collectionId - The collection to search in
    /// @param conceptId - The concept UUID
    /// @param language - The language code (e.g., "en")
    /// @returns The label string, or undefined if not found
    #[wasm_bindgen(js_name = lookupLabel)]
    pub fn lookup_label(
        &self,
        collection_id: &str,
        concept_id: &str,
        language: &str,
    ) -> Option<String> {
        self.inner.lookup_label(collection_id, concept_id, language)
    }

    /// Look up full concept info
    ///
    /// @param collectionId - The collection to search in
    /// @param conceptId - The concept UUID
    /// @returns JSON object with concept info, or undefined if not found
    #[wasm_bindgen(js_name = lookupConcept)]
    pub fn lookup_concept(
        &self,
        collection_id: &str,
        concept_id: &str,
    ) -> Result<JsValue, JsError> {
        match self.inner.lookup_concept(collection_id, concept_id) {
            Some(concept) => {
                serde_wasm_bindgen::to_value(concept)
                    .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
            }
            None => Ok(JsValue::UNDEFINED),
        }
    }

    /// Clear all cached collections
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Remove a specific collection from the cache
    #[wasm_bindgen(js_name = removeCollection)]
    pub fn remove_collection(&mut self, collection_id: &str) -> bool {
        self.inner.remove_collection(collection_id)
    }

    /// Get number of cached collections
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.inner.len()
    }
}

impl Default for WasmRdmCache {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Internal API for use by instance_wrapper
// =============================================================================

impl WasmRdmCache {
    /// Get the inner RdmCache (for internal use)
    pub(crate) fn inner(&self) -> &RdmCache {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_label_lookup() {
        let mut cache = WasmRdmCache::new();

        let concepts_json = r#"[
            {
                "id": "concept-1",
                "prefLabel": {"en": "English Label", "de": "German Label"}
            },
            {
                "id": "concept-2",
                "prefLabel": {"en": "Second Concept"}
            }
        ]"#;

        cache.add_collection_from_json("collection-1", concepts_json).unwrap();

        assert!(cache.has_collection("collection-1"));
        assert!(!cache.has_collection("collection-2"));

        assert_eq!(
            cache.lookup_label("collection-1", "concept-1", "en"),
            Some("English Label".to_string())
        );
        assert_eq!(
            cache.lookup_label("collection-1", "concept-1", "de"),
            Some("German Label".to_string())
        );
        // Fallback to en
        assert_eq!(
            cache.lookup_label("collection-1", "concept-1", "fr"),
            Some("English Label".to_string())
        );
        // Not found
        assert_eq!(
            cache.lookup_label("collection-1", "concept-3", "en"),
            None
        );
    }

    #[test]
    fn test_clear_cache() {
        let mut cache = WasmRdmCache::new();

        cache.add_collection_from_json("coll-1", r#"[{"id": "c1", "prefLabel": {"en": "C1"}}]"#).unwrap();
        cache.add_collection_from_json("coll-2", r#"[{"id": "c2", "prefLabel": {"en": "C2"}}]"#).unwrap();

        assert_eq!(cache.get_collection_ids().len(), 2);

        cache.clear();
        assert_eq!(cache.get_collection_ids().len(), 0);
    }
}
