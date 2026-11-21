//! Static types as core reads them from JSON, for the bindings that hand them
//! back as plain data: NAPI's `createStatic*` (WASM's are classes over the
//! same structs) and Python's `to_dict`.

use crate::graph::{
    StaticCard, StaticCardsXNodesXWidgets, StaticConstraint, StaticEdge, StaticFunctionsXGraphs,
    StaticGraph, StaticGraphMeta, StaticNode, StaticNodegroup, StaticPublication, StaticResource,
    StaticResourceDescriptors, StaticTile,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

fn via<T: DeserializeOwned + Serialize>(json: Value) -> Result<Value, String> {
    let typed: T = serde_json::from_value(json).map_err(|e| e.to_string())?;
    serde_json::to_value(&typed).map_err(|e| e.to_string())
}

/// Parse `json` as the static type `kind` (`"node"`, `"resource"`, ...) and
/// serialise it back: core's reading of it, with defaults filled, aliases
/// resolved and empty fields as core writes them.
pub fn canonical_static(kind: &str, json: Value) -> Result<Value, String> {
    match kind {
        "node" => via::<StaticNode>(json),
        "nodegroup" => via::<StaticNodegroup>(json),
        "edge" => via::<StaticEdge>(json),
        "card" => via::<StaticCard>(json),
        "constraint" => via::<StaticConstraint>(json),
        "cards_x_nodes_x_widgets" => via::<StaticCardsXNodesXWidgets>(json),
        "functions_x_graphs" => via::<StaticFunctionsXGraphs>(json),
        "publication" => via::<StaticPublication>(json),
        "graph_meta" => via::<StaticGraphMeta>(json),
        "tile" => via::<StaticTile>(json),
        "resource" => via::<StaticResource>(json),
        "resource_descriptors" => via::<StaticResourceDescriptors>(json),
        "graph" => via::<StaticGraph>(json),
        other => Err(format!("unknown static type: {other}")),
    }
    .map_err(|e| format!("{kind}: {e}"))
}

/// A resource built from input, as every binding's constructor builds one:
/// parsed, with `tiles_loaded` settled ([`StaticResource::infer_tiles_loaded`]).
pub fn construct_static_resource(json: Value) -> Result<StaticResource, String> {
    let mut resource: StaticResource =
        serde_json::from_value(json).map_err(|e| format!("resource: {e}"))?;
    resource.infer_tiles_loaded();
    Ok(resource)
}

/// The static type `kind` built from input: [`canonical_static`], except that
/// a resource is [`construct_static_resource`]'s.
pub fn construct_static(kind: &str, json: Value) -> Result<Value, String> {
    if kind == "resource" {
        serde_json::to_value(construct_static_resource(json)?).map_err(|e| e.to_string())
    } else {
        canonical_static(kind, json)
    }
}

/// The resources of a business data file, as core's one reader of them
/// ([`crate::parse_business_data_bytes`]) gives them.
pub fn resources_from_business_data_json(json: &str) -> Result<Vec<StaticResource>, String> {
    crate::parse_business_data_bytes(json.as_bytes())
        .map_err(|e| format!("Failed to parse business_data JSON: {e}"))
}
