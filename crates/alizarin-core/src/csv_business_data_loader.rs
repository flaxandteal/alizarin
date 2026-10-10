//! CSV Business Data Loader
//!
//! Parses a CSV file of resource instances into [`StaticResource`]s,
//! resolving node aliases to UUIDs against a built [`StaticGraph`] and
//! concept labels to UUIDs against [`SkosCollection`]s.
//!
//! ## CSV Format
//!
//! ```csv
//! ResourceID,name_value,name_type,birth_date
//! john-1,John Smith,Preferred Name,1850-03-15
//! john-1,Seán Mac Gabhann,Alternative Name,
//! jane-1,Jane Doe,Birth Name,1820-06-01
//! ```
//!
//! - First column: `ResourceID` — human-readable identifier (stored as legacyid)
//! - Remaining columns: node aliases from the graph
//! - Multiple rows per ResourceID for cardinality-n nodegroups
//! - Concept values as labels (resolved against collections)
//! - Multilingual columns via `alias (lang)` headers
//! - Empty cells are skipped
//!
//! ## Example
//! ```rust,ignore
//! use alizarin_core::csv_business_data_loader::build_resources_from_business_csv;
//!
//! let resources = build_resources_from_business_csv(
//!     csv_data, &graph, &collections, Default::default(),
//! ).unwrap();
//! ```

use std::collections::HashMap;

use crate::csv_model_loader::{CsvModelDiagnostic, CsvModelError, DiagnosticLevel};
use crate::extension_type_registry::ExtensionTypeRegistry;
use crate::graph::{
    StaticGraph, StaticNode, StaticResource, StaticResourceDescriptors, StaticResourceMetadata,
    StaticTile,
};
use crate::graph_mutator::{generate_uuid_v5, generate_uuid_v5_with_ns};
use crate::label_resolution::ConceptLookup;
use crate::skos::SkosCollection;
use crate::type_coercion::{coerce_geojson, normalize_date_string};
use crate::type_serialization::SerializationContext;
use serde_json::Value;

/// Options for business data CSV loading
#[derive(Debug, Clone)]
pub struct BusinessDataCsvOptions {
    /// Default language code for string values (default: "en")
    pub default_language: String,
    /// Whether to error on unresolved concept labels (default: true)
    pub strict_concepts: bool,
    /// Override the base UUID v5 namespace for tile ID generation.
    /// When building separate layers that share a graph model, each layer
    /// should use a distinct namespace so tile IDs don't collide for the
    /// same resource + nodegroup + sortorder. Resource IDs are unaffected
    /// (they always use the default namespace so cross-layer lookup works).
    /// If None, uses the default Alizarin namespace.
    pub uuid_namespace: Option<String>,
}

impl Default for BusinessDataCsvOptions {
    fn default() -> Self {
        Self {
            default_language: "en".to_string(),
            strict_concepts: true,
            uuid_namespace: None,
        }
    }
}

/// A parsed column header
#[derive(Debug, Clone)]
struct ColumnMapping {
    alias: String,
    language: Option<String>,
    node: ColumnNode,
}

/// Resolved node info for a column
#[derive(Debug, Clone)]
struct ColumnNode {
    nodeid: String,
    nodegroup_id: String,
    datatype: String,
}

/// Build concept label→value-id lookup from collections.
///
/// Maps each prefLabel to that label's **value id** (`SkosValue.id`), NOT the
/// concept id. Arches stores the matched value's id in a concept tile — and the
/// ORM read side (`ConceptValueViewModel.getConceptValue`) indexes collection
/// values by value id — so storing the value id is what lets a CSV-loaded concept
/// resolve back on read (C1).
fn build_concept_lookup(
    collections: &[SkosCollection],
) -> HashMap<String, HashMap<String, String>> {
    // collection_id -> (lowercase_label -> value_id)
    let mut lookup: HashMap<String, HashMap<String, String>> = HashMap::new();

    for coll in collections {
        let mut labels: HashMap<String, String> = HashMap::new();
        for concept in coll.all_concepts.values() {
            for pref_label in concept.pref_labels.values() {
                labels.insert(pref_label.value.to_lowercase(), pref_label.id.clone());
            }
        }
        lookup.insert(coll.id.clone(), labels);
    }

    lookup
}

/// Find the prefLabel **value id** for a concept id, searching the collections.
/// Prefers the English prefLabel (the canonical one Arches/alizarin mint value ids
/// under), falling back to any available prefLabel. Used to convert a concept id
/// (as the shared RdmCache lookup returns) into the value id a tile should store.
fn value_id_for_concept(concept_id: &str, collections: &[SkosCollection]) -> Option<String> {
    for coll in collections {
        if let Some(concept) = coll.all_concepts.values().find(|c| c.id == concept_id) {
            return concept
                .pref_labels
                .get("en")
                .or_else(|| concept.pref_labels.values().next())
                .map(|v| v.id.clone());
        }
    }
    None
}

/// Find which collection a concept node references
fn find_node_collection_id(node: &StaticNode, collections: &[SkosCollection]) -> Option<String> {
    // Check node config for rdmCollection
    if let Some(rdm_coll) = node.config.get("rdmCollection") {
        if let Some(coll_id) = rdm_coll.as_str() {
            if !coll_id.is_empty() {
                return Some(coll_id.to_string());
            }
        }
    }
    // Fallback: if there's only one collection, use it (common in simple models)
    // Otherwise, try to match by name convention
    if collections.len() == 1 {
        return Some(collections[0].id.clone());
    }
    None
}

/// Parse a header like "alias" or "alias (lang)" into (alias, Option<lang>)
fn parse_header(header: &str) -> (String, Option<String>) {
    let trimmed = header.trim();
    if let Some(paren_start) = trimmed.rfind('(') {
        if trimmed.ends_with(')') {
            let alias = trimmed[..paren_start].trim().to_string();
            let lang = trimmed[paren_start + 1..trimmed.len() - 1]
                .trim()
                .to_string();
            if !alias.is_empty() && !lang.is_empty() {
                return (alias, Some(lang));
            }
        }
    }
    (trimmed.to_string(), None)
}

