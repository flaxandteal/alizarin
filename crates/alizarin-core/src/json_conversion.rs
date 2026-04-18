use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cell::RefCell;
/// Hierarchical tree conversion for resources
///
/// This module provides bidirectional conversion between:
/// - **Tiled format**: `{"business_data": {"resources": [StaticResource, ...]}}` (Arches export format)
/// - **Tree format**: Array of nested hierarchical JSON using node aliases as keys `[{...}, {...}]`
///
/// These are NOT simple serialization functions - they perform structural transformation:
/// - `tiles_to_tree()`: Tiled resources → Array of nested tree objects
/// - `tree_to_tiles()`: Array of nested tree objects → Tiled resources with business_data wrapper
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::extension_type_registry::ExtensionTypeRegistry;
use crate::graph::{IndexedGraph, StaticGraph};
use crate::graph::{StaticResource, StaticResourceMetadata};
use crate::graph_mutator::generate_uuid_v5;
use crate::instance_wrapper_core::is_node_single_cardinality_with;
use crate::pseudo_value_core::{
    PseudoListCore, PseudoValueCore, TileBuilder, TileBuilderContext, VisitorContext,
};
use crate::registry::is_list_datatype;
use crate::string_utils::snake_to_camel;
use crate::type_coercion::coerce_value_with_registry;
use crate::{StaticNode, StaticTile};

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
    let name = descriptors
        .name
        .clone()
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
        let tiles = resource
            .tiles
            .as_ref()
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
    let nodes_by_alias = graph
        .nodes_by_alias_arc()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;

    let edges = graph
        .edges_map()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;

    // Build pseudo_cache from tiles
    let pseudo_cache = build_pseudo_cache_from_tiles(tiles, nodes_by_alias, graph, edges);

    // Create root pseudo value
    let root = graph.root_node();
    let root_alias = root.alias.clone().unwrap_or_default();

    let child_node_ids = graph
        .get_child_ids(&root.nodeid)
        .cloned()
        .unwrap_or_default();

    let root_pseudo =
        PseudoValueCore::from_node_and_tile(Arc::new(root.clone()), None, None, child_node_ids);

    let root_list =
        PseudoListCore::from_values_with_cardinality(root_alias.clone(), vec![root_pseudo], true);

    let mut full_cache = pseudo_cache;
    full_cache.insert(root_alias.clone(), root_list.clone());

    let ctx = VisitorContext::new(&full_cache, nodes_by_alias, edges);

    let mut tree = if let Some(root_value) = root_list.values.first() {
        root_value.to_json(&ctx)
    } else {
        Value::Object(Map::new())
    };

    // Add metadata to tree
    if let Some(obj) = tree.as_object_mut() {
        obj.insert(
            "resourceinstanceid".to_string(),
            Value::String(metadata.resourceinstanceid.clone()),
        );
        obj.insert(
            "graph_id".to_string(),
            Value::String(metadata.graph_id.clone()),
        );
        if let Some(ref name) = metadata.descriptors.name {
            obj.insert("_name".to_string(), Value::String(name.clone()));
        }
        if let Some(ref desc) = metadata.descriptors.description {
            obj.insert("_description".to_string(), Value::String(desc.clone()));
        }
        if let Some(ref slug) = metadata.descriptors.slug {
            obj.insert("_slug".to_string(), Value::String(slug.clone()));
        }
        match metadata.legacyid {
            Some(ref legacyid) => {
                obj.insert("legacyid".to_string(), Value::String(legacyid.clone()));
            }
            None => {
                obj.insert("legacyid".to_string(), Value::Null);
            }
        }
    }

    Ok(tree)
}

