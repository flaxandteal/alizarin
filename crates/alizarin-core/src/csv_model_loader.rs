//! CSV Model Loader
//!
//! Parses a 3-CSV format (graph.csv, nodes.csv, collections.csv) into
//! [`GraphInstruction`]s and [`SkosCollection`]s, suitable for building
//! an Arches resource model via the existing graph mutator pipeline.
//!
//! ## CSV Format
//!
//! ### graph.csv (single data row)
//! `name,ontology_class,author,description,is_resource`
//!
//! ### nodes.csv
//! `parent_alias,alias,name,datatype,cardinality,ontology_class,parent_property,description,collection_name,required,searchable,exportable,sortorder`
//!
//! ### collections.csv
//! `collection_name,concept_label,parent_label,sort_order`
//!
//! ## Example
//! ```rust,ignore
//! use alizarin_core::csv_model_loader::{parse_model_csvs, build_graph_from_model_csvs};
//!
//! let result = build_graph_from_model_csvs(graph_csv, nodes_csv, Some(collections_csv), Default::default());
//! let (graph, collections) = result.unwrap();
//! ```

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::graph_mutator::{GraphInstruction, MutatorOptions};
use crate::rdm_namespace::{
    generate_collection_uuid, generate_concept_uuid, generate_value_uuid, parse_rdm_namespace,
};
use crate::skos::{SkosCollection, SkosConcept, SkosNodeType, SkosValue};
use crate::StaticGraph;

/// Severity level for a diagnostic
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

/// A single validation diagnostic
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CsvModelDiagnostic {
    pub level: DiagnosticLevel,
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
}

impl std::fmt::Display for CsvModelDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self.level {
            DiagnosticLevel::Error => "ERROR",
            DiagnosticLevel::Warning => "WARN",
        };
        if let Some(line) = self.line {
            write!(f, "[{}] {}:{}: {}", level, self.file, line, self.message)
        } else {
            write!(f, "[{}] {}: {}", level, self.file, self.message)
        }
    }
}

/// Error type for CSV model loading
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CsvModelError {
    pub diagnostics: Vec<CsvModelDiagnostic>,
}

impl std::fmt::Display for CsvModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for d in &self.diagnostics {
            writeln!(f, "{}", d)?;
        }
        Ok(())
    }
}

impl std::error::Error for CsvModelError {}

/// Parsed row from graph.csv
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphRow {
    pub name: String,
    pub ontology_class: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub is_resource: Option<bool>,
}

/// Parsed row from nodes.csv
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeRow {
    pub parent_alias: Option<String>,
    pub alias: String,
    pub name: String,
    pub datatype: String,
    pub cardinality: String,
    pub ontology_class: String,
    pub parent_property: String,
    pub description: Option<String>,
    pub collection_name: Option<String>,
    pub required: Option<bool>,
    pub searchable: Option<bool>,
    pub exportable: Option<bool>,
    pub sortorder: Option<i32>,
}

/// Parsed row from collections.csv
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionRow {
    pub collection_name: String,
    pub concept_label: String,
    pub parent_label: Option<String>,
    pub sort_order: Option<i32>,
}

/// Result of parsing the 3-CSV bundle
#[derive(Debug, Clone)]
pub struct ModelCsvBundle {
    pub graph: GraphRow,
    pub nodes: Vec<NodeRow>,
    pub collections: Vec<CollectionRow>,
}

const VALID_DATATYPES: &[&str] = &[
    "semantic",
    "string",
    "concept",
    "concept-list",
    "number",
    "date",
    "boolean",
    "geojson-feature-collection",
    "domain-value",
    "domain-value-list",
    "file-list",
    "resource-instance",
    "resource-instance-list",
];

const CRM_PREFIX: &str = "http://www.cidoc-crm.org/cidoc-crm/";

/// Split a CSV `ontology_class` cell into individual URIs. Accepts pipe-separated
/// values so a single model can declare classes from more than one ontology.
/// Empty entries and whitespace are trimmed out.
fn split_class_cell(raw: &str) -> Vec<String> {
    raw.split('|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn get_field<'a>(
    record: &'a csv::StringRecord,
    headers: &csv::StringRecord,
    name: &str,
) -> Option<&'a str> {
    headers
        .iter()
        .position(|h| h == name)
        .and_then(|i| record.get(i))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
}

fn get_field_required<'a>(
    record: &'a csv::StringRecord,
    headers: &csv::StringRecord,
    name: &str,
    file: &str,
    line: usize,
    diagnostics: &mut Vec<CsvModelDiagnostic>,
) -> Option<&'a str> {
    match get_field(record, headers, name) {
        Some(v) => Some(v),
        None => {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: file.to_string(),
                line: Some(line),
                message: format!("missing required field \"{}\"", name),
            });
            None
        }
    }
}

