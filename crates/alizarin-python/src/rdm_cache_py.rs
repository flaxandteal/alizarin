/// RDM (Reference Data Manager) Cache for concept collections.
///
/// This module provides Python bindings that wrap core types from alizarin-core:
/// - RdmConcept: A concept value from a collection (wraps core::RdmConcept)
/// - RdmCollection: A collection of concepts (wraps core::RdmCollection)
/// - RdmCache: Cache for storing fetched collections (wraps core::RdmCache)
///
/// Python-specific features:
/// - Async loader callback for lazy loading
/// - Global singleton for automatic label resolution
/// - SKOS RDF/XML parsing
/// - UUID generation from labels

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use uuid::Uuid;

// Core types from alizarin-core
use alizarin_core::rdm_cache::{
    RdmCache as CoreRdmCache,
    RdmCollection as CoreRdmCollection,
    RdmConcept as CoreRdmConcept,
};

// SKOS parser and serializer from core crate
use alizarin_core::skos::{
    parse_skos_to_collections, collection_to_skos_xml,
    SkosCollection, SkosConcept, SkosValue,
};

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

// =============================================================================
// Concept Types
// =============================================================================

/// A concept value from an RDM collection.
///
/// Wraps core::RdmConcept with Python bindings.
#[derive(Clone, Debug)]
#[pyclass(name = "RustRdmConcept")]
pub struct RdmConcept {
    inner: CoreRdmConcept,
}

impl RdmConcept {
    /// Create from core concept
    pub fn from_core(inner: CoreRdmConcept) -> Self {
        Self { inner }
    }

    /// Get the inner core concept
    pub fn inner(&self) -> &CoreRdmConcept {
        &self.inner
    }

    /// Convert to core concept (consuming)
    pub fn into_inner(self) -> CoreRdmConcept {
        self.inner
    }
}

#[pymethods]
impl RdmConcept {
    #[new]
    fn new(id: String, pref_label: HashMap<String, String>) -> Self {
        Self {
            inner: CoreRdmConcept {
                id,
                pref_label,
                alt_labels: HashMap::new(),
                broader: vec![],
                narrower: vec![],
                scope_note: HashMap::new(),
            },
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
            inner: CoreRdmConcept {
                id,
                pref_label,
                alt_labels: HashMap::new(),
                broader: vec![],
                narrower: vec![],
                scope_note: HashMap::new(),
            },
        })
    }

    #[getter]
    fn id(&self) -> &str {
        &self.inner.id
    }

    #[getter]
    fn pref_label(&self) -> HashMap<String, String> {
        self.inner.pref_label.clone()
    }

    #[getter]
    fn alt_labels(&self) -> HashMap<String, Vec<String>> {
        self.inner.alt_labels.clone()
    }

    #[getter]
    fn broader(&self) -> Vec<String> {
        self.inner.broader.clone()
    }

    #[getter]
    fn narrower(&self) -> Vec<String> {
        self.inner.narrower.clone()
    }

    /// Get preferred label for a language
    fn get_label(&self, language: &str) -> Option<String> {
        self.inner.get_label(language)
    }

    fn __str__(&self) -> String {
        self.inner.get_label("en").unwrap_or_default()
    }

    fn __repr__(&self) -> String {
        format!("RustRdmConcept(id={}, labels={:?})", self.inner.id, self.inner.pref_label)
    }

    /// Convert to Python dict
    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let json_str = serde_json::to_string(&self.inner)
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
///
/// Wraps core::RdmCollection with Python-specific features like
/// UUID generation from labels.
#[derive(Clone, Debug, Default)]
#[pyclass(name = "RustRdmCollection")]
pub struct RdmCollection {
    inner: CoreRdmCollection,
}

impl RdmCollection {
    /// Create from core collection
    pub fn from_core(inner: CoreRdmCollection) -> Self {
        Self { inner }
    }

    /// Get the inner core collection
    pub fn inner(&self) -> &CoreRdmCollection {
        &self.inner
    }

    /// Get mutable inner core collection
    pub fn inner_mut(&mut self) -> &mut CoreRdmCollection {
        &mut self.inner
    }

    /// Convert to core collection (consuming)
    pub fn into_inner(self) -> CoreRdmCollection {
        self.inner
    }
}

/// Namespace UUID for generating collection IDs from names
/// Derived from the project's base namespace: uuid5("1a79f1c8-9505-4bea-a18e-28a053f725ca", "collection")
const COLLECTION_NAMESPACE: &str = "a8e5f3b2-7c41-5d8a-9f12-3b4c5d6e7f8a";

