//! Core RDM Cache for concept lookups
//!
//! This module provides a platform-agnostic cache for Reference Data Manager (RDM) collections,
//! enabling UUID -> label lookups for concept and concept-list datatypes.
//!
//! The WASM bindings (alizarin-wasm) wrap this with WasmRdmCache for JavaScript interop.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// =============================================================================
// RDM Concept
// =============================================================================

/// A concept from an RDM collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdmConcept {
    /// Concept ID (UUID)
    pub id: String,
    /// Preferred labels by language code
    #[serde(default, rename = "prefLabel")]
    pub pref_label: HashMap<String, String>,
    /// Alternative labels by language code
    #[serde(default, rename = "altLabels")]
    pub alt_labels: HashMap<String, Vec<String>>,
    /// Broader concepts (parent IDs)
    #[serde(default)]
    pub broader: Vec<String>,
    /// Narrower concepts (child IDs)
    #[serde(default)]
    pub narrower: Vec<String>,
    /// Scope notes by language
    #[serde(default, rename = "scopeNote")]
    pub scope_note: HashMap<String, String>,
}

impl RdmConcept {
    /// Get the preferred label for a language, with fallbacks
    pub fn get_label(&self, language: &str) -> Option<String> {
        self.pref_label.get(language)
            .or_else(|| self.pref_label.get("en"))
            .or_else(|| self.pref_label.values().next())
            .cloned()
    }
}

// =============================================================================
// RDM Collection
// =============================================================================

/// A collection of RDM concepts
#[derive(Debug, Clone, Default)]
pub struct RdmCollection {
    /// Collection ID
    pub id: String,
    /// Collection name (optional, for display)
    pub name: Option<String>,
    /// Concepts indexed by their ID
    concepts: HashMap<String, RdmConcept>,
    /// Top-level concepts (no broader)
    top_concepts: Vec<String>,
}

impl RdmCollection {
    pub fn new(id: String) -> Self {
        Self {
            id,
            name: None,
            concepts: HashMap::new(),
            top_concepts: vec![],
        }
    }

    /// Create a new collection with a name
    pub fn with_name(id: String, name: String) -> Self {
        Self {
            id,
            name: Some(name),
            concepts: HashMap::new(),
            top_concepts: vec![],
        }
    }

    /// Add a concept to the collection
    pub fn add_concept(&mut self, concept: RdmConcept) {
        let id = concept.id.clone();
        if concept.broader.is_empty() {
            self.top_concepts.push(id.clone());
        }
        self.concepts.insert(id, concept);
    }

    /// Get top-level concepts (no broader)
    pub fn get_top_concepts(&self) -> Vec<&RdmConcept> {
        self.top_concepts.iter()
            .filter_map(|id| self.concepts.get(id))
            .collect()
    }

    /// Get a concept by ID
    pub fn get_concept(&self, concept_id: &str) -> Option<&RdmConcept> {
        self.concepts.get(concept_id)
    }

    /// Get the label for a concept in this collection
    pub fn get_label(&self, concept_id: &str, language: &str) -> Option<String> {
        self.get_concept(concept_id)
            .and_then(|c| c.get_label(language))
    }

    /// Parse collection from JSON array of concepts
    pub fn from_concepts_json(id: String, json: &str) -> Result<Self, String> {
        let concepts: Vec<RdmConcept> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse concepts JSON: {}", e))?;

        let mut collection = Self::new(id);
        for concept in concepts {
            collection.add_concept(concept);
        }
        Ok(collection)
    }

    /// Get the number of concepts in this collection
    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    /// Check if a concept exists in the collection
    pub fn has_concept(&self, concept_id: &str) -> bool {
        self.concepts.contains_key(concept_id)
    }

    /// Get all concept IDs
    pub fn get_concept_ids(&self) -> Vec<&String> {
        self.concepts.keys().collect()
    }

    /// Find a concept by exact label match (case-insensitive)
    ///
    /// Searches pref_label and alt_labels across all languages.
    /// Returns None if no match or multiple matches (ambiguous).
    pub fn find_by_label(&self, label: &str) -> Option<&RdmConcept> {
        let label_lower = label.to_lowercase();
        let matches: Vec<_> = self.concepts.values()
            .filter(|c| {
                // Check pref_label in any language
                c.pref_label.values().any(|p| p.to_lowercase() == label_lower) ||
                // Check alt_labels in any language
                c.alt_labels.values().any(|alts|
                    alts.iter().any(|l| l.to_lowercase() == label_lower)
                )
            })
            .collect();

        // Only return if exactly one match (unambiguous)
        if matches.len() == 1 {
            matches.into_iter().next()
        } else {
            None
        }
    }

    /// Find all concepts by exact label match (case-insensitive)
    pub fn find_all_by_label(&self, label: &str) -> Vec<&RdmConcept> {
        let label_lower = label.to_lowercase();
        self.concepts.values()
            .filter(|c| {
                c.pref_label.values().any(|p| p.to_lowercase() == label_lower) ||
                c.alt_labels.values().any(|alts|
                    alts.iter().any(|l| l.to_lowercase() == label_lower)
                )
            })
            .collect()
    }