/// Parse the three CSV strings into a [`ModelCsvBundle`].
///
/// Returns diagnostics for parse errors. If any error-level diagnostics
/// are present, the bundle may be incomplete but is still returned for
/// reporting purposes.
pub fn parse_model_csvs(
    graph_csv: &str,
    nodes_csv: &str,
    collections_csv: Option<&str>,
) -> Result<(ModelCsvBundle, Vec<CsvModelDiagnostic>), CsvModelError> {
    let mut diagnostics = Vec::new();

    // --- graph.csv ---
    let graph = parse_graph_csv(graph_csv, &mut diagnostics)?;

    // --- nodes.csv ---
    let nodes = parse_nodes_csv(nodes_csv, &mut diagnostics);

    // --- collections.csv ---
    let collections = if let Some(csv) = collections_csv {
        parse_collections_csv(csv, &mut diagnostics)
    } else {
        Vec::new()
    };

    let bundle = ModelCsvBundle {
        graph,
        nodes,
        collections,
    };
    Ok((bundle, diagnostics))
}

fn parse_graph_csv(
    csv_text: &str,
    diagnostics: &mut Vec<CsvModelDiagnostic>,
) -> Result<GraphRow, CsvModelError> {
    let mut reader = csv::Reader::from_reader(csv_text.as_bytes());
    let headers = reader
        .headers()
        .map_err(|e| CsvModelError {
            diagnostics: vec![CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "graph.csv".to_string(),
                line: Some(1),
                message: format!("failed to parse headers: {}", e),
            }],
        })?
        .clone();

    let record = reader
        .records()
        .next()
        .ok_or_else(|| CsvModelError {
            diagnostics: vec![CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "graph.csv".to_string(),
                line: None,
                message: "expected exactly 1 data row".to_string(),
            }],
        })?
        .map_err(|e| CsvModelError {
            diagnostics: vec![CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "graph.csv".to_string(),
                line: Some(2),
                message: format!("failed to parse row: {}", e),
            }],
        })?;

    let name = get_field_required(&record, &headers, "name", "graph.csv", 2, diagnostics)
        .unwrap_or("")
        .to_string();

    if name.is_empty() {
        return Err(CsvModelError {
            diagnostics: vec![CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "graph.csv".to_string(),
                line: Some(2),
                message: "\"name\" is required".to_string(),
            }],
        });
    }

    Ok(GraphRow {
        name,
        ontology_class: get_field(&record, &headers, "ontology_class").map(String::from),
        author: get_field(&record, &headers, "author").map(String::from),
        description: get_field(&record, &headers, "description").map(String::from),
        is_resource: get_field(&record, &headers, "is_resource").map(|v| v == "true"),
    })
}

fn parse_nodes_csv(csv_text: &str, diagnostics: &mut Vec<CsvModelDiagnostic>) -> Vec<NodeRow> {
    let mut reader = csv::Reader::from_reader(csv_text.as_bytes());
    let headers = match reader.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "nodes.csv".to_string(),
                line: Some(1),
                message: format!("failed to parse headers: {}", e),
            });
            return Vec::new();
        }
    };

    let mut rows = Vec::new();
    for (i, result) in reader.records().enumerate() {
        let line = i + 2;
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Error,
                    file: "nodes.csv".to_string(),
                    line: Some(line),
                    message: format!("failed to parse row: {}", e),
                });
                continue;
            }
        };

        let alias = get_field_required(&record, &headers, "alias", "nodes.csv", line, diagnostics)
            .unwrap_or("")
            .to_string();
        let name = get_field_required(&record, &headers, "name", "nodes.csv", line, diagnostics)
            .unwrap_or("")
            .to_string();
        let datatype = get_field_required(
            &record,
            &headers,
            "datatype",
            "nodes.csv",
            line,
            diagnostics,
        )
        .unwrap_or("")
        .to_string();
        let cardinality = get_field_required(
            &record,
            &headers,
            "cardinality",
            "nodes.csv",
            line,
            diagnostics,
        )
        .unwrap_or("1")
        .to_string();
        let ontology_class = get_field_required(
            &record,
            &headers,
            "ontology_class",
            "nodes.csv",
            line,
            diagnostics,
        )
        .unwrap_or("")
        .to_string();
        let parent_property = get_field_required(
            &record,
            &headers,
            "parent_property",
            "nodes.csv",
            line,
            diagnostics,
        )
        .unwrap_or("")
        .to_string();

        if alias.is_empty() {
            continue; // already reported
        }

        rows.push(NodeRow {
            parent_alias: get_field(&record, &headers, "parent_alias").map(String::from),
            alias,
            name,
            datatype,
            cardinality,
            ontology_class,
            parent_property,
            description: get_field(&record, &headers, "description").map(String::from),
            collection_name: get_field(&record, &headers, "collection_name").map(String::from),
            required: get_field(&record, &headers, "required").map(|v| v == "true"),
            searchable: get_field(&record, &headers, "searchable").map(|v| v != "false"),
            exportable: get_field(&record, &headers, "exportable").map(|v| v == "true"),
            sortorder: get_field(&record, &headers, "sortorder").and_then(|v| v.parse().ok()),
        });
    }
    rows
}