/// Format a string value for tile data
fn format_string_value(value: &str, language: &str) -> serde_json::Value {
    serde_json::json!({
        language: {
            "value": value,
            "direction": "ltr"
        }
    })
}

/// Merge a language variant into an existing string value
fn merge_string_language(
    existing: &serde_json::Value,
    value: &str,
    language: &str,
) -> serde_json::Value {
    let mut obj = match existing.as_object() {
        Some(o) => o.clone(),
        None => serde_json::Map::new(),
    };
    obj.insert(
        language.to_string(),
        serde_json::json!({
            "value": value,
            "direction": "ltr"
        }),
    );
    serde_json::Value::Object(obj)
}

/// Context for coercing CSV cell values
struct CoerceContext<'a> {
    collections: &'a [SkosCollection],
    concept_lookup: &'a HashMap<String, HashMap<String, String>>,
    /// Shared SKOS RdmCache (label -> id). When present it is authoritative,
    /// the same concept identity the read side resolves back, so it wins over
    /// the collections-derived fallback.
    external_lookup: Option<&'a dyn ConceptLookup>,
    diagnostics: &'a mut Vec<CsvModelDiagnostic>,
    line: usize,
    strict_concepts: bool,
    /// Extension-datatype handlers; `None` = core-only coercion.
    registry: Option<&'a ExtensionTypeRegistry>,
}

/// The allowed target model graphids for a link node, read from each
/// `config.graphs[*].graphid` (populated from the `graphs` column in nodes.csv,
/// see C4). Empty if the node declares no target models.
fn link_target_graphids(node: &StaticNode) -> Vec<&str> {
    node.config
        .get("graphs")
        .and_then(|g| g.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| entry.get("graphid").and_then(|v| v.as_str()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Build one Arches resource-instance link object from a single CSV cell token.
///
/// A token that is already a UUID is used verbatim. A non-UUID token (a business
/// `ResourceID`, e.g. `ferrymen`) is resolved to the deterministic resource
/// instance id of the target model — `uuid5(("resource", target_graphid), token)`,
/// the same derivation the loader uses when it builds that resource — provided the
/// link node declares exactly one target model in `config.graphs` (C4). With no
/// target model, or with several (so the key is ambiguous), a non-UUID token cannot
/// be resolved and is rejected with an error diagnostic rather than stored as an
/// unresolvable literal (C3).
fn build_resource_link(
    token: &str,
    node: &StaticNode,
    node_label: &str,
    ctx: &mut CoerceContext<'_>,
) -> Option<serde_json::Value> {
    let resource_id = if uuid::Uuid::parse_str(token).is_ok() {
        token.to_string()
    } else {
        let targets = link_target_graphids(node);
        match targets.as_slice() {
            [single] => generate_uuid_v5(("resource", Some(single)), token),
            other => {
                let reason = if other.is_empty() {
                    "the node declares no target model (graphs) to derive a resource id from"
                } else {
                    "the node allows multiple target models, so a bare ResourceID is ambiguous — \
                     use a resource instance UUID"
                };
                ctx.diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Error,
                    file: "business_data.csv".to_string(),
                    line: Some(ctx.line),
                    message: format!(
                        "Cannot resolve ResourceID '{}' for link node '{}': value is not a UUID and {}",
                        token, node_label, reason
                    ),
                });
                return None;
            }
        }
    };

    let rxr_id = generate_uuid_v5(("resource-x-resource", None), &resource_id);
    Some(serde_json::json!({
        "resourceId": resource_id,
        "resourceXresourceId": rxr_id,
        "ontologyProperty": "",
        "inverseOntologyProperty": ""
    }))
}

/// Convert a CSV cell value to the appropriate tile data value
fn coerce_value(
    raw: &str,
    datatype: &str,
    language: &str,
    node: &StaticNode,
    ctx: &mut CoerceContext<'_>,
) -> Option<serde_json::Value> {
    if raw.is_empty() {
        return None;
    }

    let node_label = node.alias.as_deref().unwrap_or(&node.nodeid);

    match datatype {
        "string" => Some(format_string_value(raw, language)),
        "number" => match raw.parse::<f64>() {
            Ok(n) => Some(serde_json::json!(n)),
            Err(_) => {
                ctx.diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Error,
                    file: "business_data.csv".to_string(),
                    line: Some(ctx.line),
                    message: format!("Cannot parse '{}' as number for node '{}'", raw, node_label),
                });
                None
            }
        },
        "date" => {
            let normalized = normalize_date_string(raw).unwrap_or_else(|_| raw.to_string());
            Some(serde_json::Value::String(normalized))
        }
        "boolean" => match raw.to_lowercase().as_str() {
            "true" | "yes" | "1" => Some(serde_json::Value::Bool(true)),
            "false" | "no" | "0" => Some(serde_json::Value::Bool(false)),
            _ => {
                ctx.diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Error,
                    file: "business_data.csv".to_string(),
                    line: Some(ctx.line),
                    message: format!(
                        "Cannot parse '{}' as boolean for node '{}'",
                        raw, node_label
                    ),
                });
                None
            }
        },
        "concept" | "domain-value" => {
            resolve_concept_label(raw, node, ctx).map(serde_json::Value::String)
        }
        "concept-list" | "domain-value-list" => {
            let ids: Vec<serde_json::Value> = raw
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .filter_map(|label| resolve_concept_label(label, node, ctx))
                .map(serde_json::Value::String)
                .collect();
            if ids.is_empty() {
                None
            } else {
                Some(serde_json::Value::Array(ids))
            }
        }
        // A single resource-instance is stored in the SAME shape as a list — an
        // array of link objects — just constrained to one entry, matching how
        // Arches and the ORM read side (ResourceInstanceViewModel) treat it (C2).
        "resource-instance" => {
            let link = build_resource_link(raw.trim(), node, node_label, ctx)?;
            Some(serde_json::Value::Array(vec![link]))
        }
        "resource-instance-list" => {
            let arr: Vec<serde_json::Value> = raw
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .filter_map(|r| build_resource_link(r, node, node_label, ctx))
                .collect();
            if arr.is_empty() {
                None
            } else {
                Some(serde_json::Value::Array(arr))
            }
        }
        "geojson-feature-collection" => match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(v) => {
                let result = coerce_geojson(&v);
                if result.is_error() {
                    ctx.diagnostics.push(CsvModelDiagnostic {
                        level: DiagnosticLevel::Error,
                        file: "business_data.csv".to_string(),
                        line: Some(ctx.line),
                        message: format!(
                            "Invalid GeoJSON for node '{}': {}",
                            node_label,
                            result.error.unwrap_or_default()
                        ),
                    });
                    None
                } else {
                    for warning in &result.warnings {
                        ctx.diagnostics.push(CsvModelDiagnostic {
                            level: DiagnosticLevel::Warning,
                            file: "business_data.csv".to_string(),
                            line: Some(ctx.line),
                            message: format!("Node '{}': {}", node_label, warning),
                        });
                    }
                    Some(result.tile_data)
                }
            }
            Err(_) => {
                ctx.diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Error,
                    file: "business_data.csv".to_string(),
                    line: Some(ctx.line),
                    message: format!("Cannot parse GeoJSON for node '{}'", node_label),
                });
                None
            }
        },
        "file-list" => {
            ctx.diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Warning,
                file: "business_data.csv".to_string(),
                line: Some(ctx.line),
                message: format!(
                    "file-list datatype not supported in CSV import for node '{}'",
                    node_label
                ),
            });
            None
        }
        "semantic" => None,
        _ => {
            // Extension datatypes core doesn't own (e.g. CLM `reference`):
            // delegate to the registered handler's coerce, then resolve any
            // RDM lookup markers it emits so we get bare concept UUIDs.
            if let Some(reg) = ctx.registry {
                let cfg = serde_json::Value::Object(node.config.clone().into_iter().collect());
                if let Ok(Some(result)) = reg.coerce(
                    datatype,
                    &serde_json::Value::String(raw.to_string()),
                    Some(&cfg),
                ) {
                    return Some(resolve_rdm_markers(result.tile_data, node, ctx));
                }
            }
            ctx.diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Warning,
                file: "business_data.csv".to_string(),
                line: Some(ctx.line),
                message: format!(
                    "Unknown datatype '{}' for node '{}', storing as raw string",
                    datatype, node_label
                ),
            });
            Some(serde_json::Value::String(raw.to_string()))
        }
    }
}

