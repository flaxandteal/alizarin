//! File system loader for prebuild directories
//!
//! This module handles loading graphs and other data from the prebuild
//! directory structure used by starches-builder.

use crate::graph::{IndexedGraph, StaticGraph, StaticResource, StaticResourceDescriptors, StaticResourceMetadata, StaticResourceSummary, StaticTile};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

// ============================================================================
// Business Data File Deserialization Types
// ============================================================================

/// Top-level wrapper for business_data JSON files
#[derive(Debug, Deserialize)]
struct BusinessDataFile {
    business_data: BusinessDataContent,
}

/// Content of the business_data section
#[derive(Debug, Deserialize)]
struct BusinessDataContent {
    #[serde(default)]
    resources: Vec<BusinessDataResource>,
}

/// A single resource in the business_data file
#[derive(Debug, Deserialize)]
struct BusinessDataResource {
    resourceinstance: BusinessDataResourceInstance,
    #[serde(default)]
    metadata: Option<HashMap<String, String>>,
}

/// The resourceinstance object within a resource
#[derive(Debug, Deserialize)]
struct BusinessDataResourceInstance {
    resourceinstanceid: String,
    graph_id: String,
    name: String,
    #[serde(default)]
    descriptors: Option<LanguageNestedDescriptors>,
    #[serde(default)]
    createdtime: Option<String>,
    #[serde(default)]
    lastmodified: Option<String>,
    #[serde(default)]
    publication_id: Option<String>,
    #[serde(default)]
    principaluser_id: Option<i32>,
    #[serde(default)]
    legacyid: Option<String>,
    #[serde(default)]
    graph_publication_id: Option<String>,
}

/// Language-nested descriptors (e.g., { "en": { "name": "...", ... } })
#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct LanguageNestedDescriptors {
    languages: HashMap<String, StaticResourceDescriptors>,
}

impl LanguageNestedDescriptors {
    /// Get descriptors for the preferred language (default: "en")
    fn get_for_lang(&self, lang: &str) -> Option<StaticResourceDescriptors> {
        self.languages
            .get(lang)
            .or_else(|| self.languages.get("en"))
            .or_else(|| self.languages.values().next())
            .cloned()
    }
}

impl BusinessDataResource {
    /// Convert to StaticResourceSummary
    fn to_summary(&self) -> StaticResourceSummary {
        let ri = &self.resourceinstance;
        StaticResourceSummary {
            resourceinstanceid: ri.resourceinstanceid.clone(),
            graph_id: ri.graph_id.clone(),
            name: ri.name.clone(),
            descriptors: ri.descriptors.as_ref().and_then(|d| d.get_for_lang("en")),
            metadata: self.metadata.clone().unwrap_or_default(),
            createdtime: ri.createdtime.clone(),
            lastmodified: ri.lastmodified.clone(),
            publication_id: ri.publication_id.clone(),
            principaluser_id: ri.principaluser_id,
            legacyid: ri.legacyid.clone(),
            graph_publication_id: ri.graph_publication_id.clone(),
        }
    }
}

// ============================================================================
// Fast Count Types (minimal deserialization for counting)
// ============================================================================

/// Minimal struct for fast counting - only deserializes what we need
#[derive(Debug, Deserialize)]
struct BusinessDataFileCount {
    business_data: BusinessDataContentCount,
}

/// Count content - resources as raw values we just count
#[derive(Debug, Deserialize)]
struct BusinessDataContentCount {
    #[serde(default)]
    resources: Vec<BusinessDataResourceCount>,
}

/// Minimal resource - only deserialize graph_id for filtering
#[derive(Debug, Deserialize)]
struct BusinessDataResourceCount {
    resourceinstance: BusinessDataResourceInstanceCount,
}

#[derive(Debug, Deserialize)]
struct BusinessDataResourceInstanceCount {
    graph_id: String,
}

