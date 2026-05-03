//! Prebuild directory exporter.
//!
//! The converse of [`PrebuildLoader`](crate::loader::PrebuildLoader). Builds
//! export data structures from registered graphs and RDM collections, then
//! optionally writes them to the Arches "pkg" directory structure.
//!
//! Two-layer design:
//! - **Data-building layer** (platform-agnostic): produces [`PrebuildExportData`]
//! - **Directory-writing layer** (`fs_writer`, not available on WASM): writes to disk

use std::collections::HashSet;

use crate::rdm_cache::{rdm_to_skos_collection_excluding, RdmCache, RdmCollection};
use crate::registry::{get_graph, get_registered_graph_ids};
use crate::skos::{collection_to_skos_xml, SkosConcept};
use crate::string_utils::sort_json_keys;
use crate::StaticGraph;

// ============================================================================
// Types
// ============================================================================

/// A single file to be written, as a relative path + content string.
#[derive(Debug, Clone)]
pub struct ExportFile {
    /// Relative path within the pkg directory (e.g. "graphs/resource_models/MyModel.json")
    pub relative_path: String,
    /// File content (JSON or XML)
    pub content: String,
}

/// Complete export data, ready to be written to a directory.
///
/// This is the platform-agnostic output of the data-building layer.
/// WASM consumers use this directly; NAPI/Python consumers pass it
/// to [`fs_writer::write_to_directory`].
#[derive(Debug, Clone, Default)]
pub struct PrebuildExportData {
    pub graph_files: Vec<ExportFile>,
    pub reference_data_files: Vec<ExportFile>,
    pub business_data_files: Vec<ExportFile>,
}

impl PrebuildExportData {
    /// Get all files as a flat list.
    pub fn all_files(&self) -> Vec<&ExportFile> {
        let mut files = Vec::new();
        files.extend(self.graph_files.iter());
        files.extend(self.reference_data_files.iter());
        files.extend(self.business_data_files.iter());
        files
    }

    /// Total number of files.
    pub fn file_count(&self) -> usize {
        self.graph_files.len() + self.reference_data_files.len() + self.business_data_files.len()
    }
}

