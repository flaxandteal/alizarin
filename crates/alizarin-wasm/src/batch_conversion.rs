use std::ops::Deref;

use crate::graph::StaticGraph;
use serde_json::Value;

/// Batch conversion functions for WASM - parallel processing
///
/// This module provides batch conversion functions that process multiple resources
/// in parallel, matching the Python API and enabling optimized bulk operations.
use wasm_bindgen::prelude::*;

/// Single tree to tiles conversion with from_camel and strict parameters
///
/// This provides a more complete API than the basic json_conversion functions.
///
/// Args:
///     tree_json: Single tree structure as JSON string
///     graph: StaticGraph wrapper
///     from_camel: If true, convert keys from camelCase to snake_case
///     strict: If true, fail on any validation error
///     id_key: Optional key for deterministic UUID v5 resourceinstanceid generation
///     tile_namespace: Optional layer namespace (a UUID). Given, cardinality-n
///         tile ids are derived so the same tree yields the same ids; each layer
///         must use its own. Omitted, they stay random.
///
/// Returns:
///     BusinessDataWrapper format: {business_data: {resources: [...]}}
#[wasm_bindgen(js_name = treeToTiles)]
pub fn tree_to_tiles_enhanced(
    tree_json: &str,
    graph: &StaticGraph,
    from_camel: bool,
    strict: bool,
    id_key: Option<String>,
    random_ids: Option<bool>,
    tile_namespace: Option<String>,
) -> Result<JsValue, JsError> {
    let opts = alizarin_core::TreeToTilesOptions {
        id_key: id_key.as_deref(),
        from_camel,
        strict,
        random_ids: random_ids.unwrap_or(true),
        tile_namespace: tile_namespace.as_deref(),
        ..Default::default()
    };
    let ext_registry = crate::extension_registry::build_extension_registry();
    let wrapper =
        alizarin_core::tree_to_tiles_from_json(tree_json, graph, &opts, Some(&ext_registry), None)
            .map_err(|e| JsError::new(&e))?;
    crate::js_json::to_value(&wrapper)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Single tiles to tree conversion
///
/// Args:
///     resource_json: Single resource as JSON string: StaticResource, or flat
///         `{resourceinstanceid, graph_id, tiles}` (see core's `resource_for_tree`)
///     graph: StaticGraph wrapper
///
/// Returns:
///     Tree structure with resourceinstanceid and graph_id metadata
#[wasm_bindgen(js_name = tilesToTree)]
pub fn tiles_to_tree_enhanced(
    resource_json: &str,
    graph: &StaticGraph,
) -> Result<JsValue, JsError> {
    let tree = alizarin_core::tiles_to_tree_from_json(resource_json, graph.deref())
        .map_err(|e| JsError::new(&e))?;
    crate::js_json::to_value(&tree)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Convert tiles to a card-structured tree.
///
/// Groups data by the card/widget hierarchy (UI structure) rather than by
/// node edges (data structure). Each card contains its widgets' values and
/// nested child cards.
///
/// Args:
///     resource_json: Resource JSON string (same format as tilesToTree)
///     graph: StaticGraph wrapper (must have cards loaded)
///
/// Returns:
///     Card-structured tree with widgets and nested cards
#[wasm_bindgen(js_name = cardsToTree)]
pub fn cards_to_tree(resource_json: &str, graph: &StaticGraph) -> Result<JsValue, JsError> {
    let tree = alizarin_core::cards_to_tree_from_json(resource_json, graph.deref())
        .map_err(|e| JsError::new(&e))?;
    crate::js_json::to_value(&tree)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Batch convert multiple JSON trees to tiles in parallel
///
/// Args:
///     trees_json: Array of tree structures as JSON string
///     graph: StaticGraph wrapper
///     from_camel: If true, convert keys from camelCase to snake_case
///     strict: If true, fail on any validation error
///     id_keys_json: Optional JSON array of keys for deterministic UUID v5 generation (one per tree)
///     random_ids: Optional boolean - if not True and no id_keys, will try to use `slug`
///         descriptor with uuid5, erroring instead of using uuid4.
///     tile_namespace: Optional layer namespace for derived cardinality-n tile
///         ids, as for `treeToTiles`.
///
/// Returns:
///     {business_data: {resources: [...]}, errors, error_count, warnings, warning_count}
///     (core's `BatchTreesToTiles`, as on NAPI and Python).
#[wasm_bindgen(js_name = batchTreesToTiles)]
#[allow(clippy::too_many_arguments)]
pub fn batch_trees_to_tiles(
    trees_json: &str,
    graph: &StaticGraph,
    from_camel: bool,
    strict: bool,
    id_keys_json: Option<String>,
    random_ids: Option<bool>,
    resolve_markers: Option<bool>,
    scopes_json: Option<String>,
    tile_namespace: Option<String>,
) -> Result<JsValue, JsError> {
    let id_keys: Option<Vec<String>> = id_keys_json
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|e| JsError::new(&format!("Failed to parse id_keys: {}", e)))?;
    let scopes: Option<Value> = scopes_json
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|e| JsError::new(&format!("Failed to parse scopes: {}", e)))?;
    let opts = alizarin_core::BatchOptions {
        graph_id: graph.graph_id(),
        from_camel,
        strict,
        random_ids: random_ids.unwrap_or(false),
        resolve_markers: resolve_markers.unwrap_or(false),
        id_keys: id_keys.as_deref(),
        scopes: scopes.as_ref(),
        tile_namespace: tile_namespace.as_deref(),
    };
    let ext_registry = crate::extension_registry::build_extension_registry();
    let output = alizarin_core::batch_trees_to_tiles_from_json(
        trees_json,
        graph,
        &opts,
        Some(&ext_registry),
        None,
    )
    .map_err(|e| JsError::new(&e))?;
    crate::js_json::to_value(&output)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Batch convert multiple tiled resources to JSON trees
///
/// Args:
///     resources_json: Array of resources as JSON string
///     graph: StaticGraph wrapper
///     strict: If true, fail on any conversion error
///
/// Returns:
///     {success: bool, results: [...], count: N, errors: [...], error_count: M}
#[wasm_bindgen(js_name = batchTilesToTrees)]
pub fn batch_tiles_to_trees(
    resources_json: &str,
    graph: &StaticGraph,
    strict: bool,
) -> Result<JsValue, JsError> {
    let output =
        alizarin_core::batch_tiles_to_trees_from_json(resources_json, Some(graph.deref()), strict)
            .map_err(|e| JsError::new(&e))?;
    crate::js_json::to_value(&output)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Merge multiple resources with the same resourceinstanceid into one
///
/// Concatenates tiles from all resources, detecting and warning about duplicate tileids.
/// All resources must have the same resourceinstanceid.
///
/// Args:
///     resources_json: Array of StaticResource objects as JSON string
///
/// Returns:
///     {resource: StaticResource, warnings: string[]}
#[wasm_bindgen(js_name = mergeResources)]
pub fn merge_resources_wasm(resources_json: &str) -> Result<JsValue, JsError> {
    let result =
        alizarin_core::merge_resources_from_json(resources_json).map_err(|e| JsError::new(&e))?;
    crate::js_json::to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Batch merge resources from multiple JSON strings
///
/// Takes an array of JSON strings. Each string can be:
/// - An array of StaticResource objects: [{...}, {...}]
/// - A single StaticResource object: {resourceinstance: {...}, tiles: [...]}
/// - A BusinessDataWrapper: {business_data: {resources: [...]}}
///
/// Groups all resources by resourceinstanceid and merges each group.
///
/// Args:
///     batches_json: Array of JSON strings containing resources in any supported format
///     recompute_descriptors: If true, recomputes descriptors from merged tiles using
///         graphs from the registry. Graphs must be registered via WASMResourceModelWrapper.
///
/// Returns:
///     {resources: StaticResource[], warnings: string[]}
#[wasm_bindgen(js_name = batchMergeResources)]
pub fn batch_merge_resources_wasm(
    batches_json: &str,
    recompute_descriptors: Option<bool>,
    strict: Option<bool>,
) -> Result<JsValue, JsError> {
    let batches: Vec<String> = serde_json::from_str(batches_json)
        .map_err(|e| JsError::new(&format!("Failed to parse batches array: {}", e)))?;
    let result = alizarin_core::batch_merge_resources_from_json(
        &batches,
        recompute_descriptors.unwrap_or(false),
        strict.unwrap_or(true),
    )
    .map_err(|e| JsError::new(&e))?;
    crate::js_json::to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Compose one resource across an ordered layer stack, entirely in memory — the
/// binding form of the substrate's `hydrate_layers` (no DuckDB). The same resource
/// as it exists in each layer is supplied **topmost-first** in `resources_json`;
/// tiles are merged (identical tiles deduped, topmost wins) and cardinality-1
/// nodegroups unified with PerNodegroup (the topmost layer overrides a
/// single-valued group whole; multi-valued groups accumulate).
///
/// `base_graph_json` is the base model; `overlay_graphs_json` is a JSON array of
/// overlay `StaticGraph`s **bottom-to-top** (may be `[]`) — composition runs
/// against the merged (overlaid) model via `LayeredGraph`. `strict` makes a
/// cross-layer conflict on a single-valued group an error instead of a warning.
///
/// Returns `{ resource, warnings }`.
#[wasm_bindgen(js_name = composeResourceLayers)]
pub fn compose_resource_layers(
    resources_json: &str,
    base_graph_json: &str,
    overlay_graphs_json: &str,
    strict: Option<bool>,
) -> Result<JsValue, JsError> {
    let composed = alizarin_core::graph::compose_resource_layers_from_json(
        resources_json,
        base_graph_json,
        overlay_graphs_json,
        strict.unwrap_or(false),
    )
    .map_err(|e| JsError::new(&e))?;

    crate::js_json::to_value(&composed)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// A resource's lifecycle state resolved against its graph's lifecycle.
///
/// `resource_json` is a `StaticResource`; `lifecycle_json` is the graph's
/// `resource_instance_lifecycle` (`"null"` for none). An absent stored state is
/// the initial state; an undefined id is `"unknown"`; layers that disagreed
/// give `"conflict"`. Returns `{ status, state_id, is_initial, conflict }`.
#[wasm_bindgen(js_name = resourceLifecycleState)]
pub fn resource_lifecycle_state(
    resource_json: &str,
    lifecycle_json: &str,
) -> Result<JsValue, JsError> {
    let report = alizarin_core::graph::lifecycle_state_from_json(resource_json, lifecycle_json)
        .map_err(|e| JsError::new(&e))?;
    crate::js_json::to_value(&report)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Move a resource to lifecycle state `target`, returning `{ resource, event }`:
/// a tiles-free overlay record to compose over the resource, and the change.
///
/// `strict` (default `true`) requires a permitted edge (`next ∪ previous`) and
/// refuses to leave a conflict or unknown state; pass `false` explicitly for
/// Arches' membership-only check.
#[wasm_bindgen(js_name = transitionResourceLifecycle)]
pub fn transition_resource_lifecycle(
    resource_json: &str,
    lifecycle_json: &str,
    target: &str,
    strict: Option<bool>,
) -> Result<JsValue, JsError> {
    let result = alizarin_core::graph::transition_from_json(
        resource_json,
        lifecycle_json,
        target,
        strict.unwrap_or(true),
    )
    .map_err(|e| JsError::new(&e))?;
    crate::js_json::to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

// Tests for transform_keys_to_snake are in alizarin-core/src/string_utils.rs
