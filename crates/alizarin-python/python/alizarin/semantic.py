"""
SemanticViewModel matching TypeScript semantic.ts

SemanticViewModel represents a semantic (nested) node in the resource tree.
It provides lazy-loading access to child values.
"""

from __future__ import annotations

from typing import (
    Any,
    Dict,
    List,
    Optional,
    TYPE_CHECKING,
)

if TYPE_CHECKING:
    from .static_types import StaticNode, StaticTile
    from .pseudos import PseudoValue, IPseudo


# =============================================================================
# SemanticViewModel
# =============================================================================

class SemanticViewModel:
    """
    ViewModel for semantic (nested) nodes.

    Matches TypeScript SemanticViewModel class.
    Provides lazy-loading access to child values via attribute access.

    Attributes:
        __parent_wkri: Parent resource instance
        __tile: The tile this semantic node belongs to (may be None for root)
        __node: The StaticNode definition
        __child_values: Cache of loaded child values
        __parent_pseudo: Parent PseudoValue for context
    """

    __slots__ = (
        "__parent_wkri",
        "__tile",
        "__node",
        "__child_values",
        "__parent_pseudo",
        "_",
        "then",
    )

    def __init__(
        self,
        parent_wkri: Optional[Any],
        tile: Optional[StaticTile],
        node: StaticNode,
    ) -> None:
        """
        Initialize SemanticViewModel.

        Args:
            parent_wkri: Parent resource instance (WKRI)
            tile: The tile containing this semantic node
            node: The StaticNode definition
        """
        self.__parent_wkri: Optional[Any] = parent_wkri
        self.__tile: Optional[StaticTile] = tile
        self.__node: StaticNode = node
        self.__child_values: Dict[str, Any] = {}
        self.__parent_pseudo: Optional[PseudoValue[Any]] = None
        self._: Optional[Any] = None  # IViewModel compatibility
        self.then: None = None  # Not thenable

    def __repr__(self) -> str:
        alias = self.__node.alias if self.__node else "?"
        tile_id = self.__tile.tileid if self.__tile else "no-tile"
        return f"<SemanticViewModel {alias} tile={tile_id}>"

    async def __str__(self) -> str:
        """String representation with child values."""
        entries = [f"{k}: {v}" for k, v in self.__child_values.items()]
        return f"[[{', '.join(entries)}]]"

    def __for_json_cache(self) -> None:
        """Semantic nodes don't have cache entries."""
        return None

    def describe_field(self) -> Optional[str]:
        """Get field description from parent pseudo."""
        if self.__parent_pseudo:
            return self.__parent_pseudo.describe_field()
        return None

    def describe_field_group(self) -> Optional[str]:
        """Get field group description from parent pseudo."""
        if self.__parent_pseudo:
            return self.__parent_pseudo.describe_field_group()
        return None

    async def to_object(self) -> Dict[str, Any]:
        """
        Convert semantic node to a dictionary with all child values.

        Matches TypeScript SemanticViewModel.toObject().
        Loads all children and returns them as a dictionary.

        Returns:
            Dictionary mapping alias to value for all children.
        """
        if not self.__parent_wkri or not hasattr(self.__parent_wkri, "_i"):
            return {}

        instance_wrapper = self.__parent_wkri._i
        if not instance_wrapper:
            return {}

        model = instance_wrapper.model
        if not model:
            return {}

        # Get all child node aliases
        child_aliases = model.get_child_node_aliases(self.__node.nodeid)

        result: Dict[str, Any] = {}
        for alias in child_aliases:
            try:
                child_pseudo = await self.__get_child_value(alias)
                if child_pseudo:
                    value = await child_pseudo.get_value()
                    result[alias] = value
            except Exception as e:
                # Log but continue with other children
                print(f"Warning: Failed to load child {alias}: {e}")
                continue

        return result

    async def __get(self, key: str) -> Any:
        """
        Get child value by key.

        Args:
            key: The child node alias

        Returns:
            The child ViewModel value
        """
        # A value assigned since the last commit()
        instance_wrapper = getattr(self.__parent_wkri, "_i", None)
        if instance_wrapper is not None:
            tile_id = self.__tile.tileid if self.__tile else None
            pending = instance_wrapper.pending_value(tile_id, key)
            if pending is not None:
                return pending[0]
        child_value = await self.__get_child_value(key)
        if not child_value:
            return None
        return await child_value.get_value()

    def __has(self, key: str) -> bool:
        """
        Check if a child key exists.

        Args:
            key: The child node alias

        Returns:
            True if the key exists as a child
        """
        if not self.__parent_wkri or not hasattr(self.__parent_wkri, "_i"):
            return False

        instance_wrapper = self.__parent_wkri._i
        if not instance_wrapper:
            return False

        model = instance_wrapper.model
        if not model:
            return False

        child_aliases = model.get_child_node_aliases(self.__node.nodeid)
        return key in child_aliases

    async def __get_child_value(self, key: str) -> Optional[IPseudo]:
        """
        Get a child pseudo value by alias.

        Args:
            key: The child node alias

        Returns:
            The child PseudoValue or None
        """
        parent = self.__parent_wkri
        node = self.__node

        if not parent or not hasattr(parent, "_i"):
            return None

        instance_wrapper = parent._i
        if not instance_wrapper:
            return None

        model = instance_wrapper.model
        if not model:
            return None

        child_aliases = model.get_child_node_aliases(node.nodeid)

        if key not in child_aliases:
            raise ValueError(
                f"Semantic node does not have this key: {key} ({', '.join(child_aliases)})"
            )

        # Check cache first
        if key in self.__child_values:
            return self.__child_values[key]

        # Core's lookup within this tile, fetching what a lazy instance lacks
        # first: as TypeScript's SemanticViewModel, on NAPI and WASM.
        child = await instance_wrapper.semantic_child(self.__tile, node, key)
        if child is None:
            return None
        if hasattr(child, "parent_value"):
            child.parent_value = self.__parent_pseudo
        self.__child_values[key] = child
        return child

    async def __as_tile_data(self) -> tuple[None, List[Any]]:
        """
        Convert to tile data format.

        Semantic nodes don't have direct tile values - only their children do.

        Returns:
            Tuple of (None, list of relationships)
        """
        relationships: List[Any] = []
        for child in self.__child_values.values():
            if child and hasattr(child, "get_tile"):
                _, child_relationships = child.get_tile()
                relationships.extend(child_relationships)
        return (None, relationships)

    async def for_json(self) -> Dict[str, Any]:
        """
        Convert to JSON-serializable format.

        Returns all child values as a dictionary.
        """
        return await self.to_object()

    @staticmethod
    async def _create(
        tile: Optional[StaticTile],
        node: StaticNode,
        _value: Any,
        parent: Optional[Any],
    ) -> SemanticViewModel:
        """
        Factory method to create a SemanticViewModel.

        Args:
            tile: The tile containing this semantic node
            node: The StaticNode definition
            _value: Ignored - semantic nodes don't have direct values
            parent: Parent resource instance

        Returns:
            New SemanticViewModel instance
        """
        # Note: value parameter is ignored - semantic nodes don't have direct values,
        # their children are loaded lazily via __get_child_value
        return SemanticViewModel(parent, tile, node)

    def __getattr__(self, name: str) -> Any:
        """
        Attribute access for child values.

        Allows accessing child nodes via attribute syntax: semantic.child_alias
        """
        # Avoid infinite recursion for private attributes
        if name.startswith("_"):
            raise AttributeError(f"'{type(self).__name__}' object has no attribute '{name}'")

        # Return an awaitable that gets the child value

        async def get_child() -> Any:
            return await self.__get(name)

        # Return a coroutine that can be awaited
        return get_child()

    def __setattr__(self, name: str, value: Any) -> None:
        """
        Assigning a child records a pending edit, applied by the resource's
        commit(). Matches TypeScript SemanticViewModel's proxy set.
        """
        if name.startswith("_") or name == "then":
            object.__setattr__(self, name, value)
            return
        instance_wrapper = getattr(self.__parent_wkri, "_i", None)
        if instance_wrapper is None:
            raise ValueError(f"Cannot set {name}: this value has no resource to edit")
        instance_wrapper.assign(self.__tile, self.__node, name, value)
