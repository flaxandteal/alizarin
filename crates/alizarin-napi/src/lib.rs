mod instance_wrapper_napi;
mod json_arg;
mod node_config_napi;

use instance_wrapper_napi::NapiRdmCache;
pub(crate) use json_arg::{JsonArg, JsonOut};

use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use alizarin_core::extension_type_registry::ExtensionTypeRegistry;
use alizarin_core::graph_mutator::MutatorOptions;
use alizarin_core::label_resolution::ConceptLookup;
use alizarin_core::skos::SkosCollection;
use alizarin_core::type_serialization::SerializationContext;
use alizarin_core::{
    build_graph_from_model_csvs, build_resources_from_business_csv,
    build_resources_from_business_csv_with_context, wrap_business_data, StaticGraph,
    StaticResource, StaticResourceRegistry, StaticResourceSummary,
};

// ============================================================================
// Extension registry (shared across all calls)
// ============================================================================

/// Build a fresh extension registry from the C-ABI handlers registered in core.
///
/// Extensions register over the ABI at load (see [`register_extension_handler`]),
/// not as compile-time deps — so this crate carries no `ext/*` dependency. Rebuilt
/// per call (cheap: wraps registered handler names in `Arc`), mirroring Python.
fn extension_registry() -> ExtensionTypeRegistry {
    alizarin_core::build_extension_registry_from_registered()
}

/// Register an extension type handler from a raw `TypeHandlerInfo` pointer, passed
/// from the extension's native module through JS as a `BigInt`.
///
/// The pointer-passing (Python-capsule-style) approach avoids per-value
/// serialization: the fn-pointers are called directly Rust→Rust. Returns the
/// datatype name on success.
#[napi(js_name = "registerExtensionHandler")]
pub fn register_extension_handler(handler_ptr: BigInt) -> Result<String> {
    let ptr = handler_ptr.get_u64().1 as *const alizarin_extension_api::TypeHandlerInfo;
    // SAFETY: `ptr` is a `TypeHandlerInfo` produced by an extension's native module
    // and handed across as a BigInt; core verifies the ABI fingerprint before
    // reading any field or calling any fn. The extension must keep it alive.
    unsafe { alizarin_core::register_handler_from_ptr(ptr) }.map_err(napi::Error::from_reason)
}

/// Register an extension's derive provider from its raw `DeriveProviderInfo`
/// pointer (as a `BigInt`), under the provider id it carries, which is
/// returned. As WASM's `registerDeriveProvider` (JS callbacks) and Python's
/// (a capsule).
#[napi(js_name = "registerDeriveProvider")]
pub fn register_derive_provider(info_ptr: BigInt) -> Result<String> {
    let ptr = info_ptr.get_u64().1 as *const alizarin_extension_api::DeriveProviderInfo;
    // SAFETY: `ptr` is a `DeriveProviderInfo` from an extension's native
    // module; core checks its ABI fingerprint before reading anything else.
    unsafe { alizarin_core::derive_abi::register_derive_provider_from_ptr(ptr) }
        .map_err(napi::Error::from_reason)
}

/// Whether a derive provider is registered under `providerId`. As on WASM
/// and Python.
#[napi(js_name = "hasDeriveProvider")]
pub fn has_derive_provider(provider_id: String) -> bool {
    alizarin_core::get_global_functions_registry()
        .derive(&provider_id)
        .is_some()
}

