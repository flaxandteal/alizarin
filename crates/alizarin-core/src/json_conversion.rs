/// Hierarchical tree conversion for resources
///
/// This module provides bidirectional conversion between:
/// - **Tiled format**: `{"business_data": {"resources": [StaticResource, ...]}}` (Arches export format)
/// - **Tree format**: Array of nested hierarchical JSON using node aliases as keys `[{...}, {...}]`
///
/// These are NOT simple serialization functions - they perform structural transformation:
/// - `tiles_to_tree()`: Tiled resources → Array of nested tree objects
/// - `tree_to_tiles()`: Array of nested tree objects → Tiled resources with business_data wrapper

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::{Value, Map};
use serde::{Serialize, Deserialize};

use crate::graph::{StaticGraph, IndexedGraph};
use crate::pseudo_value_core::{
    PseudoValueCore, PseudoListCore, TileBuilder, TileBuilderContext, VisitorContext,
};
use crate::{StaticTile, StaticNode};
use crate::graph::{StaticResource, StaticResourceMetadata};
use crate::type_coercion::coerce_value;
use crate::graph_mutator::generate_uuid_v5;

/// Wrapper for business data import/export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDataWrapper {
    pub business_data: BusinessData,
}

/// Business data containing resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessData {
    pub resources: Vec<StaticResource>,
}

/// Create a StaticResource from basic components with computed descriptors
pub fn create_static_resource(
    resourceinstanceid: String,
    graph_id: String,
    tiles: Vec<StaticTile>,
    graph: &StaticGraph,
) -> StaticResource {
    let indexed = IndexedGraph::new(graph.clone());
    let descriptors = indexed.build_descriptors(&tiles);

    // Use name from descriptors, or fallback to resourceinstanceid
    let name = descriptors.name.clone()
        .unwrap_or_else(|| resourceinstanceid.clone());

    StaticResource {
        resourceinstance: StaticResourceMetadata {
            descriptors,
            graph_id,
            name,
            resourceinstanceid,
            publication_id: None,
            principaluser_id: None,
            legacyid: None,
            graph_publication_id: None,
            createdtime: None,
            lastmodified: None,
        },
        tiles: Some(tiles),
        metadata: HashMap::new(),
        cache: None,
        scopes: None,
        tiles_loaded: Some(true),
    }
}

/// Convert tiled resources to nested tree array
///
/// **Structural transformation** (not just serialization):
/// - Input: `{"business_data": {"resources": [StaticResource, ...]}}` OR single StaticResource
/// - Output: Array of nested JSON tree objects `[{...}, {...}]`
///
/// Each resource tree uses node aliases as keys.
pub fn tiles_to_tree(input: &Value, graph: &StaticGraph) -> Result<Value, String> {
    let resources = extract_resources(input)?;

    let mut tree_resources = Vec::new();

    for resource in resources {
        let tiles = resource.tiles.as_ref()
            .ok_or_else(|| "Resource has no tiles".to_string())?;

        let tree = resource_tiles_to_tree(tiles, &resource.resourceinstance, graph)?;
        tree_resources.push(tree);
    }

    Ok(Value::Array(tree_resources))
}

/// Convert a single resource's tiles to tree format
fn resource_tiles_to_tree(
    tiles: &[StaticTile],
    metadata: &StaticResourceMetadata,
    graph: &StaticGraph,
) -> Result<Value, String> {
    let nodes_by_alias = graph.nodes_by_alias_arc()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;

    let edges = graph.edges_map()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;

    // Build pseudo_cache from tiles
    let pseudo_cache = build_pseudo_cache_from_tiles(
        tiles,
        &nodes_by_alias,
        graph,
        edges,
    );

    // Create root pseudo value
    let root = graph.root_node();
    let root_alias = root.alias.clone().unwrap_or_default();

    let child_node_ids = graph.get_child_ids(&root.nodeid)
        .cloned()
        .unwrap_or_default();

    let root_pseudo = PseudoValueCore::from_node_and_tile(
        Arc::new(root.clone()),
        None,
        None,
        child_node_ids,
    );

    let root_list = PseudoListCore::from_values_with_cardinality(
        root_alias.clone(),
        vec![root_pseudo],
        true,
    );

    let mut full_cache = pseudo_cache;
    full_cache.insert(root_alias.clone(), root_list.clone());

    let ctx = VisitorContext::new(
        &full_cache,
        &nodes_by_alias,
        &edges,
    );

    let mut tree = if let Some(root_value) = root_list.values.first() {
        root_value.to_json(&ctx)
    } else {
        Value::Object(Map::new())
    };

    // Add metadata to tree
    if let Some(obj) = tree.as_object_mut() {
        obj.insert("resourceinstanceid".to_string(), Value::String(metadata.resourceinstanceid.clone()));
        obj.insert("graph_id".to_string(), Value::String(metadata.graph_id.clone()));
        if let Some(ref name) = metadata.descriptors.name {
            obj.insert("_name".to_string(), Value::String(name.clone()));
        }
        if let Some(ref desc) = metadata.descriptors.description {
            obj.insert("_description".to_string(), Value::String(desc.clone()));
        }
        if let Some(ref legacyid) = metadata.legacyid {
            obj.insert("legacyid".to_string(), Value::String(legacyid.clone()));
        }
    }

    Ok(tree)
}

