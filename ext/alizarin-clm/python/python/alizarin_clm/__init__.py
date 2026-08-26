"""
Alizarin CLM Extension

This extension provides the "reference" datatype for Controlled List Manager integration.

Usage:
    Simply import this module to register the reference datatype:

    >>> import alizarin_clm

    The reference datatype will be automatically registered with alizarin's
    CUSTOM_DATATYPES registry.
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
    import warnings
    warnings.warn(
        "resolve_reference_markers is deprecated: resolve during coercion with "
        "batch_trees_to_tiles(..., resolve_markers=True) instead, which dispatches "
        "to the CLM handler (deterministic IDs, one pass, no second phase). This "
        "Python reimplementation will be removed once notebooks migrate.",
        DeprecationWarning,
        stacklevel=2,
    )
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

    def _build_item_uri(item_id: str) -> str:
        # Single source of truth: the Rust builder owns the default base and the
        # UUID check. (Raises ValueError on a non-UUID id, matching native.)
        from . import _rust as rust_ext
        return rust_ext.build_item_uri(item_id)

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
            "uri": getattr(concept, 'uri', None) or _build_item_uri(concept.id),
            "list_id": collection_id,
            "labels": labels,
        }

    def _resolve_marker(marker: dict, node_id: str, config: dict) -> dict:
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

    # Memoise distinct markers: the resolved reference is identical wherever the
    # same (collection, uuid/label) appears, so resolve each once. In ETL the
    # same controlled-list terms repeat across many resources.
    _marker_cache: dict = {}

    def resolve_marker(marker: dict, node_id: str, config: dict) -> dict:
        collection_id = config.get("controlledList") or config.get("rdmCollection")
        key = None
        if collection_id:
            if marker.get("__needs_rdm_lookup") and marker.get("uuid"):
                key = (collection_id, "id", marker["uuid"])
            elif marker.get("__needs_rdm_label_lookup") and marker.get("label"):
                key = (collection_id, "label", marker["label"])
        if key is not None and key in _marker_cache:
            return _marker_cache[key]
        result = _resolve_marker(marker, node_id, config)
        if key is not None:
            _marker_cache[key] = result
        return result

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

    node_id = params.get("subject") or params["node_id"]
    collection_id = params.get("object") or params["collection_id"]
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
    except Exception as e:
        raise RuntimeError(f"Failed to register CLM list datatype: {e}") from e


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
    except Exception as e:
        raise RuntimeError(f"Failed to register CLM widgets: {e}") from e


def set_clm_base_uri(uri: str) -> None:
    """
    Set the CLM base URI for generating reference item URIs.

    This should match `{PUBLIC_SERVER_ADDRESS}/plugins/controlled-list-manager/item/`.
    When set, new StaticReference objects will use this as the URI prefix
    instead of the `urn:uuid:` fallback.

    Args:
        uri: Base URI (trailing slash added if missing),
             e.g. "http://localhost:8000/plugins/controlled-list-manager/item/"
    """
    from . import _rust as rust_ext
    rust_ext.set_clm_base_uri(uri)


def get_clm_base_uri() -> "str | None":
    """Get the current CLM base URI, or None if not set."""
    from . import _rust as rust_ext
    return rust_ext.get_clm_base_uri()


def clear_clm_base_uri() -> None:
    """Clear the CLM base URI (reverts to urn:uuid: fallback)."""
    from . import _rust as rust_ext
    rust_ext.clear_clm_base_uri()


def _configure_clm_base_uri_from_settings() -> bool:
    """
    Auto-configure CLM base URI from Django's PUBLIC_SERVER_ADDRESS if available.

    Returns True if configured, False otherwise.
    """
    try:
        from django.conf import settings
        public_server = getattr(settings, 'PUBLIC_SERVER_ADDRESS', None)
        if public_server:
            base = public_server.rstrip('/')
            force_script = getattr(settings, 'FORCE_SCRIPT_NAME', None)
            if force_script:
                base = f"{base}/{force_script.strip('/')}"
            uri = f"{base}/plugins/controlled-list-manager/item/"
            set_clm_base_uri(uri)
            return True
    except Exception:
        pass
    return False


# Auto-register on import
_rust_available = _register_rust_handler()
_register_python_handler()
_mutation_available = _register_mutation_handler()
_list_datatype_registered = _register_list_datatype()
_widgets_registered = _register_widgets()
_clm_uri_configured = _configure_clm_base_uri_from_settings()


__all__ = [
    # Version
    "__version__",
    # CLM base URI configuration
    "set_clm_base_uri",
    "get_clm_base_uri",
    "clear_clm_base_uri",
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