/// Whether any value folded into `summary` (a `datatype` fold summary, e.g. an
/// Aggregate node value) could match `predicate`. Conservative: `false` only
/// when certain, so whatever the summary covers can be skipped. Predicates:
/// `{range: {min, max}}` for number/date/edtf, or a handler's own (geo:
/// `{bbox: [minx, miny, maxx, maxy]}`); anything undecidable gives `true`. A
/// null summary folded nothing. Equivalent to WASM's and Python's.
#[napi(js_name = "foldMayMatch")]
pub fn fold_may_match(
    datatype: String,
    #[napi(ts_arg_type = "any")] summary: Option<JsonArg>,
    #[napi(ts_arg_type = "any")] predicate: JsonArg,
    #[napi(ts_arg_type = "any")] config: Option<JsonArg>,
) -> bool {
    alizarin_core::datatype_fold::fold_may_match(
        &datatype,
        &summary.map_or(serde_json::Value::Null, |s| s.0),
        &predicate.0,
        config.map(|c| c.0).as_ref(),
        Some(&extension_registry()),
    )
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

/// The language a plain string is taken as (coercion, edits), as on WASM and
/// Python: core's, process-wide.
#[napi]
pub fn get_current_language() -> String {
    alizarin_core::type_coercion::get_current_language()
}

/// Set the language a plain string is taken as, as on WASM and Python.
#[napi]
pub fn set_current_language(language: String) {
    alizarin_core::type_coercion::set_current_language(&language);
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

    /// Build complete export data as JSON (without writing to filesystem):
    /// core's `{graphFiles, referenceDataFiles, businessDataFiles}`, each a
    /// list of `{relativePath, content}`, as WASM's `buildPrebuildExport`.
    #[napi]
    pub fn build_export_data(
        &self,
        graph_ids: Option<Vec<String>>,
        base_uri: String,
    ) -> Result<serde_json::Value> {
        let data = alizarin_core::build_prebuild_export(graph_ids.as_deref(), None, &base_uri)
            .map_err(exporter_err)?;
        serde_json::to_value(data).map_err(|e| napi::Error::from_reason(e.to_string()))
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

    pub(crate) fn from_inner(inner: StaticGraph) -> Self {
        NapiStaticGraph { inner }
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

    /// Alias matching the WASM `graphid` property for backend consistency.
    #[napi(getter, js_name = "graphid")]
    pub fn graphid(&self) -> String {
        self.inner.graphid.clone()
    }

    // WASM StaticGraph's read API. Changes go through `applyMutationsFromJson`
    // (TS: GraphMutator), as on every backend. Collections are converted
    // straight to JS (no serde_json::Value step) on each read; callers that
    // read them often cache them (the TS ResourceModelWrapper does).

    /// The graph's nodes, as on WASM.
    #[napi(getter, ts_return_type = "any[]")]
    pub fn nodes(&self, env: Env) -> Result<napi::JsUnknown> {
        json_arg::to_js(&env, &self.inner.nodes_slice())
    }

    #[napi(getter, ts_return_type = "any")]
    pub fn root(&self, env: Env) -> Result<napi::JsUnknown> {
        json_arg::to_js(&env, &self.inner.root)
    }

    #[napi(getter, ts_return_type = "any[]")]
    pub fn edges(&self, env: Env) -> Result<napi::JsUnknown> {
        json_arg::to_js(&env, &self.inner.edges_slice())
    }

    #[napi(getter, ts_return_type = "any[]")]
    pub fn nodegroups(&self, env: Env) -> Result<napi::JsUnknown> {
        json_arg::to_js(&env, &self.inner.nodegroups_slice())
    }

    #[napi(getter, ts_return_type = "any[] | null")]
    pub fn cards(&self, env: Env) -> Result<napi::JsUnknown> {
        json_arg::to_js(&env, &self.inner.cards)
    }

    #[napi(
        getter,
        js_name = "cards_x_nodes_x_widgets",
        ts_return_type = "any[] | null"
    )]
    pub fn cards_x_nodes_x_widgets(&self, env: Env) -> Result<napi::JsUnknown> {
        json_arg::to_js(&env, &self.inner.cards_x_nodes_x_widgets)
    }

    #[napi(
        getter,
        js_name = "functions_x_graphs",
        ts_return_type = "any[] | null"
    )]
    pub fn functions_x_graphs(&self, env: Env) -> Result<napi::JsUnknown> {
        json_arg::to_js(&env, &self.inner.functions_x_graphs)
    }

    #[napi(getter, ts_return_type = "any")]
    pub fn description(&self, env: Env) -> Result<napi::JsUnknown> {
        json_arg::to_js(&env, &self.inner.description)
    }

    #[napi(getter, ts_return_type = "any")]
    pub fn subtitle(&self, env: Env) -> Result<napi::JsUnknown> {
        json_arg::to_js(&env, &self.inner.subtitle)
    }

    #[napi(getter)]
    pub fn isresource(&self) -> Option<bool> {
        self.inner.isresource
    }

    #[napi(getter)]
    pub fn author(&self) -> Option<String> {
        self.inner.author.clone()
    }

    #[napi(getter)]
    pub fn iconclass(&self) -> Option<String> {
        self.inner.iconclass.clone()
    }

    #[napi(getter)]
    pub fn slug(&self) -> Option<String> {
        self.inner.slug.clone()
    }

    #[napi(getter, js_name = "template_id")]
    pub fn template_id(&self) -> Option<String> {
        self.inner.template_id.clone()
    }

    #[napi(getter)]
    pub fn version(&self) -> Option<String> {
        self.inner.version.clone()
    }

    /// The whole graph as plain data, so `JSON.stringify` works on it.
    #[napi(js_name = "toJSON")]
    pub fn to_json(&self) -> serde_json::Value {
        json_or_null(&self.inner)
    }

    #[napi]
    pub fn copy(&self) -> NapiStaticGraph {
        NapiStaticGraph {
            inner: self.inner.clone(),
        }
    }

    #[napi(getter)]
    pub fn name(&self) -> serde_json::Value {
        serde_json::to_value(&self.inner.name).unwrap_or(serde_json::Value::Null)
    }

    /// Graph-level config as a plain JS object, preserving arbitrary host keys —
    /// parity with the WASM `config` getter (C11).
    #[napi(getter)]
    pub fn config(&self) -> serde_json::Value {
        self.inner.config.clone()
    }

    /// Register this graph in the global registry so NapiResourceInstanceWrapper can use it.
    #[napi]
    pub fn register(&self) {
        alizarin_core::register_graph_owned(self.inner.clone());
    }

    /// Set a descriptor template for computing resource name/description/slug.
    ///
    /// Template placeholders use `<Node Name>` syntax, e.g. `"<Headword>"`.
    /// Descriptor types: "name", "description", "slug".
    #[napi(js_name = "setDescriptorTemplate")]
    pub fn set_descriptor_template(
        &mut self,
        descriptor_type: String,
        string_template: String,
    ) -> Result<()> {
        self.inner
            .set_descriptor_template(&descriptor_type, &string_template)
            .map_err(napi::Error::from_reason)
    }
}

// ============================================================================
// NapiStaticResourceRegistry
// ============================================================================

#[napi]
pub struct NapiStaticResourceRegistry {
    pub(crate) inner: StaticResourceRegistry,
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

