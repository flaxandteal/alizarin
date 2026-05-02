"""
ResourceModelWrapper for graph model management.

Matches TypeScript ResourceModelWrapper in graphManager.ts
"""

from __future__ import annotations

import json
import logging
from typing import (
    Any,
    Dict,
    Generic,
    List,
    Optional,
    Set,
    TYPE_CHECKING,
    TypedDict,
    TypeVar,
    Union,
)

logger = logging.getLogger(__name__)

if TYPE_CHECKING:
    from .static_types import StaticGraph, StaticNode, StaticEdge, StaticNodegroup, StaticTile
    from .graph_manager import IRIVM, IWKRM

# Type Variables
RIVM = TypeVar("RIVM", bound="IRIVM")


class ConditionalPermission(TypedDict):
    """Conditional permission rule for filtering tiles by data values."""
    path: str  # JSON path to evaluate (e.g., ".data.uuid.field.name")
    allowed: List[str]  # Tile is permitted if value at path is in this list


# Permission value: bool for simple allow/deny, or conditional dict
PermissionValue = Union[bool, ConditionalPermission]


class ResourceModelWrapper(Generic[RIVM]):
    """
    Wrapper for a resource model graph with permissions and pruning.

    Matches TypeScript ResourceModelWrapper.

    Attributes:
        wkrm: Well-Known Resource Model metadata
        graph: The StaticGraph
        nodes: Node cache (ID -> StaticNode)
        edges: Edge cache (domain_node_id -> list of edges)
        nodegroups: Nodegroup cache (ID -> StaticNodegroup)
    """

    __slots__ = (
        "wkrm",
        "graph",
        "nodes",
        "edges",
        "nodegroups",
        "_permitted_nodegroups",
        "_nodes_by_alias",
        "_rust_model",
    )

    def __init__(
        self,
        wkrm: IWKRM,
        graph: StaticGraph,
    ) -> None:
        """
        Initialize ResourceModelWrapper.

        Args:
            wkrm: Well-Known Resource Model metadata
            graph: The StaticGraph for this model
        """
        self.wkrm: IWKRM = wkrm
        self.graph: StaticGraph = graph

        # Caches
        self.nodes: Optional[Dict[str, StaticNode]] = None
        self.edges: Optional[Dict[str, List[StaticEdge]]] = None
        self.nodegroups: Optional[Dict[str, StaticNodegroup]] = None
        self._nodes_by_alias: Optional[Dict[str, StaticNode]] = None

        # Permissions
        self._permitted_nodegroups: Dict[str, PermissionValue] = {}

        # Rust-side model wrapper for delegation
        self._rust_model: Any = None
        self._init_rust_model()

    def _init_rust_model(self) -> None:
        """Initialize the Rust-side PyResourceModelWrapper for delegation.

        Ensures the graph is registered in the Rust registry, then creates
        the Rust-side wrapper.
        """
        from . import register_graph, PyResourceModelWrapper
        # Ensure graph is registered (idempotent if already registered)
        register_graph(json.dumps(self.graph.to_dict()))
        self._rust_model = PyResourceModelWrapper(self.graph.graphid)

    @classmethod
    def from_graph_id(
        cls,
        graph_id: str,
    ) -> "ResourceModelWrapper[RIVM]":
        """
        Create a ResourceModelWrapper from a registered graph ID.

        Convenience method that handles all the boilerplate of loading a graph
        from the Rust registry and setting up the model wrapper.

        Args:
            graph_id: The graph ID returned by register_graph()

        Returns:
            A fully initialized ResourceModelWrapper with nodes built

        Raises:
            ValueError: If the graph ID is not found

        Example:
            >>> import alizarin
            >>> graph_id = alizarin.register_graph(graph_json)
            >>> model = ResourceModelWrapper.from_graph_id(graph_id)
            >>> # model is ready to use - nodes are already built
        """
        from . import get_graph_json
        from .static_types import StaticGraph
        from .graph_manager import WKRM

        graph_json_str = get_graph_json(graph_id)
        if graph_json_str is None:
            raise ValueError(f"Graph not found for ID: {graph_id}")

        graph_data = json.loads(graph_json_str)
        graph = StaticGraph.from_dict(graph_data)
        wkrm = WKRM.from_meta(graph.meta)

        model = cls(wkrm, graph)
        model.build_nodes()
        return model

    def build_nodes(self) -> None:
        """Build node, edge, and nodegroup caches."""
        self.nodes = {node.nodeid: node for node in self.graph.nodes}
        self._nodes_by_alias = {
            node.alias: node
            for node in self.graph.nodes
            if node.alias
        }

        # Build edges map (domain_node_id -> list of edges)
        self.edges = {}
        for edge in self.graph.edges:
            if edge.domainnode_id not in self.edges:
                self.edges[edge.domainnode_id] = []
            self.edges[edge.domainnode_id].append(edge)

        # Build nodegroups map
        self.nodegroups = {ng.nodegroupid: ng for ng in self.graph.nodegroups}

    def get_node_objects(self) -> Dict[str, StaticNode]:
        """
        Get all nodes as a dictionary by ID.

        Returns:
            Dictionary mapping node ID to StaticNode
        """
        if self.nodes is None:
            self.build_nodes()
        return self.nodes or {}

    def get_node_objects_by_alias(self) -> Dict[str, StaticNode]:
        """
        Get all nodes as a dictionary by alias.

        Returns:
            Dictionary mapping alias to StaticNode
        """
        if self._nodes_by_alias is None:
            self.build_nodes()
        return self._nodes_by_alias or {}

    def get_nodegroup_objects(self) -> Dict[str, StaticNodegroup]:
        """
        Get all nodegroups as a dictionary.

        Returns:
            Dictionary mapping nodegroup ID to StaticNodegroup
        """
        if self.nodegroups is None:
            self.build_nodes()
        return self.nodegroups or {}

    def get_nodegroup_ids(self) -> List[str]:
        """
        Get all nodegroup IDs.

        Returns:
            List of nodegroup IDs
        """
        return list(self.get_nodegroup_objects().keys())

    def get_edges(self) -> Dict[str, List[StaticEdge]]:
        """
        Get edges grouped by domain node.

        Returns:
            Dictionary mapping domain node ID to list of edges
        """
        if self.edges is None:
            self.build_nodes()
        return self.edges or {}

    def get_child_nodes(self, parent_node_id: str) -> Dict[str, StaticNode]:
        """
        Get all child nodes of a parent node.

        Args:
            parent_node_id: The parent node ID

        Returns:
            Dictionary mapping alias to StaticNode for children
        """
        edges = self.get_edges()
        nodes = self.get_node_objects()

        result: Dict[str, StaticNode] = {}
        if parent_node_id in edges:
            for edge in edges[parent_node_id]:
                child_node = nodes.get(edge.rangenode_id)
                if child_node and child_node.alias:
                    result[child_node.alias] = child_node

        return result

    def get_child_node_aliases(self, parent_node_id: str) -> List[str]:
        """
        Get aliases of all child nodes.

        Args:
            parent_node_id: The parent node ID

        Returns:
            List of child node aliases
        """
        child_nodes = self.get_child_nodes(parent_node_id)
        return list(child_nodes.keys())

    def get_node_object_from_alias(self, alias: str) -> Optional[StaticNode]:
        """
        Get a node by its alias.

        Args:
            alias: The node alias

        Returns:
            StaticNode or None if not found
        """
        nodes_by_alias = self.get_node_objects_by_alias()
        return nodes_by_alias.get(alias)

    def get_root_node(self) -> StaticNode:
        """
        Get the root node of the graph.

        Returns:
            The root StaticNode
        """
        return self.graph.root

    def get_nodegroup_name(self, nodegroup_id: str) -> str:
        """
        Get human-readable nodegroup name.

        Args:
            nodegroup_id: The nodegroup ID

        Returns:
            Nodegroup name (from first node in group)
        """
        nodes = self.get_node_objects()
        for node in nodes.values():
            if node.nodegroup_id == nodegroup_id:
                return node.name or node.alias or nodegroup_id
        return nodegroup_id

    def create_pseudo_value(
        self,
        alias: Optional[str],
        tile: Any,
        parent: Any,
    ) -> Any:
        """
        Create a pseudo value for a node.

        Args:
            alias: Node alias
            tile: Tile data
            parent: Parent pseudo

        Returns:
            New PseudoValue instance
        """
        from .pseudos import create_pseudo_value

        node = None
        if alias:
            node = self.get_node_object_from_alias(alias)

        if node is None:
            raise ValueError(f"Node not found for alias: {alias}")

        return create_pseudo_value(
            node=node,
            tile=tile,
            value=None,
            parent_pseudo=parent,
            model=self,
        )

    def get_collections(self, accessible_only: bool = False) -> List[str]:
        """
        Get all unique collection IDs from concept nodes.

        Args:
            accessible_only: Whether to filter by permissions

        Returns:
            List of collection IDs
        """
        nodes = self.get_node_objects()
        collections: Set[str] = set()

        for node in nodes.values():
            if node.datatype == "concept":
                # Check permissions if requested
                if accessible_only:
                    if not self.is_nodegroup_permitted(node.nodegroup_id, None):
                        continue

                # Get collection ID from config
                if node.config and "rdmCollection" in node.config:
                    collections.add(node.config["rdmCollection"])

        return list(collections)

    def get_branch_publication_ids(self) -> List[str]:
        """
        Get all unique branch publication IDs.

        Returns:
            List of branch publication IDs
        """
        nodes = self.get_node_objects()
        pub_ids: Set[str] = set()

        for node in nodes.values():
            if node.sourcebranchpublication_id:
                pub_ids.add(node.sourcebranchpublication_id)

        return list(pub_ids)

    def get_permitted_nodegroups(self) -> Dict[str, PermissionValue]:
        """
        Get the current permission map.

        Returns:
            Dictionary mapping nodegroup ID to permission rule (bool or conditional)
        """
        if not self._permitted_nodegroups:
            # Default: all permitted
            nodes = self.get_node_objects()
            self._permitted_nodegroups = {"": True}  # Root always permitted

            for node in nodes.values():
                ng_id = node.nodegroup_id or ""
                if ng_id not in self._permitted_nodegroups:
                    self._permitted_nodegroups[ng_id] = True

        return self._permitted_nodegroups

    def set_permitted_nodegroups(
        self,
        permissions: Dict[str, PermissionValue],
    ) -> None:
        """
        Set nodegroup permissions with support for both boolean and conditional rules.

        Keys can be nodegroup IDs or node aliases (for semantic nodes).

        Values can be:
        - bool: True/False for simple allow/deny
        - dict: {"path": ".data.uuid.field", "allowed": ["value1", "value2"]}
          for conditional filtering based on tile data

        Args:
            permissions: Dictionary mapping nodegroup ID to permission rule

        Raises:
            ValueError: If any permission rule is invalid
        """
        errors: List[str] = []

        for key, value in permissions.items():
            if not isinstance(key, str):
                errors.append(f"Invalid key (not a string): {key!r}")
                continue

            if isinstance(value, bool):
                continue  # Valid boolean permission

            if isinstance(value, dict):
                # Validate conditional permission
                if "path" not in value:
                    errors.append(f"Invalid conditional rule for '{key}': 'path' key is required")
                elif not isinstance(value.get("path"), str):
                    errors.append(f"Invalid conditional rule for '{key}': 'path' must be a string")
                elif not value.get("path"):
                    errors.append(f"Invalid conditional rule for '{key}': 'path' cannot be empty")

                if "allowed" not in value:
                    errors.append(f"Invalid conditional rule for '{key}': 'allowed' key is required")
                elif not isinstance(value.get("allowed"), list):
                    errors.append(f"Invalid conditional rule for '{key}': 'allowed' must be a list")
                elif not value.get("allowed"):
                    errors.append(f"Invalid conditional rule for '{key}': 'allowed' list cannot be empty")
                else:
                    for i, item in enumerate(value.get("allowed", [])):
                        if not isinstance(item, str):
                            errors.append(f"Invalid conditional rule for '{key}': 'allowed[{i}]' must be a string")
                continue

            errors.append(
                f"Invalid permission value for '{key}': expected bool or {{path, allowed}} dict"
            )

        if errors:
            raise ValueError(
                "Permission validation errors:\n  - " + "\n  - ".join(errors)
            )

        self._permitted_nodegroups = permissions.copy()
        self._rust_model.set_permitted_nodegroups(permissions)

    def is_nodegroup_permitted(
        self,
        nodegroup_id: Optional[str],
        tile: Optional[StaticTile] = None,
    ) -> bool:
        """
        Check if a nodegroup/tile is permitted.

        Delegates to the Rust-side GraphModelAccess for evaluation.

        Args:
            nodegroup_id: The nodegroup ID to check
            tile: The tile to evaluate (required for conditional permissions)

        Returns:
            True if the nodegroup/tile is permitted
        """
        ng_id = nodegroup_id or ""

        if tile is not None:
            tile_json = json.dumps(tile.to_dict() if hasattr(tile, "to_dict") else tile)
            return self._rust_model.is_tile_permitted(ng_id, tile_json)

        return self._rust_model.is_nodegroup_permitted(ng_id)

    def prune_graph(self, user: Optional[Any] = None) -> None:
        """
        Remove unpermitted nodes, edges, nodegroups, and cards from the graph.

        Delegates to Rust-side GraphModelAccess.prune_graph().

        Args:
            user: Optional user for permission checking (unused)
        """
        self._rust_model.prune_graph(None)

        # Refresh Python-side graph from Rust export and rebuild caches
        graph_data = self._rust_model.export_graph()
        from .static_types import StaticGraph
        self.graph = StaticGraph.from_dict(graph_data)

        self.nodes = None
        self.edges = None
        self.nodegroups = None
        self._nodes_by_alias = None
        self.build_nodes()

    def export_graph(self) -> StaticGraph:
        """
        Export the current graph (after pruning, etc.).

        Returns:
            The current StaticGraph
        """
        return self.graph

    def get_root(self) -> Any:
        """
        Get root NodeViewModel for navigation.

        Returns:
            SemanticViewModel for the root node
        """
        from .semantic import SemanticViewModel
        return SemanticViewModel(None, None, self.graph.root)

    async def all(
        self,
        params: Optional[Dict[str, Any]] = None,
    ) -> List[RIVM]:
        """
        Get all resources of this model.

        Args:
            params: Optional parameters (limit, lazy, etc.)

        Returns:
            List of resource instances
        """
        # TBD: implement resource loading from client
        return []


__all__ = [
    "ResourceModelWrapper",
]
