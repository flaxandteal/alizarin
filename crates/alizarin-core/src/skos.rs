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

// SKOS namespace URIs (documented for reference):
// - SKOS: http://www.w3.org/2004/02/skos/core#
// - RDF:  http://www.w3.org/1999/02/22-rdf-syntax-ns#
// - DCTERMS: http://purl.org/dc/terms/

// SKOS predicates
const SKOS_CONCEPT: &str = "http://www.w3.org/2004/02/skos/core#Concept";
const SKOS_CONCEPT_SCHEME: &str = "http://www.w3.org/2004/02/skos/core#ConceptScheme";
const SKOS_COLLECTION: &str = "http://www.w3.org/2004/02/skos/core#Collection";
const SKOS_PREF_LABEL: &str = "http://www.w3.org/2004/02/skos/core#prefLabel";
const SKOS_ALT_LABEL: &str = "http://www.w3.org/2004/02/skos/core#altLabel";
const SKOS_SCOPE_NOTE: &str = "http://www.w3.org/2004/02/skos/core#scopeNote";
const SKOS_NARROWER: &str = "http://www.w3.org/2004/02/skos/core#narrower";
const SKOS_BROADER: &str = "http://www.w3.org/2004/02/skos/core#broader";
const SKOS_MEMBER: &str = "http://www.w3.org/2004/02/skos/core#member";
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

/// The type of SKOS grouping structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkosNodeType {
    /// A ConceptScheme uses narrower/broader hierarchy
    #[default]
    ConceptScheme,
    /// A Collection uses flat member relationships (Arches-compatible)
    Collection,
}

/// A parsed SKOS collection/concept scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkosCollection {
    pub id: String,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(rename = "prefLabels")]
    pub pref_labels: HashMap<String, SkosValue>,
    #[serde(rename = "altLabels", default)]
    pub alt_labels: HashMap<String, Vec<SkosValue>>,
    #[serde(rename = "scopeNotes", default)]
    pub scope_notes: HashMap<String, SkosValue>,
    /// The type of this grouping (ConceptScheme or Collection)
    #[serde(rename = "nodeType", default)]
    pub node_type: SkosNodeType,
    /// For ConceptScheme: hierarchical concepts (top-level with children)
    /// For Collection: flat member concepts (no hierarchy)
    pub concepts: HashMap<String, SkosConcept>,
    #[serde(rename = "__allConcepts")]
    pub all_concepts: HashMap<String, SkosConcept>,
    #[serde(rename = "__values")]
    pub values: HashMap<String, SkosValue>,
}

/// Internal structure for collecting triples during parsing
#[derive(Debug, Default)]
struct ParsedData {
    /// URI -> type (Concept, ConceptScheme, or Collection)
    types: HashMap<String, String>,
    /// URI -> labels (predicate, value, language)
    labels: HashMap<String, Vec<(String, String, String)>>,
    /// URI -> scope notes (language, value)
    scope_notes: HashMap<String, Vec<(String, String)>>,
    /// URI -> narrower URIs (for ConceptScheme hierarchy)
    narrower: HashMap<String, Vec<String>>,
    /// URI -> broader URIs (for ConceptScheme hierarchy)
    broader: HashMap<String, Vec<String>>,
    /// Collection URI -> member URIs (for Collection membership)
    members: HashMap<String, Vec<String>>,
    /// URI -> scheme URIs
    in_scheme: HashMap<String, Vec<String>>,
    /// Scheme URI -> title
    scheme_titles: HashMap<String, String>,
    /// URI -> identifier
    identifiers: HashMap<String, String>,
    /// URI -> sort order
    sort_orders: HashMap<String, i32>,
}

/// Extract UUID from URI or generate a deterministic one via UUID5.
fn extract_or_generate_id(uri: &str) -> String {
    use std::sync::OnceLock;

    static UUID_RE: OnceLock<regex_lite::Regex> = OnceLock::new();
    let uuid_regex = UUID_RE.get_or_init(|| {
        regex_lite::Regex::new(r"([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})")
            .unwrap()
    });

    if let Some(caps) = uuid_regex.captures(uri) {
        return caps.get(1).unwrap().as_str().to_string();
    }

    // Generate deterministic UUID from URI using proper UUID v5
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, uri.as_bytes()).to_string()
}

/// Generate a value ID using UUID5 (delegates to rdm_namespace).
fn generate_value_id(concept_id: &str, lang: &str, value: &str) -> String {
    crate::rdm_namespace::generate_value_uuid(concept_id, value, lang).to_string()
}