/// Error type for export operations.
#[derive(Debug)]
pub enum ExportError {
    SerializationError(String),
    RegistryError(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::SerializationError(s) => write!(f, "Serialization error: {}", s),
            ExportError::RegistryError(s) => write!(f, "Registry error: {}", s),
            ExportError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self {
        ExportError::IoError(e)
    }
}

impl From<serde_json::Error> for ExportError {
    fn from(e: serde_json::Error) -> Self {
        ExportError::SerializationError(e.to_string())
    }
}

// ============================================================================
// Data-building layer (platform-agnostic)
// ============================================================================

/// Wrap a graph in the Arches export format `{"graph": [<graph_data>]}` with
/// deterministically sorted keys.
fn wrap_graph_json(graph: &StaticGraph) -> Result<serde_json::Value, ExportError> {
    let graph_value = serde_json::to_value(graph).map_err(|e| {
        ExportError::SerializationError(format!(
            "Failed to serialize graph {}: {}",
            graph.graphid, e
        ))
    })?;
    let sorted = sort_json_keys(graph_value);
    Ok(serde_json::json!({ "graph": [sorted] }))
}

/// Determine the filename for a graph based on its display name.
///
/// Uses the English name (or graphid as fallback), sanitized for filesystem use.
/// Matches the behaviour of the quartz-graphs `run.py` exporter.
fn graph_filename(graph: &StaticGraph) -> String {
    let name = graph.display_name();
    let name = if name.is_empty() {
        graph.graphid.clone()
    } else {
        name
    };
    let sanitized = name.replace('/', "_");
    format!("{}.json", sanitized.trim())
}

/// Export specific graphs by ID from the global registry.
///
/// Classifies each as `resource_models` or `branches` based on `isresource`,
/// wraps as `{"graph": [sorted_data]}`.
pub fn export_graphs(graph_ids: &[String]) -> Result<Vec<ExportFile>, ExportError> {
    let mut files = Vec::new();

    for graph_id in graph_ids {
        let graph = get_graph(graph_id).ok_or_else(|| {
            ExportError::RegistryError(format!("Graph '{}' not registered", graph_id))
        })?;

        let subdir = if graph.isresource.unwrap_or(false) {
            "resource_models"
        } else {
            "branches"
        };

        let filename = graph_filename(&graph);
        let wrapped = wrap_graph_json(&graph)?;
        let content = serde_json::to_string_pretty(&wrapped)?;

        files.push(ExportFile {
            relative_path: format!("graphs/{}/{}", subdir, filename),
            content,
        });
    }

    Ok(files)
}

/// Export all registered graphs.
pub fn export_all_graphs() -> Result<Vec<ExportFile>, ExportError> {
    let ids = get_registered_graph_ids();
    export_graphs(&ids)
}

/// Export RDM collections as SKOS ConceptScheme XML files.
///
/// Iterates all collections in the cache, converts each to SKOS XML
/// as a ConceptScheme and writes to `reference_data/controlled_lists/`.
/// This format is compatible with the `arches_controlled_lists` importer.
///
/// Concepts that appear in multiple collections are only emitted once (in the
/// first collection encountered, sorted by collection ID). This prevents
/// duplicate-key errors when the CLM importer creates `ListItem` rows.
pub fn export_collections(
    rdm_cache: &RdmCache,
    base_uri: &str,
) -> Result<Vec<ExportFile>, ExportError> {
    let mut files = Vec::new();
    let mut seen_concept_ids: HashSet<String> = HashSet::new();

    let mut collection_ids = rdm_cache.get_collection_ids();
    collection_ids.sort();

    for collection_id in &collection_ids {
        if let Some(collection) = rdm_cache.get_collection(collection_id) {
            let file = export_single_collection_excluding(
                collection,
                base_uri,
                "ConceptScheme",
                &mut seen_concept_ids,
            )?;
            if let Some(file) = file {
                files.push(file);
            }
        }
    }

    Ok(files)
}

/// Export a single RDM collection as SKOS XML.
///
/// Callers that need per-collection control over `node_type` can use this
/// instead of [`export_collections`].
pub fn export_single_collection(
    collection: &RdmCollection,
    base_uri: &str,
    node_type: &str,
) -> Result<ExportFile, ExportError> {
    let skos = rdm_to_skos_collection_excluding(collection, node_type, &HashSet::new());
    let xml = collection_to_skos_xml(&skos, base_uri);

    Ok(ExportFile {
        relative_path: format!("reference_data/controlled_lists/{}.xml", collection.id),
        content: xml,
    })
}

/// Export a single collection, excluding already-seen concept IDs.
///
/// Adds this collection's concept IDs to `seen_concept_ids` after building.
/// Returns `None` if the collection would be empty after exclusion.
fn export_single_collection_excluding(
    collection: &RdmCollection,
    base_uri: &str,
    node_type: &str,
    seen_concept_ids: &mut HashSet<String>,
) -> Result<Option<ExportFile>, ExportError> {
    let skos = rdm_to_skos_collection_excluding(collection, node_type, seen_concept_ids);

    // Collect all concept IDs from this collection (including nested children)
    for concept in skos.concepts.values() {
        collect_concept_ids(concept, seen_concept_ids);
    }

    // Skip empty collections
    if skos.concepts.is_empty() {
        return Ok(None);
    }

    let xml = collection_to_skos_xml(&skos, base_uri);

    Ok(Some(ExportFile {
        relative_path: format!("reference_data/controlled_lists/{}.xml", collection.id),
        content: xml,
    }))
}

/// Recursively collect all concept IDs from a concept tree.
fn collect_concept_ids(concept: &SkosConcept, ids: &mut HashSet<String>) {
    ids.insert(concept.id.clone());
    if let Some(children) = &concept.children {
        for child in children {
            collect_concept_ids(child, ids);
        }
    }
}

/// Build a complete prebuild export from registered graphs and an RDM cache.
///
/// This is the main entry point for the data-building layer.
///
/// - `graph_ids`: if `Some`, export only these graphs; if `None`, export all registered graphs.
/// - `rdm_cache`: if `Some`, export collections as SKOS XML.
/// - `base_uri`: base URI for SKOS resources.
pub fn build_prebuild_export(
    graph_ids: Option<&[String]>,
    rdm_cache: Option<&RdmCache>,
    base_uri: &str,
) -> Result<PrebuildExportData, ExportError> {
    let graph_files = match graph_ids {
        Some(ids) => export_graphs(ids)?,
        None => export_all_graphs()?,
    };

    let reference_data_files = match rdm_cache {
        Some(cache) => export_collections(cache, base_uri)?,
        None => Vec::new(),
    };

    Ok(PrebuildExportData {
        graph_files,
        reference_data_files,
        ..Default::default()
    })
}

// ============================================================================
// Directory-writing layer (filesystem, not available in WASM)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub mod fs_writer {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Write [`PrebuildExportData`] to a directory on disk.
    ///
    /// Creates subdirectories as needed. Existing files with the same
    /// names are overwritten.
    ///
    /// Returns the list of absolute paths written.
    pub fn write_to_directory(
        data: &PrebuildExportData,
        out_dir: &Path,
    ) -> Result<Vec<String>, ExportError> {
        let mut written = Vec::new();

        for file in data.all_files() {
            let full_path = out_dir.join(&file.relative_path);

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&full_path, &file.content)?;
            written.push(full_path.display().to_string());
        }