    /// Search concepts by label prefix (case-insensitive)
    pub fn search(&self, query: &str, language: Option<&str>) -> Vec<&RdmConcept> {
        let lang = language.unwrap_or("en");
        let query_lower = query.to_lowercase();

        self.concepts.values()
            .filter(|c| {
                // Check pref_label
                if let Some(label) = c.pref_label.get(lang) {
                    if label.to_lowercase().starts_with(&query_lower) {
                        return true;
                    }
                }
                // Check alt_labels
                if let Some(alts) = c.alt_labels.get(lang) {
                    if alts.iter().any(|l| l.to_lowercase().starts_with(&query_lower)) {
                        return true;
                    }
                }
                false
            })
            .collect()
    }
}

// =============================================================================
// RDM Cache
// =============================================================================

/// Cache for RDM collections, enabling concept UUID -> label lookups
#[derive(Debug, Clone, Default)]
pub struct RdmCache {
    collections: HashMap<String, RdmCollection>,
}

impl RdmCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }

    /// Add a collection from JSON
    ///
    /// @param collection_id - The collection identifier
    /// @param concepts_json - JSON array of concepts with {id, prefLabel: {lang: label}}
    pub fn add_collection_from_json(
        &mut self,
        collection_id: &str,
        concepts_json: &str,
    ) -> Result<(), String> {
        let collection = RdmCollection::from_concepts_json(
            collection_id.to_string(),
            concepts_json,
        )?;

        self.collections.insert(collection_id.to_string(), collection);
        Ok(())
    }

    /// Add a collection directly
    pub fn add_collection(&mut self, collection: RdmCollection) {
        self.collections.insert(collection.id.clone(), collection);
    }

    /// Check if a collection is loaded
    pub fn has_collection(&self, collection_id: &str) -> bool {
        self.collections.contains_key(collection_id)
    }

    /// Get all loaded collection IDs
    pub fn get_collection_ids(&self) -> Vec<String> {
        self.collections.keys().cloned().collect()
    }

    /// Look up the label for a concept
    ///
    /// @param collection_id - The collection to search in
    /// @param concept_id - The concept UUID
    /// @param language - The language code (e.g., "en")
    /// @returns The label string, or None if not found
    pub fn lookup_label(
        &self,
        collection_id: &str,
        concept_id: &str,
        language: &str,
    ) -> Option<String> {
        self.collections.get(collection_id)
            .and_then(|c| c.get_label(concept_id, language))
    }

    /// Look up full concept info
    pub fn lookup_concept(
        &self,
        collection_id: &str,
        concept_id: &str,
    ) -> Option<&RdmConcept> {
        self.collections.get(collection_id)
            .and_then(|c| c.get_concept(concept_id))
    }

    /// Get a collection by ID
    pub fn get_collection(&self, collection_id: &str) -> Option<&RdmCollection> {
        self.collections.get(collection_id)
    }

    /// Clear all cached collections
    pub fn clear(&mut self) {
        self.collections.clear();
    }

    /// Remove a specific collection from the cache
    pub fn remove_collection(&mut self, collection_id: &str) -> bool {
        self.collections.remove(collection_id).is_some()
    }

    /// Get the number of cached collections
    pub fn len(&self) -> usize {
        self.collections.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.collections.is_empty()
    }

    /// Validate that a concept exists in a collection
    pub fn validate_concept(&self, collection_id: &str, concept_id: &str) -> bool {
        self.collections.get(collection_id)
            .map(|c| c.has_concept(concept_id))
            .unwrap_or(false)
    }

    /// Look up a concept by label in a specific collection
    ///
    /// Returns the concept if exactly one match is found.
    /// Returns None if no match or ambiguous (multiple matches).
    pub fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<&RdmConcept> {
        self.collections.get(collection_id)
            .and_then(|c| c.find_by_label(label))
    }

    /// Look up a concept by label, returning all matches
    pub fn lookup_all_by_label(&self, collection_id: &str, label: &str) -> Vec<&RdmConcept> {
        self.collections.get(collection_id)
            .map(|c| c.find_all_by_label(label))
            .unwrap_or_default()
    }

    /// Search across all collections (for autocomplete)
    pub fn search_all(&self, query: &str, language: Option<&str>) -> Vec<(&str, &RdmConcept)> {
        self.collections.iter()
            .flat_map(|(coll_id, collection)| {
                collection.search(query, language)
                    .into_iter()
                    .map(move |c| (coll_id.as_str(), c))
            })
            .collect()
    }
}

// =============================================================================
// ConceptLookup Implementation
// =============================================================================

use crate::label_resolution::ConceptLookup;

impl ConceptLookup for RdmCache {
    fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<String> {
        self.collections.get(collection_id)
            .and_then(|c| c.find_by_label(label))
            .map(|c| c.id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_label_lookup() {
        let mut cache = RdmCache::new();

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
        let mut cache = RdmCache::new();

        cache.add_collection_from_json("coll-1", r#"[{"id": "c1", "prefLabel": {"en": "C1"}}]"#).unwrap();
        cache.add_collection_from_json("coll-2", r#"[{"id": "c2", "prefLabel": {"en": "C2"}}]"#).unwrap();

        assert_eq!(cache.get_collection_ids().len(), 2);

        cache.clear();
        assert_eq!(cache.get_collection_ids().len(), 0);
    }
}
