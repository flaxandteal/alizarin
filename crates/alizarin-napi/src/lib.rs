use std::collections::HashMap;
use std::sync::OnceLock;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use alizarin_core::extension_type_registry::ExtensionTypeRegistry;
use alizarin_core::graph_mutator::MutatorOptions;
use alizarin_core::skos::SkosCollection;
use alizarin_core::type_serialization::SerializationContext;
use alizarin_core::{
    build_graph_from_model_csvs, build_resources_from_business_csv, wrap_business_data,
    PrebuildLoader, StaticGraph, StaticResource, StaticResourceRegistry,
};

// ============================================================================
// Extension registry (shared across all calls)
// ============================================================================

fn extension_registry() -> &'static ExtensionTypeRegistry {
    static REGISTRY: OnceLock<ExtensionTypeRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = ExtensionTypeRegistry::new();
        registry.register(
            alizarin_clm_core::DATATYPE_NAME,
            alizarin_clm_core::create_reference_handler(),
        );
        registry.register(
            alizarin_filelist_core::DATATYPE_NAME,
            alizarin_filelist_core::create_filelist_handler(),
        );
        registry
    })
}

// ============================================================================
// Error conversion
// ============================================================================

fn loader_err(e: alizarin_core::LoaderError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

// ============================================================================
// NapiPrebuildLoader
// ============================================================================

#[napi]
pub struct NapiPrebuildLoader {
    inner: PrebuildLoader,
}

#[napi]
impl NapiPrebuildLoader {
    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        let inner = PrebuildLoader::new(&path).map_err(loader_err)?;
        Ok(NapiPrebuildLoader { inner })
    }

    /// Load a single graph from a JSON file path (relative to prebuild root or absolute).
    #[napi]
    pub fn load_graph(&self, path: String) -> Result<NapiStaticGraph> {
        let graph = self.inner.load_graph(&path).map_err(loader_err)?;
        Ok(NapiStaticGraph { inner: graph })
    }

    /// Load all graphs from prebuild/graphs/resource_models/
    #[napi]
    pub fn load_all_graphs(&self) -> Result<Vec<NapiStaticGraph>> {
        let graphs = self.inner.load_all_graphs().map_err(loader_err)?;
        Ok(graphs
            .into_iter()
            .map(|g| NapiStaticGraph { inner: g })
            .collect())
    }

    /// Load full resources (with tiles) from a single business data file,
    /// filtered by graph ID.
    #[napi]
    pub fn load_full_resources_from_file(
        &self,
        path: String,
        graph_id: String,
    ) -> Result<serde_json::Value> {
        let resources = self
            .inner
            .load_full_resources_from_file(std::path::Path::new(&path), &graph_id)
            .map_err(loader_err)?;
        serde_json::to_value(&resources).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Load all full resources from a single business data file (all graphs).
    #[napi]
    pub fn load_all_full_resources_from_file(&self, path: String) -> Result<serde_json::Value> {
        let resources = self
            .inner
            .load_all_full_resources_from_file(std::path::Path::new(&path))
            .map_err(loader_err)?;
        serde_json::to_value(&resources).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Find all business data JSON files in the prebuild directory.
    #[napi]
    pub fn find_business_data_files(&self) -> Result<Vec<String>> {
        let files = self.inner.find_business_data_files().map_err(loader_err)?;
        Ok(files.into_iter().map(|p| p.display().to_string()).collect())
    }

    /// Get prebuild directory info.
    #[napi]
    pub fn get_info(&self) -> Result<serde_json::Value> {
        let info = self.inner.get_info().map_err(loader_err)?;
        Ok(serde_json::json!({
            "path": info.path.display().to_string(),
            "hasGraphs": info.has_graphs,
            "hasBusinessData": info.has_business_data,
            "hasReferenceData": info.has_reference_data,
            "hasIndexTemplates": info.has_index_templates,
            "graphFiles": info.graph_files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        }))
    }

    /// Count resources for a given graph ID.
    #[napi]
    pub fn count_resources_for_graph(&self, graph_id: String) -> Result<u32> {
        let count = self
            .inner
            .count_resources_for_graph(&graph_id)
            .map_err(loader_err)?;
        Ok(count as u32)
    }
}

// ============================================================================
// NapiStaticGraph
// ============================================================================

#[napi]
pub struct NapiStaticGraph {
    inner: StaticGraph,
}

#[napi]
impl NapiStaticGraph {
    /// Parse a graph from a JSON string (the file content, not a file path).
    #[napi(factory)]
    pub fn from_json_string(json_str: String) -> Result<Self> {
        let graph = StaticGraph::from_json_string(&json_str).map_err(napi::Error::from_reason)?;
        Ok(NapiStaticGraph { inner: graph })
    }

    #[napi(getter)]
    pub fn graph_id(&self) -> String {
        self.inner.graphid.clone()
    }

    #[napi(getter)]
    pub fn name(&self) -> serde_json::Value {
        serde_json::to_value(&self.inner.name).unwrap_or(serde_json::Value::Null)
    }
}

// ============================================================================
// NapiStaticResourceRegistry
// ============================================================================

#[napi]
pub struct NapiStaticResourceRegistry {
    inner: StaticResourceRegistry,
}

impl Default for NapiStaticResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl NapiStaticResourceRegistry {
    #[napi(constructor)]
    pub fn new() -> Self {
        NapiStaticResourceRegistry {
            inner: StaticResourceRegistry::new(),
        }
    }

    /// Insert full resources (with tiles) from a JSON array of StaticResource.
    #[napi]
    pub fn merge_from_resources_json(
        &mut self,
        resources_json: String,
        store_full: Option<bool>,
    ) -> Result<()> {
        let resources: Vec<StaticResource> = serde_json::from_str(&resources_json)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        self.inner
            .merge_from_resources(&resources, store_full.unwrap_or(true), true);
        Ok(())
    }

    /// Insert full resources from a business_data JSON file string.
    #[napi]
    pub fn merge_from_business_data_json(
        &mut self,
        business_data_json: String,
        store_full: Option<bool>,
    ) -> Result<()> {
        // Parse the business_data wrapper to extract resources
        let file: serde_json::Value = serde_json::from_str(&business_data_json)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        let resources_val = file
            .get("business_data")
            .and_then(|bd| bd.get("resources"))
            .ok_or_else(|| {
                napi::Error::from_reason("JSON missing business_data.resources".to_string())
            })?;

        let resources: Vec<StaticResource> = serde_json::from_value(resources_val.clone())
            .map_err(|e| napi::Error::from_reason(format!("Failed to parse resources: {}", e)))?;

        self.inner
            .merge_from_resources(&resources, store_full.unwrap_or(true), true);
        Ok(())
    }

    /// Build an inverted index: visibility value -> [resource IDs].
    #[napi]
    pub fn get_value_to_resources_index(
        &self,
        graph: &NapiStaticGraph,
        node_identifier: String,
        flatten_localized: Option<bool>,
    ) -> Result<HashMap<String, Vec<String>>> {
        let ctx = SerializationContext {
            extension_registry: Some(extension_registry()),
            ..SerializationContext::empty()
        };
        self.inner
            .get_value_to_resources_index_with_context(
                &graph.inner,
                &node_identifier,
                flatten_localized.unwrap_or(true),
                Some(&ctx),
            )
            .map_err(napi::Error::from_reason)
    }

    /// Extract values from one node in tiles where another node matches a filter.
    ///
    /// Both nodes must be in the same nodegroup. Returns raw JSON values from
    /// the extract node for each tile where the filter node's display value
    /// contains any of the filter values.
    #[napi]
    pub fn get_filtered_tile_values(
        &self,
        graph: &NapiStaticGraph,
        filter_node: String,
        filter_values: Vec<String>,
        extract_node: String,
        flatten_localized: Option<bool>,
        required_scope: Option<String>,
    ) -> Result<serde_json::Value> {
        let filter_refs: Vec<&str> = filter_values.iter().map(|s| s.as_str()).collect();
        let ctx = SerializationContext {
            extension_registry: Some(extension_registry()),
            ..SerializationContext::empty()
        };
        let results = self
            .inner
            .get_filtered_tile_values(
                &graph.inner,
                &filter_node,
                &filter_refs,
                &extract_node,
                flatten_localized.unwrap_or(true),
                Some(&ctx),
                required_scope.as_deref(),
            )
            .map_err(napi::Error::from_reason)?;
        serde_json::to_value(&results).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Build a forward index: resource ID -> [node values].
    #[napi]
    pub fn get_node_values_index(
        &self,
        graph: &NapiStaticGraph,
        node_identifier: String,
    ) -> Result<serde_json::Value> {
        let index = self
            .inner
            .get_node_values_index(&graph.inner, &node_identifier)
            .map_err(napi::Error::from_reason)?;
        serde_json::to_value(&index).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    #[napi(getter)]
    pub fn length(&self) -> u32 {
        self.inner.len() as u32
    }

    #[napi]
    pub fn contains(&self, resource_id: String) -> bool {
        self.inner.contains(&resource_id)
    }

    #[napi]
    pub fn has_full(&self, resource_id: String) -> bool {
        self.inner.has_full(&resource_id)
    }
}

// ============================================================================
// CSV model and business data loading
// ============================================================================

fn csv_err(e: alizarin_core::CsvModelError) -> napi::Error {
    let msgs: Vec<String> = e
        .diagnostics
        .iter()
        .map(|d| {
            if let Some(line) = d.line {
                format!("{:?}: {}:{}: {}", d.level, d.file, line, d.message)
            } else {
                format!("{:?}: {}: {}", d.level, d.file, d.message)
            }
        })
        .collect();
    napi::Error::from_reason(msgs.join("\n"))
}

/// Build a StaticGraph from model CSVs (graph.csv, nodes.csv, optional collections.csv).
///
/// Returns the graph as a JSON string. The `rdm_namespace` is used for
/// deterministic ID generation (typically a UUID or URL).
#[napi]
pub fn build_graph_from_csvs(
    graph_csv: String,
    nodes_csv: String,
    collections_csv: Option<String>,
    rdm_namespace: String,
) -> Result<serde_json::Value> {
    let (graph, collections) = build_graph_from_model_csvs(
        &graph_csv,
        &nodes_csv,
        collections_csv.as_deref(),
        &rdm_namespace,
        MutatorOptions::default(),
    )
    .map_err(csv_err)?;

    let graph_json =
        serde_json::to_value(&graph).map_err(|e| napi::Error::from_reason(e.to_string()))?;
    let collections_json =
        serde_json::to_value(&collections).map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(serde_json::json!({
        "graph": graph_json,
        "collections": collections_json,
    }))
}

/// Build StaticResources from a business data CSV, given a graph JSON string
/// and collections JSON string (as returned by `buildGraphFromCsvs`).
///
/// Returns the resources wrapped in the `{ business_data: { resources: [...] } }`
/// format expected by PrebuildLoader.
#[napi]
pub fn build_business_data_from_csv(
    csv_data: String,
    graph_json: String,
    collections_json: String,
) -> Result<serde_json::Value> {
    let graph: StaticGraph = serde_json::from_str(&graph_json)
        .map_err(|e| napi::Error::from_reason(format!("Invalid graph JSON: {e}")))?;
    let collections: Vec<SkosCollection> = serde_json::from_str(&collections_json)
        .map_err(|e| napi::Error::from_reason(format!("Invalid collections JSON: {e}")))?;

    let resources =
        build_resources_from_business_csv(&csv_data, &graph, &collections, Default::default())
            .map_err(csv_err)?;

    Ok(wrap_business_data(&resources))
}
