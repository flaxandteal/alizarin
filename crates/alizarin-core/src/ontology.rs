//! RDFS Ontology Parser and Validator
//!
//! Parses RDFS/XML ontology files (e.g. CIDOC CRM) and validates
//! ontology class and property constraints on graph mutations.
//!
//! Follows the same validation model as Arches:
//! - Every node's ontology class must exist in the loaded ontology
//! - Every edge's property must be valid for the domain→range class pair
//! - RDFS subclass hierarchies are expanded transitively

use rio_api::model::{Subject, Term, Triple};
use rio_api::parser::TriplesParser;
use rio_xml::RdfXmlParser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// RDF/RDFS namespace URIs
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_CLASS: &str = "http://www.w3.org/2000/01/rdf-schema#Class";
const RDFS_SUBCLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
const RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
const RDF_PROPERTY: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#Property";

/// Parsed from ontology_config.json (same format Arches uses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyConfig {
    pub base: String,
    #[serde(default)]
    pub base_name: String,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub base_version: String,
    #[serde(default)]
    pub base_id: String,
}

/// Per-class validation rules (mirrors Arches' OntologyClass.target)
#[derive(Debug, Clone, Default)]
pub struct ClassRules {
    /// Properties where this class is the domain: property → valid range classes
    pub down: HashMap<String, HashSet<String>>,
    /// Properties where this class is the range: property → valid domain classes
    pub up: HashMap<String, HashSet<String>>,
}

/// Ontology validation error detail
#[derive(Debug, Clone)]
pub enum OntologyValidationDetail {
    /// The ontology class URI is not known in the loaded ontology
    UnknownClass(String),
    /// The property URI is not known in the loaded ontology
    UnknownProperty(String),
    /// The property is not valid for the given domain class
    InvalidDomainForProperty {
        property: String,
        domain_class: String,
        /// The declared domain classes for this property (from the ontology)
        expected_domains: Vec<String>,
    },
    /// The range class is not valid for the given property on the domain class
    InvalidRangeForProperty {
        property: String,
        range_class: String,
        domain_class: String,
        /// The declared range classes for this property (from the ontology)
        expected_ranges: Vec<String>,
    },
}

/// Shorten a URI for display: "http://www.cidoc-crm.org/cidoc-crm/E70_Thing" → "E70_Thing"
fn short_uri(uri: &str) -> &str {
    uri.rsplit_once('/')
        .map(|(_, name)| name)
        .or_else(|| uri.rsplit_once('#').map(|(_, name)| name))
        .unwrap_or(uri)
}

impl std::fmt::Display for OntologyValidationDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OntologyValidationDetail::UnknownClass(class) => {
                write!(f, "Unknown ontology class: {}", class)
            }
            OntologyValidationDetail::UnknownProperty(prop) => {
                write!(f, "Unknown ontology property: {}", prop)
            }
            OntologyValidationDetail::InvalidDomainForProperty {
                property,
                domain_class,
                expected_domains,
            } => {
                let expected: Vec<&str> = expected_domains.iter().map(|s| short_uri(s)).collect();
                write!(
                    f,
                    "Property '{}' is not valid for domain class '{}' (declared domain: {})",
                    short_uri(property),
                    short_uri(domain_class),
                    if expected.is_empty() {
                        "none".to_string()
                    } else {
                        expected.join(", ")
                    }
                )
            }
            OntologyValidationDetail::InvalidRangeForProperty {
                property,
                range_class,
                domain_class,
                expected_ranges,
            } => {
                let expected: Vec<&str> = expected_ranges.iter().map(|s| short_uri(s)).collect();
                write!(
                    f,
                    "Class '{}' is not a valid range for property '{}' on domain '{}' (declared range: {})",
                    short_uri(range_class),
                    short_uri(property),
                    short_uri(domain_class),
                    if expected.is_empty() { "none".to_string() } else { expected.join(", ") }
                )
            }
        }
    }
}

/// Ontology loading/parsing error
#[derive(Debug, Clone)]
pub enum OntologyError {
    /// Error parsing RDF/XML
    ParseError(String),
}

impl std::fmt::Display for OntologyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OntologyError::ParseError(msg) => write!(f, "Ontology parse error: {}", msg),
        }
    }
}