/// Extract resources from input (handles both wrapper format and single resource)
fn extract_resources(input: &Value) -> Result<Vec<StaticResource>, String> {
    // Try business_data wrapper format first
    if let Some(bd) = input.get("business_data") {
        if let Some(resources) = bd.get("resources") {
            if let Some(arr) = resources.as_array() {
                let mut result = Vec::new();
                for r in arr {
                    let resource: StaticResource = serde_json::from_value(r.clone())
                        .map_err(|e| format!("Failed to parse resource: {}", e))?;
                    result.push(resource);
                }
                return Ok(result);
            }
        }
    }

    // Try single StaticResource
    if let Ok(resource) = serde_json::from_value::<StaticResource>(input.clone()) {
        return Ok(vec![resource]);
    }

    Err("Input must be BusinessDataWrapper or StaticResource".to_string())
}

/// Build pseudo_cache from tiles
///
/// This also creates synthetic entries for parent semantic collector nodes
/// when their child nodegroups have tiles but the parent nodegroup doesn't.
fn build_pseudo_cache_from_tiles(
    tiles: &[StaticTile],
    nodes_by_alias: &HashMap<String, Arc<StaticNode>>,
    graph: &StaticGraph,
    edges: &HashMap<String, Vec<String>>,
) -> HashMap<String, PseudoListCore> {
    use std::collections::HashSet;

    let mut pseudo_cache: HashMap<String, PseudoListCore> = HashMap::new();

    // Track which nodegroups have tiles
    let nodegroups_with_tiles: HashSet<&str> = tiles.iter()
        .map(|t| t.nodegroup_id.as_str())
        .collect();

    for tile in tiles {
        let tile_arc = Arc::new(tile.clone());

        let nodes_in_ng = graph.get_nodes_in_nodegroup(&tile.nodegroup_id);

        for node in nodes_in_ng {
            let alias = match &node.alias {
                Some(a) if !a.is_empty() => a.clone(),
                _ => continue,
            };

            let child_node_ids = edges.get(&node.nodeid)
                .cloned()
                .unwrap_or_default();

            let tile_data = tile.data.get(&node.nodeid).cloned();

            let node_arc = nodes_by_alias.get(&alias)
                .map(Arc::clone)
                .unwrap_or_else(|| Arc::new(node.clone()));

            let pv = PseudoValueCore::from_node_and_tile(
                node_arc,
                Some(Arc::clone(&tile_arc)),
                tile_data,
                child_node_ids,
            );

            let is_single = node.nodegroup_id.as_ref()
                .and_then(|ng_id| graph.get_nodegroup_by_id(ng_id))
                .map(|ng| ng.cardinality.as_ref().map(|c| c != "n").unwrap_or(true))
                .unwrap_or(true);

            pseudo_cache
                .entry(alias.clone())
                .and_modify(|existing| {
                    let new_list = PseudoListCore::from_values_with_cardinality(
                        alias.clone(),
                        vec![pv.clone()],
                        is_single,
                    );
                    existing.merge(new_list);
                })
                .or_insert_with(|| {
                    PseudoListCore::from_values_with_cardinality(
                        alias.clone(),
                        vec![pv],
                        is_single,
                    )
                });
        }

        // Create synthetic entries for parent semantic collector nodes
        // Walk up the parentnodegroup_id chain and create entries for
        // semantic collectors that don't have their own tiles
        let mut current_ng_id = Some(tile.nodegroup_id.clone());

        while let Some(ng_id) = current_ng_id {
            let nodegroup = match graph.get_nodegroup_by_id(&ng_id) {
                Some(ng) => ng,
                None => break,
            };

            // Get parent nodegroup
            let parent_ng_id = match &nodegroup.parentnodegroup_id {
                Some(pid) => pid.clone(),
                None => break, // No parent, we're done
            };

            // Check if parent nodegroup already has tiles
            if nodegroups_with_tiles.contains(parent_ng_id.as_str()) {
                // Parent has tiles, skip up to grandparent
                current_ng_id = Some(parent_ng_id);
                continue;
            }

            // Check if we already created an entry for the parent
            let parent_nodegroup = match graph.get_nodegroup_by_id(&parent_ng_id) {
                Some(ng) => ng,
                None => break,
            };

            // Find the grouping/semantic node for the parent nodegroup
            // This is the node that acts as the semantic collector
            let grouping_node_id = parent_nodegroup.grouping_node_id.as_ref()
                .unwrap_or(&parent_ng_id); // Fallback to nodegroup_id if not set

            // Find the semantic collector node
            let semantic_node = graph.nodes_slice()
                .iter()
                .find(|n| n.nodeid == *grouping_node_id);

            if let Some(semantic_node) = semantic_node {
                if let Some(ref alias) = semantic_node.alias {
                    if !alias.is_empty() && !pseudo_cache.contains_key(alias) {
                        // Create a synthetic pseudo value for this semantic collector
                        let child_node_ids = edges.get(&semantic_node.nodeid)
                            .cloned()
                            .unwrap_or_default();

                        let node_arc = nodes_by_alias.get(alias)
                            .map(Arc::clone)
                            .unwrap_or_else(|| Arc::new(semantic_node.clone()));

                        // Create with no tile - this is a synthetic entry
                        let pv = PseudoValueCore::from_node_and_tile(
                            node_arc,
                            None, // No tile for synthetic collectors
                            None, // No tile data
                            child_node_ids,
                        );

                        let is_single = parent_nodegroup.cardinality.as_ref()
                            .map(|c| c != "n")
                            .unwrap_or(true);

                        pseudo_cache.insert(
                            alias.clone(),
                            PseudoListCore::from_values_with_cardinality(
                                alias.clone(),
                                vec![pv],
                                is_single,
                            )
                        );
                    }
                }
            }

            // Continue up to grandparent
            current_ng_id = Some(parent_ng_id);
        }
    }

    pseudo_cache
}