fn parse_collections_csv(
    csv_text: &str,
    diagnostics: &mut Vec<CsvModelDiagnostic>,
) -> Vec<CollectionRow> {
    let mut reader = csv::Reader::from_reader(csv_text.as_bytes());
    let headers = match reader.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "collections.csv".to_string(),
                line: Some(1),
                message: format!("failed to parse headers: {}", e),
            });
            return Vec::new();
        }
    };

    let mut rows = Vec::new();
    for (i, result) in reader.records().enumerate() {
        let line = i + 2;
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Error,
                    file: "collections.csv".to_string(),
                    line: Some(line),
                    message: format!("failed to parse row: {}", e),
                });
                continue;
            }
        };

        let collection_name = get_field_required(
            &record,
            &headers,
            "collection_name",
            "collections.csv",
            line,
            diagnostics,
        )
        .unwrap_or("")
        .to_string();
        let concept_label = get_field_required(
            &record,
            &headers,
            "concept_label",
            "collections.csv",
            line,
            diagnostics,
        )
        .unwrap_or("")
        .to_string();

        if collection_name.is_empty() || concept_label.is_empty() {
            continue;
        }

        rows.push(CollectionRow {
            collection_name,
            concept_label,
            parent_label: get_field(&record, &headers, "parent_label").map(String::from),
            sort_order: get_field(&record, &headers, "sort_order").and_then(|v| v.parse().ok()),
        });
    }
    rows
}

