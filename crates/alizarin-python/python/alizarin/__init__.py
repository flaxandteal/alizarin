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

tiles_to_json_tree: Optional[Any] = None
json_tree_to_tiles: Optional[Any] = None
batch_trees_to_tiles: Optional[Any] = None
batch_tiles_to_trees: Optional[Any] = None
_alizarin_rust: Optional[Any] = None

try:
    # Try importing from the built extension (maturin puts it in the package)
    from . import alizarin as _alizarin_rust
    tiles_to_json_tree = _alizarin_rust.tiles_to_json_tree
    json_tree_to_tiles = _alizarin_rust.json_tree_to_tiles
    batch_trees_to_tiles = _alizarin_rust.batch_trees_to_tiles
    batch_tiles_to_trees = _alizarin_rust.batch_tiles_to_trees
except (ImportError, AttributeError):
    # Rust module not yet compiled - functions will remain None
    pass


# =============================================================================
# Rust-backed Wrapper Classes
# =============================================================================

class StaticGraphRust:
    """
    Python wrapper for Rust StaticGraph.
    Handles conversion between Python dicts and JSON strings.

    This is a Rust-backed implementation. For pure Python, use
    static_types.StaticGraph directly.
    """

    def __init__(self, json_str: str) -> None:
        """Create StaticGraph from JSON string."""
        if _alizarin_rust is None:
            raise ImportError("Rust extension not available")
        self._core = _alizarin_rust.StaticGraphCore(json_str)

    @property
    def graphid(self) -> str:
        """Get graph ID."""
        return self._core.graphid

    @property
    def name(self) -> Dict[str, str]:
        """Get graph name as dict."""
        return json.loads(self._core.name_json())

    def to_dict(self) -> Dict[str, Any]:
        """Export graph as dict."""
        return json.loads(self._core.to_json())


class StaticResourceRust:
    """
    Python wrapper for Rust StaticResource.
    Handles conversion between Python dicts and JSON strings.

    This is a Rust-backed implementation. For pure Python, use
    static_types.StaticResource directly.
    """

    def __init__(self, json_str: str) -> None:
        """Create StaticResource from JSON string."""
        if _alizarin_rust is None:
            raise ImportError("Rust extension not available")
        self._core = _alizarin_rust.StaticResourceCore(json_str)

    @property
    def resourceinstanceid(self) -> str:
        """Get resource instance ID."""
        return self._core.resourceinstanceid

    @property
    def graph_id(self) -> str:
        """Get graph ID."""
        return self._core.graph_id

    def get_tiles(self) -> List[Any]:
        """Get tiles as list."""
        return json.loads(self._core.get_tiles_json())

    def to_dict(self) -> Dict[str, Any]:
        """Export resource as dict."""
        return json.loads(self._core.to_json())


# Aliases for backwards compatibility
StaticGraph = StaticGraphRust
StaticResource = StaticResourceRust


# =============================================================================
# Rust Conversion Functions
# =============================================================================

def resource_to_json(
    resource: StaticResourceRust,
    graph: StaticGraphRust,
) -> Dict[str, Any]:
    """
    Convert a tiled resource to nested JSON dict.

    Args:
        resource: StaticResource instance
        graph: StaticGraph model

    Returns:
        Nested dict structure representing the resource tree
    """
    if _alizarin_rust is None:
        raise ImportError("Rust extension not available")
    json_str = _alizarin_rust.resource_to_json_string(resource._core, graph._core)
    return json.loads(json_str)


def json_to_resource(
    json_dict: Dict[str, Any],
    graph: StaticGraphRust,
) -> StaticResourceRust:
    """
    Convert nested JSON dict to a tiled resource.

    Args:
        json_dict: Nested JSON structure
        graph: StaticGraph model

    Returns:
        StaticResource instance
    """
    if _alizarin_rust is None:
        raise ImportError("Rust extension not available")
    json_str = json.dumps(json_dict)
    core_resource = _alizarin_rust.json_string_to_resource(json_str, graph._core)

    # Wrap the core resource
    wrapper = StaticResourceRust.__new__(StaticResourceRust)
    wrapper._core = core_resource
    return wrapper


# =============================================================================
# Static Types (pure Python, matching TypeScript static-types.ts)
# =============================================================================

from .static_types import (
    # Translatable strings
    StaticTranslatableString,
    # Nodes (pure Python versions)
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
    # Note: StaticGraph is the Rust-backed version above
    # Use static_types.StaticGraph for pure Python
    # Resources
    StaticResourceDescriptors,
    StaticResourceMetadata,
    StaticResourceSummary,
    StaticResourceReference,
    # Note: StaticResource is the Rust-backed version above
    # Use static_types.StaticResource for pure Python
    # RDM (Concepts/Collections)
    StaticValue,
    StaticConcept,
    StaticCollection,
    StaticDomainValue,
)

