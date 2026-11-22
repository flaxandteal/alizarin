"""
Protocol definitions and WKRM for resource models.

Matches TypeScript interfaces.ts with complete interfaces.
Graph and resource storage is handled by the Rust registry.
"""

from __future__ import annotations

import json
import re
from typing import (
    Any,
    Awaitable,
    Callable,
    Dict,
    List,
    Optional,
    Protocol,
    TYPE_CHECKING,
    TypeVar,
    runtime_checkable,
)
from dataclasses import dataclass, field

if TYPE_CHECKING:
    from .static_types import (
        StaticGraphMeta,
        StaticNode,
        StaticNodegroup,
        StaticTile,
        StaticCollection,
    )
    from .pseudos import IPseudo


# =============================================================================
# Type Variables
# =============================================================================

RIVM = TypeVar("RIVM", bound="IRIVM")
T = TypeVar("T")


def _normalize_model_name(name: str) -> str:
    """A model name compared case, spaces and punctuation aside, as
    TypeScript's normalizeModelName."""
    return re.sub(r"[^a-z0-9]", "", name.lower())


# =============================================================================
# Protocol Definitions (matching TypeScript interfaces.ts)
# =============================================================================

@runtime_checkable
class IRIVM(Protocol):
    """
    Interface for Resource Instance ViewModel.

    Matches TypeScript IRIVM<T> interface.
    """
    id: str
    then: None
    __parentPseudo: Optional[IPseudo]

    @property
    def _i(self) -> Optional[IInstanceWrapper]:
        """Get instance wrapper."""
        ...

    @property
    def __(self) -> Optional[IModelWrapper]:
        """Get model wrapper."""
        ...


@runtime_checkable
class IWKRM(Protocol):
    """
    Well-Known Resource Model metadata.

    Matches TypeScript IWKRM interface.
    """
    model_name: str
    model_class_name: str
    graph_id: str
    meta: StaticGraphMeta


@runtime_checkable
class IModelWrapper(Protocol[RIVM]):
    """
    Interface for Model Wrapper.

    Matches TypeScript IModelWrapper<T> interface.
    """
    wkrm: IWKRM

    async def all(
        self,
        params: Optional[Dict[str, Any]] = None,
    ) -> List[RIVM]:
        """Get all resources of this model."""
        ...

    def get_permitted_nodegroups(self) -> Dict[Optional[str], bool]:
        """Get nodegroup permission map."""
        ...

    def is_nodegroup_permitted(
        self,
        nodegroup_id: str,
        tile: Optional[StaticTile],
    ) -> bool:
        """Check if nodegroup is permitted."""
        ...

    def get_child_nodes(self, node_id: str) -> Dict[str, StaticNode]:
        """Get child nodes of a node."""
        ...

    def get_node_objects_by_alias(self) -> Dict[str, StaticNode]:
        """Get all nodes by alias."""
        ...

    def get_node_objects(self) -> Dict[str, StaticNode]:
        """Get all nodes by ID."""
        ...

    def get_nodegroup_objects(self) -> Dict[str, StaticNodegroup]:
        """Get all nodegroups by ID."""
        ...

    def get_edges(self) -> Dict[str, List[str]]:
        """Get edges grouped by domain node."""
        ...

    def get_nodegroup_name(self, nodegroup_id: str) -> str:
        """Get human-readable nodegroup name."""
        ...

    def create_pseudo_value(
        self,
        alias: Optional[str],
        tile: Any,
        parent: Any,
    ) -> Any:
        """Create a pseudo value."""
        ...


@runtime_checkable
class IInstanceWrapper(Protocol[RIVM]):
    """
    Interface for Instance Wrapper.

    Matches TypeScript IInstanceWrapper<T> interface.
    """
    resource: Optional[Any]
    model: IModelWrapper[RIVM]

    async def load_nodes(self, aliases: List[str]) -> None:
        """Load specific node aliases."""
        ...

    def all_entries(self) -> Any:
        """Get all cached entries."""
        ...

    def add_pseudo(
        self,
        child_node: StaticNode,
        tile: Optional[StaticTile],
        node: StaticNode,
    ) -> IPseudo:
        """Add a pseudo value."""
        ...

    async def set_orm_attribute(self, key: str, value: Any) -> None:
        """Set an ORM attribute."""
        ...

    async def get_orm_attribute(self, key: str) -> Any:
        """Get an ORM attribute."""
        ...

    async def get_value_cache(
        self,
        build: bool,
        get_meta: Optional[Callable[[Any], Awaitable[Optional[Dict[str, Any]]]]],
    ) -> Optional[Dict[str, Dict[str, Any]]]:
        """Get the value cache."""
        ...

    async def get_root(self) -> Optional[IPseudo]:
        """Get root pseudo."""
        ...

    async def get_root_view_model(self) -> Any:
        """Get root view model."""
        ...

    async def populate(self, lazy: bool) -> None:
        """Populate with tile data."""
        ...

    async def retrieve_pseudo(
        self,
        key: str,
        dflt: Any = None,
        raise_error: bool = False,
    ) -> Optional[List[IPseudo]]:
        """Retrieve a pseudo value."""
        ...

    async def has_pseudo(self, key: str) -> bool:
        """Check if pseudo exists."""
        ...

    def set_pseudo(self, key: str, value: Any) -> None:
        """Set a pseudo value."""
        ...

    async def set_default_pseudo(self, key: str, value: Any) -> Any:
        """Set default pseudo value."""
        ...


