#![allow(deprecated)]
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
    RdmCache as CoreRdmCache, RdmCollection as CoreRdmCollection, RdmConcept as CoreRdmConcept,
    RdmValue as CoreRdmValue,
};
use alizarin_core::rdm_namespace::{
    generate_collection_uuid as core_generate_collection,
    generate_concept_uuid as core_generate_concept,
    labels_to_deterministic_string as core_labels_to_string,
    parse_rdm_namespace as core_parse_namespace,
};

// SKOS parser and serializer from core crate
use alizarin_core::skos::{
    collection_to_skos_xml, parse_skos_to_collections, SkosCollection, SkosConcept, SkosNodeType,
    SkosValue,
};

// For get_current_language
use alizarin_core::type_coercion::get_current_language;

// Label resolution from core
use alizarin_core::label_resolution::{
    self, build_alias_to_collection_map, find_needed_collections, ConceptLookup,
    LabelResolutionConfig,
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
/// Note: This clones the cache, so you can continue using the original.
///
/// Example:
///     cache = RustRdmCache()
///     cache.add_collection(my_collection)
///     set_global_rdm_cache(cache)
///
///     # Now json_tree_to_tiles will auto-resolve labels
///     result = json_tree_to_tiles(tree_json, ...)
///     # cache is still usable here
#[pyfunction]
pub fn set_global_rdm_cache(cache: &RdmCache) {
    if let Ok(mut guard) = GLOBAL_RDM_CACHE.write() {
        *guard = Some(cache.clone());
    }
}

/// Get a clone of the global RDM cache, if set.
#[pyfunction]
pub fn get_global_rdm_cache() -> Option<RdmCache> {
    GLOBAL_RDM_CACHE.read().ok().and_then(|guard| guard.clone())
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

/// Add a collection directly to the global RDM cache.
///
/// This modifies the global cache in-place. If no global cache exists,
/// creates one first.
///
/// Example:
///     collection = RustRdmCollection.from_labels("Test", ["A", "B"])
///     add_collection_to_global_cache(collection)
#[pyfunction]
pub fn add_collection_to_global_cache(collection: RdmCollection) {
    if let Ok(mut guard) = GLOBAL_RDM_CACHE.write() {
        if guard.is_none() {
            *guard = Some(RdmCache::default());
        }
        if let Some(ref mut cache) = *guard {
            cache.inner.add_collection(collection.into_inner());
        }
    }
}

/// Add collections from SKOS XML directly to the global RDM cache.
///
/// This parses the SKOS XML and adds all collections to the global cache.
/// If no global cache exists, creates one first.
///
/// Args:
///     xml_content: SKOS XML string
///     base_uri: Base URI for resolving relative URIs
///
/// Returns:
///     List of collection IDs that were added
///
/// Example:
///     ids = add_from_skos_xml_to_global_cache(xml_string, "http://example.org/")
#[pyfunction]
pub fn add_from_skos_xml_to_global_cache(
    xml_content: &str,
    base_uri: &str,
) -> PyResult<Vec<String>> {
    let skos_collections = parse_skos_to_collections(xml_content, base_uri).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse SKOS XML: {}", e))
    })?;

    let mut added_ids = Vec::new();

    if let Ok(mut guard) = GLOBAL_RDM_CACHE.write() {
        if guard.is_none() {
            *guard = Some(RdmCache::default());
        }
        if let Some(ref mut cache) = *guard {
            for skos_coll in skos_collections {
                let rdm_collection = RdmCache::skos_to_rdm_collection(&skos_coll);
                let id = rdm_collection.id.clone();
                cache.inner.add_collection(rdm_collection);
                added_ids.push(id);
            }
        }
    }

    Ok(added_ids)
}

/// Merge additional flat labels into a collection in the global RDM cache.
///
/// Retrieves the collection, adds new labels (skipping duplicates by
/// deterministic concept ID), and stores it back in the global cache.
///
/// Args:
///     collection_id: ID of the collection to update
///     labels: List of labels (strings or {lang: label} dicts) to add
///
/// Returns:
///     Number of new concepts added
///
/// Raises:
///     ValueError: If collection_id is not in the global cache
#[pyfunction]
pub fn update_collection_in_global_cache(
    py: Python,
    collection_id: String,
    labels: Vec<PyObject>,
) -> PyResult<usize> {
    let core_coll = {
        let guard = GLOBAL_RDM_CACHE.read().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to read global RDM cache")
        })?;
        let cache = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Global RDM cache not set")
        })?;
        cache
            .inner
            .get_collection(&collection_id)
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Collection '{}' not found in global cache",
                    collection_id
                ))
            })?
            .clone()
    };

    let mut wrapper = RdmCollection::from_core(core_coll);
    let added = wrapper.update_from_labels(py, labels)?;

    if let Ok(mut guard) = GLOBAL_RDM_CACHE.write() {
        if let Some(ref mut cache) = *guard {
            cache.inner.add_collection(wrapper.into_inner());
        }
    }
    Ok(added)
}

/// Merge additional hierarchical entries into a collection in the global RDM cache.
///
/// Retrieves the collection, adds new entries from a nested dict (skipping
/// duplicates, recursing into children of existing parents), and stores it back.
///
/// Args:
///     collection_id: ID of the collection to update
///     structure: Nested dict where keys are labels, values are None (leaf) or dict (children)
///
/// Returns:
///     Number of new concepts added
///
/// Raises:
///     ValueError: If collection_id is not in the global cache
#[pyfunction]
pub fn update_collection_nested_in_global_cache(
    py: Python,
    collection_id: String,
    structure: &Bound<'_, PyDict>,
) -> PyResult<usize> {
    let core_coll = {
        let guard = GLOBAL_RDM_CACHE.read().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to read global RDM cache")
        })?;
        let cache = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Global RDM cache not set")
        })?;
        cache
            .inner
            .get_collection(&collection_id)
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Collection '{}' not found in global cache",
                    collection_id
                ))
            })?
            .clone()
    };

    let mut wrapper = RdmCollection::from_core(core_coll);
    let added = wrapper.update_from_nested_labels(py, structure)?;

    if let Ok(mut guard) = GLOBAL_RDM_CACHE.write() {
        if let Some(ref mut cache) = *guard {
            cache.inner.add_collection(wrapper.into_inner());
        }
    }
    Ok(added)
}

