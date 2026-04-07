//! WASM bindings for the JSON-based Graph Mutation API.
//!
//! This module exposes a simple JSON-based API for mutating graphs,
//! including support for extension mutations via JS callbacks.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

use alizarin_core::graph_mutator::{
    apply_mutations_create_from_json as core_apply_mutations_create,
    apply_mutations_from_json as core_apply_mutations,
    apply_mutations_from_json_with_extensions as core_apply_mutations_with_ext,
    generate_uuid_v5 as core_generate_uuid, get_mutation_schema as core_get_schema,
    ExtensionMutationHandler, ExtensionMutationRegistry, MutationConformance, MutationError,
    MutatorOptions,
};
use alizarin_core::StaticGraph as CoreStaticGraph;

use crate::graph::StaticGraph;

// Thread-local registry of JS mutation handler callbacks
// WASM is single-threaded, so thread_local is safe
thread_local! {
    static JS_MUTATION_HANDLERS: RefCell<HashMap<String, JsMutationCallbacks>> =
        RefCell::new(HashMap::new());
}

/// JS callbacks for a mutation handler
struct JsMutationCallbacks {
    apply_fn: js_sys::Function,
    conformance: MutationConformance,
}

/// A mutation handler that delegates to a JS callback.
struct JsMutationHandler {
    name: String,
    conformance: MutationConformance,
}

impl JsMutationHandler {
    fn new(name: String, conformance: MutationConformance) -> Self {
        Self { name, conformance }
    }
}

// WASM is single-threaded, so Send+Sync is safe
unsafe impl Send for JsMutationHandler {}
unsafe impl Sync for JsMutationHandler {}

impl ExtensionMutationHandler for JsMutationHandler {
    fn apply(
        &self,
        graph: &mut CoreStaticGraph,
        params: &serde_json::Value,
        _options: &MutatorOptions,
    ) -> Result<(), MutationError> {
        JS_MUTATION_HANDLERS.with(|handlers| {
            let handlers = handlers.borrow();

            if let Some(cbs) = handlers.get(&self.name) {
                // Serialize graph and params to JSON for JS
                let graph_json = serde_json::to_string(graph).map_err(|e| {
                    MutationError::Other(format!("Failed to serialize graph: {}", e))
                })?;
                let params_json = serde_json::to_string(params).map_err(|e| {
                    MutationError::Other(format!("Failed to serialize params: {}", e))
                })?;

                // Convert to JsValue
                let js_graph = js_sys::JSON::parse(&graph_json)
                    .map_err(|_| MutationError::Other("Failed to parse graph JSON".to_string()))?;
                let js_params = js_sys::JSON::parse(&params_json)
                    .map_err(|_| MutationError::Other("Failed to parse params JSON".to_string()))?;

                // Call JS function
                let result = cbs
                    .apply_fn
                    .call2(&JsValue::NULL, &js_graph, &js_params)
                    .map_err(|e| {
                        MutationError::Other(
                            e.as_string()
                                .unwrap_or_else(|| "JS handler error".to_string()),
                        )
                    })?;

                // Parse result back as mutated graph
                let result_json = js_sys::JSON::stringify(&result)
                    .map_err(|_| MutationError::Other("Failed to stringify result".to_string()))?
                    .as_string()
                    .ok_or_else(|| MutationError::Other("Result is not a string".to_string()))?;

                let mutated: CoreStaticGraph = serde_json::from_str(&result_json).map_err(|e| {
                    MutationError::Other(format!("Invalid graph JSON returned: {}", e))
                })?;

                // Replace graph contents
                *graph = mutated;
                Ok(())
            } else {
                Err(MutationError::Other(format!(
                    "No handler registered for: {}",
                    self.name
                )))
            }
        })
    }

    fn conformance(&self) -> MutationConformance {
        self.conformance
    }

    fn description(&self) -> &str {
        "JS extension mutation handler"
    }
}

