"""
Pseudo value classes matching TypeScript pseudos.ts

Pseudos wrap raw tile data and provide typed access to values.
Now backed by Rust (RustPseudoValue/RustPseudoList) for single source of truth.

Architecture (matching JS/TS):
- PseudoValue wraps RustPseudoValue (like JS wraps WasmPseudoValue)
- PseudoList wraps RustPseudoList (like JS wraps WasmPseudoList)
- matching_entries() calls Rust - no duplication
"""

from __future__ import annotations

import json
from typing import (
    Any,
    Awaitable,
    Dict,
    Generic,
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

# Import Rust-backed types (from alizarin.alizarin - the compiled Rust module)
try:
    from alizarin.alizarin import RustPseudoValue, RustPseudoList
    HAS_RUST_PSEUDOS = True
except ImportError:
    HAS_RUST_PSEUDOS = False
    RustPseudoValue = None
    RustPseudoList = None


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
    This wraps RustPseudoValue (like JS wraps WasmPseudoValue).

    Attributes:
        _rust: The Rust backing (RustPseudoValue) - all core state lives here
        node: The StaticNode this value belongs to
        tile: The StaticTile containing this value (may be None for root)
        parent_value: Parent PseudoValue for nested structures
        _view_model: Cached ViewModel instance
    """

    __slots__ = (
        "_rust",
        "node",
        "tile",
        "_value",
        "parent_value",
        "_view_model",
        "_parent_wkri",
        "_model",
        "_inner",
        "_snapshot",
    )

    def __init__(
        self,
        node: StaticNode,
        tile: Optional[StaticTile],
        value: Any,
        parent_pseudo: Optional[PseudoValue[Any]] = None,
        parent_wkri: Optional[Any] = None,
        model: Optional[Any] = None,
        rust_value: Optional[Any] = None,
    ) -> None:
        self._rust: Optional[Any] = rust_value
        self.node: StaticNode = node
        self.tile: Optional[StaticTile] = tile
        self._value: Any = value
        self.parent_value: Optional[PseudoValue[Any]] = parent_pseudo
        self._view_model: Optional[VM] = None
        self._parent_wkri: Optional[Any] = parent_wkri
        self._model: Optional[Any] = model
        self._inner: Optional[PseudoValue[Any]] = None
        self._snapshot: Optional[Dict[str, Any]] = None

    @classmethod
    def from_rust(
        cls,
        rust_value: Any,
        parent_wkri: Optional[Any] = None,
        model: Optional[Any] = None,
    ) -> "PseudoValue[Any]":
        """Create PseudoValue from RustPseudoValue (preferred constructor)."""
        from .static_types import StaticNode, StaticTile

        # Get node - try from model first, otherwise reconstruct from Rust JSON
        node_id = rust_value.node_id
        node = None
        if model:
            node = model.get_node_object_from_id(node_id)
        if node is None:
            # Reconstruct from Rust JSON
            node_json_str = rust_value.get_node_json()
            if node_json_str:
                node_dict = json.loads(node_json_str)
                node = StaticNode.from_dict(node_dict)

        # Reconstruct tile from Rust JSON
        tile = None
        tile_json_str = rust_value.get_tile_json()
        if tile_json_str:
            tile_dict = json.loads(tile_json_str)
            tile = StaticTile.from_dict(tile_dict)

        # Get tile data
        tile_data = rust_value.get_tile_data()

        pv = cls(
            node=node,
            tile=tile,
            value=tile_data,
            parent_wkri=parent_wkri,
            model=model,
            rust_value=rust_value,
        )
        return pv

    def __repr__(self) -> str:
        alias = self.node.alias if self.node else "?"
        tile_id = self.tile.tileid if self.tile else "no-tile"
        return f"<PseudoValue {alias} tile={tile_id}>"

    # =========================================================================
    # Property accessors (proxy to Rust when available)
    # =========================================================================

    @property
    def datatype(self) -> str:
        """Get the datatype (may be 'semantic' for inner nodes)."""
        if self._rust:
            return self._rust.datatype
        return self.node.datatype if self.node else "unknown"

    @property
    def is_outer(self) -> bool:
        """Check if this is an outer node (has inner)."""
        if self._rust:
            return self._rust.is_outer
        return self._inner is not None

    @property
    def is_inner(self) -> bool:
        """Check if this is an inner node (datatype overridden to semantic)."""
        if self._rust:
            return self._rust.is_inner
        return False

    @property
    def is_collector(self) -> bool:
        """Check if this is a collector node."""
        if self._rust:
            return self._rust.is_collector
        return False

    @property
    def independent(self) -> bool:
        """Check if this tile is independent (not inherited from parent)."""
        if self._rust:
            return self._rust.independent
        return self.tile is None

    @property
    def inner(self) -> Optional["PseudoValue[Any]"]:
        """Get the inner PseudoValue (if this is an outer node)."""
        if self._inner is None and self._rust and self._rust.is_outer:
            rust_inner = self._rust.get_inner()
            if rust_inner:
                self._inner = PseudoValue.from_rust(
                    rust_inner,
                    parent_wkri=self._parent_wkri,
                    model=self._model,
                )
                self._inner.parent_value = self
        return self._inner

    @inner.setter
    def inner(self, value: Optional["PseudoValue[Any]"]) -> None:
        """Set the inner PseudoValue."""
        self._inner = value
        if self._rust and value and value._rust:
            self._rust.set_inner(value._rust)

    # =========================================================================
    # Value access methods
    # =========================================================================

    async def get_value(self) -> Optional[VM]:
        """
        Get the wrapped value as a ViewModel.

        Lazily creates the ViewModel based on datatype on first access.
        """
        if self._view_model is None:
            from .view_models import get_view_model
            self._view_model = await get_view_model(
                parentPseudo=self,
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
        if self._view_model is not None and hasattr(self._view_model, "__asTileData"):
            tile_data = self._view_model.__asTileData()
            if hasattr(tile_data, "__await__"):
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

    def clear(self) -> None:
        """Clear the cached value."""
        self._view_model = None
        self._value = None

    def get_children(self, direct: Optional[bool] = None) -> List["IPseudo"]:
        """
        Get child pseudo values.

        Matches JS/TS PseudoValue.getChildren() method.

        Args:
            direct: If True, only return direct children (not implemented yet)

        Returns:
            List of child IPseudo objects
        """
        children: List[IPseudo] = []

        # Check cached value for children
        if self._view_model is not None:
            if hasattr(self._view_model, 'get_children'):
                vm_children = self._view_model.get_children()
                if vm_children:
                    children.extend(vm_children)

        # Recurse into inner if present
        if self._inner is not None:
            children.extend(self._inner.get_children(direct))

        return children

    async def get_child_types(self) -> Dict[str, Any]:
        """
        Get types of child nodes.

        Matches JS/TS PseudoValue.getChildTypes() method.

        Returns:
            Dict mapping child aliases to their types
        """
        child_types: Dict[str, Any] = {}

        # Get child types from ViewModel if available
        value = await self.get_value()
        if value is not None and hasattr(value, 'get_child_types'):
            vm_types = value.get_child_types()
            if isinstance(vm_types, dict):
                child_types.update(vm_types)

        # Recurse into inner if present
        if self._inner is not None:
            inner_types = await self._inner.get_child_types()
            child_types.update(inner_types)

        return child_types

    def get_length(self) -> int:
        """
        Get the number of children.

        Matches JS/TS PseudoValue.getLength() method.

        Returns:
            Number of child pseudo values
        """
        return len(self.get_children())


# =============================================================================
# PseudoList Implementation
# =============================================================================

class PseudoList(List[PseudoValue[VM]], Generic[VM]):
    """
    A list of PseudoValue objects.

    Matches TypeScript PseudoList which extends Array.
    This wraps RustPseudoList for matching_entries.

    Attributes:
        alias: The node alias this list represents
        is_single: Whether this list should contain at most one item
        _rust: The Rust backing (RustPseudoList) if available
    """

    def __init__(
        self,
        alias: str,
        values: Optional[List[PseudoValue[VM]]] = None,
        is_single: bool = False,
        rust_list: Optional[Any] = None,
    ) -> None:
        super().__init__(values or [])
        self.alias: str = alias
        self.is_single: bool = is_single
        self._parent_wkri: Optional[Any] = None
        self._parent_model: Optional[Any] = None
        self._rust: Optional[Any] = rust_list

    @classmethod
    def from_rust(
        cls,
        rust_list: Any,
        parent_wkri: Optional[Any] = None,
        model: Optional[Any] = None,
    ) -> "PseudoList[Any]":
        """Create PseudoList from RustPseudoList (preferred constructor)."""
        alias = rust_list.alias
        is_single = rust_list.is_single

        # Wrap each RustPseudoValue
        wrapped_values: List[PseudoValue[Any]] = []
        for rust_value in rust_list:
            pv = PseudoValue.from_rust(rust_value, parent_wkri=parent_wkri, model=model)
            wrapped_values.append(pv)

        result = cls(alias=alias, values=wrapped_values, is_single=is_single, rust_list=rust_list)
        result._parent_wkri = parent_wkri
        result._parent_model = model
        return result

    def __repr__(self) -> str:
        return f"<PseudoList alias={self.alias} len={len(self)} single={self.is_single}>"

    def matching_entries(
        self,
        parent_tile_id: Optional[str],
        nodegroup_id: Optional[str],
        parent_nodegroup_id: Optional[str] = None,
    ) -> List[PseudoValue[VM]]:
        """
        Filter entries matching the given tile and nodegroup context.

        This is critical for the inner/outer value filtering pattern.
        NOW CALLS RUST - single source of truth.

        Args:
            parent_tile_id: The parent tile ID to match against
            nodegroup_id: The nodegroup ID to match against
            parent_nodegroup_id: The parent's nodegroup ID (used to distinguish same vs different nodegroup children)

        Returns:
            List of matching PseudoValue objects
        """
        # Rust is the single source of truth for matching_entries
        if self._rust is None:
            raise RuntimeError(
                "matching_entries requires Rust backing. "
                "PseudoList must be created via from_rust() or have a valid _rust attribute."
            )

        rust_matches = self._rust.matching_entries(parent_tile_id, nodegroup_id, parent_nodegroup_id)
        # Wrap the results
        return [
            PseudoValue.from_rust(rv, parent_wkri=self._parent_wkri, model=self._parent_model)
            for rv in rust_matches
        ]

    async def sorted(self) -> List[Any]:
        """
        Sort values by tile sortorder.

        Matches JS/TS PseudoList.sorted() method.
        """
        resolved = await self.get_value()

        def get_sortorder(val: Any) -> int:
            if val and hasattr(val, '__parentPseudo'):
                pseudo = val.__parentPseudo
                if pseudo and pseudo.tile and hasattr(pseudo.tile, 'sortorder'):
                    return pseudo.tile.sortorder or 0
            return 0

        return sorted(resolved, key=get_sortorder)

    async def get_value(self) -> List[Optional[VM]]:
        """Get all values as ViewModels."""
        return [await v.get_value() for v in self]

    async def for_json(self) -> Any:
        """Get JSON-serializable representation of all values."""
        sorted_values = await self.sorted()
        results = []
        for v in sorted_values:
            if v is not None and hasattr(v, 'for_json'):
                result = v.for_json()
                if hasattr(result, '__await__'):
                    result = await result
                results.append(result)
            else:
                results.append(v)

        # If cardinality-1, unwrap to single value
        if self.is_single:
            return results[0] if results else None
        return results

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
    This is the ONLY place where Rust values should be wrapped in Python classes.

    Args:
        rust_value: The Rust pseudo value to wrap (RustPseudoValue or RustPseudoList)
        parent_wkri: Parent resource instance wrapper
        model: The model wrapper

    Returns:
        Wrapped PseudoValue or PseudoList, or None if rust_value is None
    """
    if rust_value is None:
        return None

    # Check if it's a RustPseudoList (has get_all_values method)
    if hasattr(rust_value, 'get_all_values'):
        return PseudoList.from_rust(rust_value, parent_wkri=parent_wkri, model=model)
    else:
        # It's a single RustPseudoValue
        return PseudoValue.from_rust(rust_value, parent_wkri=parent_wkri, model=model)


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
    When possible, creates a Rust-backed PseudoValue.

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
    # Try to create Rust-backed value if available
    rust_value = None
    if HAS_RUST_PSEUDOS and RustPseudoValue is not None:
        try:
            # Use to_dict() if available (for dataclasses), otherwise __dict__
            if hasattr(node, 'to_dict'):
                node_dict = node.to_dict()
            elif hasattr(node, '__dict__'):
                node_dict = node.__dict__
            else:
                node_dict = {}
            node_json = json.dumps(node_dict)

            tile_json = None
            if tile is not None:
                if hasattr(tile, 'to_dict'):
                    tile_dict = tile.to_dict()
                elif hasattr(tile, '__dict__'):
                    tile_dict = tile.__dict__
                else:
                    tile_dict = {}
                tile_json = json.dumps(tile_dict)

            tile_data_json = json.dumps(value) if value is not None else None
            rust_value = RustPseudoValue(
                node_json=node_json,
                tile_json=tile_json,
                tile_data_json=tile_data_json,
            )
        except Exception:
            # Fall back to pure Python if Rust creation fails
            pass

    return PseudoValue(
        node=node,
        tile=tile,
        value=value,
        parent_pseudo=parent_pseudo,
        parent_wkri=parent_wkri,
        model=model,
        rust_value=rust_value,
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
    When possible, creates a Rust-backed PseudoList.

    Args:
        alias: The node alias
        values: Initial list of values
        is_single: Whether this should contain at most one item
        parent_wkri: Parent resource instance wrapper
        model: The model wrapper

    Returns:
        New PseudoList instance
    """
    # Try to create Rust-backed list if available
    rust_list = None
    if HAS_RUST_PSEUDOS and RustPseudoList is not None:
        try:
            rust_list = RustPseudoList(alias=alias, is_single=is_single)
            # Add values to Rust list if they have Rust backing
            if values:
                for v in values:
                    if v._rust is not None:
                        rust_list.add_value(v._rust)
        except Exception:
            rust_list = None

    result = PseudoList(alias=alias, values=values, is_single=is_single, rust_list=rust_list)
    result._parent_wkri = parent_wkri
    result._parent_model = model
    return result
