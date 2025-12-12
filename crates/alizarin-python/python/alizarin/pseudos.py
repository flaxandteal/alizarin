"""
Pseudo value classes matching TypeScript pseudos.ts

Pseudos wrap raw tile data and provide typed access to values.
Uses Protocol classes for structural typing to match TypeScript interfaces.
"""

from __future__ import annotations

from typing import (
    Any,
    Awaitable,
    Callable,
    Dict,
    Generic,
    Iterator,
    List,
    Optional,
    Protocol,
    Tuple,
    TypeVar,
    Union,
    TYPE_CHECKING,
    runtime_checkable,
)

if TYPE_CHECKING:
    from .static_types import StaticNode, StaticTile
    from .view_models import IViewModel


# =============================================================================
# Type Variables
# =============================================================================

VM = TypeVar("VM", bound="IViewModel")
T = TypeVar("T")


# =============================================================================
# Protocol Definitions (matching TypeScript interfaces.ts)
# =============================================================================

@runtime_checkable
class IPseudo(Protocol):
    """
    Protocol for pseudo values.

    Matches TypeScript IPseudo interface from interfaces.ts.
    """
    parent_value: Optional[IPseudo]
    tile: Any
    node: Any

    def get_value(self) -> Awaitable[Union[IViewModel, None, List[IViewModel]]]:
        """Get the wrapped value as a ViewModel."""
        ...

    async def for_json(self) -> Any:
        """Get JSON-serializable representation."""
        ...

    def is_iterable(self) -> bool:
        """Check if this pseudo represents an iterable (list) value."""
        ...

    def describe_field(self) -> str:
        """Get human-readable field description."""
        ...

    def describe_field_group(self) -> str:
        """Get human-readable field group description."""
        ...


# =============================================================================
# PseudoValue Implementation
# =============================================================================

class PseudoValue(Generic[VM]):
    """
    Wraps a single value from a tile.

    Matches TypeScript PseudoValue<VM extends IViewModel>.
    This corresponds to the wrapper around RustPseudoValue.

    Attributes:
        node: The StaticNode this value belongs to
        tile: The StaticTile containing this value (may be None for root)
        parent_value: Parent PseudoValue for nested structures
        _value: The raw value from tile data
        _view_model: Cached ViewModel instance
    """

    __slots__ = (
        "node",
        "tile",
        "_value",
        "parent_value",
        "_view_model",
        "_parent_wkri",
        "_model",
    )

    def __init__(
        self,
        node: StaticNode,
        tile: Optional[StaticTile],
        value: Any,
        parent_pseudo: Optional[PseudoValue[Any]] = None,
        parent_wkri: Optional[Any] = None,
        model: Optional[Any] = None,
    ) -> None:
        self.node: StaticNode = node
        self.tile: Optional[StaticTile] = tile
        self._value: Any = value
        self.parent_value: Optional[PseudoValue[Any]] = parent_pseudo
        self._view_model: Optional[VM] = None
        self._parent_wkri: Optional[Any] = parent_wkri
        self._model: Optional[Any] = model

    def __repr__(self) -> str:
        alias = self.node.alias if self.node else "?"
        tile_id = self.tile.tileid if self.tile else "no-tile"
        return f"<PseudoValue {alias} tile={tile_id}>"

    async def get_value(self) -> Optional[VM]:
        """
        Get the wrapped value as a ViewModel.

        Lazily creates the ViewModel based on datatype on first access.
        """
        if self._view_model is None:
            from .view_models import get_view_model
            self._view_model = await get_view_model(
                parent_pseudo=self,
                tile=self.tile,
                node=self.node,
                data=self._value,
                parent=self._parent_wkri,
            )
        return self._view_model

    async def for_json(self) -> Any:
        """Get JSON-serializable representation."""
        value = await self.get_value()
        if value is None:
            return None
        if hasattr(value, "for_json"):
            result = value.for_json()
            if hasattr(result, "__await__"):
                return await result
            return result
        return value

    def get_tile(self) -> Tuple[Optional[Any], List[Any]]:
        """
        Get tile data for saving.

        Returns:
            Tuple of (tile_value, relationships)
        """
        # For simple values, return the value and empty relationships
        if self._view_model is not None and hasattr(self._view_model, "__asTileData"):
            tile_data = self._view_model.__asTileData()
            if hasattr(tile_data, "__await__"):
                # Cannot await in sync method, return placeholder
                return (None, [])
            return (tile_data, [])
        return (self._value, [])

    def is_iterable(self) -> bool:
        """Check if this pseudo represents an iterable value."""
        return False

    def describe_field(self) -> str:
        """Get human-readable field description."""
        if self.node:
            return self.node.alias or self.node.name or self.node.nodeid
        return "(unknown field)"

    def describe_field_group(self) -> str:
        """Get human-readable field group description."""
        if self.node and self.node.nodegroup_id:
            return self.node.nodegroup_id
        return "(no group)"


# =============================================================================
# PseudoList Implementation
# =============================================================================

