//! Card-based tree traversal.
//!
//! Produces a tree structured by the card hierarchy (cards → widgets → child cards)
//! instead of the node/edge hierarchy. Reuses the same `pseudo_cache` infrastructure
//! as `tiles_to_tree` — no data duplication.
//!
//! Two entry points:
//! - [`cards_to_tree`]: Eager — builds its own pseudo_cache from tiles, serializes everything.
//! - [`serialize_card`] / [`serialize_root_cards`]: Works against an *existing* cache (e.g.
//!   one managed by `ResourceInstanceWrapperCore`), enabling lazy per-card loading.

use serde_json::{json, Map, Value};
use std::collections::HashMap;

use crate::graph::card_index::{CardIndex, CardRef};
use crate::graph::StaticGraph;
use crate::json_conversion::{build_pseudo_cache_from_tiles, extract_resources};
use crate::node_config::NodeConfigManager;
use crate::pseudo_value_core::{matches_tile_filter, PseudoListCore};
use crate::type_serialization::{SerializationContext, SerializationOptions};

/// Bundles the serialization parameters that thread through all card traversal functions.
pub struct CardSerializationParams<'a> {
    pub opts: &'a SerializationOptions,
    pub ser_ctx: &'a SerializationContext<'a>,
    pub node_config_manager: Option<&'a NodeConfigManager>,
}

/// Shared context for card traversal — graph data + serialization params.
struct TraversalCtx<'a> {
    card_index: &'a CardIndex,
    pseudo_cache: &'a HashMap<String, PseudoListCore>,
    graph: &'a StaticGraph,
    params: CardSerializationParams<'a>,
}

// =============================================================================
// Eager entry point (builds its own cache)
// =============================================================================

/// Convert tiles to a card-structured tree.
///
/// The output groups data by cards (UI structure) rather than by node edges (data structure).
/// Each card contains its widgets' values and nested child cards.
pub fn cards_to_tree(input: &Value, graph: &StaticGraph) -> Result<Value, String> {
    let card_index = graph
        .card_index()
        .ok_or_else(|| "Graph has no card index — cards may not be loaded".to_string())?;

    let resources = extract_resources(input)?;
    let mut results = Vec::new();

    for resource in resources {
        let tiles = resource
            .tiles
            .as_ref()
            .ok_or_else(|| "Resource has no tiles".to_string())?;

        let nodes_by_alias = graph
            .nodes_by_alias_arc()
            .ok_or_else(|| "Graph indices not built".to_string())?;
        let edges = graph
            .edges_map()
            .ok_or_else(|| "Graph indices not built".to_string())?;

        let pseudo_cache = build_pseudo_cache_from_tiles(tiles, nodes_by_alias, graph, edges);

        let root_cards = serialize_root_cards(card_index, &pseudo_cache, graph, None, None);

        let mut resource_obj = Map::new();
        resource_obj.insert(
            "resourceinstance".to_string(),
            serde_json::to_value(&resource.resourceinstance).unwrap_or(Value::Object(Map::new())),
        );
        resource_obj.insert("cards".to_string(), Value::Array(root_cards));
        results.push(Value::Object(resource_obj));
    }

    Ok(Value::Array(results))
}

// =============================================================================
// Cache-agnostic serialization (works with any pseudo_cache)
// =============================================================================

/// Serialize all root cards from an existing pseudo_cache.
///
/// `max_depth`: `None` = unlimited recursion, `Some(0)` = root cards' widgets only
/// (no child card content), `Some(1)` = root + immediate children, etc.
///
/// `params`: pass `None` for tile_data mode (default), or provide
/// `CardSerializationParams` with display options + a populated `SerializationContext`
/// for display mode.
pub fn serialize_root_cards(
    card_index: &CardIndex,
    pseudo_cache: &HashMap<String, PseudoListCore>,
    graph: &StaticGraph,
    max_depth: Option<usize>,
    params: Option<CardSerializationParams>,
) -> Vec<Value> {
    let default_opts;
    let default_ctx;
    let params = match params {
        Some(p) => p,
        None => {
            default_opts = SerializationOptions::tile_data();
            default_ctx = SerializationContext::default();
            CardSerializationParams {
                opts: &default_opts,
                ser_ctx: &default_ctx,
                node_config_manager: None,
            }
        }
    };

    let ctx = TraversalCtx {
        card_index,
        pseudo_cache,
        graph,
        params,
    };

    card_index
        .root_card_ids
        .iter()
        .filter_map(|card_id| {
            let card_ref = card_index.cards_by_id.get(card_id)?;
            let v = card_to_json(card_ref, &ctx, None, None, max_depth);
            if v.is_null() {
                None
            } else {
                Some(v)
            }
        })
        .collect()
}

