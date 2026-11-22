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

import inspect
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


def _model_name(model: Optional[Any]) -> Optional[str]:
    """The model's name, as TypeScript's wkrm.modelName."""
    wkrm = getattr(model, "wkrm", None) if model is not None else None
    return getattr(wkrm, "model_name", None) or None


def _describe_field(model: Optional[Any], node: Optional[StaticNode], is_list: bool) -> str:
    """Core's describe_field for a pseudo's node and model."""
    from .alizarin import describe_field

    if node is None:
        return "[(uninitialized node)]" if is_list else "Unavailable field"
    return describe_field(_model_name(model), node.name, is_list)


def _describe_field_group(model: Optional[Any], node: Optional[StaticNode], is_list: bool) -> str:
    """Core's describe_field_group: the nodegroup's name by core's rule."""
    from .alizarin import describe_field_group, nodegroup_name

    if node is None:
        return "[(uninitialized node)]" if is_list else "Unavailable field"
    group = nodegroup_name(node.graph_id, node.nodegroup_id) if model is not None and node.nodegroup_id else None
    return describe_field_group(_model_name(model), node.name, group, is_list)


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
        from .static_types import StaticNode, StaticTile, from_core

        # Get node - try from model first, otherwise reconstruct from Rust JSON
        node_id = rust_value.node_id
        node = None
        if model:
            node = model.get_node_objects().get(node_id)
        if node is None:
            # Reconstruct from Rust JSON
            node_json_str = rust_value.get_node_json()
            if node_json_str:
                node_dict = json.loads(node_json_str)
                node = from_core(StaticNode, node_dict)

        # Reconstruct tile from Rust JSON
        tile = None
        tile_json_str = rust_value.get_tile_json()
        if tile_json_str:
            tile_dict = json.loads(tile_json_str)
            tile = from_core(StaticTile, tile_dict)

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
                # An inner is semantic (its children), as TypeScript passes it.
                isInner=self.is_inner,
            )
            # An outer node's children are reached through its value's `_`:
            # the inner (semantic) value, as TypeScript attaches it.
            if self._view_model is not None and self.is_outer and self.inner is not None:
                self._view_model._ = await self.inner.get_value()
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
        from .view_models import as_tile_data_method

        # The view model's own tile form; its __asTileData is name-mangled, so
        # a bare hasattr never found it. An async one can't be awaited here:
        # its value is core's, which this pseudo holds as _value.
        as_tile_data = (
            as_tile_data_method(self._view_model) if self._view_model is not None else None
        )
        if as_tile_data is not None:
            tile_data = as_tile_data()
            if inspect.iscoroutine(tile_data):
                tile_data.close()
            elif not inspect.isawaitable(tile_data):
                return (tile_data, [])
        return (self._value, [])

    def is_iterable(self) -> bool:
        """Check if this pseudo represents an iterable value."""
        return False

    def describe_field(self) -> str:
        """How the ORM names this field: core's, as TypeScript's describeField."""
        return _describe_field(self._model, self.node, False)

    def describe_field_group(self) -> str:
        """How the ORM names this field's group: core's, as TypeScript's describeFieldGroup."""
        return _describe_field_group(self._model, self.node, False)

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

class EditableList(list):
    """
    A multi-valued child's values, recording edits: `append` (or `extend`,
    `+=`) adds a tile from a dict of values by alias, and `del`, `pop` and
    `remove` remove one, all applied by the resource's `commit()`. Matches
    TypeScript's PseudoList push and splice. Inserting or replacing an item
    isn't supported: tile order follows sortorder, and an item's values are
    set on the item.
    """

    def __init__(
        self,
        values: List[Any],
        wrapper: Optional[Any],
        alias: str,
        parent_tile_id: Optional[str],
        tile_ids: List[Optional[str]],
    ) -> None:
        super().__init__(values)
        self._wrapper = wrapper
        self._alias = alias
        self._parent_tile_id = parent_tile_id
        # Each item's tile id, or, for an item appended since the last commit,
        # its edit's index.
        self._tile_ids: List[Any] = list(tile_ids)

    def _require_wrapper(self) -> Any:
        if self._wrapper is None:
            raise ValueError(f"Cannot edit {self._alias}: it has no resource to edit")
        return self._wrapper

    def append(self, item: Any) -> None:
        wrapper = self._require_wrapper()
        ref = wrapper.record_edit({
            "op": "add",
            "alias": self._alias,
            "parent": self._parent_tile_id,
            "values": item or {},
        })
        self._tile_ids.append(ref)
        super().append(item)
        wrapper.set_pending_value(self._parent_tile_id, self._alias, self)

    def extend(self, items: Any) -> None:
        for item in items:
            self.append(item)

    def __iadd__(self, items: Any) -> "EditableList":
        self.extend(items)
        return self

    def __delitem__(self, index: Any) -> None:
        wrapper = self._require_wrapper()
        positions = range(len(self))[index]
        positions = list(positions) if isinstance(index, slice) else [positions]
        tile_ids = [self._tile_ids[i] for i in positions]
        if any(not isinstance(tile, str) for tile in tile_ids):
            raise ValueError(
                "Cannot remove an item added since the last commit(): discard() instead"
            )
        for tile in tile_ids:
            wrapper.record_edit({"op": "remove", "tile": tile})
        for i in sorted(positions, reverse=True):
            del self._tile_ids[i]
            super().__delitem__(i)
        wrapper.set_pending_value(self._parent_tile_id, self._alias, self)

    def pop(self, index: int = -1) -> Any:
        item = self[index]
        del self[index]
        return item

    def remove(self, value: Any) -> None:
        del self[self.index(value)]

    def insert(self, index: int, item: Any) -> None:
        raise ValueError("Add items with append: tile order follows sortorder")

    def __setitem__(self, index: Any, value: Any) -> None:
        raise ValueError(
            f"Set values on the {self._alias} item instead of replacing it"
        )


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

    def _node(self) -> Optional[StaticNode]:
        if self._parent_model is None:
            return None
        return self._parent_model.get_node_objects_by_alias().get(self.alias)

    def describe_field(self) -> str:
        """How the ORM names this list field, in brackets: as TypeScript's."""
        return _describe_field(self._parent_model, self._node(), True)

    def describe_field_group(self) -> str:
        """How the ORM names this list field's group, in brackets: as TypeScript's."""
        return _describe_field_group(self._parent_model, self._node(), True)

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
        Values in tile sortorder: tiles without one after the numbered ones,
        ties and unnumbered tiles in load order (as core orders semantic
        children).

        Matches JS/TS PseudoList.sorted() method.
        """

        def sortorder_key(value: Any) -> tuple[bool, int]:
            tile = getattr(value, "tile", None)
            sortorder = getattr(tile, "sortorder", None)
            return (sortorder is None, sortorder or 0)

        # Stable, so equal keys keep load order.
        return [await v.get_value() for v in sorted(self, key=sortorder_key)]

    async def get_value(self) -> List[Optional[VM]]:
        """
        Get all values as ViewModels: an EditableList, so append and del
        record edits, as TypeScript's PseudoList push and splice do.
        """
        values = [await v.get_value() for v in self]
        wrapper = getattr(self._parent_wkri, "_i", None)
        first_tile = self[0].tile if len(self) > 0 else None
        return EditableList(
            values,
            wrapper=wrapper,
            alias=self.alias,
            parent_tile_id=first_tile.parenttile_id if first_tile else None,
            tile_ids=[v.tile.tileid if v.tile else None for v in self],
        )

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
