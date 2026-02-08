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

use alizarin_core::rdm_cache::RdmCache;
use serde::Serialize;
use wasm_bindgen::prelude::*;

// =============================================================================
// RDM Value Info (for JS return type)
// =============================================================================

/// Information about a label value, returned by lookupValue
///
/// This matches the StaticValue interface expected by JS ViewModels.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RdmValueInfo {
    /// Value ID (UUID) - unique identifier for this specific label
    pub id: String,
    /// The label text
    pub value: String,
    /// Concept ID that this value belongs to
    pub concept_id: String,
    /// Language code (e.g., "en", "de")
    pub language: String,
}

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
        self.inner
            .add_collection_from_json(collection_id, concepts_json)
            .map_err(|e| JsError::new(&e))
    }

    /// Check if a collection is loaded (returns false if collection_id is null)
    #[wasm_bindgen(js_name = hasCollection)]
    pub fn has_collection(&self, collection_id: Option<String>) -> bool {
        match collection_id {
            Some(id) => self.inner.has_collection(&id),
            None => false,
        }
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
    /// @returns The label string, or undefined if not found or any param is null
    #[wasm_bindgen(js_name = lookupLabel)]
    pub fn lookup_label(
        &self,
        collection_id: Option<String>,
        concept_id: Option<String>,
        language: Option<String>,
    ) -> Option<String> {
        let collection_id = collection_id?;
        let concept_id = concept_id?;
        let language = language?;
        self.inner
            .lookup_label(&collection_id, &concept_id, &language)
    }

    /// Look up full concept info
    ///
    /// @param collectionId - The collection to search in
    /// @param conceptId - The concept UUID
    /// @returns JSON object with concept info, or undefined if not found or any param is null
    #[wasm_bindgen(js_name = lookupConcept)]
    pub fn lookup_concept(
        &self,
        collection_id: Option<String>,
        concept_id: Option<String>,
    ) -> Result<JsValue, JsError> {
        let collection_id = match collection_id {
            Some(id) => id,
            None => return Ok(JsValue::UNDEFINED),
        };
        let concept_id = match concept_id {
            Some(id) => id,
            None => return Ok(JsValue::UNDEFINED),
        };
        match self.inner.lookup_concept(&collection_id, &concept_id) {
            Some(concept) => serde_wasm_bindgen::to_value(concept)
                .map_err(|e| JsError::new(&format!("Serialization error: {}", e))),
            None => Ok(JsValue::UNDEFINED),
        }
    }

    /// Get the first parent ID for a concept
    ///
    /// @param collectionId - The collection to search in
    /// @param conceptId - The concept UUID
    /// @returns The parent concept ID, or undefined if no parent (top-level concept) or any param is null
    #[wasm_bindgen(js_name = getParentId)]
    pub fn get_parent_id(
        &self,
        collection_id: Option<String>,
        concept_id: Option<String>,
    ) -> Option<String> {
        let collection_id = collection_id?;
        let concept_id = concept_id?;
        self.inner.get_parent_id(&collection_id, &concept_id)
    }

    // =========================================================================
    // Value ID Lookups (for StaticValue compatibility)
    // =========================================================================

    /// Look up a value by its VALUE ID
    ///
    /// This is the primary lookup method used by ViewModels.
    /// Returns full value info including concept ID and language.
    ///
    /// @param collectionId - The collection to search in
    /// @param valueId - The value UUID (not concept UUID)
    /// @returns RdmValueInfo object, or undefined if not found or any param is null
    #[wasm_bindgen(js_name = lookupValue)]
    pub fn lookup_value(
        &self,
        collection_id: Option<String>,
        value_id: Option<String>,
    ) -> Result<JsValue, JsError> {
        let collection_id = match collection_id {
            Some(id) => id,
            None => return Ok(JsValue::UNDEFINED),
        };
        let value_id = match value_id {
            Some(id) => id,
            None => return Ok(JsValue::UNDEFINED),
        };
        match self.inner.lookup_value(&collection_id, &value_id) {
            Some(value) => {
                let info = RdmValueInfo {
                    id: value.id.clone(),
                    value: value.value.clone(),
                    concept_id: value.concept_id.clone(),
                    language: value.language.clone(),
                };
                serde_wasm_bindgen::to_value(&info)
                    .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
            }
            None => Ok(JsValue::UNDEFINED),
        }
    }

    /// Get concept ID from value ID
    ///
    /// Returns the concept ID that contains the given value ID.
    ///
    /// @param collectionId - The collection to search in
    /// @param valueId - The value UUID
    /// @returns The concept ID, or undefined if not found or any param is null
    #[wasm_bindgen(js_name = getConceptIdForValue)]
    pub fn get_concept_id_for_value(
        &self,
        collection_id: Option<String>,
        value_id: Option<String>,
    ) -> Option<String> {
        let collection_id = collection_id?;
        let value_id = value_id?;
        self.inner
            .get_concept_id_for_value(&collection_id, &value_id)
            .map(|s| s.to_string())
    }

    /// Validate that a value exists in a collection
    ///
    /// @param collectionId - The collection to check
    /// @param valueId - The value UUID to validate
    /// @returns true if the value exists (returns false if any param is null)
    #[wasm_bindgen(js_name = validateValue)]
    pub fn validate_value(&self, collection_id: Option<String>, value_id: Option<String>) -> bool {
        match (collection_id, value_id) {
            (Some(cid), Some(vid)) => self.inner.validate_value(&cid, &vid),
            _ => false,
        }
    }

    // =========================================================================
    // Label Resolution (using cache as lookup)
    // =========================================================================

    /// Resolve labels to UUIDs using this cache for lookups.
    ///
    /// This is more efficient than `resolveLabelsWithLookup` because it uses
    /// the cache's internal indexes instead of requiring JS to build a lookup table.
    ///
    /// @param treeJson - JSON string of the tree to resolve
    /// @param aliasToCollection - Map<string, string> mapping node aliases to collection IDs
    /// @param strict - If true, return error for unresolved labels
    /// @returns JSON string with labels resolved to UUIDs
    #[wasm_bindgen(js_name = resolveLabels)]
    pub fn resolve_labels(
        &self,
        tree_json: &str,
        alias_to_collection: JsValue,
        strict: bool,
    ) -> Result<String, JsError> {
        use alizarin_core::label_resolution;
        use std::collections::HashMap;

        let tree: serde_json::Value = serde_json::from_str(tree_json)
            .map_err(|e| JsError::new(&format!("Invalid tree JSON: {}", e)))?;

        let alias_map: HashMap<String, String> =
            serde_wasm_bindgen::from_value(alias_to_collection)
                .map_err(|e| JsError::new(&format!("Invalid alias map: {}", e)))?;

        let resolved = label_resolution::resolve_labels(tree, &alias_map, &self.inner, strict)
            .map_err(|e| JsError::new(&e.message))?;

        serde_json::to_string(&resolved)
            .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
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

        cache
            .add_collection_from_json("collection-1", concepts_json)
            .unwrap();

        assert!(cache.has_collection(Some("collection-1".to_string())));
        assert!(!cache.has_collection(Some("collection-2".to_string())));

        assert_eq!(
            cache.lookup_label(
                Some("collection-1".to_string()),
                Some("concept-1".to_string()),
                Some("en".to_string())
            ),
            Some("English Label".to_string())
        );
        assert_eq!(
            cache.lookup_label(
                Some("collection-1".to_string()),
                Some("concept-1".to_string()),
                Some("de".to_string())
            ),
            Some("German Label".to_string())
        );
        // Fallback to en
        assert_eq!(
            cache.lookup_label(
                Some("collection-1".to_string()),
                Some("concept-1".to_string()),
                Some("fr".to_string())
            ),
            Some("English Label".to_string())
        );
        // Not found
        assert_eq!(
            cache.lookup_label(
                Some("collection-1".to_string()),
                Some("concept-3".to_string()),
                Some("en".to_string())
            ),
            None
        );
    }

    #[test]
    fn test_clear_cache() {
        let mut cache = WasmRdmCache::new();

        cache
            .add_collection_from_json("coll-1", r#"[{"id": "c1", "prefLabel": {"en": "C1"}}]"#)
            .unwrap();
        cache
            .add_collection_from_json("coll-2", r#"[{"id": "c2", "prefLabel": {"en": "C2"}}]"#)
            .unwrap();

        assert_eq!(cache.get_collection_ids().len(), 2);

        cache.clear();
        assert_eq!(cache.get_collection_ids().len(), 0);
    }

    #[test]
    fn test_value_id_methods() {
        let mut cache = WasmRdmCache::new();

        // JSON with explicit value IDs
        let concepts_json = r#"[
            {
                "id": "concept-1",
                "prefLabels": {
                    "en": { "id": "value-1-en", "value": "English Label" }
                }
            }
        ]"#;

        cache
            .add_collection_from_json("coll-1", concepts_json)
            .unwrap();

        // Test get_concept_id_for_value
        assert_eq!(
            cache.get_concept_id_for_value(
                Some("coll-1".to_string()),
                Some("value-1-en".to_string())
            ),
            Some("concept-1".to_string())
        );
        assert_eq!(
            cache.get_concept_id_for_value(
                Some("coll-1".to_string()),
                Some("nonexistent".to_string())
            ),
            None
        );

        // Test validate_value
        assert!(cache.validate_value(Some("coll-1".to_string()), Some("value-1-en".to_string())));
        assert!(!cache.validate_value(Some("coll-1".to_string()), Some("nonexistent".to_string())));

        // Note: lookup_value returns JsValue which can't be easily tested
        // outside WASM context. The underlying logic is tested in alizarin-core.
    }

    #[test]
    fn test_get_parent_id() {
        let mut cache = WasmRdmCache::new();

        let concepts_json = r#"[
            {
                "id": "parent-concept",
                "prefLabel": {"en": "Parent"}
            },
            {
                "id": "child-concept",
                "prefLabel": {"en": "Child"},
                "broader": ["parent-concept"]
            }
        ]"#;

        cache
            .add_collection_from_json("coll-1", concepts_json)
            .unwrap();

        // Child should have parent
        assert_eq!(
            cache.get_parent_id(
                Some("coll-1".to_string()),
                Some("child-concept".to_string())
            ),
            Some("parent-concept".to_string())
        );

        // Parent has no parent (top-level)
        assert_eq!(
            cache.get_parent_id(
                Some("coll-1".to_string()),
                Some("parent-concept".to_string())
            ),
            None
        );
    }
}
