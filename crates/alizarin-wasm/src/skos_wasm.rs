//! WASM bindings for SKOS parsing functions.
//!
//! Re-exports core SKOS types and adds wasm-bindgen bindings.

use wasm_bindgen::prelude::*;

// Re-export core types
pub use alizarin_core::skos::{
    parse_skos_to_collections, collection_to_skos_xml, collections_to_skos_xml,
    SkosCollection, SkosConcept, SkosLabel, SkosValue,
};

/// Parse SKOS RDF/XML and return collections as JS value
#[wasm_bindgen(js_name = parseSkosXml)]
pub fn parse_skos_xml(xml_content: &str, base_uri: &str) -> Result<JsValue, JsValue> {
    let collections = parse_skos_to_collections(xml_content, base_uri)
        .map_err(|e| JsValue::from_str(&e))?;

    serde_wasm_bindgen::to_value(&collections)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Parse SKOS RDF/XML and return a single collection as JS value
#[wasm_bindgen(js_name = parseSkosXmlToCollection)]
pub fn parse_skos_xml_to_collection(xml_content: &str, base_uri: &str) -> Result<JsValue, JsValue> {
    let mut collections = parse_skos_to_collections(xml_content, base_uri)
        .map_err(|e| JsValue::from_str(&e))?;

    if collections.is_empty() {
        return Err(JsValue::from_str("No SKOS ConceptScheme found in XML"));
    }

    let collection = collections.remove(0);
    serde_wasm_bindgen::to_value(&collection)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Serialize a SkosCollection to SKOS RDF/XML (WASM binding)
#[wasm_bindgen(js_name = collectionToSkosXml)]
pub fn collection_to_skos_xml_wasm(collection_js: JsValue, base_uri: &str) -> Result<String, JsValue> {
    let collection: SkosCollection = serde_wasm_bindgen::from_value(collection_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize collection: {}", e)))?;

    Ok(collection_to_skos_xml(&collection, base_uri))
}

/// Serialize multiple SkosCollections to SKOS RDF/XML (WASM binding)
#[wasm_bindgen(js_name = collectionsToSkosXml)]
pub fn collections_to_skos_xml_wasm(collections_js: JsValue, base_uri: &str) -> Result<String, JsValue> {
    let collections: Vec<SkosCollection> = serde_wasm_bindgen::from_value(collections_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize collections: {}", e)))?;

    Ok(collections_to_skos_xml(&collections, base_uri))
}
