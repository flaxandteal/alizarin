/// RDM (Reference Data Manager) Cache for concept collections.
///
/// This module provides:
/// - RdmConcept: A concept value from a collection
/// - RdmCollection: A collection of concepts
/// - RdmCache: Cache for storing fetched collections
///
/// Pattern: Python fetches collections asynchronously, Rust looks them up synchronously.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

// SKOS parser from core crate
use alizarin_core::skos::{parse_skos_to_collections, SkosCollection};

// For get_current_language
use alizarin_core::type_coercion::get_current_language;

// Label resolution from core
use alizarin_core::label_resolution::{
    self, build_alias_to_collection_map, find_needed_collections,
    ConceptLookup, LabelResolutionConfig,
};

// =============================================================================
// Global RDM Cache Singleton
// =============================================================================

lazy_static::lazy_static! {
    /// Global RDM cache singleton for automatic label resolution
    static ref GLOBAL_RDM_CACHE: RwLock<Option<RdmCache>> = RwLock::new(None);
}

/// Set the global RDM cache for automatic label resolution.
///
/// Once set, functions like `json_tree_to_tiles` will automatically
/// resolve label strings to UUIDs using this cache.
///
/// Example:
///     cache = RustRdmCache()
///     cache.add_collection(my_collection)
///     set_global_rdm_cache(cache)
///
///     # Now json_tree_to_tiles will auto-resolve labels
///     result = json_tree_to_tiles(tree_json, ...)
#[pyfunction]
pub fn set_global_rdm_cache(cache: RdmCache) {
    if let Ok(mut guard) = GLOBAL_RDM_CACHE.write() {
        *guard = Some(cache);
    }
}

/// Get a clone of the global RDM cache, if set.
#[pyfunction]
pub fn get_global_rdm_cache() -> Option<RdmCache> {
    GLOBAL_RDM_CACHE
        .read()
        .ok()
        .and_then(|guard| guard.clone())
}

/// Clear the global RDM cache.
#[pyfunction]
pub fn clear_global_rdm_cache() {
    if let Ok(mut guard) = GLOBAL_RDM_CACHE.write() {
        *guard = None;
    }
}

/// Check if a global RDM cache is set.
#[pyfunction]
pub fn has_global_rdm_cache() -> bool {
    GLOBAL_RDM_CACHE
        .read()
        .ok()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

/// Internal: Get reference to global cache for Rust-side operations
pub fn with_global_rdm_cache<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&RdmCache) -> R,
{
    GLOBAL_RDM_CACHE
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().map(f))
}

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

    /// Create a concept from an ID and a label.
    ///
    /// The label can be either:
    /// - A string: will be stored under the current language (from get_current_language)
    /// - A dict {lang: label}: will be stored as pref_labels for each language
    ///
    /// For auto-generating IDs, use `RustRdmCollection.add_from_label()` instead.
    ///
    /// Example:
    ///     concept = RustRdmConcept.from_label("uuid-1", "Category A")
    ///     concept = RustRdmConcept.from_label("uuid-1", {"en": "Category A", "de": "Kategorie A"})
    #[staticmethod]
    fn from_label(py: Python, id: String, label: PyObject) -> PyResult<Self> {
        let pref_label = if let Ok(s) = label.extract::<String>(py) {
            let lang = get_current_language();
            let mut map = HashMap::new();
            map.insert(lang, s);
            map
        } else if let Ok(dict) = label.downcast::<PyDict>(py) {
            let mut map = HashMap::new();
            for (key, value) in dict.iter() {
                let lang: String = key.extract()?;
                let label_str: String = value.extract()?;
                map.insert(lang, label_str);
            }
            map
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "label must be a string or dict of {language: label}"
            ));
        };

        Ok(Self {
            id,
            pref_label,
            alt_labels: HashMap::new(),
            broader: vec![],
            narrower: vec![],
            scope_note: HashMap::new(),
        })
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
    /// Collection name (optional, for display)
    name: Option<String>,
    /// Concepts indexed by their ID
    concepts: HashMap<String, RdmConcept>,
    /// Top-level concepts (no broader)
    top_concepts: Vec<String>,
}

/// Namespace UUID for generating collection IDs from names
/// Derived from the project's base namespace: uuid5("1a79f1c8-9505-4bea-a18e-28a053f725ca", "collection")
const COLLECTION_NAMESPACE: &str = "a8e5f3b2-7c41-5d8a-9f12-3b4c5d6e7f8a";