    /// Merge full resources given as JS objects. Same semantics and defaults as the
    /// WASM `mergeFromResources`: summaries only unless `storeFull`, and related
    /// resources in `__cache` merged as summaries unless `includeCaches` is false.
    #[napi]
    pub fn merge_from_resources(
        &mut self,
        #[napi(ts_arg_type = "Array<any>")] resources: Vec<JsonArg>,
        store_full: Option<bool>,
        include_caches: Option<bool>,
    ) -> Result<()> {
        let resources = resources
            .into_iter()
            .map(|resource| serde_json::from_value(resource.0))
            .collect::<std::result::Result<Vec<StaticResource>, _>>()
            .map_err(|e| napi::Error::from_reason(format!("Invalid resource: {}", e)))?;
        self.inner.merge_from_resources(
            &resources,
            store_full.unwrap_or(false),
            include_caches.unwrap_or(true),
        );
        Ok(())
    }

    /// Add a single resource summary given as a JS object.
    #[napi]
    pub fn insert(&mut self, #[napi(ts_arg_type = "any")] summary: JsonArg) -> Result<()> {
        let summary: StaticResourceSummary = serde_json::from_value(summary.0)
            .map_err(|e| napi::Error::from_reason(format!("Invalid summary: {}", e)))?;
        self.inner.insert(summary);
        Ok(())
    }

    /// Load every business data file under a prebuild's `business_data/`
    /// (recursively, in path order), as `loadFromBusinessDataBytes` loads one,
    /// and return all their refs in that order. As Python's
    /// `load_prebuild_business_data`; WASM has no filesystem.
    #[napi]
    pub fn load_prebuild_business_data(
        &mut self,
        prebuild_dir: String,
        store_full: Option<bool>,
        include_caches: Option<bool>,
    ) -> Result<Vec<serde_json::Value>> {
        let refs = self
            .inner
            .load_prebuild_business_data(
                &prebuild_dir,
                store_full.unwrap_or(true),
                include_caches.unwrap_or(true),
            )
            .map_err(loader_err)?;
        refs.iter()
            .map(serde_json::to_value)
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| napi::Error::from_reason(e.to_string()))
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
        let refs = self
            .inner
            .load_from_business_data_bytes(
                &bytes,
                store_full.unwrap_or(true),
                include_caches.unwrap_or(true),
            )
            .map_err(|e| {
                napi::Error::from_reason(format!("Failed to parse business data: {}", e))
            })?;
        refs.iter()
            .map(serde_json::to_value)
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| napi::Error::from_reason(e.to_string()))
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
        rdm_cache: Option<&NapiRdmCache>,
    ) -> Result<HashMap<String, Vec<String>>> {
        let ext_registry = extension_registry();
        let ctx = SerializationContext {
            extension_registry: Some(&ext_registry),
            external_resolver: rdm_cache
                .map(|r| r.inner() as &dyn alizarin_core::type_serialization::ExternalResolver),
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
        let ext_registry = extension_registry();
        let ctx = SerializationContext {
            extension_registry: Some(&ext_registry),
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

    /// Run a resource's Aggregate functions, with this registry as the slice.
    ///
    /// The resource must be loaded in full here, and its graph and its members'
    /// graphs registered. Its Aggregate functions (e.g. the built-in fold) write
    /// node values into its tiles, which are updated in place; the resource itself
    /// is not part of the slice. Returns the node values that changed,
    /// `[{tileid, nodeid, value}]` (`[]` if nothing did), for a consumer caching
    /// them. Equivalent to Python's `ResourceRegistry.apply_aggregate_functions`.
    #[napi]
    pub fn apply_aggregate_functions(&mut self, target_id: String) -> Result<serde_json::Value> {
        let changed = alizarin_core::apply_aggregate_functions_in_registry(
            &mut self.inner,
            &target_id,
            &extension_registry(),
        )
        .map_err(napi::Error::from_reason)?;
        serde_json::to_value(changed).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Expand a resource's Aggregate node values with members added since the
    /// last call, without revisiting earlier ones (`addedIds` must already be
    /// loaded in full here). Call it when the node values are about to be
    /// consumed: additions compose, so the ids since the last call go in one
    /// batch. Add each member once; re-adding one counts it again. Errors if a
    /// provider can't add incrementally. Returns the node values that changed,
    /// as `applyAggregateFunctions` does. Equivalent to WASM's and Python's.
    #[napi]
    pub fn apply_aggregate_additions(
        &mut self,
        target_id: String,
        added_ids: Vec<String>,
    ) -> Result<serde_json::Value> {
        let changed = alizarin_core::apply_aggregate_additions_in_registry(
            &mut self.inner,
            &target_id,
            &added_ids,
            &extension_registry(),
        )
        .map_err(napi::Error::from_reason)?;
        serde_json::to_value(changed).map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Add `count` to the pending count each of a resource's Aggregate functions
    /// declares (`pending` in its entry): members added but not yet applied, so
    /// a consumer can tell its node values are stale, and by how much. Runs and
    /// additions reset it to 0. Returns the node values that changed.
    /// Equivalent to WASM's and Python's.
    #[napi]
    pub fn mark_aggregate_pending(
        &mut self,
        target_id: String,
        count: u32,
    ) -> Result<serde_json::Value> {
        let changed = alizarin_core::mark_aggregate_pending_in_registry(
            &mut self.inner,
            &target_id,
            count.into(),
        )
        .map_err(napi::Error::from_reason)?;
        serde_json::to_value(changed).map_err(|e| napi::Error::from_reason(e.to_string()))
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

    /// Get all full resources for a graph as JSON objects. Summary-only entries are
    /// skipped; a null, undefined or unknown graph id gives an empty array.
    #[napi]
    pub fn get_all_full_for_graph(
        &self,
        graph_id: Option<String>,
    ) -> Result<Vec<serde_json::Value>> {
        let Some(graph_id) = graph_id else {
            return Ok(Vec::new());
        };
        self.inner
            .iter_full()
            .filter(|(_, r)| r.resourceinstance.graph_id == graph_id)
            .map(|(_, r)| {
                serde_json::to_value(r)
                    .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e)))
            })
            .collect()
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

    /// Get diagnostic stats about registry contents (entry counts, tile counts, etc.)
    #[napi]
    pub fn memory_stats(&self) -> Result<serde_json::Value> {
        let stats = self.inner.memory_stats();
        serde_json::to_value(&stats)
            .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e)))
    }

    /// Get detailed stats including estimated byte sizes (expensive — re-serializes all data)
    #[napi]
    pub fn memory_stats_detailed(&self) -> Result<serde_json::Value> {
        let stats = self.inner.memory_stats_detailed();
        serde_json::to_value(&stats)
            .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e)))
    }

    /// Populate __cache on resources with summaries for referenced resources.
    ///
    /// Uses the graph to identify resource-instance nodes, then populates
    /// cache entries for each referenced resource found in this registry.
    ///
    /// If `enrich_relationships` is true, also adds ontologyProperty to tile data.
    /// If `recompute_descriptors` is true, recomputes resource name/description from
    /// descriptor templates set on the graph.
    ///
    /// Returns `{ resources, unknownReferences, hasUnknown }`.
    #[napi(js_name = "populateCachesFromJson")]
    pub fn populate_caches_from_json(
        &self,
        resources_json: String,
        graph: &NapiStaticGraph,
        enrich_relationships: Option<bool>,
        strict: Option<bool>,
        recompute_descriptors: Option<bool>,
    ) -> Result<serde_json::Value> {
        let output = self
            .inner
            .populate_caches_from_json(
                &resources_json,
                &graph.inner,
                enrich_relationships.unwrap_or(true),
                strict.unwrap_or(false),
                recompute_descriptors.unwrap_or(false),
            )
            .map_err(napi::Error::from_reason)?;
        serde_json::to_value(output).map_err(|e| napi::Error::from_reason(e.to_string()))
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

/// Apply a mutation request (core's `MutationRequest` JSON) to `graph`, giving
/// a new graph: core's, as WASM's `applyMutationsFromJson` and Python's
/// `apply_mutations_from_json`.
#[napi(js_name = "applyMutationsFromJson")]
pub fn apply_mutations_from_json(
    graph: &NapiStaticGraph,
    mutations_json: String,
) -> Result<NapiStaticGraph> {
    alizarin_core::graph_mutator::apply_mutations_from_json(&graph.inner, &mutations_json)
        .map(|inner| NapiStaticGraph { inner })
        .map_err(napi::Error::from_reason)
}

/// `value` as JSON, or null if it can't be (it always can for graph types).
fn json_or_null<T: serde::Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
}