// =============================================================================
// Global RDM Namespace for UUID Generation
// =============================================================================

lazy_static::lazy_static! {
    /// Global namespace UUID for deterministic RDM ID generation.
    /// Must be set before creating collections/concepts from labels.
    static ref GLOBAL_RDM_NAMESPACE: RwLock<Option<Uuid>> = RwLock::new(None);
}

/// Set the global RDM namespace for deterministic UUID generation.
///
/// This namespace is used when creating collections and concepts from labels
/// without explicit IDs. Must be set before using from_labels, from_nested_labels,
/// add_from_label, or add_child_from_label with auto-generated IDs.
///
/// Args:
///     namespace: Either a valid UUID string or a URL to use as the namespace.
///                If a URL is provided (starts with http:// or https://), it will
///                be converted to a deterministic UUID using UUID5 with the standard
///                URL namespace. This allows using the same URL as both the RDF
///                namespace and the UUID generation namespace.
///
/// Example:
///     # Using a UUID directly
///     set_rdm_namespace("550e8400-e29b-41d4-a716-446655440000")
///
///     # Using a URL (same URL can be used as RDF namespace)
///     set_rdm_namespace("http://example.org/rdm/")
///     collection = RustRdmCollection.from_labels("MyCollection", ["A", "B"])
#[pyfunction]
pub fn set_rdm_namespace(namespace: &str) -> PyResult<()> {
    let uuid =
        core_parse_namespace(namespace).map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
    if let Ok(mut guard) = GLOBAL_RDM_NAMESPACE.write() {
        *guard = Some(uuid);
    }
    Ok(())
}

/// Get the current global RDM namespace, if set.
///
/// Returns:
///     The namespace UUID as a string, or None if not set
#[pyfunction]
pub fn get_rdm_namespace() -> Option<String> {
    GLOBAL_RDM_NAMESPACE
        .read()
        .ok()
        .and_then(|guard| guard.map(|u| u.to_string()))
}

/// Check if a global RDM namespace is set.
#[pyfunction]
pub fn has_rdm_namespace() -> bool {
    GLOBAL_RDM_NAMESPACE
        .read()
        .ok()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

/// Clear the global RDM namespace.
#[pyfunction]
pub fn clear_rdm_namespace() {
    if let Ok(mut guard) = GLOBAL_RDM_NAMESPACE.write() {
        *guard = None;
    }
}

/// Get the global namespace or return an error if not set.
fn get_required_namespace() -> PyResult<Uuid> {
    GLOBAL_RDM_NAMESPACE
        .read()
        .ok()
        .and_then(|guard| *guard)
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "RDM namespace not set. Call set_rdm_namespace() before creating \
                 collections or concepts from labels without explicit IDs.",
            )
        })
}

/// Convert a label (string or multilingual dict) to a consistent string for UUID generation.
///
/// For strings: returns the string directly
/// For dicts: sorts by language code, concatenates as "lang1:value1|lang2:value2"
///
/// This ensures deterministic UUID generation regardless of dict ordering.
fn label_to_deterministic_string(py: Python, label: &PyObject) -> PyResult<String> {
    if let Ok(s) = label.extract::<String>(py) {
        return Ok(s);
    }

    if let Ok(dict) = label.downcast::<PyDict>(py) {
        let mut labels_map: HashMap<String, String> = HashMap::new();
        for (key, value) in dict.iter() {
            let lang: String = key.extract()?;
            let label_str: String = value.extract()?;
            labels_map.insert(lang, label_str);
        }
        // Use core function for deterministic string generation
        return Ok(core_labels_to_string(&labels_map));
    }

    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
        "label must be a string or dict of {language: label}",
    ))
}

/// Extract a trimmed deterministic label string from a PyObject label.
///
/// Mirrors the logic in `add_from_label` for computing the string used in
/// concept ID generation: plain strings are trimmed, multilingual dicts
/// go through `label_to_deterministic_string` then trimmed.
fn extract_label_string(py: Python, label: &PyObject) -> PyResult<String> {
    if let Ok(s) = label.extract::<String>(py) {
        return Ok(s.trim().to_string());
    }
    if label.downcast::<PyDict>(py).is_ok() {
        return Ok(label_to_deterministic_string(py, label)?.trim().to_string());
    }
    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
        "label must be a string or dict of {language: label}",
    ))
}

/// Generate a deterministic UUID for a collection from its name.
///
/// Uses: uuid5(global_namespace, "collection/" + name)
fn generate_collection_id(name: &str) -> PyResult<String> {
    let namespace = get_required_namespace()?;
    Ok(core_generate_collection(&namespace, name).to_string())
}

