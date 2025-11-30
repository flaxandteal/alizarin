//! File system loader for prebuild directories
//!
//! This module handles loading graphs and other data from the prebuild
//! directory structure used by starches-builder.

use crate::graph::{IndexedGraph, StaticGraph};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_not_found() {
        let result = PrebuildLoader::new("/nonexistent/path");
        assert!(matches!(result, Err(LoaderError::NotFound(_))));
    }
}
