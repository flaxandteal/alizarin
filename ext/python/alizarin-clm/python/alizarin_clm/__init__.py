"""
Alizarin CLM Extension

This extension provides the "reference" datatype for Controlled List Manager integration.

Usage:
    Simply import this module to register the reference datatype:

    >>> import alizarin_clm

    The reference datatype will be automatically registered with alizarin's
    CUSTOM_DATATYPES registry.

Label Resolution:
    CLM provides its own label resolution for 'reference' nodes:

    >>> from alizarin_clm import resolve_reference_labels
    >>> resolved = resolve_reference_labels(tree_json, graph_json)

    This looks for nodes with 'controlledList' in config and resolves
    label strings to UUIDs using the global RDM cache.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

# Import static types
from .static_types import StaticReference, StaticReferenceLabel

# Import view models
from .view_models import (
    ReferenceValueViewModel,
    ReferenceListViewModel,
    ReferenceMergedDataType,
)

__version__ = "0.1.0"


async def resolve_reference_labels(
    tree_json: str,
    graph_json: str,
    rdm_cache: Optional[Any] = None,
    strict: bool = False,
) -> str:
    """
    Resolve label strings to UUIDs for 'reference' nodes in a JSON tree.

    This is the CLM extension's label resolver. It uses the centralized Rust
    implementation for consistent behavior with the JS/WASM version.

    If the cache has a loader set, missing collections will be fetched
    automatically (lazy loading).

    Args:
        tree_json: JSON string of the tree to process
        graph_json: JSON string of the graph definition
        rdm_cache: Optional RdmCache instance. If not provided, uses global cache.
        strict: If True, raise errors for unresolved labels. If False, pass through.

    Returns:
        JSON string with labels resolved to UUIDs

    Raises:
        ValueError: If strict=True and labels cannot be resolved

    Example:
        >>> from alizarin.alizarin import RustRdmCache, RustRdmCollection
        >>> from alizarin_clm import resolve_reference_labels
        >>>
        >>> # With lazy loading
        >>> async def my_loader(collection_id):
        ...     data = await fetch_collection(collection_id)
        ...     return RustRdmCollection.from_json(data)
        >>>
        >>> cache = RustRdmCache(loader=my_loader)
        >>> resolved = await resolve_reference_labels(tree, graph_json, cache)
        >>>
        >>> # Or with pre-loaded cache
        >>> cache = RustRdmCache()
        >>> cache.add_collection(my_collection)
        >>> resolved = await resolve_reference_labels(tree, graph_json, cache)
    """
    # Get cache to use
    cache = rdm_cache
    if cache is None:
        try:
            from alizarin.alizarin import get_global_rdm_cache
            cache = get_global_rdm_cache()
        except ImportError:
            pass

    if cache is None:
        # No cache available, return tree unchanged
        return tree_json

    # Try to use the centralized Rust implementation
    try:
        from alizarin.alizarin import (
            resolve_labels as rust_resolve_labels,
            get_needed_collections,
        )

        # First, fetch any needed collections (lazy loading)
        needed_ids = get_needed_collections(
            tree_json,
            graph_json,
            resolvable_datatypes=["reference"],  # CLM only handles reference
            config_keys=["controlledList", "rdmCollection"],
        )

        # Lazy load any missing collections
        for collection_id in needed_ids:
            if coro := cache.fetch_if_missing(collection_id):
                collection = await coro
                if collection is not None:
                    cache.add_collection(collection)

        # Use the Rust implementation
        resolved_json, _ = rust_resolve_labels(
            tree_json,
            graph_json,
            cache,
            resolvable_datatypes=["reference"],
            config_keys=["controlledList", "rdmCollection"],
            strict=strict,
        )
        return resolved_json

    except ImportError:
        # Rust extension not available, fall back to Python implementation
        return await _resolve_reference_labels_python(
            tree_json, graph_json, cache, strict
        )


async def _resolve_reference_labels_python(
    tree_json: str,
    graph_json: str,
    cache: Any,
    strict: bool,
) -> str:
    """Fallback Python implementation for when Rust extension is unavailable."""
    import json
    import re

    _UUID_PATTERN = re.compile(
        r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',
        re.IGNORECASE
    )

    # Parse inputs
    tree = json.loads(tree_json)
    graph_data = json.loads(graph_json)

    # Handle wrapped graph format
    if "graph" in graph_data and isinstance(graph_data["graph"], list):
        graph = graph_data["graph"][0]
    else:
        graph = graph_data

    # Build alias -> collection_id mapping for reference nodes
    alias_to_config: dict[str, str] = {}
    for node in graph.get("nodes", []):
        alias = node.get("alias")
        datatype = node.get("datatype", "")
        config = node.get("config", {}) or {}

        if alias and datatype == "reference":
            collection_id = config.get("controlledList") or config.get("rdmCollection")
            if collection_id:
                alias_to_config[alias] = collection_id

    if not alias_to_config:
        return tree_json

    # Find which collections are actually needed
    needed_collections: set[str] = set()

    def find_needed(value: Any, alias: Optional[str]) -> None:
        if isinstance(value, dict):
            if "_value" in value:
                find_needed(value["_value"], alias)
            else:
                for k, v in value.items():
                    find_needed(v, k)
        elif isinstance(value, list):
            for item in value:
                find_needed(item, alias)
        elif isinstance(value, str):
            if alias and alias in alias_to_config:
                needed_collections.add(alias_to_config[alias])

    find_needed(tree, None)

    # Lazy load any missing collections
    for collection_id in needed_collections:
        if coro := cache.fetch_if_missing(collection_id):
            collection = await coro
            if collection is not None:
                cache.add_collection(collection)

    # Resolve labels recursively
    errors: list[str] = []

    def resolve_value(value: Any, alias: Optional[str]) -> Any:
        if isinstance(value, dict):
            if "_value" in value:
                value["_value"] = resolve_value(value["_value"], alias)
                return value
            return {k: resolve_value(v, k) for k, v in value.items()}
        elif isinstance(value, list):
            return [resolve_value(item, alias) for item in value]
        elif isinstance(value, str):
            if alias and alias in alias_to_config:
                collection_id = alias_to_config[alias]
                if _UUID_PATTERN.match(value):
                    return value
                concept = cache.lookup_by_label(collection_id, value)
                if concept is not None:
                    return concept.id
                elif strict:
                    errors.append(
                        f"Label '{value}' not found in collection '{collection_id}' "
                        f"for reference field '{alias}'"
                    )
            return value
        else:
            return value

    resolved_tree = resolve_value(tree, None)

    if errors:
        raise ValueError("Failed to resolve reference labels:\n  " + "\n  ".join(errors))

    return json.dumps(resolved_tree)


def _register_rust_handler() -> bool:
    """
    Register the Rust coercion handler with alizarin.

    Returns True if successful, False if Rust extension not available.
    """
    try:
        # Import the Rust extension
        from . import _rust as rust_ext

        # Import alizarin's registration function
        import alizarin
        if not hasattr(alizarin, 'register_type_handler') or alizarin.register_type_handler is None:
            # Rust extension functions not available
            return False

        # Get the capsule and register it
        capsule = rust_ext.get_reference_handler_capsule()
        alizarin.register_type_handler(capsule)
        return True
    except ImportError:
        # Rust extension not built yet - this is fine
        return False
    except Exception as e:
        print(f"Warning: Failed to register CLM Rust handler: {e}")
        return False


def _register_python_handler() -> None:
    """
    Register the Python ViewModel with alizarin's CUSTOM_DATATYPES.

    This is always done, regardless of whether Rust handler is available.
    """
    try:
        from alizarin.view_models import CUSTOM_DATATYPES
        CUSTOM_DATATYPES["reference"] = ReferenceMergedDataType
    except ImportError as e:
        print(f"Warning: Could not register CLM Python handler: {e}")


def _reference_change_collection_handler(graph_json: str, params_json: str) -> str:
    """
    Mutation handler for clm.reference_change_collection.

    Changes the collection (controlledList/rdmCollection) for a reference node.

    Args:
        graph_json: The graph as JSON string
        params_json: Mutation parameters as JSON string
            - node_id: Node ID or alias to modify
            - collection_id: New collection ID
            - config_key: Config key to update (default: "controlledList")

    Returns:
        Modified graph as JSON string
    """
    import json

    graph = json.loads(graph_json)
    params = json.loads(params_json)

    node_id = params["node_id"]
    collection_id = params["collection_id"]
    config_key = params.get("config_key", "controlledList")

    # Find node by ID or alias
    target_node = None
    for node in graph["nodes"]:
        if node.get("nodeid") == node_id or node.get("alias") == node_id:
            target_node = node
            break

    if target_node is None:
        raise ValueError(f"Node not found: {node_id}")

    # Validate it's a reference type
    if target_node["datatype"] not in ("reference", "reference-list"):
        raise ValueError(
            f"Node {node_id} is not a reference type: {target_node['datatype']}"
        )

    # Update config
    if "config" not in target_node or target_node["config"] is None:
        target_node["config"] = {}
    target_node["config"][config_key] = collection_id

    return json.dumps(graph)


def _register_mutation_handler() -> bool:
    """
    Register the CLM extension mutation handler with alizarin.

    Returns True if successful, False if mutation API not available.
    """
    try:
        import alizarin

        if not hasattr(alizarin, 'register_extension_mutation'):
            return False
        if alizarin.register_extension_mutation is None:
            return False

        # Only register if not already registered
        if hasattr(alizarin, 'has_extension_mutation') and alizarin.has_extension_mutation("clm.reference_change_collection"):
            return True

        alizarin.register_extension_mutation(
            "clm.reference_change_collection",
            _reference_change_collection_handler,
            "AlwaysConformant",
        )
        return True
    except ImportError:
        return False
    except Exception as e:
        print(f"Warning: Failed to register CLM mutation handler: {e}")
        return False


# Auto-register on import
_rust_available = _register_rust_handler()
_register_python_handler()
_mutation_available = _register_mutation_handler()


__all__ = [
    # Version
    "__version__",
    # Label resolution
    "resolve_reference_labels",
    # Static types
    "StaticReference",
    "StaticReferenceLabel",
    # View models
    "ReferenceValueViewModel",
    "ReferenceListViewModel",
    "ReferenceMergedDataType",
]
