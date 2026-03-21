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

from typing import Any, Optional  # noqa: F401 - used in type annotations

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


async def resolve_reference_markers(
    business_data_json: str,
    graph_json: str,
    rdm_cache: Optional[Any] = None,
    strict: bool = False,
) -> str:
    """
    Resolve __needs_rdm_lookup and __needs_rdm_label_lookup markers in tile data
    to full StaticReference objects with embedded labels.

    This should be called after batch_trees_to_tiles to resolve
    any markers that were created during coercion. By resolving at write time,
    display-time collection fetching is avoided.

    Args:
        business_data_json: JSON string of business_data result from batch_trees_to_tiles
        graph_json: JSON string of the graph definition
        rdm_cache: Optional RdmCache instance. If not provided, uses global cache.
        strict: If True, raise errors for unresolved markers. If False, pass through.

    Returns:
        JSON string with markers resolved to full StaticReference objects

    Raises:
        ValueError: If strict=True and markers cannot be resolved

    Example:
        >>> from alizarin import batch_trees_to_tiles
        >>> from alizarin_clm import resolve_reference_markers
        >>>
        >>> result = batch_trees_to_tiles(trees_json, graph_id)
        >>> resolved = await resolve_reference_markers(
        ...     json.dumps(result),
        ...     graph_json,
        ...     rdm_cache
        ... )
    """
    import json
    import re
    import uuid as uuid_module

    _UUID_PATTERN = re.compile(
        r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',
        re.IGNORECASE
    )

    # Get cache to use
    cache = rdm_cache
    if cache is None:
        try:
            from alizarin import get_global_rdm_cache
            cache = get_global_rdm_cache()
        except ImportError:
            pass

    if cache is None:
        # No cache available, return unchanged
        return business_data_json

    # Parse inputs
    business_data = json.loads(business_data_json)
    graph_data = json.loads(graph_json)

    # Handle wrapped graph format
    if "graph" in graph_data and isinstance(graph_data["graph"], list):
        graph = graph_data["graph"][0]
    else:
        graph = graph_data

    # Build node_id -> config mapping for reference nodes
    node_configs: dict[str, dict] = {}
    for node in graph.get("nodes", []):
        nodeid = node.get("nodeid")
        datatype = node.get("datatype", "")
        config = node.get("config", {}) or {}

        if nodeid and datatype in ("reference", "reference-list"):
            node_configs[nodeid] = config

    if not node_configs:
        return business_data_json

    # Collect all needed collections from markers
    needed_collections: set[str] = set()
    resources = business_data.get("business_data", {}).get("resources", [])

    for resource in resources:
        tiles = resource.get("tiles", [])
        for tile in tiles:
            data = tile.get("data", {})
            for node_id, value in data.items():
                if node_id not in node_configs:
                    continue

                config = node_configs[node_id]
                collection_id = config.get("controlledList") or config.get("rdmCollection")
                if not collection_id:
                    continue

                # Check for markers in value (could be array or single)
                values = value if isinstance(value, list) else [value]
                for v in values:
                    if isinstance(v, dict):
                        if v.get("__needs_rdm_lookup") or v.get("__needs_rdm_label_lookup"):
                            needed_collections.add(collection_id)

    # Lazy load any missing collections
    for collection_id in needed_collections:
        if hasattr(cache, 'fetch_if_missing'):
            if coro := cache.fetch_if_missing(collection_id):
                collection = await coro
                if collection is not None:
                    cache.add_collection(collection)

    # Resolve markers
    errors: list[str] = []

    def build_static_reference(concept: Any, collection_id: str) -> dict:
        """Build a StaticReference dict from an RDM concept."""
        labels = []
        # Get pref_label - it's a dict of language -> label
        pref_label = getattr(concept, 'pref_label', {}) or {}
        for lang, label_value in pref_label.items():
            labels.append({
                "id": str(uuid_module.uuid4()),  # Generate label ID
                "language_id": lang,
                "list_item_id": concept.id,
                "value": label_value,
                "valuetype_id": "prefLabel",
            })

        return {
            "uri": getattr(concept, 'uri', '') or concept.id,
            "list_id": collection_id,
            "labels": labels,
        }

    def resolve_marker(marker: dict, node_id: str, config: dict) -> dict:
        """Resolve a single marker to a StaticReference."""
        collection_id = config.get("controlledList") or config.get("rdmCollection")
        if not collection_id:
            if strict:
                errors.append(f"Node {node_id}: No collection configured")
            return marker

        # Get collection from cache
        collection = cache.get_collection(collection_id) if hasattr(cache, 'get_collection') else None
        if collection is None:
            if strict:
                errors.append(f"Node {node_id}: Collection {collection_id} not found in cache")
            return marker

        if marker.get("__needs_rdm_lookup") and marker.get("uuid"):
            # Look up by UUID
            concept_id = marker["uuid"]
            concept = collection.get_concept(concept_id) if hasattr(collection, 'get_concept') else None
            if concept is None:
                if strict:
                    errors.append(f"Node {node_id}: Concept {concept_id} not found in collection {collection_id}")
                return marker
            return build_static_reference(concept, collection_id)

        elif marker.get("__needs_rdm_label_lookup") and marker.get("label"):
            # Look up by label
            label = marker["label"]
            # Use the cache's lookup_by_label method
            concept = cache.lookup_by_label(collection_id, label) if hasattr(cache, 'lookup_by_label') else None
            if concept is None:
                if strict:
                    errors.append(f"Node {node_id}: Label '{label}' not found in collection {collection_id}")
                return marker
            return build_static_reference(concept, collection_id)

        return marker

    def resolve_value(value: Any, node_id: str, config: dict) -> Any:
        """Recursively resolve markers in a value."""
        if isinstance(value, dict):
            if value.get("__needs_rdm_lookup") or value.get("__needs_rdm_label_lookup"):
                return resolve_marker(value, node_id, config)
            # Already a full reference or other object - return as-is
            return value
        elif isinstance(value, list):
            return [resolve_value(item, node_id, config) for item in value]
        else:
            return value

    # Process all tiles
    for resource in resources:
        tiles = resource.get("tiles", [])
        for tile in tiles:
            data = tile.get("data", {})
            for node_id, value in list(data.items()):
                if node_id not in node_configs:
                    continue
                config = node_configs[node_id]
                data[node_id] = resolve_value(value, node_id, config)

    if errors:
        raise ValueError("Failed to resolve reference markers:\n  " + "\n  ".join(errors))

    return json.dumps(business_data)


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
        raise RuntimeError(f"Failed to register CLM Rust handler: {e}") from e