// =============================================================================
// Basic Mutation API
// =============================================================================

/// Apply mutations to a graph from a JSON string.
///
/// # Arguments
/// * `graph` - The source graph (will be cloned, not modified)
/// * `mutations_json` - JSON string containing a MutationRequest
///
/// # Returns
/// A new StaticGraph with mutations applied
///
/// # Example JSON
/// ```json
/// {
///   "mutations": [
///     {
///       "AddNode": {
///         "parent_alias": "root",
///         "alias": "child",
///         "name": "Child Node",
///         "cardinality": "N",
///         "datatype": "string",
///         "ontology_class": "E41_Appellation",
///         "parent_property": "P1_is_identified_by",
///         "options": {}
///       }
///     }
///   ],
///   "options": {
///     "autocreate_card": true,
///     "autocreate_widget": true
///   }
/// }
/// ```
#[wasm_bindgen(js_name = applyMutationsFromJson)]
pub fn apply_mutations_from_json(
    graph: &StaticGraph,
    mutations_json: &str,
) -> Result<StaticGraph, JsValue> {
    let core_graph: &CoreStaticGraph = graph;
    core_apply_mutations(core_graph, mutations_json)
        .map(StaticGraph::from)
        .map_err(|e| JsValue::from_str(&e))
}

/// Generate a deterministic UUID v5 from group and key.
///
/// This matches the behavior of the Rust UUID generation for cross-platform
/// consistency.
///
/// # Arguments
/// * `group_type` - The type of group (e.g., "graph", "node")
/// * `group_id` - Optional ID for the group
/// * `key` - The key to generate a UUID for
///
/// # Returns
/// A UUID string
#[wasm_bindgen(js_name = generateUuidV5)]
pub fn generate_uuid_v5(group_type: &str, group_id: Option<String>, key: &str) -> String {
    core_generate_uuid((group_type, group_id.as_deref()), key)
}

/// Get the JSON schema for mutation types.
///
/// Returns a documentation object describing the structure of each mutation type.
/// Useful for building UI forms or validating mutation payloads.
#[wasm_bindgen(js_name = getMutationSchema)]
pub fn get_mutation_schema() -> JsValue {
    let schema = core_get_schema();
    serde_wasm_bindgen::to_value(&schema).unwrap_or(JsValue::NULL)
}

// =============================================================================
// Extension Mutation API
// =============================================================================

/// Register an extension mutation handler from JS.
///
/// The handler function should have the signature:
///     function handler(graph: object, params: object): object
///
/// It receives the current graph and mutation params, and should return
/// the modified graph.
///
/// # Arguments
/// * `name` - The mutation name (e.g., "clm.reference_change_collection")
/// * `handler` - JS function to handle the mutation
/// * `conformance` - Conformance level: "AlwaysConformant", "BranchConformant",
///                   "ModelConformant", or "NonConformant"
#[wasm_bindgen(js_name = registerExtensionMutation)]
pub fn register_extension_mutation(
    name: &str,
    handler: js_sys::Function,
    conformance: Option<String>,
) -> Result<(), JsValue> {
    let conformance_level = match conformance.as_deref().unwrap_or("AlwaysConformant") {
        "AlwaysConformant" => MutationConformance::AlwaysConformant,
        "BranchConformant" => MutationConformance::BranchConformant,
        "ModelConformant" => MutationConformance::ModelConformant,
        "NonConformant" => MutationConformance::NonConformant,
        other => {
            return Err(JsValue::from_str(&format!(
                "Invalid conformance level: {}",
                other
            )))
        }
    };

    JS_MUTATION_HANDLERS.with(|handlers| {
        handlers.borrow_mut().insert(
            name.to_string(),
            JsMutationCallbacks {
                apply_fn: handler,
                conformance: conformance_level,
            },
        );
    });

    Ok(())
}

