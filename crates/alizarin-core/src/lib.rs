/// Alizarin Core Library
///
/// Platform-agnostic core functionality that can be used from:
/// - JavaScript/TypeScript via WASM bindings (alizarin-wasm)
/// - Python via PyO3 bindings (alizarin-python)
/// - Native Rust applications (alizarin-explorer)
/// - Other languages via C FFI

pub mod graph;
pub mod graph_mutator;
pub mod interner;
pub mod json_conversion;
pub mod label_resolution;
pub mod loader;
pub mod node_config;
pub mod pseudo_value_core;
pub mod rdm_cache;
pub mod registry;
pub mod skos;
pub mod type_coercion;
pub mod type_serialization;

// Graph types
pub use graph::{
    GraphWrapper, IndexedGraph, StaticEdge, StaticGraph, StaticGraphMeta, StaticNode, StaticNodegroup,
    StaticTranslatableString, ITERABLE_DATATYPES, is_iterable_datatype,
    // Additional Static* types
    StaticConstraint, StaticPublication, StaticCardsXNodesXWidgets, StaticFunctionsXGraphs,
    StaticCard, StaticTile, StaticResourceDescriptors, StaticResourceMetadata,
    StaticResourceSummary, StaticResourceReference, StaticResource,
    // Resource merging
    MergeResult, BatchMergeResult, merge_resources, batch_merge_resources,
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

// Type serialization (output formatting)
pub use type_serialization::{
    // Options and result
    SerializationMode, SerializationOptions, SerializationResult, SerializationContext,
    // Dispatcher functions
    serialize_value, serialize_tile_data, serialize_display,
    // Individual serializers
    serialize_string, serialize_url, serialize_geojson, serialize_non_localized_string,
    serialize_number, serialize_date, serialize_edtf,
    serialize_boolean, serialize_domain_value, serialize_domain_value_list,
    serialize_concept, serialize_concept_list,
    serialize_resource_instance, serialize_resource_instance_list,
    // Resolver types
    DomainValueResolver, ConceptResolver,
    // Extension support
    ExtensionDisplaySerializer, DisplaySerializerRegistry,
};

// Label resolution
pub use label_resolution::{
    build_alias_to_collection_map, find_needed_collections, is_valid_uuid,
    resolve_labels, resolve_labels_full, ConceptLookup, LabelResolutionConfig,
    LabelResolutionError, DEFAULT_CONFIG_KEYS, DEFAULT_RESOLVABLE_DATATYPES,
};

// JSON conversion (tiles <-> tree)
pub use json_conversion::{
    tiles_to_tree, tree_to_tiles, tree_to_tiles_strict,
    tree_to_tiles_with_id_key, tree_to_tiles_strict_with_id_key,
    BusinessData, BusinessDataWrapper, create_static_resource,
};

// Pseudo value core types (for JSON conversion)
pub use pseudo_value_core::{
    matches_semantic_child, matches_tile_filter, PseudoListCore, PseudoValueCore,
    TileBuilder, TileBuilderContext, VisitorContext,
};

// RDM cache types
pub use rdm_cache::{RdmCache, RdmCollection, RdmConcept};

// Graph mutator types (builder pattern for graph construction)
pub use graph_mutator::{
    // Builder and options
    GraphMutator, MutatorOptions,
    // Mutation types (Command pattern)
    GraphMutation, AddNodeParams, AddNodegroupParams, AddEdgeParams,
    AddCardParams, AddWidgetParams, NodeOptions, CardOptions,
    RenameNodeParams, RenameGraphParams,
    // Extension mutations
    ExtensionMutationParams, ExtensionMutationHandler, ExtensionMutationRegistry,
    MutationConformance,
    // JSON-based mutation API
    MutationRequest, MutationRequestOptions,
    apply_mutations_from_json, apply_mutations, mutations_to_json, get_mutation_schema,
    // Extension-aware mutation API
    apply_mutations_with_extensions, apply_mutations_from_json_with_extensions,
    // Widget types
    Widget, CardComponent, default_card_component,
    get_default_widget_for_datatype, WIDGETS, DEFAULT_CARD_COMPONENT_ID,
    // Utilities
    Cardinality, MutationError, generate_uuid_v5, slugify,
};

// Graph registry (for looking up graphs by graph_id)
pub use registry::{
    register_graph, register_graph_owned, get_graph, is_graph_registered,
    unregister_graph, clear_registry, registry_size,
};
