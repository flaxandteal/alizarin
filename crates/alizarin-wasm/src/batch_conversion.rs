use crate::graph::StaticGraph;
use crate::json_conversion::create_static_resource;
use alizarin_core::tiles_to_tree as core_tiles_to_tree;
use alizarin_core::{
    batch_merge_resources, merge_resources, transform_keys_to_snake, tree_to_tiles, StaticResource,
};
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
) -> Result<JsValue, JsValue> {
    // Parse tree from JSON
    let mut tree: Value = serde_json::from_str(tree_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse tree: {}", e)))?;

    // Transform keys if requested
    if from_camel {
        tree = transform_keys_to_snake(tree);
    }

    // Add graph_id if not present
    let graph_id = graph.graph_id();
    if let Value::Object(ref mut map) = tree {
        if !map.contains_key("graph_id") {
            map.insert("graph_id".to_string(), Value::String(graph_id.to_string()));
        }
    }

    // Convert tree to tiles (with optional id_key for deterministic UUID)
    let id_key_ref = id_key.as_deref();
    let result = tree_to_tiles(&tree, graph, strict, id_key_ref);

    match result {
        Ok(business_data) => {
            // Serialize to JS value
            serde_wasm_bindgen::to_value(&business_data)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
        }
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

/// Single tiles to tree conversion
///
/// Args:
///     resource_json: Single resource as JSON string
///     graph: StaticGraph wrapper
///
/// Returns:
///     Tree structure with resourceinstanceid and graph_id metadata
#[wasm_bindgen(js_name = tilesToTree)]
pub fn tiles_to_tree_enhanced(
    resource_json: &str,
    graph: &StaticGraph,
) -> Result<JsValue, JsValue> {
    // Parse resource from JSON
    let resource: Value = serde_json::from_str(resource_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse resource: {}", e)))?;

    let graph_id = graph.graph_id();

    // Extract resource_id
    let resource_id = resource
        .get("resourceinstance")
        .and_then(|ri| ri.get("resourceinstanceid"))
        .or_else(|| resource.get("resourceinstanceid"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Check if input is already in StaticResource format
    let input_json = if resource.get("resourceinstance").is_some() {
        resource.clone()
    } else {
        // Old format - convert to StaticResource
        let tiles: Vec<alizarin_core::StaticTile> = resource
            .get("tiles")
            .map(|t| serde_json::from_value(t.clone()))
            .transpose()
            .map_err(|e| JsValue::from_str(&format!("Failed to parse tiles: {}", e)))?
            .unwrap_or_default();

        let static_resource =
            create_static_resource(resource_id.clone(), graph_id.to_string(), tiles, graph);

        serde_json::to_value(&static_resource)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize resource: {}", e)))?
    };

    // Call tiles_to_tree
    match core_tiles_to_tree(&input_json, graph) {
        Ok(json_tree_array) => {
            // Extract first element and add metadata
            let mut tree = json_tree_array
                .as_array()
                .and_then(|arr| arr.first().cloned())
                .unwrap_or(Value::Object(serde_json::Map::new()));

            if let Value::Object(ref mut map) = tree {
                map.insert("resourceinstanceid".to_string(), Value::String(resource_id));
                map.insert("graph_id".to_string(), Value::String(graph_id.to_string()));
            }

            // Convert to JS value
            serde_wasm_bindgen::to_value(&tree)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
        }
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

/// Batch convert multiple JSON trees to tiles in parallel
///
/// Args:
///     trees_json: Array of tree structures as JSON string
///     graph: StaticGraph wrapper
///     from_camel: If true, convert keys from camelCase to snake_case
///     strict: If true, fail on any validation error
///     id_keys_json: Optional JSON array of keys for deterministic UUID v5 generation (one per tree)
///
/// Returns:
///     BusinessDataWrapper format: {business_data: {resources: [...]}, errors: [...]}
///
/// Note: In WASM, parallelism is limited by JavaScript's single-threaded nature,
/// but we can still optimize by processing in batches and reducing boundary crossings.
#[wasm_bindgen(js_name = batchTreesToTiles)]
pub fn batch_trees_to_tiles(
    trees_json: &str,
    graph: &StaticGraph,
    from_camel: bool,
    strict: bool,
    id_keys_json: Option<String>,
) -> Result<JsValue, JsValue> {
    // Parse trees from JSON
    let mut trees: Vec<Value> = serde_json::from_str(trees_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse trees: {}", e)))?;

    // Parse id_keys if provided
    let id_keys: Option<Vec<String>> = id_keys_json
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|e| JsValue::from_str(&format!("Failed to parse id_keys: {}", e)))?;

    // Validate id_keys length matches trees length
    if let Some(ref keys) = id_keys {
        if keys.len() != trees.len() {
            return Err(JsValue::from_str(&format!(
                "id_keys length ({}) must match trees length ({})",
                keys.len(),
                trees.len()
            )));
        }
    }

    let graph_id = graph.graph_id();
    let mut resources = Vec::new();
    let mut errors = Vec::new();

    // Process each tree
    for (i, tree) in trees.iter_mut().enumerate() {
        // Transform keys if requested
        if from_camel {
            *tree = transform_keys_to_snake(tree.clone());
        }

        // Add graph_id to tree if not present
        if let Value::Object(ref mut map) = tree {
            if !map.contains_key("graph_id") {
                map.insert("graph_id".to_string(), Value::String(graph_id.to_string()));
            }
        }

        // Get id_key for this tree (if id_keys array provided)
        let id_key_ref = id_keys.as_ref().map(|keys| keys[i].as_str());

        // Convert tree to tiles (with optional id_key for deterministic UUID)
        let result = tree_to_tiles(tree, graph, strict, id_key_ref);

        match result {
            Ok(business_data) => {
                // Extract first resource (full StaticResource with resourceinstance metadata)
                if let Some(resource) = business_data.business_data.resources.into_iter().next() {
                    match serde_json::to_value(&resource) {
                        Ok(resource_value) => resources.push(resource_value),
                        Err(e) => errors.push(format!("Tree {}: Failed to serialize: {}", i, e)),
                    }
                } else {
                    errors.push(format!("Tree {}: No resources returned", i));
                }
            }
            Err(e) => {
                errors.push(format!("Tree {}: {}", i, e));
                if strict {
                    return Err(JsValue::from_str(&format!("Strict mode error: {}", e)));
                }
            }
        }
    }

    // Return BusinessDataWrapper format with errors alongside
    let output = serde_json::json!({
        "business_data": {
            "resources": resources
        },
        "errors": errors,
        "error_count": errors.len()
    });

    // Convert to JS value
    serde_wasm_bindgen::to_value(&output)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
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
) -> Result<JsValue, JsValue> {
    // Parse resources from JSON
    let resources: Vec<Value> = serde_json::from_str(resources_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse resources: {}", e)))?;

    let graph_id = graph.graph_id();
    let mut results = Vec::new();
    let mut errors = Vec::new();

    // Process each resource
    for (i, resource) in resources.iter().enumerate() {
        // Extract resource_id from either format:
        // - New format: resource.resourceinstance.resourceinstanceid
        // - Old format: resource.resourceinstanceid
        let resource_id = resource
            .get("resourceinstance")
            .and_then(|ri| ri.get("resourceinstanceid"))
            .or_else(|| resource.get("resourceinstanceid"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Check if input is already in StaticResource format (has resourceinstance)
        let input_json = if resource.get("resourceinstance").is_some() {
            // Already in StaticResource format, use directly
            resource.clone()
        } else {
            // Old format - convert to StaticResource
            let tiles: Vec<alizarin_core::StaticTile> = resource
                .get("tiles")
                .map(|t| serde_json::from_value(t.clone()))
                .transpose()
                .map_err(|e| format!("Resource {}: Failed to parse tiles: {}", i, e))
                .map_err(|e| JsValue::from_str(&e))?
                .unwrap_or_default();

            let static_resource =
                create_static_resource(resource_id.clone(), graph_id.to_string(), tiles, graph);

            serde_json::to_value(&static_resource)
                .map_err(|e| format!("Resource {}: Failed to serialize: {}", i, e))
                .map_err(|e| JsValue::from_str(&e))?
        };

        // Call tiles_to_tree (returns array)
        match core_tiles_to_tree(&input_json, graph) {
            Ok(json_tree_array) => {
                // Extract first element and add metadata
                let mut tree = json_tree_array
                    .as_array()
                    .and_then(|arr| arr.first().cloned())
                    .unwrap_or(Value::Object(serde_json::Map::new()));

                if let Value::Object(ref mut map) = tree {
                    map.insert("resourceinstanceid".to_string(), Value::String(resource_id));
                    map.insert("graph_id".to_string(), Value::String(graph_id.to_string()));
                }

                results.push(tree);
            }
            Err(e) => {
                let error_msg = format!("Resource {}: {}", i, e);
                errors.push(error_msg.clone());
                if strict {
                    return Err(JsValue::from_str(&format!(
                        "Strict mode error: {}",
                        error_msg
                    )));
                }
            }
        }
    }

    // Return result based on strict mode
    let output = if errors.is_empty() {
        serde_json::json!({
            "success": true,
            "results": results,
            "count": results.len()
        })
    } else {
        serde_json::json!({
            "success": false,
            "results": results,
            "count": results.len(),
            "errors": errors,
            "error_count": errors.len()
        })
    };

    // Convert to JS value
    serde_wasm_bindgen::to_value(&output)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
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
pub fn merge_resources_wasm(resources_json: &str) -> Result<JsValue, JsValue> {
    let resources: Vec<StaticResource> = serde_json::from_str(resources_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse resources: {}", e)))?;

    match merge_resources(resources) {
        Ok(result) => {
            let output = serde_json::json!({
                "resource": result.resource,
                "warnings": result.warnings
            });
            serde_wasm_bindgen::to_value(&output)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
        }
        Err(e) => Err(JsValue::from_str(&e)),
    }
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
) -> Result<JsValue, JsValue> {
    // Parse the outer array of JSON strings
    let batch_strings: Vec<String> = serde_json::from_str(batches_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse batches array: {}", e)))?;

    // Parse each batch string into Vec<StaticResource>
    let mut resource_batches: Vec<Vec<StaticResource>> = Vec::new();
    for (i, batch_str) in batch_strings.iter().enumerate() {
        // Try to parse as JSON value first to determine format
        let value: Value = serde_json::from_str(batch_str)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse batch {}: {}", i, e)))?;

        let batch: Vec<StaticResource> = match &value {
            // Array of resources
            Value::Array(_) => serde_json::from_value(value).map_err(|e| {
                JsValue::from_str(&format!(
                    "Batch {}: Failed to parse as resource array: {}",
                    i, e
                ))
            })?,
            // Object - could be BusinessDataWrapper or single resource
            Value::Object(map) => {
                if let Some(bd) = map.get("business_data") {
                    // BusinessDataWrapper format
                    if let Some(resources) = bd.get("resources") {
                        serde_json::from_value(resources.clone()).map_err(|e| {
                            JsValue::from_str(&format!(
                                "Batch {}: Failed to parse business_data.resources: {}",
                                i, e
                            ))
                        })?
                    } else {
                        return Err(JsValue::from_str(&format!(
                            "Batch {}: business_data missing 'resources' field",
                            i
                        )));
                    }
                } else if map.contains_key("resourceinstance") {
                    // Single StaticResource
                    let resource: StaticResource = serde_json::from_value(value).map_err(|e| {
                        JsValue::from_str(&format!(
                            "Batch {}: Failed to parse as single resource: {}",
                            i, e
                        ))
                    })?;
                    vec![resource]
                } else {
                    return Err(JsValue::from_str(&format!("Batch {}: Unrecognized format - expected array, BusinessDataWrapper, or StaticResource", i)));
                }
            }
            _ => {
                return Err(JsValue::from_str(&format!(
                    "Batch {}: Expected array or object",
                    i
                )));
            }
        };

        resource_batches.push(batch);
    }

    let strict = strict.unwrap_or(true);
    let result = batch_merge_resources(
        resource_batches,
        recompute_descriptors.unwrap_or(false),
        strict,
    );

    if let Some(ref error) = result.error {
        return Err(JsValue::from_str(error));
    }

    let output = serde_json::json!({
        "resources": result.resources,
        "warnings": result.warnings
    });
    serde_wasm_bindgen::to_value(&output)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

// Tests for transform_keys_to_snake are in alizarin-core/src/string_utils.rs