/// Apply a mutation request that may start with `CreateGraph` (then `graph`
/// is omitted): core's, as WASM's and Python's `applyMutationsCreate`.
#[napi(js_name = "applyMutationsCreate")]
pub fn apply_mutations_create(
    mutations_json: String,
    graph: Option<&NapiStaticGraph>,
) -> Result<NapiStaticGraph> {
    alizarin_core::graph_mutator::apply_mutations_create_from_json(
        &mutations_json,
        graph.map(|g| &g.inner),
    )
    .map(|inner| NapiStaticGraph { inner })
    .map_err(napi::Error::from_reason)
}

/// The documented shape of each mutation type: core's, as on WASM and Python.
#[napi(js_name = "getMutationSchema")]
pub fn get_mutation_schema() -> serde_json::Value {
    alizarin_core::graph_mutator::get_mutation_schema()
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
    let built = build_graph_from_model_csvs(
        &graph_csv,
        &nodes_csv,
        collections_csv.as_deref(),
        &rdm_namespace,
        MutatorOptions::default(),
    )
    .map_err(csv_err)?;
    serde_json::to_value(alizarin_core::GraphFromCsvs::from(built))
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Build StaticResources from a business data CSV, given a graph JSON string
/// and collections JSON string (as returned by `buildGraphFromCsvs`).
///
/// Returns the resources wrapped in the `{ business_data: { resources: [...] } }`
/// format expected by PrebuildLoader.
///
/// `uuid_namespace` is a layer namespace (a UUID): given, multi-valued tile ids
/// are derived and reproducible, and each layer must use its own; omitted, they
/// are random.
#[napi]
pub fn build_business_data_from_csv(
    csv_data: String,
    graph_json: String,
    collections_json: String,
    default_language: Option<String>,
    strict_concepts: Option<bool>,
    uuid_namespace: Option<String>,
    rdm_cache: Option<&NapiRdmCache>,
) -> Result<serde_json::Value> {
    let graph: StaticGraph = serde_json::from_str(&graph_json)
        .map_err(|e| napi::Error::from_reason(format!("Invalid graph JSON: {e}")))?;
    let collections: Vec<SkosCollection> = serde_json::from_str(&collections_json)
        .map_err(|e| napi::Error::from_reason(format!("Invalid collections JSON: {e}")))?;

    let options = alizarin_core::csv_business_data_loader::BusinessDataCsvOptions {
        default_language: default_language.unwrap_or_else(|| "en".to_string()),
        strict_concepts: strict_concepts.unwrap_or(true),
        uuid_namespace,
    };

    let reg = extension_registry();
    let resources = if let Some(rc) = rdm_cache {
        let ctx = SerializationContext {
            concept_lookup: Some(rc.inner() as &dyn ConceptLookup),
            extension_registry: Some(&reg),
            ..SerializationContext::empty()
        };
        build_resources_from_business_csv_with_context(
            &csv_data,
            &graph,
            &collections,
            Some(&reg),
            options,
            Some(&ctx),
        )
    } else {
        build_resources_from_business_csv(&csv_data, &graph, &collections, options)
    }
    .map_err(csv_err)?;

    Ok(wrap_business_data(&resources))
}

/// Compose one resource across an ordered layer stack, in memory — the binding
/// form of the substrate's `hydrate_layers` (no DuckDB). The same resource as it
/// exists in each layer is supplied **topmost-first** in `resources_json`; tiles
/// are merged (identical tiles deduped, topmost wins) and cardinality-1 nodegroups
/// unified with PerNodegroup (topmost overrides a single-valued group whole;
/// multi-valued groups accumulate).
///
/// `overlay_graphs_json` is a JSON array of overlay `StaticGraph`s bottom-to-top
/// (may be `[]`); composition runs against the merged model via `LayeredGraph`.
/// `strict` makes a cross-layer conflict on a single-valued group an error.
/// Returns `{ resource, warnings }`.
#[napi]
pub fn compose_resource_layers(
    resources_json: String,
    base_graph_json: String,
    overlay_graphs_json: String,
    strict: Option<bool>,
) -> Result<serde_json::Value> {
    let composed = alizarin_core::graph::compose_resource_layers_from_json(
        &resources_json,
        &base_graph_json,
        &overlay_graphs_json,
        strict.unwrap_or(false),
    )
    .map_err(napi::Error::from_reason)?;

    serde_json::to_value(&composed)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize result: {e}")))
}

/// A resource's lifecycle state resolved against its graph's lifecycle
/// (`lifecycle_json` is the graph's `resource_instance_lifecycle`, `"null"` for
/// none). Returns `{ status, state_id, is_initial, conflict }`.
#[napi]
pub fn resource_lifecycle_state(
    resource_json: String,
    lifecycle_json: String,
) -> Result<serde_json::Value> {
    let report = alizarin_core::graph::lifecycle_state_from_json(&resource_json, &lifecycle_json)
        .map_err(napi::Error::from_reason)?;
    serde_json::to_value(&report)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize result: {e}")))
}

/// Move a resource to lifecycle state `target`, returning `{ resource, event }`.
/// `strict` defaults to `true` (permitted edges only); pass `false` explicitly
/// for Arches' membership-only check.
#[napi]
pub fn transition_resource_lifecycle(
    resource_json: String,
    lifecycle_json: String,
    target: String,
    strict: Option<bool>,
) -> Result<serde_json::Value> {
    let result = alizarin_core::graph::transition_from_json(
        &resource_json,
        &lifecycle_json,
        &target,
        strict.unwrap_or(true),
    )
    .map_err(napi::Error::from_reason)?;
    serde_json::to_value(&result)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize result: {e}")))
}

// ============================================================================
// Extension handler direct access
// ============================================================================

/// Coerce a value using the registered extension handler for the given datatype.
///
/// Returns `{ tileData, displayValue, warnings }` (core's `ExtensionCoercion`,
/// as Python's `coerce_with_extension`), or null if no handler can coerce.
#[napi(js_name = "extensionCoerce")]
pub fn extension_coerce(
    datatype: String,
    #[napi(ts_arg_type = "any")] value: JsonArg,
    #[napi(ts_arg_type = "any")] config: Option<JsonArg>,
) -> Result<Option<serde_json::Value>> {
    extension_registry()
        .coerce_output(&datatype, &value.0, config.map(|c| c.0).as_ref())
        .map_err(|e| napi::Error::from_reason(e.message))?
        .map(serde_json::to_value)
        .transpose()
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Render a display string for tile data using the extension handler.
///
/// Returns the display string, or null if no handler or no display.
#[napi(js_name = "extensionRenderDisplay")]
pub fn extension_render_display(
    datatype: String,
    #[napi(ts_arg_type = "any")] tile_data: JsonArg,
    language: String,
) -> Result<Option<String>> {
    let registry = extension_registry();
    registry
        .render_display(&datatype, &tile_data.0, &language, None)
        .map_err(|e| napi::Error::from_reason(e.message))
}

/// Resolve markers in tile data using the extension handler.
///
/// Returns the resolved tile data value.
#[napi(js_name = "extensionResolveMarkers")]
pub fn extension_resolve_markers(
    datatype: String,
    #[napi(ts_arg_type = "any")] tile_data: JsonArg,
    language: String,
) -> Result<serde_json::Value> {
    let registry = extension_registry();
    registry
        .resolve_markers(&datatype, &tile_data.0, &language)
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
// SKOS parsing
// ============================================================================

/// Parse SKOS RDF/XML and return all collections as a JSON array.
#[napi(js_name = "parseSkosXml")]
pub fn parse_skos_xml(xml_content: String, base_uri: String) -> Result<serde_json::Value> {
    let collections = alizarin_core::skos::parse_skos_to_collections(&xml_content, &base_uri)
        .map_err(napi::Error::from_reason)?;
    serde_json::to_value(&collections).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Parse SKOS RDF/XML and return a single collection (the first one found).
#[napi(js_name = "parseSkosXmlToCollection")]
pub fn parse_skos_xml_to_collection(
    xml_content: String,
    base_uri: String,
) -> Result<serde_json::Value> {
    let mut collections = alizarin_core::skos::parse_skos_to_collections(&xml_content, &base_uri)
        .map_err(napi::Error::from_reason)?;
    if collections.is_empty() {
        return Err(napi::Error::from_reason(
            "No SKOS ConceptScheme found in XML".to_string(),
        ));
    }
    let collection = collections.remove(0);
    serde_json::to_value(&collection).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Serialize a single SkosCollection to SKOS RDF/XML.
#[napi(js_name = "collectionToSkosXml")]
pub fn collection_to_skos_xml_napi(
    #[napi(ts_arg_type = "any")] collection_json: JsonArg,
    base_uri: String,
) -> Result<String> {
    let collection: SkosCollection = serde_json::from_value(collection_json.0).map_err(|e| {
        napi::Error::from_reason(format!("Failed to deserialize collection: {}", e))
    })?;
    Ok(alizarin_core::skos::collection_to_skos_xml(
        &collection,
        &base_uri,
    ))
}

/// Serialize multiple SkosCollections to SKOS RDF/XML.
#[napi(js_name = "collectionsToSkosXml")]
pub fn collections_to_skos_xml_napi(
    #[napi(ts_arg_type = "any")] collections_json: JsonArg,
    base_uri: String,
) -> Result<String> {
    let collections: Vec<SkosCollection> =
        serde_json::from_value(collections_json.0).map_err(|e| {
            napi::Error::from_reason(format!("Failed to deserialize collections: {}", e))
        })?;
    Ok(alizarin_core::skos::collections_to_skos_xml(
        &collections,
        &base_uri,
    ))
}

// ============================================================================
// Aggregate functions
// ============================================================================

/// Run a resource's Aggregate functions over a business-data file, streamed one
/// resource at a time (so a large layer is never held in memory).
///
/// The resource's graph and its members' graphs must be registered. The resource
/// itself is left out of the slice if the file contains it. Returns the resource
/// with its Aggregate node values written. Equivalent to Python's
/// `apply_aggregate_functions_from_file`.
#[napi(js_name = "applyAggregateFunctionsFromFile")]
pub fn apply_aggregate_functions_from_file(
    resource_json: String,
    business_data_path: String,
) -> Result<serde_json::Value> {
    let mut resource: StaticResource = serde_json::from_str(&resource_json)
        .map_err(|e| napi::Error::from_reason(format!("invalid resource: {e}")))?;
    alizarin_core::apply_aggregate_functions_to_resource(
        &mut resource,
        &alizarin_core::BusinessDataFileSlice::new(business_data_path),
        &extension_registry(),
    )
    .map_err(napi::Error::from_reason)?;
    serde_json::to_value(&resource).map_err(|e| napi::Error::from_reason(e.to_string()))
}

// ============================================================================
// Label resolution utilities
// ============================================================================

/// Build a mapping from node alias to collection ID based on graph definition.
///
/// Equivalent to WASM's `buildAliasToCollectionMap`.
#[napi(js_name = "buildAliasToCollectionMap")]
pub fn build_alias_to_collection_map(
    graph_json: String,
    resolvable_datatypes: Option<Vec<String>>,
    config_keys: Option<Vec<String>>,
) -> Result<HashMap<String, String>> {
    let graph: serde_json::Value = serde_json::from_str(&graph_json)
        .map_err(|e| napi::Error::from_reason(format!("Invalid graph JSON: {e}")))?;

    let config = alizarin_core::LabelResolutionConfig {
        resolvable_datatypes: resolvable_datatypes.unwrap_or_else(|| {
            alizarin_core::DEFAULT_RESOLVABLE_DATATYPES
                .iter()
                .map(|s| s.to_string())
                .collect()
        }),
        config_keys: config_keys.unwrap_or_else(|| {
            alizarin_core::DEFAULT_CONFIG_KEYS
                .iter()
                .map(|s| s.to_string())
                .collect()
        }),
        strict: false,
    };

    let registry = alizarin_core::build_extension_registry_from_registered();
    Ok(alizarin_core::build_alias_to_collection_map(
        &graph,
        &config,
        Some(&registry),
    ))
}

/// Scan a JSON tree to find which collections are needed for resolution.
///
/// Equivalent to WASM's `findNeededCollections`.
#[napi(js_name = "findNeededCollections")]
pub fn find_needed_collections(
    tree_json: String,
    alias_to_collection: HashMap<String, String>,
) -> Result<Vec<String>> {
    let tree: serde_json::Value = serde_json::from_str(&tree_json)
        .map_err(|e| napi::Error::from_reason(format!("Invalid tree JSON: {e}")))?;

    let needed = alizarin_core::find_needed_collections(&tree, &alias_to_collection);
    Ok(needed.into_iter().collect())
}

/// Metadata header for a resource summary given as a JS object. The native
/// registry hands back plain objects, so this stands in for the WASM
/// `StaticResourceSummary.toMetadata()`.
#[napi]
pub fn summary_to_metadata(
    #[napi(ts_arg_type = "any")] summary: JsonArg,
) -> Result<serde_json::Value> {
    let summary: StaticResourceSummary = serde_json::from_value(summary.0)
        .map_err(|e| napi::Error::from_reason(format!("Invalid summary: {}", e)))?;
    serde_json::to_value(summary.to_metadata())
        .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e)))
}

/// A static type (`"node"`, `"resource"`, ...) built from a JS object, as
/// core builds it: what the WASM classes' constructors hold.
#[napi]
pub fn construct_static(
    kind: String,
    #[napi(ts_arg_type = "any")] value: JsonArg,
) -> Result<JsonOut> {
    alizarin_core::construct_static(&kind, value.0)
        .map(JsonOut)
        .map_err(napi::Error::from_reason)
}

/// One alias-keyed tree to a resource, as `{business_data: {resources: [...]}}`:
/// core's `tree_to_tiles_from_json`, as WASM's `treeToTiles` (same arguments)
/// and Python's `json_tree_to_tiles`.
#[napi]
pub fn tree_to_tiles(
    tree_json: String,
    graph: &NapiStaticGraph,
    from_camel: bool,
    strict: bool,
    id_key: Option<String>,
    random_ids: Option<bool>,
    tile_namespace: Option<String>,
) -> Result<JsonOut> {
    let opts = alizarin_core::TreeToTilesOptions {
        id_key: id_key.as_deref(),
        from_camel,
        strict,
        random_ids: random_ids.unwrap_or(true),
        tile_namespace: tile_namespace.as_deref(),
        ..Default::default()
    };
    let ext = extension_registry();
    let wrapper =
        alizarin_core::tree_to_tiles_from_json(&tree_json, &graph.inner, &opts, Some(&ext), None)
            .map_err(napi::Error::from_reason)?;
    serde_json::to_value(wrapper)
        .map(JsonOut)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Many alias-keyed trees to resources: core's `batch_trees_to_tiles_from_json`,
/// as WASM's `batchTreesToTiles` (same arguments) and Python's
/// `batch_trees_to_tiles`. Returns `{business_data, errors, error_count,
/// warnings, warning_count}`.
#[napi]
#[allow(clippy::too_many_arguments)]
pub fn batch_trees_to_tiles(
    trees_json: String,
    graph: &NapiStaticGraph,
    from_camel: bool,
    strict: bool,
    id_keys_json: Option<String>,
    random_ids: Option<bool>,
    resolve_markers: Option<bool>,
    scopes_json: Option<String>,
    tile_namespace: Option<String>,
) -> Result<JsonOut> {
    let parse = |s: Option<String>, what: &str| -> Result<Option<serde_json::Value>> {
        s.map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| napi::Error::from_reason(format!("Failed to parse {what}: {e}")))
    };
    let id_keys: Option<Vec<String>> = parse(id_keys_json, "id_keys")?
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| napi::Error::from_reason(format!("Failed to parse id_keys: {e}")))?;
    let scopes = parse(scopes_json, "scopes")?;
    let opts = alizarin_core::BatchOptions {
        graph_id: &graph.inner.graphid,
        from_camel,
        strict,
        random_ids: random_ids.unwrap_or(false),
        resolve_markers: resolve_markers.unwrap_or(false),
        id_keys: id_keys.as_deref(),
        scopes: scopes.as_ref(),
        tile_namespace: tile_namespace.as_deref(),
    };
    let ext = extension_registry();
    let output = alizarin_core::batch_trees_to_tiles_from_json(
        &trees_json,
        &graph.inner,
        &opts,
        Some(&ext),
        None,
    )
    .map_err(napi::Error::from_reason)?;
    serde_json::to_value(output)
        .map(JsonOut)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Resources with one resourceinstanceid merged into one, as `{resource,
/// warnings}`: core's `merge_resources_from_json`, as WASM's `mergeResources`.
#[napi]
pub fn merge_resources(resources_json: String) -> Result<JsonOut> {
    let result = alizarin_core::merge_resources_from_json(&resources_json)
        .map_err(napi::Error::from_reason)?;
    serde_json::to_value(result)
        .map(JsonOut)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Batches of resources (each an array, one resource, or a business data
/// file, as JSON strings, in a JSON array) merged per resourceinstanceid, as
/// `{resources, warnings}`: core's `batch_merge_resources_from_json`, as WASM's
/// `batchMergeResources` (same arguments).
#[napi]
pub fn batch_merge_resources(
    batches_json: String,
    recompute_descriptors: Option<bool>,
    strict: Option<bool>,
) -> Result<JsonOut> {
    let batches: Vec<String> = serde_json::from_str(&batches_json)
        .map_err(|e| napi::Error::from_reason(format!("Failed to parse batches array: {e}")))?;
    let result = alizarin_core::batch_merge_resources_from_json(
        &batches,
        recompute_descriptors.unwrap_or(false),
        strict.unwrap_or(true),
    )
    .map_err(napi::Error::from_reason)?;
    serde_json::to_value(result)
        .map(JsonOut)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// How the ORM names a field: core's `describe_field` (`"<model> - <node
/// name>"`, a list's in brackets), as WASM and Python give it.
#[napi]
pub fn describe_field(model_name: Option<String>, node_name: String, list: bool) -> String {
    alizarin_core::describe_field(model_name.as_deref(), &node_name, list)
}

/// How the ORM names a field's group: core's `describe_field_group`.
#[napi]
pub fn describe_field_group(
    model_name: Option<String>,
    node_name: String,
    nodegroup_name: Option<String>,
    list: bool,
) -> String {
    alizarin_core::describe_field_group(
        model_name.as_deref(),
        &node_name,
        nodegroup_name.as_deref(),
        list,
    )
}

/// Whether two nodes (whole or partial objects) are the same apart from their
/// ids: core's `compare_nodes`, as WASM's `StaticNode.compare`. One of
/// `"same"`, `"same_ids_partial"`, `"nodeid_conflict"`,
/// `"nodegroup_id_conflict"`, `"graph_id_conflict"`, `"different"`.
#[napi]
pub fn compare_nodes(
    #[napi(ts_arg_type = "any")] a: JsonArg,
    #[napi(ts_arg_type = "any")] b: JsonArg,
) -> String {
    alizarin_core::compare_nodes(&a.0, &b.0)
        .as_str()
        .to_string()
}

/// The resources of a business data JSON string, as core builds them: the
/// WASM `StaticResource.fromBusinessDataJsonString`.
#[napi]
pub fn resources_from_business_data_json(json: String) -> Result<JsonOut> {
    let resources = alizarin_core::resources_from_business_data_json(&json)
        .map_err(napi::Error::from_reason)?;
    drop(json);
    serde_json::to_value(resources)
        .map(JsonOut)
        .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e)))
}

/// A lightweight resource (no tiles, `tiles_loaded: false`) from a registry
/// summary, as core builds it: the WASM `StaticResource.fromSummary`.
#[napi]
pub fn static_resource_from_summary(
    #[napi(ts_arg_type = "any")] summary: JsonArg,
) -> Result<serde_json::Value> {
    let summary: StaticResourceSummary = serde_json::from_value(summary.0)
        .map_err(|e| napi::Error::from_reason(format!("Invalid summary: {}", e)))?;
    serde_json::to_value(alizarin_core::StaticResource::from_summary(&summary))
        .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e)))
}

