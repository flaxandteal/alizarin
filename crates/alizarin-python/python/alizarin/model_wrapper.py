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
        use_rust_cache: Whether to use Rust-side caching
        nodes: Node cache (ID -> StaticNode)
        edges: Edge cache (domain_node_id -> list of edges)
        nodegroups: Nodegroup cache (ID -> StaticNodegroup)
    """

    __slots__ = (
        "wkrm",
        "graph",
        "use_rust_cache",
        "nodes",
        "edges",
        "nodegroups",
        "_permitted_nodegroups",
        "_nodes_by_alias",
    )

    def __init__(
        self,
        wkrm: IWKRM,
        graph: StaticGraph,
        use_rust_cache: bool = False,
    ) -> None:
        """
        Initialize ResourceModelWrapper.

        Args:
            wkrm: Well-Known Resource Model metadata
            graph: The StaticGraph for this model
            use_rust_cache: Whether to use Rust-side caching
        """
        self.wkrm: IWKRM = wkrm
        self.graph: StaticGraph = graph
        self.use_rust_cache: bool = use_rust_cache

        # Caches
        self.nodes: Optional[Dict[str, StaticNode]] = None
        self.edges: Optional[Dict[str, List[StaticEdge]]] = None
        self.nodegroups: Optional[Dict[str, StaticNodegroup]] = None
        self._nodes_by_alias: Optional[Dict[str, StaticNode]] = None

        # Permissions
        self._permitted_nodegroups: Dict[str, PermissionValue] = {}

    @classmethod
    def from_graph_id(
        cls,
        graph_id: str,
        use_rust_cache: bool = True,
    ) -> "ResourceModelWrapper[RIVM]":
        """
        Create a ResourceModelWrapper from a registered graph ID.

        Convenience method that handles all the boilerplate of loading a graph
        from the Rust registry and setting up the model wrapper.

        Args:
            graph_id: The graph ID returned by register_graph()
            use_rust_cache: Whether to use Rust-side caching (default True)

        Returns:
            A fully initialized ResourceModelWrapper with nodes built

        Raises:
            RuntimeError: If the Rust extension is not loaded
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

        if get_graph_json is None:
            raise RuntimeError("Alizarin Rust extension not loaded")

        graph_json_str = get_graph_json(graph_id)
        if graph_json_str is None:
            raise ValueError(f"Graph not found for ID: {graph_id}")

        graph_data = json.loads(graph_json_str)
        graph = StaticGraph.from_dict(graph_data)
        wkrm = WKRM.from_meta(graph.meta)

        model = cls(wkrm, graph, use_rust_cache=use_rust_cache)
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

    def is_nodegroup_permitted(
        self,
        nodegroup_id: Optional[str],
        tile: Optional[StaticTile],
    ) -> bool:
        """
        Check if a nodegroup/tile is permitted.

        For boolean permissions, returns the boolean value.
        For conditional permissions, evaluates the tile data path and checks
        if the value is in the allowed set.

        Args:
            nodegroup_id: The nodegroup ID to check
            tile: The tile to evaluate (required for conditional permissions)

        Returns:
            True if the nodegroup/tile is permitted
        """
        ng_id = nodegroup_id or ""

        # Check direct permission
        if ng_id not in self._permitted_nodegroups:
            # Default to permitted
            return True

        permission = self._permitted_nodegroups[ng_id]

        # Simple boolean permission
        if isinstance(permission, bool):
            return permission

        # Conditional permission - need tile data
        if tile is None:
            # For nodegroup-level checks (no tile), conditional permissions allow
            # the nodegroup itself, individual tiles will be filtered
            return True

        # Evaluate the path against the tile
        path = permission.get("path", "")
        allowed = set(permission.get("allowed", []))

        value = self._evaluate_tile_path(tile, path)
        permitted = value is not None and value in allowed

        # Debug logging for conditional filtering
        tile_id = getattr(tile, "tileid", None) or "(no id)"
        logger.debug(
            "[alizarin] Conditional filter: nodegroup=%s, tile=%s, path=%s, value=%r, allowed=%r, permitted=%s",
            ng_id, tile_id, path, value, allowed, permitted
        )

        if value is None:
            # Path doesn't resolve - deny by default
            return False

        return permitted

    @staticmethod
    def _evaluate_tile_path(tile: StaticTile, path: str) -> Optional[str]:
        """
        Evaluate a JSON path against a tile's data.

        Path format: ".data.uuid.field.subfield" or "data.uuid.field.subfield"
        Returns the string value at that path, or None if not found/not a string.
        """
        path = path.lstrip(".")
        segments = path.split(".")

        if not segments:
            return None

        # Start navigation - first segment should be "data" for tile data
        tile_data = getattr(tile, "data", None) or {}

        if segments[0] == "data":
            if len(segments) < 2:
                return None
            # Get the node's data by the next segment (node_id/uuid)
            current = tile_data.get(segments[1])
            start_idx = 2
        else:
            # Direct path into data - treat first segment as node_id
            current = tile_data.get(segments[0])
            start_idx = 1

        if current is None:
            return None

        # Navigate remaining segments
        for segment in segments[start_idx:]:
            if isinstance(current, dict):
                current = current.get(segment)
            else:
                return None
            if current is None:
                return None

        # Extract string value
        if isinstance(current, str):
            return current

        # Try common nested patterns (e.g., {"en": "value"})
        if isinstance(current, dict):
            for key in ("en", "value", "label", "name"):
                if key in current and isinstance(current[key], str):
                    return current[key]

        return None

    def prune_graph(self, user: Optional[Any] = None) -> None:
        """
        Remove unpermitted nodes, edges, nodegroups, and cards from the graph.

        Matches TypeScript ResourceModelWrapper.pruneGraph

        Args:
            user: Optional user for permission checking (unused)
        """
        nodes = self.get_node_objects()

        # Determine which nodes to keep
        nodes_to_keep: Set[str] = set()

        # Always keep root
        nodes_to_keep.add(self.graph.root.nodeid)

        # Keep permitted nodes and their ancestors
        for node in nodes.values():
            ng_id = node.nodegroup_id or ""

            if self.is_nodegroup_permitted(ng_id, None):
                # Keep this node
                nodes_to_keep.add(node.nodeid)

                # Keep all ancestors
                self._add_ancestors(node.nodeid, nodes_to_keep)

        # Filter graph components
        self.graph.nodes = [
            n for n in self.graph.nodes
            if n.nodeid in nodes_to_keep
        ]
        self.graph.edges = [
            e for e in self.graph.edges
            if e.domainnode_id in nodes_to_keep and e.rangenode_id in nodes_to_keep
        ]

        # Filter nodegroups - keep if any node in it is kept
        kept_nodegroup_ids = {
            n.nodegroup_id
            for n in self.graph.nodes
            if n.nodegroup_id
        }
        self.graph.nodegroups = [
            ng for ng in self.graph.nodegroups
            if ng.nodegroupid in kept_nodegroup_ids
        ]

        # Filter cards
        if self.graph.cards:
            self.graph.cards = [
                c for c in self.graph.cards
                if c.nodegroup_id in kept_nodegroup_ids
            ]

        # Rebuild caches
        self.nodes = None
        self.edges = None
        self.nodegroups = None
        self._nodes_by_alias = None
        self.build_nodes()

    def _add_ancestors(
        self,
        node_id: str,
        nodes_to_keep: Set[str],
    ) -> None:
        """
        Recursively add all ancestor nodes.

        Args:
            node_id: Starting node ID
            nodes_to_keep: Set to add ancestors to
        """
        edges = self.get_edges()

        # Find edges where this node is the range (child)
        for domain_id, edge_list in edges.items():
            for edge in edge_list:
                if edge.rangenode_id == node_id and domain_id not in nodes_to_keep:
                    nodes_to_keep.add(domain_id)
                    self._add_ancestors(domain_id, nodes_to_keep)

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