/// Parse Arches-style JSON-wrapped label values
/// Arches labels can be: {"id": "uuid", "value": "label"} or just "label"
fn parse_arches_label(raw_value: &str, fallback_id: &str, lang: &str) -> (String, String) {
    // Try to parse as JSON object with id and value fields
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw_value) {
        if let Some(obj) = parsed.as_object() {
            let id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| generate_value_id(fallback_id, lang, raw_value));
            let value = obj
                .get("value")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| raw_value.to_string());
            return (id, value);
        }
    }
    // Not JSON, treat as plain string
    (
        generate_value_id(fallback_id, lang, raw_value),
        raw_value.to_string(),
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
                    self.labels.entry(subject_uri).or_default().push((
                        predicate.to_string(),
                        value,
                        lang,
                    ));
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
            SKOS_MEMBER => {
                if let Term::NamedNode(obj) = triple.object {
                    self.members
                        .entry(subject_uri)
                        .or_default()
                        .push(obj.iri.to_string());
                }
            }
            SKOS_SCOPE_NOTE => {
                if let Term::Literal(lit) = triple.object {
                    let (value, lang) = match lit {
                        Literal::Simple { value } => (value.to_string(), "en".to_string()),
                        Literal::LanguageTaggedString { value, language } => {
                            (value.to_string(), language.to_string())
                        }
                        Literal::Typed { value, .. } => (value.to_string(), "en".to_string()),
                    };
                    self.scope_notes
                        .entry(subject_uri)
                        .or_default()
                        .push((lang, value));
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

/// Parse SKOS RDF/XML and return collections
pub fn parse_skos_to_collections(
    xml_content: &str,
    base_uri: &str,
) -> Result<Vec<SkosCollection>, String> {
    let mut data = ParsedData::default();

    // Parse base URI into Iri
    let base_iri =
        oxiri::Iri::parse(base_uri.to_string()).map_err(|e| format!("Invalid base URI: {}", e))?;

    // Parse RDF/XML
    let mut parser = RdfXmlParser::new(xml_content.as_bytes(), Some(base_iri));

    parser
        .parse_all(&mut |triple| {
            data.process_triple(triple);
            Ok(()) as Result<(), std::io::Error>
        })
        .map_err(|e| format!("RDF/XML parse error: {}", e))?;

    // Find all concept schemes
    let scheme_uris: Vec<String> = data
        .types
        .iter()
        .filter(|(_, t)| *t == SKOS_CONCEPT_SCHEME)
        .map(|(uri, _)| uri.clone())
        .collect();

    // Find all concepts
    let concept_uris: Vec<String> = data
        .types
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
                    pref_labels.insert(
                        lang.clone(),
                        SkosValue {
                            id: generate_value_id(&id, lang, value),
                            value: value.clone(),
                        },
                    );
                }
            }
        }

        // If no prefLabel, use first available label
        if pref_labels.is_empty() {
            if let Some(labels) = data.labels.get(uri) {
                if let Some((_, value, lang)) = labels.first() {
                    pref_labels.insert(
                        lang.clone(),
                        SkosValue {
                            id: generate_value_id(&id, lang, value),
                            value: value.clone(),
                        },
                    );
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
        _sort_orders: &HashMap<String, i32>,
    ) -> Option<SkosConcept> {
        let concept = all_concepts.get(uri)?;
        let mut result = concept.clone();

        if let Some(child_uris) = children_map.get(uri) {
            let mut children: Vec<SkosConcept> = child_uris
                .iter()
                .filter_map(|child_uri| {
                    build_concept_tree(child_uri, all_concepts, children_map, _sort_orders)
                })
                .collect();

            // Sort children by sort_order
            children.sort_by(|a, b| {
                a.sort_order
                    .unwrap_or(999)
                    .cmp(&b.sort_order.unwrap_or(999))
            });

            if !children.is_empty() {
                result.children = Some(children);
            }
        }

        Some(result)
    }

    // Filter a concept tree to only include concepts that are members of the collection
    fn filter_concept_tree_to_members(
        concept: &SkosConcept,
        member_uris: &std::collections::HashSet<&String>,
    ) -> SkosConcept {
        let mut result = concept.clone();

        if let Some(ref children) = concept.children {
            let filtered_children: Vec<SkosConcept> = children
                .iter()
                .filter(|child| {
                    // Include child if it or any of its descendants is a member
                    let child_uri = child.uri.as_ref().unwrap_or(&child.id);
                    member_uris.contains(child_uri)
                        || member_uris.iter().any(|m| m.as_str() == child.id.as_str())
                })
                .map(|child| filter_concept_tree_to_members(child, member_uris))
                .collect();

            result.children = if filtered_children.is_empty() {
                None
            } else {
                Some(filtered_children)
            };
        }

        result
    }

    // Build collections
    let mut collections: Vec<SkosCollection> = Vec::new();

    for scheme_uri in &scheme_uris {
        let scheme_id = extract_or_generate_id(scheme_uri);
        let title = data
            .scheme_titles
            .get(scheme_uri)
            .cloned()
            .unwrap_or_else(|| scheme_id.clone());

        // Find concepts in this scheme
        let scheme_concept_uris: Vec<&String> = data
            .in_scheme
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
            if let Some(concept) =
                build_concept_tree(uri, &all_concepts, &children_map, &data.sort_orders)
            {
                concepts.insert(concept.id.clone(), concept);
            }
        }

        let mut pref_labels = HashMap::new();
        pref_labels.insert(
            "en".to_string(),
            SkosValue {
                id: generate_value_id(&scheme_id, "en", &title),
                value: title,
            },
        );

        // Build alt_labels for this scheme
        let mut alt_labels: HashMap<String, Vec<SkosValue>> = HashMap::new();
        if let Some(labels) = data.labels.get(scheme_uri) {
            for (pred, value, lang) in labels {
                if pred == SKOS_ALT_LABEL {
                    alt_labels.entry(lang.clone()).or_default().push(SkosValue {
                        id: generate_value_id(&scheme_id, lang, value),
                        value: value.clone(),
                    });
                }
            }
        }

        // Build scope_notes for this scheme
        let mut scope_notes: HashMap<String, SkosValue> = HashMap::new();
        if let Some(notes) = data.scope_notes.get(scheme_uri) {
            for (lang, value) in notes {
                scope_notes.insert(
                    lang.clone(),
                    SkosValue {
                        id: generate_value_id(&scheme_id, lang, value),
                        value: value.clone(),
                    },
                );
            }
        }

        collections.push(SkosCollection {
            id: scheme_id,
            uri: Some(scheme_uri.clone()),
            pref_labels,
            alt_labels,
            scope_notes,
            node_type: SkosNodeType::ConceptScheme,
            concepts,
            all_concepts: HashMap::new(),
            values: HashMap::new(),
        });
    }

    // Find all SKOS Collections (distinct from ConceptSchemes)
    let collection_uris: Vec<String> = data
        .types
        .iter()
        .filter(|(_, t)| *t == SKOS_COLLECTION)
        .map(|(uri, _)| uri.clone())
        .collect();

    // Build SKOS Collections (flat member structure, no hierarchy)
    for collection_uri in &collection_uris {
        let collection_id = extract_or_generate_id(collection_uri);

        // Build prefLabels for the collection
        let mut pref_labels: HashMap<String, SkosValue> = HashMap::new();
        if let Some(labels) = data.labels.get(collection_uri) {
            for (pred, value, lang) in labels {
                if pred == SKOS_PREF_LABEL {
                    // Handle Arches JSON-wrapped labels: {"id": "...", "value": "..."}
                    let (label_id, label_value) = parse_arches_label(value, &collection_id, lang);
                    pref_labels.insert(
                        lang.clone(),
                        SkosValue {
                            id: label_id,
                            value: label_value,
                        },
                    );
                }
            }
        }

        // Build alt_labels for the collection
        let mut alt_labels: HashMap<String, Vec<SkosValue>> = HashMap::new();
        if let Some(labels) = data.labels.get(collection_uri) {
            for (pred, value, lang) in labels {
                if pred == SKOS_ALT_LABEL {
                    let (label_id, label_value) = parse_arches_label(value, &collection_id, lang);
                    alt_labels.entry(lang.clone()).or_default().push(SkosValue {
                        id: label_id,
                        value: label_value,
                    });
                }
            }
        }

        // Build scope_notes for the collection
        let mut scope_notes: HashMap<String, SkosValue> = HashMap::new();
        if let Some(notes) = data.scope_notes.get(collection_uri) {
            for (lang, value) in notes {
                let (note_id, note_value) = parse_arches_label(value, &collection_id, lang);
                scope_notes.insert(
                    lang.clone(),
                    SkosValue {
                        id: note_id,
                        value: note_value,
                    },
                );
            }
        }

        // Get member concepts - find top-level concepts that are members
        // Collections use member for membership but concepts can have hierarchy via narrower/broader
        let mut concepts: HashMap<String, SkosConcept> = HashMap::new();
        if let Some(member_uris) = data.members.get(collection_uri) {
            // Find member URIs that are top-level (not narrower of another member)
            let member_set: std::collections::HashSet<&String> = member_uris.iter().collect();
            let top_level_members: Vec<&String> = member_uris
                .iter()
                .filter(|uri| {
                    // This is a top-level member if no other member has it as narrower
                    !member_uris.iter().any(|other| {
                        if let Some(children) = children_map.get(other.as_str()) {
                            children.contains(&(**uri).to_string())
                        } else {
                            false
                        }
                    })
                })
                .collect();

            for member_uri in top_level_members {
                if let Some(concept) =
                    build_concept_tree(member_uri, &all_concepts, &children_map, &data.sort_orders)
                {
                    // Filter children to only include those that are also members
                    let filtered_concept = filter_concept_tree_to_members(&concept, &member_set);
                    concepts.insert(filtered_concept.id.clone(), filtered_concept);
                }
            }
        }

        collections.push(SkosCollection {
            id: collection_id,
            uri: Some(collection_uri.clone()),
            pref_labels,
            alt_labels,
            scope_notes,
            node_type: SkosNodeType::Collection,
            concepts,
            all_concepts: HashMap::new(),
            values: HashMap::new(),
        });
    }

    // If no schemes or collections found, create default collection with all concepts
    if collections.is_empty() && !all_concepts.is_empty() {
        let top_level: Vec<String> = concept_uris
            .iter()
            .filter(|uri| !all_narrower.contains(*uri))
            .cloned()
            .collect();

        let mut concepts: HashMap<String, SkosConcept> = HashMap::new();
        for uri in &top_level {
            if let Some(concept) =
                build_concept_tree(uri, &all_concepts, &children_map, &data.sort_orders)
            {
                concepts.insert(concept.id.clone(), concept);
            }
        }

        let default_id = extract_or_generate_id(base_uri);
        let mut pref_labels = HashMap::new();
        pref_labels.insert(
            "en".to_string(),
            SkosValue {
                id: generate_value_id(&default_id, "en", "Imported Concepts"),
                value: "Imported Concepts".to_string(),
            },
        );

        collections.push(SkosCollection {
            id: default_id,
            uri: Some(base_uri.to_string()),
            pref_labels,
            alt_labels: HashMap::new(),
            scope_notes: HashMap::new(),
            node_type: SkosNodeType::ConceptScheme,
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

/// XML escape helper for attribute values (escapes quotes)
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// XML escape helper for element content (quotes don't need escaping)
fn xml_escape_content(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// =============================================================================
// Deterministic Sorting Helpers
// =============================================================================
// These ensure XML and JSON outputs are identical for the same input,
// enabling git diff comparisons.

/// Get sorted iterator over HashMap by key
fn sorted_by_key<K: Ord, V>(map: &HashMap<K, V>) -> Vec<(&K, &V)> {
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by_key(|(k, _)| *k);
    entries
}

/// Get sorted children by ID
fn sorted_children(children: &Option<Vec<SkosConcept>>) -> Vec<&SkosConcept> {
    match children {
        Some(kids) => {
            let mut sorted: Vec<&SkosConcept> = kids.iter().collect();
            sorted.sort_by_key(|c| &c.id);
            sorted
        }
        None => vec![],
    }
}

/// Get sorted concepts from HashMap by ID
fn sorted_concepts(concepts: &HashMap<String, SkosConcept>) -> Vec<&SkosConcept> {
    let mut sorted: Vec<_> = concepts.values().collect();
    sorted.sort_by_key(|c| &c.id);
    sorted
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

    // Add prefLabels (sorted by language for deterministic output)
    for (lang, value) in sorted_by_key(&concept.pref_labels) {
        output.push_str(&format!(
            "    <skos:prefLabel xml:lang=\"{}\">{}</skos:prefLabel>\n",
            xml_escape(lang),
            xml_escape_content(&value.value)
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
            xml_escape_content(source)
        ));
    }

    // Add narrower relationships for children (sorted by ID for deterministic output)
    for child in sorted_children(&concept.children) {
        let child_uri = match &child.uri {
            Some(uri) if !uri.is_empty() => uri.clone(),
            _ => format!("{}{}", base_uri, child.id),
        };
        output.push_str(&format!(
            "    <skos:narrower rdf:resource=\"{}\"/>\n",
            xml_escape(&child_uri)
        ));
    }

    output.push_str("  </skos:Concept>\n");

    // Recursively write children (sorted by ID for deterministic output)
    for child in sorted_children(&concept.children) {
        write_concept_xml(child, scheme_uri, base_uri, Some(&concept_uri), output);
    }
}

/// Write a concept for a Collection (uses member relation but concepts can have hierarchy)
fn write_collection_concept_xml(
    concept: &SkosConcept,
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

    // Add broader (parent) relationship if this is a child concept
    if let Some(parent) = parent_uri {
        output.push_str(&format!(
            "    <skos:broader rdf:resource=\"{}\"/>\n",
            xml_escape(parent)
        ));
    }

    // Add prefLabels (sorted by language for deterministic output)
    for (lang, value) in sorted_by_key(&concept.pref_labels) {
        output.push_str(&format!(
            "    <skos:prefLabel xml:lang=\"{}\">{}</skos:prefLabel>\n",
            xml_escape(lang),
            xml_escape_content(&value.value)
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
            xml_escape_content(source)
        ));
    }

    // Add narrower relationships for children (sorted by ID for deterministic output)
    for child in sorted_children(&concept.children) {
        let child_uri = match &child.uri {
            Some(uri) if !uri.is_empty() => uri.clone(),
            _ => format!("{}{}", base_uri, child.id),
        };
        output.push_str(&format!(
            "    <skos:narrower rdf:resource=\"{}\"/>\n",
            xml_escape(&child_uri)
        ));
    }

    output.push_str("  </skos:Concept>\n");

    // Recursively write children (sorted by ID for deterministic output)
    for child in sorted_children(&concept.children) {
        write_collection_concept_xml(child, base_uri, Some(&concept_uri), output);
    }
}

/// Collect all concept URIs from a concept tree (for member listing)
fn collect_all_concept_uris(concept: &SkosConcept, base_uri: &str, uris: &mut Vec<String>) {
    let concept_uri = match &concept.uri {
        Some(uri) if !uri.is_empty() => uri.clone(),
        _ => format!("{}{}", base_uri, concept.id),
    };
    uris.push(concept_uri);

    if let Some(ref children) = concept.children {
        for child in children {
            collect_all_concept_uris(child, base_uri, uris);
        }
    }
}

/// Serialize a SkosCollection to SKOS RDF/XML format
pub fn collection_to_skos_xml(collection: &SkosCollection, base_uri: &str) -> String {
    let mut output = String::new();

    // XML declaration and RDF root with namespaces
    output.push_str(
        r#"<?xml version="1.0" encoding="utf-8"?>
<rdf:RDF
  xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
  xmlns:skos="http://www.w3.org/2004/02/skos/core#"
  xmlns:dcterms="http://purl.org/dc/terms/"
  xmlns:arches="http://localhost:8000/"
>
"#,
    );

    // Build collection/scheme URI
    let entity_uri = collection
        .uri
        .clone()
        .unwrap_or_else(|| format!("{}{}", base_uri, collection.id));

    match collection.node_type {
        SkosNodeType::Collection => {
            // Write SKOS Collection (flat, member-based)
            output.push_str(&format!(
                "  <skos:Collection rdf:about=\"{}\">\n",
                xml_escape(&entity_uri)
            ));

            // Add prefLabels (sorted by language for deterministic output)
            for (lang, value) in sorted_by_key(&collection.pref_labels) {
                let json_value = serde_json::json!({
                    "id": value.id,
                    "value": value.value
                });
                output.push_str(&format!(
                    "    <skos:prefLabel xml:lang=\"{}\">{}</skos:prefLabel>\n",
                    xml_escape(lang),
                    xml_escape_content(&json_value.to_string())
                ));
            }

            // Add altLabels (sorted by language for deterministic output)
            for (lang, values) in sorted_by_key(&collection.alt_labels) {
                for value in values {
                    let json_value = serde_json::json!({
                        "id": value.id,
                        "value": value.value
                    });
                    output.push_str(&format!(
                        "    <skos:altLabel xml:lang=\"{}\">{}</skos:altLabel>\n",
                        xml_escape(lang),
                        xml_escape_content(&json_value.to_string())
                    ));
                }
            }

            // Add scopeNotes (sorted by language for deterministic output)
            for (lang, value) in sorted_by_key(&collection.scope_notes) {
                let json_value = serde_json::json!({
                    "id": value.id,
                    "value": value.value
                });
                output.push_str(&format!(
                    "    <skos:scopeNote xml:lang=\"{}\">{}</skos:scopeNote>\n",
                    xml_escape(lang),
                    xml_escape_content(&json_value.to_string())
                ));
            }

            // Add member references for ALL concepts (sorted for deterministic output)
            let mut all_concept_uris: Vec<String> = Vec::new();
            for concept in sorted_concepts(&collection.concepts) {
                collect_all_concept_uris(concept, base_uri, &mut all_concept_uris);
            }
            // Sort member URIs for deterministic output
            all_concept_uris.sort();
            for concept_uri in &all_concept_uris {
                output.push_str(&format!(
                    "    <skos:member>\n      <skos:Concept rdf:about=\"{}\"/>\n    </skos:member>\n",
                    xml_escape(concept_uri)
                ));
            }

            output.push_str("  </skos:Collection>\n");

            // Write the concept definitions (sorted by ID for deterministic output)
            for concept in sorted_concepts(&collection.concepts) {
                write_collection_concept_xml(concept, base_uri, None, &mut output);
            }
        }
        SkosNodeType::ConceptScheme => {
            // Write ConceptScheme (hierarchical, narrower/broader)
            output.push_str(&format!(
                "  <skos:ConceptScheme rdf:about=\"{}\">\n",
                xml_escape(&entity_uri)
            ));

            // Add title from prefLabels (sorted by language for deterministic output)
            for (lang, value) in sorted_by_key(&collection.pref_labels) {
                output.push_str(&format!(
                    "    <dcterms:title xml:lang=\"{}\">{}</dcterms:title>\n",
                    xml_escape(lang),
                    xml_escape_content(&value.value)
                ));
            }

            // Add altLabels (sorted by language for deterministic output)
            for (lang, values) in sorted_by_key(&collection.alt_labels) {
                for value in values {
                    output.push_str(&format!(
                        "    <skos:altLabel xml:lang=\"{}\">{}</skos:altLabel>\n",
                        xml_escape(lang),
                        xml_escape_content(&value.value)
                    ));
                }
            }

            // Add scopeNotes (sorted by language for deterministic output)
            for (lang, value) in sorted_by_key(&collection.scope_notes) {
                output.push_str(&format!(
                    "    <skos:scopeNote xml:lang=\"{}\">{}</skos:scopeNote>\n",
                    xml_escape(lang),
                    xml_escape_content(&value.value)
                ));
            }

            // Add hasTopConcept references for each top-level concept (sorted by ID)
            for concept in sorted_concepts(&collection.concepts) {
                let concept_uri = match &concept.uri {
                    Some(uri) if !uri.is_empty() => uri.clone(),
                    _ => format!("{}{}", base_uri, concept.id),
                };
                output.push_str(&format!(
                    "    <skos:hasTopConcept rdf:resource=\"{}\"/>\n",
                    xml_escape(&concept_uri)
                ));
            }

            output.push_str("  </skos:ConceptScheme>\n");

            // Write all top-level concepts (and their children recursively, sorted by ID)
            for concept in sorted_concepts(&collection.concepts) {
                write_concept_xml(concept, &entity_uri, base_uri, None, &mut output);
            }
        }
    }

    output.push_str("</rdf:RDF>\n");

    output
}

/// Serialize multiple SkosCollections to SKOS RDF/XML
pub fn collections_to_skos_xml(collections: &[SkosCollection], base_uri: &str) -> String {
    let mut output = String::new();

    // XML declaration and RDF root with namespaces
    output.push_str(
        r#"<?xml version="1.0" encoding="utf-8"?>
<rdf:RDF
  xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
  xmlns:skos="http://www.w3.org/2004/02/skos/core#"
  xmlns:dcterms="http://purl.org/dc/terms/"
  xmlns:arches="http://localhost:8000/"
>
"#,
    );

    for collection in collections {
        let entity_uri = collection
            .uri
            .clone()
            .unwrap_or_else(|| format!("{}{}", base_uri, collection.id));

        match collection.node_type {
            SkosNodeType::Collection => {
                // Write SKOS Collection
                output.push_str(&format!(
                    "  <skos:Collection rdf:about=\"{}\">\n",
                    xml_escape(&entity_uri)
                ));

                // Sorted by language for deterministic output
                for (lang, value) in sorted_by_key(&collection.pref_labels) {
                    let json_value = serde_json::json!({
                        "id": value.id,
                        "value": value.value
                    });
                    output.push_str(&format!(
                        "    <skos:prefLabel xml:lang=\"{}\">{}</skos:prefLabel>\n",
                        xml_escape(lang),
                        xml_escape_content(&json_value.to_string())
                    ));
                }

                // Sorted by language for deterministic output
                for (lang, values) in sorted_by_key(&collection.alt_labels) {
                    for value in values {
                        let json_value = serde_json::json!({
                            "id": value.id,
                            "value": value.value
                        });
                        output.push_str(&format!(
                            "    <skos:altLabel xml:lang=\"{}\">{}</skos:altLabel>\n",
                            xml_escape(lang),
                            xml_escape_content(&json_value.to_string())
                        ));
                    }
                }

                // Sorted by language for deterministic output
                for (lang, value) in sorted_by_key(&collection.scope_notes) {
                    let json_value = serde_json::json!({
                        "id": value.id,
                        "value": value.value
                    });
                    output.push_str(&format!(
                        "    <skos:scopeNote xml:lang=\"{}\">{}</skos:scopeNote>\n",
                        xml_escape(lang),
                        xml_escape_content(&json_value.to_string())
                    ));
                }

                // Add member references for ALL concepts (including nested children)
                // Collect from sorted concepts for deterministic output
                let mut all_concept_uris: Vec<String> = Vec::new();
                for concept in sorted_concepts(&collection.concepts) {
                    collect_all_concept_uris(concept, base_uri, &mut all_concept_uris);
                }
                // Sort the final list for deterministic output
                all_concept_uris.sort();
                for concept_uri in &all_concept_uris {
                    output.push_str(&format!(
                        "    <skos:member>\n      <skos:Concept rdf:about=\"{}\"/>\n    </skos:member>\n",
                        xml_escape(concept_uri)
                    ));
                }

                output.push_str("  </skos:Collection>\n");

                // Write the concept definitions (sorted by ID for deterministic output)
                for concept in sorted_concepts(&collection.concepts) {
                    write_collection_concept_xml(concept, base_uri, None, &mut output);
                }
            }
            SkosNodeType::ConceptScheme => {
                // Write ConceptScheme
                output.push_str(&format!(
                    "  <skos:ConceptScheme rdf:about=\"{}\">\n",
                    xml_escape(&entity_uri)
                ));

                // Sorted by language for deterministic output
                for (lang, value) in sorted_by_key(&collection.pref_labels) {
                    output.push_str(&format!(
                        "    <dcterms:title xml:lang=\"{}\">{}</dcterms:title>\n",
                        xml_escape(lang),
                        xml_escape_content(&value.value)
                    ));
                }

                // Sorted by language for deterministic output
                for (lang, values) in sorted_by_key(&collection.alt_labels) {
                    for value in values {
                        output.push_str(&format!(
                            "    <skos:altLabel xml:lang=\"{}\">{}</skos:altLabel>\n",
                            xml_escape(lang),
                            xml_escape_content(&value.value)
                        ));
                    }
                }

                // Sorted by language for deterministic output
                for (lang, value) in sorted_by_key(&collection.scope_notes) {
                    output.push_str(&format!(
                        "    <skos:scopeNote xml:lang=\"{}\">{}</skos:scopeNote>\n",
                        xml_escape(lang),
                        xml_escape_content(&value.value)
                    ));
                }

                // Add hasTopConcept references for each top-level concept (sorted by ID)
                for concept in sorted_concepts(&collection.concepts) {
                    let concept_uri = match &concept.uri {
                        Some(uri) if !uri.is_empty() => uri.clone(),
                        _ => format!("{}{}", base_uri, concept.id),
                    };
                    output.push_str(&format!(
                        "    <skos:hasTopConcept rdf:resource=\"{}\"/>\n",
                        xml_escape(&concept_uri)
                    ));
                }

                output.push_str("  </skos:ConceptScheme>\n");

                // Write concepts (sorted by ID for deterministic output)
                for concept in sorted_concepts(&collection.concepts) {
                    write_concept_xml(concept, &entity_uri, base_uri, None, &mut output);
                }
            }
        }
    }

    output.push_str("</rdf:RDF>\n");

    output
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
        pref_labels.insert(
            "en".to_string(),
            SkosValue {
                id: "label-1".to_string(),
                value: "Test Collection".to_string(),
            },
        );

        let mut concept_labels = HashMap::new();
        concept_labels.insert(
            "en".to_string(),
            SkosValue {
                id: "concept-label-1".to_string(),
                value: "Test Concept".to_string(),
            },
        );

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
            uri: None,
            pref_labels,
            alt_labels: HashMap::new(),
            scope_notes: HashMap::new(),
            node_type: SkosNodeType::ConceptScheme,
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
        pref_labels.insert(
            "en".to_string(),
            SkosValue {
                id: "label-1".to_string(),
                value: "Hierarchical Collection".to_string(),
            },
        );

        let mut child_labels = HashMap::new();
        child_labels.insert(
            "en".to_string(),
            SkosValue {
                id: "child-label-1".to_string(),
                value: "Child Concept".to_string(),
            },
        );

        let child = SkosConcept {
            id: "child-1".to_string(),
            uri: Some("http://example.org/concept/child".to_string()),
            pref_labels: child_labels,
            source: None,
            sort_order: Some(1),
            children: None,
        };

        let mut parent_labels = HashMap::new();
        parent_labels.insert(
            "en".to_string(),
            SkosValue {
                id: "parent-label-1".to_string(),
                value: "Parent Concept".to_string(),
            },
        );

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
            uri: None,
            pref_labels,
            alt_labels: HashMap::new(),
            scope_notes: HashMap::new(),
            node_type: SkosNodeType::ConceptScheme,
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
        assert_eq!(
            xml_escape("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&apos;xss&apos;)&lt;/script&gt;"
        );
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
        assert_eq!(
            collections[0].concepts.len(),
            collections2[0].concepts.len()
        );

        // Verify concept labels are preserved
        for (id, concept) in &collections[0].concepts {
            // Find matching concept in round-tripped collection
            let found = collections2[0].concepts.values().find(|c| {
                c.pref_labels
                    .values()
                    .any(|v| concept.pref_labels.values().any(|v2| v.value == v2.value))
            });
            assert!(
                found.is_some(),
                "Concept with id {} not found after round-trip",
                id
            );
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
        let child_labels: Vec<&str> = children
            .iter()
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

    // ========================================================================
    // SKOS Collection (flat member-based) Tests - Arches compatibility
    // ========================================================================

    const TEST_ARCHES_COLLECTION: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:skos="http://www.w3.org/2004/02/skos/core#"
         xmlns:dcterms="http://purl.org/dc/terms/">
  <skos:Collection rdf:about="http://localhost:8000/7dde2f92-9f8a-44cf-817f-ec8c5c736f69">
    <skos:prefLabel xml:lang="en-us">{"id": "956f8913-f728-4f82-b3ae-3aaf4ce7891a", "value": "Test Collection"}</skos:prefLabel>
    <skos:altLabel xml:lang="en-us">{"id": "5e328859-7a75-494f-948d-730169def957", "value": "Test Alt"}</skos:altLabel>
    <skos:scopeNote xml:lang="en-us">{"id": "d91df30b-3c8b-4455-93de-77ff1096cb9d", "value": "Testing collection"}</skos:scopeNote>
    <skos:member>
      <skos:Concept rdf:about="http://localhost:8000/86be632e-0dad-4d88-b5da-3d65875d6239"/>
    </skos:member>
    <skos:member>
      <skos:Concept rdf:about="http://localhost:8000/54c5c8ac-890d-4f8e-b19a-dfa2401eaea3"/>
    </skos:member>
  </skos:Collection>
  <skos:Concept rdf:about="http://localhost:8000/86be632e-0dad-4d88-b5da-3d65875d6239">
    <skos:prefLabel xml:lang="en">Concept One</skos:prefLabel>
  </skos:Concept>
  <skos:Concept rdf:about="http://localhost:8000/54c5c8ac-890d-4f8e-b19a-dfa2401eaea3">
    <skos:prefLabel xml:lang="en">Concept Two</skos:prefLabel>
  </skos:Concept>
</rdf:RDF>"#;

    #[test]
    fn test_parse_arches_collection() {
        let result = parse_skos_to_collections(TEST_ARCHES_COLLECTION, "http://localhost:8000/");
        assert!(result.is_ok());
        let collections = result.unwrap();

        // Should have one Collection
        assert_eq!(collections.len(), 1);
        let collection = &collections[0];

        // Should be a Collection type, not ConceptScheme
        assert_eq!(collection.node_type, SkosNodeType::Collection);

        // Should have the correct prefLabel (parsed from JSON)
        assert!(collection.pref_labels.contains_key("en-us"));
        assert_eq!(collection.pref_labels["en-us"].value, "Test Collection");
        assert_eq!(
            collection.pref_labels["en-us"].id,
            "956f8913-f728-4f82-b3ae-3aaf4ce7891a"
        );

        // Should have altLabel
        assert!(collection.alt_labels.contains_key("en-us"));
        assert_eq!(collection.alt_labels["en-us"][0].value, "Test Alt");

        // Should have scopeNote
        assert!(collection.scope_notes.contains_key("en-us"));
        assert_eq!(collection.scope_notes["en-us"].value, "Testing collection");

        // Should have 2 member concepts (flat, no hierarchy)
        assert_eq!(collection.concepts.len(), 2);

        // Concepts should not have children (flat structure)
        for concept in collection.concepts.values() {
            assert!(concept.children.is_none());
        }
    }

    #[test]
    fn test_serialize_arches_collection() {
        // Create a Collection (not ConceptScheme)
        let mut pref_labels = HashMap::new();
        pref_labels.insert(
            "en-us".to_string(),
            SkosValue {
                id: "label-uuid-1".to_string(),
                value: "Test Collection".to_string(),
            },
        );

        let mut alt_labels = HashMap::new();
        alt_labels.insert(
            "en-us".to_string(),
            vec![SkosValue {
                id: "alt-uuid-1".to_string(),
                value: "Alt Label".to_string(),
            }],
        );

        let mut scope_notes = HashMap::new();
        scope_notes.insert(
            "en-us".to_string(),
            SkosValue {
                id: "note-uuid-1".to_string(),
                value: "A scope note".to_string(),
            },
        );

        let mut concept_labels = HashMap::new();
        concept_labels.insert(
            "en".to_string(),
            SkosValue {
                id: "concept-label-1".to_string(),
                value: "Member Concept".to_string(),
            },
        );

        let concept = SkosConcept {
            id: "concept-1".to_string(),
            uri: Some("http://localhost:8000/concept-1".to_string()),
            pref_labels: concept_labels,
            source: None,
            sort_order: None,
            children: None,
        };

        let mut concepts = HashMap::new();
        concepts.insert("concept-1".to_string(), concept);

        let collection = SkosCollection {
            id: "collection-1".to_string(),
            uri: Some("http://localhost:8000/collection-1".to_string()),
            pref_labels,
            alt_labels,
            scope_notes,
            node_type: SkosNodeType::Collection,
            concepts,
            all_concepts: HashMap::new(),
            values: HashMap::new(),
        };

        let xml = collection_to_skos_xml(&collection, "http://localhost:8000/");

        // Verify Collection structure (not ConceptScheme)
        assert!(xml.contains("skos:Collection"));
        assert!(!xml.contains("skos:ConceptScheme"));

        // Verify member relation (not narrower/broader)
        assert!(xml.contains("skos:member"));
        assert!(!xml.contains("skos:narrower"));
        assert!(!xml.contains("skos:broader"));

        // Verify labels are JSON-wrapped for Arches compatibility
        assert!(xml.contains("\"id\":"));
        assert!(xml.contains("\"value\":"));
        assert!(xml.contains("Test Collection"));
        assert!(xml.contains("Alt Label"));
        assert!(xml.contains("A scope note"));
    }

    #[test]
    fn test_round_trip_arches_collection() {
        // Parse Arches Collection
        let result = parse_skos_to_collections(TEST_ARCHES_COLLECTION, "http://localhost:8000/");
        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 1);

        let original = &collections[0];
        assert_eq!(original.node_type, SkosNodeType::Collection);

        // Serialize back to XML
        let xml = collection_to_skos_xml(original, "http://localhost:8000/");

        // Parse again
        let result2 = parse_skos_to_collections(&xml, "http://localhost:8000/");
        assert!(result2.is_ok());
        let collections2 = result2.unwrap();
        assert_eq!(collections2.len(), 1);

        let round_tripped = &collections2[0];

        // Should still be a Collection
        assert_eq!(round_tripped.node_type, SkosNodeType::Collection);

        // Should preserve prefLabel with ID
        assert!(round_tripped.pref_labels.contains_key("en-us"));
        assert_eq!(round_tripped.pref_labels["en-us"].value, "Test Collection");
        assert_eq!(
            round_tripped.pref_labels["en-us"].id,
            "956f8913-f728-4f82-b3ae-3aaf4ce7891a"
        );

        // Should preserve member count
        assert_eq!(round_tripped.concepts.len(), original.concepts.len());

        // Members should still be flat (no children)
        for concept in round_tripped.concepts.values() {
            assert!(concept.children.is_none());
        }
    }

    #[test]
    fn test_parse_arches_label_json() {
        // Test the JSON label parsing
        let (id, value) = parse_arches_label(
            r#"{"id": "uuid-123", "value": "Label Text"}"#,
            "fallback",
            "en",
        );
        assert_eq!(id, "uuid-123");
        assert_eq!(value, "Label Text");
    }

    #[test]
    fn test_parse_arches_label_plain() {
        // Test plain text label parsing
        let (id, value) = parse_arches_label("Plain Label", "fallback", "en");
        assert_ne!(id, "fallback"); // Should generate a hash-based ID
        assert_eq!(value, "Plain Label");
    }

    // ==========================================================================
    // Hierarchical Collection Tests
    // ==========================================================================

    /// Test data for a Collection with hierarchical concepts (narrower/broader)
    const TEST_HIERARCHICAL_COLLECTION: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:skos="http://www.w3.org/2004/02/skos/core#"
         xmlns:dcterms="http://purl.org/dc/terms/">
  <skos:Collection rdf:about="http://localhost:8000/hierarchical-collection">
    <skos:prefLabel xml:lang="en">{"id": "coll-label-1", "value": "Hierarchical Collection"}</skos:prefLabel>
    <skos:member>
      <skos:Concept rdf:about="http://localhost:8000/parent-concept"/>
    </skos:member>
    <skos:member>
      <skos:Concept rdf:about="http://localhost:8000/child-concept-1"/>
    </skos:member>
    <skos:member>
      <skos:Concept rdf:about="http://localhost:8000/child-concept-2"/>
    </skos:member>
  </skos:Collection>
  <skos:Concept rdf:about="http://localhost:8000/parent-concept">
    <skos:prefLabel xml:lang="en">Parent</skos:prefLabel>
    <skos:narrower rdf:resource="http://localhost:8000/child-concept-1"/>
    <skos:narrower rdf:resource="http://localhost:8000/child-concept-2"/>
  </skos:Concept>
  <skos:Concept rdf:about="http://localhost:8000/child-concept-1">
    <skos:prefLabel xml:lang="en">Child One</skos:prefLabel>
    <skos:broader rdf:resource="http://localhost:8000/parent-concept"/>
  </skos:Concept>
  <skos:Concept rdf:about="http://localhost:8000/child-concept-2">
    <skos:prefLabel xml:lang="en">Child Two</skos:prefLabel>
    <skos:broader rdf:resource="http://localhost:8000/parent-concept"/>
  </skos:Concept>
</rdf:RDF>"#;

    #[test]
    fn test_parse_hierarchical_collection() {
        let result =
            parse_skos_to_collections(TEST_HIERARCHICAL_COLLECTION, "http://localhost:8000/");
        assert!(result.is_ok());
        let collections = result.unwrap();

        // Should have one Collection
        assert_eq!(collections.len(), 1);
        let collection = &collections[0];

        // Should be a Collection type
        assert_eq!(collection.node_type, SkosNodeType::Collection);

        // Should have one top-level concept (the parent)
        assert_eq!(collection.concepts.len(), 1);

        // The parent should have children
        let parent = collection.concepts.values().next().unwrap();
        assert_eq!(parent.pref_labels["en"].value, "Parent");
        assert!(parent.children.is_some());

        let children = parent.children.as_ref().unwrap();
        assert_eq!(children.len(), 2);

        // Children should have correct labels
        let child_labels: Vec<&str> = children
            .iter()
            .map(|c| c.pref_labels["en"].value.as_str())
            .collect();
        assert!(child_labels.contains(&"Child One"));
        assert!(child_labels.contains(&"Child Two"));
    }

    #[test]
    fn test_serialize_hierarchical_collection_with_children() {
        // Create a Collection with hierarchical concepts
        let mut pref_labels = HashMap::new();
        pref_labels.insert(
            "en".to_string(),
            SkosValue {
                id: "label-1".to_string(),
                value: "Hierarchical Collection".to_string(),
            },
        );

        // Create child concepts
        let mut child1_labels = HashMap::new();
        child1_labels.insert(
            "en".to_string(),
            SkosValue {
                id: "child1-label".to_string(),
                value: "Child One".to_string(),
            },
        );
        let child1 = SkosConcept {
            id: "child-1".to_string(),
            uri: None,
            pref_labels: child1_labels,
            source: None,
            sort_order: None,
            children: None,
        };

        let mut child2_labels = HashMap::new();
        child2_labels.insert(
            "en".to_string(),
            SkosValue {
                id: "child2-label".to_string(),
                value: "Child Two".to_string(),
            },
        );
        let child2 = SkosConcept {
            id: "child-2".to_string(),
            uri: None,
            pref_labels: child2_labels,
            source: None,
            sort_order: None,
            children: None,
        };

        // Create parent with children
        let mut parent_labels = HashMap::new();
        parent_labels.insert(
            "en".to_string(),
            SkosValue {
                id: "parent-label".to_string(),
                value: "Parent".to_string(),
            },
        );
        let parent = SkosConcept {
            id: "parent".to_string(),
            uri: None,
            pref_labels: parent_labels,
            source: None,
            sort_order: None,
            children: Some(vec![child1, child2]),
        };

        let mut concepts = HashMap::new();
        concepts.insert("parent".to_string(), parent);

        let collection = SkosCollection {
            id: "coll-1".to_string(),
            uri: None,
            pref_labels,
            alt_labels: HashMap::new(),
            scope_notes: HashMap::new(),
            node_type: SkosNodeType::Collection,
            concepts,
            all_concepts: HashMap::new(),
            values: HashMap::new(),
        };

        let xml = collection_to_skos_xml(&collection, "http://localhost:8000/");

        // Should be a Collection
        assert!(xml.contains("skos:Collection"));

        // Should have 3 members (parent + 2 children) - all concepts listed as members
        // Each member has opening <skos:member> and closing </skos:member>, so count opening tags
        let member_count = xml.matches("<skos:member>").count();
        assert_eq!(
            member_count, 3,
            "Should list all concepts including children as members"
        );

        // Should have narrower/broader relationships on concepts
        assert!(
            xml.contains("skos:narrower"),
            "Parent should have narrower relationships"
        );
        assert!(
            xml.contains("skos:broader"),
            "Children should have broader relationships"
        );

        // Should contain all concept labels
        assert!(xml.contains("Parent"));
        assert!(xml.contains("Child One"));
        assert!(xml.contains("Child Two"));
    }

    #[test]
    fn test_round_trip_hierarchical_collection() {
        // Parse hierarchical Collection
        let result =
            parse_skos_to_collections(TEST_HIERARCHICAL_COLLECTION, "http://localhost:8000/");
        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 1);

        let original = &collections[0];
        assert_eq!(original.node_type, SkosNodeType::Collection);

        // Verify original has hierarchy
        assert_eq!(original.concepts.len(), 1);
        let original_parent = original.concepts.values().next().unwrap();
        assert!(original_parent.children.is_some());
        assert_eq!(original_parent.children.as_ref().unwrap().len(), 2);

        // Serialize to XML
        let xml = collection_to_skos_xml(original, "http://localhost:8000/");

        // Parse again
        let result2 = parse_skos_to_collections(&xml, "http://localhost:8000/");
        assert!(result2.is_ok());
        let collections2 = result2.unwrap();
        assert_eq!(collections2.len(), 1);

        let round_tripped = &collections2[0];

        // Should still be a Collection
        assert_eq!(round_tripped.node_type, SkosNodeType::Collection);

        // Should preserve hierarchy: one top-level concept with 2 children
        assert_eq!(round_tripped.concepts.len(), 1);
        let rt_parent = round_tripped.concepts.values().next().unwrap();
        assert!(
            rt_parent.children.is_some(),
            "Hierarchy should be preserved after round-trip"
        );
        assert_eq!(rt_parent.children.as_ref().unwrap().len(), 2);

        // Verify child labels preserved
        let child_labels: Vec<&str> = rt_parent
            .children
            .as_ref()
            .unwrap()
            .iter()
            .map(|c| c.pref_labels["en"].value.as_str())
            .collect();
        assert!(child_labels.contains(&"Child One"));
        assert!(child_labels.contains(&"Child Two"));
    }
}