/// Extract resources from input (handles both wrapper format and single resource)
pub(crate) fn extract_resources(input: &Value) -> Result<Vec<StaticResource>, String> {
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
pub(crate) fn build_pseudo_cache_from_tiles(
    tiles: &[StaticTile],
    nodes_by_alias: &HashMap<String, Arc<StaticNode>>,
    graph: &StaticGraph,
    edges: &HashMap<String, Vec<String>>,
) -> HashMap<String, PseudoListCore> {
    use std::collections::HashSet;

    let mut pseudo_cache: HashMap<String, PseudoListCore> = HashMap::new();

    // Sort tiles by sortorder to ensure consistent ordering (lowest first = primary)
    let mut sorted_tiles: Vec<&StaticTile> = tiles.iter().collect();
    sorted_tiles.sort_by_key(|t| t.sortorder.unwrap_or(i32::MAX));

    // Track which nodegroups have tiles
    let nodegroups_with_tiles: HashSet<&str> =
        tiles.iter().map(|t| t.nodegroup_id.as_str()).collect();

    for tile in sorted_tiles {
        let tile_arc = Arc::new(tile.clone());

        let nodes_in_ng = graph.get_nodes_in_nodegroup(&tile.nodegroup_id);

        for node in nodes_in_ng {
            let alias = match &node.alias {
                Some(a) if !a.is_empty() => a.clone(),
                _ => continue,
            };

            let child_node_ids = edges.get(&node.nodeid).cloned().unwrap_or_default();

            let tile_data = tile.data.get(&node.nodeid).cloned();

            let node_arc = nodes_by_alias
                .get(&alias)
                .map(Arc::clone)
                .unwrap_or_else(|| Arc::new(node.clone()));

            let pv = PseudoValueCore::from_node_and_tile(
                node_arc,
                Some(Arc::clone(&tile_arc)),
                tile_data,
                child_node_ids,
            );

            // Use shared cardinality logic
            let is_single = is_node_single_cardinality_with(node, |ng_id| {
                graph
                    .get_nodegroup_by_id(ng_id)
                    .and_then(|ng| ng.cardinality.clone())
            });

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
                    PseudoListCore::from_values_with_cardinality(alias.clone(), vec![pv], is_single)
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
            let grouping_node_id = parent_nodegroup
                .grouping_node_id
                .as_ref()
                .unwrap_or(&parent_ng_id); // Fallback to nodegroup_id if not set

            // Find the semantic collector node
            let semantic_node = graph
                .nodes_slice()
                .iter()
                .find(|n| n.nodeid == *grouping_node_id);

            if let Some(semantic_node) = semantic_node {
                if let Some(ref alias) = semantic_node.alias {
                    if !alias.is_empty() && !pseudo_cache.contains_key(alias) {
                        // Create a synthetic pseudo value for this semantic collector
                        let child_node_ids = edges
                            .get(&semantic_node.nodeid)
                            .cloned()
                            .unwrap_or_default();

                        let node_arc = nodes_by_alias
                            .get(alias)
                            .map(Arc::clone)
                            .unwrap_or_else(|| Arc::new(semantic_node.clone()));

                        // Create with no tile - this is a synthetic entry
                        let pv = PseudoValueCore::from_node_and_tile(
                            node_arc,
                            None, // No tile for synthetic collectors
                            None, // No tile data
                            child_node_ids,
                        );

                        let is_single = parent_nodegroup
                            .cardinality
                            .as_ref()
                            .map(|c| c != "n")
                            .unwrap_or(true);

                        pseudo_cache.insert(
                            alias.clone(),
                            PseudoListCore::from_values_with_cardinality(
                                alias.clone(),
                                vec![pv],
                                is_single,
                            ),
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
///
/// # Arguments
/// * `json` - Tree structure to convert
/// * `graph` - Graph definition
/// * `strict` - If true, fails on unknown fields (recommended). Defaults to true.
/// * `id_key` - Optional key for deterministic UUID v5 generation. When provided and
///   the tree does not contain a `resourceinstanceid`, a deterministic UUID v5 will
///   be generated using the key and graph ID as namespace.
pub fn tree_to_tiles(
    json: &Value,
    graph: &StaticGraph,
    strict: bool,
    id_key: Option<&str>,
) -> Result<BusinessDataWrapper, String> {
    tree_to_tiles_with_options(json, graph, strict, id_key, false, true, false, None)
}

/// Convert tree to tiles with camelCase key support.
///
/// When `from_camel` is true, tree keys like `associatedActors` will match
/// node aliases like `associated_actors`. Value structures (like `{"resourceId": "..."}`)
/// are preserved unchanged.
/// * `has_extension_handlers` - If true, unknown datatypes are allowed through (extensions
///   will handle them post-coercion). If false in strict mode, unknown datatypes produce errors.
/// * `extension_registry` - Optional extension type registry for coercing extension types.
///   When provided, extension handlers are consulted for unknown datatypes during coercion.
#[allow(clippy::too_many_arguments)]
pub fn tree_to_tiles_with_options(
    json: &Value,
    graph: &StaticGraph,
    strict: bool,
    id_key: Option<&str>,
    from_camel: bool,
    random_ids: bool,
    has_extension_handlers: bool,
    extension_registry: Option<&ExtensionTypeRegistry>,
) -> Result<BusinessDataWrapper, String> {
    let trees = extract_tree_resources(json)?;

    let mut resources = Vec::new();

    for tree in trees {
        let resource = single_tree_to_resource(
            &tree,
            graph,
            strict,
            id_key,
            from_camel,
            random_ids,
            has_extension_handlers,
            extension_registry,
        )?;
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
#[allow(clippy::too_many_arguments)]
fn single_tree_to_resource(
    json: &Value,
    graph: &StaticGraph,
    strict: bool,
    id_key: Option<&str>,
    from_camel: bool,
    random_ids: bool,
    has_extension_handlers: bool,
    extension_registry: Option<&ExtensionTypeRegistry>,
) -> Result<StaticResource, String> {
    let obj = json
        .as_object()
        .ok_or_else(|| "JSON must be an object".to_string())?;

    // Extract metadata from tree
    // Priority: explicit resourceinstanceid > id_key (UUID v5) > slug (UUID v5) > random UUID v4
    let explicit_id = obj
        .get("resourceinstanceid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let id_from_key = id_key.map(|key| generate_uuid_v5(("resource", Some(&graph.graphid)), key));
    let needs_slug_id = explicit_id.is_none() && id_from_key.is_none() && !random_ids;

    // Use temp ID if slug-based; will be replaced after building tiles
    let resource_id = explicit_id
        .clone()
        .or(id_from_key)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let graph_id = obj
        .get("graph_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| graph.graphid.clone());

    let legacyid = obj
        .get("legacyid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let nodes_by_alias = graph
        .nodes_by_alias_arc()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;
    let edges = graph
        .edges_map()
        .ok_or_else(|| "Graph indices not built - call build_indices() first".to_string())?;

    // Strict mode validation
    let known_metadata = [
        "resourceinstanceid",
        "graph_id",
        "legacyid",
        "_name",
        "_description",
        "_map_popup",
        "_slug",
    ];
    if strict {
        for key in obj.keys() {
            // Check both exact key and snake_case version (for camelCase input)
            let key_matches = known_metadata.contains(&key.as_str())
                || nodes_by_alias.contains_key(key)
                || (from_camel && nodes_by_alias.contains_key(&crate::camel_to_snake(key)));
            if !key_matches {
                return Err(format!(
                    "Unknown field '{}' not found in graph aliases",
                    key
                ));
            }
        }
    }

    let mut pseudo_cache: HashMap<String, PseudoListCore> = HashMap::new();

    let root = graph.root_node();
    build_pseudo_values_from_json(
        obj,
        &Arc::new(root.clone()),
        nodes_by_alias,
        graph,
        edges,
        &resource_id,
        None,
        &mut pseudo_cache,
        strict,
        from_camel,
        has_extension_handlers,
        extension_registry,
    )?;

    // Track visited aliases to prevent O(n²) duplicate traversal
    let visited_aliases = RefCell::new(HashSet::new());

    let ctx = TileBuilderContext {
        pseudo_cache: &pseudo_cache,
        nodes_by_alias,
        edges,
        resourceinstance_id: resource_id.clone(),
        depth: 0,
        max_depth: 100,
        visited_aliases: &visited_aliases,
    };

    let mut tiles_map: HashMap<String, TileBuilder> = HashMap::new();

    if let Some(child_ids) = edges.get(&root.nodeid) {
        for child_id in child_ids {
            let child_node = nodes_by_alias.values().find(|n| n.nodeid == *child_id);

            if let Some(child_node) = child_node {
                if let Some(alias) = &child_node.alias {
                    // Mark as visited before processing (prevents re-traversal from children)
                    visited_aliases.borrow_mut().insert(alias.clone());
                    if let Some(pseudo_list) = pseudo_cache.get(alias) {
                        pseudo_list.collect_tiles(&ctx, &mut tiles_map);
                    }
                }
            }
        }
    }

    let mut tiles: Vec<StaticTile> = tiles_map
        .values()
        .map(|builder| builder.to_static_tile())
        .collect();

    // Calculate descriptors from tiles
    let indexed = IndexedGraph::new(graph.clone());
    let descriptors =
        indexed.build_descriptors_with_context(&tiles, &mut Vec::new(), None, extension_registry);

    // If no explicit ID or id_key, derive resource ID from slug descriptor
    let resource_id = if needs_slug_id {
        let slug = descriptors.slug.as_ref().ok_or_else(|| {
            "No slug descriptor configured but slug-based ID generation requested. \
             Configure a slug template, provide id_keys, or set random_ids=true."
                .to_string()
        })?;
        // Detect unresolved <Placeholder> in the slug (template evaluation failed silently)
        if slug.contains('<') {
            return Err(format!(
                "Slug contains unresolved placeholder(s): '{}'. \
                 Check that all <Node Name> references in the slug template have matching tile data.",
                slug
            ));
        }
        let real_id = generate_uuid_v5(("resource", Some(&graph.graphid)), slug);
        for tile in &mut tiles {
            tile.resourceinstance_id = real_id.clone();
        }
        real_id
    } else {
        resource_id
    };

    // Use name from descriptors, or fallback to resourceinstanceid
    let name = descriptors
        .name
        .clone()
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
#[allow(clippy::too_many_arguments)]
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
    from_camel: bool,
    has_extension_handlers: bool,
    extension_registry: Option<&ExtensionTypeRegistry>,
) -> Result<(), String> {
    let child_ids = edges.get(&current_node.nodeid).cloned().unwrap_or_default();

    for child_id in child_ids {
        let child_node = nodes_by_alias.values().find(|n| n.nodeid == child_id);

        let child_node = match child_node {
            Some(n) => Arc::clone(n),
            None => continue,
        };

        let child_alias = match &child_node.alias {
            Some(a) if !a.is_empty() => a.clone(),
            _ => continue,
        };

        // Try exact alias first, then camelCase version if from_camel is true
        let json_value = json_obj.get(&child_alias).or_else(|| {
            if from_camel {
                let camel_key = snake_to_camel(&child_alias);
                json_obj.get(&camel_key)
            } else {
                None
            }
        });

        let json_value = match json_value {
            Some(v) => v,
            None => continue,
        };

        let nodegroup_id = child_node
            .nodegroup_id
            .as_ref()
            .ok_or_else(|| format!("Node {} has no nodegroup_id", child_id))?;

        // Use shared cardinality logic
        let is_single = is_node_single_cardinality_with(&child_node, |ng_id| {
            graph
                .get_nodegroup_by_id(ng_id)
                .and_then(|ng| ng.cardinality.clone())
        });

        let config_value = if child_node.config.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(
                child_node
                    .config
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ))
        };

        let child_child_ids = edges.get(&child_node.nodeid).cloned().unwrap_or_default();

        let shares_nodegroup = parent_tile
            .as_ref()
            .map(|pt| pt.nodegroup_id == *nodegroup_id)
            .unwrap_or(false);

        // Extract parent tile ID for setting parenttile_id on new tiles
        // Only needed when NOT sharing nodegroup (creating a new tile)
        let parent_tile_id = if shares_nodegroup {
            None
        } else {
            parent_tile.as_ref().and_then(|pt| pt.tileid.as_ref())
        };

        let has_graph_children = !child_child_ids.is_empty();
        let mut values: Vec<PseudoValueCore> = Vec::new();

        // Check if this is an "outer" node: has children but also has its own non-semantic datatype
        // Outer nodes use "_" key for their own value
        let is_outer_node = has_graph_children && child_node.datatype != "semantic";

        let valid_child_aliases: std::collections::HashSet<&str> = if strict && has_graph_children {
            let mut aliases: std::collections::HashSet<&str> = child_child_ids
                .iter()
                .filter_map(|id| nodes_by_alias.values().find(|n| n.nodeid == *id))
                .filter_map(|n| n.alias.as_deref())
                .collect();
            // Allow "_" for outer nodes (nodes with their own value plus children)
            if is_outer_node {
                aliases.insert("_");
            }
            aliases
        } else {
            std::collections::HashSet::new()
        };

        // For list-type datatypes (concept-list, resource-instance-list, reference, etc.),
        // the array IS the value - don't iterate over it.
        // Extensions register their list datatypes via register_list_datatype().
        let is_list_type = is_list_datatype(&child_node.datatype);

        if json_value.is_array() && !is_list_type {
            // SAFETY: is_array() check guarantees as_array() succeeds
            let array = json_value.as_array().expect("checked is_array()");
            for (idx, item) in array.iter().enumerate() {
                // Use array index as sortorder (0 = primary/first)
                let sortorder = Some(idx as i32);

                if has_graph_children {
                    if let Some(item_obj) = item.as_object() {
                        if strict {
                            for key in item_obj.keys() {
                                // Check both exact key and snake_case version (for camelCase input)
                                let key_matches = valid_child_aliases.contains(key.as_str())
                                    || (from_camel
                                        && valid_child_aliases
                                            .contains(crate::camel_to_snake(key).as_str()));
                                if !key_matches {
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
                            if shares_nodegroup {
                                parent_tile.clone()
                            } else {
                                None
                            },
                            parent_tile_id,
                            sortorder,
                            extension_registry,
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
                            from_camel,
                            has_extension_handlers,
                            extension_registry,
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
                        if shares_nodegroup {
                            parent_tile.clone()
                        } else {
                            None
                        },
                        parent_tile_id,
                        sortorder,
                        strict,
                        has_extension_handlers,
                        extension_registry,
                    )?;
                    values.push(pv);
                }
            }
        } else if has_graph_children && json_value.is_object() {
            // SAFETY: is_object() check guarantees as_object() succeeds
            let item_obj = json_value.as_object().expect("checked is_object()");

            if strict {
                for key in item_obj.keys() {
                    // Check both exact key and snake_case version (for camelCase input)
                    let key_matches = valid_child_aliases.contains(key.as_str())
                        || (from_camel
                            && valid_child_aliases.contains(crate::camel_to_snake(key).as_str()));
                    if !key_matches {
                        return Err(format!(
                            "Unknown field '{}' in '{}' - valid fields: {:?}",
                            key, child_alias, valid_child_aliases
                        ));
                    }
                }
            }

            // Single value gets sortorder 0 (primary)
            let (pv, tile) = create_pseudo_value_from_json(
                item_obj,
                &child_node,
                &child_child_ids,
                nodegroup_id,
                &config_value,
                resource_id,
                if shares_nodegroup {
                    parent_tile.clone()
                } else {
                    None
                },
                parent_tile_id,
                Some(0),
                extension_registry,
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
                from_camel,
                has_extension_handlers,
                extension_registry,
            )?;
        } else {
            // Single leaf value gets sortorder 0 (primary)
            let (pv, _tile) = create_pseudo_value_from_leaf(
                json_value,
                &child_node,
                &child_child_ids,
                nodegroup_id,
                &config_value,
                resource_id,
                if shares_nodegroup {
                    parent_tile.clone()
                } else {
                    None
                },
                parent_tile_id,
                Some(0),
                strict,
                has_extension_handlers,
                extension_registry,
            )?;
            values.push(pv);
        }

        if !values.is_empty() {
            let list = PseudoListCore::from_values_with_cardinality(
                child_alias.clone(),
                values,
                is_single,
            );

            pseudo_cache
                .entry(child_alias)
                .and_modify(|existing| existing.merge(list.clone()))
                .or_insert(list);
        }
    }

    Ok(())
}

/// Create a PseudoValueCore from a JSON object
#[allow(clippy::too_many_arguments)]
fn create_pseudo_value_from_json(
    json_obj: &Map<String, Value>,
    node: &Arc<StaticNode>,
    child_node_ids: &[String],
    nodegroup_id: &str,
    config: &Option<Value>,
    resource_id: &str,
    shared_tile: Option<Arc<StaticTile>>,
    parent_tile_id: Option<&String>,
    sortorder: Option<i32>,
    extension_registry: Option<&ExtensionTypeRegistry>,
) -> (PseudoValueCore, Arc<StaticTile>) {
    let tile = shared_tile.unwrap_or_else(|| {
        let tile_id = uuid::Uuid::new_v4().to_string();

        let mut new_tile = StaticTile::new_empty(nodegroup_id.to_string());
        new_tile.tileid = Some(tile_id);
        new_tile.resourceinstance_id = resource_id.to_string();
        new_tile.parenttile_id = parent_tile_id.cloned();
        new_tile.sortorder = sortorder;
        Arc::new(new_tile)
    });

    // "_" key holds the outer node's own value (for nodes with both value and semantic children)
    let tile_data = json_obj
        .get("_")
        .map(|value| {
            let coerced = coerce_value_with_registry(
                &node.datatype,
                value,
                config.as_ref(),
                extension_registry,
            );
            if !coerced.is_null() && coerced.error.is_none() {
                coerced.tile_data
            } else {
                Value::Null
            }
        })
        .filter(|v| !v.is_null());

    let pv = PseudoValueCore::from_node_and_tile(
        Arc::clone(node),
        Some(Arc::clone(&tile)),
        tile_data,
        child_node_ids.to_vec(),
    );

    (pv, tile)
}

/// Create a PseudoValueCore from a leaf JSON value
#[allow(clippy::too_many_arguments)]
fn create_pseudo_value_from_leaf(
    json_value: &Value,
    node: &Arc<StaticNode>,
    child_node_ids: &[String],
    nodegroup_id: &str,
    config: &Option<Value>,
    resource_id: &str,
    shared_tile: Option<Arc<StaticTile>>,
    parent_tile_id: Option<&String>,
    sortorder: Option<i32>,
    strict: bool,
    has_extension_handlers: bool,
    extension_registry: Option<&ExtensionTypeRegistry>,
) -> Result<(PseudoValueCore, Arc<StaticTile>), String> {
    let tile = shared_tile.unwrap_or_else(|| {
        let tile_id = uuid::Uuid::new_v4().to_string();

        let mut new_tile = StaticTile::new_empty(nodegroup_id.to_string());
        new_tile.tileid = Some(tile_id);
        new_tile.resourceinstance_id = resource_id.to_string();
        new_tile.parenttile_id = parent_tile_id.cloned();
        new_tile.sortorder = sortorder;
        Arc::new(new_tile)
    });

    // Check for "_" or "_value" key (outer node value conventions)
    let value_to_coerce = json_value
        .as_object()
        .and_then(|obj| obj.get("_").or_else(|| obj.get("_value")))
        .unwrap_or(json_value);

    let coerced = coerce_value_with_registry(
        &node.datatype,
        value_to_coerce,
        config.as_ref(),
        extension_registry,
    );

    // In strict mode, report coercion errors
    if strict {
        if let Some(ref error) = coerced.error {
            let node_alias = node.alias.as_deref().unwrap_or(&node.nodeid);
            return Err(format!(
                "Coercion error for '{}' ({}): {}",
                node_alias, node.datatype, error
            ));
        }
        if coerced.passthrough && !has_extension_handlers {
            let node_alias = node.alias.as_deref().unwrap_or(&node.nodeid);
            return Err(format!(
                "Unknown datatype '{}' for node '{}'. If this is an extension type \
                 (e.g. 'reference'), ensure the extension is imported and skip_extensions \
                 is not set.",
                node.datatype, node_alias
            ));
        }
    }

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

    Ok((pv, tile))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn load_group_graph() -> StaticGraph {
        // Use workspace root for test data
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let test_file = workspace_root.join("tests/data/models/Group.json");
        let json_str = fs::read_to_string(&test_file).expect("Failed to read Group.json");
        let json: serde_json::Value =
            serde_json::from_str(&json_str).expect("Failed to parse Group.json");

        let graph_data = json["graph"][0].clone();

        let mut core_graph: StaticGraph =
            serde_json::from_value(graph_data).expect("Failed to deserialize StaticGraph");
        core_graph.build_indices();

        core_graph
    }

    fn create_test_business_data(graph: &StaticGraph) -> BusinessDataWrapper {
        let mut tiles = Vec::new();

        let basic_info_ng = graph
            .nodegroups_slice()
            .iter()
            .find(|ng| {
                graph.nodes_slice().iter().any(|n| {
                    n.alias.as_deref() == Some("basic_info")
                        && n.nodegroup_id.as_ref() == Some(&ng.nodegroupid)
                })
            })
            .expect("Could not find basic_info nodegroup");

        let mut tile = StaticTile::new_empty(basic_info_ng.nodegroupid.clone());
        tile.resourceinstance_id = "test-resource-123".to_string();
        tile.tileid = Some("test-tile-1".to_string());

        if let Some(name_node) = graph
            .nodes_slice()
            .iter()
            .find(|n| n.alias.as_deref() == Some("name"))
        {
            tile.data.insert(
                name_node.nodeid.clone(),
                serde_json::json!({"en": "Test Group", "ga": "Grúpa Tástála"}),
            );
        }

        if let Some(desc_node) = graph
            .nodes_slice()
            .iter()
            .find(|n| n.alias.as_deref() == Some("description"))
        {
            tile.data.insert(
                desc_node.nodeid.clone(),
                serde_json::json!("A test group for unit testing"),
            );
        }

        tiles.push(tile);

        // Calculate descriptors
        let indexed = IndexedGraph::new(graph.clone());
        let descriptors = indexed.build_descriptors(&tiles);
        let name = descriptors
            .name
            .clone()
            .unwrap_or_else(|| "test-resource-123".to_string());

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
        let tree = tiles_to_tree(&input, &graph).expect("tiles_to_tree failed");

        assert!(tree.is_array(), "Result should be an array");

        let resources = tree.as_array().unwrap();
        assert_eq!(resources.len(), 1, "Should have one resource");

        let resource_tree = &resources[0];
        // Tree format has resourceinstanceid and graph_id at root level (not nested in resourceinstance)
        assert!(
            resource_tree.get("resourceinstanceid").is_some(),
            "Should include resourceinstanceid"
        );
        assert!(
            resource_tree.get("graph_id").is_some(),
            "Should include graph_id"
        );
    }

    #[test]
    fn test_tree_to_tiles_array() {
        let graph = load_group_graph();

        // Input: array of tree objects
        // Note: description is in statement nodegroup, not basic_info
        let trees = serde_json::json!([{
            "resourceinstanceid": "test-resource-456",
            "graph_id": graph.graphid,
            "basic_info": [{
                "name": {"en": "JSON Test Group", "ga": "Grúpa Tástála JSON"}
            }],
            "statement": [{
                "description": "Created from JSON tree"
            }]
        }]);

        let result = tree_to_tiles(&trees, &graph, true, None).expect("tree_to_tiles failed");

        assert_eq!(result.business_data.resources.len(), 1);
        let resource = &result.business_data.resources[0];
        assert_eq!(
            resource.resourceinstance.resourceinstanceid,
            "test-resource-456"
        );
        assert_eq!(resource.resourceinstance.graph_id, graph.graphid);
        assert!(
            resource
                .tiles
                .as_ref()
                .map(|t| !t.is_empty())
                .unwrap_or(false),
            "Should have created tiles"
        );
    }

    #[test]
    fn test_tree_to_tiles_single_object() {
        let graph = load_group_graph();

        // Input: single tree object (no array wrapper)
        // Note: description is in statement nodegroup, not basic_info
        let tree = serde_json::json!({
            "resourceinstanceid": "test-resource-789",
            "graph_id": graph.graphid,
            "basic_info": [{
                "name": {"en": "Single Resource Test"}
            }],
            "statement": [{
                "description": "Testing single resource input"
            }]
        });

        let result = tree_to_tiles(&tree, &graph, true, None).expect("tree_to_tiles failed");

        assert_eq!(result.business_data.resources.len(), 1);
        let resource = &result.business_data.resources[0];
        assert_eq!(
            resource.resourceinstance.resourceinstanceid,
            "test-resource-789"
        );
    }

    #[test]
    fn test_round_trip() {
        let graph = load_group_graph();

        // Create initial tree array
        // Note: description is in statement nodegroup, not basic_info
        let initial_trees = serde_json::json!([{
            "resourceinstanceid": "round-trip-test",
            "graph_id": graph.graphid,
            "basic_info": [{
                "name": {"en": "Round Trip Test", "ga": "Tástáil Timpeall"}
            }],
            "statement": [{
                "description": "Testing round trip conversion"
            }]
        }]);

        // Convert to tiles (business_data format)
        let tiles_result =
            tree_to_tiles(&initial_trees, &graph, true, None).expect("tree_to_tiles failed");

        // Convert back to tree array
        let tiles_json = serde_json::to_value(&tiles_result).unwrap();
        let tree_result = tiles_to_tree(&tiles_json, &graph).expect("tiles_to_tree failed");

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

        let result = tree_to_tiles(&tree, &graph, true, None).expect("tree_to_tiles failed");

        // Extract first resource
        let resource = &result.business_data.resources[0];

        // Serialize to JSON to see the format
        let json = serde_json::to_string_pretty(resource).unwrap();
        println!("Serialized StaticResource:\n{}", json);

        // Parse back and check structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Check that resourceinstance is nested (not flattened)
        assert!(
            parsed.get("resourceinstance").is_some(),
            "Expected nested 'resourceinstance', got: {}",
            json
        );
        assert!(
            parsed.get("resourceinstanceid").is_none(),
            "Should NOT have 'resourceinstanceid' at root level, got: {}",
            json
        );
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

        let mut graph: StaticGraph =
            serde_json::from_value(graph_json).expect("Failed to deserialize graph");
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
            }),
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
        println!(
            "Nodes in child-ng: {:?}",
            nodes_in_child_ng
                .iter()
                .map(|n| &n.alias)
                .collect::<Vec<_>>()
        );

        // Debug: check edges
        let edges = graph.edges_map().unwrap();
        println!("Edges: {:?}", edges);

        // Debug: check nodes_by_alias
        let nodes_by_alias = graph.nodes_by_alias_arc().unwrap();
        println!(
            "Nodes by alias: {:?}",
            nodes_by_alias.keys().collect::<Vec<_>>()
        );

        let tree = tiles_to_tree(&business_data, &graph).expect("tiles_to_tree failed");

        println!(
            "Tree output:\n{}",
            serde_json::to_string_pretty(&tree).unwrap()
        );

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
        let result1 = tree_to_tiles(&tree, &graph, false, Some("my-unique-key"))
            .expect("First conversion failed");
        let result2 = tree_to_tiles(&tree, &graph, false, Some("my-unique-key"))
            .expect("Second conversion failed");

        let id1 = &result1.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;
        let id2 = &result2.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;

        assert_eq!(
            id1, id2,
            "Same id_key should produce same resourceinstanceid"
        );

        // Different id_key should produce different resourceinstanceid
        let result3 = tree_to_tiles(&tree, &graph, false, Some("different-key"))
            .expect("Third conversion failed");
        let id3 = &result3.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;

        assert_ne!(
            id1, id3,
            "Different id_key should produce different resourceinstanceid"
        );

        // No id_key should produce random UUID (different each time is probabilistic)
        let result4 = tree_to_tiles(&tree, &graph, false, None).expect("Fourth conversion failed");
        let id4 = &result4.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;

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
        let result =
            tree_to_tiles(&tree, &graph, false, Some("ignored-key")).expect("Conversion failed");
        let id = &result.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;

        assert_eq!(
            id, "explicit-id-123",
            "Explicit resourceinstanceid should take precedence"
        );
    }

    /// Test that nested semantic nodes with data children work correctly.
    /// Mimics the Heritage Item structure: system_reference_numbers -> uuid -> resourceid
    #[test]
    fn test_nested_semantic_with_data_child() {
        // Create a graph that mimics: root -> system_ref_numbers (semantic) -> uuid (semantic) -> resourceid (string)
        // All in the same nodegroup (as in Heritage Item)
        let graph_json = serde_json::json!({
            "graphid": "test-nested-graph",
            "name": {"en": "Test Nested Graph"},
            "root": {
                "nodeid": "root-id",
                "name": "Heritage Item",
                "alias": "heritage_item",
                "datatype": "semantic",
                "graph_id": "test-nested-graph",
                "istopnode": true
            },
            "nodes": [
                {
                    "nodeid": "root-id",
                    "name": "Heritage Item",
                    "alias": "heritage_item",
                    "datatype": "semantic",
                    "graph_id": "test-nested-graph",
                    "istopnode": true
                },
                {
                    "nodeid": "sys-ref-id",
                    "name": "System Reference Numbers",
                    "alias": "system_reference_numbers",
                    "datatype": "semantic",
                    "nodegroup_id": "sys-ref-ng",
                    "graph_id": "test-nested-graph",
                    "is_collector": true
                },
                {
                    "nodeid": "uuid-id",
                    "name": "UUID",
                    "alias": "uuid",
                    "datatype": "semantic",
                    "nodegroup_id": "sys-ref-ng",
                    "graph_id": "test-nested-graph"
                },
                {
                    "nodeid": "resourceid-id",
                    "name": "ResourceID",
                    "alias": "resourceid",
                    "datatype": "string",
                    "nodegroup_id": "sys-ref-ng",
                    "graph_id": "test-nested-graph"
                }
            ],
            "nodegroups": [
                {
                    "nodegroupid": "sys-ref-ng",
                    "cardinality": "n",
                    "grouping_node_id": "sys-ref-id"
                }
            ],
            "edges": [
                {"edgeid": "edge-1", "domainnode_id": "root-id", "rangenode_id": "sys-ref-id"},
                {"edgeid": "edge-2", "domainnode_id": "sys-ref-id", "rangenode_id": "uuid-id"},
                {"edgeid": "edge-3", "domainnode_id": "uuid-id", "rangenode_id": "resourceid-id"}
            ]
        });

        let mut graph: StaticGraph =
            serde_json::from_value(graph_json).expect("Failed to deserialize graph");
        graph.build_indices();

        // Test input matching Heritage Item structure
        let tree = serde_json::json!({
            "system_reference_numbers": {
                "uuid": {
                    "resourceid": "650284"
                }
            }
        });

        println!(
            "Input tree:\n{}",
            serde_json::to_string_pretty(&tree).unwrap()
        );

        let result = tree_to_tiles(&tree, &graph, true, None).expect("tree_to_tiles failed");

        let resource = &result.business_data.resources[0];
        let tiles = resource.tiles.as_ref().expect("Should have tiles");

        println!("Output tiles:");
        for tile in tiles {
            println!("  Tile nodegroup_id: {}", tile.nodegroup_id);
            println!(
                "  Tile data: {}",
                serde_json::to_string_pretty(&tile.data).unwrap()
            );
        }

        // Find the tile for sys-ref-ng
        let sys_ref_tile = tiles
            .iter()
            .find(|t| t.nodegroup_id == "sys-ref-ng")
            .expect("Should have sys-ref-ng tile");

        // The resourceid value should be in the tile data under the resourceid node ID
        let resourceid_value = sys_ref_tile.data.get("resourceid-id");
        assert!(
            resourceid_value.is_some(),
            "Tile data should contain resourceid-id. Got: {}",
            serde_json::to_string_pretty(&sys_ref_tile.data).unwrap()
        );

        // String datatype coerces to i18n format {"en": {"value": "...", "direction": "ltr"}}
        let expected = serde_json::json!({"en": {"value": "650284", "direction": "ltr"}});
        assert_eq!(
            resourceid_value.unwrap(),
            &expected,
            "resourceid should be i18n formatted '650284'. Got: {:?}",
            resourceid_value
        );
    }

    /// Test with multiple levels of nesting - cardinality "n" with array wrapper
    #[test]
    fn test_nested_semantic_with_array_wrapper() {
        let graph_json = serde_json::json!({
            "graphid": "test-nested-graph",
            "name": {"en": "Test Nested Graph"},
            "root": {
                "nodeid": "root-id",
                "name": "Heritage Item",
                "alias": "heritage_item",
                "datatype": "semantic",
                "graph_id": "test-nested-graph",
                "istopnode": true
            },
            "nodes": [
                {
                    "nodeid": "root-id",
                    "name": "Heritage Item",
                    "alias": "heritage_item",
                    "datatype": "semantic",
                    "graph_id": "test-nested-graph",
                    "istopnode": true
                },
                {
                    "nodeid": "sys-ref-id",
                    "name": "System Reference Numbers",
                    "alias": "system_reference_numbers",
                    "datatype": "semantic",
                    "nodegroup_id": "sys-ref-ng",
                    "graph_id": "test-nested-graph",
                    "is_collector": true
                },
                {
                    "nodeid": "uuid-id",
                    "name": "UUID",
                    "alias": "uuid",
                    "datatype": "semantic",
                    "nodegroup_id": "sys-ref-ng",
                    "graph_id": "test-nested-graph"
                },
                {
                    "nodeid": "resourceid-id",
                    "name": "ResourceID",
                    "alias": "resourceid",
                    "datatype": "string",
                    "nodegroup_id": "sys-ref-ng",
                    "graph_id": "test-nested-graph"
                }
            ],
            "nodegroups": [
                {
                    "nodegroupid": "sys-ref-ng",
                    "cardinality": "n",
                    "grouping_node_id": "sys-ref-id"
                }
            ],
            "edges": [
                {"edgeid": "edge-1", "domainnode_id": "root-id", "rangenode_id": "sys-ref-id"},
                {"edgeid": "edge-2", "domainnode_id": "sys-ref-id", "rangenode_id": "uuid-id"},
                {"edgeid": "edge-3", "domainnode_id": "uuid-id", "rangenode_id": "resourceid-id"}
            ]
        });

        let mut graph: StaticGraph =
            serde_json::from_value(graph_json).expect("Failed to deserialize graph");
        graph.build_indices();

        // Test with array wrapper (how it would come from notebook with cardinality "n")
        let tree = serde_json::json!({
            "system_reference_numbers": [{
                "uuid": {
                    "resourceid": "650284"
                }
            }]
        });

        println!(
            "Input tree (array wrapper):\n{}",
            serde_json::to_string_pretty(&tree).unwrap()
        );

        let result = tree_to_tiles(&tree, &graph, true, None).expect("tree_to_tiles failed");

        let resource = &result.business_data.resources[0];
        let tiles = resource.tiles.as_ref().expect("Should have tiles");

        println!("Output tiles (array wrapper):");
        for tile in tiles {
            println!("  Tile nodegroup_id: {}", tile.nodegroup_id);
            println!(
                "  Tile data: {}",
                serde_json::to_string_pretty(&tile.data).unwrap()
            );
        }

        // Find the tile for sys-ref-ng
        let sys_ref_tile = tiles
            .iter()
            .find(|t| t.nodegroup_id == "sys-ref-ng")
            .expect("Should have sys-ref-ng tile");

        // The resourceid value should be in the tile data
        let resourceid_value = sys_ref_tile.data.get("resourceid-id");
        assert!(
            resourceid_value.is_some(),
            "Tile data should contain resourceid-id. Got: {}",
            serde_json::to_string_pretty(&sys_ref_tile.data).unwrap()
        );
    }

    /// Test using real Heritage Item graph structure (loaded from file if available)
    #[test]
    fn test_real_heritage_item_system_ref_numbers() {
        // Try to load Heritage Item graph - skip test if not available
        let heritage_item_path = PathBuf::from("/home/philtweir/Cód/Cliant/Quartz/tmp.quartz-starches-buildings-test/prebuild/graphs/resource_models/Heritage Item.json");
        let json_str = match fs::read_to_string(&heritage_item_path) {
            Ok(s) => s,
            Err(_) => {
                println!(
                    "Heritage Item.json not found at {:?}, skipping test",
                    heritage_item_path
                );
                return;
            }
        };

        let json: serde_json::Value =
            serde_json::from_str(&json_str).expect("Failed to parse Heritage Item.json");

        let graph_data = json["graph"][0].clone();

        let mut graph: StaticGraph =
            serde_json::from_value(graph_data).expect("Failed to deserialize StaticGraph");
        graph.build_indices();

        // Test input matching what the notebook would send - using INTEGER like the real case
        let tree = serde_json::json!({
            "system_reference_numbers": {
                "uuid": {
                    "resourceid": 650284  // Integer, not string!
                }
            }
        });

        println!(
            "Input tree:\n{}",
            serde_json::to_string_pretty(&tree).unwrap()
        );
        println!("Graph ID: {}", graph.graphid);

        // Debug: print the relevant node aliases
        let nodes_by_alias = graph.nodes_by_alias_arc().unwrap();
        println!(
            "system_reference_numbers node exists: {}",
            nodes_by_alias.contains_key("system_reference_numbers")
        );
        println!("uuid node exists: {}", nodes_by_alias.contains_key("uuid"));
        println!(
            "resourceid node exists: {}",
            nodes_by_alias.contains_key("resourceid")
        );

        if let Some(uuid_node) = nodes_by_alias.get("uuid") {
            println!("uuid node datatype: {}", uuid_node.datatype);
            println!("uuid node nodegroup_id: {:?}", uuid_node.nodegroup_id);
        }

        if let Some(resourceid_node) = nodes_by_alias.get("resourceid") {
            println!("resourceid node datatype: {}", resourceid_node.datatype);
            println!(
                "resourceid node nodegroup_id: {:?}",
                resourceid_node.nodegroup_id
            );
        }

        // Check edges
        let edges = graph.edges_map().unwrap();
        if let Some(sys_ref_node) = nodes_by_alias.get("system_reference_numbers") {
            println!(
                "system_reference_numbers -> children: {:?}",
                edges.get(&sys_ref_node.nodeid)
            );
        }
        if let Some(uuid_node) = nodes_by_alias.get("uuid") {
            println!("uuid -> children: {:?}", edges.get(&uuid_node.nodeid));
        }

        let result = tree_to_tiles(&tree, &graph, true, None).expect("tree_to_tiles failed");

        let resource = &result.business_data.resources[0];
        let tiles = resource.tiles.as_ref().expect("Should have tiles");

        println!("\nOutput tiles:");
        for tile in tiles {
            println!("  Tile nodegroup_id: {}", tile.nodegroup_id);
            println!(
                "  Tile data: {}",
                serde_json::to_string_pretty(&tile.data).unwrap()
            );
        }

        // Find the tile for System Reference Numbers nodegroup
        let sys_ref_ng = "325a2f2f-efe4-11eb-9b0c-a87eeabdefba";
        let sys_ref_tile = tiles.iter().find(|t| t.nodegroup_id == sys_ref_ng);

        assert!(
            sys_ref_tile.is_some(),
            "Should have System Reference Numbers tile. Got tiles: {:?}",
            tiles.iter().map(|t| &t.nodegroup_id).collect::<Vec<_>>()
        );

        let sys_ref_tile = sys_ref_tile.unwrap();
        let resourceid_node_id = "325a430a-efe4-11eb-810b-a87eeabdefba";

        let resourceid_value = sys_ref_tile.data.get(resourceid_node_id);
        assert!(
            resourceid_value.is_some(),
            "Tile data should contain resourceid node. Got: {}",
            serde_json::to_string_pretty(&sys_ref_tile.data).unwrap()
        );
    }

    /// Test that parenttile_id is set correctly for nested nodegroups
    #[test]
    fn test_parenttile_id_set_for_nested_nodegroups() {
        // Create a graph with nested nodegroups:
        // root -> parent_ng (location_data) -> child_ng (geometry -> coordinates)
        let graph_json = serde_json::json!({
            "graphid": "test-parenttile-graph",
            "name": {"en": "Test Parenttile Graph"},
            "root": {
                "nodeid": "root-id",
                "name": "Root",
                "alias": "root",
                "datatype": "semantic",
                "graph_id": "test-parenttile-graph",
                "istopnode": true
            },
            "nodes": [
                {
                    "nodeid": "root-id",
                    "name": "Root",
                    "alias": "root",
                    "datatype": "semantic",
                    "graph_id": "test-parenttile-graph",
                    "istopnode": true
                },
                {
                    "nodeid": "location-data-id",
                    "name": "Location Data",
                    "alias": "location_data",
                    "datatype": "semantic",
                    "nodegroup_id": "parent-ng",
                    "graph_id": "test-parenttile-graph",
                    "is_collector": true
                },
                {
                    "nodeid": "geometry-id",
                    "name": "Geometry",
                    "alias": "geometry",
                    "datatype": "semantic",
                    "nodegroup_id": "child-ng",
                    "graph_id": "test-parenttile-graph"
                },
                {
                    "nodeid": "coordinates-id",
                    "name": "Coordinates",
                    "alias": "coordinates",
                    "datatype": "geojson-feature-collection",
                    "nodegroup_id": "child-ng",
                    "graph_id": "test-parenttile-graph"
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
                {"edgeid": "edge-3", "domainnode_id": "geometry-id", "rangenode_id": "coordinates-id"}
            ]
        });

        let mut graph: StaticGraph =
            serde_json::from_value(graph_json).expect("Failed to deserialize graph");
        graph.build_indices();

        // Input tree with nested structure
        let tree = serde_json::json!({
            "location_data": {
                "geometry": {
                    "coordinates": {
                        "type": "FeatureCollection",
                        "features": [{
                            "type": "Feature",
                            "geometry": {
                                "type": "Point",
                                "coordinates": [-6.26, 53.35]
                            }
                        }]
                    }
                }
            }
        });

        let result = tree_to_tiles(&tree, &graph, true, None).expect("tree_to_tiles failed");

        let resource = &result.business_data.resources[0];
        let tiles = resource.tiles.as_ref().expect("Should have tiles");

        println!("Generated tiles:");
        for tile in tiles {
            println!(
                "  nodegroup: {}, tileid: {:?}, parenttile_id: {:?}",
                tile.nodegroup_id, tile.tileid, tile.parenttile_id
            );
        }

        // Find parent and child tiles
        let parent_tile = tiles
            .iter()
            .find(|t| t.nodegroup_id == "parent-ng")
            .expect("Should have parent-ng tile");
        let child_tile = tiles
            .iter()
            .find(|t| t.nodegroup_id == "child-ng")
            .expect("Should have child-ng tile");

        // Parent tile should have no parenttile_id (it's at the root level)
        assert!(
            parent_tile.parenttile_id.is_none(),
            "Parent tile should have no parenttile_id. Got: {:?}",
            parent_tile.parenttile_id
        );

        // Child tile should have parenttile_id pointing to parent tile
        assert_eq!(
            child_tile.parenttile_id.as_ref(),
            parent_tile.tileid.as_ref(),
            "Child tile's parenttile_id should match parent tile's tileid. \
             Child parenttile_id: {:?}, Parent tileid: {:?}",
            child_tile.parenttile_id,
            parent_tile.tileid
        );
    }

    /// Helper: build a graph with a name node and optionally a slug descriptor template.
    fn build_slug_test_graph(slug_template: Option<&str>) -> StaticGraph {
        use crate::graph::DESCRIPTOR_FUNCTION_ID;

        let mut descriptor_types = serde_json::json!({
            "name": {
                "nodegroup_id": "name-ng",
                "string_template": "<Name>"
            }
        });

        if let Some(tmpl) = slug_template {
            descriptor_types["slug"] = serde_json::json!({
                "nodegroup_id": "name-ng",
                "string_template": tmpl
            });
        }

        let graph_json = serde_json::json!({
            "graphid": "slug-test-graph",
            "name": {"en": "Slug Test Graph"},
            "root": {
                "nodeid": "root-id",
                "name": "Root",
                "alias": "root",
                "datatype": "semantic",
                "graph_id": "slug-test-graph",
                "istopnode": true
            },
            "nodes": [
                {
                    "nodeid": "root-id",
                    "name": "Root",
                    "alias": "root",
                    "datatype": "semantic",
                    "graph_id": "slug-test-graph",
                    "istopnode": true
                },
                {
                    "nodeid": "name-node-id",
                    "name": "Name",
                    "alias": "name",
                    "datatype": "string",
                    "nodegroup_id": "name-ng",
                    "graph_id": "slug-test-graph"
                }
            ],
            "nodegroups": [
                { "nodegroupid": "name-ng", "cardinality": "1" }
            ],
            "edges": [
                { "domainnode_id": "root-id", "rangenode_id": "name-node-id" }
            ],
            "functions_x_graphs": [
                {
                    "config": { "descriptor_types": descriptor_types },
                    "function_id": DESCRIPTOR_FUNCTION_ID,
                    "graph_id": "slug-test-graph",
                    "id": "fxg-1"
                }
            ]
        });

        let mut graph: StaticGraph =
            serde_json::from_value(graph_json).expect("slug test graph JSON");
        graph.build_indices();
        graph
    }

    #[test]
    fn test_slug_based_id_is_deterministic() {
        let graph = build_slug_test_graph(Some("<Name>"));

        let tree = serde_json::json!({
            "graph_id": "slug-test-graph",
            "name": {"en": {"value": "My Test Resource", "direction": "ltr"}}
        });

        // random_ids=false → slug-based UUID5
        let result1 =
            tree_to_tiles_with_options(&tree, &graph, false, None, false, false, false, None)
                .expect("First conversion failed");
        let result2 =
            tree_to_tiles_with_options(&tree, &graph, false, None, false, false, false, None)
                .expect("Second conversion failed");

        let id1 = &result1.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;
        let id2 = &result2.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;

        assert_eq!(id1, id2, "Same tree data should produce same resource ID");
        assert!(uuid::Uuid::parse_str(id1).is_ok(), "Should be valid UUID");
    }

    #[test]
    fn test_slug_based_id_differs_for_different_data() {
        let graph = build_slug_test_graph(Some("<Name>"));

        let tree1 = serde_json::json!({
            "graph_id": "slug-test-graph",
            "name": {"en": {"value": "Resource Alpha", "direction": "ltr"}}
        });
        let tree2 = serde_json::json!({
            "graph_id": "slug-test-graph",
            "name": {"en": {"value": "Resource Beta", "direction": "ltr"}}
        });

        let result1 =
            tree_to_tiles_with_options(&tree1, &graph, false, None, false, false, false, None)
                .expect("First conversion failed");
        let result2 =
            tree_to_tiles_with_options(&tree2, &graph, false, None, false, false, false, None)
                .expect("Second conversion failed");

        let id1 = &result1.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;
        let id2 = &result2.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;

        assert_ne!(
            id1, id2,
            "Different data should produce different resource IDs"
        );
    }

    #[test]
    fn test_slug_based_id_errors_without_slug_configured() {
        // Graph with NO slug descriptor template
        let graph = build_slug_test_graph(None);

        let tree = serde_json::json!({
            "graph_id": "slug-test-graph",
            "name": {"en": {"value": "Test", "direction": "ltr"}}
        });

        // random_ids=false, no id_key, no slug configured → error
        let result =
            tree_to_tiles_with_options(&tree, &graph, false, None, false, false, false, None);

        assert!(result.is_err(), "Should error without slug configured");
        let err = result.unwrap_err();
        assert!(
            err.contains("No slug descriptor configured"),
            "Error should mention slug not configured. Got: {}",
            err
        );
    }

    #[test]
    fn test_random_ids_bypasses_slug() {
        // Graph with NO slug descriptor template
        let graph = build_slug_test_graph(None);

        let tree = serde_json::json!({
            "graph_id": "slug-test-graph",
            "name": {"en": {"value": "Test", "direction": "ltr"}}
        });

        // random_ids=true → should succeed even without slug
        let result =
            tree_to_tiles_with_options(&tree, &graph, false, None, false, true, false, None)
                .expect("random_ids=true should not require slug");

        let id = &result.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;
        assert!(uuid::Uuid::parse_str(id).is_ok(), "Should be valid UUID");
    }

    #[test]
    fn test_explicit_id_takes_precedence_over_slug() {
        let graph = build_slug_test_graph(Some("<Name>"));

        let tree = serde_json::json!({
            "resourceinstanceid": "my-explicit-id",
            "graph_id": "slug-test-graph",
            "name": {"en": {"value": "Test", "direction": "ltr"}}
        });

        // Even with random_ids=false, explicit ID should win
        let result =
            tree_to_tiles_with_options(&tree, &graph, false, None, false, false, false, None)
                .expect("Explicit ID should work");

        let id = &result.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;
        assert_eq!(id, "my-explicit-id", "Explicit ID should take precedence");
    }

    #[test]
    fn test_id_key_takes_precedence_over_slug() {
        let graph = build_slug_test_graph(Some("<Name>"));

        let tree = serde_json::json!({
            "graph_id": "slug-test-graph",
            "name": {"en": {"value": "Test", "direction": "ltr"}}
        });

        // With id_key provided, slug should not be used
        let result = tree_to_tiles_with_options(
            &tree,
            &graph,
            false,
            Some("my-key"),
            false,
            false,
            false,
            None,
        )
        .expect("id_key should work");

        let id = &result.business_data.resources[0]
            .resourceinstance
            .resourceinstanceid;

        // Should be a uuid5 from the id_key, not from the slug
        let expected_id = crate::generate_uuid_v5(("resource", Some("slug-test-graph")), "my-key");
        assert_eq!(id, &expected_id, "id_key should produce uuid5 from key");
    }

    #[test]
    fn test_slug_based_id_patches_all_tiles() {
        let graph = build_slug_test_graph(Some("<Name>"));

        let tree = serde_json::json!({
            "graph_id": "slug-test-graph",
            "name": {"en": {"value": "Tile Patch Test", "direction": "ltr"}}
        });

        let result =
            tree_to_tiles_with_options(&tree, &graph, false, None, false, false, false, None)
                .expect("Conversion failed");

        let resource = &result.business_data.resources[0];
        let resource_id = &resource.resourceinstance.resourceinstanceid;

        // Every tile should have the same resourceinstance_id as the resource
        if let Some(tiles) = &resource.tiles {
            assert!(!tiles.is_empty(), "Should have at least one tile");
            for tile in tiles {
                assert_eq!(
                    &tile.resourceinstance_id, resource_id,
                    "Tile resourceinstance_id should match resource ID"
                );
            }
        }
    }

    /// Helper: build a graph with two string nodes (name + code) in the same nodegroup,
    /// and a slug template that references only one of them.
    fn build_two_node_slug_graph(slug_template: &str) -> StaticGraph {
        use crate::graph::DESCRIPTOR_FUNCTION_ID;

        let descriptor_types = serde_json::json!({
            "name": {
                "nodegroup_id": "info-ng",
                "string_template": "<Name>"
            },
            "slug": {
                "nodegroup_id": "info-ng",
                "string_template": slug_template
            }
        });

        let graph_json = serde_json::json!({
            "graphid": "two-node-slug-graph",
            "name": {"en": "Two Node Slug Graph"},
            "root": {
                "nodeid": "root-id",
                "name": "Root",
                "alias": "root",
                "datatype": "semantic",
                "graph_id": "two-node-slug-graph",
                "istopnode": true
            },
            "nodes": [
                {
                    "nodeid": "root-id",
                    "name": "Root",
                    "alias": "root",
                    "datatype": "semantic",
                    "graph_id": "two-node-slug-graph",
                    "istopnode": true
                },
                {
                    "nodeid": "name-node-id",
                    "name": "Name",
                    "alias": "name",
                    "datatype": "string",
                    "nodegroup_id": "info-ng",
                    "graph_id": "two-node-slug-graph"
                },
                {
                    "nodeid": "code-node-id",
                    "name": "Code",
                    "alias": "code",
                    "datatype": "string",
                    "nodegroup_id": "info-ng",
                    "graph_id": "two-node-slug-graph"
                }
            ],
            "nodegroups": [
                { "nodegroupid": "info-ng", "cardinality": "1" }
            ],
            "edges": [
                { "domainnode_id": "root-id", "rangenode_id": "name-node-id" },
                { "domainnode_id": "root-id", "rangenode_id": "code-node-id" }
            ],
            "functions_x_graphs": [
                {
                    "config": { "descriptor_types": descriptor_types },
                    "function_id": DESCRIPTOR_FUNCTION_ID,
                    "graph_id": "two-node-slug-graph",
                    "id": "fxg-1"
                }
            ]
        });

        let mut graph: StaticGraph =
            serde_json::from_value(graph_json).expect("two-node slug test graph JSON");
        graph.build_indices();
        graph
    }

    #[test]
    fn test_slug_unresolved_placeholder_errors() {
        // Slug template references both <Name> and <Code>, but we only provide name data
        let graph = build_two_node_slug_graph("<Name>-<Code>");

        // Tree with name but no code — <Code> placeholder stays unresolved
        let tree = serde_json::json!({
            "graph_id": "two-node-slug-graph",
            "name": {"en": {"value": "Test", "direction": "ltr"}}
        });

        let result =
            tree_to_tiles_with_options(&tree, &graph, false, None, false, false, false, None);

        assert!(result.is_err(), "Should error on unresolved placeholder");
        let err = result.unwrap_err();
        assert!(
            err.contains("unresolved placeholder"),
            "Error should mention unresolved placeholder. Got: {}",
            err
        );
    }
}