/// Serialize a single card (by ID) from an existing pseudo_cache.
///
/// `max_depth`: `None` = unlimited, `Some(0)` = this card's widgets only, etc.
///
/// `params`: pass `None` for tile_data mode (default), or provide
/// `CardSerializationParams` with display options + a populated `SerializationContext`
/// for display mode.
///
/// The caller is responsible for ensuring the relevant nodegroups are loaded
/// into `pseudo_cache` before calling this. Use
/// [`CardIndex::nodegroup_ids_for_card`] to discover which nodegroups are needed.
#[allow(clippy::too_many_arguments)]
pub fn serialize_card(
    card_id: &str,
    card_index: &CardIndex,
    pseudo_cache: &HashMap<String, PseudoListCore>,
    parent_tile_id: Option<&str>,
    parent_nodegroup_id: Option<&str>,
    graph: &StaticGraph,
    max_depth: Option<usize>,
    params: Option<CardSerializationParams>,
) -> Result<Value, String> {
    let card_ref = card_index
        .cards_by_id
        .get(card_id)
        .ok_or_else(|| format!("Card '{}' not found in index", card_id))?;

    let default_opts;
    let default_ctx;
    let params = match params {
        Some(p) => p,
        None => {
            default_opts = SerializationOptions::tile_data();
            default_ctx = SerializationContext::default();
            CardSerializationParams {
                opts: &default_opts,
                ser_ctx: &default_ctx,
                node_config_manager: None,
            }
        }
    };

    let ctx = TraversalCtx {
        card_index,
        pseudo_cache,
        graph,
        params,
    };

    Ok(card_to_json(
        card_ref,
        &ctx,
        parent_tile_id,
        parent_nodegroup_id,
        max_depth,
    ))
}

// =============================================================================
// Internal serialization (shared by eager and lazy paths)
// =============================================================================

/// Serialize a single card and its children.
fn card_to_json(
    card: &CardRef,
    ctx: &TraversalCtx,
    parent_tile_id: Option<&str>,
    parent_nodegroup_id: Option<&str>,
    max_depth: Option<usize>,
) -> Value {
    let nodegroup_id = &card.nodegroup_id;

    // Determine cardinality
    let cardinality = ctx
        .graph
        .nodegroups
        .iter()
        .find(|ng| ng.nodegroupid == *nodegroup_id)
        .and_then(|ng| ng.cardinality.as_deref())
        .unwrap_or("1");

    // Find matching tiles for this card's nodegroup.
    let matching_tile_ids = find_matching_tile_ids(
        nodegroup_id,
        parent_tile_id,
        parent_nodegroup_id,
        ctx.pseudo_cache,
        ctx.graph,
    );

    let widgets = ctx
        .card_index
        .widgets_by_card
        .get(&card.cardid)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    let child_card_ids = ctx
        .card_index
        .card_children
        .get(&card.cardid)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    let child_depth = max_depth.map(|d| d.saturating_sub(1));

    if cardinality == "n" {
        // Multi-cardinality: produce an array of instances
        let instances: Vec<Value> = matching_tile_ids
            .iter()
            .map(|tile_id| {
                build_card_instance(
                    tile_id.as_deref(),
                    nodegroup_id,
                    widgets,
                    child_card_ids,
                    ctx,
                    max_depth,
                    child_depth,
                )
            })
            .collect();

        json!({
            "card_id": card.cardid,
            "name": card.name,
            "component_id": card.component_id,
            "cardinality": "n",
            "visible": card.visible,
            "active": card.active,
            "instances": instances,
        })
    } else {
        // Single-cardinality: produce one object (first match, or empty)
        let tile_id = matching_tile_ids.first().and_then(|t| t.as_deref());
        let instance = build_card_instance(
            tile_id,
            nodegroup_id,
            widgets,
            child_card_ids,
            ctx,
            max_depth,
            child_depth,
        );

        let mut obj = Map::new();
        obj.insert("card_id".to_string(), json!(card.cardid));
        obj.insert(
            "name".to_string(),
            serde_json::to_value(&card.name).unwrap_or(Value::Null),
        );
        obj.insert("component_id".to_string(), json!(card.component_id));
        obj.insert("cardinality".to_string(), json!("1"));
        obj.insert("visible".to_string(), json!(card.visible));
        obj.insert("active".to_string(), json!(card.active));
        // Merge instance fields into card
        if let Value::Object(inst) = instance {
            for (k, v) in inst {
                obj.insert(k, v);
            }
        }
        Value::Object(obj)
    }
}

/// Build a single card instance (widgets + child cards for one tile).
///
/// `current_depth` controls whether widgets are rendered (always, if we got here).
/// `child_depth` controls whether child cards recurse further.
fn build_card_instance(
    tile_id: Option<&str>,
    nodegroup_id: &str,
    widgets: &[crate::graph::card_index::CardWidgetRef],
    child_card_ids: &[String],
    ctx: &TraversalCtx,
    current_depth: Option<usize>,
    child_depth: Option<usize>,
) -> Value {
    let mut widget_values = Vec::new();

    for widget in widgets {
        if !widget.visible {
            continue;
        }

        let value = get_widget_value(&widget.node_alias, tile_id, ctx);

        widget_values.push(json!({
            "node_alias": widget.node_alias,
            "node_id": widget.node_id,
            "widget_id": widget.widget_id,
            "widget_name": widget.widget_name,
            "label": widget.label,
            "sortorder": widget.sortorder,
            "value": value,
        }));
    }

    // Recurse into child cards (unless depth exhausted)
    let child_cards = if current_depth == Some(0) {
        Vec::new()
    } else {
        let mut children = Vec::new();
        for child_card_id in child_card_ids {
            if let Some(child_ref) = ctx.card_index.cards_by_id.get(child_card_id) {
                if !child_ref.active {
                    continue;
                }
                let child_json =
                    card_to_json(child_ref, ctx, tile_id, Some(nodegroup_id), child_depth);
                children.push(child_json);
            }
        }
        children
    };

    json!({
        "tile_id": tile_id,
        "widgets": widget_values,
        "cards": child_cards,
    })
}