/// Convert nested tree array to tiled resource format with business_data wrapper
///
/// **Structural transformation**:
/// - Input: Array of nested tree objects `[{...}, {...}]` OR single tree object `{...}`
/// - Output: `{"business_data": {"resources": [StaticResource, ...]}}`
///
/// Descriptors are calculated automatically from tiles.
pub fn tree_to_tiles(
    json: &Value,
    graph: &StaticGraph,
) -> Result<BusinessDataWrapper, String> {
    tree_to_tiles_internal(json, graph, false, None)
}

/// Convert with strict validation (fails on unknown fields)
pub fn tree_to_tiles_strict(
    json: &Value,
    graph: &StaticGraph,
) -> Result<BusinessDataWrapper, String> {
    tree_to_tiles_internal(json, graph, true, None)
}

/// Convert tree to tiles with optional deterministic ID generation
///
/// When `id_key` is provided and the tree does not contain a `resourceinstanceid`,
/// a deterministic UUID v5 will be generated using the key and graph ID as namespace.
/// This enables consistent IDs across multiple runs with the same input.
///
/// # Arguments
/// * `json` - Tree structure to convert
/// * `graph` - Graph definition
/// * `id_key` - Optional key for deterministic UUID v5 generation
pub fn tree_to_tiles_with_id_key(
    json: &Value,
    graph: &StaticGraph,
    id_key: Option<&str>,
) -> Result<BusinessDataWrapper, String> {
    tree_to_tiles_internal(json, graph, false, id_key)
}

/// Convert with strict validation and optional deterministic ID
pub fn tree_to_tiles_strict_with_id_key(
    json: &Value,
    graph: &StaticGraph,
    id_key: Option<&str>,
) -> Result<BusinessDataWrapper, String> {
    tree_to_tiles_internal(json, graph, true, id_key)
}

fn tree_to_tiles_internal(
    json: &Value,
    graph: &StaticGraph,
    strict: bool,
    id_key: Option<&str>,
) -> Result<BusinessDataWrapper, String> {
    let trees = extract_tree_resources(json)?;

    let mut resources = Vec::new();

    for tree in trees {
        let resource = single_tree_to_resource(&tree, graph, strict, id_key)?;
        resources.push(resource);
    }

    Ok(BusinessDataWrapper {
        business_data: BusinessData { resources },
    })
}

/// Extract tree resources from input (array or single object)
fn extract_tree_resources(json: &Value) -> Result<Vec<Value>, String> {
    // Array of tree objects
    if let Some(arr) = json.as_array() {
        return Ok(arr.clone());
    }

    // Single tree object
    if json.is_object() {
        return Ok(vec![json.clone()]);
    }

    Err("Input must be array of tree objects or single tree object".to_string())
}