/// Validate a parsed [`ModelCsvBundle`] without building anything.
///
/// Returns all diagnostics (errors and warnings). Useful for
/// checking CSV output from the arches-model skill.
pub fn validate_model_csvs(bundle: &ModelCsvBundle) -> Vec<CsvModelDiagnostic> {
    let mut diagnostics = Vec::new();

    // --- Validate nodes ---
    let mut aliases: HashSet<&str> = HashSet::new();
    let mut parent_aliases: HashSet<&str> = HashSet::new();
    let mut collection_refs: HashSet<&str> = HashSet::new();

    for (i, node) in bundle.nodes.iter().enumerate() {
        let line = i + 2;

        // Unique alias
        if !aliases.insert(&node.alias) {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "nodes.csv".to_string(),
                line: Some(line),
                message: format!("duplicate alias \"{}\"", node.alias),
            });
        }

        // Valid datatype
        if !VALID_DATATYPES.contains(&node.datatype.as_str()) {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "nodes.csv".to_string(),
                line: Some(line),
                message: format!("invalid datatype \"{}\"", node.datatype),
            });
        }

        // Valid cardinality
        if node.cardinality != "1" && node.cardinality != "n" {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "nodes.csv".to_string(),
                line: Some(line),
                message: format!(
                    "invalid cardinality \"{}\" (must be \"1\" or \"n\")",
                    node.cardinality
                ),
            });
        }

        // Ontology URI prefix — warn per class if any are outside CIDOC-CRM.
        // Multi-class cells use `|` as separator so each entry is checked
        // independently. This stays a warning so non-CRM ontologies are allowed.
        for class in split_class_cell(&node.ontology_class) {
            if !class.starts_with(CRM_PREFIX) {
                diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Warning,
                    file: "nodes.csv".to_string(),
                    line: Some(line),
                    message: format!("ontology_class \"{}\" does not use CIDOC-CRM prefix", class),
                });
            }
        }
        if !node.parent_property.starts_with(CRM_PREFIX) && !node.parent_property.is_empty() {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Warning,
                file: "nodes.csv".to_string(),
                line: Some(line),
                message: format!(
                    "parent_property \"{}\" does not use CIDOC-CRM prefix",
                    node.parent_property
                ),
            });
        }

        // Concept nodes must have collection_name
        if (node.datatype == "concept" || node.datatype == "concept-list")
            && node.collection_name.is_none()
        {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Warning,
                file: "nodes.csv".to_string(),
                line: Some(line),
                message: format!("concept node \"{}\" has no collection_name", node.alias),
            });
        }

        if let Some(ref cn) = node.collection_name {
            collection_refs.insert(cn.as_str());
        }

        if let Some(ref pa) = node.parent_alias {
            parent_aliases.insert(pa.as_str());
        }
    }

    // Dangling parent_alias references
    for (i, node) in bundle.nodes.iter().enumerate() {
        if let Some(ref pa) = node.parent_alias {
            if !aliases.contains(pa.as_str()) {
                diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Error,
                    file: "nodes.csv".to_string(),
                    line: Some(i + 2),
                    message: format!("parent_alias \"{}\" not found in defined aliases", pa),
                });
            }
        }
    }

    // Semantic nodes should have children
    for node in &bundle.nodes {
        if node.datatype == "semantic" && !parent_aliases.contains(node.alias.as_str()) {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Warning,
                file: "nodes.csv".to_string(),
                line: None,
                message: format!("semantic node \"{}\" has no children", node.alias),
            });
        }
    }

    // --- Validate collections ---
    let mut collection_names: HashSet<&str> = HashSet::new();
    let mut concepts_by_collection: HashMap<&str, HashSet<&str>> = HashMap::new();

    for (i, row) in bundle.collections.iter().enumerate() {
        let line = i + 2;
        collection_names.insert(&row.collection_name);

        let labels = concepts_by_collection
            .entry(&row.collection_name)
            .or_default();
        labels.insert(&row.concept_label);

        // Check parent_label references within same collection
        if let Some(ref parent) = row.parent_label {
            if !labels.contains(parent.as_str()) {
                // Parent may appear later in file, but we can still flag forward refs
                // that never resolve. We'll do a second pass below.
            }
            // Check for self-reference
            if parent == &row.concept_label {
                diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Error,
                    file: "collections.csv".to_string(),
                    line: Some(line),
                    message: format!(
                        "concept \"{}\" references itself as parent",
                        row.concept_label
                    ),
                });
            }
        }
    }

    // Second pass: check parent_label references resolve within collection
    for (i, row) in bundle.collections.iter().enumerate() {
        if let Some(ref parent) = row.parent_label {
            if let Some(labels) = concepts_by_collection.get(row.collection_name.as_str()) {
                if !labels.contains(parent.as_str()) {
                    diagnostics.push(CsvModelDiagnostic {
                        level: DiagnosticLevel::Error,
                        file: "collections.csv".to_string(),
                        line: Some(i + 2),
                        message: format!(
                            "parent_label \"{}\" not found in collection \"{}\"",
                            parent, row.collection_name
                        ),
                    });
                }
            }
        }
    }

    // Cross-reference: nodes.csv collection_name → collections.csv.
    //
    // Warning (not Error) because many Arches projects load concept collections from
    // external SKOS XML at RDM-load time rather than declaring them inline in the
    // model CSVs. In that workflow a node can legitimately reference a collection by
    // name without a matching row here; the real rdmCollection UUID is resolved at
    // Arches startup from the SKOS import, not from this CSV.
    for cn in &collection_refs {
        if !collection_names.contains(cn) {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Warning,
                file: "nodes.csv".to_string(),
                line: None,
                message: format!(
                    "references collection \"{}\" but it is not defined in collections.csv (expected if loaded from external SKOS)",
                    cn
                ),
            });
        }
    }

    // Graph-level ontology check — warn per class, allow pipe-separated lists.
    if let Some(ref oc) = bundle.graph.ontology_class {
        for class in split_class_cell(oc) {
            if !class.starts_with(CRM_PREFIX) {
                diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Warning,
                    file: "graph.csv".to_string(),
                    line: None,
                    message: format!("ontology_class \"{}\" does not use CIDOC-CRM prefix", class),
                });
            }
        }
    }

    diagnostics
}