class PseudoList(List[PseudoValue[VM]], Generic[VM]):
    """
    A list of PseudoValue objects.

    Matches TypeScript PseudoList which extends Array.
    This wraps RustPseudoList from Rust.

    Attributes:
        alias: The node alias this list represents
        is_single: Whether this list should contain at most one item
        _parent_wkri: Parent resource instance
        _parent_model: Parent model wrapper
    """

    def __init__(
        self,
        alias: str,
        values: Optional[List[PseudoValue[VM]]] = None,
        is_single: bool = False,
    ) -> None:
        super().__init__(values or [])
        self.alias: str = alias
        self.is_single: bool = is_single
        self._parent_wkri: Optional[Any] = None
        self._parent_model: Optional[Any] = None

    def __repr__(self) -> str:
        return f"<PseudoList alias={self.alias} len={len(self)} single={self.is_single}>"

    def matching_entries(
        self,
        parent_tile_id: Optional[str],
        nodegroup_id: Optional[str],
    ) -> List[PseudoValue[VM]]:
        """
        Filter entries matching the given tile and nodegroup context.

        This is critical for the inner/outer value filtering pattern.
        Matches RustPseudoList::matching_entries in Rust.

        Args:
            parent_tile_id: The parent tile ID to match against
            nodegroup_id: The nodegroup ID to match against

        Returns:
            List of matching PseudoValue objects
        """
        matches: List[PseudoValue[VM]] = []

        for value in self:
            if not isinstance(value, PseudoValue):
                continue

            # Skip if no tile
            if value.tile is None:
                continue

            # Check nodegroup match
            if nodegroup_id is not None:
                if value.tile.nodegroup_id != nodegroup_id:
                    continue

            # Check tile context
            if parent_tile_id is None:
                # Root level - only match tiles without parents
                if value.tile.parenttile_id is not None:
                    continue
            else:
                # Must match parent tile context
                # Either same tile (inner value) or child tile (different nodegroup)
                tile_id = value.tile.tileid
                parent_id = value.tile.parenttile_id

                if tile_id == parent_tile_id:
                    # Same tile - inner value
                    matches.append(value)
                    continue
                elif parent_id == parent_tile_id:
                    # Child tile
                    matches.append(value)
                    continue
                else:
                    # Different context
                    continue

            matches.append(value)

        return matches

    async def get_value(self) -> List[Optional[VM]]:
        """Get all values as ViewModels."""
        return [await v.get_value() for v in self]

    async def for_json(self) -> List[Any]:
        """Get JSON-serializable representation of all values."""
        return [await v.for_json() for v in self]

    def is_iterable(self) -> bool:
        """Check if this pseudo represents an iterable value."""
        return True


# =============================================================================
# Factory Functions
# =============================================================================

def wrap_rust_pseudo(
    rust_value: Any,
    parent_wkri: Any,
    model: Any,
) -> Optional[Union[PseudoValue[Any], PseudoList[Any]]]:
    """
    Wrap a Rust pseudo value in Python objects.

    Matches TypeScript wrapRustPseudo function.
    This creates the second cache layer (Python wrapped objects).

    Args:
        rust_value: The Rust pseudo value to wrap
        parent_wkri: Parent resource instance wrapper
        model: The model wrapper

    Returns:
        Wrapped PseudoValue or PseudoList, or None if rust_value is None
    """
    if rust_value is None:
        return None

    # Check if it's a list type from Rust
    if hasattr(rust_value, "__iter__") and hasattr(rust_value, "alias"):
        # It's a RustPseudoList
        alias = getattr(rust_value, "alias", "")
        is_single = getattr(rust_value, "is_single", False)

        wrapped_values: List[PseudoValue[Any]] = []
        for item in rust_value:
            wrapped_item = _wrap_single_rust_pseudo(item, parent_wkri, model)
            if wrapped_item is not None:
                wrapped_values.append(wrapped_item)

        result = PseudoList(alias=alias, values=wrapped_values, is_single=is_single)
        result._parent_wkri = parent_wkri
        result._parent_model = model
        return result

    # It's a single RustPseudoValue
    return _wrap_single_rust_pseudo(rust_value, parent_wkri, model)


def _wrap_single_rust_pseudo(
    rust_value: Any,
    parent_wkri: Any,
    model: Any,
) -> Optional[PseudoValue[Any]]:
    """
    Wrap a single Rust pseudo value.

    Args:
        rust_value: The Rust pseudo value to wrap
        parent_wkri: Parent resource instance wrapper
        model: The model wrapper

    Returns:
        Wrapped PseudoValue or None
    """
    if rust_value is None:
        return None

    # Extract data from Rust value
    node = getattr(rust_value, "node", None)
    tile = getattr(rust_value, "tile", None)
    value = getattr(rust_value, "value", None)

    if node is None:
        return None

    return PseudoValue(
        node=node,
        tile=tile,
        value=value,
        parent_wkri=parent_wkri,
        model=model,
    )


def create_pseudo_value(
    node: StaticNode,
    tile: Optional[StaticTile],
    value: Any,
    parent_pseudo: Optional[PseudoValue[Any]] = None,
    parent_wkri: Optional[Any] = None,
    model: Optional[Any] = None,
) -> PseudoValue[Any]:
    """
    Create a new PseudoValue.

    Factory function for creating pseudo values from Python.

    Args:
        node: The StaticNode this value belongs to
        tile: The StaticTile containing this value
        value: The raw value
        parent_pseudo: Optional parent pseudo for nested structures
        parent_wkri: Parent resource instance wrapper
        model: The model wrapper

    Returns:
        New PseudoValue instance
    """
    return PseudoValue(
        node=node,
        tile=tile,
        value=value,
        parent_pseudo=parent_pseudo,
        parent_wkri=parent_wkri,
        model=model,
    )


def create_pseudo_list(
    alias: str,
    values: Optional[List[PseudoValue[Any]]] = None,
    is_single: bool = False,
    parent_wkri: Optional[Any] = None,
    model: Optional[Any] = None,
) -> PseudoList[Any]:
    """
    Create a new PseudoList.

    Factory function for creating pseudo lists from Python.

    Args:
        alias: The node alias
        values: Initial list of values
        is_single: Whether this should contain at most one item
        parent_wkri: Parent resource instance wrapper
        model: The model wrapper

    Returns:
        New PseudoList instance
    """
    result = PseudoList(alias=alias, values=values, is_single=is_single)
    result._parent_wkri = parent_wkri
    result._parent_model = model
    return result