/// Convert a single tree to StaticResource
fn single_tree_to_resource(
    json: &Value,
    graph: &StaticGraph,
    strict: bool,
    id_key: Option<&str>,
) -> Result<StaticResource, String> {
    let obj = json.as_object()
        .ok_or_else(|| "JSON must be an object".to_string())?;

    // Extract metadata from tree
    // Priority: explicit resourceinstanceid > id_key (UUID v5) > random UUID v4
    let resource_id = obj.get("resourceinstanceid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            // If id_key provided, generate deterministic UUID v5
            id_key.map(|key| {
                generate_uuid_v5(("resource", Some(&graph.graphid)), key)
            })
        })
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let graph_id = obj.get("graph_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| graph.graphid.clone());

    let legacyid = obj.get("legacyid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let nodes_by_alias = graph.nodes_by_alias_arc()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;
    let edges = graph.edges_map()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;

    // Strict mode validation
    let known_metadata = ["resourceinstanceid", "graph_id", "legacyid", "_name", "_description", "_map_popup"];
    if strict {
        for key in obj.keys() {
            if !known_metadata.contains(&key.as_str()) && !nodes_by_alias.contains_key(key) {
                return Err(format!("Unknown field '{}' not found in graph aliases", key));
            }
        }
    }

    let mut pseudo_cache: HashMap<String, PseudoListCore> = HashMap::new();

    let root = graph.root_node();
    build_pseudo_values_from_json(
        obj,
        &Arc::new(root.clone()),
        &nodes_by_alias,
        graph,
        edges,
        &resource_id,
        None,
        &mut pseudo_cache,
        strict,
    )?;

    let ctx = TileBuilderContext {
        pseudo_cache: &pseudo_cache,
        nodes_by_alias: &nodes_by_alias,
        edges: &edges,
        resourceinstance_id: resource_id.clone(),
        depth: 0,
        max_depth: 100,
    };

    let mut tiles_map: HashMap<String, TileBuilder> = HashMap::new();

    if let Some(child_ids) = edges.get(&root.nodeid) {
        for child_id in child_ids {
            let child_node = nodes_by_alias.values()
                .find(|n| n.nodeid == *child_id);

            if let Some(child_node) = child_node {
                if let Some(alias) = &child_node.alias {
                    if let Some(pseudo_list) = pseudo_cache.get(alias) {
                        pseudo_list.collect_tiles(&ctx, &mut tiles_map);
                    }
                }
            }
        }
    }

    let tiles: Vec<StaticTile> = tiles_map.values()
        .map(|builder| builder.to_static_tile())
        .collect();

    // Calculate descriptors from tiles
    let indexed = IndexedGraph::new(graph.clone());
    let descriptors = indexed.build_descriptors(&tiles);

    // Use name from descriptors, or fallback to resourceinstanceid
    let name = descriptors.name.clone()
        .unwrap_or_else(|| resource_id.clone());

    Ok(StaticResource {
        resourceinstance: StaticResourceMetadata {
            descriptors,
            graph_id,
            name,
            resourceinstanceid: resource_id,
            publication_id: None,
            principaluser_id: None,
            legacyid,
            graph_publication_id: None,
            createdtime: None,
            lastmodified: None,
        },
        tiles: Some(tiles),
        metadata: HashMap::new(),
        cache: None,
        scopes: None,
        tiles_loaded: Some(true),
    })
}

/// Build PseudoValueCore tree from JSON and populate pseudo_cache
fn build_pseudo_values_from_json(
    json_obj: &Map<String, Value>,
    current_node: &Arc<StaticNode>,
    nodes_by_alias: &HashMap<String, Arc<StaticNode>>,
    graph: &StaticGraph,
    edges: &HashMap<String, Vec<String>>,
    resource_id: &str,
    parent_tile: Option<Arc<StaticTile>>,
    pseudo_cache: &mut HashMap<String, PseudoListCore>,
    strict: bool,
) -> Result<(), String> {
    let child_ids = edges.get(&current_node.nodeid)
        .cloned()
        .unwrap_or_default();

    for child_id in child_ids {
        let child_node = nodes_by_alias.values()
            .find(|n| n.nodeid == child_id);

        let child_node = match child_node {
            Some(n) => Arc::clone(n),
            None => continue,
        };

        let child_alias = match &child_node.alias {
            Some(a) if !a.is_empty() => a.clone(),
            _ => continue,
        };

        let json_value = match json_obj.get(&child_alias) {
            Some(v) => v,
            None => continue,
        };

        let nodegroup_id = child_node.nodegroup_id.as_ref()
            .ok_or_else(|| format!("Node {} has no nodegroup_id", child_id))?;

        let is_single = graph.get_nodegroup_by_id(nodegroup_id)
            .map(|ng| ng.cardinality.as_ref().map(|c| c != "n").unwrap_or(true))
            .unwrap_or(true);

        let config_value = if child_node.config.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(
                child_node.config.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            ))
        };

        let child_child_ids = edges.get(&child_node.nodeid)
            .cloned()
            .unwrap_or_default();

        let shares_nodegroup = parent_tile.as_ref()
            .map(|pt| pt.nodegroup_id == *nodegroup_id)
            .unwrap_or(false);

        let has_graph_children = !child_child_ids.is_empty();
        let mut values: Vec<PseudoValueCore> = Vec::new();

        let valid_child_aliases: std::collections::HashSet<&str> = if strict && has_graph_children {
            child_child_ids.iter()
                .filter_map(|id| nodes_by_alias.values().find(|n| n.nodeid == *id))
                .filter_map(|n| n.alias.as_deref())
                .collect()
        } else {
            std::collections::HashSet::new()
        };

        if json_value.is_array() {
            let array = json_value.as_array().unwrap();
            for item in array {
                if has_graph_children {
                    if let Some(item_obj) = item.as_object() {
                        if strict {
                            for key in item_obj.keys() {
                                if !valid_child_aliases.contains(key.as_str()) {
                                    return Err(format!(
                                        "Unknown field '{}' in '{}' - valid fields: {:?}",
                                        key, child_alias, valid_child_aliases
                                    ));
                                }
                            }
                        }

                        let (pv, tile) = create_pseudo_value_from_json(
                            item_obj,
                            &child_node,
                            &child_child_ids,
                            nodegroup_id,
                            &config_value,
                            resource_id,
                            if shares_nodegroup { parent_tile.clone() } else { None },
                        );

                        values.push(pv);

                        build_pseudo_values_from_json(
                            item_obj,
                            &child_node,
                            nodes_by_alias,
                            graph,
                            edges,
                            resource_id,
                            Some(tile),
                            pseudo_cache,
                            strict,
                        )?;
                    }
                } else {
                    let (pv, _tile) = create_pseudo_value_from_leaf(
                        item,
                        &child_node,
                        &child_child_ids,
                        nodegroup_id,
                        &config_value,
                        resource_id,
                        if shares_nodegroup { parent_tile.clone() } else { None },
                    );
                    values.push(pv);
                }
            }
        } else if has_graph_children && json_value.is_object() {
            let item_obj = json_value.as_object().unwrap();

            if strict {
                for key in item_obj.keys() {
                    if !valid_child_aliases.contains(key.as_str()) {
                        return Err(format!(
                            "Unknown field '{}' in '{}' - valid fields: {:?}",
                            key, child_alias, valid_child_aliases
                        ));
                    }
                }
            }

            let (pv, tile) = create_pseudo_value_from_json(
                item_obj,
                &child_node,
                &child_child_ids,
                nodegroup_id,
                &config_value,
                resource_id,
                if shares_nodegroup { parent_tile.clone() } else { None },
            );

            values.push(pv);

            build_pseudo_values_from_json(
                item_obj,
                &child_node,
                nodes_by_alias,
                graph,
                edges,
                resource_id,
                Some(tile),
                pseudo_cache,
                strict,
            )?;
        } else {
            let (pv, _tile) = create_pseudo_value_from_leaf(
                json_value,
                &child_node,
                &child_child_ids,
                nodegroup_id,
                &config_value,
                resource_id,
                if shares_nodegroup { parent_tile.clone() } else { None },
            );
            values.push(pv);
        }

        if !values.is_empty() {
            let list = PseudoListCore::from_values_with_cardinality(
                child_alias.clone(),
                values,
                is_single,
            );

            pseudo_cache.entry(child_alias)
                .and_modify(|existing| existing.merge(list.clone()))
                .or_insert(list);
        }
    }

    Ok(())
}