/// One resource as a node tree, against the registered graph it names.
///
/// `resourceJson` is a StaticResource, or flat `{resourceinstanceid,
/// graph_id, tiles}` (see core's `resource_for_tree`).
#[napi(js_name = "tilesToTree")]
pub fn tiles_to_tree(resource_json: String) -> Result<serde_json::Value> {
    alizarin_core::tiles_to_tree_from_json_registered(&resource_json)
        .map_err(napi::Error::from_reason)
}

/// One resource as a card tree (`{resourceinstance, cards}`), against the
/// registered graph it names.
#[napi(js_name = "cardsToTree")]
pub fn cards_to_tree(resource_json: String) -> Result<serde_json::Value> {
    alizarin_core::cards_to_tree_from_json_registered(&resource_json)
        .map_err(napi::Error::from_reason)
}

/// Each resource of a JSON array as a node tree, against the registered graph
/// each names: `{success, results, count, errors, error_count}`. Strict, the
/// first failure throws.
#[napi(js_name = "batchTilesToTrees")]
pub fn batch_tiles_to_trees(resources_json: String, strict: bool) -> Result<serde_json::Value> {
    let outcome = alizarin_core::batch_tiles_to_trees_from_json(&resources_json, None, strict)
        .map_err(napi::Error::from_reason)?;
    serde_json::to_value(outcome)
        .map_err(|e| napi::Error::from_reason(format!("Serialization failed: {}", e)))
}

