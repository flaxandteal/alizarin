//! SKOS RDF/XML Parser and Serializer
//!
//! Parses SKOS concept schemes from RDF/XML format into structures
//! compatible with Alizarin's StaticCollection format, and serializes
//! them back to SKOS RDF/XML.

use rio_api::model::{Literal, Subject, Term, Triple};
use rio_api::parser::TriplesParser;
use rio_xml::RdfXmlParser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

// SKOS namespace URIs (prefixes for documentation, not used in code)
#[allow(dead_code)]
const SKOS_NS: &str = "http://www.w3.org/2004/02/skos/core#";
#[allow(dead_code)]
const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
#[allow(dead_code)]
const DCTERMS_NS: &str = "http://purl.org/dc/terms/";

// SKOS predicates
const SKOS_CONCEPT: &str = "http://www.w3.org/2004/02/skos/core#Concept";
const SKOS_CONCEPT_SCHEME: &str = "http://www.w3.org/2004/02/skos/core#ConceptScheme";
const SKOS_PREF_LABEL: &str = "http://www.w3.org/2004/02/skos/core#prefLabel";
const SKOS_ALT_LABEL: &str = "http://www.w3.org/2004/02/skos/core#altLabel";
const SKOS_NARROWER: &str = "http://www.w3.org/2004/02/skos/core#narrower";
const SKOS_BROADER: &str = "http://www.w3.org/2004/02/skos/core#broader";
const SKOS_IN_SCHEME: &str = "http://www.w3.org/2004/02/skos/core#inScheme";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const DCTERMS_TITLE: &str = "http://purl.org/dc/terms/title";
const DCTERMS_IDENTIFIER: &str = "http://purl.org/dc/terms/identifier";

/// A label with language tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkosLabel {
    pub id: String,
    pub value: String,
    pub language_id: String,
    pub valuetype_id: String,
    pub list_item_id: String,
}

/// A parsed SKOS concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkosConcept {
    pub id: String,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(rename = "prefLabels")]
    pub pref_labels: HashMap<String, SkosValue>,
    pub source: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
    pub children: Option<Vec<SkosConcept>>,
}

/// A simple value with id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkosValue {
    pub id: String,
    pub value: String,
}

/// A parsed SKOS collection/concept scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkosCollection {
    pub id: String,
    #[serde(rename = "prefLabels")]
    pub pref_labels: HashMap<String, SkosValue>,
    pub concepts: HashMap<String, SkosConcept>,
    #[serde(rename = "__allConcepts")]
    pub all_concepts: HashMap<String, SkosConcept>,
    #[serde(rename = "__values")]
    pub values: HashMap<String, SkosValue>,
}

/// Internal structure for collecting triples during parsing
#[derive(Debug, Default)]
struct ParsedData {
    /// URI -> type (Concept or ConceptScheme)
    types: HashMap<String, String>,
    /// URI -> labels (predicate, value, language)
    labels: HashMap<String, Vec<(String, String, String)>>,
    /// URI -> narrower URIs
    narrower: HashMap<String, Vec<String>>,
    /// URI -> broader URIs
    broader: HashMap<String, Vec<String>>,
    /// URI -> scheme URIs
    in_scheme: HashMap<String, Vec<String>>,
    /// Scheme URI -> title
    scheme_titles: HashMap<String, String>,
    /// URI -> identifier
    identifiers: HashMap<String, String>,
    /// URI -> sort order
    sort_orders: HashMap<String, i32>,
}