# Import pure Python versions with different names
from .static_types import (
    StaticGraph as StaticGraphPython,
    StaticResource as StaticResourcePython,
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
    FileListViewModel,
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
# Graph Manager (matching TypeScript graphManager.ts)
# =============================================================================

from .graph_manager import (
    # Protocols
    IRIVM,
    IWKRM,
    IModelWrapper,
    IInstanceWrapper,
    IReferenceDataManager,
    IGraphManager,
    # Classes
    WKRM,
    GraphManager,
    StaticStore,
    # Singletons
    graph_manager,
    static_store,
)

# =============================================================================
# Model Wrapper (matching TypeScript graphManager.ts)
# =============================================================================

from .model_wrapper import ResourceModelWrapper

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

register_type_handler = None
has_type_handler = None
coerce_with_extension = None
has_display_renderer = None
render_display_with_extension = None

try:
    from . import alizarin as _alizarin_rust
    if hasattr(_alizarin_rust, 'register_type_handler'):
        register_type_handler = _alizarin_rust.register_type_handler
        has_type_handler = _alizarin_rust.has_type_handler
        coerce_with_extension = _alizarin_rust.coerce_with_extension
        has_display_renderer = _alizarin_rust.has_display_renderer
        render_display_with_extension = _alizarin_rust.render_display_with_extension
except (ImportError, AttributeError):
    pass

# =============================================================================
# Graph Mutation API (JSON-based mutations)
# =============================================================================

from . import alizarin as _alizarin_rust
apply_mutations_from_json = _alizarin_rust.apply_mutations_from_json
apply_mutations_with_extensions = _alizarin_rust.apply_mutations_with_extensions
register_extension_mutation = _alizarin_rust.register_extension_mutation
has_extension_mutation = _alizarin_rust.has_extension_mutation
list_extension_mutations = _alizarin_rust.list_extension_mutations
generate_uuid_v5 = _alizarin_rust.generate_uuid_v5
get_mutation_schema = _alizarin_rust.get_mutation_schema

# =============================================================================
# RDM (Reference Data Manager) Cache
# =============================================================================

from . import alizarin as _alizarin_rust
RustRdmConcept = _alizarin_rust.RustRdmConcept
RustRdmCollection = _alizarin_rust.RustRdmCollection
RustRdmCache = _alizarin_rust.RustRdmCache
set_global_rdm_cache = _alizarin_rust.set_global_rdm_cache
get_global_rdm_cache = _alizarin_rust.get_global_rdm_cache
clear_global_rdm_cache = _alizarin_rust.clear_global_rdm_cache
has_global_rdm_cache = _alizarin_rust.has_global_rdm_cache
resolve_labels = _alizarin_rust.resolve_labels
get_needed_collections = _alizarin_rust.get_needed_collections
is_valid_uuid = _alizarin_rust.is_valid_uuid

# =============================================================================
# Version
# =============================================================================

__version__ = "0.2.1-alpha.8"

# =============================================================================
# Exports
# =============================================================================

__all__ = [
    # Version
    "__version__",
    # Core Rust functions
    "tiles_to_json_tree",
    "json_tree_to_tiles",
    "batch_trees_to_tiles",
    "batch_tiles_to_trees",
    "resource_to_json",
    "json_to_resource",
    # Rust-backed wrappers
    "StaticGraph",
    "StaticResource",
    "StaticGraphRust",
    "StaticResourceRust",
    # Pure Python versions
    "StaticGraphPython",
    "StaticResourcePython",
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
    "StaticResourceDescriptors",
    "StaticResourceMetadata",
    "StaticResourceSummary",
    "StaticResourceReference",
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
    "FileListViewModel",
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
    # Graph Manager
    "IRIVM",
    "IWKRM",
    "IModelWrapper",
    "IInstanceWrapper",
    "IReferenceDataManager",
    "IGraphManager",
    "WKRM",
    "GraphManager",
    "StaticStore",
    "graph_manager",
    "static_store",
    # Model Wrapper
    "ResourceModelWrapper",
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
    # RDM Cache
    "RustRdmConcept",
    "RustRdmCollection",
    "RustRdmCache",
    "set_global_rdm_cache",
    "get_global_rdm_cache",
    "clear_global_rdm_cache",
    "has_global_rdm_cache",
    "resolve_labels",
    "get_needed_collections",
    "is_valid_uuid",
    # Graph Mutation API
    "apply_mutations_from_json",
    "apply_mutations_with_extensions",
    "register_extension_mutation",
    "has_extension_mutation",
    "list_extension_mutations",
    "generate_uuid_v5",
    "get_mutation_schema",
]