/// Generate a deterministic UUID for a concept from its collection and label.
///
/// Uses: uuid5(collection_id_as_uuid, label_string)
fn generate_concept_id(collection_id: &str, label_string: &str) -> PyResult<String> {
    let namespace = Uuid::parse_str(collection_id).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Collection ID must be a valid UUID for auto-generation: {}",
            e
        ))
    })?;
    Ok(core_generate_concept(&namespace, label_string).to_string())
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
        // Convert string labels to RdmValue (IDs will be generated when added to collection)
        let pref_label_values: HashMap<String, CoreRdmValue> = pref_label
            .into_iter()
            .map(|(lang, value)| (lang, CoreRdmValue::new("__pending__".to_string(), value)))
            .collect();

        Self {
            inner: CoreRdmConcept {
                id,
                pref_label: pref_label_values,
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
        let pref_label: HashMap<String, CoreRdmValue> = if let Ok(s) = label.extract::<String>(py) {
            let lang = get_current_language();
            let mut map = HashMap::new();
            map.insert(lang, CoreRdmValue::new("__pending__".to_string(), s));
            map
        } else if let Ok(dict) = label.downcast::<PyDict>(py) {
            let mut map = HashMap::new();
            for (key, value) in dict.iter() {
                let lang: String = key.extract()?;
                let label_str: String = value.extract()?;
                map.insert(
                    lang,
                    CoreRdmValue::new("__pending__".to_string(), label_str),
                );
            }
            map
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "label must be a string or dict of {language: label}",
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
        // Extract just the value strings for backward compatibility
        self.inner
            .pref_label
            .iter()
            .map(|(lang, v)| (lang.clone(), v.value.clone()))
            .collect()
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
        format!(
            "RustRdmConcept(id={}, labels={:?})",
            self.inner.id, self.inner.pref_label
        )
    }

    /// Convert to Python dict
    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let json_str = serde_json::to_string(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let json_module = py.import_bound("json")?;
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
///
/// Both node types support hierarchical concepts (narrower/broader relationships):
/// - "ConceptScheme" (default): Uses skos:inScheme for membership
/// - "Collection": Uses skos:member for membership (Arches-compatible), lists all concepts including children
#[derive(Clone, Debug, Default)]
#[pyclass(name = "RustRdmCollection")]
pub struct RdmCollection {
    inner: CoreRdmCollection,
    /// Node type: "ConceptScheme" or "Collection"
    node_type: String,
}

impl RdmCollection {
    /// Create from core collection (defaults to ConceptScheme)
    pub fn from_core(inner: CoreRdmCollection) -> Self {
        Self {
            inner,
            node_type: "ConceptScheme".to_string(),
        }
    }

    /// Create from core collection with specified node type
    pub fn from_core_with_type(inner: CoreRdmCollection, node_type: &str) -> Self {
        Self {
            inner,
            node_type: node_type.to_string(),
        }
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

    /// Get the node type
    pub fn get_node_type(&self) -> &str {
        &self.node_type
    }
}

#[pymethods]
impl RdmCollection {
    #[new]
    #[pyo3(signature = (id, node_type=None))]
    fn new(id: String, node_type: Option<String>) -> Self {
        Self {
            inner: CoreRdmCollection::new(id),
            node_type: node_type.unwrap_or_else(|| "ConceptScheme".to_string()),
        }
    }

    /// Create a collection from a name and list of labels.
    ///
    /// Creates top-level concepts only. Use add_child_from_label() to create
    /// hierarchical relationships (narrower/broader) between concepts.
    ///
    /// Both ConceptScheme and Collection types support hierarchical concepts.
    /// The difference is how membership is expressed in SKOS RDF:
    /// - ConceptScheme: uses skos:inScheme on concepts
    /// - Collection: uses skos:member to list all concepts (including children)
    ///
    /// Args:
    ///     name: The collection name (also used to generate ID if not provided)
    ///     labels: List of concept labels (strings only - IDs are auto-generated)
    ///     id: Optional explicit collection ID. If not provided, generates uuid5 from name.
    ///     node_type: "ConceptScheme" (default) or "Collection" (Arches-compatible)
    ///
    /// Example:
    ///     # Auto-generate collection ID from name, concept IDs from labels
    ///     collection = RustRdmCollection.from_labels("Categories", ["Animals"])
    ///
    ///     # Add a child concept with hierarchy
    ///     parent = collection.find_by_label("Animals")
    ///     collection.add_child_from_label(parent.id, "Mammals")
    ///
    ///     # Create an Arches-compatible Collection (with hierarchy support)
    ///     collection = RustRdmCollection.from_labels(
    ///         "Categories",
    ///         ["Animals"],
    ///         node_type="Collection"
    ///     )
    ///
    ///     # Explicit collection ID
    ///     collection = RustRdmCollection.from_labels("Categories", ["Cat A", "Cat B"], id="my-uuid")
    #[staticmethod]
    #[pyo3(signature = (name, labels, id=None, node_type=None))]
    fn from_labels(
        py: Python,
        name: String,
        labels: Vec<PyObject>,
        id: Option<String>,
        node_type: Option<String>,
    ) -> PyResult<Self> {
        // Generate or use provided collection ID
        let collection_id = match id {
            Some(explicit_id) => explicit_id,
            None => generate_collection_id(&name)?,
        };

        let mut collection = Self {
            inner: CoreRdmCollection::with_name(collection_id, name),
            node_type: node_type.unwrap_or_else(|| "ConceptScheme".to_string()),
        };

        // Add each label as a concept
        for label in labels {
            collection.add_from_label(py, label, None)?;
        }

        Ok(collection)
    }

    /// Create a hierarchical collection from a nested dictionary structure.
    ///
    /// The structure uses labels as keys and either None (leaf) or dict (children) as values:
    ///
    /// Args:
    ///     name: The collection name
    ///     structure: Nested dict where keys are labels, values are None (leaf) or dict (children)
    ///     id: Optional explicit collection ID
    ///     node_type: "ConceptScheme" (default) or "Collection" (Arches-compatible)
    ///
    /// Example:
    ///     # Create a hierarchical taxonomy
    ///     collection = RustRdmCollection.from_nested_labels(
    ///         "Animals",
    ///         {
    ///             "Mammals": {
    ///                 "Dogs": None,
    ///                 "Cats": None
    ///             },
    ///             "Birds": {
    ///                 "Eagles": None,
    ///                 "Sparrows": None
    ///             }
    ///         }
    ///     )
    ///
    ///     # Result: 6 concepts with hierarchy:
    ///     # - Mammals (top-level)
    ///     #   - Dogs (child of Mammals)
    ///     #   - Cats (child of Mammals)
    ///     # - Birds (top-level)
    ///     #   - Eagles (child of Birds)
    ///     #   - Sparrows (child of Birds)
    ///
    ///     # For flat collections, values can all be None:
    ///     flat = RustRdmCollection.from_nested_labels(
    ///         "Colors",
    ///         {"Red": None, "Green": None, "Blue": None}
    ///     )
    #[staticmethod]
    #[pyo3(signature = (name, structure, id=None, node_type=None))]
    fn from_nested_labels(
        py: Python,
        name: String,
        structure: &Bound<'_, PyDict>,
        id: Option<String>,
        node_type: Option<String>,
    ) -> PyResult<Self> {
        // Generate or use provided collection ID
        let collection_id = match id {
            Some(explicit_id) => explicit_id,
            None => generate_collection_id(&name)?,
        };

        let mut collection = Self {
            inner: CoreRdmCollection::with_name(collection_id, name),
            node_type: node_type.unwrap_or_else(|| "ConceptScheme".to_string()),
        };

        // Recursive helper to add concepts from nested structure
        fn add_nested(
            py: Python,
            collection: &mut RdmCollection,
            structure: &Bound<'_, PyDict>,
            parent_id: Option<String>,
        ) -> PyResult<()> {
            for (key, value) in structure.iter() {
                // Key is the label (string or multilingual dict)
                let label: PyObject = key.into_py(py);

                // Add concept (as child if parent_id is set)
                let concept_id = if let Some(ref pid) = parent_id {
                    collection.add_child_from_label(py, pid.clone(), label, None)?
                } else {
                    collection.add_from_label(py, label, None)?
                };

                // If value is a dict, recurse for children
                if let Ok(children_dict) = value.downcast::<PyDict>() {
                    add_nested(py, collection, children_dict, Some(concept_id))?;
                }
                // If value is None or anything else, it's a leaf node (already added)
            }
            Ok(())
        }

        add_nested(py, &mut collection, structure, None)?;

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

    /// Get the node type ("ConceptScheme" or "Collection")
    #[getter]
    fn node_type(&self) -> &str {
        &self.node_type
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
    fn add_from_label(
        &mut self,
        py: Python,
        label: PyObject,
        id: Option<String>,
    ) -> PyResult<String> {
        // Extract pref_label as HashMap and get deterministic string for ID generation
        // Labels are trimmed to normalize whitespace
        let (pref_label, label_string): (HashMap<String, CoreRdmValue>, String) =
            if let Ok(s) = label.extract::<String>(py) {
                let trimmed = s.trim().to_string();
                let lang = get_current_language();
                let mut map = HashMap::new();
                map.insert(
                    lang,
                    CoreRdmValue::new("__pending__".to_string(), trimmed.clone()),
                );
                (map, trimmed)
            } else if let Ok(dict) = label.downcast::<PyDict>(py) {
                let mut map = HashMap::new();
                for (key, value) in dict.iter() {
                    let lang: String = key.extract()?;
                    let label_str: String = value.extract()?;
                    map.insert(
                        lang,
                        CoreRdmValue::new("__pending__".to_string(), label_str.trim().to_string()),
                    );
                }
                // Get deterministic string from multilingual dict (uses trimmed values)
                let det_string = label_to_deterministic_string(py, &label)?
                    .trim()
                    .to_string();
                (map, det_string)
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "label must be a string or dict of {language: label}",
                ));
            };

        // Determine the concept ID
        let concept_id = match id {
            Some(explicit_id) => explicit_id,
            None => generate_concept_id(&self.inner.id, &label_string)?,
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

    /// Add a child concept under an existing parent concept.
    ///
    /// This creates a narrower/broader relationship between the parent and child.
    /// Both ConceptScheme and Collection types support hierarchical concepts.
    ///
    /// Args:
    ///     parent_id: The ID of the parent concept
    ///     label: Label as string or {lang: label} dict
    ///     id: Optional explicit ID for the child concept
    ///
    /// Returns:
    ///     The ID of the newly created child concept
    ///
    /// Raises:
    ///     ValueError: If parent_id doesn't exist in the collection
    ///
    /// Example:
    ///     collection = RustRdmCollection.from_labels("Categories", ["Animals"])
    ///     parent_id = collection.find_by_label("Animals").id
    ///     child_id = collection.add_child_from_label(parent_id, "Mammals")
    #[pyo3(signature = (parent_id, label, id=None))]
    fn add_child_from_label(
        &mut self,
        py: Python,
        parent_id: String,
        label: PyObject,
        id: Option<String>,
    ) -> PyResult<String> {
        // Verify parent exists
        if !self.inner.has_concept(&parent_id) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Parent concept '{}' not found in collection",
                parent_id
            )));
        }

        // Extract pref_label as HashMap and get deterministic string for ID generation
        // Labels are trimmed to normalize whitespace
        let (pref_label, label_string): (HashMap<String, CoreRdmValue>, String) =
            if let Ok(s) = label.extract::<String>(py) {
                let trimmed = s.trim().to_string();
                let lang = get_current_language();
                let mut map = HashMap::new();
                map.insert(
                    lang,
                    CoreRdmValue::new("__pending__".to_string(), trimmed.clone()),
                );
                (map, trimmed)
            } else if let Ok(dict) = label.downcast::<PyDict>(py) {
                let mut map = HashMap::new();
                for (key, value) in dict.iter() {
                    let lang: String = key.extract()?;
                    let label_str: String = value.extract()?;
                    map.insert(
                        lang,
                        CoreRdmValue::new("__pending__".to_string(), label_str.trim().to_string()),
                    );
                }
                // Get deterministic string from multilingual dict (uses trimmed values)
                let det_string = label_to_deterministic_string(py, &label)?
                    .trim()
                    .to_string();
                (map, det_string)
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "label must be a string or dict of {language: label}",
                ));
            };

        // Determine the concept ID
        let concept_id = match id {
            Some(explicit_id) => explicit_id,
            None => generate_concept_id(&self.inner.id, &label_string)?,
        };

        // Create concept with broader set to parent
        let concept = CoreRdmConcept {
            id: concept_id.clone(),
            pref_label,
            alt_labels: HashMap::new(),
            broader: vec![parent_id.clone()],
            narrower: vec![],
            scope_note: HashMap::new(),
        };

        // Update parent's narrower list
        if let Some(parent) = self.inner.get_concept_mut(&parent_id) {
            parent.narrower.push(concept_id.clone());
        }

        self.inner.add_concept(concept);
        Ok(concept_id)
    }

    /// Merge additional flat labels into this collection, skipping duplicates.
    ///
    /// A label is considered a duplicate if its deterministic concept ID
    /// (uuid5 of collection_id + label) already exists in the collection.
    ///
    /// Returns the number of new concepts added.
    ///
    /// Args:
    ///     labels: List of labels (strings or {lang: label} dicts)
    ///
    /// Example:
    ///     collection = RustRdmCollection.from_labels("Colors", ["Red", "Green"])
    ///     added = collection.update_from_labels(["Green", "Blue"])
    ///     # added == 1 (Green skipped, Blue added)
    #[pyo3(signature = (labels,))]
    fn update_from_labels(&mut self, py: Python, labels: Vec<PyObject>) -> PyResult<usize> {
        let mut added = 0;
        for label in labels {
            let label_string = extract_label_string(py, &label)?;
            let concept_id = generate_concept_id(&self.inner.id, &label_string)?;
            if self.inner.has_concept(&concept_id) {
                continue;
            }
            self.add_from_label(py, label, None)?;
            added += 1;
        }
        Ok(added)
    }

    /// Merge additional hierarchical entries into this collection, skipping duplicates.
    ///
    /// Walks the nested dict structure. For each label:
    /// - If its deterministic concept ID already exists, skips creation but still
    ///   recurses into children (so new children of existing parents get added).
    /// - If it doesn't exist, creates it as top-level or as child of its parent.
    ///
    /// Returns the number of new concepts added.
    ///
    /// Args:
    ///     structure: Nested dict where keys are labels, values are None (leaf) or dict (children)
    ///
    /// Example:
    ///     col = RustRdmCollection.from_nested_labels("Animals", {"Mammals": {"Dogs": None}})
    ///     added = col.update_from_nested_labels({"Mammals": {"Cats": None}, "Birds": None})
    ///     # added == 2 (Mammals skipped, Cats and Birds added)
    #[pyo3(signature = (structure,))]
    fn update_from_nested_labels(
        &mut self,
        py: Python,
        structure: &Bound<'_, PyDict>,
    ) -> PyResult<usize> {
        fn merge_nested(
            py: Python,
            collection: &mut RdmCollection,
            structure: &Bound<'_, PyDict>,
            parent_id: Option<String>,
        ) -> PyResult<usize> {
            let mut added = 0;
            for (key, value) in structure.iter() {
                let label: PyObject = key.into_py(py);
                let label_string = extract_label_string(py, &label)?;
                let concept_id = generate_concept_id(&collection.inner.id, &label_string)?;

                if collection.inner.has_concept(&concept_id) {
                    // Already exists — recurse for potential new children
                    if let Ok(children_dict) = value.downcast::<PyDict>() {
                        added += merge_nested(py, collection, children_dict, Some(concept_id))?;
                    }
                } else {
                    // New concept
                    if let Some(ref pid) = parent_id {
                        collection.add_child_from_label(py, pid.clone(), label, None)?;
                    } else {
                        collection.add_from_label(py, label, None)?;
                    }
                    added += 1;

                    if let Ok(children_dict) = value.downcast::<PyDict>() {
                        added += merge_nested(py, collection, children_dict, Some(concept_id))?;
                    }
                }
            }
            Ok(added)
        }

        merge_nested(py, self, structure, None)
    }

    /// Look up a concept by ID
    fn get_concept(&self, concept_id: &str) -> Option<RdmConcept> {
        self.inner
            .get_concept(concept_id)
            .cloned()
            .map(RdmConcept::from_core)
    }

    /// Get the first parent ID for a concept (from broader field)
    ///
    /// Returns None if the concept doesn't exist or has no parent (top-level concept).
    fn get_parent_id(&self, concept_id: &str) -> Option<String> {
        self.inner.get_parent_id(concept_id)
    }

    // =========================================================================
    // Value ID Lookups (for StaticValue compatibility)
    // =========================================================================

    /// Look up a value by its VALUE ID
    ///
    /// This is the primary lookup method used by ViewModels.
    /// Returns a dict with {id, value, conceptId, language} or None if not found.
    fn get_value_by_id(&self, py: Python, value_id: &str) -> PyResult<Option<PyObject>> {
        match self.inner.get_value_by_id(value_id) {
            Some(value) => {
                let dict = pyo3::types::PyDict::new_bound(py);
                dict.set_item("id", &value.id)?;
                dict.set_item("value", &value.value)?;
                dict.set_item("conceptId", &value.concept_id)?;
                dict.set_item("language", &value.language)?;
                Ok(Some(dict.into()))
            }
            None => Ok(None),
        }
    }

    /// Get concept ID from value ID
    ///
    /// Returns the concept ID that contains the given value ID.
    fn get_concept_id_for_value(&self, value_id: &str) -> Option<String> {
        self.inner
            .get_concept_id_for_value(value_id)
            .map(|s| s.to_string())
    }

    /// Check if a value ID exists in this collection
    fn has_value(&self, value_id: &str) -> bool {
        self.inner.has_value(value_id)
    }

    /// Get all value IDs in this collection
    fn get_value_ids(&self) -> Vec<String> {
        self.inner.get_value_ids().into_iter().cloned().collect()
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
        self.inner
            .get_top_concepts()
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
        self.inner
            .search(query, language)
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
        self.inner
            .find_by_label(label)
            .cloned()
            .map(RdmConcept::from_core)
    }

    /// Find a concept by exact label match, returning all matches
    fn find_all_by_label(&self, label: &str) -> Vec<RdmConcept> {
        self.inner
            .find_all_by_label(label)
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
    /// The output format depends on node_type:
    /// - "ConceptScheme": Uses skos:ConceptScheme with narrower/broader relations
    /// - "Collection": Uses skos:Collection with member relations (Arches-compatible)
    ///
    /// Args:
    ///     base_uri: Base URI for the SKOS resources (e.g., "http://example.org/")
    ///
    /// Returns:
    ///     SKOS RDF/XML as a string
    ///
    /// Example:
    ///     # Create a ConceptScheme (default)
    ///     collection = RustRdmCollection.from_labels("Test", ["A", "B", "C"])
    ///     xml = collection.to_skos_xml("http://example.org/")
    ///
    ///     # Create an Arches-compatible Collection
    ///     collection = RustRdmCollection.from_labels("Test", ["A", "B"], node_type="Collection")
    ///     xml = collection.to_skos_xml("http://example.org/")  # Uses skos:Collection
    fn to_skos_xml(&self, base_uri: &str) -> PyResult<String> {
        // Convert RdmCollection to SkosCollection with appropriate node_type
        let skos_collection = rdm_to_skos_collection(&self.inner, &self.node_type);

        // Serialize to XML using core function
        Ok(collection_to_skos_xml(&skos_collection, base_uri))
    }

    /// Export collection to simplified JSON format (Arches prebuild compatible).
    ///
    /// This produces a JSON structure matching the format used by arches-orm's
    /// `export_simplified_collection()` method, suitable for prebuild reference data.
    ///
    /// Format:
    /// {
    ///     "id": "collection-id",
    ///     "prefLabels": {"": {"value": "collection-name"}},
    ///     "concepts": {
    ///         "concept-id": {
    ///             "id": "concept-id",
    ///             "prefLabels": {"en": {"id": "value-id", "value": "Label"}},
    ///             "source": null,
    ///             "sortOrder": null,
    ///             "children": [...]  // if any
    ///         }
    ///     }
    /// }
    ///
    /// Example:
    ///     collection = RustRdmCollection.from_nested_labels("Animals", {"Mammals": {"Dogs": None}})
    ///     json_str = collection.to_simplified_json()
    ///     with open(f"{collection.id}.json", "w") as f:
    ///         f.write(json_str)
    fn to_simplified_json(&self) -> PyResult<String> {
        let mut concepts_map = serde_json::Map::new();

        // Build concept tree for top-level concepts only
        // Sort by ID for deterministic output
        let mut top_concepts: Vec<_> = self.inner.get_top_concepts();
        top_concepts.sort_by_key(|c| &c.id);
        for concept in top_concepts {
            let concept_json = self.concept_to_simplified_json(concept);
            concepts_map.insert(concept.id.clone(), concept_json);
        }

        // Build collection prefLabels
        let mut pref_labels = serde_json::Map::new();
        let mut empty_label = serde_json::Map::new();
        empty_label.insert(
            "value".to_string(),
            serde_json::Value::String(
                self.inner
                    .name
                    .clone()
                    .unwrap_or_else(|| self.inner.id.clone()),
            ),
        );
        pref_labels.insert("".to_string(), serde_json::Value::Object(empty_label));

        let result = serde_json::json!({
            "id": self.inner.id,
            "prefLabels": pref_labels,
            "concepts": concepts_map
        });

        serde_json::to_string_pretty(&result).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize to JSON: {}",
                e
            ))
        })
    }
}