/// Convert a [`ModelCsvBundle`] to [`GraphInstruction`]s.
///
/// The returned instructions start with `create_model` and are followed
/// by `add_node` instructions in topological order (parents before children).
///
/// # Arguments
/// * `bundle` - Parsed CSV bundle
/// * `rdm_namespace` - RDM namespace string (UUID or URL), used for deterministic
///   collection/concept ID generation. Must match the namespace configured in
///   the target Arches instance.
pub fn model_csvs_to_instructions(
    bundle: &ModelCsvBundle,
    rdm_namespace: &str,
) -> Result<Vec<GraphInstruction>, CsvModelError> {
    let ns = parse_rdm_namespace(rdm_namespace).map_err(|e| CsvModelError {
        diagnostics: vec![CsvModelDiagnostic {
            level: DiagnosticLevel::Error,
            file: "(namespace)".to_string(),
            line: None,
            message: e,
        }],
    })?;

    let mut instructions = Vec::new();

    // Slugify graph name for root alias
    let root_alias = crate::graph_mutator::slugify(&bundle.graph.name);

    // create_model instruction
    let mut create = GraphInstruction::new("create_model", &root_alias, "");
    create = create.with_str("name", &bundle.graph.name);
    if let Some(ref oc) = bundle.graph.ontology_class {
        let classes = split_class_cell(oc);
        if classes.len() == 1 {
            create = create.with_str("ontology_class", &classes[0]);
        } else if !classes.is_empty() {
            create = create.with_param(
                "ontology_class",
                serde_json::Value::Array(
                    classes.into_iter().map(serde_json::Value::String).collect(),
                ),
            );
        }
    }
    instructions.push(create);

    // Build collection name→id map for concept nodes
    let collection_ids = build_collection_id_map(bundle, &ns);

    // Topologically sort nodes (parents before children)
    let sorted = topological_sort(&bundle.nodes);

    // add_node instructions
    for node in &sorted {
        let subject = match &node.parent_alias {
            Some(pa) => pa.as_str(),
            None => root_alias.as_str(),
        };

        let mut instr = GraphInstruction::new("add_node", subject, &node.alias);
        instr = instr.with_str("name", &node.name);
        instr = instr.with_str("datatype", &node.datatype);
        instr = instr.with_str("cardinality", &node.cardinality);
        // Multi-class cells are pipe-separated; pass as array when more than one.
        let classes = split_class_cell(&node.ontology_class);
        if classes.len() == 1 {
            instr = instr.with_str("ontology_class", &classes[0]);
        } else if !classes.is_empty() {
            instr = instr.with_param(
                "ontology_class",
                serde_json::Value::Array(
                    classes.into_iter().map(serde_json::Value::String).collect(),
                ),
            );
        }
        instr = instr.with_str("parent_property", &node.parent_property);

        if let Some(ref desc) = node.description {
            instr = instr.with_str("description", desc);
        }
        if let Some(req) = node.required {
            instr = instr.with_param("isrequired", serde_json::Value::Bool(req));
        }
        if let Some(search) = node.searchable {
            instr = instr.with_param("issearchable", serde_json::Value::Bool(search));
        }
        if let Some(exp) = node.exportable {
            instr = instr.with_param("exportable", serde_json::Value::Bool(exp));
        }
        if let Some(so) = node.sortorder {
            instr = instr.with_param("sortorder", serde_json::Value::Number(so.into()));
        }

        // For concept nodes, attach collection ID as config
        if node.datatype == "concept" || node.datatype == "concept-list" {
            if let Some(ref cn) = node.collection_name {
                if let Some(cid) = collection_ids.get(cn.as_str()) {
                    let config = serde_json::json!({ "rdmCollection": cid });
                    instr = instr.with_param("config", config);
                }
            }
        }

        instructions.push(instr);
    }

    Ok(instructions)
}