/// Find tile IDs that match a card's nodegroup, filtered by parent tile.
fn find_matching_tile_ids(
    nodegroup_id: &str,
    parent_tile_id: Option<&str>,
    parent_nodegroup_id: Option<&str>,
    pseudo_cache: &HashMap<String, PseudoListCore>,
    graph: &StaticGraph,
) -> Vec<Option<String>> {
    // Find any PseudoList for a node in this nodegroup
    let nodes_in_ng = graph.get_nodes_in_nodegroup(nodegroup_id);
    if nodes_in_ng.is_empty() {
        // No nodes in this nodegroup — could be a container card with only child cards
        return vec![None];
    }

    // Try each node's alias until we find one with values in the cache
    for node in &nodes_in_ng {
        if let Some(alias) = &node.alias {
            if let Some(pseudo_list) = pseudo_cache.get(alias) {
                let parent_tid = parent_tile_id.map(|s| s.to_string());
                let ng_id = nodegroup_id.to_string();
                let parent_ng_id = parent_nodegroup_id.map(|s| s.to_string());
                let matching = pseudo_list.values.iter().filter(|v| match v.tile.as_ref() {
                    Some(tile) => matches_tile_filter(
                        tile,
                        parent_tid.as_ref(),
                        Some(&ng_id),
                        parent_ng_id.as_ref(),
                    ),
                    None => parent_tile_id.is_none(),
                });

                let tile_ids: Vec<Option<String>> = matching
                    .map(|v| v.tile.as_ref().and_then(|t| t.tileid.clone()))
                    .collect();

                if !tile_ids.is_empty() {
                    // Deduplicate (multiple nodes in same nodegroup share same tiles)
                    let mut seen = std::collections::HashSet::new();
                    let unique: Vec<Option<String>> = tile_ids
                        .into_iter()
                        .filter(|tid| seen.insert(tid.clone()))
                        .collect();
                    return unique;
                }
            }
        }
    }

    // No matching tiles found
    vec![]
}

/// Get a widget's value from the pseudo_cache.
///
/// Builds a per-node `SerializationContext` using the `node_config_manager` so that
/// domain values, boolean labels, concept lookups etc. resolve correctly in display mode.
fn get_widget_value(node_alias: &str, tile_id: Option<&str>, ctx: &TraversalCtx) -> Value {
    let pseudo_list = match ctx.pseudo_cache.get(node_alias) {
        Some(list) => list,
        None => return Value::Null,
    };

    // Find the value matching this tile
    for value in &pseudo_list.values {
        let matches = match value.tile.as_ref() {
            Some(tile) => {
                if let Some(tid) = tile_id {
                    tile.tileid.as_deref() == Some(tid)
                } else {
                    true // No tile filter
                }
            }
            None => tile_id.is_none(),
        };

        if matches {
            if let Some(ref data) = value.tile_data {
                // Build per-node context with the node's specific config
                let node_config = ctx
                    .params
                    .node_config_manager
                    .and_then(|ncm| ncm.get(&value.node.nodeid));
                let per_node_ctx = ctx.params.ser_ctx.with_node_config(node_config);

                return crate::type_serialization::serialize_value(
                    &value.node.datatype,
                    data,
                    ctx.params.opts,
                    Some(&per_node_ctx),
                )
                .value;
            }
        }
    }

    Value::Null
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cards_to_tree_requires_card_index() {
        let input = json!({
            "business_data": {
                "resources": [{
                    "resourceinstance": {
                        "resourceinstanceid": "test-id",
                        "graph_id": "test-graph",
                        "name": "Test"
                    },
                    "tiles": []
                }]
            }
        });

        // A graph with no cards should fail gracefully
        let graph_json = r#"{
            "graphid": "test-graph",
            "name": {"en": "Test"},
            "isresource": true,
            "nodes": [
                {"nodeid": "root", "name": "Root", "alias": "root", "datatype": "semantic",
                 "graph_id": "test-graph", "is_collector": false, "isrequired": false,
                 "exportable": false, "hascustomalias": false, "issearchable": false, "istopnode": true}
            ],
            "nodegroups": [],
            "edges": [],
            "root": {"nodeid": "root", "name": "Root", "alias": "root", "datatype": "semantic",
                     "graph_id": "test-graph", "is_collector": false, "isrequired": false,
                     "exportable": false, "hascustomalias": false, "issearchable": false, "istopnode": true}
        }"#;
        let graph = StaticGraph::from_json_string(graph_json).unwrap();

        let result = cards_to_tree(&input, &graph);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no card index"));
    }
}
