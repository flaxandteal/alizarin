/// Alizarin Core Library
///
/// Platform-agnostic core functionality that can be used from:
/// - JavaScript/TypeScript via WASM bindings (alizarin-wasm)
/// - Python via PyO3 bindings (alizarin-python)
/// - Native Rust applications (alizarin-explorer)
/// - Other languages via C FFI
pub mod extension_type_registry;
pub mod graph;
pub mod graph_mutator;
pub mod instance_wrapper_core;
pub mod interner;
pub mod json_conversion;
pub mod label_resolution;
pub mod loader;
pub mod node_config;
pub mod permissions;
pub mod pseudo_value_core;
pub mod rdm_cache;
pub mod rdm_namespace;
pub mod registry;
pub mod skos;
pub mod string_utils;
pub mod type_coercion;
pub mod type_serialization;

// Graph types
pub use graph::{
    batch_merge_resources,
    build_backedges,
    find_root_node,
    is_iterable_datatype,
    merge_resources,
    // Graph pruning
    prune_graph,
    BatchMergeResult,
    GraphWrapper,
    IndexedGraph,
    // Resource merging
    MergeResult,
    PruneError,
    StaticCard,
    StaticCardsXNodesXWidgets,
    // Additional Static* types
    StaticConstraint,
    StaticEdge,
    StaticFunctionsXGraphs,
    StaticGraph,
    StaticGraphMeta,
    StaticNode,
    StaticNodegroup,
    StaticPublication,
    StaticResource,
    StaticResourceDescriptors,
    StaticResourceMetadata,
    StaticResourceReference,
    StaticResourceSummary,
    StaticTile,
    StaticTranslatableString,
    ITERABLE_DATATYPES,
};

// Loader
pub use loader::{LoaderError, PrebuildInfo, PrebuildLoader};

// Interner types
#[cfg(feature = "multi-threaded")]
pub use interner::ThreadSafeInterner;
pub use interner::{InternedId, Interner, InternerExt};

// Node config types
pub use node_config::{
    NodeConfig, NodeConfigBoolean, NodeConfigConcept, NodeConfigDomain, NodeConfigManager,
    NodeConfigReference, StaticDomainValue,
};

// Type coercion
pub use type_coercion::{
    // Phase 3: Config-dependent types
    coerce_boolean,
    coerce_concept_list,
    // Phase 4: RDM-dependent types
    coerce_concept_value,
    // Phase 1: Simple scalars
    coerce_date,
    coerce_domain_value,
    coerce_domain_value_list,
    coerce_edtf,
    // Phase 2: Dict types
    coerce_geojson,
    coerce_non_localized_string,
    coerce_number,
    // Phase 5: Format normalization
    coerce_resource_instance,
    coerce_resource_instance_list,
    coerce_string,
    coerce_url,
    // Dispatcher and result
    coerce_value,
    // Language configuration
    get_current_language,
    set_current_language,
    CoercionResult,
    DEFAULT_LANGUAGE,
};

// Type serialization (output formatting)
pub use type_serialization::{
    serialize_boolean,
    serialize_concept,
    serialize_concept_list,
    serialize_date,
    serialize_display,
    serialize_domain_value,
    serialize_domain_value_list,
    serialize_edtf,
    serialize_geojson,
    serialize_non_localized_string,
    serialize_number,
    serialize_resource_instance,
    serialize_resource_instance_list,
    // Individual serializers
    serialize_string,
    serialize_tile_data,
    serialize_url,
    // Dispatcher functions
    serialize_value,
    ConceptResolver,
    DisplaySerializerRegistry,
    // Resolver types
    DomainValueResolver,
    // Extension support
    ExtensionDisplaySerializer,
    SerializationContext,
    // Options and result
    SerializationMode,
    SerializationOptions,
    SerializationResult,
};

// Label resolution
pub use label_resolution::{
    build_alias_to_collection_map, find_needed_collections, is_valid_uuid, resolve_labels,
    resolve_labels_full, ConceptLookup, LabelResolutionConfig, LabelResolutionError,
    DEFAULT_CONFIG_KEYS, DEFAULT_RESOLVABLE_DATATYPES,
};

// JSON conversion (tiles <-> tree)
pub use json_conversion::{
    create_static_resource, tiles_to_tree, tree_to_tiles, BusinessData, BusinessDataWrapper,
};

// Pseudo value core types (for JSON conversion)
pub use pseudo_value_core::{
    matches_tile_filter, PseudoListCore, PseudoValueCore, TileBuilder, TileBuilderContext,
    VisitorContext,
};

// Instance wrapper core types (for populate, semantic traversal)
pub use instance_wrapper_core::{
    is_node_single_cardinality, is_node_single_cardinality_with, matches_semantic_child,
    EnsureNodegroupResult, LoadState, ModelAccess, PopulateResult, ResourceInstanceWrapperCore,
    SemanticChildError, SemanticChildResult, ValuesFromNodegroupResult,
};

// RDM cache types
pub use rdm_cache::{RdmCache, RdmCollection, RdmConcept};

// RDM namespace utilities (deterministic UUID generation)
pub use rdm_namespace::{
    generate_collection_uuid, generate_concept_uuid, generate_concept_uuid_from_str,
    generate_value_uuid, labels_to_deterministic_string, parse_rdm_namespace,
};

// Graph mutator types (builder pattern for graph construction)
pub use graph_mutator::{
    apply_mutations,
    apply_mutations_create_from_json,
    apply_mutations_from_json,
    apply_mutations_from_json_with_extensions,
    // Extension-aware mutation API
    apply_mutations_with_extensions,
    default_card_component,
    generate_uuid_v5,
    get_default_widget_for_datatype,
    get_mutation_schema,
    mutations_to_json,
    slugify,
    AddCardParams,
    AddEdgeParams,
    AddNodeParams,
    AddNodegroupParams,
    AddWidgetParams,
    CardComponent,
    CardOptions,
    // Utilities
    Cardinality,
    CreateGraphParams,
    ExtensionMutationHandler,
    // Extension mutations
    ExtensionMutationParams,
    ExtensionMutationRegistry,
    // Mutation types (Command pattern)
    GraphMutation,
    // Builder and options
    GraphMutator,
    MutationConformance,
    MutationError,
    // JSON-based mutation API
    MutationRequest,
    MutationRequestOptions,
    MutatorOptions,
    NodeOptions,
    RenameGraphParams,
    RenameNodeParams,
    // Widget types
    Widget,
    DEFAULT_CARD_COMPONENT_ID,
    WIDGETS,
};

// Graph registry (for looking up graphs by graph_id)
pub use registry::{
    clear_registry, get_graph, is_graph_registered, register_graph, register_graph_owned,
    registry_size, unregister_graph,
};

// List datatype registry (for datatypes where array IS the value)
pub use registry::{
    is_list_datatype, list_datatypes, register_list_datatype, unregister_list_datatype,
};

// Widget mapping registry (for extension datatype -> widget mappings)
pub use registry::{
    get_widget_for_datatype, register_widget_for_datatype, unregister_widget_for_datatype,
    widget_mappings,
};

// Widget registry (for extension widget definitions)
pub use registry::{
    get_registered_widget, register_widget, registered_widgets, unregister_widget, RegisteredWidget,
};

// Permission rules (for conditional tile filtering)
pub use permissions::{evaluate_tile_path, PermissionRule};

// String utilities
pub use string_utils::{camel_to_snake, transform_keys_to_snake};

// Extension type registry (unified handler infrastructure for WASM/Python)
pub use extension_type_registry::{
    ExtensionError, ExtensionTypeHandler, ExtensionTypeRegistry, HandlerCapabilities,
};