/// Build [`SkosCollection`]s from the collections rows in a [`ModelCsvBundle`].
///
/// # Arguments
/// * `bundle` - Parsed CSV bundle
/// * `rdm_namespace` - RDM namespace string (UUID or URL)
pub fn model_csvs_to_collections(
    bundle: &ModelCsvBundle,
    rdm_namespace: &str,
) -> Result<Vec<SkosCollection>, CsvModelError> {
    let ns = parse_rdm_namespace(rdm_namespace).map_err(|e| CsvModelError {
        diagnostics: vec![CsvModelDiagnostic {
            level: DiagnosticLevel::Error,
            file: "(namespace)".to_string(),
            line: None,
            message: e,
        }],
    })?;

    // Group rows by collection_name
    let mut grouped: HashMap<&str, Vec<&CollectionRow>> = HashMap::new();
    for row in &bundle.collections {
        grouped.entry(&row.collection_name).or_default().push(row);
    }

    let mut collections = Vec::new();
    for (name, rows) in &grouped {
        let coll_uuid = generate_collection_uuid(&ns, name);
        let collection_id = coll_uuid.to_string();

        let label_value_id = generate_value_uuid(&collection_id, name, "en");
        let pref_labels = {
            let mut m = HashMap::new();
            m.insert(
                "en".to_string(),
                SkosValue {
                    id: label_value_id.to_string(),
                    value: name.to_string(),
                },
            );
            m
        };

        // Build concept hierarchy: first create all concepts, then link children
        let mut concept_map: HashMap<&str, SkosConcept> = HashMap::new();
        let mut all_concepts: HashMap<String, SkosConcept> = HashMap::new();
        let mut values: HashMap<String, SkosValue> = HashMap::new();

        // First pass: create all concepts
        for row in rows {
            let concept_uuid = generate_concept_uuid(&coll_uuid, &row.concept_label);
            let concept_id = concept_uuid.to_string();

            let label_vid = generate_value_uuid(&concept_id, &row.concept_label, "en");
            let concept = SkosConcept {
                id: concept_id.clone(),
                uri: None,
                pref_labels: {
                    let mut m = HashMap::new();
                    m.insert(
                        "en".to_string(),
                        SkosValue {
                            id: label_vid.to_string(),
                            value: row.concept_label.clone(),
                        },
                    );
                    m
                },
                source: None,
                sort_order: row.sort_order,
                children: Some(Vec::new()),
            };
            values.insert(
                label_vid.to_string(),
                SkosValue {
                    id: label_vid.to_string(),
                    value: row.concept_label.clone(),
                },
            );
            all_concepts.insert(concept_id, concept.clone());
            concept_map.insert(&row.concept_label, concept);
        }

        // Second pass: build hierarchy by cloning children into parents
        let mut top_level_labels: Vec<&str> = Vec::new();
        for row in rows {
            if let Some(ref parent_label) = row.parent_label {
                // Clone child first to avoid simultaneous borrow
                let child = concept_map.get(row.concept_label.as_str()).cloned();
                if let (Some(parent), Some(child)) =
                    (concept_map.get_mut(parent_label.as_str()), child)
                {
                    if let Some(ref mut children) = parent.children {
                        children.push(child);
                    }
                }
            } else {
                top_level_labels.push(&row.concept_label);
            }
        }

        // Top-level concepts map (excludes children)
        let mut top_concepts: HashMap<String, SkosConcept> = HashMap::new();
        for label in &top_level_labels {
            if let Some(concept) = concept_map.get(label) {
                top_concepts.insert(concept.id.clone(), concept.clone());
            }
        }

        collections.push(SkosCollection {
            id: collection_id,
            uri: None,
            pref_labels,
            alt_labels: HashMap::new(),
            scope_notes: HashMap::new(),
            node_type: SkosNodeType::ConceptScheme,
            concepts: top_concepts,
            all_concepts,
            values,
        });
    }

    Ok(collections)
}

/// Build a graph and collections from the 3-CSV format.
///
/// This is the main entry point. It parses, validates, converts to
/// instructions, and builds the graph via the standard mutation pipeline.
///
/// # Arguments
/// * `graph_csv` - Contents of graph.csv
/// * `nodes_csv` - Contents of nodes.csv
/// * `collections_csv` - Contents of collections.csv (optional)
/// * `rdm_namespace` - RDM namespace string (UUID or URL) for deterministic ID generation
/// * `options` - Mutator options (autocreate cards/widgets, ontology validation)
///
/// Returns the built graph and any SKOS collections, or an error with diagnostics.
pub fn build_graph_from_model_csvs(
    graph_csv: &str,
    nodes_csv: &str,
    collections_csv: Option<&str>,
    rdm_namespace: &str,
    options: MutatorOptions,
) -> Result<(StaticGraph, Vec<SkosCollection>), CsvModelError> {
    let (bundle, mut diagnostics) = parse_model_csvs(graph_csv, nodes_csv, collections_csv)?;

    // Validate
    let validation = validate_model_csvs(&bundle);
    let has_errors = validation.iter().any(|d| d.level == DiagnosticLevel::Error);
    diagnostics.extend(validation);

    if has_errors {
        return Err(CsvModelError { diagnostics });
    }

    // Convert to instructions and build
    let instructions = model_csvs_to_instructions(&bundle, rdm_namespace)?;
    let graph = crate::graph_mutator::build_graph_from_instructions(instructions, options)
        .map_err(|e| CsvModelError {
            diagnostics: {
                diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Error,
                    file: "(build)".to_string(),
                    line: None,
                    message: e,
                });
                diagnostics
            },
        })?;

    let collections = model_csvs_to_collections(&bundle, rdm_namespace)?;

    Ok((graph, collections))
}