/// Resolve the RDM lookup markers an extension handler's `coerce` emits for
/// unresolved labels (`{"__needs_rdm_label_lookup": true, "label": ...}`) into
/// the bare concept UUID that indexing expects. A `{"__needs_rdm_lookup": true,
/// "uuid": ...}` marker is already an id. Anything else passes through,
/// recursing into arrays.
fn resolve_rdm_markers(value: Value, node: &StaticNode, ctx: &mut CoerceContext<'_>) -> Value {
    match value {
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|v| resolve_rdm_markers(v, node, ctx))
                .collect(),
        ),
        Value::Object(ref obj)
            if obj
                .get("__needs_rdm_label_lookup")
                .and_then(|v| v.as_bool())
                .unwrap_or(false) =>
        {
            match obj.get("label").and_then(|v| v.as_str()) {
                Some(label) => resolve_concept_label(label, node, ctx)
                    .map(Value::String)
                    .unwrap_or(value),
                None => value,
            }
        }
        Value::Object(ref obj)
            if obj
                .get("__needs_rdm_lookup")
                .and_then(|v| v.as_bool())
                .unwrap_or(false) =>
        {
            obj.get("uuid")
                .and_then(|v| v.as_str())
                .map(|s| Value::String(s.to_string()))
                .unwrap_or(value)
        }
        other => other,
    }
}

/// Resolve a concept label to the **value id** a concept tile should store.
///
/// Arches stores the matched value's id (not the concept id) in a concept tile,
/// and the ORM read side looks concepts up by value id, so this returns a value id
/// (C1). A value already supplied as a UUID is assumed to be a value id and passes
/// through unchanged, mirroring Arches' `transform_value_for_tile`.
fn resolve_concept_label(
    label: &str,
    node: &StaticNode,
    ctx: &mut CoerceContext<'_>,
) -> Option<String> {
    let lower = label.to_lowercase();

    // If it's already a UUID, return as-is (assumed to be a value id).
    if uuid::Uuid::parse_str(label).is_ok() {
        return Some(label.to_string());
    }

    let coll_id = find_node_collection_id(node, ctx.collections);

    if let Some(ext) = ctx.external_lookup {
        // Authoritative path: resolve through the shared SKOS RdmCache, scoped to
        // the node's own collection. The cache resolves a label to a concept id, so
        // convert it to the concept's prefLabel value id for tile storage; if the
        // concept isn't in our local collections, fall back to the id as returned.
        if let Some(cid) = &coll_id {
            if let Some(id) = ext.lookup_by_label(cid, label) {
                return Some(value_id_for_concept(&id, ctx.collections).unwrap_or(id));
            }
        }
    } else {
        // Fallback (no shared cache): collections-derived lookup, node's
        // collection first, then any collection.
        if let Some(cid) = &coll_id {
            if let Some(labels) = ctx.concept_lookup.get(cid) {
                if let Some(concept_id) = labels.get(&lower) {
                    return Some(concept_id.clone());
                }
            }
        }
        for labels in ctx.concept_lookup.values() {
            if let Some(concept_id) = labels.get(&lower) {
                return Some(concept_id.clone());
            }
        }
    }

    let level = if ctx.strict_concepts {
        DiagnosticLevel::Error
    } else {
        DiagnosticLevel::Warning
    };
    ctx.diagnostics.push(CsvModelDiagnostic {
        level,
        file: "business_data.csv".to_string(),
        line: Some(ctx.line),
        message: format!(
            "Cannot resolve concept label '{}' for node '{}'",
            label,
            node.alias.as_deref().unwrap_or(&node.nodeid)
        ),
    });
    None
}

