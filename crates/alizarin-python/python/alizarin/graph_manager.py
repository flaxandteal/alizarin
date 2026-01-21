"""
Protocol definitions and WKRM for resource models.

Matches TypeScript interfaces.ts with complete interfaces.
Graph and resource storage is handled by the Rust registry.
"""

from __future__ import annotations

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
]
