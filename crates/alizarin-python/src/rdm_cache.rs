/// RDM (Reference Data Manager) Cache for concept collections.
///
/// This module provides:
/// - RdmConcept: A concept value from a collection
/// - RdmCollection: A collection of concepts
/// - RdmCache: Cache for storing fetched collections
///
/// Pattern: Python fetches collections asynchronously, Rust looks them up synchronously.

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Concept Types
// =============================================================================

/// A concept value from an RDM collection.
///
/// Matches the structure returned by RDM APIs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[pyclass(name = "RustRdmConcept")]
pub struct RdmConcept {
    /// Concept ID (URI or UUID)
    #[serde(default)]
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

#[pymethods]
impl RdmConcept {
    #[new]
    fn new(id: String, pref_label: HashMap<String, String>) -> Self {
        Self {
            id,
            pref_label,
            alt_labels: HashMap::new(),
            broader: vec![],
            narrower: vec![],
            scope_note: HashMap::new(),
        }
    }

    #[getter]
    fn id(&self) -> &str {
        &self.id
    }

    #[getter]
    fn pref_label(&self) -> HashMap<String, String> {
        self.pref_label.clone()
    }

    #[getter]
    fn alt_labels(&self) -> HashMap<String, Vec<String>> {
        self.alt_labels.clone()
    }

    #[getter]
    fn broader(&self) -> Vec<String> {
        self.broader.clone()
    }

    #[getter]
    fn narrower(&self) -> Vec<String> {
        self.narrower.clone()
    }

    /// Get preferred label for a language
    fn get_label(&self, language: &str) -> Option<String> {
        self.pref_label.get(language)
            .or_else(|| self.pref_label.get("en"))
            .or_else(|| self.pref_label.values().next())
            .cloned()
    }

    fn __str__(&self) -> String {
        self.get_label("en").unwrap_or_default()
    }

    fn __repr__(&self) -> String {
        format!("RustRdmConcept(id={}, labels={:?})", self.id, self.pref_label)
    }

    /// Convert to Python dict
    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let json_str = serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let json_module = py.import("json")?;
        let py_dict = json_module.call_method1("loads", (json_str,))?;
        Ok(py_dict.to_object(py))
    }
}

// =============================================================================
// Collection Types
// =============================================================================

/// A collection of concepts from RDM.
#[derive(Clone, Debug, Default)]
#[pyclass(name = "RustRdmCollection")]
pub struct RdmCollection {
    /// Collection ID (URI or UUID)
    pub id: String,
    /// Concepts indexed by their ID
    concepts: HashMap<String, RdmConcept>,
    /// Top-level concepts (no broader)
    top_concepts: Vec<String>,
}

#[pymethods]
impl RdmCollection {
    #[new]
    fn new(id: String) -> Self {
        Self {
            id,
            concepts: HashMap::new(),
            top_concepts: vec![],
        }
    }

    #[getter]
    fn id(&self) -> &str {
        &self.id
    }

    /// Add a concept to the collection
    fn add_concept(&mut self, concept: RdmConcept) {
        let id = concept.id.clone();
        if concept.broader.is_empty() {
            self.top_concepts.push(id.clone());
        }
        self.concepts.insert(id, concept);
    }

    /// Look up a concept by ID
    fn get_concept(&self, concept_id: &str) -> Option<RdmConcept> {
        self.concepts.get(concept_id).cloned()
    }

    /// Check if concept exists in collection
    fn has_concept(&self, concept_id: &str) -> bool {
        self.concepts.contains_key(concept_id)
    }

    /// Get all concept IDs
    fn get_concept_ids(&self) -> Vec<String> {
        self.concepts.keys().cloned().collect()
    }

    /// Get top-level concepts
    fn get_top_concepts(&self) -> Vec<RdmConcept> {
        self.top_concepts.iter()
            .filter_map(|id| self.concepts.get(id).cloned())
            .collect()
    }

    /// Get number of concepts
    fn __len__(&self) -> usize {
        self.concepts.len()
    }

