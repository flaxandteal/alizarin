//! Core graph data structures for Arches/Alizarin
//!
//! These are platform-agnostic types that can be used from:
//! - JavaScript/TypeScript via WASM bindings (alizarin main crate)
//! - Native Rust applications (alizarin-explorer)
//! - Python via PyO3 bindings (alizarin-python)
//!
//! ## Submodules
//!
//! - `translatable` - Multi-language string support
//! - `nodes` - Node, Nodegroup, and Edge types
//! - `tile` - Tile type for data storage
//! - `cards` - UI configuration types
//! - `descriptors` - Resource descriptor configuration
//! - `resources` - Resource types and metadata
//! - `meta` - Lightweight graph metadata
//! - `lifecycle` - Resource instance lifecycles: compiled index, state resolution, transitions
//! - `static_graph` - StaticGraph and its lazy self-indexing

pub mod card_index;
mod cards;
mod computed_tiles;
mod descriptors;
mod functions;
mod graph_lookup;
mod layered_graph;
mod lifecycle;
mod meta;
mod prune;
mod resources;
mod static_graph;
mod tile;

// The plain graph data an extension reads too, defined in the extension API
// (see notes/extension-interface-split.md), under the module names the graph
// modules here import them by.
mod nodes {
    pub use alizarin_extension_api::graph::{StaticEdge, StaticNode, StaticNodegroup};
}
mod translatable {
    pub use alizarin_extension_api::graph::StaticTranslatableString;
}
pub use alizarin_extension_api::graph::{
    compare_nodes, compare_nodes_from_json, NodeComparison, StaticEdge, StaticNode,
    StaticNodegroup, StaticTranslatableString,
};
pub(crate) use alizarin_extension_api::serde_helpers;

// Re-export all public types
pub use cards::{
    StaticCard, StaticCardsXNodesXWidgets, StaticConstraint, StaticFunctionsXGraphs,
    StaticPublication,
};
pub use computed_tiles::{
    derive_functions, resolve_derive_nodes, DeriveConfig, DeriveMode, ResolvedNode,
};
// The former names, kept for existing callers.
#[allow(deprecated)]
pub use computed_tiles::{compute_tiles_functions, ComputeTilesConfig};
pub use descriptors::{
    DescriptorConfig, DescriptorTypeConfig, StaticResourceDescriptors, AGGREGATE_FUNCTION_ID,
    COMPUTE_TILES_FUNCTION_ID, DESCRIPTOR_FUNCTION_ID,
};
pub use functions::{
    aggregate_functions, apply_aggregate_additions, apply_aggregate_additions_in_registry,
    apply_aggregate_additions_to_resource, apply_aggregate_functions,
    apply_aggregate_functions_in_registry, apply_aggregate_functions_to_resource,
    apply_derive_functions, apply_stored_derive_functions,
    apply_stored_derive_functions_to_resources, default_functions_registry, derived_tile_id,
    mark_aggregate_pending, mark_aggregate_pending_in_registry, stored_derive_functions,
    AggregateConfig, AggregateContext, AggregateProvider, DeriveProvider, DescriptorProvider,
    FunctionsRegistry, NodeValueWrite, PendingNode, RegisteredFunction, StoredDeriveChanges,
};
pub use graph_lookup::GraphLookup;
pub use layered_graph::LayeredGraph;
pub use lifecycle::{
    lifecycle_state_from_json, transition, transition_from_json, EffectiveState, LifecycleIndex,
    LifecycleStateReport, StaticLifecycle, StaticLifecycleState, StoredState, TransitionEvent,
    TransitionResult, MAX_LIFECYCLE_STATES,
};
pub use meta::StaticGraphMeta;
pub use prune::{build_backedges, find_root_node, prune_graph, PruneError};
pub use resources::{
    batch_merge_resources, batch_merge_resources_from_json, compose_resource_across_layers,
    compose_resource_layers_from_json, merge_resources, merge_resources_from_json,
    parse_resources_from_json_str, unify_cardinality_one_tiles, BatchMergeResult, ComposedResource,
    MergeAccumulator, MergeResult, PopulateCachesOutput, PopulateCachesResult,
    RelatedResourceEntry, ResourceCache, ResourceEntry, StaticResource, StaticResourceMetadata,
    StaticResourceReference, StaticResourceRegistry, StaticResourceSummary, TileMergeMode,
    UnknownReference,
};
pub use static_graph::{GraphWrapper, StaticGraph};
pub use tile::{
    allocate_multi_tile_id, canonical_tile_id, multi_tile_id, tile_namespace, StaticTile,
    TILE_NAMESPACE_URI,
};

/// Datatypes that represent iterable/list values
pub const ITERABLE_DATATYPES: &[&str] = &[
    "concept-list",
    "resource-instance-list",
    "domain-value-list",
];

/// Check if a datatype is iterable
pub fn is_iterable_datatype(datatype: &str) -> bool {
    ITERABLE_DATATYPES.contains(&datatype)
}