impl RdmCollection {
    /// Helper to convert a concept to simplified JSON format (recursive for children)
    fn concept_to_simplified_json(&self, concept: &CoreRdmConcept) -> serde_json::Value {
        let mut pref_labels = serde_json::Map::new();
        // Sort languages for deterministic output
        let mut sorted_labels: Vec<_> = concept.pref_label.iter().collect();
        sorted_labels.sort_by_key(|(lang, _)| *lang);
        for (lang, rdm_value) in sorted_labels {
            // Use the existing value ID from RdmValue, or generate one
            let value_id = if rdm_value.id.is_empty() || rdm_value.id == "__pending__" {
                generate_skos_value_id(&concept.id, lang, &rdm_value.value)
            } else {
                rdm_value.id.clone()
            };
            let mut label_obj = serde_json::Map::new();
            label_obj.insert("id".to_string(), serde_json::Value::String(value_id));
            label_obj.insert(
                "value".to_string(),
                serde_json::Value::String(rdm_value.value.clone()),
            );
            pref_labels.insert(lang.clone(), serde_json::Value::Object(label_obj));
        }

        let mut concept_obj = serde_json::json!({
            "id": concept.id,
            "prefLabels": pref_labels,
            "source": serde_json::Value::Null,
            "sortOrder": serde_json::Value::Null
        });

        // Add children if any
        if !concept.narrower.is_empty() {
            // Sort children by ID for deterministic output
            let mut sorted_narrower: Vec<_> = concept.narrower.iter().collect();
            sorted_narrower.sort();
            let children: Vec<serde_json::Value> = sorted_narrower
                .iter()
                .filter_map(|child_id| self.inner.get_concept(child_id))
                .map(|child| self.concept_to_simplified_json(child))
                .collect();

            if !children.is_empty() {
                concept_obj
                    .as_object_mut()
                    .unwrap()
                    .insert("children".to_string(), serde_json::Value::Array(children));
            }
        }

        concept_obj
    }
}