@runtime_checkable
class IReferenceDataManager(Protocol):
    """
    Interface for Reference Data Manager.

    Matches TypeScript IReferenceDataManager interface.
    """

    async def retrieve_collection(self, id: str) -> StaticCollection:
        """Retrieve a collection by ID."""
        ...


# =============================================================================
# WKRM (Well-Known Resource Model)
# =============================================================================

@dataclass
class WKRM:
    """
    Well-Known Resource Model metadata wrapper.

    Matches TypeScript WKRM from graphManager.ts.
    """
    graphid: str
    name: str = ""
    slug: str = ""
    model_name: str = ""
    model_class_name: str = ""
    relatable_resource_model_ids: List[str] = field(default_factory=list)
    meta: Optional[StaticGraphMeta] = None

    @property
    def graph_id(self) -> str:
        """Alias for graphid."""
        return self.graphid

    @classmethod
    def from_meta(cls, meta: StaticGraphMeta) -> WKRM:
        """Create WKRM from StaticGraphMeta."""
        name = meta.name if isinstance(meta.name, str) else meta.name.get("en", "")
        return cls(
            graphid=meta.graphid,
            name=name,
            slug=meta.slug or "",
            model_name=name,
            model_class_name=name.replace(" ", ""),
            relatable_resource_model_ids=meta.relatable_resource_model_ids,
            meta=meta,
        )


# =============================================================================
# GraphManager - Thin coordinator (does NOT duplicate core functionality)
# =============================================================================

