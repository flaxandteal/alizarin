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
use crate::graph::{
    StaticGraph, StaticNode, StaticResource, StaticResourceDescriptors, StaticResourceMetadata,
    StaticTile,
};
use crate::graph_mutator::generate_uuid_v5;
use crate::skos::SkosCollection;

/// Options for business data CSV loading
#[derive(Debug, Clone)]
pub struct BusinessDataCsvOptions {
    /// Default language code for string values (default: "en")
    pub default_language: String,
    /// Whether to error on unresolved concept labels (default: true)
    pub strict_concepts: bool,
}

impl Default for BusinessDataCsvOptions {
    fn default() -> Self {
        Self {
            default_language: "en".to_string(),
            strict_concepts: true,
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

/// Build concept label→UUID lookup from collections
fn build_concept_lookup(
    collections: &[SkosCollection],
) -> HashMap<String, HashMap<String, String>> {
    // collection_id -> (lowercase_label -> concept_id)
    let mut lookup: HashMap<String, HashMap<String, String>> = HashMap::new();

    for coll in collections {
        let mut labels: HashMap<String, String> = HashMap::new();
        for (concept_id, concept) in &coll.all_concepts {
            for pref_label in concept.pref_labels.values() {
                labels.insert(pref_label.value.to_lowercase(), concept_id.clone());
            }
        }
        lookup.insert(coll.id.clone(), labels);
    }

    lookup
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
    diagnostics: &'a mut Vec<CsvModelDiagnostic>,
    line: usize,
    strict_concepts: bool,
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
        "date" => Some(serde_json::Value::String(raw.to_string())),
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
        "resource-instance-list" => {
            let arr: Vec<serde_json::Value> = raw
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|r| {
                    let rxr_id = generate_uuid_v5(("resource-x-resource", None), r);
                    serde_json::json!({
                        "resourceId": r,
                        "resourceXresourceId": rxr_id,
                        "ontologyProperty": "",
                        "inverseOntologyProperty": ""
                    })
                })
                .collect();
            if arr.is_empty() {
                None
            } else {
                Some(serde_json::Value::Array(arr))
            }
        }
        "geojson-feature-collection" => match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(v) => Some(v),
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

/// Resolve a concept label to its UUID
fn resolve_concept_label(
    label: &str,
    node: &StaticNode,
    ctx: &mut CoerceContext<'_>,
) -> Option<String> {
    let lower = label.to_lowercase();

    // If it's already a UUID, return as-is
    if uuid::Uuid::parse_str(label).is_ok() {
        return Some(label.to_string());
    }

    // Try to find via collection mapping
    if let Some(coll_id) = find_node_collection_id(node, ctx.collections) {
        if let Some(labels) = ctx.concept_lookup.get(&coll_id) {
            if let Some(concept_id) = labels.get(&lower) {
                return Some(concept_id.clone());
            }
        }
    }

    // Fallback: search all collections
    for labels in ctx.concept_lookup.values() {
        if let Some(concept_id) = labels.get(&lower) {
            return Some(concept_id.clone());
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
    let mut diagnostics: Vec<CsvModelDiagnostic> = Vec::new();

    // Build lookup indices
    let alias_to_node: HashMap<String, &StaticNode> = graph
        .nodes
        .iter()
        .filter_map(|n| n.alias.as_ref().map(|a| (a.clone(), n)))
        .collect();

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
                    diagnostics: &mut diagnostics,
                    line: *line,
                    strict_concepts: options.strict_concepts,
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
                    let tileid = generate_uuid_v5(
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

                let tileid = generate_uuid_v5(("tile", Some(&resourceinstanceid)), ng_id);

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
}