// ============================================================================
// Full Resource Loading Types
// ============================================================================

/// Full business data file with complete resource data including tiles
#[derive(Debug, Deserialize)]
struct BusinessDataFileFull {
    business_data: BusinessDataContentFull,
}

#[derive(Debug, Deserialize)]
struct BusinessDataContentFull {
    #[serde(default)]
    resources: Vec<BusinessDataResourceFull>,
}

/// Full resource with tiles
#[derive(Debug, Deserialize)]
struct BusinessDataResourceFull {
    resourceinstance: BusinessDataResourceInstanceFull,
    #[serde(default)]
    tiles: Option<Vec<StaticTile>>,
    #[serde(default)]
    metadata: Option<HashMap<String, String>>,
}

/// Full resource instance for loading
#[derive(Debug, Deserialize)]
struct BusinessDataResourceInstanceFull {
    resourceinstanceid: String,
    graph_id: String,
    name: String,
    #[serde(default)]
    descriptors: Option<LanguageNestedDescriptors>,
    #[serde(default)]
    createdtime: Option<String>,
    #[serde(default)]
    lastmodified: Option<String>,
    #[serde(default)]
    publication_id: Option<String>,
    #[serde(default)]
    principaluser_id: Option<i32>,
    #[serde(default)]
    legacyid: Option<String>,
    #[serde(default)]
    graph_publication_id: Option<String>,
}

impl BusinessDataResourceFull {
    /// Convert to StaticResource
    fn to_static_resource(&self) -> StaticResource {
        let ri = &self.resourceinstance;
        let descriptors = ri.descriptors.as_ref()
            .and_then(|d| d.get_for_lang("en"))
            .unwrap_or_default();

        StaticResource {
            resourceinstance: StaticResourceMetadata {
                resourceinstanceid: ri.resourceinstanceid.clone(),
                graph_id: ri.graph_id.clone(),
                name: ri.name.clone(),
                descriptors,
                createdtime: ri.createdtime.clone(),
                lastmodified: ri.lastmodified.clone(),
                publication_id: ri.publication_id.clone(),
                principaluser_id: ri.principaluser_id,
                legacyid: ri.legacyid.clone(),
                graph_publication_id: ri.graph_publication_id.clone(),
            },
            tiles: self.tiles.clone(),
            metadata: self.metadata.clone().unwrap_or_default(),
            cache: None,
            scopes: None,
            tiles_loaded: Some(true),
        }
    }
}

/// Error type for loader operations
#[derive(Debug)]
pub enum LoaderError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    GraphError(String),
    NotFound(String),
}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::IoError(e) => write!(f, "IO error: {}", e),
            LoaderError::JsonError(e) => write!(f, "JSON error: {}", e),
            LoaderError::GraphError(s) => write!(f, "Graph error: {}", s),
            LoaderError::NotFound(s) => write!(f, "Not found: {}", s),
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        LoaderError::IoError(e)
    }
}

impl From<serde_json::Error> for LoaderError {
    fn from(e: serde_json::Error) -> Self {
        LoaderError::JsonError(e)
    }
}

/// Metadata about the prebuild directory
#[derive(Debug, Clone)]
pub struct PrebuildInfo {
    pub path: PathBuf,
    pub has_graphs: bool,
    pub has_business_data: bool,
    pub has_reference_data: bool,
    pub has_index_templates: bool,
    pub graph_files: Vec<PathBuf>,
}

/// Loader for prebuild directories
pub struct PrebuildLoader {
    root_path: PathBuf,
}

impl PrebuildLoader {
    /// Create a new loader for the given prebuild directory
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, LoaderError> {
        let root_path = path.as_ref().to_path_buf();
        if !root_path.exists() {
            return Err(LoaderError::NotFound(format!(
                "Prebuild directory not found: {}",
                root_path.display()
            )));
        }
        Ok(PrebuildLoader { root_path })
    }