#[pymethods]
impl RdmCollection {
    #[new]
    fn new(id: String) -> Self {
        Self {
            inner: CoreRdmCollection::new(id),
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
            inner: CoreRdmCollection::with_name(collection_id, name),
        };

        // Add each label as a concept
        for label in labels {
            collection.add_from_label(py, label, None)?;
        }

        Ok(collection)
    }

    #[getter]
    fn id(&self) -> &str {
        &self.inner.id
    }

    #[getter]
    fn name(&self) -> Option<&str> {
        self.inner.name.as_deref()
    }

    /// Add a concept to the collection
    fn add_concept(&mut self, concept: RdmConcept) {
        self.inner.add_concept(concept.into_inner());
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
                let namespace = Uuid::parse_str(&self.inner.id).map_err(|e| {
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

        let concept = CoreRdmConcept {
            id: concept_id.clone(),
            pref_label,
            alt_labels: HashMap::new(),
            broader: vec![],
            narrower: vec![],
            scope_note: HashMap::new(),
        };

        self.inner.add_concept(concept);
        Ok(concept_id)
    }

    /// Look up a concept by ID
    fn get_concept(&self, concept_id: &str) -> Option<RdmConcept> {
        self.inner.get_concept(concept_id).cloned().map(RdmConcept::from_core)
    }

    /// Check if concept exists in collection
    fn has_concept(&self, concept_id: &str) -> bool {
        self.inner.has_concept(concept_id)
    }

    /// Get all concept IDs
    fn get_concept_ids(&self) -> Vec<String> {
        self.inner.get_concept_ids().into_iter().cloned().collect()
    }

    /// Get top-level concepts
    fn get_top_concepts(&self) -> Vec<RdmConcept> {
        self.inner.get_top_concepts()
            .into_iter()
            .cloned()
            .map(RdmConcept::from_core)
            .collect()
    }

    /// Get number of concepts
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Search concepts by label (case-insensitive prefix match)
    fn search(&self, query: &str, language: Option<&str>) -> Vec<RdmConcept> {
        self.inner.search(query, language)
            .into_iter()
            .cloned()
            .map(RdmConcept::from_core)
            .collect()
    }

    /// Find a concept by exact label match (case-insensitive)
    ///
    /// Searches pref_label and alt_labels across all languages.
    /// Returns None if no match or multiple matches (ambiguous).
    fn find_by_label(&self, label: &str) -> Option<RdmConcept> {
        self.inner.find_by_label(label).cloned().map(RdmConcept::from_core)
    }

    /// Find a concept by exact label match, returning all matches
    fn find_all_by_label(&self, label: &str) -> Vec<RdmConcept> {
        self.inner.find_all_by_label(label)
            .into_iter()
            .cloned()
            .map(RdmConcept::from_core)
            .collect()
    }

    /// Serialize this collection to SKOS RDF/XML format.
    ///
    /// This calls the core serialization function and returns the XML string
    /// for Python to write to a file or send over the network.
    ///
    /// Args:
    ///     base_uri: Base URI for the SKOS resources (e.g., "http://example.org/")
    ///
    /// Returns:
    ///     SKOS RDF/XML as a string
    ///
    /// Example:
    ///     collection = RustRdmCollection.from_labels("Test", ["A", "B", "C"])
    ///     xml = collection.to_skos_xml("http://example.org/")
    ///     with open("output.xml", "w") as f:
    ///         f.write(xml)
    fn to_skos_xml(&self, base_uri: &str) -> PyResult<String> {
        // Convert RdmCollection to SkosCollection
        let skos_collection = rdm_to_skos_collection(&self.inner);

        // Serialize to XML using core function
        Ok(collection_to_skos_xml(&skos_collection, base_uri))
    }
}

// =============================================================================
// RDM ↔ SKOS Conversion
// =============================================================================

/// Generate a value ID for SKOS format
fn generate_skos_value_id(concept_id: &str, lang: &str, value: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    format!("{}/{}/{}", concept_id, lang, value).hash(&mut hasher);
    let hash = hasher.finish();

    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (hash >> 32) as u32,
        ((hash >> 16) & 0xFFFF) as u16,
        (hash & 0xFFFF) as u16,
        ((hash >> 48) & 0xFFFF) as u16,
        hash & 0xFFFFFFFFFFFF
    )
}

/// Convert RdmCollection to SkosCollection for serialization
fn rdm_to_skos_collection(rdm: &CoreRdmCollection) -> SkosCollection {
    // Build collection pref_labels
    let mut collection_pref_labels = HashMap::new();
    if let Some(ref name) = rdm.name {
        collection_pref_labels.insert(
            "en".to_string(),
            SkosValue {
                id: generate_skos_value_id(&rdm.id, "en", name),
                value: name.clone(),
            },
        );
    }

    // Convert all concepts (flat list first) - using public API
    let mut all_skos_concepts: HashMap<String, SkosConcept> = HashMap::new();
    let mut all_narrower_ids: HashSet<String> = HashSet::new();

    for concept_id in rdm.get_concept_ids() {
        if let Some(rdm_concept) = rdm.get_concept(concept_id) {
            // Convert pref_labels to SkosValue format
            let mut pref_labels = HashMap::new();
            for (lang, label) in &rdm_concept.pref_label {
                pref_labels.insert(
                    lang.clone(),
                    SkosValue {
                        id: generate_skos_value_id(concept_id, lang, label),
                        value: label.clone(),
                    },
                );
            }

            let skos_concept = SkosConcept {
                id: concept_id.clone(),
                uri: None,  // No URI in RDM format
                pref_labels,
                source: Some(concept_id.clone()),
                sort_order: None,  // No sort order in RDM format
                children: None,  // Will be built hierarchically later
            };

            all_skos_concepts.insert(concept_id.clone(), skos_concept);

            // Collect all narrower IDs
            all_narrower_ids.extend(rdm_concept.narrower.iter().cloned());
        }
    }

    // Build hierarchy - find top-level concepts (those not in anyone's narrower list)
    let mut top_level_concepts: HashMap<String, SkosConcept> = HashMap::new();

    for concept_id in rdm.get_concept_ids() {
        if !all_narrower_ids.contains(concept_id) {
            // This is a top-level concept
            if let Some(concept_with_children) = build_concept_tree(
                concept_id,
                &all_skos_concepts,
                rdm,
            ) {
                top_level_concepts.insert(concept_id.clone(), concept_with_children);
            }
        }
    }

    // If no hierarchy, just use all concepts as top-level
    if top_level_concepts.is_empty() {
        top_level_concepts = all_skos_concepts.clone();
    }

    SkosCollection {
        id: rdm.id.clone(),
        pref_labels: collection_pref_labels,
        concepts: top_level_concepts,
        all_concepts: all_skos_concepts,
        values: HashMap::new(),
    }
}

/// Build a concept tree recursively from RDM narrower relationships
fn build_concept_tree(
    concept_id: &str,
    all_concepts: &HashMap<String, SkosConcept>,
    rdm_collection: &CoreRdmCollection,
) -> Option<SkosConcept> {
    let mut concept = all_concepts.get(concept_id)?.clone();

    // Get narrower concepts from RDM using public API
    if let Some(rdm_concept) = rdm_collection.get_concept(concept_id) {
        if !rdm_concept.narrower.is_empty() {
            let mut children = Vec::new();
            for child_id in &rdm_concept.narrower {
                if let Some(child) = build_concept_tree(child_id, all_concepts, rdm_collection) {
                    children.push(child);
                }
            }
            if !children.is_empty() {
                concept.children = Some(children);
            }
        }
    }

    Some(concept)
}

// =============================================================================
// RDM Cache
// =============================================================================

/// Cache for RDM concept collections.
///
/// Wraps core::RdmCache with Python-specific features:
/// - Async loader callback for lazy loading
/// - SKOS RDF/XML parsing
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
    inner: CoreRdmCache,
    /// Optional async loader for lazy loading (stored as Python object)
    #[pyo3(get, set)]
    loader: Option<PyObject>,
}