/// Build resources from a business data CSV.
///
/// Resolves node aliases to UUIDs from the graph, concept labels to UUIDs
/// from the collections. Generates deterministic UUIDs for resources and tiles.
///
/// # Arguments
/// * `csv_data` - The CSV string (headers + data rows)
/// * `graph` - A built StaticGraph with node definitions
/// * `collections` - SKOS collections for concept resolution
/// * `options` - Loading options
///
/// # Returns
/// A list of StaticResources, or error with diagnostics
pub fn build_resources_from_business_csv(
    csv_data: &str,
    graph: &StaticGraph,
    collections: &[SkosCollection],
    options: BusinessDataCsvOptions,
) -> Result<Vec<StaticResource>, CsvModelError> {
    build_resources_from_business_csv_with_context(
        csv_data,
        graph,
        collections,
        None,
        options,
        None,
    )
}

/// Build resources, resolving concept/reference labels to concept ids.
///
/// When `context` carries a `concept_lookup` (the shared SKOS-backed RdmCache),
/// labels resolve through it, the same concept identity the read side later
/// resolves back. When absent, resolution falls back to a lookup derived from
/// `collections`.
///
/// When `registry` is provided, unknown datatypes are coerced via their
/// extension handler instead of stored as bare strings.
pub fn build_resources_from_business_csv_with_context(
    csv_data: &str,
    graph: &StaticGraph,
    collections: &[SkosCollection],
    registry: Option<&ExtensionTypeRegistry>,
    options: BusinessDataCsvOptions,
    context: Option<&SerializationContext>,
) -> Result<Vec<StaticResource>, CsvModelError> {
    let mut diagnostics: Vec<CsvModelDiagnostic> = Vec::new();

    // Build lookup indices
    let alias_to_node: HashMap<String, &StaticNode> = graph
        .nodes
        .iter()
        .filter_map(|n| n.alias.as_ref().map(|a| (a.clone(), n)))
        .collect();

    let external_lookup = context.and_then(|c| c.concept_lookup);
    let concept_lookup = build_concept_lookup(collections);

    // Parse CSV
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_bytes());

    // Parse headers
    let headers = reader.headers().map_err(|e| CsvModelError {
        diagnostics: vec![CsvModelDiagnostic {
            level: DiagnosticLevel::Error,
            file: "business_data.csv".to_string(),
            line: Some(1),
            message: format!("Failed to parse CSV headers: {}", e),
        }],
    })?;

    let header_vec: Vec<String> = headers.iter().map(|h| h.to_string()).collect();

    // First column must be ResourceID
    if header_vec.is_empty() || header_vec[0].to_lowercase() != "resourceid" {
        return Err(CsvModelError {
            diagnostics: vec![CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "business_data.csv".to_string(),
                line: Some(1),
                message: "First column must be 'ResourceID'".to_string(),
            }],
        });
    }

    // Map columns to nodes
    let mut column_mappings: Vec<Option<ColumnMapping>> = Vec::new();
    column_mappings.push(None); // ResourceID column

    for header in &header_vec[1..] {
        let (alias, lang) = parse_header(header);
        if let Some(node) = alias_to_node.get(&alias) {
            if node.datatype == "semantic" {
                diagnostics.push(CsvModelDiagnostic {
                    level: DiagnosticLevel::Warning,
                    file: "business_data.csv".to_string(),
                    line: Some(1),
                    message: format!(
                        "Column '{}' maps to semantic node '{}' — semantic nodes don't carry data, column will be ignored",
                        header, alias
                    ),
                });
                column_mappings.push(None);
            } else {
                column_mappings.push(Some(ColumnMapping {
                    alias: alias.clone(),
                    language: lang,
                    node: ColumnNode {
                        nodeid: node.nodeid.clone(),
                        nodegroup_id: node
                            .nodegroup_id
                            .clone()
                            .unwrap_or_else(|| node.nodeid.clone()),
                        datatype: node.datatype.clone(),
                    },
                }));
            }
        } else {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Warning,
                file: "business_data.csv".to_string(),
                line: Some(1),
                message: format!(
                    "Column '{}' does not match any node alias in graph '{}'",
                    header,
                    graph.name.to_string_default()
                ),
            });
            column_mappings.push(None);
        }
    }

    // Read all rows and group by ResourceID
    let mut resource_rows: Vec<(String, Vec<(usize, csv::StringRecord)>)> = Vec::new();
    let mut current_id: Option<String> = None;

    for (row_idx, result) in reader.records().enumerate() {
        let line = row_idx + 2; // 1-indexed, after header
        let record = result.map_err(|e| CsvModelError {
            diagnostics: vec![CsvModelDiagnostic {
                level: DiagnosticLevel::Error,
                file: "business_data.csv".to_string(),
                line: Some(line),
                message: format!("Failed to parse CSV row: {}", e),
            }],
        })?;

        let resource_id = record.get(0).unwrap_or("").trim().to_string();
        if resource_id.is_empty() {
            diagnostics.push(CsvModelDiagnostic {
                level: DiagnosticLevel::Warning,
                file: "business_data.csv".to_string(),
                line: Some(line),
                message: "Empty ResourceID, skipping row".to_string(),
            });
            continue;
        }

        match &current_id {
            Some(id) if id == &resource_id => {
                resource_rows.last_mut().unwrap().1.push((line, record));
            }
            _ => {
                // Check for non-contiguous duplicates
                if resource_rows.iter().any(|(id, _)| id == &resource_id) {
                    diagnostics.push(CsvModelDiagnostic {
                        level: DiagnosticLevel::Error,
                        file: "business_data.csv".to_string(),
                        line: Some(line),
                        message: format!(
                            "Non-contiguous ResourceID '{}' — rows for the same resource must be grouped together",
                            resource_id
                        ),
                    });
                    continue;
                }
                current_id = Some(resource_id.clone());
                resource_rows.push((resource_id, vec![(line, record)]));
            }
        }
    }

    // Check for errors so far
    if diagnostics
        .iter()
        .any(|d| d.level == DiagnosticLevel::Error)
    {
        return Err(CsvModelError { diagnostics });
    }

    // Build nodegroup cardinality lookup
    let ng_cardinality: HashMap<String, String> = graph
        .nodegroups
        .iter()
        .map(|ng| {
            (
                ng.nodegroupid.clone(),
                ng.cardinality.clone().unwrap_or_else(|| "1".to_string()),
            )
        })
        .collect();

    // Build parent nodegroup lookup
    let ng_parent: HashMap<String, String> = graph
        .nodegroups
        .iter()
        .filter_map(|ng| {
            ng.parentnodegroup_id
                .as_ref()
                .map(|p| (ng.nodegroupid.clone(), p.clone()))
        })
        .collect();

    // Build resources
    let mut resources: Vec<StaticResource> = Vec::new();

    // Tile ID generator: uses custom namespace if provided, default otherwise.
    // Resource IDs always use the default namespace so cross-layer lookup works.
    let gen_tile_id = |group: (&str, Option<&str>), key: &str| -> String {
        match &options.uuid_namespace {
            Some(ns) => generate_uuid_v5_with_ns(ns, group, key),
            None => generate_uuid_v5(group, key),
        }
    };

    for (resource_id, rows) in &resource_rows {
        let resourceinstanceid = generate_uuid_v5(("resource", Some(&graph.graphid)), resource_id);

        // Determine display name: use first non-empty string value, or resource_id
        let display_name = find_display_name(rows, &column_mappings, resource_id);

        // Group data by nodegroup
        // For each nodegroup, collect: Vec<(row_index, HashMap<nodeid, value>)>
        type TileData = HashMap<String, serde_json::Value>;
        let mut ng_data: HashMap<String, Vec<(usize, TileData)>> = HashMap::new();

        for (line, record) in rows {
            // For this row, group cell values by nodegroup
            let mut row_ng_data: HashMap<String, HashMap<String, serde_json::Value>> =
                HashMap::new();

            for (col_idx, mapping) in column_mappings.iter().enumerate() {
                let Some(mapping) = mapping else { continue };
                let raw = record.get(col_idx).unwrap_or("").trim();
                if raw.is_empty() {
                    continue;
                }

                let language = mapping
                    .language
                    .as_deref()
                    .unwrap_or(&options.default_language);

                let node = alias_to_node.get(&mapping.alias).unwrap();

                let mut ctx = CoerceContext {
                    collections,
                    concept_lookup: &concept_lookup,
                    external_lookup,
                    diagnostics: &mut diagnostics,
                    line: *line,
                    strict_concepts: options.strict_concepts,
                    registry,
                };
                let value = coerce_value(raw, &mapping.node.datatype, language, node, &mut ctx);

                if let Some(val) = value {
                    let ng_entry = row_ng_data
                        .entry(mapping.node.nodegroup_id.clone())
                        .or_default();

                    // Handle multilingual merge for strings
                    if mapping.node.datatype == "string" && mapping.language.is_some() {
                        if let Some(existing) = ng_entry.get(&mapping.node.nodeid) {
                            let merged = merge_string_language(existing, raw, language);
                            ng_entry.insert(mapping.node.nodeid.clone(), merged);
                        } else {
                            ng_entry.insert(mapping.node.nodeid.clone(), val);
                        }
                    } else {
                        ng_entry.insert(mapping.node.nodeid.clone(), val);
                    }
                }
            }

            // Merge this row's data into the nodegroup tracker
            for (ng_id, data) in row_ng_data {
                let entries = ng_data.entry(ng_id).or_default();
                entries.push((*line, data));
            }
        }

        // Check for errors before building tiles
        if diagnostics
            .iter()
            .any(|d| d.level == DiagnosticLevel::Error)
        {
            return Err(CsvModelError { diagnostics });
        }

        // Build tiles
        let mut tiles: Vec<StaticTile> = Vec::new();

        // Track parent tiles for nested nodegroups
        let mut parent_tile_ids: HashMap<String, Vec<String>> = HashMap::new();

        // Sort nodegroups: parents first (those without parentnodegroup_id)
        let mut ng_ids: Vec<String> = ng_data.keys().cloned().collect();
        ng_ids.sort_by_key(|ng_id| if ng_parent.contains_key(ng_id) { 1 } else { 0 });

        for ng_id in &ng_ids {
            let entries = ng_data.get(ng_id).unwrap();
            let cardinality = ng_cardinality.get(ng_id).map(|s| s.as_str()).unwrap_or("1");

            let parent_ng_id = ng_parent.get(ng_id);

            if cardinality == "n" {
                // Each entry (row) gets its own tile
                for (sortorder, (_line, data)) in entries.iter().enumerate() {
                    let tileid = gen_tile_id(
                        ("tile", Some(&resourceinstanceid)),
                        &format!("{}/{}", ng_id, sortorder),
                    );

                    let parenttile_id = parent_ng_id
                        .and_then(|png_id| {
                            parent_tile_ids.get(png_id).and_then(|ids| {
                                // Match by sortorder for cardinality-n parents,
                                // or first tile for cardinality-1 parents
                                ids.get(sortorder).or_else(|| ids.first())
                            })
                        })
                        .cloned();

                    tiles.push(StaticTile {
                        tileid: Some(tileid.clone()),
                        nodegroup_id: ng_id.clone(),
                        parenttile_id,
                        resourceinstance_id: resourceinstanceid.clone(),
                        sortorder: Some(sortorder as i32),
                        provisionaledits: None,
                        data: data.clone(),
                    });

                    parent_tile_ids
                        .entry(ng_id.clone())
                        .or_default()
                        .push(tileid);
                }
            } else {
                // Cardinality 1: merge all entries into a single tile
                let mut merged_data: HashMap<String, serde_json::Value> = HashMap::new();
                for (_line, data) in entries {
                    for (node_id, value) in data {
                        if let Some(existing) = merged_data.get(node_id) {
                            // For strings, merge languages; otherwise, warn on conflict
                            if existing.is_object() && value.is_object() {
                                let merged = merge_objects(
                                    existing.as_object().unwrap(),
                                    value.as_object().unwrap(),
                                );
                                merged_data
                                    .insert(node_id.clone(), serde_json::Value::Object(merged));
                            } else if existing != value {
                                diagnostics.push(CsvModelDiagnostic {
                                    level: DiagnosticLevel::Warning,
                                    file: "business_data.csv".to_string(),
                                    line: Some(*_line),
                                    message: format!(
                                        "Conflicting values for node '{}' in cardinality-1 nodegroup '{}', using last value",
                                        node_id, ng_id
                                    ),
                                });
                                merged_data.insert(node_id.clone(), value.clone());
                            }
                        } else {
                            merged_data.insert(node_id.clone(), value.clone());
                        }
                    }
                }

                let tileid = gen_tile_id(("tile", Some(&resourceinstanceid)), ng_id);

                let parenttile_id = parent_ng_id
                    .and_then(|png_id| parent_tile_ids.get(png_id).and_then(|ids| ids.first()))
                    .cloned();

                tiles.push(StaticTile {
                    tileid: Some(tileid.clone()),
                    nodegroup_id: ng_id.clone(),
                    parenttile_id,
                    resourceinstance_id: resourceinstanceid.clone(),
                    sortorder: Some(0),
                    provisionaledits: None,
                    data: merged_data,
                });

                parent_tile_ids
                    .entry(ng_id.clone())
                    .or_default()
                    .push(tileid);
            }
        }

        resources.push(StaticResource {
            resourceinstance: StaticResourceMetadata {
                resourceinstanceid: resourceinstanceid.clone(),
                graph_id: graph.graphid.clone(),
                name: display_name,
                legacyid: Some(resource_id.clone()),
                descriptors: StaticResourceDescriptors::default(),
                publication_id: None,
                principaluser_id: None,
                graph_publication_id: None,
                createdtime: None,
                lastmodified: None,
            },
            tiles: Some(tiles),
            metadata: HashMap::new(),
            cache: None,
            scopes: None,
            tiles_loaded: Some(true),
        });
    }

    // Check for any remaining errors
    if diagnostics
        .iter()
        .any(|d| d.level == DiagnosticLevel::Error)
    {
        return Err(CsvModelError { diagnostics });
    }

    Ok(resources)
}

