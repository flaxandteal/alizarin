mod instance_wrapper_napi;

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
    StaticGraph, StaticResource, StaticResourceRegistry,
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
// NapiPrebuildExporter
// ============================================================================

fn exporter_err(e: alizarin_core::ExportError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

#[napi]
#[derive(Default)]
pub struct NapiPrebuildExporter {}

#[napi]
impl NapiPrebuildExporter {
    #[napi(constructor)]
    pub fn new() -> Self {
        NapiPrebuildExporter {}
    }

    /// Export registered graphs to a directory.
    ///
    /// Classifies graphs as resource_models or branches based on `isresource`,
    /// writes as `{"graph": [graph_data]}` JSON files with sorted keys.
    #[napi]
    pub fn export_graphs(&self, graph_ids: Vec<String>, out_dir: String) -> Result<Vec<String>> {
        let data = alizarin_core::export_graphs(&graph_ids).map_err(exporter_err)?;
        let export_data = alizarin_core::PrebuildExportData {
            graph_files: data,
            ..Default::default()
        };
        alizarin_core::write_to_directory(&export_data, std::path::Path::new(&out_dir))
            .map_err(exporter_err)
    }

    /// Export all registered graphs to a directory.
    #[napi]
    pub fn export_all_graphs(&self, out_dir: String) -> Result<Vec<String>> {
        let data = alizarin_core::export_all_graphs().map_err(exporter_err)?;
        let export_data = alizarin_core::PrebuildExportData {
            graph_files: data,
            ..Default::default()
        };
        alizarin_core::write_to_directory(&export_data, std::path::Path::new(&out_dir))
            .map_err(exporter_err)
    }

    /// Build complete export data as JSON (without writing to filesystem).
    ///
    /// Returns an object with `files` array of `{relativePath, content}` entries.
    #[napi]
    pub fn build_export_data(
        &self,
        graph_ids: Option<Vec<String>>,
        base_uri: String,
    ) -> Result<serde_json::Value> {
        let ids = graph_ids.as_deref();
        let data =
            alizarin_core::build_prebuild_export(ids, None, &base_uri).map_err(exporter_err)?;

        let files: Vec<serde_json::Value> = data
            .all_files()
            .iter()
            .map(|f| {
                serde_json::json!({
                    "relativePath": f.relative_path,
                    "content": f.content,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "files": files,
            "graphFileCount": data.graph_files.len(),
            "referenceDataFileCount": data.reference_data_files.len(),
        }))
    }

    /// Get IDs of all registered graphs.
    #[napi]
    pub fn get_registered_graph_ids(&self) -> Vec<String> {
        alizarin_core::get_registered_graph_ids()
    }
}

// ============================================================================
// NapiStaticGraph
// ============================================================================

#[napi]
pub struct NapiStaticGraph {
    inner: StaticGraph,
}

impl NapiStaticGraph {
    pub(crate) fn inner_ref(&self) -> &StaticGraph {
        &self.inner
    }
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

    /// Register this graph in the global registry so NapiResourceInstanceWrapper can use it.
    #[napi]
    pub fn register(&self) {
        alizarin_core::register_graph_owned(self.inner.clone());
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

    /// Load a business_data file from raw bytes (Buffer), parse entirely in Rust,
    /// merge into this registry, and return lightweight refs.
    ///
    /// Equivalent to the WASM `loadFromBusinessDataBytes` but runs natively —
    /// no WASM linear memory limit, and panics produce real stack traces.
    #[napi]
    pub fn load_from_business_data_bytes(
        &mut self,
        bytes: Buffer,
        store_full: Option<bool>,
        include_caches: Option<bool>,
    ) -> Result<Vec<serde_json::Value>> {
        let resources = alizarin_core::parse_business_data_bytes(&bytes).map_err(|e| {
            napi::Error::from_reason(format!("Failed to parse business data: {}", e))
        })?;

        let store_full = store_full.unwrap_or(true);
        let include_caches = include_caches.unwrap_or(true);

        let refs: Vec<serde_json::Value> = resources
            .iter()
            .map(|r| {
                let is_public = r
                    .scopes
                    .as_ref()
                    .and_then(|s| s.as_array())
                    .map(|arr| arr.iter().any(|v| v.as_str() == Some("public")))
                    .unwrap_or(false);
                serde_json::json!({
                    "resourceinstanceid": r.resourceinstance.resourceinstanceid,
                    "graph_id": r.resourceinstance.graph_id,
                    "isPublic": is_public,
                })
            })
            .collect();

        self.inner
            .merge_from_resources(&resources, store_full, include_caches);

        Ok(refs)
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

    /// Get the full resource (with tiles) as a JSON object, or null if only summary stored.
    #[napi]
    pub fn get_full(&self, resource_id: String) -> Result<Option<serde_json::Value>> {
        match self.inner.get_full(&resource_id) {
            Some(r) => serde_json::to_value(r)
                .map(Some)
                .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e))),
            None => Ok(None),
        }
    }

    /// Get a summary for a resource (works for both summary and full entries), or null if unknown.
    #[napi]
    pub fn get_summary(&self, resource_id: String) -> Result<Option<serde_json::Value>> {
        match self.inner.get_summary(&resource_id) {
            Some(s) => serde_json::to_value(&s)
                .map(Some)
                .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e))),
            None => Ok(None),
        }
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