def _register_python_handler() -> None:
    """
    Register the Python ViewModel with alizarin's CUSTOM_DATATYPES.

    This is always done, regardless of whether Rust handler is available.
    """
    try:
        from alizarin.view_models import CUSTOM_DATATYPES
        CUSTOM_DATATYPES["reference"] = ReferenceMergedDataType
    except ImportError as e:
        raise ImportError(f"Could not register CLM Python handler: {e}") from e


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
        raise RuntimeError(f"Failed to register CLM mutation handler: {e}") from e


def _register_list_datatype() -> bool:
    """
    Register 'reference' as a list datatype with alizarin.

    List datatypes have arrays that should be treated as the value itself,
    not iterated over during tree-to-tiles conversion.

    Returns True if successful, False if API not available.
    """
    try:
        import alizarin

        if not hasattr(alizarin, 'register_list_datatype'):
            return False
        if alizarin.register_list_datatype is None:
            return False

        alizarin.register_list_datatype("reference")
        return True
    except ImportError:
        return False
    except Exception:
        return False


def _register_widgets() -> bool:
    """
    Register CLM widgets with alizarin's widget registry.

    This allows the mutation system to find the correct widgets when
    creating nodes with 'reference' or 'reference-list' datatypes.

    Returns True if successful, False if API not available.
    """
    try:
        import alizarin

        if not hasattr(alizarin, 'register_widget'):
            return False
        if alizarin.register_widget is None:
            return False

        # Reference select widget (from arches-controlled-lists)
        # Widget ID: 19e56148-82b8-47eb-b66e-f6243639a1a8
        # Widget name: reference-select-widget
        alizarin.register_widget(
            "19e56148-82b8-47eb-b66e-f6243639a1a8",
            "reference-select-widget",
            "reference",
            '{"placeholder": "Select an option", "i18n_properties": ["placeholder"]}'
        )

        # Map 'reference' datatype to reference-select-widget
        alizarin.register_widget_for_datatype("reference", "reference-select-widget")

        # Map 'reference-list' to the same widget (handles both single and multi)
        alizarin.register_widget_for_datatype("reference-list", "reference-select-widget")

        return True
    except ImportError:
        return False
    except Exception:
        return False


# Auto-register on import
_rust_available = _register_rust_handler()
_register_python_handler()
_mutation_available = _register_mutation_handler()
_list_datatype_registered = _register_list_datatype()
_widgets_registered = _register_widgets()


__all__ = [
    # Version
    "__version__",
    # Label resolution
    "resolve_reference_labels",
    # Marker resolution (for write-time resolution)
    "resolve_reference_markers",
    # Static types
    "StaticReference",
    "StaticReferenceLabel",
    # View models
    "ReferenceValueViewModel",
    "ReferenceListViewModel",
    "ReferenceMergedDataType",
]