/// Find the display name from the first non-empty string column value
fn find_display_name(
    rows: &[(usize, csv::StringRecord)],
    column_mappings: &[Option<ColumnMapping>],
    fallback: &str,
) -> String {
    for (_line, record) in rows {
        for (col_idx, mapping) in column_mappings.iter().enumerate() {
            if let Some(mapping) = mapping {
                if mapping.node.datatype == "string" && mapping.language.is_none() {
                    let raw = record.get(col_idx).unwrap_or("").trim();
                    if !raw.is_empty() {
                        return raw.to_string();
                    }
                }
            }
        }
    }
    fallback.to_string()
}

/// Merge two JSON objects (for multilingual string merging)
fn merge_objects(
    a: &serde_json::Map<String, serde_json::Value>,
    b: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut result = a.clone();
    for (k, v) in b {
        result.insert(k.clone(), v.clone());
    }
    result
}

/// Wrap resources in the Arches business_data format
pub fn wrap_business_data(resources: &[StaticResource]) -> serde_json::Value {
    serde_json::json!({
        "business_data": {
            "resources": resources
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_model_loader::build_graph_from_model_csvs;
    use crate::graph_mutator::MutatorOptions;

    const GRAPH_CSV: &str = "name,ontology_class,author,description,is_resource
Historical Person,http://www.cidoc-crm.org/cidoc-crm/E21_Person,,A person,true";

    const NODES_CSV: &str = "\
parent_alias,alias,name,datatype,cardinality,ontology_class,parent_property,description,collection_name,required,searchable,exportable,sortorder
,names,Names,semantic,n,http://www.cidoc-crm.org/cidoc-crm/E41_Appellation,http://www.cidoc-crm.org/cidoc-crm/P1_is_identified_by,,,,,,1
names,name_value,Name Value,string,1,http://www.cidoc-crm.org/cidoc-crm/E33_Linguistic_Object,http://www.cidoc-crm.org/cidoc-crm/P3_has_note,,,true,true,,2
names,name_type,Name Type,concept,1,http://www.cidoc-crm.org/cidoc-crm/E55_Type,http://www.cidoc-crm.org/cidoc-crm/P2_has_type,,Name Types,,,,3
,birth_date,Birth Date,date,1,http://www.cidoc-crm.org/cidoc-crm/E52_Time-Span,http://www.cidoc-crm.org/cidoc-crm/P4_has_time-span,,,,,,4
,person_type,Person Type,concept,1,http://www.cidoc-crm.org/cidoc-crm/E55_Type,http://www.cidoc-crm.org/cidoc-crm/P2_has_type,,Person Types,,,,5";

    const COLLECTIONS_CSV: &str = "\
collection_name,concept_label,parent_label,sort_order
Name Types,Preferred Name,,1
Name Types,Alternative Name,,2
Name Types,Birth Name,,3
Person Types,Historical Figure,,1
Person Types,Fictional Character,,2";

    fn build_test_graph() -> (StaticGraph, Vec<SkosCollection>) {
        build_graph_from_model_csvs(
            GRAPH_CSV,
            NODES_CSV,
            Some(COLLECTIONS_CSV),
            "https://example.org/test",
            MutatorOptions::default(),
        )
        .expect("Failed to build test graph")
    }

    #[test]
    fn test_single_resource() {
        let (graph, collections) = build_test_graph();

        let csv = "\
ResourceID,name_value,birth_date
john-1,John Smith,1850-03-15";

        let resources = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        )
        .expect("Should build successfully");

        assert_eq!(resources.len(), 1);
        let r = &resources[0];
        assert_eq!(r.resourceinstance.name, "John Smith");
        assert_eq!(r.resourceinstance.legacyid.as_deref(), Some("john-1"));
        assert_eq!(r.resourceinstance.graph_id, graph.graphid);
        assert!(r.tiles.as_ref().unwrap().len() >= 1);
    }

    #[test]
    fn test_cardinality_n() {
        let (graph, collections) = build_test_graph();

        let csv = "\
ResourceID,name_value,name_type
john-1,John Smith,Preferred Name
john-1,Seán Mac Gabhann,Alternative Name";

        let resources =
            build_resources_from_business_csv(csv, &graph, &collections, Default::default())
                .expect("Should build successfully");

        assert_eq!(resources.len(), 1);
        let tiles = resources[0].tiles.as_ref().unwrap();
        // names is cardinality-n, so 2 rows = 2 tiles for that nodegroup
        let names_ng = graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("names"))
            .unwrap()
            .nodeid
            .clone();
        let name_tiles: Vec<_> = tiles
            .iter()
            .filter(|t| t.nodegroup_id == names_ng)
            .collect();
        assert_eq!(name_tiles.len(), 2);
    }

    #[test]
    fn test_concept_resolution() {
        let (graph, collections) = build_test_graph();

        let csv = "\
ResourceID,name_value,name_type
john-1,John Smith,Preferred Name";

        let resources =
            build_resources_from_business_csv(csv, &graph, &collections, Default::default())
                .expect("Should build successfully");

        let tiles = resources[0].tiles.as_ref().unwrap();
        // Find the tile that has name_type data
        let name_type_node = graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("name_type"))
            .unwrap();

        let has_concept = tiles.iter().any(|t| {
            t.data
                .get(&name_type_node.nodeid)
                .map(|v| v.is_string() && uuid::Uuid::parse_str(v.as_str().unwrap()).is_ok())
                .unwrap_or(false)
        });
        assert!(has_concept, "name_type should be resolved to a UUID");
    }

    #[test]
    fn test_concept_stored_as_value_id_not_concept_id() {
        // C1: a concept tile must store the matched prefLabel's VALUE id (what the
        // ORM read side resolves by), not the concept id.
        let (graph, collections) = build_test_graph();
        let csv = "ResourceID,name_value,name_type\njohn-1,John Smith,Preferred Name";
        let resources =
            build_resources_from_business_csv(csv, &graph, &collections, Default::default())
                .expect("Should build");

        let name_type = graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("name_type"))
            .unwrap();
        let stored = resources[0]
            .tiles
            .as_ref()
            .unwrap()
            .iter()
            .find_map(|t| t.data.get(&name_type.nodeid).and_then(|v| v.as_str()))
            .expect("name_type value stored");

        // The concept labelled "Preferred Name": its id and its prefLabel value id.
        let (concept_id, value_id) = collections
            .iter()
            .flat_map(|c| c.all_concepts.values())
            .find_map(|c| {
                c.pref_labels
                    .values()
                    .find(|v| v.value == "Preferred Name")
                    .map(|v| (c.id.clone(), v.id.clone()))
            })
            .expect("'Preferred Name' concept should exist");

        assert_eq!(
            stored, value_id,
            "concept tile must store the value id (C1)"
        );
        assert_ne!(
            stored, concept_id,
            "concept tile must NOT store the concept id (C1)"
        );
    }

    #[test]
    fn test_multilingual() {
        let (graph, collections) = build_test_graph();

        let csv = "\
ResourceID,name_value,name_value (ga)
john-1,John Smith,Seán Mac Gabhann";

        let resources = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        )
        .expect("Should build successfully");

        let tiles = resources[0].tiles.as_ref().unwrap();
        let name_node = graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("name_value"))
            .unwrap();

        // Find the tile with name data
        let name_tile = tiles
            .iter()
            .find(|t| t.data.contains_key(&name_node.nodeid))
            .expect("Should have a tile with name data");

        let name_val = &name_tile.data[&name_node.nodeid];
        assert!(name_val.get("en").is_some(), "Should have English value");
        assert!(name_val.get("ga").is_some(), "Should have Irish value");
    }

    #[test]
    fn test_multiple_resources() {
        let (graph, collections) = build_test_graph();

        let csv = "\
ResourceID,name_value,birth_date
john-1,John Smith,1850-03-15
jane-1,Jane Doe,1820-06-01";

        let resources = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        )
        .expect("Should build successfully");

        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].resourceinstance.name, "John Smith");
        assert_eq!(resources[1].resourceinstance.name, "Jane Doe");

        // UUIDs should be different
        assert_ne!(
            resources[0].resourceinstance.resourceinstanceid,
            resources[1].resourceinstance.resourceinstanceid
        );
    }

    #[test]
    fn test_deterministic_uuids() {
        let (graph, collections) = build_test_graph();

        let csv = "\
ResourceID,name_value
john-1,John Smith";

        let r1 = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        )
        .unwrap();

        let r2 = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(
            r1[0].resourceinstance.resourceinstanceid,
            r2[0].resourceinstance.resourceinstanceid
        );
        assert_eq!(
            r1[0].tiles.as_ref().unwrap()[0].tileid,
            r2[0].tiles.as_ref().unwrap()[0].tileid,
        );
    }

    #[test]
    fn test_unknown_alias_warning() {
        let (graph, collections) = build_test_graph();

        let csv = "\
ResourceID,name_value,nonexistent_field
john-1,John Smith,some value";

        // Should succeed with warning (unknown column ignored)
        let result = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_resourceid_skipped() {
        let (graph, collections) = build_test_graph();

        let csv = "\
ResourceID,name_value
,John Smith
john-1,Jane Doe";

        let resources = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        )
        .expect("Should build (skipping empty ResourceID)");

        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].resourceinstance.name, "Jane Doe");
    }

    #[test]
    fn test_wrap_business_data() {
        let (graph, collections) = build_test_graph();

        let csv = "\
ResourceID,name_value
john-1,John Smith";

        let resources = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        )
        .unwrap();

        let wrapped = wrap_business_data(&resources);
        assert!(wrapped.get("business_data").is_some());
        assert!(
            wrapped["business_data"]["resources"]
                .as_array()
                .unwrap()
                .len()
                == 1
        );
    }

    // --- CSV link cluster (C2/C3/C4) --------------------------------------

    const LINK_GRAPH_CSV: &str = "name,ontology_class,author,description,is_resource