// ============================================================================
// Extension handler direct access
// ============================================================================

/// Coerce a value using the registered extension handler for the given datatype.
///
/// Returns `{ tileData, displayValue }` or null if no handler is registered.
#[napi(js_name = "extensionCoerce")]
pub fn extension_coerce(
    datatype: String,
    value: serde_json::Value,
    config: Option<serde_json::Value>,
) -> Result<Option<serde_json::Value>> {
    let registry = extension_registry();
    match registry.coerce(&datatype, &value, config.as_ref()) {
        Ok(Some(result)) => {
            let output = serde_json::json!({
                "tileData": result.tile_data,
                "displayValue": result.display_value,
            });
            Ok(Some(output))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(napi::Error::from_reason(e.message)),
    }
}

/// Render a display string for tile data using the extension handler.
///
/// Returns the display string, or null if no handler or no display.
#[napi(js_name = "extensionRenderDisplay")]
pub fn extension_render_display(
    datatype: String,
    tile_data: serde_json::Value,
    language: String,
) -> Result<Option<String>> {
    let registry = extension_registry();
    registry
        .render_display(&datatype, &tile_data, &language)
        .map_err(|e| napi::Error::from_reason(e.message))
}

/// Resolve markers in tile data using the extension handler.
///
/// Returns the resolved tile data value.
#[napi(js_name = "extensionResolveMarkers")]
pub fn extension_resolve_markers(
    datatype: String,
    tile_data: serde_json::Value,
    language: String,
) -> Result<serde_json::Value> {
    let registry = extension_registry();
    registry
        .resolve_markers(&datatype, &tile_data, &language)
        .map_err(|e| napi::Error::from_reason(e.message))
}

/// Check if an extension handler is registered for the given datatype.
#[napi(js_name = "hasExtensionHandler")]
pub fn has_extension_handler(datatype: String) -> bool {
    extension_registry().has(&datatype)
}

/// List all registered extension handler datatypes.
#[napi(js_name = "getRegisteredExtensionHandlers")]
pub fn get_registered_extension_handlers() -> Vec<String> {
    extension_registry()
        .list()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

// ============================================================================
// Prebuild import (high-level convenience)
// ============================================================================

/// Import a prebuild/pkg directory: register graphs, load SKOS collections,
/// and load ontology configs.
///
/// 1. Loads and registers all graphs from graphs/resource_models/ and graphs/branches/
/// 2. Parses SKOS XML from reference_data/collections/ into the global RDM cache
/// 3. Loads ontology configs from ontologies/ (if present)
///
/// Returns `{ graphIds, collectionIds, collections, ontologyConfigs }`.
#[napi(js_name = "importPrebuild")]
pub fn import_prebuild(prebuild_dir: String, base_uri: String) -> Result<serde_json::Value> {
    let result = alizarin_core::import_prebuild(&prebuild_dir, &base_uri).map_err(loader_err)?;

    let collections_json = serde_json::to_value(&result.collections)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    let ontology_configs: Vec<serde_json::Value> = result
        .ontology_configs
        .iter()
        .filter_map(|c| serde_json::to_value(c).ok())
        .collect();

    Ok(serde_json::json!({
        "graphIds": result.graph_ids,
        "collectionIds": result.collection_ids,
        "collections": collections_json,
        "ontologyConfigs": ontology_configs,
    }))
}