/// Create a PseudoValueCore from a JSON object
fn create_pseudo_value_from_json(
    json_obj: &Map<String, Value>,
    node: &Arc<StaticNode>,
    child_node_ids: &[String],
    nodegroup_id: &str,
    config: &Option<Value>,
    resource_id: &str,
    shared_tile: Option<Arc<StaticTile>>,
) -> (PseudoValueCore, Arc<StaticTile>) {
    let tile = shared_tile.unwrap_or_else(|| {
        let tile_id = uuid::Uuid::new_v4().to_string();

        let mut new_tile = StaticTile::new_empty(nodegroup_id.to_string());
        new_tile.tileid = Some(tile_id);
        new_tile.resourceinstance_id = resource_id.to_string();
        Arc::new(new_tile)
    });

    let tile_data = json_obj.get("_value").map(|value| {
        let coerced = coerce_value(&node.datatype, value, config.as_ref());
        if !coerced.is_null() && coerced.error.is_none() {
            coerced.tile_data
        } else {
            Value::Null
        }
    }).filter(|v| !v.is_null());

    let pv = PseudoValueCore::from_node_and_tile(
        Arc::clone(node),
        Some(Arc::clone(&tile)),
        tile_data,
        child_node_ids.to_vec(),
    );

    (pv, tile)
}