    /// Get information about what's in the prebuild directory
    pub fn get_info(&self) -> Result<PrebuildInfo, LoaderError> {
        let graphs_dir = self.root_path.join("graphs");
        let business_data_dir = self.root_path.join("business_data");
        let reference_data_dir = self.root_path.join("reference_data");
        let index_templates_dir = self.root_path.join("indexTemplates");

        let graph_files = if graphs_dir.exists() {
            self.find_graph_files(&graphs_dir)?
        } else {
            Vec::new()
        };

        Ok(PrebuildInfo {
            path: self.root_path.clone(),
            has_graphs: !graph_files.is_empty(),
            has_business_data: business_data_dir.exists(),
            has_reference_data: reference_data_dir.exists(),
            has_index_templates: index_templates_dir.exists(),
            graph_files,
        })
    }

    /// Find all graph JSON files in the graphs directory
    fn find_graph_files(&self, graphs_dir: &Path) -> Result<Vec<PathBuf>, LoaderError> {
        let mut files = Vec::new();

        // Check resource_models subdirectory
        let resource_models = graphs_dir.join("resource_models");
        if resource_models.exists() {
            for entry in fs::read_dir(&resource_models)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    files.push(path);
                }
            }
        }

        // Check branches subdirectory
        let branches = graphs_dir.join("branches");
        if branches.exists() {
            for entry in fs::read_dir(&branches)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    /// Load a single graph from a JSON file
    pub fn load_graph<P: AsRef<Path>>(&self, path: P) -> Result<StaticGraph, LoaderError> {
        let content = fs::read_to_string(path.as_ref())?;
        StaticGraph::from_json_string(&content).map_err(LoaderError::GraphError)
    }

    /// Load a single graph and create an indexed version
    pub fn load_indexed_graph<P: AsRef<Path>>(&self, path: P) -> Result<IndexedGraph, LoaderError> {
        let graph = self.load_graph(path)?;
        Ok(IndexedGraph::new(graph))
    }

    /// Load all graphs from the graphs directory
    pub fn load_all_graphs(&self) -> Result<Vec<StaticGraph>, LoaderError> {
        let info = self.get_info()?;
        let mut graphs = Vec::new();

        for path in &info.graph_files {
            match self.load_graph(path) {
                Ok(graph) => graphs.push(graph),
                Err(e) => {
                    eprintln!("Warning: Failed to load graph {}: {}", path.display(), e);
                }
            }
        }

        Ok(graphs)
    }

    /// Load all graphs and create indexed versions
    pub fn load_all_indexed_graphs(&self) -> Result<Vec<IndexedGraph>, LoaderError> {
        let graphs = self.load_all_graphs()?;
        Ok(graphs.into_iter().map(IndexedGraph::new).collect())
    }

    /// Load all graphs into a map keyed by graph ID
    pub fn load_graphs_by_id(&self) -> Result<HashMap<String, IndexedGraph>, LoaderError> {
        let graphs = self.load_all_indexed_graphs()?;
        Ok(graphs
            .into_iter()
            .map(|g| (g.graph.graphid.clone(), g))
            .collect())
    }

    /// Get the path to a specific subdirectory
    pub fn get_subdir(&self, name: &str) -> PathBuf {
        self.root_path.join(name)
    }

    /// Get the root path
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Find all business data JSON files (searches recursively)
    pub fn find_business_data_files(&self, _graph_id: &str) -> Result<Vec<PathBuf>, LoaderError> {
        let business_data_dir = self.root_path.join("business_data");
        if !business_data_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        self.collect_json_files(&business_data_dir, &mut files)?;
        Ok(files)
    }

    /// Recursively collect all JSON files from a directory
    fn collect_json_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), LoaderError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.collect_json_files(&path, files)?;
            } else if path.extension().map(|e| e == "json").unwrap_or(false) {
                files.push(path);
            }
        }
        Ok(())
    }

    /// Load resource summaries from a single business data file
    /// Uses typed deserialization for fast parsing
    pub fn load_resource_summaries_from_file(
        &self,
        path: &Path,
        graph_id: &str,
    ) -> Result<Vec<StaticResourceSummary>, LoaderError> {
        let content = fs::read_to_string(path)?;
        let file: BusinessDataFile = serde_json::from_str(&content)?;

        let summaries: Vec<StaticResourceSummary> = file
            .business_data
            .resources
            .into_iter()
            .filter(|r| r.resourceinstance.graph_id == graph_id)
            .map(|r| r.to_summary())
            .collect();

        Ok(summaries)
    }

    /// Load resource summaries for a graph, with optional limit
    /// Returns (summaries, has_more)
    pub fn load_resource_summaries(
        &self,
        graph_id: &str,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<StaticResourceSummary>, bool), LoaderError> {
        let files = self.find_business_data_files(graph_id)?;
        let mut all_summaries = Vec::new();

        for file in &files {
            match self.load_resource_summaries_from_file(file, graph_id) {
                Ok(summaries) => all_summaries.extend(summaries),
                Err(e) => {
                    eprintln!("Warning: Failed to load resources from {}: {}", file.display(), e);
                }
            }
        }

        // Apply offset and limit
        let total = all_summaries.len();
        let has_more = offset + limit < total;
        let summaries: Vec<_> = all_summaries
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        Ok((summaries, has_more))
    }

    /// Get total count of resources for a graph (without loading all data)
    pub fn count_resources_for_graph(&self, graph_id: &str) -> Result<usize, LoaderError> {
        let files = self.find_business_data_files(graph_id)?;
        let mut count = 0;

        for file in &files {
            count += self.fast_count_resources_in_file(file, graph_id)?;
        }

        Ok(count)
    }

    /// Fast count of resources in a single file (minimal deserialization)
    pub fn fast_count_resources_in_file(&self, path: &Path, graph_id: &str) -> Result<usize, LoaderError> {
        let content = fs::read_to_string(path)?;
        let file_data: BusinessDataFileCount = serde_json::from_str(&content)?;

        let count = file_data
            .business_data
            .resources
            .iter()
            .filter(|r| r.resourceinstance.graph_id == graph_id)
            .count();

        Ok(count)
    }

    /// Get file counts for per-file progress tracking
    /// Returns Vec of (file_path, resource_count) for each file
    pub fn get_business_data_file_counts(&self, graph_id: &str) -> Result<Vec<(PathBuf, usize)>, LoaderError> {
        let files = self.find_business_data_files(graph_id)?;
        let mut result = Vec::with_capacity(files.len());

        for file in files {
            let count = self.fast_count_resources_in_file(&file, graph_id)?;
            if count > 0 {
                result.push((file, count));
            }
        }

        Ok(result)
    }

    /// Load a full StaticResource by its resourceinstanceid
    /// Searches through all business_data files to find the resource
    pub fn load_full_resource(&self, resource_id: &str, graph_id: &str) -> Result<StaticResource, LoaderError> {
        let files = self.find_business_data_files(graph_id)?;

        for file in &files {
            let content = fs::read_to_string(file)?;
            let file_data: BusinessDataFileFull = serde_json::from_str(&content)?;

            for resource in file_data.business_data.resources {
                if resource.resourceinstance.resourceinstanceid == resource_id {
                    return Ok(resource.to_static_resource());
                }
            }
        }

        Err(LoaderError::NotFound(format!(
            "Resource {} not found in graph {}",
            resource_id, graph_id
        )))
    }

    // =========================================================================
    // Parallel Loading Methods (requires "parallel" feature)
    // =========================================================================

    /// Load resources from multiple files in parallel, sending batches via channel.
    /// Falls back to sequential loading if "parallel" feature is not enabled.
    ///
    /// The callback is called for each file's results as they complete.
    /// Returns total count of resources loaded.
    #[cfg(feature = "parallel")]
    pub fn load_resources_parallel(
        &self,
        files: &[(PathBuf, usize)],
        graph_id: &str,
        tx: &Sender<Vec<StaticResourceSummary>>,
    ) -> Result<usize, LoaderError> {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let total_loaded = AtomicUsize::new(0);
        let graph_id = graph_id.to_string();

        // Process files in parallel using rayon
        files.par_iter().for_each(|(file_path, _count)| {
            if let Ok(summaries) = self.load_resource_summaries_from_file(file_path, &graph_id) {
                if !summaries.is_empty() {
                    total_loaded.fetch_add(summaries.len(), Ordering::Relaxed);
                    let _ = tx.send(summaries);
                }
            }
        });

        Ok(total_loaded.load(Ordering::Relaxed))
    }

    /// Sequential fallback when parallel feature is not enabled
    #[cfg(not(feature = "parallel"))]
    pub fn load_resources_parallel(
        &self,
        files: &[(PathBuf, usize)],
        graph_id: &str,
        tx: &Sender<Vec<StaticResourceSummary>>,
    ) -> Result<usize, LoaderError> {
        let mut total_loaded = 0;

        for (file_path, _count) in files {
            if let Ok(summaries) = self.load_resource_summaries_from_file(file_path, graph_id) {
                if !summaries.is_empty() {
                    total_loaded += summaries.len();
                    let _ = tx.send(summaries);
                }
            }
        }

        Ok(total_loaded)
    }

    /// Count resources in files in parallel (for initial count phase)
    #[cfg(feature = "parallel")]
    pub fn count_resources_parallel(
        &self,
        files: &[PathBuf],
        graph_id: &str,
    ) -> Vec<(PathBuf, usize)> {
        files
            .par_iter()
            .filter_map(|file| {
                match self.fast_count_resources_in_file(file, graph_id) {
                    Ok(count) if count > 0 => Some((file.clone(), count)),
                    _ => None,
                }
            })
            .collect()
    }

    /// Sequential fallback for counting
    #[cfg(not(feature = "parallel"))]
    pub fn count_resources_parallel(
        &self,
        files: &[PathBuf],
        graph_id: &str,
    ) -> Vec<(PathBuf, usize)> {
        files
            .iter()
            .filter_map(|file| {
                match self.fast_count_resources_in_file(file, graph_id) {
                    Ok(count) if count > 0 => Some((file.clone(), count)),
                    _ => None,
                }
            })
            .collect()
    }

    // =========================================================================
    // Preindex Loading Methods
    // =========================================================================

    /// Find all preindex .pi files (searches recursively)
    pub fn find_preindex_files(&self, _graph_id: &str) -> Result<Vec<PathBuf>, LoaderError> {
        let preindex_dir = self.root_path.join("preindex");
        if !preindex_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        self.collect_pi_files(&preindex_dir, &mut files)?;
        Ok(files)
    }

    /// Recursively collect all .pi files from a directory
    fn collect_pi_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), LoaderError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.collect_pi_files(&path, files)?;
            } else if path.extension().map(|e| e == "pi").unwrap_or(false) {
                files.push(path);
            }
        }
        Ok(())
    }

    /// Load resource summaries from preindex .pi files
    /// .pi files contain StaticResourceSummary objects directly (one per line or as JSON array)
    pub fn load_preindex_summaries(
        &self,
        graph_id: &str,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<StaticResourceSummary>, bool), LoaderError> {
        let files = self.find_preindex_files(graph_id)?;
        let mut all_summaries = Vec::new();

        for file in &files {
            match self.load_preindex_file(file, graph_id) {
                Ok(summaries) => all_summaries.extend(summaries),
                Err(e) => {
                    eprintln!("Warning: Failed to load preindex from {}: {}", file.display(), e);
                }
            }
        }

        // Apply offset and limit
        let total = all_summaries.len();
        let has_more = offset + limit < total;
        let summaries: Vec<_> = all_summaries
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        Ok((summaries, has_more))
    }

    /// Load a single preindex file
    fn load_preindex_file(
        &self,
        path: &Path,
        graph_id: &str,
    ) -> Result<Vec<StaticResourceSummary>, LoaderError> {
        let content = fs::read_to_string(path)?;
        let mut summaries = Vec::new();

        // Try parsing as JSON array first
        if let Ok(array) = serde_json::from_str::<Vec<StaticResourceSummary>>(&content) {
            for summary in array {
                if summary.graph_id == graph_id {
                    summaries.push(summary);
                }
            }
            return Ok(summaries);
        }

        // Try parsing as newline-delimited JSON (NDJSON)
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(summary) = serde_json::from_str::<StaticResourceSummary>(line) {
                if summary.graph_id == graph_id {
                    summaries.push(summary);
                }
            }
        }

        Ok(summaries)
    }

    /// Count resources in preindex files for a graph
    pub fn count_preindex_resources_for_graph(&self, graph_id: &str) -> Result<usize, LoaderError> {
        let files = self.find_preindex_files(graph_id)?;
        let mut count = 0;

        for file in &files {
            if let Ok(summaries) = self.load_preindex_file(file, graph_id) {
                count += summaries.len();
            }
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::StaticGraph;
    use std::path::PathBuf;

    #[test]
    fn test_loader_not_found() {
        let result = PrebuildLoader::new("/nonexistent/path");
        assert!(matches!(result, Err(LoaderError::NotFound(_))));
    }

    #[test]
    fn test_parse_coral_format_json() {
        // Test parsing JSON without the new Arches-HER 2.0+ fields
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let test_path = PathBuf::from(manifest_dir)
            .parent().unwrap()
            .parent().unwrap()
            .join("tests/data/models/Person.json");

        let content = std::fs::read_to_string(&test_path)
            .expect("Failed to read test JSON file");

        let data: serde_json::Value = serde_json::from_str(&content)
            .expect("Failed to parse JSON");

        let graph_json = &data["graph"][0];

        // Verify the old format doesn't have the new fields
        assert!(graph_json.get("source_identifier_id").is_none() ||
                graph_json["source_identifier_id"].is_null());

        // Parse as StaticGraph - this should succeed with defaults for missing fields
        let graph: StaticGraph = serde_json::from_value(graph_json.clone())
            .expect("Failed to parse StaticGraph from Coral format");

        assert!(!graph.graphid.is_empty());
        assert!(graph.source_identifier_id.is_none()); // Defaults to None
        assert!(graph.is_active.is_none()); // Defaults to None
        assert!(!graph.nodes.is_empty());
    }

    #[test]
    fn test_parse_arches_her_format_json() {
        // Test parsing JSON with the new Arches-HER 2.0+ fields
        let json = r#"{
            "graphid": "test-graph-id",
            "name": {"en": "Test Graph"},
            "nodes": [],
            "edges": [],
            "nodegroups": [],
            "cards": [],
            "cards_x_nodes_x_widgets": [],
            "functions_x_graphs": [],
            "root": {
                "nodeid": "root-node-id",
                "name": "Root Node",
                "datatype": "semantic",
                "graph_id": "test-graph-id"
            },
            "source_identifier_id": "some-source-id",
            "is_active": true,
            "has_unpublished_changes": false,
            "is_copy_immutable": false
        }"#;

        let graph: StaticGraph = serde_json::from_str(json)
            .expect("Failed to parse StaticGraph with Arches-HER fields");

        assert_eq!(graph.graphid, "test-graph-id");
        assert_eq!(graph.source_identifier_id, Some("some-source-id".to_string()));
        assert_eq!(graph.is_active, Some(true));
        assert_eq!(graph.has_unpublished_changes, Some(false));
    }
}
