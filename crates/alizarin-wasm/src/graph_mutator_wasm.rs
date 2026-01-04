//! WASM bindings for the JSON-based Graph Mutation API.
//!
//! This module exposes a simple JSON-based API for mutating graphs,
//! avoiding the need for complex wrapper types.

use wasm_bindgen::prelude::*;

use alizarin_core::graph_mutator::{
    apply_mutations_from_json as core_apply_mutations,
    get_mutation_schema as core_get_schema,
    generate_uuid_v5 as core_generate_uuid,
};
use alizarin_core::StaticGraph as CoreStaticGraph;

use crate::graph::StaticGraph;

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
pub fn generate_uuid_v5(
    group_type: &str,
    group_id: Option<String>,
    key: &str,
) -> String {
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
