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

mod translatable;
mod nodes;
mod tile;
mod cards;
mod descriptors;
mod resources;
mod meta;
mod static_graph;

// Re-export all public types
pub use translatable::StaticTranslatableString;
pub use nodes::{StaticNode, StaticNodegroup, StaticEdge};
pub use tile::StaticTile;
pub use cards::{StaticCard, StaticConstraint, StaticPublication, StaticCardsXNodesXWidgets, StaticFunctionsXGraphs};
pub use descriptors::{StaticResourceDescriptors, DescriptorConfig, DescriptorTypeConfig, DESCRIPTOR_FUNCTION_ID};
pub use resources::{StaticResource, StaticResourceMetadata, StaticResourceSummary, StaticResourceReference};
pub use meta::StaticGraphMeta;
pub use static_graph::{StaticGraph, IndexedGraph, GraphWrapper};

/// Datatypes that represent iterable/list values
pub const ITERABLE_DATATYPES: &[&str] = &["concept-list", "resource-instance-list", "domain-value-list"];

/// Check if a datatype is iterable
pub fn is_iterable_datatype(datatype: &str) -> bool {
    ITERABLE_DATATYPES.contains(&datatype)
}