        Ok(written)
    }

    /// Convenience: build export data and write to directory in one step.
    pub fn export_prebuild_to_directory(
        graph_ids: Option<&[String]>,
        rdm_cache: Option<&RdmCache>,
        base_uri: &str,
        out_dir: &Path,
    ) -> Result<Vec<String>, ExportError> {
        let data = super::build_prebuild_export(graph_ids, rdm_cache, base_uri)?;
        write_to_directory(&data, out_dir)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{register_graph_owned, unregister_graph};

    fn test_graph(id: &str, is_resource: bool) -> StaticGraph {
        let json = format!(
            r#"{{
                "graphid": "{}",
                "name": {{"en": "Test {}"}},
                "isresource": {},
                "nodes": [{{
                    "nodeid": "root",
                    "name": "Root",
                    "datatype": "semantic",
                    "graph_id": "{}"
                }}],
                "root": {{
                    "nodeid": "root",
                    "name": "Root",
                    "datatype": "semantic",
                    "graph_id": "{}"
                }}
            }}"#,
            id, id, is_resource, id, id
        );
        StaticGraph::from_json_string(&json).expect("Failed to create test graph")
    }

    #[test]
    fn test_export_graphs_classification() {
        let rm = test_graph("exporter-rm-1", true);
        let branch = test_graph("exporter-branch-1", false);
        register_graph_owned(rm);
        register_graph_owned(branch);

        let files = export_graphs(&["exporter-rm-1".into(), "exporter-branch-1".into()]).unwrap();
        assert_eq!(files.len(), 2);

        let rm_file = files
            .iter()
            .find(|f| f.relative_path.contains("resource_models"))
            .unwrap();
        let br_file = files
            .iter()
            .find(|f| f.relative_path.contains("branches"))
            .unwrap();

        // Verify wrapped format
        let rm_json: serde_json::Value = serde_json::from_str(&rm_file.content).unwrap();
        assert!(rm_json["graph"].is_array());
        assert_eq!(rm_json["graph"][0]["graphid"], "exporter-rm-1");

        let br_json: serde_json::Value = serde_json::from_str(&br_file.content).unwrap();
        assert!(br_json["graph"].is_array());
        assert_eq!(br_json["graph"][0]["graphid"], "exporter-branch-1");

        unregister_graph("exporter-rm-1");
        unregister_graph("exporter-branch-1");
    }

    #[test]
    fn test_deterministic_output() {
        let g = test_graph("exporter-det-1", true);
        register_graph_owned(g);

        let files1 = export_graphs(&["exporter-det-1".into()]).unwrap();
        let files2 = export_graphs(&["exporter-det-1".into()]).unwrap();

        assert_eq!(files1[0].content, files2[0].content);

        unregister_graph("exporter-det-1");
    }

    #[test]
    fn test_export_collections() {
        let mut cache = RdmCache::new();
        cache
            .add_collection_from_json(
                "exporter-coll-1",
                r#"[{"id": "c1", "prefLabel": {"en": "Concept One"}}]"#,
            )
            .unwrap();

        let files = export_collections(&cache, "http://example.org/").unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0]
            .relative_path
            .contains("reference_data/controlled_lists/"));
        assert!(files[0].content.contains("xml"));
        assert!(files[0].content.contains("Concept One"));
    }

    #[test]
    fn test_export_missing_graph() {
        let result = export_graphs(&["nonexistent-exporter-test".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_prebuild_export() {
        let g = test_graph("exporter-full-1", true);
        register_graph_owned(g);

        let mut cache = RdmCache::new();
        cache
            .add_collection_from_json(
                "exporter-coll-2",
                r#"[{"id": "c2", "prefLabel": {"en": "Test Concept"}}]"#,
            )
            .unwrap();

        let data = build_prebuild_export(
            Some(&["exporter-full-1".into()]),
            Some(&cache),
            "http://example.org/",
        )
        .unwrap();

        assert_eq!(data.graph_files.len(), 1);
        assert_eq!(data.reference_data_files.len(), 1);
        assert_eq!(data.business_data_files.len(), 0);
        assert_eq!(data.file_count(), 2);
        assert_eq!(data.all_files().len(), 2);

        unregister_graph("exporter-full-1");
    }

    #[test]
    fn test_graph_round_trip() {
        // Verify the exported format can be parsed back by from_json_string
        let g = test_graph("exporter-roundtrip-1", true);
        register_graph_owned(g);

        let files = export_graphs(&["exporter-roundtrip-1".into()]).unwrap();
        let parsed = StaticGraph::from_json_string(&files[0].content).unwrap();
        assert_eq!(parsed.graphid, "exporter-roundtrip-1");

        unregister_graph("exporter-roundtrip-1");
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod fs_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_write_to_directory() {
        let data = PrebuildExportData {
            graph_files: vec![ExportFile {
                relative_path: "graphs/resource_models/Test.json".into(),
                content: r#"{"graph": [{}]}"#.into(),
            }],
            ..Default::default()
        };

        let tmp = std::env::temp_dir().join("alizarin_exporter_test");
        let _ = std::fs::remove_dir_all(&tmp);

        let written = fs_writer::write_to_directory(&data, &tmp).unwrap();
        assert_eq!(written.len(), 1);
        assert!(PathBuf::from(&written[0]).exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