#[pymethods]
impl RdmCollection {
    #[new]
    fn new(id: String) -> Self {
        Self {
            id,
            name: None,
            concepts: HashMap::new(),
            top_concepts: vec![],
        }
    }

    /// Create a collection from a name and list of labels.
    ///
    /// Args:
    ///     name: The collection name (also used to generate ID if not provided)
    ///     labels: List of concept labels (strings only - IDs are auto-generated)
    ///     id: Optional explicit collection ID. If not provided, generates uuid5 from name.
    ///
    /// Example:
    ///     # Auto-generate collection ID from name, concept IDs from labels
    ///     collection = RustRdmCollection.from_labels("Categories", ["Cat A", "Cat B", "Cat C"])
    ///
    ///     # Explicit collection ID
    ///     collection = RustRdmCollection.from_labels("Categories", ["Cat A", "Cat B"], id="my-uuid")
    #[staticmethod]
    #[pyo3(signature = (name, labels, id=None))]
    fn from_labels(py: Python, name: String, labels: Vec<PyObject>, id: Option<String>) -> PyResult<Self> {
        // Generate or use provided collection ID
        let collection_id = match id {
            Some(explicit_id) => explicit_id,
            None => {
                let namespace = Uuid::parse_str(COLLECTION_NAMESPACE).unwrap();
                Uuid::new_v5(&namespace, name.as_bytes()).to_string()
            }
        };

        let mut collection = Self {
            id: collection_id,
            name: Some(name),
            concepts: HashMap::new(),
            top_concepts: vec![],
        };

        // Add each label as a concept
        for label in labels {
            collection.add_from_label(py, label, None)?;
        }

        Ok(collection)
    }

    #[getter]
    fn id(&self) -> &str {
        &self.id
    }

    #[getter]
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Add a concept to the collection
    fn add_concept(&mut self, concept: RdmConcept) {
        let id = concept.id.clone();
        if concept.broader.is_empty() {
            self.top_concepts.push(id.clone());
        }
        self.concepts.insert(id, concept);
    }