// =============================================================================
// RDM ↔ SKOS Conversion
// =============================================================================

/// Generate a value ID for SKOS format (delegates to rdm_namespace UUID5).
fn generate_skos_value_id(concept_id: &str, lang: &str, value: &str) -> String {
    alizarin_core::rdm_namespace::generate_value_uuid(concept_id, value, lang).to_string()
}

/// Convert RdmCollection to SkosCollection for serialization
fn rdm_to_skos_collection(rdm: &CoreRdmCollection, node_type: &str) -> SkosCollection {
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
            for (lang, rdm_value) in &rdm_concept.pref_label {
                // Use existing value ID from RdmValue, or generate one
                let value_id = if rdm_value.id.is_empty() || rdm_value.id == "__pending__" {
                    generate_skos_value_id(concept_id, lang, &rdm_value.value)
                } else {
                    rdm_value.id.clone()
                };
                pref_labels.insert(
                    lang.clone(),
                    SkosValue {
                        id: value_id,
                        value: rdm_value.value.clone(),
                    },
                );
            }

            let skos_concept = SkosConcept {
                id: concept_id.clone(),
                uri: None, // No URI in RDM format
                pref_labels,
                source: Some(concept_id.clone()),
                sort_order: None, // No sort order in RDM format
                children: None,   // Will be built hierarchically later (for ConceptScheme only)
            };

            all_skos_concepts.insert(concept_id.clone(), skos_concept);

            // Collect all narrower IDs (only relevant for ConceptScheme)
            all_narrower_ids.extend(rdm_concept.narrower.iter().cloned());
        }
    }

    // Determine node type
    let skos_node_type = if node_type == "Collection" {
        SkosNodeType::Collection
    } else {
        SkosNodeType::ConceptScheme
    };

    // Build hierarchy for both ConceptScheme and Collection types
    // (Collections use member for membership but concepts can have narrower/broader)
    let mut hierarchy: HashMap<String, SkosConcept> = HashMap::new();

    for concept_id in rdm.get_concept_ids() {
        if !all_narrower_ids.contains(concept_id) {
            // This is a top-level concept
            if let Some(concept_with_children) =
                build_concept_tree(concept_id, &all_skos_concepts, rdm)
            {
                hierarchy.insert(concept_id.clone(), concept_with_children);
            }
        }
    }

    // If no hierarchy, just use all concepts as top-level
    let top_level_concepts = if hierarchy.is_empty() {
        all_skos_concepts.clone()
    } else {
        hierarchy
    };

    SkosCollection {
        id: rdm.id.clone(),
        uri: None,
        pref_labels: collection_pref_labels,
        alt_labels: HashMap::new(),
        scope_notes: HashMap::new(),
        node_type: skos_node_type,
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
            skos.pref_labels
                .get("en")
                .map(|v| v.value.clone())
                .unwrap_or_else(|| skos.id.clone()),
        );

        // Recursive function to add a concept and its children
        // parent_id is None for top-level concepts
        fn add_concept_recursive(
            rdm: &mut CoreRdmCollection,
            skos_concept: &SkosConcept,
            parent_id: Option<&str>,
        ) {
            let mut pref_label: HashMap<String, CoreRdmValue> = HashMap::new();
            for (lang, skos_value) in &skos_concept.pref_labels {
                // Use the existing value ID from SKOS
                pref_label.insert(
                    lang.clone(),
                    CoreRdmValue::new(skos_value.id.clone(), skos_value.value.clone()),
                );
            }

            // Extract narrower IDs from children
            let narrower: Vec<String> = skos_concept
                .children
                .as_ref()
                .map(|children| children.iter().map(|c| c.id.clone()).collect())
                .unwrap_or_default();

            // Set broader to parent if this is a child concept
            let broader = parent_id.map(|p| vec![p.to_string()]).unwrap_or_default();

            let rdm_concept = CoreRdmConcept {
                id: skos_concept.id.clone(),
                pref_label,
                alt_labels: HashMap::new(),
                broader,
                narrower,
                scope_note: HashMap::new(),
            };

            rdm.add_concept(rdm_concept);

            // Recursively add children with this concept as parent
            if let Some(ref children) = skos_concept.children {
                for child in children {
                    add_concept_recursive(rdm, child, Some(&skos_concept.id));
                }
            }
        }

        // Add all top-level concepts (and their children recursively)
        for skos_concept in skos.concepts.values() {
            add_concept_recursive(&mut rdm, skos_concept, None);
        }

        // Also add from all_concepts if not already added (fallback for flat structures)
        if rdm.is_empty() && !skos.all_concepts.is_empty() {
            for skos_concept in skos.all_concepts.values() {
                if !rdm.has_concept(&skos_concept.id) {
                    add_concept_recursive(&mut rdm, skos_concept, None);
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
    fn add_collection_from_json(
        &mut self,
        collection_id: &str,
        concepts_json: &str,
    ) -> PyResult<()> {
        self.inner
            .add_collection_from_json(collection_id, concepts_json)
            .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
    }

    /// Add a pre-built collection
    fn add_collection(&mut self, collection: RdmCollection) {
        self.inner.add_collection(collection.into_inner());
    }

    /// Merge additional flat labels into an existing collection in the cache.
    ///
    /// Retrieves the collection, adds new labels (skipping duplicates), and
    /// stores it back. No clone-and-replace needed from Python.
    ///
    /// Args:
    ///     collection_id: ID of the collection to update
    ///     labels: List of labels (strings or {lang: label} dicts) to add
    ///
    /// Returns:
    ///     Number of new concepts added
    ///
    /// Raises:
    ///     ValueError: If collection_id is not in the cache
    #[pyo3(signature = (collection_id, labels))]
    fn update_collection(
        &mut self,
        py: Python,
        collection_id: String,
        labels: Vec<PyObject>,
    ) -> PyResult<usize> {
        // Clone out, mutate via the Python wrapper, put back
        let core_coll = self
            .inner
            .get_collection(&collection_id)
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Collection '{}' not found in cache",
                    collection_id
                ))
            })?
            .clone();
        let mut wrapper = RdmCollection::from_core(core_coll);
        let added = wrapper.update_from_labels(py, labels)?;
        self.inner.add_collection(wrapper.into_inner());
        Ok(added)
    }

    /// Merge additional hierarchical entries into an existing collection in the cache.
    ///
    /// Retrieves the collection, adds new entries (skipping duplicates, recursing
    /// into children of existing parents), and stores it back.
    ///
    /// Args:
    ///     collection_id: ID of the collection to update
    ///     structure: Nested dict where keys are labels, values are None (leaf) or dict (children)
    ///
    /// Returns:
    ///     Number of new concepts added
    ///
    /// Raises:
    ///     ValueError: If collection_id is not in the cache
    #[pyo3(signature = (collection_id, structure))]
    fn update_collection_nested(
        &mut self,
        py: Python,
        collection_id: String,
        structure: &Bound<'_, PyDict>,
    ) -> PyResult<usize> {
        let core_coll = self
            .inner
            .get_collection(&collection_id)
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Collection '{}' not found in cache",
                    collection_id
                ))
            })?
            .clone();
        let mut wrapper = RdmCollection::from_core(core_coll);
        let added = wrapper.update_from_nested_labels(py, structure)?;
        self.inner.add_collection(wrapper.into_inner());
        Ok(added)
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
        let skos_collections = parse_skos_to_collections(xml_content, base_uri).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse SKOS XML: {}",
                e
            ))
        })?;

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
        self.inner
            .get_collection(collection_id)
            .cloned()
            .map(RdmCollection::from_core)
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
        self.inner
            .lookup_concept(collection_id, concept_id)
            .cloned()
            .map(RdmConcept::from_core)
    }

    /// Get the first parent ID for a concept
    ///
    /// Returns None if the collection doesn't exist, concept doesn't exist,
    /// or concept has no parent (top-level concept).
    fn get_parent_id(&self, collection_id: &str, concept_id: &str) -> Option<String> {
        self.inner.get_parent_id(collection_id, concept_id)
    }

    // =========================================================================
    // Value ID Lookups (for StaticValue compatibility)
    // =========================================================================

    /// Look up a value by its VALUE ID
    ///
    /// This is the primary lookup method used by ViewModels.
    /// Returns a dict with {id, value, conceptId, language} or None if not found.
    fn lookup_value(
        &self,
        py: Python,
        collection_id: &str,
        value_id: &str,
    ) -> PyResult<Option<PyObject>> {
        match self.inner.lookup_value(collection_id, value_id) {
            Some(value) => {
                let dict = pyo3::types::PyDict::new_bound(py);
                dict.set_item("id", &value.id)?;
                dict.set_item("value", &value.value)?;
                dict.set_item("conceptId", &value.concept_id)?;
                dict.set_item("language", &value.language)?;
                Ok(Some(dict.into()))
            }
            None => Ok(None),
        }
    }

    /// Get concept ID from value ID
    ///
    /// Returns the concept ID that contains the given value ID.
    fn get_concept_id_for_value(&self, collection_id: &str, value_id: &str) -> Option<String> {
        self.inner
            .get_concept_id_for_value(collection_id, value_id)
            .map(|s| s.to_string())
    }

    /// Validate that a value exists in a collection
    fn validate_value(&self, collection_id: &str, value_id: &str) -> bool {
        self.inner.validate_value(collection_id, value_id)
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
        self.inner
            .lookup_by_label(collection_id, label)
            .cloned()
            .map(RdmConcept::from_core)
    }

    /// Look up a concept by label, returning all matches
    fn lookup_all_by_label(&self, collection_id: &str, label: &str) -> Vec<RdmConcept> {
        self.inner
            .lookup_all_by_label(collection_id, label)
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
        self.inner
            .search_all(query, language)
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
        self.inner
            .lookup_by_label(collection_id, label)
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
    let tree: serde_json::Value = serde_json::from_str(tree_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid tree JSON: {}", e))
    })?;

    let graph: serde_json::Value = serde_json::from_str(graph_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid graph JSON: {}", e))
    })?;

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
    m.add_function(wrap_pyfunction!(add_collection_to_global_cache, m)?)?;
    m.add_function(wrap_pyfunction!(add_from_skos_xml_to_global_cache, m)?)?;
    m.add_function(wrap_pyfunction!(update_collection_in_global_cache, m)?)?;
    m.add_function(wrap_pyfunction!(
        update_collection_nested_in_global_cache,
        m
    )?)?;

    // Global RDM namespace functions for deterministic UUID generation
    m.add_function(wrap_pyfunction!(set_rdm_namespace, m)?)?;
    m.add_function(wrap_pyfunction!(get_rdm_namespace, m)?)?;
    m.add_function(wrap_pyfunction!(has_rdm_namespace, m)?)?;
    m.add_function(wrap_pyfunction!(clear_rdm_namespace, m)?)?;

    // Label resolution functions
    m.add_function(wrap_pyfunction!(resolve_labels, m)?)?;
    m.add_function(wrap_pyfunction!(get_needed_collections, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_uuid, m)?)?;

    Ok(())
}
