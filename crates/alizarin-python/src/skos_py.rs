//! Python bindings for SKOS parsing functions.
//!
//! Provides PyO3 bindings for SKOS RDF/XML parsing and serialization,
//! matching the WASM API for cross-platform consistency.

use pyo3::prelude::*;

use alizarin_core::skos::{
    parse_skos_to_collections, collection_to_skos_xml, collections_to_skos_xml,
    SkosCollection,
};

/// Parse SKOS RDF/XML and return collections as JSON string.
///
/// Args:
///     xml_content: SKOS RDF/XML content as string
///     base_uri: Base URI for resolving relative URIs
///
/// Returns:
///     JSON string containing array of SkosCollection objects
///
/// Raises:
///     ValueError: If XML parsing fails
#[pyfunction]
pub fn parse_skos_xml(xml_content: &str, base_uri: &str) -> PyResult<String> {
    let collections = parse_skos_to_collections(xml_content, base_uri)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    serde_json::to_string(&collections)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Serialization error: {}", e)
        ))
}

/// Parse SKOS RDF/XML and return a single collection as JSON string.
///
/// Args:
///     xml_content: SKOS RDF/XML content as string
///     base_uri: Base URI for resolving relative URIs
///
/// Returns:
///     JSON string containing a single SkosCollection object
///
/// Raises:
///     ValueError: If XML parsing fails or no ConceptScheme found
#[pyfunction]
pub fn parse_skos_xml_to_collection(xml_content: &str, base_uri: &str) -> PyResult<String> {
    let mut collections = parse_skos_to_collections(xml_content, base_uri)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    if collections.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "No SKOS ConceptScheme found in XML"
        ));
    }

    let collection = collections.remove(0);
    serde_json::to_string(&collection)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Serialization error: {}", e)
        ))
}

/// Serialize a SkosCollection to SKOS RDF/XML.
///
/// Args:
///     collection_json: JSON string containing a SkosCollection object
///     base_uri: Base URI for the output XML
///
/// Returns:
///     SKOS RDF/XML string
///
/// Raises:
///     ValueError: If JSON parsing fails
#[pyfunction]
pub fn skos_collection_to_xml(collection_json: &str, base_uri: &str) -> PyResult<String> {
    let collection: SkosCollection = serde_json::from_str(collection_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to deserialize collection: {}", e)
        ))?;

    Ok(collection_to_skos_xml(&collection, base_uri))
}

/// Serialize multiple SkosCollections to SKOS RDF/XML.
///
/// Args:
///     collections_json: JSON string containing array of SkosCollection objects
///     base_uri: Base URI for the output XML
///
/// Returns:
///     SKOS RDF/XML string
///
/// Raises:
///     ValueError: If JSON parsing fails
#[pyfunction]
pub fn skos_collections_to_xml(collections_json: &str, base_uri: &str) -> PyResult<String> {
    let collections: Vec<SkosCollection> = serde_json::from_str(collections_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to deserialize collections: {}", e)
        ))?;

    Ok(collections_to_skos_xml(&collections, base_uri))
}

/// Register SKOS functions with the Python module
pub fn register_module(m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_skos_xml, m)?)?;
    m.add_function(wrap_pyfunction!(parse_skos_xml_to_collection, m)?)?;
    m.add_function(wrap_pyfunction!(skos_collection_to_xml, m)?)?;
    m.add_function(wrap_pyfunction!(skos_collections_to_xml, m)?)?;
    Ok(())
}