/// Extract UUID from URI or generate a deterministic one
fn extract_or_generate_id(uri: &str) -> String {
    // Try to extract UUID from URI
    let uuid_regex = regex_lite::Regex::new(
        r"([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})"
    ).unwrap();

    if let Some(caps) = uuid_regex.captures(uri) {
        return caps.get(1).unwrap().as_str().to_string();
    }

    // Generate deterministic UUID from URI using simple hash
    // In production, use proper UUID v5
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    uri.hash(&mut hasher);
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

/// Generate a value ID
fn generate_value_id(concept_id: &str, lang: &str, value: &str) -> String {
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

impl ParsedData {
    fn process_triple(&mut self, triple: Triple) {
        let subject_uri = match triple.subject {
            Subject::NamedNode(n) => n.iri.to_string(),
            _ => return,
        };

        let predicate = triple.predicate.iri;

        match predicate {
            RDF_TYPE => {
                if let Term::NamedNode(obj) = triple.object {
                    self.types.insert(subject_uri, obj.iri.to_string());
                }
            }
            SKOS_PREF_LABEL | SKOS_ALT_LABEL => {
                if let Term::Literal(lit) = triple.object {
                    let (value, lang) = match lit {
                        Literal::Simple { value } => (value.to_string(), "en".to_string()),
                        Literal::LanguageTaggedString { value, language } => {
                            (value.to_string(), language.to_string())
                        }
                        Literal::Typed { value, .. } => (value.to_string(), "en".to_string()),
                    };
                    self.labels
                        .entry(subject_uri)
                        .or_default()
                        .push((predicate.to_string(), value, lang));
                }
            }
            SKOS_NARROWER => {
                if let Term::NamedNode(obj) = triple.object {
                    self.narrower
                        .entry(subject_uri)
                        .or_default()
                        .push(obj.iri.to_string());
                }
            }
            SKOS_BROADER => {
                if let Term::NamedNode(obj) = triple.object {
                    self.broader
                        .entry(subject_uri)
                        .or_default()
                        .push(obj.iri.to_string());
                }
            }
            SKOS_IN_SCHEME => {
                if let Term::NamedNode(obj) = triple.object {
                    self.in_scheme
                        .entry(subject_uri)
                        .or_default()
                        .push(obj.iri.to_string());
                }
            }
            DCTERMS_TITLE => {
                if let Term::Literal(lit) = triple.object {
                    let value = match lit {
                        Literal::Simple { value } => value.to_string(),
                        Literal::LanguageTaggedString { value, .. } => value.to_string(),
                        Literal::Typed { value, .. } => value.to_string(),
                    };
                    self.scheme_titles.insert(subject_uri, value);
                }
            }
            DCTERMS_IDENTIFIER => {
                if let Term::Literal(lit) = triple.object {
                    let value = match lit {
                        Literal::Simple { value } => value.to_string(),
                        Literal::LanguageTaggedString { value, .. } => value.to_string(),
                        Literal::Typed { value, .. } => value.to_string(),
                    };
                    self.identifiers.insert(subject_uri, value);
                }
            }
            _ => {
                // Check for sort order (various possible predicates)
                if predicate.contains("sortorder") || predicate.contains("sortOrder") {
                    if let Term::Literal(lit) = triple.object {
                        let value = match lit {
                            Literal::Simple { value } => value,
                            Literal::LanguageTaggedString { value, .. } => value,
                            Literal::Typed { value, .. } => value,
                        };
                        if let Ok(order) = value.parse::<i32>() {
                            self.sort_orders.insert(subject_uri, order);
                        }
                    }
                }
            }
        }
    }
}

/// Parse SKOS RDF/XML and return collections as JSON string
#[wasm_bindgen(js_name = parseSkosXml)]
pub fn parse_skos_xml(xml_content: &str, base_uri: &str) -> Result<JsValue, JsValue> {
    let collections = parse_skos_to_collections(xml_content, base_uri)
        .map_err(|e| JsValue::from_str(&e))?;

    serde_wasm_bindgen::to_value(&collections)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Parse SKOS RDF/XML and return a single collection as JSON string
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

/// Internal parsing function
pub fn parse_skos_to_collections(xml_content: &str, base_uri: &str) -> Result<Vec<SkosCollection>, String> {
    let mut data = ParsedData::default();

    // Parse base URI into Iri
    let base_iri = oxiri::Iri::parse(base_uri.to_string())
        .map_err(|e| format!("Invalid base URI: {}", e))?;

    // Parse RDF/XML
    let mut parser = RdfXmlParser::new(xml_content.as_bytes(), Some(base_iri));

    parser.parse_all(&mut |triple| {
        data.process_triple(triple);
        Ok(()) as Result<(), std::io::Error>
    }).map_err(|e| format!("RDF/XML parse error: {}", e))?;

    // Find all concept schemes
    let scheme_uris: Vec<String> = data.types
        .iter()
        .filter(|(_, t)| *t == SKOS_CONCEPT_SCHEME)
        .map(|(uri, _)| uri.clone())
        .collect();

    // Find all concepts
    let concept_uris: Vec<String> = data.types
        .iter()
        .filter(|(_, t)| *t == SKOS_CONCEPT)
        .map(|(uri, _)| uri.clone())
        .collect();

    // Build concept map
    let mut all_concepts: HashMap<String, SkosConcept> = HashMap::new();

    for uri in &concept_uris {
        let id = extract_or_generate_id(uri);

        // Build prefLabels
        let mut pref_labels: HashMap<String, SkosValue> = HashMap::new();
        if let Some(labels) = data.labels.get(uri) {
            for (pred, value, lang) in labels {
                if pred == SKOS_PREF_LABEL {
                    pref_labels.insert(lang.clone(), SkosValue {
                        id: generate_value_id(&id, lang, value),
                        value: value.clone(),
                    });
                }
            }
        }

        // If no prefLabel, use first available label
        if pref_labels.is_empty() {
            if let Some(labels) = data.labels.get(uri) {
                if let Some((_, value, lang)) = labels.first() {
                    pref_labels.insert(lang.clone(), SkosValue {
                        id: generate_value_id(&id, lang, value),
                        value: value.clone(),
                    });
                }
            }
        }

        let concept = SkosConcept {
            id: id.clone(),
            uri: Some(uri.clone()),
            pref_labels,
            source: data.identifiers.get(uri).cloned().or(Some(uri.clone())),
            sort_order: data.sort_orders.get(uri).copied(),
            children: None,
        };

        all_concepts.insert(uri.clone(), concept);
    }

    // Build hierarchy - find narrower relationships
    // Use HashSet to deduplicate (both narrower and broader can specify the same relationship)
    let mut children_map: HashMap<String, HashSet<String>> = HashMap::new();
    for (parent_uri, narrower_uris) in &data.narrower {
        children_map
            .entry(parent_uri.clone())
            .or_default()
            .extend(narrower_uris.iter().cloned());
    }

    // Also build from broader relationships
    for (child_uri, broader_uris) in &data.broader {
        for parent_uri in broader_uris {
            children_map
                .entry(parent_uri.clone())
                .or_default()
                .insert(child_uri.clone());
        }
    }

    // Convert HashSet to Vec for the rest of the processing
    let children_map: HashMap<String, Vec<String>> = children_map
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect();

    // Find top-level concepts (not narrower of anything)
    let all_narrower: HashSet<String> = children_map.values().flatten().cloned().collect();

    // Recursive function to build concept with children
    fn build_concept_tree(
        uri: &str,
        all_concepts: &HashMap<String, SkosConcept>,
        children_map: &HashMap<String, Vec<String>>,
        sort_orders: &HashMap<String, i32>,
    ) -> Option<SkosConcept> {
        let concept = all_concepts.get(uri)?;
        let mut result = concept.clone();

        if let Some(child_uris) = children_map.get(uri) {
            let mut children: Vec<SkosConcept> = child_uris
                .iter()
                .filter_map(|child_uri| {
                    build_concept_tree(child_uri, all_concepts, children_map, sort_orders)
                })
                .collect();

            // Sort children by sort_order
            children.sort_by(|a, b| {
                a.sort_order.unwrap_or(999).cmp(&b.sort_order.unwrap_or(999))
            });

            if !children.is_empty() {
                result.children = Some(children);
            }
        }

        Some(result)
    }

    // Build collections
    let mut collections: Vec<SkosCollection> = Vec::new();

    for scheme_uri in &scheme_uris {
        let scheme_id = extract_or_generate_id(scheme_uri);
        let title = data.scheme_titles.get(scheme_uri)
            .cloned()
            .unwrap_or_else(|| scheme_id.clone());

        // Find concepts in this scheme
        let scheme_concept_uris: Vec<&String> = data.in_scheme
            .iter()
            .filter(|(_, schemes)| schemes.contains(scheme_uri))
            .map(|(concept_uri, _)| concept_uri)
            .collect();

        // Find top-level concepts for this scheme
        let top_level: Vec<String> = scheme_concept_uris
            .iter()
            .filter(|uri| !all_narrower.contains(**uri))
            .map(|uri| (*uri).clone())
            .collect();

        // Build concept trees
        let mut concepts: HashMap<String, SkosConcept> = HashMap::new();
        for uri in &top_level {
            if let Some(concept) = build_concept_tree(uri, &all_concepts, &children_map, &data.sort_orders) {
                concepts.insert(concept.id.clone(), concept);
            }
        }

        let mut pref_labels = HashMap::new();
        pref_labels.insert("en".to_string(), SkosValue {
            id: generate_value_id(&scheme_id, "en", &title),
            value: title,
        });

        collections.push(SkosCollection {
            id: scheme_id,
            pref_labels,
            concepts,
            all_concepts: HashMap::new(),
            values: HashMap::new(),
        });
    }

    // If no schemes found, create default collection with all concepts
    if collections.is_empty() && !all_concepts.is_empty() {
        let top_level: Vec<String> = concept_uris
            .iter()
            .filter(|uri| !all_narrower.contains(*uri))
            .cloned()
            .collect();

        let mut concepts: HashMap<String, SkosConcept> = HashMap::new();
        for uri in &top_level {
            if let Some(concept) = build_concept_tree(uri, &all_concepts, &children_map, &data.sort_orders) {
                concepts.insert(concept.id.clone(), concept);
            }
        }

        let default_id = extract_or_generate_id(base_uri);
        let mut pref_labels = HashMap::new();
        pref_labels.insert("en".to_string(), SkosValue {
            id: generate_value_id(&default_id, "en", "Imported Concepts"),
            value: "Imported Concepts".to_string(),
        });

        collections.push(SkosCollection {
            id: default_id,
            pref_labels,
            concepts,
            all_concepts: HashMap::new(),
            values: HashMap::new(),
        });
    }

    Ok(collections)
}

// ============================================================================
// SKOS XML Writer
// ============================================================================

/// XML escape helper
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Write a concept and its children recursively to XML
fn write_concept_xml(
    concept: &SkosConcept,
    scheme_uri: &str,
    base_uri: &str,
    parent_uri: Option<&str>,
    output: &mut String,
) {
    let concept_uri = match &concept.uri {
        Some(uri) if !uri.is_empty() => uri.clone(),
        _ => format!("{}{}", base_uri, concept.id),
    };

    output.push_str(&format!(
        "  <skos:Concept rdf:about=\"{}\">\n",
        xml_escape(&concept_uri)
    ));

    // Add inScheme
    output.push_str(&format!(
        "    <skos:inScheme rdf:resource=\"{}\"/>\n",
        xml_escape(scheme_uri)
    ));

    // Add broader (parent) relationship if this is a child concept
    if let Some(parent) = parent_uri {
        output.push_str(&format!(
            "    <skos:broader rdf:resource=\"{}\"/>\n",
            xml_escape(parent)
        ));
    }

    // Add prefLabels
    for (lang, value) in &concept.pref_labels {
        output.push_str(&format!(
            "    <skos:prefLabel xml:lang=\"{}\">{}</skos:prefLabel>\n",
            xml_escape(lang),
            xml_escape(&value.value)
        ));
    }

    // Add sort order if present
    if let Some(order) = concept.sort_order {
        output.push_str(&format!(
            "    <arches:sortorder rdf:datatype=\"http://www.w3.org/2001/XMLSchema#integer\">{}</arches:sortorder>\n",
            order
        ));
    }

    // Add identifier/source if present
    if let Some(ref source) = concept.source {
        output.push_str(&format!(
            "    <dcterms:identifier>{}</dcterms:identifier>\n",
            xml_escape(source)
        ));
    }

    // Add narrower relationships for children
    if let Some(ref children) = concept.children {
        for child in children {
            let child_uri = match &child.uri {
                Some(uri) if !uri.is_empty() => uri.clone(),
                _ => format!("{}{}", base_uri, child.id),
            };
            output.push_str(&format!(
                "    <skos:narrower rdf:resource=\"{}\"/>\n",
                xml_escape(&child_uri)
            ));
        }
    }

    output.push_str("  </skos:Concept>\n");

    // Recursively write children
    if let Some(ref children) = concept.children {
        for child in children {
            write_concept_xml(child, scheme_uri, base_uri, Some(&concept_uri), output);
        }
    }
}

/// Serialize a SkosCollection to SKOS RDF/XML format
pub fn collection_to_skos_xml(collection: &SkosCollection, base_uri: &str) -> String {
    let mut output = String::new();

    // XML declaration and RDF root with namespaces
    output.push_str(r#"<?xml version="1.0" encoding="utf-8"?>
<rdf:RDF
  xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
  xmlns:skos="http://www.w3.org/2004/02/skos/core#"
  xmlns:dcterms="http://purl.org/dc/terms/"
  xmlns:arches="http://localhost:8000/"
>
"#);

    // Build scheme URI
    let scheme_uri = format!("{}{}", base_uri, collection.id);

    // Write ConceptScheme
    output.push_str(&format!(
        "  <skos:ConceptScheme rdf:about=\"{}\">\n",
        xml_escape(&scheme_uri)
    ));

    // Add title from prefLabels
    for (lang, value) in &collection.pref_labels {
        output.push_str(&format!(
            "    <dcterms:title xml:lang=\"{}\">{}</dcterms:title>\n",
            xml_escape(lang),
            xml_escape(&value.value)
        ));
    }

    output.push_str("  </skos:ConceptScheme>\n");

    // Write all top-level concepts (and their children recursively)
    for concept in collection.concepts.values() {
        write_concept_xml(concept, &scheme_uri, base_uri, None, &mut output);
    }

    output.push_str("</rdf:RDF>\n");

    output
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

    let mut output = String::new();

    // XML declaration and RDF root with namespaces
    output.push_str(r#"<?xml version="1.0" encoding="utf-8"?>
<rdf:RDF
  xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
  xmlns:skos="http://www.w3.org/2004/02/skos/core#"
  xmlns:dcterms="http://purl.org/dc/terms/"
  xmlns:arches="http://localhost:8000/"
>
"#);

    for collection in &collections {
        let scheme_uri = format!("{}{}", base_uri, collection.id);

        // Write ConceptScheme
        output.push_str(&format!(
            "  <skos:ConceptScheme rdf:about=\"{}\">\n",
            xml_escape(&scheme_uri)
        ));

        for (lang, value) in &collection.pref_labels {
            output.push_str(&format!(
                "    <dcterms:title xml:lang=\"{}\">{}</dcterms:title>\n",
                xml_escape(lang),
                xml_escape(&value.value)
            ));
        }

        output.push_str("  </skos:ConceptScheme>\n");

        // Write concepts
        for concept in collection.concepts.values() {
            write_concept_xml(concept, &scheme_uri, base_uri, None, &mut output);
        }
    }

    output.push_str("</rdf:RDF>\n");

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SKOS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:skos="http://www.w3.org/2004/02/skos/core#"
         xmlns:dcterms="http://purl.org/dc/terms/">
  <skos:ConceptScheme rdf:about="http://example.org/scheme/1">
    <dcterms:title>Test Scheme</dcterms:title>
  </skos:ConceptScheme>
  <skos:Concept rdf:about="http://example.org/concept/1">
    <skos:inScheme rdf:resource="http://example.org/scheme/1"/>
    <skos:prefLabel xml:lang="en">Concept One</skos:prefLabel>
  </skos:Concept>
</rdf:RDF>"#;

    const TEST_SKOS_WITH_HIERARCHY: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:skos="http://www.w3.org/2004/02/skos/core#"
         xmlns:dcterms="http://purl.org/dc/terms/"
         xmlns:arches="http://localhost:8000/">
  <skos:ConceptScheme rdf:about="http://example.org/scheme/test-scheme">
    <dcterms:title>Hierarchical Test</dcterms:title>
  </skos:ConceptScheme>
  <skos:Concept rdf:about="http://example.org/concept/parent">
    <skos:inScheme rdf:resource="http://example.org/scheme/test-scheme"/>
    <skos:prefLabel xml:lang="en">Parent Concept</skos:prefLabel>
    <skos:prefLabel xml:lang="de">Elternkonzept</skos:prefLabel>
    <arches:sortorder rdf:datatype="http://www.w3.org/2001/XMLSchema#integer">0</arches:sortorder>
    <skos:narrower rdf:resource="http://example.org/concept/child1"/>
    <skos:narrower rdf:resource="http://example.org/concept/child2"/>
  </skos:Concept>
  <skos:Concept rdf:about="http://example.org/concept/child1">
    <skos:inScheme rdf:resource="http://example.org/scheme/test-scheme"/>
    <skos:broader rdf:resource="http://example.org/concept/parent"/>
    <skos:prefLabel xml:lang="en">Child One</skos:prefLabel>
    <arches:sortorder rdf:datatype="http://www.w3.org/2001/XMLSchema#integer">1</arches:sortorder>
  </skos:Concept>
  <skos:Concept rdf:about="http://example.org/concept/child2">
    <skos:inScheme rdf:resource="http://example.org/scheme/test-scheme"/>
    <skos:broader rdf:resource="http://example.org/concept/parent"/>
    <skos:prefLabel xml:lang="en">Child Two</skos:prefLabel>
    <arches:sortorder rdf:datatype="http://www.w3.org/2001/XMLSchema#integer">2</arches:sortorder>
  </skos:Concept>
</rdf:RDF>"#;

    #[test]
    fn test_parse_skos() {
        let result = parse_skos_to_collections(TEST_SKOS, "http://example.org/");
        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 1);
        assert!(!collections[0].concepts.is_empty());
    }

    #[test]
    fn test_parse_hierarchical_skos() {
        let result = parse_skos_to_collections(TEST_SKOS_WITH_HIERARCHY, "http://example.org/");
        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 1);

        let collection = &collections[0];
        // Should have one top-level concept (parent)
        assert_eq!(collection.concepts.len(), 1);

        // Find the parent concept
        let parent = collection.concepts.values().next().unwrap();
        assert!(parent.pref_labels.contains_key("en"));
        assert_eq!(parent.pref_labels["en"].value, "Parent Concept");

        // Parent should have 2 children
        assert!(parent.children.is_some());
        let children = parent.children.as_ref().unwrap();
        assert_eq!(children.len(), 2);

        // Children should be sorted by sort_order
        assert_eq!(children[0].pref_labels["en"].value, "Child One");
        assert_eq!(children[1].pref_labels["en"].value, "Child Two");
    }

    #[test]
    fn test_serialize_collection_to_xml() {
        // Create a simple collection
        let mut pref_labels = HashMap::new();
        pref_labels.insert("en".to_string(), SkosValue {
            id: "label-1".to_string(),
            value: "Test Collection".to_string(),
        });

        let mut concept_labels = HashMap::new();
        concept_labels.insert("en".to_string(), SkosValue {
            id: "concept-label-1".to_string(),
            value: "Test Concept".to_string(),
        });

        let concept = SkosConcept {
            id: "concept-1".to_string(),
            uri: Some("http://example.org/concept/1".to_string()),
            pref_labels: concept_labels,
            source: Some("http://example.org/source/1".to_string()),
            sort_order: Some(0),
            children: None,
        };

        let mut concepts = HashMap::new();
        concepts.insert("concept-1".to_string(), concept);

        let collection = SkosCollection {
            id: "collection-1".to_string(),
            pref_labels,
            concepts,
            all_concepts: HashMap::new(),
            values: HashMap::new(),
        };

        let xml = collection_to_skos_xml(&collection, "http://example.org/");

        // Verify XML structure
        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("xmlns:skos=\"http://www.w3.org/2004/02/skos/core#\""));
        assert!(xml.contains("skos:ConceptScheme"));
        assert!(xml.contains("skos:Concept"));
        assert!(xml.contains("Test Collection"));
        assert!(xml.contains("Test Concept"));
        assert!(xml.contains("skos:prefLabel"));
        assert!(xml.contains("xml:lang=\"en\""));
    }

    #[test]
    fn test_serialize_hierarchical_collection() {
        // Create collection with nested concepts
        let mut pref_labels = HashMap::new();
        pref_labels.insert("en".to_string(), SkosValue {
            id: "label-1".to_string(),
            value: "Hierarchical Collection".to_string(),
        });

        let mut child_labels = HashMap::new();
        child_labels.insert("en".to_string(), SkosValue {
            id: "child-label-1".to_string(),
            value: "Child Concept".to_string(),
        });

        let child = SkosConcept {
            id: "child-1".to_string(),
            uri: Some("http://example.org/concept/child".to_string()),
            pref_labels: child_labels,
            source: None,
            sort_order: Some(1),
            children: None,
        };

        let mut parent_labels = HashMap::new();
        parent_labels.insert("en".to_string(), SkosValue {
            id: "parent-label-1".to_string(),
            value: "Parent Concept".to_string(),
        });

        let parent = SkosConcept {
            id: "parent-1".to_string(),
            uri: Some("http://example.org/concept/parent".to_string()),
            pref_labels: parent_labels,
            source: None,
            sort_order: Some(0),
            children: Some(vec![child]),
        };

        let mut concepts = HashMap::new();
        concepts.insert("parent-1".to_string(), parent);

        let collection = SkosCollection {
            id: "hier-collection".to_string(),
            pref_labels,
            concepts,
            all_concepts: HashMap::new(),
            values: HashMap::new(),
        };

        let xml = collection_to_skos_xml(&collection, "http://example.org/");

        // Verify hierarchy is represented
        assert!(xml.contains("skos:narrower"));
        assert!(xml.contains("skos:broader"));
        assert!(xml.contains("Parent Concept"));
        assert!(xml.contains("Child Concept"));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("test"), "test");
        assert_eq!(xml_escape("a < b"), "a &lt; b");
        assert_eq!(xml_escape("a > b"), "a &gt; b");
        assert_eq!(xml_escape("a & b"), "a &amp; b");
        assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(xml_escape("it's"), "it&apos;s");
        assert_eq!(xml_escape("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&apos;xss&apos;)&lt;/script&gt;");
    }

    #[test]
    fn test_round_trip_simple() {
        // Parse -> Serialize -> Parse should preserve data
        let result = parse_skos_to_collections(TEST_SKOS, "http://example.org/");
        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 1);

        // Serialize back to XML
        let xml = collection_to_skos_xml(&collections[0], "http://example.org/");

        // Parse again
        let result2 = parse_skos_to_collections(&xml, "http://example.org/");
        assert!(result2.is_ok());
        let collections2 = result2.unwrap();
        assert_eq!(collections2.len(), 1);

        // Verify concept count matches
        assert_eq!(collections[0].concepts.len(), collections2[0].concepts.len());

        // Verify concept labels are preserved
        for (id, concept) in &collections[0].concepts {
            // Find matching concept in round-tripped collection
            let found = collections2[0].concepts.values()
                .find(|c| c.pref_labels.values().any(|v|
                    concept.pref_labels.values().any(|v2| v.value == v2.value)
                ));
            assert!(found.is_some(), "Concept with id {} not found after round-trip", id);
        }
    }

    #[test]
    fn test_round_trip_hierarchical() {
        // Parse hierarchical SKOS
        let result = parse_skos_to_collections(TEST_SKOS_WITH_HIERARCHY, "http://example.org/");
        assert!(result.is_ok());
        let collections = result.unwrap();

        // Serialize
        let xml = collection_to_skos_xml(&collections[0], "http://example.org/");

        // Parse again
        let result2 = parse_skos_to_collections(&xml, "http://example.org/");
        assert!(result2.is_ok());
        let collections2 = result2.unwrap();

        // Should still have 1 top-level concept
        assert_eq!(collections2[0].concepts.len(), 1);

        // Should still have children
        let parent = collections2[0].concepts.values().next().unwrap();
        assert!(parent.children.is_some());

        let children = parent.children.as_ref().unwrap();
        assert_eq!(children.len(), 2);

        // Children labels preserved
        let child_labels: Vec<&str> = children.iter()
            .filter_map(|c| c.pref_labels.get("en").map(|v| v.value.as_str()))
            .collect();
        assert!(child_labels.contains(&"Child One"));
        assert!(child_labels.contains(&"Child Two"));
    }

    #[test]
    fn test_round_trip_multilingual() {
        // Parse with multiple languages
        let result = parse_skos_to_collections(TEST_SKOS_WITH_HIERARCHY, "http://example.org/");
        assert!(result.is_ok());
        let collections = result.unwrap();

        let parent = collections[0].concepts.values().next().unwrap();
        // Original should have both en and de labels
        assert!(parent.pref_labels.contains_key("en"));
        assert!(parent.pref_labels.contains_key("de"));
        assert_eq!(parent.pref_labels["de"].value, "Elternkonzept");

        // Serialize and parse again
        let xml = collection_to_skos_xml(&collections[0], "http://example.org/");
        let result2 = parse_skos_to_collections(&xml, "http://example.org/");
        assert!(result2.is_ok());
        let collections2 = result2.unwrap();

        let parent2 = collections2[0].concepts.values().next().unwrap();
        // Should preserve both languages
        assert!(parent2.pref_labels.contains_key("en"));
        assert!(parent2.pref_labels.contains_key("de"));
        assert_eq!(parent2.pref_labels["de"].value, "Elternkonzept");
    }
}
