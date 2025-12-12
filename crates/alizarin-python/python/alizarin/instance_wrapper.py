"""
ResourceInstanceWrapper with three-layer caching architecture.

Matches TypeScript ResourceInstanceWrapper in graphManager.ts
"""

from __future__ import annotations

import asyncio
from typing import (
    Any,
    Awaitable,
    Callable,
    Dict,
    Generic,
    Iterator,
    List,
    Optional,
    Tuple,
    TYPE_CHECKING,
    TypeVar,
    Union,
)

if TYPE_CHECKING:
    from .static_types import StaticGraph, StaticTile, StaticNode, StaticResource
    from .model_wrapper import ResourceModelWrapper
    from .graph_manager import IRIVM, IModelWrapper

from .pseudos import PseudoValue, PseudoList, wrap_rust_pseudo

# Type Variables
RIVM = TypeVar("RIVM", bound="IRIVM")
T = TypeVar("T")

# Type alias for GetMeta callback
GetMeta = Optional[Callable[[Any], Awaitable[Optional[Dict[str, Any]]]]]


class ResourceInstanceWrapper(Generic[RIVM]):
    """
    Wrapper for a resource instance with three-layer caching.

    Cache Layer Architecture:
    1. Rust pseudo_cache - Authoritative data (HashMap<String, RustPseudoList>)
    2. Python _pseudo_cache - Wrapped Python objects (Dict[str, PseudoList])
    3. Python value_cache - JSON-serializable cache (Dict[str, Dict[str, Any]])

    Matches TypeScript ResourceInstanceWrapper<RIVM>.
    """

    __slots__ = (
        "wkri",
        "wasm_wrapper",
        "model",
        "_pseudo_cache",
        "value_cache",
        "resource_id",
        "graph_id",
        "resource",
        "_tile_loader",
    )

    def __init__(
        self,
        wkri: RIVM,
        wasm_wrapper: Any,
        model: ResourceModelWrapper,
    ) -> None:
        """
        Initialize ResourceInstanceWrapper.

        Args:
            wkri: WKRI instance (resource instance view model)
            wasm_wrapper: Rust wrapper instance
            model: ResourceModelWrapper for the model
        """
        self.wkri: RIVM = wkri
        self.wasm_wrapper: Any = wasm_wrapper
        self.model: ResourceModelWrapper = model

        # Layer 2: Python pseudo cache (wrapped objects)
        self._pseudo_cache: Dict[str, PseudoList[Any]] = {}

        # Layer 3: JSON value cache
        self.value_cache: Optional[Dict[str, Dict[str, Any]]] = None

        # Resource data
        self.resource_id: Optional[str] = None
        self.graph_id: Optional[str] = None
        self.resource: Optional[StaticResource] = None

        # Tile loader callback
        self._tile_loader: Optional[Callable[[str], Awaitable[List[StaticTile]]]] = None

    def set_tile_loader(
        self,
        loader: Callable[[str], Awaitable[List[StaticTile]]],
    ) -> None:
        """
        Set the tile loader callback for lazy loading.

        Args:
            loader: Async function that loads tiles for a nodegroup
        """
        self._tile_loader = loader
        if self.wasm_wrapper and hasattr(self.wasm_wrapper, "set_tile_loader"):
            self.wasm_wrapper.set_tile_loader(loader)

    async def retrieve_pseudo(
        self,
        key: str,
        default: Any = None,
        raise_error: bool = False,
    ) -> Optional[List[PseudoValue[Any]]]:
        """
        Retrieve a pseudo value with two-tier caching.

        Matches TypeScript ResourceInstanceWrapper.retrievePseudo

        Flow:
        1. Check Python _pseudo_cache (Layer 2)
        2. If miss, query Rust pseudo_cache (Layer 1) via getCachedPseudo
        3. Wrap result and store in Python cache
        4. Return wrapped value

        Args:
            key: The node alias to retrieve
            default: Default value if not found
            raise_error: Whether to raise KeyError if not found

        Returns:
            List of PseudoValue objects, or default/None
        """
        # Check Python cache first (Layer 2)
        if key in self._pseudo_cache:
            result = self._pseudo_cache[key]
            if isinstance(result, PseudoList) and len(result) > 0:
                return list(result)

        # Query Rust cache (Layer 1)
        rust_value = None
        if self.wasm_wrapper and hasattr(self.wasm_wrapper, "get_cached_pseudo"):
            rust_value = self.wasm_wrapper.get_cached_pseudo(key)

        if rust_value is not None:
            # Wrap the Rust value
            wrapped = wrap_rust_pseudo(rust_value, self.wkri, self.model)

            if wrapped is not None:
                # Store in Python cache (Layer 2)
                if isinstance(wrapped, PseudoList):
                    self._pseudo_cache[key] = wrapped
                    return list(wrapped)
                else:
                    # Single value - wrap in list
                    pseudo_list = PseudoList(alias=key, values=[wrapped], is_single=True)
                    pseudo_list._parent_wkri = self.wkri
                    pseudo_list._parent_model = self.model
                    self._pseudo_cache[key] = pseudo_list
                    return [wrapped]

        if raise_error:
            raise KeyError(f"Pseudo value not found: {key}")

        return default

    async def has_pseudo(self, key: str) -> bool:
        """
        Check if a pseudo value exists.

        Args:
            key: The node alias to check

        Returns:
            True if the pseudo exists
        """
        if key in self._pseudo_cache:
            return len(self._pseudo_cache[key]) > 0

        if self.wasm_wrapper and hasattr(self.wasm_wrapper, "has_cached_pseudo"):
            return self.wasm_wrapper.has_cached_pseudo(key)

        return False

    def set_pseudo(self, key: str, value: Any) -> None:
        """
        Set a pseudo value.

        Args:
            key: The node alias
            value: The value to set
        """
        if isinstance(value, PseudoList):
            self._pseudo_cache[key] = value
        elif isinstance(value, PseudoValue):
            pseudo_list = PseudoList(alias=key, values=[value], is_single=True)
            pseudo_list._parent_wkri = self.wkri
            pseudo_list._parent_model = self.model
            self._pseudo_cache[key] = pseudo_list
        elif isinstance(value, list):
            pseudo_list = PseudoList(alias=key, values=value, is_single=False)
            pseudo_list._parent_wkri = self.wkri
            pseudo_list._parent_model = self.model
            self._pseudo_cache[key] = pseudo_list

    async def set_default_pseudo(self, key: str, value: Any) -> Any:
        """
        Set a pseudo value if not already set.

        Args:
            key: The node alias
            value: The default value

        Returns:
            Existing value if present, otherwise the new value
        """
        existing = await self.retrieve_pseudo(key)
        if existing is not None and len(existing) > 0:
            return existing

        self.set_pseudo(key, value)
        return await self.retrieve_pseudo(key)

    async def populate(
        self,
        tiles: Optional[List[StaticTile]] = None,
        lazy: bool = False,
    ) -> None:
        """
        Populate the wrapper with tile data.

        Matches TypeScript ResourceInstanceWrapper.populate

        Flow:
        1. Call Rust populate to build Layer 1 (pseudo_cache)
        2. Clear Python Layer 2 cache (Rust is now source of truth)
        3. Layer 3 (value_cache) remains unpopulated until buildValueCache

        Args:
            tiles: List of tiles to populate with (optional)
            lazy: Whether to use lazy loading
        """
        if self.wasm_wrapper and hasattr(self.wasm_wrapper, "populate"):
            if tiles:
                # Convert tiles to format Rust expects
                tiles_data = [self._tile_to_dict(t) for t in tiles]
                await asyncio.coroutine(lambda: self.wasm_wrapper.populate(tiles_data))()
            else:
                await asyncio.coroutine(lambda: self.wasm_wrapper.populate([]))()

        # Clear Python cache - Rust pseudo_cache is now the source of truth
        self._pseudo_cache = {}

    async def load_nodes(self, aliases: List[str]) -> None:
        """
        Load specific node aliases.

        Args:
            aliases: List of node aliases to load
        """
        for alias in aliases:
            await self.retrieve_pseudo(alias)

    def all_entries(self) -> Iterator[Tuple[str, PseudoList[Any]]]:
        """
        Iterate over all cached entries.

        Returns:
            Iterator of (alias, PseudoList) tuples
        """
        return iter(self._pseudo_cache.items())

    def add_pseudo(
        self,
        child_node: StaticNode,
        tile: Optional[StaticTile],
        node: StaticNode,
    ) -> PseudoValue[Any]:
        """
        Add a new pseudo value.

        Args:
            child_node: The child node definition
            tile: The tile (optional)
            node: The parent node

        Returns:
            New PseudoValue instance
        """
        from .pseudos import create_pseudo_value

        pseudo = create_pseudo_value(
            node=child_node,
            tile=tile,
            value=None,
            parent_wkri=self.wkri,
            model=self.model,
        )

        alias = child_node.alias or child_node.nodeid
        if alias not in self._pseudo_cache:
            self._pseudo_cache[alias] = PseudoList(alias=alias, values=[], is_single=False)
            self._pseudo_cache[alias]._parent_wkri = self.wkri
            self._pseudo_cache[alias]._parent_model = self.model

        self._pseudo_cache[alias].append(pseudo)
        return pseudo

    async def get_value_cache(
        self,
        build: bool = True,
        get_meta: GetMeta = None,
    ) -> Optional[Dict[str, Dict[str, Any]]]:
        """
        Get the JSON value cache (Layer 3).

        Matches TypeScript ResourceInstanceWrapper.getValueCache

        Args:
            build: Whether to build the cache if not present
            get_meta: Optional callback for getting metadata

        Returns:
            Value cache dictionary or None
        """
        if build:
            self.value_cache = await self.build_value_cache(get_meta)
        return self.value_cache

    async def build_value_cache(
        self,
        get_meta: GetMeta = None,
    ) -> Dict[str, Dict[str, Any]]:
        """
        Build the JSON-serializable value cache (Layer 3) from Layer 2.

        Matches TypeScript ResourceInstanceWrapper.buildValueCache

        Flow:
        1. Iterate over _pseudo_cache (Layer 2)
        2. Call __forJsonCache on each pseudo value
        3. Organize by tile ID and node ID
        4. Return structured cache

        Args:
            get_meta: Optional callback for getting metadata

        Returns:
            Cache organized by tile ID and node ID
        """
        cache_by_tile: Dict[str, Dict[str, Any]] = {}

        for alias, pseudo_list in self._pseudo_cache.items():
            if not pseudo_list:
                continue

            for pseudo in pseudo_list:
                if not isinstance(pseudo, PseudoValue):
                    continue

                value = await pseudo.get_value()

                # Only cache single values (not lists)
                if pseudo.tile and value is not None:
                    # Get cacheable JSON
                    cache_json = await self._for_json_cache(value, get_meta)

                    if cache_json is not None:
                        tile_id = pseudo.tile.tileid
                        node_id = pseudo.node.nodeid

                        if tile_id and tile_id not in cache_by_tile:
                            cache_by_tile[tile_id] = {}

                        if tile_id:
                            cache_by_tile[tile_id][node_id] = cache_json

        return cache_by_tile

    async def _for_json_cache(
        self,
        value: Any,
        get_meta: GetMeta,
    ) -> Any:
        """
        Helper to get JSON cache representation of a value.

        Args:
            value: The value to convert
            get_meta: Optional metadata callback

        Returns:
            JSON-serializable cache representation
        """
        if hasattr(value, "__for_json_cache"):
            result = value.__for_json_cache(get_meta)
            if hasattr(result, "__await__"):
                return await result
            return result
        elif hasattr(value, "for_json"):
            result = value.for_json()
            if hasattr(result, "__await__"):
                return await result
            return result
        return value

    async def get_root(self) -> Optional[PseudoValue[Any]]:
        """
        Get root pseudo value.

        Returns:
            Root PseudoValue or None
        """
        root_node = self.model.get_root_node()
        if not root_node:
            return None

        from .pseudos import create_pseudo_value
        return create_pseudo_value(
            node=root_node,
            tile=None,
            value=None,
            parent_wkri=self.wkri,
            model=self.model,
        )

    async def get_root_view_model(self) -> Any:
        """
        Get root node ViewModel.

        Alias for get_root (matches TypeScript API).

        Returns:
            Root SemanticViewModel
        """
        root_node = self.model.get_root_node()
        if not root_node:
            return None

        from .semantic import SemanticViewModel
        root_vm = SemanticViewModel(self.wkri, None, root_node)
        return root_vm

    async def set_orm_attribute(self, key: str, value: Any) -> None:
        """
        Set an ORM attribute.

        Args:
            key: Attribute name
            value: Attribute value
        """
        if self.wasm_wrapper and hasattr(self.wasm_wrapper, "set_orm_attribute"):
            self.wasm_wrapper.set_orm_attribute(key, value)

    async def get_orm_attribute(self, key: str) -> Any:
        """
        Get an ORM attribute.

        Args:
            key: Attribute name

        Returns:
            Attribute value
        """
        if self.wasm_wrapper and hasattr(self.wasm_wrapper, "get_orm_attribute"):
            return self.wasm_wrapper.get_orm_attribute(key)
        return None

    def clear_caches(self) -> None:
        """Clear all Python caches (Layer 2 and 3)."""
        self._pseudo_cache = {}
        self.value_cache = None

    def _tile_to_dict(self, tile: StaticTile) -> Dict[str, Any]:
        """
        Convert StaticTile to dictionary for Rust.

        Args:
            tile: The tile to convert

        Returns:
            Dictionary representation
        """
        return {
            "tileid": tile.tileid,
            "nodegroup_id": tile.nodegroup_id,
            "resourceinstance_id": tile.resourceinstance_id,
            "data": tile.data,
            "parenttile_id": tile.parenttile_id,
            "sortorder": tile.sortorder,
            "provisionaledits": tile.provisionaledits,
        }