/// Set the global RDM namespace (UUID or http(s) URL) for deterministic UUID
/// generation; the string itself is kept as the RDF base URI.
#[napi(js_name = "setRdmNamespace")]
pub fn set_rdm_namespace(namespace: String) -> napi::Result<()> {
    alizarin_core::set_rdm_namespace(&namespace).map_err(napi::Error::from_reason)
}

/// The global RDM namespace's UUID, or null if not set.
#[napi(js_name = "getRdmNamespace")]
pub fn get_rdm_namespace() -> Option<String> {
    alizarin_core::get_rdm_namespace().map(|u| u.to_string())
}

/// The string the global RDM namespace was set from, or null if not set.
#[napi(js_name = "getRdmNamespaceRaw")]
pub fn get_rdm_namespace_raw() -> Option<String> {
    alizarin_core::get_rdm_namespace_raw()
}

/// Whether a global RDM namespace is set.
#[napi(js_name = "hasRdmNamespace")]
pub fn has_rdm_namespace() -> bool {
    alizarin_core::has_rdm_namespace()
}

/// Clear the global RDM namespace.
#[napi(js_name = "clearRdmNamespace")]
pub fn clear_rdm_namespace() {
    alizarin_core::clear_rdm_namespace();
}

/// Check if a string is a valid UUID.
#[napi(js_name = "isValidUuid")]
pub fn is_valid_uuid(s: String) -> bool {
    alizarin_core::is_valid_uuid(&s)
}