impl std::error::Error for OntologyError {}

/// Intermediate data collected during RDF/XML parsing
#[derive(Debug, Default)]
struct RdfsParseData {
    /// All known class URIs
    classes: HashSet<String>,
    /// Direct subclass relationships: child → set of parents
    subclass_of: HashMap<String, HashSet<String>>,
    /// Property → domain class
    property_domains: HashMap<String, HashSet<String>>,
    /// Property → range class
    property_ranges: HashMap<String, HashSet<String>>,
    /// All known property URIs
    properties: HashSet<String>,
}

impl RdfsParseData {
    fn process_triple(&mut self, triple: Triple) {
        let subject_uri = match triple.subject {
            Subject::NamedNode(n) => n.iri.to_string(),
            _ => return,
        };

        let predicate = triple.predicate.iri;

        match predicate {
            RDF_TYPE => {
                if let Term::NamedNode(obj) = triple.object {
                    let obj_uri = obj.iri.to_string();
                    if obj_uri == RDFS_CLASS {
                        self.classes.insert(subject_uri);
                    } else if obj_uri == RDF_PROPERTY {
                        self.properties.insert(subject_uri);
                    }
                }
            }
            RDFS_SUBCLASS_OF => {
                if let Term::NamedNode(obj) = triple.object {
                    let parent = obj.iri.to_string();
                    // Both subject and object are classes
                    self.classes.insert(subject_uri.clone());
                    self.classes.insert(parent.clone());
                    self.subclass_of
                        .entry(subject_uri)
                        .or_default()
                        .insert(parent);
                }
            }
            RDFS_DOMAIN => {
                if let Term::NamedNode(obj) = triple.object {
                    let domain = obj.iri.to_string();
                    self.classes.insert(domain.clone());
                    self.properties.insert(subject_uri.clone());
                    self.property_domains
                        .entry(subject_uri)
                        .or_default()
                        .insert(domain);
                }
            }
            RDFS_RANGE => {
                if let Term::NamedNode(obj) = triple.object {
                    let range = obj.iri.to_string();
                    self.classes.insert(range.clone());
                    self.properties.insert(subject_uri.clone());
                    self.property_ranges
                        .entry(subject_uri)
                        .or_default()
                        .insert(range);
                }
            }
            _ => {}
        }
    }
}

/// Immutable ontology validator, constructed from parsed RDFS/XML files.
///
/// Validates that:
/// - Ontology classes exist in the loaded ontology
/// - Properties are valid for their domain class
/// - Range classes are valid targets for a given property on a domain
#[derive(Debug, Clone)]
pub struct OntologyValidator {
    known_classes: HashSet<String>,
    class_rules: HashMap<String, ClassRules>,
    /// Original declared domains per property (before subclass expansion)
    property_domains: HashMap<String, HashSet<String>>,
    /// Original declared ranges per property (before subclass expansion)
    property_ranges: HashMap<String, HashSet<String>>,
}

impl OntologyValidator {
    /// Parse one or more RDFS/XML strings and build a validator.
    ///
    /// Files are parsed in order; later files (extensions) can reference
    /// classes/properties from earlier files.
    pub fn from_rdfs_xml(files: &[&str]) -> Result<Self, OntologyError> {
        let mut data = RdfsParseData::default();

        for xml_content in files {
            let mut parser = RdfXmlParser::new(xml_content.as_bytes(), None);
            parser
                .parse_all(&mut |triple| {
                    data.process_triple(triple);
                    Ok(()) as Result<(), std::io::Error>
                })
                .map_err(|e| OntologyError::ParseError(format!("{}", e)))?;
        }

        // Compute transitive closure of subclass hierarchy
        // ancestors: class → all ancestor classes (transitive)
        let ancestors = compute_transitive_closure(&data.subclass_of);

        // Build descendants: class → all descendant classes (inverse of ancestors)
        let descendants = build_descendants(&ancestors);

        // Build class rules with expanded hierarchies
        let class_rules = build_class_rules(&data, &descendants);

        Ok(Self {
            known_classes: data.classes,
            class_rules,
            property_domains: data.property_domains,
            property_ranges: data.property_ranges,
        })
    }

