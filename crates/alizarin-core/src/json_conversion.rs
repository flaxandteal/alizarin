/// Hierarchical tree conversion for resources
///
/// This module provides bidirectional conversion between:
/// - **Tiled format**: Flat list of tiles grouped by nodegroup (Arches native format)
/// - **Tree format**: Nested hierarchical JSON using node aliases as keys (developer-friendly)
///
/// These are NOT simple serialization functions - they perform structural transformation:
/// - `tiles_to_tree()`: Flat tiles → Nested hierarchy
/// - `tree_to_tiles()`: Nested hierarchy → Flat tiles

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::{Value, Map};
use serde::{Serialize, Deserialize};

use crate::graph::StaticGraph;
use crate::pseudo_value_core::{
    PseudoValueCore, PseudoListCore, TileBuilder, TileBuilderContext, VisitorContext,
};
use crate::{StaticTile, StaticNode};
use crate::type_coercion::coerce_value;

/// Container for resource data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub resourceinstanceid: String,
    pub graph_id: String,
    pub tiles: Vec<StaticTile>,
}

/// Convert tiled resource format to nested tree structure
///
/// **Structural transformation** (not just serialization):
/// - Input: Flat list of tiles grouped by nodegroup_id
/// - Output: Nested JSON tree using node aliases as keys
pub fn tiles_to_tree(resource: &ResourceData, graph: &StaticGraph) -> Result<Value, String> {
    let nodes_by_alias = graph.nodes_by_alias_arc()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;

    let edges = graph.edges_map()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;

    // Build pseudo_cache from tiles
    let pseudo_cache = build_pseudo_cache_from_tiles(
        &resource.tiles,
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

    let ctx = VisitorContext {
        pseudo_cache: &full_cache,
        nodes_by_alias: &nodes_by_alias,
        edges: &edges,
        depth: 0,
        max_depth: 50,
    };

    if let Some(root_value) = root_list.values.first() {
        Ok(root_value.to_json(&ctx))
    } else {
        Ok(Value::Object(Map::new()))
    }
}

/// Build pseudo_cache from tiles
fn build_pseudo_cache_from_tiles(
    tiles: &[StaticTile],
    nodes_by_alias: &HashMap<String, Arc<StaticNode>>,
    graph: &StaticGraph,
    edges: &HashMap<String, Vec<String>>,
) -> HashMap<String, PseudoListCore> {
    let mut pseudo_cache: HashMap<String, PseudoListCore> = HashMap::new();

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
    }

    pseudo_cache
}

/// Convert nested tree structure to tiled resource format
pub fn tree_to_tiles(
    json: &Value,
    graph: &StaticGraph,
    resource_id: &str,
    graph_id: &str,
) -> Result<ResourceData, String> {
    tree_to_tiles_internal(json, graph, resource_id, graph_id, false)
}

/// Convert with strict validation (fails on unknown fields)
pub fn tree_to_tiles_strict(
    json: &Value,
    graph: &StaticGraph,
    resource_id: &str,
    graph_id: &str,
) -> Result<ResourceData, String> {
    tree_to_tiles_internal(json, graph, resource_id, graph_id, true)
}

fn tree_to_tiles_internal(
    json: &Value,
    graph: &StaticGraph,
    resource_id: &str,
    graph_id: &str,
    strict: bool,
) -> Result<ResourceData, String> {
    let obj = json.as_object()
        .ok_or_else(|| "JSON must be an object".to_string())?;

    let resource_id = resource_id.to_string();
    let graph_id = graph_id.to_string();

    let nodes_by_alias = graph.nodes_by_alias_arc()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;
    let edges = graph.edges_map()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;

    // Strict mode validation
    if strict {
        let known_metadata = ["resourceinstanceid", "graph_id", "legacyid"];
        for key in obj.keys() {
            if !known_metadata.contains(&key.as_str()) && !nodes_by_alias.contains_key(key) {
                return Err(format!("Unknown field '{}' not found in graph aliases", key));
            }
        }
    }

    let mut pseudo_cache: HashMap<String, PseudoListCore> = HashMap::new();
    let mut tile_counter: u32 = 0;

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
        &mut tile_counter,
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

    let all_tiles: Vec<StaticTile> = tiles_map.values()
        .map(|builder| builder.to_static_tile())
        .collect();

    Ok(ResourceData {
        resourceinstanceid: resource_id,
        graph_id,
        tiles: all_tiles,
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
    tile_counter: &mut u32,
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
                            tile_counter,
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
                            tile_counter,
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
                        tile_counter,
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
                tile_counter,
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
                tile_counter,
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
                tile_counter,
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
    tile_counter: &mut u32,
) -> (PseudoValueCore, Arc<StaticTile>) {
    let tile = shared_tile.unwrap_or_else(|| {
        *tile_counter += 1;
        let tile_id = format!("new_tile_{}", tile_counter);

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
    tile_counter: &mut u32,
) -> (PseudoValueCore, Arc<StaticTile>) {
    let tile = shared_tile.unwrap_or_else(|| {
        *tile_counter += 1;
        let tile_id = format!("new_tile_{}", tile_counter);

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

    fn create_test_resource(graph: &StaticGraph) -> ResourceData {
        let mut tiles = Vec::new();

        let basic_info_ng = graph.nodegroups
            .iter()
            .find(|ng| {
                graph.nodes
                    .iter()
                    .any(|n| n.alias.as_deref() == Some("basic_info")
                          && n.nodegroup_id.as_ref() == Some(&ng.nodegroupid))
            })
            .expect("Could not find basic_info nodegroup");

        let mut tile = StaticTile::new_empty(basic_info_ng.nodegroupid.clone());
        tile.resourceinstance_id = "test-resource-123".to_string();

        if let Some(name_node) = graph.nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("name")) {
            tile.data.insert(
                name_node.nodeid.clone(),
                serde_json::json!({"en": "Test Group", "ga": "Grúpa Tástála"})
            );
        }

        if let Some(desc_node) = graph.nodes
            .iter()
            .find(|n| n.alias.as_deref() == Some("description")) {
            tile.data.insert(
                desc_node.nodeid.clone(),
                serde_json::json!("A test group for unit testing")
            );
        }

        tiles.push(tile);

        ResourceData {
            resourceinstanceid: "test-resource-123".to_string(),
            graph_id: graph.graphid.clone(),
            tiles,
        }
    }

    #[test]
    fn test_tiles_to_tree_basic() {
        let graph = load_group_graph();
        let resource = create_test_resource(&graph);

        let tree = tiles_to_tree(&resource, &graph)
            .expect("tiles_to_tree failed");

        assert!(tree.is_object(), "Result should be an object");
        assert!(tree.get("resourceinstanceid").is_none(), "Should not include resourceinstanceid");
        assert!(tree.get("graph_id").is_none(), "Should not include graph_id");
    }

    #[test]
    fn test_tree_to_tiles_basic() {
        let graph = load_group_graph();

        let tree = serde_json::json!({
            "basic_info": [{
                "name": {"en": "JSON Test Group", "ga": "Grúpa Tástála JSON"},
                "description": "Created from JSON tree"
            }]
        });

        let resource_id = "test-resource-456";
        let graph_id = &graph.graphid;

        let resource = tree_to_tiles(&tree, &graph, resource_id, graph_id)
            .expect("tree_to_tiles failed");

        assert_eq!(resource.resourceinstanceid, "test-resource-456");
        assert_eq!(resource.graph_id, graph.graphid);
        assert!(!resource.tiles.is_empty(), "Should have created tiles");
    }
}