class ResourceInstanceWrapperCore:
    """
    Core functionality for ResourceInstanceWrapper.

    Provides static methods that can be called from tests.
    Matches functionality in Rust ResourceInstanceWrapperCore.
    """

    @staticmethod
    def matches_semantic_child(
        parent_tile_id: Optional[str],
        parent_node_id: str,
        child_node: StaticNode,
        tile: StaticTile,
    ) -> bool:
        """
        Check if a tile/node combination matches as a semantic child.

        Matches the semantic child matching logic from TypeScript semantic.ts
        and Rust instance_wrapper.rs.

        Three branches:
        1. Different nodegroup + parent_tile_id exists + (null parenttile_id OR matching parent tile)
        2. Same nodegroup + parent_tile_id exists + same tile + not collector
        3. Different nodegroup + is collector (always matches)

        Args:
            parent_tile_id: Parent tile ID (may be None for root)
            parent_node_id: Parent node ID (nodegroup)
            child_node: The child node to check
            tile: The tile to check

        Returns:
            True if this is a matching semantic child
        """
        child_nodegroup = child_node.nodegroup_id
        tile_nodegroup = tile.nodegroup_id

        # Branch 1: Different nodegroup + correct parent tile relationship
        # PORT: instance_wrapper.rs lines 370-381
        # Only matches if parent_tile_id is provided (not None)
        if child_nodegroup != parent_node_id and parent_tile_id is not None:
            # Check if tile.parenttile_id is None or equals parent_tile_id
            parent_matches = (
                tile.parenttile_id is None or
                tile.parenttile_id == parent_tile_id
            )
            if parent_matches:
                return True

        # Branch 2: Same nodegroup + shared tile + not collector
        # PORT: instance_wrapper.rs lines 385-393
        # Only matches if parent_tile_id is provided (not None)
        if child_nodegroup == parent_node_id and parent_tile_id is not None:
            if tile.tileid == parent_tile_id and not child_node.is_collector:
                return True

        # Branch 3: Different nodegroup + is collector (always matches)
        # PORT: instance_wrapper.rs lines 398-401
        if child_nodegroup != parent_node_id and child_node.is_collector:
            return True

        return False


__all__ = [
    "ResourceInstanceWrapper",
    "ResourceInstanceWrapperCore",
    "GetMeta",
]
