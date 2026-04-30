"""
Alizarin Python Bindings

This package provides Python bindings for Alizarin, matching the TypeScript/JavaScript
API for working with Arches cultural heritage data.

The architecture implements a three-layer caching system:
1. Rust pseudo_cache - authoritative source of truth for tile data
2. Python _pseudo_cache - cached wrapped Python objects (avoids repeated Rust calls)
3. Python value_cache - JSON-serializable cache for performance
"""

from __future__ import annotations

import json
from typing import Any, Dict, List, Optional, TYPE_CHECKING

# =============================================================================
# Rust Extension Import
# =============================================================================

register_graph: Optional[Any] = None
get_graph_json: Optional[Any] = None
get_graph_schema: Optional[Any] = None
tiles_to_json_tree: Optional[Any] = None
build_tree_from_tiles: Optional[Any] = None
json_tree_to_tiles: Optional[Any] = None
batch_trees_to_tiles: Optional[Any] = None
batch_tiles_to_trees: Optional[Any] = None
merge_resources: Optional[Any] = None
batch_merge_resources: Optional[Any] = None
streamed_merge_from_files: Optional[Any] = None
TreeToTilesIterator: Optional[Any] = None
ResourceRegistry: Optional[Any] = None
resolve_labels_in_tree: Optional[Any] = None
set_descriptor_template: Optional[Any] = None
_alizarin_rust: Optional[Any] = None

try:
    # Try importing from the built extension (maturin puts it in the package)
    from . import alizarin as _alizarin_rust
    register_graph = _alizarin_rust.register_graph
    get_graph_json = _alizarin_rust.get_graph_json
    get_graph_schema = _alizarin_rust.get_graph_schema
    tiles_to_json_tree = _alizarin_rust.tiles_to_json_tree
    build_tree_from_tiles = _alizarin_rust.build_tree_from_tiles
    json_tree_to_tiles = _alizarin_rust.json_tree_to_tiles
    batch_trees_to_tiles = _alizarin_rust.batch_trees_to_tiles
    batch_tiles_to_trees = _alizarin_rust.batch_tiles_to_trees
    merge_resources = _alizarin_rust.merge_resources
    batch_merge_resources = _alizarin_rust.batch_merge_resources
    streamed_merge_from_files = _alizarin_rust.streamed_merge_from_files
    TreeToTilesIterator = _alizarin_rust.TreeToTilesIterator
    ResourceRegistry = _alizarin_rust.ResourceRegistry
    resolve_labels_in_tree = _alizarin_rust.resolve_labels_in_tree
    set_descriptor_template = _alizarin_rust.set_descriptor_template
except (ImportError, AttributeError):
    # Rust module not yet compiled - functions will remain None
    pass


# =============================================================================
# Chunked Merge (memory-efficient batch merging)
# =============================================================================