impl RdmCache {
    /// Get the inner core cache
    pub fn inner(&self) -> &CoreRdmCache {
        &self.inner
    }

    /// Get mutable inner core cache
    pub fn inner_mut(&mut self) -> &mut CoreRdmCache {
        &mut self.inner
    }

    /// Convert SKOS collection to RDM collection
    fn skos_to_rdm_collection(skos: &SkosCollection) -> CoreRdmCollection {
        let mut rdm = CoreRdmCollection::with_name(
            skos.id.clone(),
            skos.pref_labels.get("en").map(|v| v.value.clone()).unwrap_or_else(|| skos.id.clone())
        );

        // Recursive function to add a concept and its children
        fn add_concept_recursive(rdm: &mut CoreRdmCollection, skos_concept: &SkosConcept) {
            let mut pref_label = HashMap::new();
            for (lang, value) in &skos_concept.pref_labels {
                pref_label.insert(lang.clone(), value.value.clone());
            }

            // Extract narrower IDs from children
            let narrower: Vec<String> = skos_concept.children
                .as_ref()
                .map(|children| children.iter().map(|c| c.id.clone()).collect())
                .unwrap_or_default();

            let rdm_concept = CoreRdmConcept {
                id: skos_concept.id.clone(),
                pref_label,
                alt_labels: HashMap::new(),
                broader: vec![],
                narrower,
                scope_note: HashMap::new(),
            };

            rdm.add_concept(rdm_concept);

            // Recursively add children
            if let Some(ref children) = skos_concept.children {
                for child in children {
                    add_concept_recursive(rdm, child);
                }
            }
        }

        // Add all top-level concepts (and their children recursively)
        for skos_concept in skos.concepts.values() {
            add_concept_recursive(&mut rdm, skos_concept);
        }

        // Also add from all_concepts if not already added (fallback for flat structures)
        if rdm.len() == 0 && !skos.all_concepts.is_empty() {
            for skos_concept in skos.all_concepts.values() {
                if !rdm.has_concept(&skos_concept.id) {
                    add_concept_recursive(&mut rdm, skos_concept);
                }
            }
        }

        rdm
    }
}