Patronage,http://www.cidoc-crm.org/cidoc-crm/E21_Person,,links,true";

    // `patron`/`allies` target the single "Faction" model; `either` allows two
    // models (ambiguous for a bare key); `orphan` has no graphs (trailing empty
    // column) — the latter two exercise the reject paths.
    const LINK_NODES_CSV: &str = "\
parent_alias,alias,name,datatype,cardinality,ontology_class,parent_property,description,collection_name,required,searchable,exportable,sortorder,graphs
,patron,Patron,resource-instance,1,http://www.cidoc-crm.org/cidoc-crm/E39_Actor,http://www.cidoc-crm.org/cidoc-crm/P51_has_former_or_current_owner,,,,,,1,Faction
,allies,Allies,resource-instance-list,n,http://www.cidoc-crm.org/cidoc-crm/E39_Actor,http://www.cidoc-crm.org/cidoc-crm/P107i_is_current_or_former_member_of,,,,,,2,Faction
,either,Either,resource-instance,1,http://www.cidoc-crm.org/cidoc-crm/E39_Actor,http://www.cidoc-crm.org/cidoc-crm/P51_has_former_or_current_owner,,,,,,3,Faction|Guild
,orphan,Orphan,resource-instance,1,http://www.cidoc-crm.org/cidoc-crm/E39_Actor,http://www.cidoc-crm.org/cidoc-crm/P51_has_former_or_current_owner,,,,,,4,";

    fn build_link_graph() -> (StaticGraph, Vec<SkosCollection>) {
        build_graph_from_model_csvs(
            LINK_GRAPH_CSV,
            LINK_NODES_CSV,
            None,
            "https://example.org/test",
            MutatorOptions::default(),
        )
        .expect("Failed to build link graph")
    }

    fn faction_graphid() -> String {
        crate::graph_mutator::generate_uuid_v5(("skeleton", None), "Faction")
    }

    fn node_by_alias<'a>(graph: &'a StaticGraph, alias: &str) -> &'a StaticNode {
        graph
            .nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some(alias))
            .unwrap_or_else(|| panic!("node '{}' not found", alias))
    }

    fn link_tile_value<'a>(
        resources: &'a [StaticResource],
        node: &StaticNode,
    ) -> Option<&'a serde_json::Value> {
        resources[0]
            .tiles
            .as_ref()
            .unwrap()
            .iter()
            .find_map(|t| t.data.get(&node.nodeid))
    }

    #[test]
    fn test_link_node_carries_target_graphid_config() {
        // C4: a `graphs` entry resolves to the target model's deterministic graphid
        // and lands in config.graphs; multiple entries produce multiple graphids.
        let (graph, _) = build_link_graph();
        let patron = node_by_alias(&graph, "patron");
        assert_eq!(
            link_target_graphids(patron),
            vec![faction_graphid().as_str()]
        );

        let guild_graphid = crate::graph_mutator::generate_uuid_v5(("skeleton", None), "Guild");
        assert_eq!(
            link_target_graphids(node_by_alias(&graph, "either")),
            vec![faction_graphid().as_str(), guild_graphid.as_str()]
        );
        // A link node with no graphs declares no targets.
        assert!(link_target_graphids(node_by_alias(&graph, "orphan")).is_empty());
    }

    #[test]
    fn test_resource_instance_singular_uses_list_shape_and_resolves_non_uuid() {
        // C2: a single resource-instance is stored as a one-element array of link
        // objects (not a bare string). C3: a non-UUID ResourceID is resolved to the
        // target model's deterministic resource id.
        let (graph, collections) = build_link_graph();
        let csv = "ResourceID,patron\nhouse-1,ferrymen";
        let resources = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        )
        .expect("Should build");

        let patron = node_by_alias(&graph, "patron");
        let value = link_tile_value(&resources, patron).expect("patron link stored");
        let arr = value
            .as_array()
            .expect("resource-instance stored as array (C2)");
        assert_eq!(arr.len(), 1);

        let expected = generate_uuid_v5(("resource", Some(&faction_graphid())), "ferrymen");
        assert_eq!(
            arr[0].get("resourceId").and_then(|v| v.as_str()),
            Some(expected.as_str()),
            "non-UUID ResourceID must resolve via uuid5(target_graph, key) (C3)"
        );
        assert!(
            arr[0].get("resourceXresourceId").is_some(),
            "link object must carry the Arches resourceXresourceId field (C2)"
        );
    }

    #[test]
    fn test_resource_instance_accepts_uuid_verbatim() {
        // C3: a value that is already a UUID is used as-is.
        let (graph, collections) = build_link_graph();
        let uuid = "11111111-1111-1111-1111-111111111111";
        let csv = format!("ResourceID,patron\nhouse-1,{}", uuid);
        let resources = build_resources_from_business_csv(
            &csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        )
        .expect("Should build");

        let patron = node_by_alias(&graph, "patron");
        let value = link_tile_value(&resources, patron).expect("patron link stored");
        assert_eq!(
            value.as_array().unwrap()[0]
                .get("resourceId")
                .and_then(|v| v.as_str()),
            Some(uuid)
        );
    }

    #[test]
    fn test_non_uuid_link_without_target_model_is_rejected() {
        // C3: a non-UUID ResourceID on a link node with no target model cannot be
        // resolved, so it is rejected with an error rather than stored as a dangling
        // literal that never links.
        let (graph, collections) = build_link_graph();
        let csv = "ResourceID,orphan\nhouse-1,ferrymen";
        let result = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        );
        let err = result.expect_err("non-UUID link without a target model must be rejected");
        assert!(
            err.diagnostics
                .iter()
                .any(|d| d.message.contains("no target model")),
            "error should explain the missing target model, got: {:?}",
            err.diagnostics
        );
    }

    #[test]
    fn test_non_uuid_link_with_multiple_targets_is_rejected_as_ambiguous() {
        // C3: a bare ResourceID against a node that allows several target models is
        // ambiguous (the key could live in any of them), so it is rejected.
        let (graph, collections) = build_link_graph();
        let csv = "ResourceID,either\nhouse-1,ferrymen";
        let result = build_resources_from_business_csv(
            csv,
            &graph,
            &collections,
            BusinessDataCsvOptions {
                strict_concepts: false,
                ..Default::default()
            },
        );
        let err = result.expect_err("ambiguous multi-target link must be rejected");
        assert!(
            err.diagnostics
                .iter()
                .any(|d| d.message.contains("ambiguous")),
            "error should explain the ambiguity, got: {:?}",
            err.diagnostics
        );
    }
}