    /// Check if a class URI is known in the loaded ontology.
    pub fn is_valid_class(&self, class_uri: &str) -> bool {
        self.known_classes.contains(class_uri)
    }

    /// Validate that a property is valid between domain and range classes.
    ///
    /// Returns Ok(()) if valid, or an OntologyValidationDetail describing the issue.
    pub fn validate_edge(
        &self,
        domain_class: &str,
        property: &str,
        range_class: &str,
    ) -> Result<(), OntologyValidationDetail> {
        // Skip validation if either class is empty (backward compat)
        if domain_class.is_empty() || property.is_empty() || range_class.is_empty() {
            return Ok(());
        }

        // Validate classes exist
        if !self.known_classes.contains(domain_class) {
            return Err(OntologyValidationDetail::UnknownClass(
                domain_class.to_string(),
            ));
        }
        if !self.known_classes.contains(range_class) {
            return Err(OntologyValidationDetail::UnknownClass(
                range_class.to_string(),
            ));
        }

        // Check domain class has this property
        if let Some(rules) = self.class_rules.get(domain_class) {
            if let Some(valid_ranges) = rules.down.get(property) {
                // Property exists for domain — check range
                if valid_ranges.contains(range_class) {
                    return Ok(());
                } else {
                    let expected_ranges: Vec<String> = self
                        .property_ranges
                        .get(property)
                        .map(|s| s.iter().cloned().collect())
                        .unwrap_or_default();
                    return Err(OntologyValidationDetail::InvalidRangeForProperty {
                        property: property.to_string(),
                        range_class: range_class.to_string(),
                        domain_class: domain_class.to_string(),
                        expected_ranges,
                    });
                }
            }
        }

        // Property not found in domain's rules
        let expected_domains: Vec<String> = self
            .property_domains
            .get(property)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        Err(OntologyValidationDetail::InvalidDomainForProperty {
            property: property.to_string(),
            domain_class: domain_class.to_string(),
            expected_domains,
        })
    }

    /// Get the number of known classes (useful for diagnostics).
    pub fn class_count(&self) -> usize {
        self.known_classes.len()
    }

    /// Get the number of classes that have rules (useful for diagnostics).
    pub fn rules_count(&self) -> usize {
        self.class_rules.len()
    }
}