    /// Search concepts by label (case-insensitive prefix match)
    fn search(&self, query: &str, language: Option<&str>) -> Vec<RdmConcept> {
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
            .cloned()
            .collect()
    }
}

impl RdmCollection {
    /// Build from JSON array of concepts (internal method)
    pub fn from_concepts_json(id: String, json_str: &str) -> Result<Self, String> {
        let concepts: Vec<RdmConcept> = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse concepts: {}", e))?;

        let mut collection = Self::new(id);
        for concept in concepts {
            collection.add_concept(concept);
        }
        Ok(collection)
    }
}

// =============================================================================
// RDM Cache
// =============================================================================

/// Cache for RDM concept collections.
///
/// Python fetches collections asynchronously and adds them to the cache.
/// Rust looks up concepts synchronously during coercion.
#[pyclass(name = "RustRdmCache")]
pub struct RdmCache {
    /// Collections indexed by collection ID
    collections: HashMap<String, RdmCollection>,
}

#[pymethods]
impl RdmCache {
    #[new]
    fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }

    /// Add a collection from JSON
    ///
    /// Args:
    ///     collection_id: The collection identifier
    ///     concepts_json: JSON array of concepts
    fn add_collection_from_json(&mut self, collection_id: &str, concepts_json: &str) -> PyResult<()> {
        let collection = RdmCollection::from_concepts_json(collection_id.to_string(), concepts_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

        self.collections.insert(collection_id.to_string(), collection);
        Ok(())
    }

    /// Add a pre-built collection
    fn add_collection(&mut self, collection: RdmCollection) {
        self.collections.insert(collection.id.clone(), collection);
    }

    /// Get a collection by ID
    fn get_collection(&self, collection_id: &str) -> Option<RdmCollection> {
        self.collections.get(collection_id).cloned()
    }

    /// Check if collection is cached
    fn has_collection(&self, collection_id: &str) -> bool {
        self.collections.contains_key(collection_id)
    }

    /// Look up a concept directly
    ///
    /// This is the main lookup method for coercion:
    /// given a collection ID and concept ID, return the concept.
    fn lookup_concept(&self, collection_id: &str, concept_id: &str) -> Option<RdmConcept> {
        self.collections.get(collection_id)
            .and_then(|c| c.get_concept(concept_id))
    }

    /// Validate that a concept exists in a collection
    fn validate_concept(&self, collection_id: &str, concept_id: &str) -> bool {
        self.collections.get(collection_id)
            .map(|c| c.has_concept(concept_id))
            .unwrap_or(false)
    }

    /// Get all cached collection IDs
    fn get_collection_ids(&self) -> Vec<String> {
        self.collections.keys().cloned().collect()
    }

    /// Clear all cached collections
    fn clear(&mut self) {
        self.collections.clear();
    }

    /// Clear a specific collection
    fn clear_collection(&mut self, collection_id: &str) {
        self.collections.remove(collection_id);
    }

    /// Get number of cached collections
    fn __len__(&self) -> usize {
        self.collections.len()
    }

    /// Search across all collections (for autocomplete)
    fn search_all(&self, query: &str, language: Option<&str>) -> Vec<(String, RdmConcept)> {
        let lang = language.unwrap_or("en");

        self.collections.iter()
            .flat_map(|(coll_id, collection)| {
                collection.search(query, Some(lang))
                    .into_iter()
                    .map(|c| (coll_id.clone(), c))
            })
            .collect()
    }
}

impl RdmCache {
    /// Get reference for Rust-side lookup
    pub fn get(&self, collection_id: &str) -> Option<&RdmCollection> {
        self.collections.get(collection_id)
    }
}

// =============================================================================
// Module Registration
// =============================================================================

/// Register RDM cache types with the Python module
pub fn register_module(m: &PyModule) -> PyResult<()> {
    m.add_class::<RdmConcept>()?;
    m.add_class::<RdmCollection>()?;
    m.add_class::<RdmCache>()?;
    Ok(())
}
