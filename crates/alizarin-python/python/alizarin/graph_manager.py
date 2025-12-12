"""
GraphManager and StaticStore for managing resource models.

Matches TypeScript graphManager.ts with complete interfaces.
"""

from __future__ import annotations

from typing import (
    Any,
    Awaitable,
    Callable,
    Dict,
    Generic,
    List,
    Optional,
    Protocol,
    Set,
    TYPE_CHECKING,
    TypeVar,
    Union,
    runtime_checkable,
)
from dataclasses import dataclass, field

if TYPE_CHECKING:
    from .static_types import (
        StaticGraph,
        StaticGraphMeta,
        StaticNode,
        StaticNodegroup,
        StaticTile,
        StaticCollection,
    )
    from .model_wrapper import ResourceModelWrapper
    from .instance_wrapper import ResourceInstanceWrapper
    from .pseudos import IPseudo


# =============================================================================
# Type Variables
# =============================================================================

RIVM = TypeVar("RIVM", bound="IRIVM")
T = TypeVar("T")


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


@runtime_checkable
class IGraphManager(Protocol[RIVM]):
    """
    Interface for Graph Manager.

    Matches TypeScript IGraphManager interface.
    """

    async def get_resource(
        self,
        resource_id: str,
        lazy: bool,
    ) -> RIVM:
        """Get a resource by ID."""
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
# GraphManager
# =============================================================================

class GraphManager(Generic[RIVM]):
    """
    Manages resource models and instances.

    Matches TypeScript GraphManager singleton.

    Attributes:
        models: Dictionary of model name to ResourceModelWrapper
        arches_client: Optional client for API communication
    """

    def __init__(self) -> None:
        self.models: Dict[str, ResourceModelWrapper] = {}
        self.arches_client: Optional[Any] = None
        self._initialized: bool = False
        self._resource_cache: Dict[str, RIVM] = {}

    async def initialize(self) -> None:
        """Initialize the graph manager."""
        if self._initialized:
            return

        # Load available models from client if available
        if self.arches_client:
            # TBD: implement client communication
            pass

        self._initialized = True

    async def get(
        self,
        model_class_or_name: Union[str, type],
    ) -> Optional[ResourceModelWrapper]:
        """
        Get a resource model wrapper by class or name.

        Args:
            model_class_or_name: Model class or name string

        Returns:
            ResourceModelWrapper for the model, or None if not found
        """
        # Extract model name
        if isinstance(model_class_or_name, str):
            model_name = model_class_or_name
        else:
            model_name = model_class_or_name.__name__

        if model_name not in self.models:
            await self._load_model(model_name)

        return self.models.get(model_name)

    async def get_resource(
        self,
        resource_id: str,
        lazy: bool = True,
    ) -> Optional[RIVM]:
        """
        Get a resource by ID.

        Args:
            resource_id: The resource instance ID
            lazy: Whether to use lazy loading

        Returns:
            Resource instance or None
        """
        # Check cache first
        if resource_id in self._resource_cache:
            return self._resource_cache[resource_id]

        if not self.arches_client:
            raise ValueError("No Arches client configured")

        # TBD: implement resource loading from client
        return None

    async def _load_model(self, model_name: str) -> None:
        """
        Load a model from the Arches client.

        Args:
            model_name: Name of the model to load
        """
        if not self.arches_client:
            raise ValueError("No Arches client configured")

        # TBD: implement model loading from client
        pass

    def register_model(
        self,
        model_name: str,
        wrapper: ResourceModelWrapper,
    ) -> None:
        """
        Register a model wrapper.

        Args:
            model_name: Name to register under
            wrapper: The ResourceModelWrapper instance
        """
        self.models[model_name] = wrapper

    def clear_cache(self) -> None:
        """Clear all caches."""
        self._resource_cache.clear()


# =============================================================================
# StaticStore
# =============================================================================

class StaticStore:
    """
    Stores static resources and metadata.

    Matches TypeScript StaticStore singleton.
    Used for caching graph models and resources without API calls.

    Attributes:
        resources: Dictionary of resource ID to resource data
        metadata: Dictionary of resource ID to metadata
        graphs: Dictionary of graph ID to StaticGraph
    """

    def __init__(self) -> None:
        self.resources: Dict[str, Any] = {}
        self.metadata: Dict[str, Any] = {}
        self.graphs: Dict[str, StaticGraph] = {}
        self.arches_client: Optional[Any] = None

    async def get_meta(
        self,
        resource_id: str,
    ) -> Optional[Dict[str, Any]]:
        """
        Get metadata for a resource.

        Args:
            resource_id: The resource instance ID

        Returns:
            Metadata dictionary or None
        """
        return self.metadata.get(resource_id)

    async def store_resource(
        self,
        resource_id: str,
        resource_data: Any,
    ) -> None:
        """
        Store a resource.

        Args:
            resource_id: The resource instance ID
            resource_data: The resource data to store
        """
        self.resources[resource_id] = resource_data

    async def get_resource(
        self,
        resource_id: str,
    ) -> Optional[Any]:
        """
        Get a stored resource.

        Args:
            resource_id: The resource instance ID

        Returns:
            Resource data or None
        """
        return self.resources.get(resource_id)

    def store_graph(
        self,
        graph_id: str,
        graph: StaticGraph,
    ) -> None:
        """
        Store a graph.

        Args:
            graph_id: The graph ID
            graph: The StaticGraph to store
        """
        self.graphs[graph_id] = graph

    def get_graph(
        self,
        graph_id: str,
    ) -> Optional[StaticGraph]:
        """
        Get a stored graph.

        Args:
            graph_id: The graph ID

        Returns:
            StaticGraph or None
        """
        return self.graphs.get(graph_id)

    def clear(self) -> None:
        """Clear all stored data."""
        self.resources.clear()
        self.metadata.clear()
        self.graphs.clear()


# =============================================================================
# Singleton Instances
# =============================================================================

graph_manager: GraphManager[Any] = GraphManager()
static_store: StaticStore = StaticStore()


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
    "IGraphManager",
    # Classes
    "WKRM",
    "GraphManager",
    "StaticStore",
    # Singletons
    "graph_manager",
    "static_store",
]