def chunked_merge_resources(
    json_strings: List[str],
    chunk_size: int = 10,
    recompute_descriptors: bool = True,
    strict: bool = True,
    on_chunk_complete: Optional[Any] = None,
) -> Dict[str, Any]:
    """
    Memory-efficient alternative to batch_merge_resources for large datasets.

    Processes json_strings in chunks of chunk_size, merging progressively.
    Optional on_chunk_complete(chunk_index, total_chunks) callback for progress.

    If strict=True (default), raises ValueError on conflicting data when unifying
    cardinality-1 tiles. Identical data and gap-filling are always allowed.
    """
    if batch_merge_resources is None:
        raise RuntimeError("Alizarin Rust extension not loaded")

    if not json_strings:
        return {"resources": [], "merge_stats": {"total_input": 0, "duplicates_removed": 0}}

    total_chunks = (len(json_strings) + chunk_size - 1) // chunk_size
    merged = None

    for chunk_idx in range(0, len(json_strings), chunk_size):
        chunk_strings = json_strings[chunk_idx:chunk_idx + chunk_size]

        chunk_result = batch_merge_resources(
            chunk_strings,
            recompute_descriptors=False,
            strict=strict,
        )

        if merged is None:
            merged = chunk_result
        else:
            merged_json = json.dumps({"business_data": {"resources": merged["resources"]}})
            chunk_json = json.dumps({"business_data": {"resources": chunk_result["resources"]}})
            merged = batch_merge_resources(
                [merged_json, chunk_json],
                recompute_descriptors=False,
                strict=strict,
            )

        if on_chunk_complete is not None:
            on_chunk_complete(chunk_idx // chunk_size + 1, total_chunks)

    if recompute_descriptors and merged and merged["resources"]:
        final_json = json.dumps({"business_data": {"resources": merged["resources"]}})
        merged = batch_merge_resources([final_json], recompute_descriptors=True, strict=strict)

    return merged or {"resources": [], "merge_stats": {"total_input": 0, "duplicates_removed": 0}}


# =============================================================================
# Static Types (pure Python, matching TypeScript static-types.ts)
# =============================================================================

from .static_types import (
    # Translatable strings
    StaticTranslatableString,
    # Nodes
    StaticNode,
    StaticEdge,
    StaticNodegroup,
    StaticConstraint,
    StaticCard,
    # Tiles
    StaticTile,
    # Publication
    StaticPublication,
    # Graph metadata
    StaticGraphMeta,
    StaticFunctionsXGraphs,
    StaticCardsXNodesXWidgets,
    # Graph
    StaticGraph,
    # Resources
    StaticResourceDescriptors,
    StaticResourceMetadata,
    StaticResourceSummary,
    StaticResourceReference,
    StaticResource,
    # RDM (Concepts/Collections)
    StaticValue,
    StaticConcept,
    StaticCollection,
    StaticDomainValue,
)

# =============================================================================
# Pseudo Values (matching TypeScript pseudos.ts)
# =============================================================================

from .pseudos import (
    # Protocol
    IPseudo,
    # Classes
    PseudoValue,
    PseudoList,
    # Factory functions
    wrap_rust_pseudo,
    create_pseudo_value,
    create_pseudo_list,
)

# =============================================================================
# Semantic ViewModel (matching TypeScript semantic.ts)
# =============================================================================

from .semantic import SemanticViewModel

# =============================================================================
# View Models (matching TypeScript viewModels.ts)
# =============================================================================

from .view_models import (
    # Protocol
    IViewModel,
    # Factory function
    get_view_model,
    # Constants
    DEFAULT_LANGUAGE,
    CUSTOM_DATATYPES,
    # ViewContext
    ViewContext,
    view_context,
    # List ViewModels
    ResourceInstanceListViewModel,
    ConceptListViewModel,
    DomainValueListViewModel,
    # Value ViewModels
    StringViewModel,
    DateViewModel,
    EDTFViewModel,
    NumberViewModel,
    BooleanViewModel,
    UrlViewModel,
    NonLocalizedStringViewModel,
    GeoJSONViewModel,
    # Reference ViewModels
    ResourceInstanceViewModel,
    ConceptValueViewModel,
    DomainValueViewModel,
    # Cache entries
    ConceptListCacheEntry,
    ConceptValueCacheEntry,
    ResourceInstanceCacheEntry,
    ResourceInstanceListCacheEntry,
)

# =============================================================================
# Graph Manager Protocols (matching TypeScript interfaces.ts)
# =============================================================================

from .graph_manager import (
    # Protocols
    IRIVM,
    IWKRM,
    IModelWrapper,
    IInstanceWrapper,
    IReferenceDataManager,
    # Classes
    WKRM,
)

# =============================================================================
# Model Wrapper (matching TypeScript graphManager.ts)
# =============================================================================

from .model_wrapper import ResourceModelWrapper


def load_model(graph_id: str, use_rust_cache: bool = True) -> ResourceModelWrapper:
    """
    Load a ResourceModelWrapper from a registered graph ID.

    Convenience function that handles all the boilerplate of loading a graph
    from the Rust registry and setting up the model wrapper.

    Args:
        graph_id: The graph ID returned by register_graph()
        use_rust_cache: Whether to use Rust-side caching (default True)

    Returns:
        A fully initialized ResourceModelWrapper with nodes built

    Example:
        >>> import alizarin
        >>> graph_id = alizarin.register_graph(graph_json)
        >>> model = alizarin.load_model(graph_id)
        >>> # model is ready to use
    """
    return ResourceModelWrapper.from_graph_id(graph_id, use_rust_cache=use_rust_cache)

# =============================================================================
# Instance Wrapper (matching TypeScript graphManager.ts)
# =============================================================================

from .instance_wrapper import (
    ResourceInstanceWrapper,
    ResourceInstanceWrapperCore,
    GetMeta,
)

# =============================================================================
# Node Config (matching TypeScript nodeConfig.ts)
# =============================================================================

from .node_config import (
    INodeConfig,
    StaticNodeConfigBoolean,
    StaticNodeConfigConcept,
    StaticNodeConfigDomain,
    StaticNodeConfigReference,
    NodeConfigManager,
    nodeConfigManager,
)

# =============================================================================
# Extension Registration (for CLM and other extensions)
# =============================================================================

from . import alizarin as _alizarin_rust
register_type_handler = _alizarin_rust.register_type_handler
has_type_handler = _alizarin_rust.has_type_handler
coerce_with_extension = _alizarin_rust.coerce_with_extension
has_display_renderer = _alizarin_rust.has_display_renderer
render_display_with_extension = _alizarin_rust.render_display_with_extension
get_registered_extension_handlers = _alizarin_rust.get_registered_extension_handlers

# =============================================================================
# Graph Mutation API (JSON-based mutations)
# =============================================================================

from . import alizarin as _alizarin_rust
apply_mutations_from_json = _alizarin_rust.apply_mutations_from_json
apply_mutations_create = _alizarin_rust.apply_mutations_create
apply_mutations_with_extensions = _alizarin_rust.apply_mutations_with_extensions
register_extension_mutation = _alizarin_rust.register_extension_mutation
has_extension_mutation = _alizarin_rust.has_extension_mutation
list_extension_mutations = _alizarin_rust.list_extension_mutations
generate_uuid_v5 = _alizarin_rust.generate_uuid_v5
get_mutation_schema = _alizarin_rust.get_mutation_schema
build_graph_from_instructions = _alizarin_rust.build_graph_from_instructions
build_graph_from_csv = _alizarin_rust.build_graph_from_csv
OntologyValidator = _alizarin_rust.OntologyValidator

# =============================================================================
# List Datatype Registry (for datatypes where array IS the value)
# =============================================================================

from . import alizarin as _alizarin_rust
register_list_datatype = _alizarin_rust.register_list_datatype
is_list_datatype = _alizarin_rust.is_list_datatype
list_datatypes = _alizarin_rust.list_datatypes

# =============================================================================
# Widget Registry (for extension datatype -> widget mappings)
# =============================================================================

register_widget_for_datatype = _alizarin_rust.register_widget_for_datatype
get_widget_for_datatype = _alizarin_rust.get_widget_for_datatype
register_widget = _alizarin_rust.register_widget
registered_widgets = _alizarin_rust.registered_widgets
widget_mappings = _alizarin_rust.widget_mappings

# =============================================================================
# RDM (Reference Data Manager) Cache
# =============================================================================

RustRdmConcept = _alizarin_rust.RustRdmConcept
RustRdmCollection = _alizarin_rust.RustRdmCollection
RustRdmCache = _alizarin_rust.RustRdmCache
set_global_rdm_cache = _alizarin_rust.set_global_rdm_cache
get_global_rdm_cache = _alizarin_rust.get_global_rdm_cache
clear_global_rdm_cache = _alizarin_rust.clear_global_rdm_cache
has_global_rdm_cache = _alizarin_rust.has_global_rdm_cache
add_collection_to_global_cache = _alizarin_rust.add_collection_to_global_cache
add_from_skos_xml_to_global_cache = _alizarin_rust.add_from_skos_xml_to_global_cache
add_from_skos_json_to_global_cache = _alizarin_rust.add_from_skos_json_to_global_cache
update_collection_in_global_cache = _alizarin_rust.update_collection_in_global_cache
update_collection_nested_in_global_cache = _alizarin_rust.update_collection_nested_in_global_cache
# RDM namespace configuration for deterministic UUID generation
set_rdm_namespace = _alizarin_rust.set_rdm_namespace
get_rdm_namespace = _alizarin_rust.get_rdm_namespace
has_rdm_namespace = _alizarin_rust.has_rdm_namespace
clear_rdm_namespace = _alizarin_rust.clear_rdm_namespace
resolve_labels = _alizarin_rust.resolve_labels
get_needed_collections = _alizarin_rust.get_needed_collections
is_valid_uuid = _alizarin_rust.is_valid_uuid

# =============================================================================
# Version
# =============================================================================

__version__ = "0.2.1-alpha.81"

# =============================================================================
# Exports
# =============================================================================

__all__ = [
    # Version
    "__version__",
    # Graph registry
    "register_graph",
    "get_graph_json",
    "get_graph_schema",
    # Core Rust functions
    "tiles_to_json_tree",
    "build_tree_from_tiles",
    "json_tree_to_tiles",
    "batch_trees_to_tiles",
    "batch_tiles_to_trees",
    "merge_resources",
    "batch_merge_resources",
    "streamed_merge_from_files",
    "chunked_merge_resources",
    "TreeToTilesIterator",
    "ResourceRegistry",
    "resolve_labels_in_tree",
    "set_descriptor_template",
    # Static Types
    "StaticTranslatableString",
    "StaticNode",
    "StaticEdge",
    "StaticNodegroup",
    "StaticConstraint",
    "StaticCard",
    "StaticTile",
    "StaticPublication",
    "StaticGraphMeta",
    "StaticFunctionsXGraphs",
    "StaticCardsXNodesXWidgets",
    "StaticGraph",
    "StaticResourceDescriptors",
    "StaticResourceMetadata",
    "StaticResourceSummary",
    "StaticResourceReference",
    "StaticResource",
    "StaticValue",
    "StaticConcept",
    "StaticCollection",
    "StaticDomainValue",
    # Pseudos
    "IPseudo",
    "PseudoValue",
    "PseudoList",
    "wrap_rust_pseudo",
    "create_pseudo_value",
    "create_pseudo_list",
    # Semantic
    "SemanticViewModel",
    # ViewModels
    "IViewModel",
    "get_view_model",
    "DEFAULT_LANGUAGE",
    "CUSTOM_DATATYPES",
    "ViewContext",
    "view_context",
    "ResourceInstanceListViewModel",
    "ConceptListViewModel",
    "DomainValueListViewModel",
    "StringViewModel",
    "DateViewModel",
    "EDTFViewModel",
    "NumberViewModel",
    "BooleanViewModel",
    "UrlViewModel",
    "NonLocalizedStringViewModel",
    "GeoJSONViewModel",
    "ResourceInstanceViewModel",
    "ConceptValueViewModel",
    "DomainValueViewModel",
    "ConceptListCacheEntry",
    "ConceptValueCacheEntry",
    "ResourceInstanceCacheEntry",
    "ResourceInstanceListCacheEntry",
    # Graph Manager Protocols
    "IRIVM",
    "IWKRM",
    "IModelWrapper",
    "IInstanceWrapper",
    "IReferenceDataManager",
    "WKRM",
    # Model Wrapper
    "ResourceModelWrapper",
    "load_model",
    # Instance Wrapper
    "ResourceInstanceWrapper",
    "ResourceInstanceWrapperCore",
    "GetMeta",
    # Node Config
    "INodeConfig",
    "StaticNodeConfigBoolean",
    "StaticNodeConfigConcept",
    "StaticNodeConfigDomain",
    "StaticNodeConfigReference",
    "NodeConfigManager",
    "nodeConfigManager",
    # Extension Registration
    "register_type_handler",
    "has_type_handler",
    "coerce_with_extension",
    "has_display_renderer",
    "render_display_with_extension",
    "get_registered_extension_handlers",
    # List Datatype Registry
    "register_list_datatype",
    "is_list_datatype",
    "list_datatypes",
    # Widget Registry
    "register_widget_for_datatype",
    "get_widget_for_datatype",
    "register_widget",
    "registered_widgets",
    "widget_mappings",
    # RDM Cache
    "RustRdmConcept",
    "RustRdmCollection",
    "RustRdmCache",
    "set_global_rdm_cache",
    "get_global_rdm_cache",
    "clear_global_rdm_cache",
    "has_global_rdm_cache",
    "add_collection_to_global_cache",
    "add_from_skos_xml_to_global_cache",
    "add_from_skos_json_to_global_cache",
    "update_collection_in_global_cache",
    "update_collection_nested_in_global_cache",
    # RDM Namespace (for deterministic UUID generation)
    "set_rdm_namespace",
    "get_rdm_namespace",
    "has_rdm_namespace",
    "clear_rdm_namespace",
    "resolve_labels",
    "get_needed_collections",
    "is_valid_uuid",
    # Graph Mutation API
    "apply_mutations_from_json",
    "apply_mutations_create",
    "apply_mutations_with_extensions",
    "register_extension_mutation",
    "has_extension_mutation",
    "list_extension_mutations",
    "generate_uuid_v5",
    "get_mutation_schema",
    "build_graph_from_instructions",
    "build_graph_from_csv",
    # Ontology Validation
    "OntologyValidator",
]
