/// Alizarin Core Library
///
/// Platform-agnostic core functionality that can be used from:
/// - JavaScript/TypeScript via WASM bindings (alizarin-wasm)
/// - Python via PyO3 bindings (alizarin-python)
/// - Native Rust applications (alizarin-explorer)
/// - Other languages via C FFI

pub mod graph;
pub mod interner;
pub mod json_conversion;
pub mod label_resolution;
pub mod loader;
pub mod node_config;
pub mod pseudo_value_core;
pub mod skos;
pub mod type_coercion;

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

// Interner types
pub use interner::{InternedId, Interner, InternerExt};
#[cfg(feature = "multi-threaded")]
pub use interner::ThreadSafeInterner;

// Node config types
pub use node_config::{
    NodeConfig, NodeConfigBoolean, NodeConfigConcept, NodeConfigDomain, NodeConfigManager,
    NodeConfigReference, StaticDomainValue,
};

// Type coercion
pub use type_coercion::{
    // Phase 1: Simple scalars
    coerce_date, coerce_edtf, coerce_non_localized_string, coerce_number,
    // Phase 2: Dict types
    coerce_geojson, coerce_string, coerce_url,
    // Phase 3: Config-dependent types
    coerce_boolean, coerce_domain_value, coerce_domain_value_list,
    // Phase 4: RDM-dependent types
    coerce_concept_value, coerce_concept_list,
    // Phase 5: Format normalization
    coerce_resource_instance, coerce_resource_instance_list,
    // Dispatcher and result
    coerce_value, CoercionResult,
    // Language configuration
    get_current_language, set_current_language, DEFAULT_LANGUAGE,
};

// Label resolution
pub use label_resolution::{
    build_alias_to_collection_map, find_needed_collections, is_valid_uuid,
    resolve_labels, resolve_labels_full, ConceptLookup, LabelResolutionConfig,
    LabelResolutionError, DEFAULT_CONFIG_KEYS, DEFAULT_RESOLVABLE_DATATYPES,
};

// JSON conversion (tiles <-> tree)
pub use json_conversion::{
    tiles_to_tree, tree_to_tiles, tree_to_tiles_strict, ResourceData,
};

// Pseudo value core types (for JSON conversion)
pub use pseudo_value_core::{
    matches_semantic_child, PseudoListCore, PseudoValueCore,
    TileBuilder, TileBuilderContext, VisitorContext,
};