class GraphManager:
    """
    Thin coordinator for managing graphs and resources, as TypeScript's
    GraphManager with its StaticStore.

    Does NOT duplicate core functionality - just wires together:
    - core's graph registry (register_graph, load_prebuild)
    - core's resource registry (`registry`), which getResource and
      ResourceModelWrapper.all read and references resolve against
    - ResourceModelWrapper for model management (load_graph)

    Resources get in by register_resource, load_prebuild (a prebuild
    directory's graphs, collections and business data), or the registry's own
    loaders (`registry.load_from_business_data_bytes`).
    """

    def __init__(self):
        """Initialize GraphManager with minimal state."""
        from . import ResourceRegistry

        # Every resource this manager knows, held by core.
        self.registry = ResourceRegistry()
        # Track registered graph IDs
        self._graph_ids: List[str] = []
        # Model wrappers cache
        self._models: Dict[str, Any] = {}
        # Graph cache
        self._graphs: Dict[str, Any] = {}

    def register_graph(self, graph_dict: Dict[str, Any]) -> str:
        """
        Register a graph using Rust core.

        Args:
            graph_dict: Graph definition (dict or StaticGraph)

        Returns:
            graph_id: The registered graph ID
        """
        from . import register_graph as rust_register_graph
        from .static_types import StaticGraph

        # Convert to StaticGraph if needed
        if not isinstance(graph_dict, StaticGraph):
            graph = StaticGraph.from_dict(graph_dict)
        else:
            graph = graph_dict

        # Use Rust core to register
        graph_json = json.dumps(graph.to_dict())
        graph_id = rust_register_graph(graph_json)

        self._graph_ids.append(graph_id)
        self._graphs[graph_id] = graph
        return graph_id

    def register_resource(self, resource_dict: Dict[str, Any]) -> str:
        """
        Add one resource to the registry.

        Args:
            resource_dict: Resource data (dict or StaticResource)

        Returns:
            resource_id: The resource instance ID
        """
        from .static_types import StaticResource

        # Convert to StaticResource if needed
        if not isinstance(resource_dict, StaticResource):
            resource = StaticResource.from_dict(resource_dict)
        else:
            resource = resource_dict

        self.registry.merge_from_resources(json.dumps([resource.to_dict()]), True)
        return resource.resourceinstance.resourceinstanceid

    def load_prebuild(self, prebuild_dir: str, base_uri: str = "http://localhost/") -> Dict[str, Any]:
        """
        Load a prebuild directory (the Arches package layout: graphs/,
        reference_data/, business_data/; see docs/prebuild.mdx): register its
        graphs, add its collections to the global RDM cache, and load its
        business data into the registry.

        Args:
            prebuild_dir: The directory.
            base_uri: The RDM namespace, for ids and SKOS XML.

        Returns:
            {"graph_ids": [...], "collection_ids": [...], "resource_ids": [...]}
        """
        from . import import_prebuild

        imported = import_prebuild(prebuild_dir, base_uri)
        for graph_id in imported["graph_ids"]:
            if graph_id not in self._graph_ids:
                self._graph_ids.append(graph_id)
        refs = self.registry.load_prebuild_business_data(prebuild_dir)
        return {
            "graph_ids": list(imported["graph_ids"]),
            "collection_ids": list(imported["collection_ids"]),
            "resource_ids": [ref["resourceinstanceid"] for ref in refs],
        }

    def load_graph(self, model: str) -> Any:
        """
        The ResourceModelWrapper for a registered graph, by its model class
        name (as WKRM.model_class_name: "DigitalObject"; case, spaces and
        punctuation aside, so "Digital Object" too) or its graph id, as
        TypeScript's graphManager.loadGraph.

        Raises:
            KeyError: naming the models known, if none matches.
        """
        from .model_wrapper import ResourceModelWrapper

        known = {self._model_class_name(gid): gid for gid in self._graph_ids}
        graph_id: Optional[str]
        if model in self._graph_ids:
            graph_id = model
        else:
            target = _normalize_model_name(model)
            graph_id = next(
                (gid for name, gid in known.items() if _normalize_model_name(name) == target),
                None,
            )
        if graph_id is None:
            names = ", ".join(sorted(known)) or "none: register a graph or load a prebuild first"
            raise KeyError(
                f"No graph for model {model}. Give a model's class name "
                f"(case and spaces aside) or its graph id; known: {names}"
            )
        if graph_id not in self._models:
            self._models[graph_id] = ResourceModelWrapper.from_graph_id(graph_id)
        return self._models[graph_id]

    def _model_class_name(self, graph_id: str) -> str:
        """A registered graph's model class name, as WKRM gives it."""
        from . import get_graph_json
        from .static_types import StaticGraph, from_core

        graph = self._graphs.get(graph_id)
        if graph is None:
            graph = from_core(StaticGraph, json.loads(get_graph_json(graph_id)))
            self._graphs[graph_id] = graph
        return WKRM.from_meta(graph.meta).model_class_name

    async def getResource(
        self,
        resource_id: str,
        lazy: bool = True,
        pruneTiles: Optional[bool] = None
    ) -> IRIVM:
        """
        Get a resource as a ResourceInstanceViewModel.

        Uses existing ResourceModelWrapper to create the view model.

        Args:
            resource_id: Resource instance ID
            lazy: Whether to use lazy loading
            pruneTiles: Whether to prune tiles

        Returns:
            ResourceInstanceViewModel

        Raises:
            ValueError: If resource or graph not found
        """
        from .model_wrapper import ResourceModelWrapper

        from .static_types import StaticResource, from_core

        found = self.registry.get_full(resource_id)
        if found is None:
            raise ValueError(
                f"Resource not found: {resource_id}. Register it (register_resource) "
                f"or load its business data (load_prebuild, registry.load_from_business_data_bytes)."
            )
        resource = from_core(StaticResource, found)

        # Get graph ID
        graph_id = resource.resourceinstance.graph_id

        # Load or get cached model
        if graph_id not in self._models:
            self._models[graph_id] = ResourceModelWrapper.from_graph_id(graph_id)

        return await self._wrap_resource(resource, self._models[graph_id], lazy)

    async def all_for_model(
        self,
        model: Any,
        limit: Optional[int] = None,
        lazy: bool = False,
    ) -> List[IRIVM]:
        """
        Every registered resource of model's graph, in registration order, as
        ResourceInstanceViewModels of model (the store behind
        ResourceModelWrapper.all, as staticStore.loadAll is in TypeScript).
        """
        from .static_types import StaticResource, from_core

        resources = [
            from_core(StaticResource, found)
            for found in self.registry.get_all_full_for_graph(model.graph.graphid)
        ]
        if limit:
            resources = resources[:limit]
        return [await self._wrap_resource(resource, model, lazy) for resource in resources]

    async def _wrap_resource(self, resource: Any, model: Any, lazy: bool) -> IRIVM:
        """resource as a ResourceInstanceViewModel of model, populated."""
        from .view_models import ResourceInstanceViewModel

        resource_id = resource.resourceinstance.resourceinstanceid
        graph_id = resource.resourceinstance.graph_id

        # Create ResourceInstanceViewModel
        rivm = ResourceInstanceViewModel(
            id=resource_id,
            modelWrapper=model,
            instanceWrapperFactory=lambda rivm_inst: None,  # Will be set below
            cacheEntry=None
        )

        # Create instance wrapper
        from .instance_wrapper import ResourceInstanceWrapper
        instance_wrapper = ResourceInstanceWrapper(
            wkri=rivm,
            model=model,
        )

        # Set resource data directly
        instance_wrapper.resource = resource
        instance_wrapper.resource_id = resource_id
        instance_wrapper.graph_id = graph_id

        rivm._i = instance_wrapper

        # Populate the instance with tiles
        tiles = resource.tiles if resource.tiles else []
        await instance_wrapper.populate(tiles=tiles, lazy=lazy)

        return rivm


# =============================================================================
# Exports
# =============================================================================

__all__ = [
    # Protocols
    "IRIVM",
    "IWKRM",
    "IModelWrapper",
    "IInstanceWrapper",
    "IReferenceDataManager",
    # Classes
    "WKRM",
    "GraphManager",
]