/// Check if an extension mutation handler is registered.
#[wasm_bindgen(js_name = hasExtensionMutation)]
pub fn has_extension_mutation(name: &str) -> bool {
    JS_MUTATION_HANDLERS.with(|handlers| handlers.borrow().contains_key(name))
}

/// Unregister an extension mutation handler.
#[wasm_bindgen(js_name = unregisterExtensionMutation)]
pub fn unregister_extension_mutation(name: &str) {
    JS_MUTATION_HANDLERS.with(|handlers| {
        handlers.borrow_mut().remove(name);
    });
}

/// Get the list of registered extension mutation names.
#[wasm_bindgen(js_name = getRegisteredExtensionMutations)]
pub fn get_registered_extension_mutations() -> js_sys::Array {
    JS_MUTATION_HANDLERS.with(|handlers| {
        let handlers = handlers.borrow();
        let arr = js_sys::Array::new();
        for name in handlers.keys() {
            arr.push(&JsValue::from_str(name));
        }
        arr
    })
}

/// Build an ExtensionMutationRegistry from all registered JS handlers.
fn build_mutation_registry() -> ExtensionMutationRegistry {
    let mut registry = ExtensionMutationRegistry::new();

    JS_MUTATION_HANDLERS.with(|handlers| {
        let handlers = handlers.borrow();
        for (name, cbs) in handlers.iter() {
            registry.register(
                name.clone(),
                Arc::new(JsMutationHandler::new(name.clone(), cbs.conformance)),
            );
        }
    });

    registry
}

/// Apply mutations with extension support.
///
/// This version uses the global extension mutation registry to handle
/// Extension mutations. Register handlers with `registerExtensionMutation`.
///
/// # Arguments
/// * `graph` - The source graph
/// * `mutations_json` - JSON string containing a MutationRequest
///
/// # Returns
/// A new StaticGraph with mutations applied
#[wasm_bindgen(js_name = applyMutationsWithExtensions)]
pub fn apply_mutations_with_extensions(
    graph: &StaticGraph,
    mutations_json: &str,
) -> Result<StaticGraph, JsValue> {
    let core_graph: &CoreStaticGraph = graph;
    let registry = build_mutation_registry();

    core_apply_mutations_with_ext(core_graph, mutations_json, Some(&registry))
        .map(StaticGraph::from)
        .map_err(|e| JsValue::from_str(&e))
}

/// Apply mutations that may create a new graph.
///
/// If `graph` is null/undefined, the first mutation must be CreateGraph which
/// creates a new graph from scratch. Remaining mutations are applied to the new graph.
///
/// If `graph` is provided, the first mutation must NOT be CreateGraph,
/// and all mutations are applied to the existing graph.
///
/// # Arguments
/// * `mutations_json` - JSON string containing a MutationRequest
/// * `graph` - Optional existing graph
///
/// # Returns
/// The resulting graph
#[wasm_bindgen(js_name = applyMutationsCreate)]
pub fn apply_mutations_create(
    mutations_json: &str,
    graph: Option<StaticGraph>,
) -> Result<StaticGraph, JsValue> {
    let existing_graph = graph.map(|g| {
        let core: &CoreStaticGraph = &g;
        core.clone()
    });

    core_apply_mutations_create(mutations_json, existing_graph.as_ref())
        .map(StaticGraph::from)
        .map_err(|e| JsValue::from_str(&e))
}