/// Validate CSVs and return diagnostics without building.
///
/// Convenience function that parses and validates the 3-CSV format,
/// returning all diagnostics. No graph is built, so no namespace is needed.
pub fn validate_model_csvs_from_strings(
    graph_csv: &str,
    nodes_csv: &str,
    collections_csv: Option<&str>,
) -> Vec<CsvModelDiagnostic> {
    match parse_model_csvs(graph_csv, nodes_csv, collections_csv) {
        Ok((bundle, mut parse_diags)) => {
            let validation = validate_model_csvs(&bundle);
            parse_diags.extend(validation);
            parse_diags
        }
        Err(e) => e.diagnostics,
    }
}

// --- Helpers ---

fn build_collection_id_map<'a>(
    bundle: &'a ModelCsvBundle,
    namespace: &Uuid,
) -> HashMap<&'a str, String> {
    let mut map = HashMap::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for row in &bundle.collections {
        if seen.insert(&row.collection_name) {
            map.insert(
                row.collection_name.as_str(),
                generate_collection_uuid(namespace, &row.collection_name).to_string(),
            );
        }
    }
    map
}

fn topological_sort(nodes: &[NodeRow]) -> Vec<&NodeRow> {
    let by_alias: HashMap<&str, &NodeRow> = nodes.iter().map(|n| (n.alias.as_str(), n)).collect();
    let mut visited: HashSet<&str> = HashSet::new();
    let mut sorted: Vec<&NodeRow> = Vec::new();

    fn visit<'a>(
        alias: &'a str,
        by_alias: &HashMap<&str, &'a NodeRow>,
        visited: &mut HashSet<&'a str>,
        sorted: &mut Vec<&'a NodeRow>,
    ) {
        if visited.contains(alias) {
            return;
        }
        visited.insert(alias);
        if let Some(node) = by_alias.get(alias) {
            if let Some(ref pa) = node.parent_alias {
                if by_alias.contains_key(pa.as_str()) {
                    visit(pa, by_alias, visited, sorted);
                }
            }
            sorted.push(node);
        }
    }

    for node in nodes {
        visit(&node.alias, &by_alias, &mut visited, &mut sorted);
    }
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRAPH_CSV: &str = r#"name,ontology_class,author,description,is_resource
Heritage Monument,http://www.cidoc-crm.org/cidoc-crm/E24_Physical_Human-Made_Thing,,A heritage monument,true"#;

    const NODES_CSV: &str = r#"parent_alias,alias,name,datatype,cardinality,ontology_class,parent_property,description,collection_name,required,searchable,exportable,sortorder
,name,Name,string,1,http://www.cidoc-crm.org/cidoc-crm/E41_Appellation,http://www.cidoc-crm.org/cidoc-crm/P1_is_identified_by,Primary name,,true,true,true,1
,monument_type,Monument Type,concept,1,http://www.cidoc-crm.org/cidoc-crm/E55_Type,http://www.cidoc-crm.org/cidoc-crm/P2_has_type,Type classification,Monument Types,true,true,true,2
,location,Location,semantic,n,http://www.cidoc-crm.org/cidoc-crm/E53_Place,http://www.cidoc-crm.org/cidoc-crm/P53_has_former_or_current_location,,,false,true,true,3
location,place_name,Place Name,string,1,http://www.cidoc-crm.org/cidoc-crm/E44_Place_Appellation,http://www.cidoc-crm.org/cidoc-crm/P87_is_identified_by,,,true,true,true,1
location,geometry,Geometry,geojson-feature-collection,1,http://www.cidoc-crm.org/cidoc-crm/E94_Space_Primitive,http://www.cidoc-crm.org/cidoc-crm/P168_place_is_defined_by,,,false,false,true,2"#;

    const COLLECTIONS_CSV: &str = r#"collection_name,concept_label,parent_label,sort_order