/// Create a PseudoValueCore from a leaf JSON value
fn create_pseudo_value_from_leaf(
    json_value: &Value,
    node: &Arc<StaticNode>,
    child_node_ids: &[String],
    nodegroup_id: &str,
    config: &Option<Value>,
    resource_id: &str,
    shared_tile: Option<Arc<StaticTile>>,
) -> (PseudoValueCore, Arc<StaticTile>) {
    let tile = shared_tile.unwrap_or_else(|| {
        let tile_id = uuid::Uuid::new_v4().to_string();

        let mut new_tile = StaticTile::new_empty(nodegroup_id.to_string());
        new_tile.tileid = Some(tile_id);
        new_tile.resourceinstance_id = resource_id.to_string();
        Arc::new(new_tile)
    });

    let value_to_coerce = json_value
        .as_object()
        .and_then(|obj| obj.get("_value"))
        .unwrap_or(json_value);

    let coerced = coerce_value(&node.datatype, value_to_coerce, config.as_ref());
    let tile_data = if !coerced.is_null() && coerced.error.is_none() {
        Some(coerced.tile_data)
    } else {
        None
    };

    let pv = PseudoValueCore::from_node_and_tile(
        Arc::clone(node),
        Some(Arc::clone(&tile)),
        tile_data,
        child_node_ids.to_vec(),
    );

    (pv, tile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn load_group_graph() -> StaticGraph {
        // Use workspace root for test data
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap()
            .to_path_buf();
        let test_file = workspace_root.join("tests/data/models/Group.json");
        let json_str = fs::read_to_string(&test_file)
            .expect("Failed to read Group.json");
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .expect("Failed to parse Group.json");

        let graph_data = json["graph"][0].clone();

        let mut core_graph: StaticGraph = serde_json::from_value(graph_data)
            .expect("Failed to deserialize StaticGraph");
        core_graph.build_indices();

        core_graph
    }

    fn create_test_business_data(graph: &StaticGraph) -> BusinessDataWrapper {
        let mut tiles = Vec::new();

        let basic_info_ng = graph.nodegroups_slice()
            .iter()
            .find(|ng| {
                graph.nodes_slice()
                    .iter()
                    .any(|n| n.alias.as_deref() == Some("basic_info")
                          && n.nodegroup_id.as_ref() == Some(&ng.nodegroupid))
            })
            .expect("Could not find basic_info nodegroup");

        let mut tile = StaticTile::new_empty(basic_info_ng.nodegroupid.clone());
        tile.resourceinstance_id = "test-resource-123".to_string();
        tile.tileid = Some("test-tile-1".to_string());

        if let Some(name_node) = graph.nodes_slice()
            .iter()
            .find(|n| n.alias.as_deref() == Some("name")) {
            tile.data.insert(
                name_node.nodeid.clone(),
                serde_json::json!({"en": "Test Group", "ga": "Grúpa Tástála"})
            );
        }

        if let Some(desc_node) = graph.nodes_slice()
            .iter()
            .find(|n| n.alias.as_deref() == Some("description")) {
            tile.data.insert(
                desc_node.nodeid.clone(),
                serde_json::json!("A test group for unit testing")
            );
        }

        tiles.push(tile);

        // Calculate descriptors
        let indexed = IndexedGraph::new(graph.clone());
        let descriptors = indexed.build_descriptors(&tiles);
        let name = descriptors.name.clone().unwrap_or_else(|| "test-resource-123".to_string());

        BusinessDataWrapper {
            business_data: BusinessData {
                resources: vec![StaticResource {
                    resourceinstance: StaticResourceMetadata {
                        descriptors,
                        graph_id: graph.graphid.clone(),
                        name,
                        resourceinstanceid: "test-resource-123".to_string(),
                        publication_id: None,
                        principaluser_id: None,
                        legacyid: None,
                        graph_publication_id: None,
                        createdtime: None,
                        lastmodified: None,
                    },
                    tiles: Some(tiles),
                    metadata: HashMap::new(),
                    cache: None,
                    scopes: None,
                    tiles_loaded: Some(true),
                }],
            },
        }
    }

    #[test]
    fn test_tiles_to_tree_basic() {
        let graph = load_group_graph();
        let business_data = create_test_business_data(&graph);

        let input = serde_json::to_value(&business_data).unwrap();
        let tree = tiles_to_tree(&input, &graph)
            .expect("tiles_to_tree failed");

        assert!(tree.is_array(), "Result should be an array");

        let resources = tree.as_array().unwrap();
        assert_eq!(resources.len(), 1, "Should have one resource");

        let resource_tree = &resources[0];
        // Tree format has resourceinstanceid and graph_id at root level (not nested in resourceinstance)
        assert!(resource_tree.get("resourceinstanceid").is_some(), "Should include resourceinstanceid");
        assert!(resource_tree.get("graph_id").is_some(), "Should include graph_id");
    }

    #[test]
    fn test_tree_to_tiles_array() {
        let graph = load_group_graph();

        // Input: array of tree objects
        let trees = serde_json::json!([{
            "resourceinstanceid": "test-resource-456",
            "graph_id": graph.graphid,
            "basic_info": [{
                "name": {"en": "JSON Test Group", "ga": "Grúpa Tástála JSON"},
                "description": "Created from JSON tree"
            }]
        }]);

        let result = tree_to_tiles(&trees, &graph)
            .expect("tree_to_tiles failed");

        assert_eq!(result.business_data.resources.len(), 1);
        let resource = &result.business_data.resources[0];
        assert_eq!(resource.resourceinstance.resourceinstanceid, "test-resource-456");
        assert_eq!(resource.resourceinstance.graph_id, graph.graphid);
        assert!(resource.tiles.as_ref().map(|t| !t.is_empty()).unwrap_or(false), "Should have created tiles");
    }

    #[test]
    fn test_tree_to_tiles_single_object() {
        let graph = load_group_graph();

        // Input: single tree object (no array wrapper)
        let tree = serde_json::json!({
            "resourceinstanceid": "test-resource-789",
            "graph_id": graph.graphid,
            "basic_info": [{
                "name": {"en": "Single Resource Test"},
                "description": "Testing single resource input"
            }]
        });

        let result = tree_to_tiles(&tree, &graph)
            .expect("tree_to_tiles failed");

        assert_eq!(result.business_data.resources.len(), 1);
        let resource = &result.business_data.resources[0];
        assert_eq!(resource.resourceinstance.resourceinstanceid, "test-resource-789");
    }

    #[test]
    fn test_round_trip() {
        let graph = load_group_graph();

        // Create initial tree array
        let initial_trees = serde_json::json!([{
            "resourceinstanceid": "round-trip-test",
            "graph_id": graph.graphid,
            "basic_info": [{
                "name": {"en": "Round Trip Test", "ga": "Tástáil Timpeall"},
                "description": "Testing round trip conversion"
            }]
        }]);

        // Convert to tiles (business_data format)
        let tiles_result = tree_to_tiles(&initial_trees, &graph)
            .expect("tree_to_tiles failed");

        // Convert back to tree array
        let tiles_json = serde_json::to_value(&tiles_result).unwrap();
        let tree_result = tiles_to_tree(&tiles_json, &graph)
            .expect("tiles_to_tree failed");

        // Verify structure
        assert!(tree_result.is_array());
        let resources = tree_result.as_array().unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0]["resourceinstanceid"], "round-trip-test");
    }

    #[test]
    fn test_tree_to_tiles_serialization_format() {
        let graph = load_group_graph();

        let tree = serde_json::json!({
            "resourceinstanceid": "test-serialize",
            "graph_id": graph.graphid,
            "basic_info": [{
                "name": {"en": "Serialize Test"}
            }]
        });

        let result = tree_to_tiles(&tree, &graph)
            .expect("tree_to_tiles failed");

        // Extract first resource
        let resource = &result.business_data.resources[0];

        // Serialize to JSON to see the format
        let json = serde_json::to_string_pretty(resource).unwrap();
        println!("Serialized StaticResource:\n{}", json);

        // Parse back and check structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Check that resourceinstance is nested (not flattened)
        assert!(parsed.get("resourceinstance").is_some(),
            "Expected nested 'resourceinstance', got: {}", json);
        assert!(parsed.get("resourceinstanceid").is_none(),
            "Should NOT have 'resourceinstanceid' at root level, got: {}", json);
    }

    /// Test that parent semantic collectors appear in output even when
    /// only child nodegroups have tiles (like location_data -> Geometry)
    #[test]
    fn test_parent_semantic_collector_without_tile() {
        // Create graph from JSON - simpler than constructing manually
        let graph_json = serde_json::json!({
            "graphid": "test-graph",
            "name": {"en": "Test Graph"},
            "root": {
                "nodeid": "root-id",
                "name": "Heritage Item",
                "alias": "heritage_item",
                "datatype": "semantic",
                "graph_id": "test-graph",
                "istopnode": true
            },
            "nodes": [
                {
                    "nodeid": "root-id",
                    "name": "Heritage Item",
                    "alias": "heritage_item",
                    "datatype": "semantic",
                    "graph_id": "test-graph",
                    "istopnode": true
                },
                {
                    "nodeid": "location-data-id",
                    "name": "Location Data",
                    "alias": "location_data",
                    "datatype": "semantic",
                    "nodegroup_id": "parent-ng",
                    "graph_id": "test-graph",
                    "is_collector": true
                },
                {
                    "nodeid": "geometry-id",
                    "name": "Geometry",
                    "alias": "geometry",
                    "datatype": "semantic",
                    "nodegroup_id": "child-ng",
                    "graph_id": "test-graph"
                },
                {
                    "nodeid": "geospatial-id",
                    "name": "Geospatial Coordinates",
                    "alias": "geospatial_coordinates",
                    "datatype": "geojson-feature-collection",
                    "nodegroup_id": "child-ng",
                    "graph_id": "test-graph"
                }
            ],
            "nodegroups": [
                {
                    "nodegroupid": "parent-ng",
                    "cardinality": "1",
                    "grouping_node_id": "location-data-id"
                },
                {
                    "nodegroupid": "child-ng",
                    "cardinality": "1",
                    "parentnodegroup_id": "parent-ng",
                    "grouping_node_id": "geometry-id"
                }
            ],
            "edges": [
                {"edgeid": "edge-1", "domainnode_id": "root-id", "rangenode_id": "location-data-id"},
                {"edgeid": "edge-2", "domainnode_id": "location-data-id", "rangenode_id": "geometry-id"},
                {"edgeid": "edge-3", "domainnode_id": "geometry-id", "rangenode_id": "geospatial-id"}
            ]
        });

        let mut graph: StaticGraph = serde_json::from_value(graph_json)
            .expect("Failed to deserialize graph");
        graph.build_indices();

        // Create a tile ONLY for the child nodegroup (child-ng), not for parent-ng
        let mut tile = StaticTile::new_empty("child-ng".to_string());
        tile.resourceinstance_id = "test-resource".to_string();
        tile.tileid = Some("tile-1".to_string());
        tile.data.insert(
            "geospatial-id".to_string(),
            serde_json::json!({
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [151.84, -26.54]
                }
            })
        );

        let business_data = serde_json::json!({
            "business_data": {
                "resources": [{
                    "resourceinstance": {
                        "resourceinstanceid": "test-resource",
                        "graph_id": "test-graph",
                        "name": "Test Resource",
                        "descriptors": {}
                    },
                    "tiles": [tile]
                }]
            }
        });

        // Debug: check what nodes are in the child nodegroup
        let nodes_in_child_ng = graph.get_nodes_in_nodegroup("child-ng");
        println!("Nodes in child-ng: {:?}", nodes_in_child_ng.iter().map(|n| &n.alias).collect::<Vec<_>>());

        // Debug: check edges
        let edges = graph.edges_map().unwrap();
        println!("Edges: {:?}", edges);

        // Debug: check nodes_by_alias
        let nodes_by_alias = graph.nodes_by_alias_arc().unwrap();
        println!("Nodes by alias: {:?}", nodes_by_alias.keys().collect::<Vec<_>>());

        let tree = tiles_to_tree(&business_data, &graph)
            .expect("tiles_to_tree failed");

        println!("Tree output:\n{}", serde_json::to_string_pretty(&tree).unwrap());

        let resources = tree.as_array().expect("Should be array");
        assert_eq!(resources.len(), 1);

        let resource = &resources[0];

        // The key test: location_data should appear even though it has no tile
        assert!(
            resource.get("location_data").is_some(),
            "location_data should appear in output even without its own tile. Got: {}",
            serde_json::to_string_pretty(resource).unwrap()
        );

        // And geometry should be nested under it
        let location_data = resource.get("location_data").unwrap();
        assert!(
            location_data.get("geometry").is_some(),
            "geometry should be nested under location_data. Got: {}",
            serde_json::to_string_pretty(location_data).unwrap()
        );
    }

    #[test]
    fn test_tree_to_tiles_with_id_key_deterministic() {
        let graph = load_group_graph();

        // Tree without resourceinstanceid
        let tree = serde_json::json!({
            "basic_info": [{
                "name": {"en": "Test"},
                "description": "Test description"
            }]
        });

        // Same id_key should produce same resourceinstanceid
        let result1 = tree_to_tiles_with_id_key(&tree, &graph, Some("my-unique-key"))
            .expect("First conversion failed");
        let result2 = tree_to_tiles_with_id_key(&tree, &graph, Some("my-unique-key"))
            .expect("Second conversion failed");

        let id1 = &result1.business_data.resources[0].resourceinstance.resourceinstanceid;
        let id2 = &result2.business_data.resources[0].resourceinstance.resourceinstanceid;

        assert_eq!(id1, id2, "Same id_key should produce same resourceinstanceid");

        // Different id_key should produce different resourceinstanceid
        let result3 = tree_to_tiles_with_id_key(&tree, &graph, Some("different-key"))
            .expect("Third conversion failed");
        let id3 = &result3.business_data.resources[0].resourceinstance.resourceinstanceid;

        assert_ne!(id1, id3, "Different id_key should produce different resourceinstanceid");

        // No id_key should produce random UUID (different each time is probabilistic)
        let result4 = tree_to_tiles_with_id_key(&tree, &graph, None)
            .expect("Fourth conversion failed");
        let id4 = &result4.business_data.resources[0].resourceinstance.resourceinstanceid;

        // Just verify it's a valid UUID format
        assert!(uuid::Uuid::parse_str(id4).is_ok(), "Should be valid UUID");
    }

    #[test]
    fn test_tree_to_tiles_explicit_id_takes_precedence() {
        let graph = load_group_graph();

        // Tree WITH explicit resourceinstanceid
        let tree = serde_json::json!({
            "resourceinstanceid": "explicit-id-123",
            "basic_info": [{
                "name": {"en": "Test"},
                "description": "Test description"
            }]
        });

        // id_key should be ignored when resourceinstanceid is present
        let result = tree_to_tiles_with_id_key(&tree, &graph, Some("ignored-key"))
            .expect("Conversion failed");
        let id = &result.business_data.resources[0].resourceinstance.resourceinstanceid;

        assert_eq!(id, "explicit-id-123", "Explicit resourceinstanceid should take precedence");
    }
}