/// Build a graph and collections from the 3-CSV model format.
///
/// Parses graph.csv, nodes.csv, and collections.csv, validates them,
/// and builds the graph via the standard mutation pipeline.
///
/// # Arguments
/// * `graph_csv` - Contents of graph.csv
/// * `nodes_csv` - Contents of nodes.csv
/// * `collections_csv` - Contents of collections.csv (optional, pass null/undefined if none)
/// * `rdm_namespace` - RDM namespace string (UUID or URL) for deterministic ID generation
///
/// # Returns
/// JSON string containing `{ "graph": <graph>, "collections": [<collection>, ...], "diagnostics": [...] }`
#[wasm_bindgen(js_name = buildGraphFromModelCsvs)]
pub fn build_graph_from_model_csvs(
    graph_csv: &str,
    nodes_csv: &str,
    collections_csv: Option<String>,
    rdm_namespace: &str,
) -> Result<JsValue, JsValue> {
    use alizarin_core::csv_model_loader;

    let result = csv_model_loader::build_graph_from_model_csvs(
        graph_csv,
        nodes_csv,
        collections_csv.as_deref(),
        rdm_namespace,
        MutatorOptions::default(),
    );

    match result {
        Ok((graph, collections)) => {
            let json = serde_json::json!({
                "graph": graph,
                "collections": collections,
            });
            // Use JSON.parse rather than serde_wasm_bindgen::to_value to ensure
            // plain JS objects (serde_wasm_bindgen produces Map objects for
            // serde_json::Value).
            let json_str = serde_json::to_string(&json)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
            js_sys::JSON::parse(&json_str)
                .map_err(|_| JsValue::from_str("Failed to parse serialized JSON"))
        }
        Err(e) => Err(JsValue::from_str(&format!("{}", e))),
    }
}

/// Validate 3-CSV model files without building.
///
/// Returns diagnostics as a JSON array of `{ level, file, line, message }`.
#[wasm_bindgen(js_name = validateModelCsvs)]
pub fn validate_model_csvs(
    graph_csv: &str,
    nodes_csv: &str,
    collections_csv: Option<String>,
) -> Result<JsValue, JsValue> {
    use alizarin_core::csv_model_loader;

    let diagnostics = csv_model_loader::validate_model_csvs_from_strings(
        graph_csv,
        nodes_csv,
        collections_csv.as_deref(),
    );

    serde_wasm_bindgen::to_value(&diagnostics)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Build resource instances from a business data CSV.
///
/// Resolves node aliases to UUIDs from the graph, concept labels to UUIDs
/// from the collections. Generates deterministic UUIDs for resources and tiles.
///
/// # Arguments
/// * `csv_data` - The CSV string with ResourceID as first column, node aliases as remaining columns
/// * `graph_json` - JSON string of the built StaticGraph (from buildGraphFromModelCsvs)
/// * `collections_json` - JSON string of the built collections array (from buildGraphFromModelCsvs)
/// * `default_language` - Default language code (optional, defaults to "en")
/// * `strict_concepts` - Whether to error on unresolved concept labels (optional, defaults to true)
///
/// # Returns
/// JSON object: `{ "business_data": { "resources": [...] } }`
#[wasm_bindgen(js_name = buildResourcesFromBusinessCsv)]
pub fn build_resources_from_business_csv(
    csv_data: &str,
    graph_json: &str,
    collections_json: &str,
    default_language: Option<String>,
    strict_concepts: Option<bool>,
) -> Result<JsValue, JsValue> {
    use alizarin_core::csv_business_data_loader;

    let graph: alizarin_core::graph::StaticGraph = serde_json::from_str(graph_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse graph JSON: {}", e)))?;

    let collections: Vec<alizarin_core::skos::SkosCollection> =
        serde_json::from_str(collections_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse collections JSON: {}", e)))?;

    let options = csv_business_data_loader::BusinessDataCsvOptions {
        default_language: default_language.unwrap_or_else(|| "en".to_string()),
        strict_concepts: strict_concepts.unwrap_or(true),
    };

    match csv_business_data_loader::build_resources_from_business_csv(
        csv_data,
        &graph,
        &collections,
        options,
    ) {
        Ok(resources) => {
            let wrapped = csv_business_data_loader::wrap_business_data(&resources);
            let json_str = serde_json::to_string(&wrapped)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
            js_sys::JSON::parse(&json_str)
                .map_err(|_| JsValue::from_str("Failed to parse serialized JSON"))
        }
        Err(e) => Err(JsValue::from_str(&format!("{}", e))),
    }
}
