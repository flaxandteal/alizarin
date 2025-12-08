/// Alizarin Core Library
///
/// Platform-agnostic core functionality that can be used from:
/// - JavaScript/TypeScript via WASM bindings (alizarin-wasm)
/// - Python via PyO3 bindings (alizarin-python)
/// - Native Rust applications (alizarin-explorer)
/// - Other languages via C FFI

pub mod graph;
pub mod loader;
pub mod pseudo_node;
pub mod pseudo_node_lite;

// Graph types
pub use graph::{
    GraphWrapper, IndexedGraph, StaticEdge, StaticGraph, StaticGraphMeta, StaticNode, StaticNodegroup,
    StaticTranslatableString, ITERABLE_DATATYPES, is_iterable_datatype,
    // Additional Static* types
    StaticConstraint, StaticPublication, StaticCardsXNodesXWidgets, StaticFunctionsXGraphs,
    StaticCard, StaticTile, StaticResourceDescriptors, StaticResourceMetadata,
    StaticResourceSummary, StaticResourceReference, StaticResource,
};

// Loader
pub use loader::{LoaderError, PrebuildInfo, PrebuildLoader};

// PseudoNode types
pub use pseudo_node::{NodeLike, PseudoNodeBuilder, PseudoNodeCore};
pub use pseudo_node_lite::{build_placeholder, PseudoNodeState};