Monument Types,Castle,,1
Monument Types,Church,,2
Monument Types,Bridge,,3
Monument Types,Fortification,,4
Monument Types,Motte,Castle,5"#;

    #[test]
    fn test_parse_and_validate() {
        let (bundle, parse_diags) =
            parse_model_csvs(GRAPH_CSV, NODES_CSV, Some(COLLECTIONS_CSV)).unwrap();
        assert!(parse_diags
            .iter()
            .all(|d| d.level != DiagnosticLevel::Error));
        assert_eq!(bundle.graph.name, "Heritage Monument");
        assert_eq!(bundle.nodes.len(), 5);
        assert_eq!(bundle.collections.len(), 5);

        let validation = validate_model_csvs(&bundle);
        let errors: Vec<_> = validation
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Error)
            .collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
    }

    const TEST_NAMESPACE: &str = "http://test.example.org/rdm/";

    #[test]
    fn test_to_instructions() {
        let (bundle, _) = parse_model_csvs(GRAPH_CSV, NODES_CSV, Some(COLLECTIONS_CSV)).unwrap();
        let instructions = model_csvs_to_instructions(&bundle, TEST_NAMESPACE).unwrap();

        assert_eq!(instructions[0].action, "create_model");
        assert_eq!(instructions.len(), 6); // 1 create + 5 nodes
                                           // Verify topological order: location before place_name and geometry
        let aliases: Vec<&str> = instructions
            .iter()
            .filter(|i| i.action == "add_node")
            .map(|i| i.object.as_str())
            .collect();
        let loc_idx = aliases.iter().position(|a| *a == "location").unwrap();
        let pn_idx = aliases.iter().position(|a| *a == "place_name").unwrap();
        let geo_idx = aliases.iter().position(|a| *a == "geometry").unwrap();
        assert!(loc_idx < pn_idx);
        assert!(loc_idx < geo_idx);
    }

    #[test]
    fn test_to_collections() {
        let (bundle, _) = parse_model_csvs(GRAPH_CSV, NODES_CSV, Some(COLLECTIONS_CSV)).unwrap();
        let collections = model_csvs_to_collections(&bundle, TEST_NAMESPACE).unwrap();

        assert_eq!(collections.len(), 1);
        let coll = &collections[0];
        assert_eq!(coll.all_concepts.len(), 5);
        // Motte should be a child of Castle, not at top level
        assert_eq!(coll.concepts.len(), 4); // Castle, Church, Bridge, Fortification at top
    }

    #[test]
    fn test_build_graph() {
        let result = build_graph_from_model_csvs(
            GRAPH_CSV,
            NODES_CSV,
            Some(COLLECTIONS_CSV),
            TEST_NAMESPACE,
            MutatorOptions::default(),
        );
        let (graph, collections) = result.unwrap();
        // root + 5 nodes
        assert_eq!(graph.nodes.len(), 6);
        assert_eq!(graph.edges.len(), 5);
        assert_eq!(collections.len(), 1);
    }

    #[test]
    fn test_dangling_parent_alias() {
        let bad_nodes = r#"parent_alias,alias,name,datatype,cardinality,ontology_class,parent_property
nonexistent,child,Child,string,1,http://www.cidoc-crm.org/cidoc-crm/E62_String,http://www.cidoc-crm.org/cidoc-crm/P3_has_note"#;
        let diags = validate_model_csvs_from_strings(GRAPH_CSV, bad_nodes, None);
        assert!(diags
            .iter()
            .any(|d| d.message.contains("nonexistent") && d.level == DiagnosticLevel::Error));
    }

    #[test]
    fn test_missing_collection() {
        // Missing collection refs are warnings (not errors) — see validate_model_csvs
        // for the rationale (external SKOS imports are a valid workflow).
        let nodes_with_concept = r#"parent_alias,alias,name,datatype,cardinality,ontology_class,parent_property,collection_name
,my_type,Type,concept,1,http://www.cidoc-crm.org/cidoc-crm/E55_Type,http://www.cidoc-crm.org/cidoc-crm/P2_has_type,Missing Collection"#;
        let diags = validate_model_csvs_from_strings(GRAPH_CSV, nodes_with_concept, None);
        assert!(diags.iter().any(
            |d| d.message.contains("Missing Collection") && d.level == DiagnosticLevel::Warning
        ));
    }
}
