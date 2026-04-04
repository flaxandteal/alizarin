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
//! - `static_graph` - StaticGraph and IndexedGraph

pub mod card_index;
mod cards;
mod descriptors;
mod meta;
mod nodes;
mod prune;
mod resources;
mod static_graph;
mod tile;
mod translatable;

// Re-export all public types
pub use cards::{
    StaticCard, StaticCardsXNodesXWidgets, StaticConstraint, StaticFunctionsXGraphs,
    StaticPublication,
};
pub use descriptors::{
    DescriptorConfig, DescriptorTypeConfig, StaticResourceDescriptors, DESCRIPTOR_FUNCTION_ID,
};
pub use meta::StaticGraphMeta;
pub use nodes::{StaticEdge, StaticNode, StaticNodegroup};
pub use prune::{build_backedges, find_root_node, prune_graph, PruneError};
pub use resources::{
    batch_merge_resources, merge_resources, unify_cardinality_one_tiles, BatchMergeResult,
    MergeResult, PopulateCachesResult, RelatedResourceEntry, ResourceCache, ResourceEntry,
    StaticResource, StaticResourceMetadata, StaticResourceReference, StaticResourceRegistry,
    StaticResourceSummary, UnknownReference,
};
pub use static_graph::{GraphWrapper, IndexedGraph, StaticGraph};
pub use tile::StaticTile;
pub use translatable::StaticTranslatableString;

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