#[pymethods]
impl RdmCache {
    #[new]
    #[pyo3(signature = (loader=None))]
    fn new(loader: Option<PyObject>) -> Self {
        Self {
            inner: CoreRdmCache::new(),
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
        if self.inner.has_collection(collection_id) {
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
        self.inner.add_collection_from_json(collection_id, concepts_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }

    /// Add a pre-built collection
    fn add_collection(&mut self, collection: RdmCollection) {
        self.inner.add_collection(collection.into_inner());
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
            self.inner.add_collection(rdm_collection);
            added_ids.push(id);
        }

        Ok(added_ids)
    }

    /// Get a collection by ID
    fn get_collection(&self, collection_id: &str) -> Option<RdmCollection> {
        self.inner.get_collection(collection_id).cloned().map(RdmCollection::from_core)
    }

    /// Check if collection is cached
    fn has_collection(&self, collection_id: &str) -> bool {
        self.inner.has_collection(collection_id)
    }

    /// Look up a concept directly
    ///
    /// This is the main lookup method for coercion:
    /// given a collection ID and concept ID, return the concept.
    fn lookup_concept(&self, collection_id: &str, concept_id: &str) -> Option<RdmConcept> {
        self.inner.lookup_concept(collection_id, concept_id)
            .cloned()
            .map(RdmConcept::from_core)
    }

    /// Validate that a concept exists in a collection
    fn validate_concept(&self, collection_id: &str, concept_id: &str) -> bool {
        self.inner.validate_concept(collection_id, concept_id)
    }

    /// Look up a concept by label in a specific collection
    ///
    /// Returns the concept if exactly one match is found.
    /// Returns None if no match or ambiguous (multiple matches).
    fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<RdmConcept> {
        self.inner.lookup_by_label(collection_id, label)
            .cloned()
            .map(RdmConcept::from_core)
    }

    /// Look up a concept by label, returning all matches
    fn lookup_all_by_label(&self, collection_id: &str, label: &str) -> Vec<RdmConcept> {
        self.inner.lookup_all_by_label(collection_id, label)
            .into_iter()
            .cloned()
            .map(RdmConcept::from_core)
            .collect()
    }

    /// Get all cached collection IDs
    fn get_collection_ids(&self) -> Vec<String> {
        self.inner.get_collection_ids()
    }

    /// Clear all cached collections
    fn clear(&mut self) {
        self.inner.clear();
    }

    /// Clear a specific collection
    fn clear_collection(&mut self, collection_id: &str) {
        self.inner.remove_collection(collection_id);
    }

    /// Get number of cached collections
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Search across all collections (for autocomplete)
    fn search_all(&self, query: &str, language: Option<&str>) -> Vec<(String, RdmConcept)> {
        self.inner.search_all(query, language)
            .into_iter()
            .map(|(coll_id, concept)| (coll_id.to_string(), RdmConcept::from_core(concept.clone())))
            .collect()
    }
}

// =============================================================================
// ConceptLookup Implementation
// =============================================================================

/// Implement the ConceptLookup trait for RdmCache
impl ConceptLookup for RdmCache {
    fn lookup_by_label(&self, collection_id: &str, label: &str) -> Option<String> {
        self.inner.lookup_by_label(collection_id, label)
            .map(|c| c.id.clone())
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

    // Use the inner core cache for label resolution
    let (resolved_json, needed_collections) =
        label_resolution::resolve_labels_full(tree_json, graph_json, cache.inner(), &config)
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