/// Get the default resolvable datatypes (concept, concept-list).
#[napi(js_name = "getDefaultResolvableDatatypes")]
pub fn get_default_resolvable_datatypes() -> Vec<String> {
    alizarin_core::DEFAULT_RESOLVABLE_DATATYPES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Get the default config keys for collection IDs.
#[napi(js_name = "getDefaultConfigKeys")]
pub fn get_default_config_keys() -> Vec<String> {
    alizarin_core::DEFAULT_CONFIG_KEYS
        .iter()
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
/// Returns `{ graphIds, collectionIds, collections, ontologyConfigs }` (core's
/// `ImportPrebuildSummary`, as Python's `import_prebuild`). Resources are not
/// loaded: use a registry's `loadPrebuildBusinessData`.
#[napi(js_name = "importPrebuild")]
pub fn import_prebuild(
    prebuild_dir: String,
    base_uri: String,
    extra_reference_data_dirs: Option<Vec<String>>,
    extra_ontology_dirs: Option<Vec<String>>,
) -> Result<serde_json::Value> {
    fn strs(dirs: &Option<Vec<String>>) -> Option<Vec<&str>> {
        dirs.as_ref()
            .map(|v| v.iter().map(String::as_str).collect())
    }
    let (reference, ontology) = (strs(&extra_reference_data_dirs), strs(&extra_ontology_dirs));
    let result = alizarin_core::import_prebuild(
        &prebuild_dir,
        &base_uri,
        reference.as_deref(),
        ontology.as_deref(),
    )
    .map_err(loader_err)?;
    serde_json::to_value(result.summary()).map_err(|e| napi::Error::from_reason(e.to_string()))
}
