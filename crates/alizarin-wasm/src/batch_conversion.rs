/// Batch conversion functions for WASM - parallel processing
///
/// This module provides batch conversion functions that process multiple resources
/// in parallel, matching the Python API and enabling optimized bulk operations.

use wasm_bindgen::prelude::*;
use serde_json::Value;
use crate::graph::StaticGraph;
use crate::json_conversion::{tree_to_tiles, tree_to_tiles_strict, create_static_resource};
use alizarin_core::{BusinessDataWrapper, tiles_to_tree as core_tiles_to_tree};

/// Transform camelCase keys to snake_case recursively
fn transform_keys_to_snake(value: Value) -> Value {
    fn camel_to_snake(s: &str) -> String {
        let mut result = String::with_capacity(s.len() + 10);
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_uppercase() {
                if !result.is_empty() {
                    let next_is_upper_or_end = chars.peek().map(|n| n.is_uppercase()).unwrap_or(true);
                    let prev_was_upper = result.chars().last().map(|p| p.is_uppercase()).unwrap_or(false);

                    if !prev_was_upper || !next_is_upper_or_end {
                        result.push('_');
                    }
                }
                result.push(c.to_lowercase().next().unwrap());
            } else {
                result.push(c);
            }
        }
        result
    }

    match value {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, val) in map {
                let new_key = camel_to_snake(&key);
                let new_val = transform_keys_to_snake(val);
                new_map.insert(new_key, new_val);
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(transform_keys_to_snake).collect())
        }
        other => other,
    }
}

/// Single tree to tiles conversion with from_camel and strict parameters
///
/// This provides a more complete API than the basic json_conversion functions.
///
/// Args:
///     tree_json: Single tree structure as JSON string
///     graph: StaticGraph wrapper
///     from_camel: If true, convert keys from camelCase to snake_case
///     strict: If true, fail on any validation error
///
/// Returns:
///     BusinessDataWrapper format: {business_data: {resources: [...]}}
#[wasm_bindgen(js_name = treeToTiles)]
pub fn tree_to_tiles_enhanced(
    tree_json: &str,
    graph: &StaticGraph,
    from_camel: bool,
    strict: bool,
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

    // Convert tree to tiles
    let result = if strict {
        tree_to_tiles_strict(&tree, graph)
    } else {
        tree_to_tiles(&tree, graph)
    };

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
    let resource_id = resource.get("resourceinstance")
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
        let tiles: Vec<alizarin_core::StaticTile> = resource.get("tiles")
            .map(|t| serde_json::from_value(t.clone()))
            .transpose()
            .map_err(|e| JsValue::from_str(&format!("Failed to parse tiles: {}", e)))?
            .unwrap_or_default();

        let static_resource = create_static_resource(
            resource_id.clone(),
            graph_id.to_string(),
            tiles,
            &**graph,
        );

        serde_json::to_value(&static_resource)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize resource: {}", e)))?
    };

    // Call tiles_to_tree
    match core_tiles_to_tree(&input_json, &**graph) {
        Ok(json_tree_array) => {
            // Extract first element and add metadata
            let mut tree = json_tree_array.as_array()
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
) -> Result<JsValue, JsValue> {
    // Parse trees from JSON
    let mut trees: Vec<Value> = serde_json::from_str(trees_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse trees: {}", e)))?;

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

        // Convert tree to tiles
        let result = if strict {
            tree_to_tiles_strict(tree, graph)
        } else {
            tree_to_tiles(tree, graph)
        };

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
        let resource_id = resource.get("resourceinstance")
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
            let tiles: Vec<alizarin_core::StaticTile> = resource.get("tiles")
                .map(|t| serde_json::from_value(t.clone()))
                .transpose()
                .map_err(|e| format!("Resource {}: Failed to parse tiles: {}", i, e))
                .map_err(|e| JsValue::from_str(&e))?
                .unwrap_or_default();

            let static_resource = create_static_resource(
                resource_id.clone(),
                graph_id.to_string(),
                tiles,
                &**graph,
            );

            serde_json::to_value(&static_resource)
                .map_err(|e| format!("Resource {}: Failed to serialize: {}", i, e))
                .map_err(|e| JsValue::from_str(&e))?
        };

        // Call tiles_to_tree (returns array)
        match core_tiles_to_tree(&input_json, &**graph) {
            Ok(json_tree_array) => {
                // Extract first element and add metadata
                let mut tree = json_tree_array.as_array()
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
                    return Err(JsValue::from_str(&format!("Strict mode error: {}", error_msg)));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_to_snake() {
        let input = serde_json::json!({
            "firstName": "John",
            "lastName": "Doe",
            "contactInfo": {
                "emailAddress": "john@example.com"
            }
        });

        let result = transform_keys_to_snake(input);

        assert_eq!(result["first_name"], "John");
        assert_eq!(result["last_name"], "Doe");
        assert_eq!(result["contact_info"]["email_address"], "john@example.com");
    }
}