    /// Add a concept from a label, optionally auto-generating the ID.
    ///
    /// The label can be either:
    /// - A string: stored under the current language, ID generated from uuid5(collection_id, label)
    /// - A dict {lang: label}: stored as pref_labels for each language (requires explicit id)
    ///
    /// Returns the generated/provided concept ID.
    ///
    /// Example:
    ///     collection = RustRdmCollection("550e8400-e29b-41d4-a716-446655440000")
    ///
    ///     # Auto-generate ID from label
    ///     id1 = collection.add_from_label("Category A")
    ///
    ///     # Explicit ID
    ///     id2 = collection.add_from_label("Category B", id="my-uuid")
    ///
    ///     # Multi-language requires explicit ID
    ///     id3 = collection.add_from_label({"en": "Category C", "de": "Kategorie C"}, id="my-uuid-2")
    #[pyo3(signature = (label, id=None))]
    fn add_from_label(&mut self, py: Python, label: PyObject, id: Option<String>) -> PyResult<String> {
        // Extract label and determine if it's a string or dict
        let (pref_label, label_string) = if let Ok(s) = label.extract::<String>(py) {
            let lang = get_current_language();
            let mut map = HashMap::new();
            map.insert(lang, s.clone());
            (map, Some(s))
        } else if let Ok(dict) = label.downcast::<PyDict>(py) {
            let mut map = HashMap::new();
            for (key, value) in dict.iter() {
                let lang: String = key.extract()?;
                let label_str: String = value.extract()?;
                map.insert(lang, label_str);
            }
            (map, None)
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "label must be a string or dict of {language: label}"
            ));
        };

        // Determine the concept ID
        let concept_id = match (id, label_string) {
            (Some(explicit_id), _) => explicit_id,
            (None, Some(label_str)) => {
                // Parse collection ID as UUID namespace
                let namespace = Uuid::parse_str(&self.id).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        format!("collection id must be a valid UUID for auto-generation: {}", e)
                    )
                })?;
                Uuid::new_v5(&namespace, label_str.as_bytes()).to_string()
            }
            (None, None) => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "id is required when label is a dict (cannot auto-generate from multiple languages)"
                ));
            }
        };

        let concept = RdmConcept {
            id: concept_id.clone(),
            pref_label,
            alt_labels: HashMap::new(),
            broader: vec![],
            narrower: vec![],
            scope_note: HashMap::new(),
        };

        self.add_concept(concept);
        Ok(concept_id)
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

    /// Find a concept by exact label match (case-insensitive)
    ///
    /// Searches pref_label and alt_labels across all languages.
    /// Returns None if no match or multiple matches (ambiguous).
    fn find_by_label(&self, label: &str) -> Option<RdmConcept> {
        let label_lower = label.to_lowercase();
        let matches: Vec<_> = self.concepts.values()
            .filter(|c| {
                // Check pref_label in any language
                for pref in c.pref_label.values() {
                    if pref.to_lowercase() == label_lower {
                        return true;
                    }
                }
                // Check alt_labels in any language
                for alts in c.alt_labels.values() {
                    if alts.iter().any(|l| l.to_lowercase() == label_lower) {
                        return true;
                    }
                }
                false
            })
            .cloned()
            .collect();

        // Only return if exactly one match (unambiguous)
        if matches.len() == 1 {
            Some(matches.into_iter().next().unwrap())
        } else {
            None
        }
    }

    /// Find a concept by exact label match, returning all matches
    fn find_all_by_label(&self, label: &str) -> Vec<RdmConcept> {
        let label_lower = label.to_lowercase();
        self.concepts.values()
            .filter(|c| {
                for pref in c.pref_label.values() {
                    if pref.to_lowercase() == label_lower {
                        return true;
                    }
                }
                for alts in c.alt_labels.values() {
                    if alts.iter().any(|l| l.to_lowercase() == label_lower) {
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
///
/// Supports lazy loading via an async loader callback:
///     async def my_loader(collection_id: str) -> RustRdmCollection | None:
///         return await fetch_from_api(collection_id)
///
///     cache = RustRdmCache(loader=my_loader)
///     await cache.ensure_collection("collection-id")  # Loads if not cached
#[derive(Clone, Default)]
#[pyclass(name = "RustRdmCache")]
pub struct RdmCache {
    /// Collections indexed by collection ID
    collections: HashMap<String, RdmCollection>,
    /// Optional async loader for lazy loading (stored as Python object)
    #[pyo3(get, set)]
    loader: Option<PyObject>,
}

#[pymethods]
impl RdmCache {
    #[new]
    #[pyo3(signature = (loader=None))]
    fn new(loader: Option<PyObject>) -> Self {
        Self {
            collections: HashMap::new(),
            loader,
        }
    }

    /// Check if collection needs loading and return it for async loading if so.
    ///
    /// This is designed to be used with Python async/await:
    ///
    ///     if coro := cache.fetch_if_missing(collection_id):
    ///         collection = await coro
    ///         if collection:
    ///             cache.add_collection(collection)
    ///
    /// Returns None if already cached or no loader set.
    /// Returns the loader coroutine if loading is needed.
    fn fetch_if_missing(&self, py: Python, collection_id: &str) -> PyResult<Option<PyObject>> {
        // Already cached?
        if self.collections.contains_key(collection_id) {
            return Ok(None);
        }

        // No loader?
        let loader = match &self.loader {
            Some(l) => l,
            None => return Ok(None),
        };

        // Call loader with collection_id - returns a coroutine
        let coro = loader.call1(py, (collection_id,))?;
        Ok(Some(coro))
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

    /// Add collection(s) from SKOS RDF/XML
    ///
    /// Parses SKOS XML and adds the concept schemes as collections.
    ///
    /// Args:
    ///     xml_content: SKOS RDF/XML as string
    ///     base_uri: Base URI for resolving relative URIs
    ///
    /// Returns:
    ///     List of collection IDs that were added
    fn add_from_skos_xml(&mut self, xml_content: &str, base_uri: &str) -> PyResult<Vec<String>> {
        let skos_collections = parse_skos_to_collections(xml_content, base_uri)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to parse SKOS XML: {}", e)
            ))?;

        let mut added_ids = Vec::new();

        for skos_coll in skos_collections {
            let rdm_collection = Self::skos_to_rdm_collection(&skos_coll);
            let id = rdm_collection.id.clone();
            self.collections.insert(id.clone(), rdm_collection);
            added_ids.push(id);
        }

        Ok(added_ids)
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

    /// Look up a concept by label in a specific collection
    ///
    /// Returns the concept if exactly one match is found.
    /// Returns None if no match or ambiguous (multiple matches).
    fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<RdmConcept> {
        self.collections.get(collection_id)
            .and_then(|c| c.find_by_label(label))
    }

    /// Look up a concept by label, returning all matches
    fn lookup_all_by_label(&self, collection_id: &str, label: &str) -> Vec<RdmConcept> {
        self.collections.get(collection_id)
            .map(|c| c.find_all_by_label(label))
            .unwrap_or_default()
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

    /// Convert SKOS collection to RDM collection
    fn skos_to_rdm_collection(skos: &SkosCollection) -> RdmCollection {
        let mut rdm = RdmCollection::new(skos.id.clone());

        // Convert all concepts from the SKOS collection
        for (concept_id, skos_concept) in &skos.all_concepts {
            let mut pref_label = HashMap::new();
            for (lang, value) in &skos_concept.pref_labels {
                pref_label.insert(lang.clone(), value.value.clone());
            }

            // Find children (narrower concepts)
            let narrower: Vec<String> = skos.all_concepts.values()
                .filter(|c| {
                    // Check if this concept has the current concept as broader
                    // We don't have broader info directly, so we skip for now
                    false
                })
                .map(|c| c.id.clone())
                .collect();

            let rdm_concept = RdmConcept {
                id: concept_id.clone(),
                pref_label,
                alt_labels: HashMap::new(), // SKOS altLabels not in current structure
                broader: vec![], // Would need to extract from hierarchy
                narrower,
                scope_note: HashMap::new(),
            };

            rdm.add_concept(rdm_concept);
        }

        rdm
    }

    /// Look up a concept by label in a specific collection (Rust API)
    ///
    /// The collection_id comes from the node's config.rdmCollection
    pub fn find_concept_by_label(&self, collection_id: &str, label: &str) -> Option<RdmConcept> {
        self.get(collection_id)?.find_by_label_rust(label)
    }

    /// Look up a concept by label, returning all matches (Rust API)
    pub fn find_all_concepts_by_label(&self, collection_id: &str, label: &str) -> Vec<RdmConcept> {
        self.get(collection_id)
            .map(|c| c.find_all_by_label_rust(label))
            .unwrap_or_default()
    }
}

impl RdmCollection {
    /// Find by label - Rust API (exact case-insensitive match)
    fn find_by_label_rust(&self, label: &str) -> Option<RdmConcept> {
        let label_lower = label.to_lowercase();
        let matches: Vec<_> = self.concepts.values()
            .filter(|c| {
                c.pref_label.values().any(|p| p.to_lowercase() == label_lower) ||
                c.alt_labels.values().any(|alts| alts.iter().any(|l| l.to_lowercase() == label_lower))
            })
            .cloned()
            .collect();

        // Only return if exactly one match (unambiguous)
        if matches.len() == 1 { matches.into_iter().next() } else { None }
    }

    /// Find all by label - Rust API
    fn find_all_by_label_rust(&self, label: &str) -> Vec<RdmConcept> {
        let label_lower = label.to_lowercase();
        self.concepts.values()
            .filter(|c| {
                c.pref_label.values().any(|p| p.to_lowercase() == label_lower) ||
                c.alt_labels.values().any(|alts| alts.iter().any(|l| l.to_lowercase() == label_lower))
            })
            .cloned()
            .collect()
    }
}

// =============================================================================
// ConceptLookup Implementation
// =============================================================================

/// Implement the ConceptLookup trait for RdmCache
impl ConceptLookup for RdmCache {
    fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<String> {
        self.find_concept_by_label(collection_id, label)
            .map(|c| c.id)
    }
}

// =============================================================================
// Label Resolution Functions
// =============================================================================

/// Resolve label strings to UUIDs in a JSON tree.
///
/// This is the core label resolution function that can be called from Python.
/// It uses the centralized Rust implementation shared with WASM.
///
/// Args:
///     tree_json: JSON string of the tree to process
///     graph_json: JSON string of the graph definition
///     cache: RdmCache instance with collections loaded
///     resolvable_datatypes: List of datatypes to resolve (default: concept, concept-list, reference)
///     config_keys: List of config keys to check for collection IDs (default: rdmCollection, controlledList)
///     strict: If True, raise errors for unresolved labels
///
/// Returns:
///     Tuple of (resolved_json, needed_collection_ids)
#[pyfunction]
#[pyo3(signature = (tree_json, graph_json, cache, resolvable_datatypes=None, config_keys=None, strict=false))]
pub fn resolve_labels(
    tree_json: &str,
    graph_json: &str,
    cache: &RdmCache,
    resolvable_datatypes: Option<Vec<String>>,
    config_keys: Option<Vec<String>>,
    strict: bool,
) -> PyResult<(String, Vec<String>)> {
    let config = LabelResolutionConfig {
        resolvable_datatypes: resolvable_datatypes.unwrap_or_else(|| {
            label_resolution::DEFAULT_RESOLVABLE_DATATYPES
                .iter()
                .map(|s| s.to_string())
                .collect()
        }),
        config_keys: config_keys.unwrap_or_else(|| {
            label_resolution::DEFAULT_CONFIG_KEYS
                .iter()
                .map(|s| s.to_string())
                .collect()
        }),
        strict,
    };

    let (resolved_json, needed_collections) =
        label_resolution::resolve_labels_full(tree_json, graph_json, cache, &config)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.message))?;

    Ok((resolved_json, needed_collections.into_iter().collect()))
}

/// Get the list of collection IDs needed for a tree (without resolving).
///
/// This is useful for pre-fetching collections before resolution.
///
/// Args:
///     tree_json: JSON string of the tree to scan
///     graph_json: JSON string of the graph definition
///     resolvable_datatypes: List of datatypes to resolve (default: concept, concept-list, reference)
///     config_keys: List of config keys to check for collection IDs (default: rdmCollection, controlledList)
///
/// Returns:
///     List of collection IDs that need to be loaded
#[pyfunction]
#[pyo3(signature = (tree_json, graph_json, resolvable_datatypes=None, config_keys=None))]
pub fn get_needed_collections(
    tree_json: &str,
    graph_json: &str,
    resolvable_datatypes: Option<Vec<String>>,
    config_keys: Option<Vec<String>>,
) -> PyResult<Vec<String>> {
    let config = LabelResolutionConfig {
        resolvable_datatypes: resolvable_datatypes.unwrap_or_else(|| {
            label_resolution::DEFAULT_RESOLVABLE_DATATYPES
                .iter()
                .map(|s| s.to_string())
                .collect()
        }),
        config_keys: config_keys.unwrap_or_else(|| {
            label_resolution::DEFAULT_CONFIG_KEYS
                .iter()
                .map(|s| s.to_string())
                .collect()
        }),
        strict: false,
    };

    // Parse inputs
    let tree: serde_json::Value = serde_json::from_str(tree_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid tree JSON: {}", e)))?;

    let graph: serde_json::Value = serde_json::from_str(graph_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid graph JSON: {}", e)))?;

    // Build alias -> collection mapping
    let alias_to_collection = build_alias_to_collection_map(&graph, &config);

    // Find needed collections
    let needed = find_needed_collections(&tree, &alias_to_collection);

    Ok(needed.into_iter().collect())
}

/// Check if a string is a valid UUID.
#[pyfunction]
pub fn is_valid_uuid(s: &str) -> bool {
    label_resolution::is_valid_uuid(s)
}

// =============================================================================
// Module Registration
// =============================================================================

/// Register RDM cache types with the Python module
pub fn register_module(m: &PyModule) -> PyResult<()> {
    m.add_class::<RdmConcept>()?;
    m.add_class::<RdmCollection>()?;
    m.add_class::<RdmCache>()?;

    // Global RDM cache singleton functions
    m.add_function(wrap_pyfunction!(set_global_rdm_cache, m)?)?;
    m.add_function(wrap_pyfunction!(get_global_rdm_cache, m)?)?;
    m.add_function(wrap_pyfunction!(clear_global_rdm_cache, m)?)?;
    m.add_function(wrap_pyfunction!(has_global_rdm_cache, m)?)?;

    // Label resolution functions
    m.add_function(wrap_pyfunction!(resolve_labels, m)?)?;
    m.add_function(wrap_pyfunction!(get_needed_collections, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_uuid, m)?)?;

    Ok(())
}