/// Compute transitive closure of a directed graph.
///
/// Input: direct edges (child → set of parents)
/// Output: transitive closure (child → all ancestors)
fn compute_transitive_closure(
    direct: &HashMap<String, HashSet<String>>,
) -> HashMap<String, HashSet<String>> {
    let mut ancestors: HashMap<String, HashSet<String>> = HashMap::new();

    // Initialize with direct relationships
    for (child, parents) in direct {
        ancestors
            .entry(child.clone())
            .or_default()
            .extend(parents.iter().cloned());
    }

    // Iterative fixed-point: keep expanding until no changes
    loop {
        let mut changed = false;
        let keys: Vec<String> = ancestors.keys().cloned().collect();

        for key in keys {
            let current: Vec<String> = ancestors
                .get(&key)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_default();

            for ancestor in current {
                if let Some(grandparents) = ancestors.get(&ancestor).cloned() {
                    let entry = ancestors.entry(key.clone()).or_default();
                    for gp in grandparents {
                        if entry.insert(gp) {
                            changed = true;
                        }
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    ancestors
}

/// Build descendants map (inverse of ancestors).
///
/// Input: ancestors (class → all ancestor classes)
/// Output: descendants (class → all descendant classes, including self)
fn build_descendants(
    ancestors: &HashMap<String, HashSet<String>>,
) -> HashMap<String, HashSet<String>> {
    let mut descendants: HashMap<String, HashSet<String>> = HashMap::new();

    for (child, ancestor_set) in ancestors {
        for ancestor in ancestor_set {
            descendants
                .entry(ancestor.clone())
                .or_default()
                .insert(child.clone());
        }
    }

    descendants
}

/// Build per-class rules with expanded subclass hierarchies.
///
/// For each property with domain D and range R:
/// - D and all subclasses of D get the property in their "down" rules
/// - R and all subclasses of R get the property in their "up" rules
/// - The valid range classes include R and all subclasses of R
/// - The valid domain classes include D and all subclasses of D
fn build_class_rules(
    data: &RdfsParseData,
    descendants: &HashMap<String, HashSet<String>>,
) -> HashMap<String, ClassRules> {
    let mut rules: HashMap<String, ClassRules> = HashMap::new();

    for prop in &data.properties {
        let domains: Vec<String> = data
            .property_domains
            .get(prop)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        let ranges: Vec<String> = data
            .property_ranges
            .get(prop)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();

        if domains.is_empty() || ranges.is_empty() {
            continue;
        }

        // Expand domains: each declared domain + all its descendants
        let mut expanded_domains: HashSet<String> = HashSet::new();
        for d in &domains {
            expanded_domains.insert(d.clone());
            if let Some(desc) = descendants.get(d) {
                expanded_domains.extend(desc.iter().cloned());
            }
        }

        // Expand ranges: each declared range + all its descendants
        let mut expanded_ranges: HashSet<String> = HashSet::new();
        for r in &ranges {
            expanded_ranges.insert(r.clone());
            if let Some(desc) = descendants.get(r) {
                expanded_ranges.extend(desc.iter().cloned());
            }
        }

        // Add "down" rules: each expanded domain class gets this property → expanded ranges
        for domain_class in &expanded_domains {
            rules
                .entry(domain_class.clone())
                .or_default()
                .down
                .entry(prop.clone())
                .or_default()
                .extend(expanded_ranges.iter().cloned());
        }

        // Add "up" rules: each expanded range class gets this property → expanded domains
        for range_class in &expanded_ranges {
            rules
                .entry(range_class.clone())
                .or_default()
                .up
                .entry(prop.clone())
                .or_default()
                .extend(expanded_domains.iter().cloned());
        }
    }

    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_ONTOLOGY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xml:base="http://example.org/test/">

<rdfs:Class rdf:about="A">
    <rdfs:label>Class A (root)</rdfs:label>
</rdfs:Class>

<rdfs:Class rdf:about="B">
    <rdfs:subClassOf rdf:resource="A"/>
</rdfs:Class>

<rdfs:Class rdf:about="C">
    <rdfs:subClassOf rdf:resource="B"/>
</rdfs:Class>

<rdfs:Class rdf:about="D">
    <rdfs:label>Class D (unrelated)</rdfs:label>
</rdfs:Class>

<rdf:Property rdf:about="P1_relates_to">
    <rdfs:domain rdf:resource="A"/>
    <rdfs:range rdf:resource="D"/>
</rdf:Property>

<rdf:Property rdf:about="P2_has_part">
    <rdfs:domain rdf:resource="B"/>
    <rdfs:range rdf:resource="A"/>
</rdf:Property>

</rdf:RDF>"#;

    #[test]
    fn test_parse_minimal_ontology() {
        let validator = OntologyValidator::from_rdfs_xml(&[MINIMAL_ONTOLOGY]).unwrap();
        assert_eq!(validator.class_count(), 4); // A, B, C, D
    }

    #[test]
    fn test_valid_class() {
        let validator = OntologyValidator::from_rdfs_xml(&[MINIMAL_ONTOLOGY]).unwrap();
        assert!(validator.is_valid_class("http://example.org/test/A"));
        assert!(validator.is_valid_class("http://example.org/test/B"));
        assert!(validator.is_valid_class("http://example.org/test/C"));
        assert!(validator.is_valid_class("http://example.org/test/D"));
        assert!(!validator.is_valid_class("http://example.org/test/Banana"));
        assert!(!validator.is_valid_class("banana"));
    }

    #[test]
    fn test_direct_domain_range() {
        let validator = OntologyValidator::from_rdfs_xml(&[MINIMAL_ONTOLOGY]).unwrap();
        // P1_relates_to: domain A → range D (direct)
        assert!(validator
            .validate_edge(
                "http://example.org/test/A",
                "http://example.org/test/P1_relates_to",
                "http://example.org/test/D",
            )
            .is_ok());
    }

    #[test]
    fn test_subclass_inherits_domain_property() {
        let validator = OntologyValidator::from_rdfs_xml(&[MINIMAL_ONTOLOGY]).unwrap();
        // B ⊑ A, so P1_relates_to should be valid for domain B
        assert!(validator
            .validate_edge(
                "http://example.org/test/B",
                "http://example.org/test/P1_relates_to",
                "http://example.org/test/D",
            )
            .is_ok());

        // C ⊑ B ⊑ A, so P1_relates_to should be valid for domain C (transitive)
        assert!(validator
            .validate_edge(
                "http://example.org/test/C",
                "http://example.org/test/P1_relates_to",
                "http://example.org/test/D",
            )
            .is_ok());
    }

    #[test]
    fn test_subclass_valid_as_range() {
        let validator = OntologyValidator::from_rdfs_xml(&[MINIMAL_ONTOLOGY]).unwrap();
        // P2_has_part: domain B → range A
        // B is a subclass of A, so B should also be valid as range
        assert!(validator
            .validate_edge(
                "http://example.org/test/B",
                "http://example.org/test/P2_has_part",
                "http://example.org/test/A",
            )
            .is_ok());
        assert!(validator
            .validate_edge(
                "http://example.org/test/B",
                "http://example.org/test/P2_has_part",
                "http://example.org/test/B",
            )
            .is_ok());
        assert!(validator
            .validate_edge(
                "http://example.org/test/B",
                "http://example.org/test/P2_has_part",
                "http://example.org/test/C",
            )
            .is_ok());
    }

    #[test]
    fn test_invalid_domain_rejected() {
        let validator = OntologyValidator::from_rdfs_xml(&[MINIMAL_ONTOLOGY]).unwrap();
        // P2_has_part has domain B. A is NOT a subclass of B, so A is invalid as domain.
        let result = validator.validate_edge(
            "http://example.org/test/A",
            "http://example.org/test/P2_has_part",
            "http://example.org/test/A",
        );
        match result {
            Err(OntologyValidationDetail::InvalidDomainForProperty {
                ref expected_domains,
                ..
            }) => {
                assert!(
                    expected_domains.contains(&"http://example.org/test/B".to_string()),
                    "Expected domains should contain B, got: {:?}",
                    expected_domains
                );
                // Check the Display output includes the declared domain
                let msg = format!("{}", result.unwrap_err());
                assert!(msg.contains("declared domain: B"), "Got: {}", msg);
            }
            other => panic!("Expected InvalidDomainForProperty, got: {:?}", other),
        }
    }

    #[test]
    fn test_invalid_range_rejected() {
        let validator = OntologyValidator::from_rdfs_xml(&[MINIMAL_ONTOLOGY]).unwrap();
        // P1_relates_to: domain A → range D
        // B is NOT D or a subclass of D, so B is invalid as range.
        let result = validator.validate_edge(
            "http://example.org/test/A",
            "http://example.org/test/P1_relates_to",
            "http://example.org/test/B",
        );
        match result {
            Err(OntologyValidationDetail::InvalidRangeForProperty {
                ref expected_ranges,
                ..
            }) => {
                assert!(
                    expected_ranges.contains(&"http://example.org/test/D".to_string()),
                    "Expected ranges should contain D, got: {:?}",
                    expected_ranges
                );
                let msg = format!("{}", result.unwrap_err());
                assert!(msg.contains("declared range: D"), "Got: {}", msg);
            }
            other => panic!("Expected InvalidRangeForProperty, got: {:?}", other),
        }
    }

    #[test]
    fn test_unknown_class_rejected() {
        let validator = OntologyValidator::from_rdfs_xml(&[MINIMAL_ONTOLOGY]).unwrap();
        let result = validator.validate_edge(
            "banana",
            "http://example.org/test/P1_relates_to",
            "http://example.org/test/D",
        );
        assert!(matches!(
            result,
            Err(OntologyValidationDetail::UnknownClass(ref c)) if c == "banana"
        ));
    }

    #[test]
    fn test_empty_values_skip_validation() {
        let validator = OntologyValidator::from_rdfs_xml(&[MINIMAL_ONTOLOGY]).unwrap();
        // Empty domain, property, or range should pass (backward compat)
        assert!(validator
            .validate_edge(
                "",
                "http://example.org/test/P1_relates_to",
                "http://example.org/test/D"
            )
            .is_ok());
        assert!(validator
            .validate_edge("http://example.org/test/A", "", "http://example.org/test/D")
            .is_ok());
        assert!(validator
            .validate_edge(
                "http://example.org/test/A",
                "http://example.org/test/P1_relates_to",
                ""
            )
            .is_ok());
    }

    #[test]
    fn test_multiple_files() {
        // Extension adds a new class E that subclasses D
        let extension = r#"<?xml version="1.0" encoding="UTF-8"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xml:base="http://example.org/test/">

<rdfs:Class rdf:about="E">
    <rdfs:subClassOf rdf:resource="D"/>
</rdfs:Class>

</rdf:RDF>"#;

        let validator = OntologyValidator::from_rdfs_xml(&[MINIMAL_ONTOLOGY, extension]).unwrap();
        assert_eq!(validator.class_count(), 5); // A, B, C, D, E

        // E ⊑ D, so E should be valid as range for P1_relates_to (which has range D)
        assert!(validator
            .validate_edge(
                "http://example.org/test/A",
                "http://example.org/test/P1_relates_to",
                "http://example.org/test/E",
            )
            .is_ok());
    }

    #[test]
    fn test_cidoc_crm_loads() {
        let base = include_str!("../../../tests/data/ontologies/cidoc_crm/cidoc_crm_v6.2.xml");
        let validator = OntologyValidator::from_rdfs_xml(&[base]).unwrap();

        // CIDOC CRM v6.2 should have a substantial number of classes
        assert!(
            validator.class_count() > 50,
            "Expected >50 classes, got {}",
            validator.class_count()
        );

        // Known classes
        assert!(validator.is_valid_class("http://www.cidoc-crm.org/cidoc-crm/E1_CRM_Entity"));
        assert!(validator.is_valid_class("http://www.cidoc-crm.org/cidoc-crm/E53_Place"));
        assert!(validator.is_valid_class("http://www.cidoc-crm.org/cidoc-crm/E55_Type"));

        // P2_has_type: domain E1_CRM_Entity → range E55_Type
        assert!(validator
            .validate_edge(
                "http://www.cidoc-crm.org/cidoc-crm/E1_CRM_Entity",
                "http://www.cidoc-crm.org/cidoc-crm/P2_has_type",
                "http://www.cidoc-crm.org/cidoc-crm/E55_Type",
            )
            .is_ok());

        // E53_Place ⊑ ... ⊑ E1_CRM_Entity, so P2_has_type should work for E53_Place too
        assert!(validator
            .validate_edge(
                "http://www.cidoc-crm.org/cidoc-crm/E53_Place",
                "http://www.cidoc-crm.org/cidoc-crm/P2_has_type",
                "http://www.cidoc-crm.org/cidoc-crm/E55_Type",
            )
            .is_ok());

        // Invalid: banana is not a class
        assert!(matches!(
            validator.validate_edge(
                "banana",
                "http://www.cidoc-crm.org/cidoc-crm/P2_has_type",
                "http://www.cidoc-crm.org/cidoc-crm/E55_Type",
            ),
            Err(OntologyValidationDetail::UnknownClass(ref c)) if c == "banana"
        ));
    }

    #[test]
    fn test_cidoc_crm_with_extensions() {
        let base = include_str!("../../../tests/data/ontologies/cidoc_crm/cidoc_crm_v6.2.xml");
        let crmsci =
            include_str!("../../../tests/data/ontologies/cidoc_crm/CRMsci_v1.2.3.rdfs.xml");
        let crmdig =
            include_str!("../../../tests/data/ontologies/cidoc_crm/CRMdig_v3.2.1.rdfs.xml");

        let validator = OntologyValidator::from_rdfs_xml(&[base, crmsci, crmdig]).unwrap();

        // CRMsci classes should be available
        assert!(validator.is_valid_class("http://www.ics.forth.gr/isl/CRMsci/S4_Observation"));

        // CRMdig classes should be available
        assert!(validator.is_valid_class("http://www.ics.forth.gr/isl/CRMdig/D1_Digital_Object"));
    }
}
