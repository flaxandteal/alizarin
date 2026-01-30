"""
View Model classes matching TypeScript viewModels.ts

ViewModels wrap tile data and provide typed, navigable interfaces.
Each ViewModel has:
- _create() static factory method
- for_json() for JSON serialization
- __forJsonCache() for cache entries (optional)
- __asTileData() to convert back to tile data format
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
    TYPE_CHECKING,
    TypeVar,
    Union,
    runtime_checkable,
)
from datetime import datetime
from dataclasses import dataclass, field
import json
import re

if TYPE_CHECKING:
    from .static_types import (
        StaticNode,
        StaticTile,
        StaticDomainValue,
        StaticValue,
        StaticConcept,
        StaticResourceReference,
    )
    from .pseudos import PseudoValue, IPseudo

# Import SemanticViewModel from its own module
from .semantic import SemanticViewModel

# Type variable for generic ViewModels
T = TypeVar('T')
RIVM = TypeVar('RIVM')

# Default language constant
DEFAULT_LANGUAGE = "en"


# ============================================================================
# Cache Entry Classes
# ============================================================================

@dataclass
class ConceptValueCacheEntry:
    """
    Cache entry for concept values.

    Matches TypeScript ConceptValueCacheEntry.
    """
    datatype: str = 'concept'
    id: str = ''
    value: str = ''
    conceptId: Optional[str] = None
    meta: Dict[str, Any] = None

    def __post_init__(self):
        if self.meta is None:
            self.meta = {}


@dataclass
class ConceptListCacheEntry:
    """
    Cache entry for concept lists.

    Matches TypeScript ConceptListCacheEntry.
    """
    datatype: str = 'concept-list'
    _: List[ConceptValueCacheEntry] = None
    meta: Dict[str, Any] = None

    def __post_init__(self):
        if self._ is None:
            self._ = []
        if self.meta is None:
            self.meta = {}
        # Ensure all entries are ConceptValueCacheEntry instances
        self._ = [
            e if isinstance(e, ConceptValueCacheEntry) else ConceptValueCacheEntry(**e)
            for e in self._
        ]


@dataclass
class ResourceInstanceCacheEntry:
    """
    Cache entry for resource instance references.

    Matches TypeScript ResourceInstanceCacheEntry.
    """
    datatype: str = 'resource-instance'
    id: str = ''
    type: str = ''
    graphId: str = ''
    title: Optional[str] = None
    meta: Dict[str, Any] = None

    def __post_init__(self):
        if self.meta is None:
            self.meta = {}
        # Prefer title from meta if available
        if 'title' in self.meta:
            self.title = self.meta['title']


@dataclass
class ResourceInstanceListCacheEntry:
    """
    Cache entry for resource instance lists.

    Matches TypeScript ResourceInstanceListCacheEntry.
    """
    datatype: str = 'resource-instance-list'
    _: List[ResourceInstanceCacheEntry] = None
    meta: Dict[str, Any] = None

    def __post_init__(self):
        if self._ is None:
            self._ = []
        if self.meta is None:
            self.meta = {}
        # Ensure all entries are ResourceInstanceCacheEntry instances
        self._ = [
            e if isinstance(e, ResourceInstanceCacheEntry) else ResourceInstanceCacheEntry(**e)
            for e in self._
        ]


# Type alias for GetMeta callback
GetMeta = Optional[Callable[[Any], Awaitable[Optional[Dict[str, Any]]]]]


# ============================================================================
# AttrPromise - Chainable Promise for Lazy Loading
# ============================================================================

class AttrPromise:
    """
    Promise-like object that supports chained property access.

    Matches TypeScript AttrPromise - allows syntax like:
        await promise.foo.bar

    Which is equivalent to:
        await (await (await promise).foo).bar

    This enables lazy-loading chained traversal through related resources.
    """

    def __init__(self, coro: Awaitable[Any]):
        """
        Create an AttrPromise from a coroutine.

        Args:
            coro: A coroutine that will resolve to a value
        """
        self._coro = coro

    def __await__(self):
        """Make this object awaitable"""
        return self._coro.__await__()

    def __getattr__(self, key: str):
        """
        Chain property access through promises.

        When you access a property on this promise, it returns another
        promise that will first await this promise, then access the
        property on the resolved value.
        """
        # Don't intercept private/internal attributes
        if key.startswith('_'):
            return object.__getattribute__(self, key)

        async def get_chained_attr():
            # First await this promise to get the resolved value
            resolved = await self._coro
            # Then get the attribute from the resolved value
            if resolved is None:
                return None
            # If it's a method, return it bound
            attr = getattr(resolved, key)
            return attr

        # Return another AttrPromise for further chaining
        return AttrPromise(get_chained_attr())

    def __getitem__(self, key: Union[int, str]):
        """
        Support indexing through promises (for lists/dicts).

        Allows: await promise[0].foo
        """
        async def get_indexed():
            resolved = await self._coro
            if resolved is None:
                return None
            return resolved[key]

        return AttrPromise(get_indexed())


# ============================================================================
# Protocol Definitions
# ============================================================================

@runtime_checkable
class IViewModel(Protocol):
    """
    Base protocol for all view models.

    Matches TypeScript IViewModel interface from interfaces.ts.
    """

    __parentPseudo: Optional[IPseudo]
    _: Optional[Union[IViewModel, Awaitable[IViewModel]]]

    async def for_json(self) -> Any:
        """Convert to JSON-serializable format."""
        ...

    def describe_field(self) -> Optional[str]:
        """Get field description from parent pseudo."""
        ...

    def describe_field_group(self) -> Optional[str]:
        """Get field group description from parent pseudo."""
        ...


# ============================================================================
# List View Models
# ============================================================================

class ConceptListViewModel(list):
    """
    List of concept values.

    Matches TypeScript ConceptListViewModel extends Array.
    """

    def __init__(self, *items):
        super().__init__(items)
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['IPseudo'] = None
        self._value: Optional[Awaitable[List[Optional['ConceptValueViewModel']]]] = None

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    async def for_json(self) -> Optional[List[Any]]:
        """Convert to JSON array"""
        value = await self._value if self._value else None
        return [await v.for_json() if v else None for v in value] if value else None

    async def __forJsonCache(self, getMeta: GetMeta) -> ConceptListCacheEntry:
        """Build cache entry for this list"""
        entries = []
        for item_promise in self:
            item = await item_promise if hasattr(item_promise, '__await__') else item_promise
            if item:
                entry = await item.__forJsonCache(getMeta)
                entries.append(entry)

        meta = await getMeta(self) if getMeta else None
        return ConceptListCacheEntry(_=entries, meta=meta)

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
        cacheEntry: Optional[ConceptListCacheEntry] = None
    ) -> 'ConceptListViewModel':
        """
        Create a ConceptListViewModel.

        Matches TypeScript ConceptListViewModel._create
        """
        nodeid = node.nodeid
        val: List[Union['ConceptValueViewModel', Awaitable[Optional['ConceptValueViewModel']]]] = []

        if not tile.data.get(nodeid):
            tile.data[nodeid] = None

        if value is not None:
            tile.data[nodeid] = []
            if not isinstance(value, list):
                raise ValueError(
                    f"Cannot set concept list value on node {nodeid} except via array: {json.dumps(value)}"
                )

            # Create ConceptValueViewModel for each item
            val = []
            for i, item in enumerate(value):
                if isinstance(item, ConceptValueViewModel):
                    val.append(item)
                else:
                    cache_item = cacheEntry._[i] if cacheEntry and i < len(cacheEntry._) else None
                    val.append(ConceptValueViewModel._create(tile, node, item, cache_item))

            # Extract IDs when all items resolve
            async def set_tile_data():
                resolved = [await v if hasattr(v, '__await__') else v for v in val]
                ids = [
                    (await v.getValue()).id if v else None
                    for v in resolved
                ]
                tile.data[nodeid] = ids

            # Schedule tile data update (fire-and-forget in Python)
            import asyncio
            asyncio.create_task(set_tile_data())
        else:
            value = []

        return ConceptListViewModel(*value)

    async def __asTileData(self) -> Optional[List[Any]]:
        """Convert to tile data format"""
        return await self._value if self._value else None


class ResourceInstanceListViewModel(list):
    """
    List of resource instance references.

    Matches TypeScript ResourceInstanceListViewModel extends Array.
    """

    def __init__(self, *items):
        super().__init__(items)
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['IPseudo'] = None
        self._value: Optional[Awaitable[List[Optional['ResourceInstanceViewModel']]]] = None

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    async def for_json(self) -> Optional[List[Any]]:
        """Convert to JSON array"""
        value = await self._value if self._value else None
        return [await v.for_json() if v else None for v in value] if value else None

    async def __forJsonCache(self, getMeta: GetMeta) -> ResourceInstanceListCacheEntry:
        """Build cache entry for this list"""
        entries = []
        for item_promise in self:
            item = await item_promise if hasattr(item_promise, '__await__') else item_promise
            if item:
                entry = await item.__forJsonCache(getMeta)
                entries.append(entry)

        meta = await getMeta(self) if getMeta else None
        return ResourceInstanceListCacheEntry(_=entries, meta=meta)

    @staticmethod
    async def _create(
        tile: Optional['StaticTile'],
        node: 'StaticNode',
        value: Any,
        cacheEntry: Optional[ResourceInstanceListCacheEntry] = None
    ) -> Optional['ResourceInstanceListViewModel']:
        """
        Create a ResourceInstanceListViewModel.

        Matches TypeScript ResourceInstanceListViewModel._create
        """
        nodeid = node.nodeid

        if tile:
            if nodeid not in tile.data:
                tile.data[nodeid] = None

            if value is not None:
                tile.data[nodeid] = []
                if not isinstance(value, list):
                    raise ValueError(
                        f"Cannot set resource instance list value on node {nodeid} except via array: {json.dumps(value)}"
                    )

                # Create ResourceInstanceViewModel for each item
                val = []
                for i, item in enumerate(value):
                    if isinstance(item, ResourceInstanceViewModel):
                        val.append(item)
                    else:
                        cache_item = cacheEntry._[i] if cacheEntry and i < len(cacheEntry._) else None
                        val.append(ResourceInstanceViewModel._create(tile, node, item, cache_item))

                # Extract IDs when all items resolve
                async def set_tile_data():
                    resolved = [await v if hasattr(v, '__await__') else v for v in val]
                    ids = [v.id if v else None for v in resolved]
                    tile.data[nodeid] = [{'resourceId': id} for id in ids if id]

                # Schedule tile data update
                import asyncio
                asyncio.create_task(set_tile_data())
                value = val
        else:
            value = []

        if not tile or not value:
            return None

        return ResourceInstanceListViewModel(*value)

    async def __asTileData(self) -> Optional[List[Any]]:
        """Convert to tile data format"""
        return await self._value if self._value else None


class DomainValueListViewModel(list):
    """
    List of domain values.

    Matches TypeScript DomainValueListViewModel extends Array.
    """

    def __init__(self, *items):
        super().__init__(items)
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['PseudoValue'] = None
        self._value: Optional[Awaitable[List[Optional['DomainValueViewModel']]]] = None

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    async def for_json(self) -> Optional[List[Any]]:
        """Convert to JSON array"""
        value = await self._value if self._value else None
        return [await v.for_json() if v else None for v in value] if value else None

    def __forJsonCache(self) -> None:
        """Domain values are on the graph, no caching needed"""
        return None

    @staticmethod
    async def _create(
        tile: Optional['StaticTile'],
        node: 'StaticNode',
        value: Any,
    ) -> 'DomainValueListViewModel':
        """
        Create a DomainValueListViewModel.

        Matches TypeScript DomainValueListViewModel._create
        """
        nodeid = node.nodeid

        if tile:
            if nodeid not in tile.data:
                tile.data[nodeid] = None

            if value is not None:
                tile.data[nodeid] = []
                if not isinstance(value, list):
                    raise ValueError(
                        f"Cannot set domain value list on node {nodeid} except via array: {json.dumps(value)}"
                    )

                # Create DomainValueViewModel for each item
                val = [
                    item if isinstance(item, DomainValueViewModel) else DomainValueViewModel._create(tile, node, item)
                    for item in value
                ]

                # Extract IDs when all items resolve
                async def set_tile_data():
                    resolved = [await v if hasattr(v, '__await__') else v for v in val]
                    ids = [
                        (await v._value).id if v else None
                        for v in resolved
                    ]
                    tile.data[nodeid] = ids

                # Schedule tile data update
                import asyncio
                asyncio.create_task(set_tile_data())
            else:
                value = []
        else:
            value = []

        return DomainValueListViewModel(*value)

    async def __asTileData(self) -> Optional[List[Any]]:
        """Convert to tile data format"""
        value = await self._value if self._value else None
        return value


# ============================================================================
# Value View Models
# ============================================================================

class ConceptValueViewModel(str):
    """
    ViewModel for concept values from RDM.

    Matches TypeScript ConceptValueViewModel extends String.
    """

    def __new__(cls, value: 'StaticValue', collection_id: Optional[str] = None):
        return str.__new__(cls, value.value)

    def __init__(self, value: 'StaticValue', collection_id: Optional[str] = None):
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['IPseudo'] = None
        self._value: Union['StaticValue', Awaitable['StaticValue']] = value
        self._collection_id: Optional[str] = collection_id

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    async def for_json(self) -> 'StaticValue':
        """Convert to JSON (StaticValue)"""
        return await self._value if hasattr(self._value, '__await__') else self._value

    async def __forJsonCache(self, getMeta: GetMeta) -> ConceptValueCacheEntry:
        """Build cache entry"""
        value = await self._value if hasattr(self._value, '__await__') else self._value
        meta = await getMeta(self) if getMeta else None
        return ConceptValueCacheEntry(
            id=value.id,
            value=value.value,
            conceptId=getattr(value, '__conceptId', None),
            meta=meta
        )

    def getValue(self) -> Union['StaticValue', Awaitable['StaticValue']]:
        """Get the underlying StaticValue"""
        return self._value

    async def parent(self) -> Optional['ConceptValueViewModel']:
        """
        Get the parent concept value, if this concept has a parent in the hierarchy.

        Returns a new ConceptValueViewModel for the parent, or None if no parent.
        Raises RuntimeError if the collection doesn't support hierarchy lookups.
        """
        from .rdm import RDM

        value = await self._value if hasattr(self._value, '__await__') else self._value
        concept_id = getattr(value, '__conceptId', None)
        if not concept_id or not self._collection_id:
            return None

        collection = await RDM.retrieveCollection(self._collection_id)
        if not hasattr(collection, 'get_parent_id'):
            raise RuntimeError(
                f"Collection {self._collection_id} does not support hierarchy lookups. "
                "Ensure WASM is initialized and the collection is a StaticCollection."
            )

        parent_id = collection.get_parent_id(concept_id)
        if not parent_id:
            return None  # Top-level concept

        parent_concept = collection.__allConcepts.get(parent_id)
        if not parent_concept or not hasattr(parent_concept, 'getPrefLabel'):
            return None

        parent_value = parent_concept.getPrefLabel()
        return ConceptValueViewModel(parent_value, self._collection_id)

    async def ancestors(self) -> List['ConceptValueViewModel']:
        """
        Get all ancestor concept values, from immediate parent to root.

        Returns a list of ConceptValueViewModels for ancestors.
        """
        result: List['ConceptValueViewModel'] = []
        current: Optional['ConceptValueViewModel'] = self

        while current is not None:
            current = await current.parent()
            if current is not None:
                result.append(current)

        return result

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
        cacheEntry: Optional[ConceptValueCacheEntry]
    ) -> Optional['ConceptValueViewModel']:
        """
        Create a ConceptValueViewModel.

        Matches TypeScript ConceptValueViewModel._create
        """
        from .static_types import StaticValue, StaticConcept
        from .rdm import RDM

        nodeid = node.nodeid
        collection_id = node.config.get('rdmCollection') if node.config else None
        if not collection_id:
            raise ValueError(f"Node {node.alias} ({node.nodeid}) missing rdmCollection in config")

        val: Optional['StaticValue'] = value

        if tile:
            if nodeid not in tile.data:
                tile.data[nodeid] = None

            if value is not None:
                # Handle StaticConcept
                if isinstance(value, StaticConcept):
                    if hasattr(value, 'getPrefLabel'):
                        val = value.getPrefLabel()
                    else:
                        raise ValueError("Recognizing value as StaticConcept, but no getPrefLabel member")

                if not value:
                    val = None
                elif isinstance(value, StaticValue):
                    # Already correct type
                    pass
                elif hasattr(value, '__await__'):
                    # Handle promise
                    resolved = await value
                    return await ConceptValueViewModel._create(tile, node, resolved, cacheEntry)
                elif isinstance(value, str):
                    # UUID string - need to look up in RDM
                    import re
                    if re.match(r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$', value, re.I):
                        if cacheEntry:
                            # Use cache entry
                            val = StaticValue(
                                id=cacheEntry.id,
                                value=cacheEntry.value,
                                __concept=None,
                                __conceptId=cacheEntry.conceptId
                            )
                            return ConceptValueViewModel(val, collection_id)
                        else:
                            # Look up in RDM
                            collection = await RDM.retrieveCollection(collection_id)
                            if not hasattr(collection, 'getConceptValue'):
                                raise ValueError(f"Collection {collection.id} must be a StaticCollection, not key/value object")

                            val = collection.getConceptValue(value)
                            if not val:
                                print(f"ERROR: Could not find concept for value {value} in collection {collection_id} for {node.alias}")

                            tile.data[nodeid] = val.id if val else None

                            if not tile or not val:
                                return None
                            return ConceptValueViewModel(val, collection_id)
                    else:
                        raise ValueError(f"Set concepts using values from collections, not strings: {value}")
                else:
                    raise ValueError("Could not set concept from this data")

                if not hasattr(val, '__await__'):
                    if not val:
                        print(f"ERROR: Could not find concept for value {value} for {node.alias} in collection {collection_id}")
                    tile.data[nodeid] = val.id if val else None

        if not tile or not val:
            return None

        return ConceptValueViewModel(val, collection_id)

    async def __asTileData(self) -> Optional[str]:
        """Convert to tile data (value ID)"""
        value = await self._value if hasattr(self._value, '__await__') else self._value
        return value.id if value else None


class DomainValueViewModel(str):
    """
    ViewModel for domain values from graph config.

    Matches TypeScript DomainValueViewModel extends String.
    """

    def __new__(cls, value: 'StaticDomainValue'):
        return str.__new__(cls, str(value))

    def __init__(self, value: 'StaticDomainValue'):
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['PseudoValue'] = None
        self._value: Union['StaticDomainValue', Awaitable['StaticDomainValue']] = value

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    async def for_json(self) -> 'StaticDomainValue':
        """Convert to JSON (StaticDomainValue)"""
        return await self._value if hasattr(self._value, '__await__') else self._value

    def __forJsonCache(self) -> None:
        """Domain values are on the graph, no caching needed"""
        return None

    def getValue(self) -> Union['StaticDomainValue', Awaitable['StaticDomainValue']]:
        """Get the underlying StaticDomainValue"""
        return self._value

    async def lang(self, lang: str) -> Optional[str]:
        """Get value in specific language"""
        value = await self._value if hasattr(self._value, '__await__') else self._value
        return value.lang(lang)

    @staticmethod
    async def _create(
        tile: Optional['StaticTile'],
        node: 'StaticNode',
        value: Any,
    ) -> Optional['DomainValueViewModel']:
        """
        Create a DomainValueViewModel.

        Matches TypeScript DomainValueViewModel._create
        """
        from .static_types import StaticDomainValue
        from .node_config import nodeConfigManager, StaticNodeConfigDomain

        nodeid = node.nodeid
        val: Optional['StaticDomainValue'] = value

        if tile:
            if nodeid not in tile.data:
                tile.data[nodeid] = None

            if value is not None:
                if not value and not isinstance(value, StaticDomainValue):
                    val = None
                elif isinstance(value, StaticDomainValue):
                    # Already a StaticDomainValue
                    val = value
                elif hasattr(value, '__await__'):
                    # Handle promise
                    resolved = await value
                    return await DomainValueViewModel._create(tile, node, resolved)
                elif isinstance(value, str):
                    # UUID string - need to look up in domain config
                    import re
                    if re.match(r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-5][0-9a-f]{3}-[089ab][0-9a-f]{3}-[0-9a-f]{12}$', value, re.I):
                        config = nodeConfigManager.retrieve(node)
                        if not config or not isinstance(config, StaticNodeConfigDomain):
                            raise ValueError(f"Cannot form domain value for {node.nodeid} without config")
                        val = config.value_from_id(value)
                    else:
                        raise ValueError("Set domain values using values from domain lists, not strings")
                else:
                    raise ValueError("Could not set domain value from this data")

                if not hasattr(val, '__await__'):
                    tile.data[nodeid] = val.id if val else None

        if not tile or not val:
            return None

        return DomainValueViewModel(val)

    async def __asTileData(self) -> Optional[str]:
        """Convert to tile data (value ID)"""
        value = await self._value if hasattr(self._value, '__await__') else self._value
        return value.id if value else None


class ResourceInstanceViewModel:
    """
    ViewModel for resource instance references with lazy loading.

    Matches TypeScript ResourceInstanceViewModel<RIVM>.
    """

    def __init__(
        self,
        id: str,
        modelWrapper: Optional[Any] = None,
        instanceWrapperFactory: Optional[Callable[[Any], Any]] = None,
        cacheEntry: Optional[ResourceInstanceCacheEntry] = None
    ):
        self.id = id
        self._i: Optional[Any] = instanceWrapperFactory(self) if instanceWrapperFactory else None
        self.__: Optional[Any] = modelWrapper
        self.__cacheEntry: Optional[ResourceInstanceCacheEntry] = None
        self.__parentPseudo: Optional['IPseudo'] = None
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.then = None  # Not thenable
        self.gm: Optional[Any] = None

        if isinstance(cacheEntry, ResourceInstanceCacheEntry):
            self.__cacheEntry = cacheEntry

    def __str__(self) -> str:
        if not self.__:
            return f"[Resource:{self.id}]"
        return f"[{self.__.wkrm.modelClassName}:{self.id or '-'}]"

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    async def __has(self, key: str) -> Optional[bool]:
        """Check if key exists (lazy-load, so returns Optional[bool])"""
        if not self._i:
            return None
        root = await self._i.getRootViewModel()
        return await root.__has(key) if root else None

    def __getattr__(self, key: str):
        """
        Lazy-load attributes from the underlying resource.

        Matches TypeScript ResourceInstanceViewModel Proxy get handler.

        Returns an AttrPromise that will:
        1. Retrieve the resource if not already loaded
        2. Get the root view model
        3. Access the requested attribute

        This enables chained access like:
            await group.member[0].names.name
        """
        # Don't intercept known attributes and private/dunder attributes
        if key in ('id', '__', '_i', 'gm', 'then', '__cacheEntry', '__parentPseudo', '_'):
            return object.__getattribute__(self, key)

        if key.startswith('_ResourceInstanceViewModel__') or key.startswith('__'):
            return object.__getattribute__(self, key)

        async def load_and_get_attr():
            # Retrieve resource if not loaded
            if not self._i:
                await self.retrieve()
                if not self._i:
                    raise ValueError("Could not retrieve resource")

            # Get the root view model (SemanticViewModel)
            root = await self._i.getRootViewModel()
            if not root:
                return None

            # Access the attribute on the root
            return getattr(root, key)

        # Return AttrPromise for chaining
        return AttrPromise(load_and_get_attr())

    async def __asTileData(self) -> Dict[str, str]:
        """Convert to tile data format"""
        return {'resourceId': self.id}

    async def __forJsonCache(self, getMeta: GetMeta) -> ResourceInstanceCacheEntry:
        """Build cache entry"""
        if not self.__:
            if self.__cacheEntry:
                return self.__cacheEntry
            else:
                _, wrapper = await self.retrieve()
        else:
            wrapper = self.__

        meta = await getMeta(self) if getMeta else None
        self.__cacheEntry = ResourceInstanceCacheEntry(
            id=self.id,
            type=wrapper.wkrm.modelClassName,
            graphId=wrapper.wkrm.graphId,
            title=None,
            meta=meta
        )
        return self.__cacheEntry

    async def for_json(self, cascade: bool = False) -> Any:
        """
        Convert to JSON (StaticResourceReference).

        Matches TypeScript ResourceInstanceViewModel.forJson
        """
        from .static_types import StaticResourceReference

        if not cascade and self.__cacheEntry:
            jsonData = {
                'type': self.__cacheEntry.type,
                'graphId': self.__cacheEntry.graphId,
                'id': self.__cacheEntry.id,
                'title': self.__cacheEntry.title,
                'meta': self.__cacheEntry.meta,
                'root': None
            }
        elif self.__:
            jsonData = {
                'type': self.__.wkrm.modelClassName,
                'graphId': self.__.wkrm.graphId,
                'id': self.id,
                'title': None,
                'meta': None,
                'root': None
            }
        else:
            jsonData = {
                'type': "(unknown)",
                'graphId': "",
                'id': self.id,
                'title': None,
                'meta': None,
                'root': None
            }

        basic = StaticResourceReference(jsonData)

        if cascade:
            if not self._i:
                await self.retrieve()
                if not self._i:
                    raise ValueError("Could not retrieve resource")

            root = await self._i.getRootViewModel()
            basic.root = await root.for_json()

        return basic

    async def retrieve(self) -> tuple[Any, Any]:
        """
        Retrieve the full resource from GraphManager.

        Returns (instanceWrapper, modelWrapper)
        """
        # Import here to avoid circular dependency
        from .view_models import viewContext

        if not viewContext.graphManager:
            raise ValueError("Cannot traverse resource relationships without a GraphManager")

        replacement = await viewContext.graphManager.getResource(self.id, True)

        # Extract wrappers from replacement resource
        iw = replacement._i
        mw = replacement.__

        self._i = iw
        self.__ = mw
        return (iw, mw)

    @staticmethod
    async def _create(
        tile: Optional['StaticTile'],
        node: 'StaticNode',
        value: Any,
        cacheEntry: Optional[ResourceInstanceCacheEntry]
    ) -> Optional['ResourceInstanceViewModel']:
        """
        Create a ResourceInstanceViewModel.

        Matches TypeScript ResourceInstanceViewModel._create
        """
        from .static_types import StaticResource, StaticResourceReference

        nodeid = node.nodeid
        val: Optional[str] = value

        if tile:
            if nodeid not in tile.data:
                tile.data[nodeid] = None

            if value is not None:
                if not value and not isinstance(value, (StaticResource, StaticResourceReference)):
                    val = None
                elif hasattr(value, '__await__'):
                    # Handle promise
                    resolved = await value
                    return await ResourceInstanceViewModel._create(tile, node, resolved, cacheEntry)
                elif isinstance(value, str):
                    # UUID string
                    import re
                    if re.match(r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$', value, re.I):
                        val = value
                    else:
                        raise ValueError(f"Set resource instances using ID, not strings in node {node.alias}: {value}")
                elif isinstance(value, dict) and 'resourceId' in value:
                    val = value['resourceId']
                elif isinstance(value, list) and len(value) < 2:
                    if len(value) == 1:
                        return await ResourceInstanceViewModel._create(tile, node, value[0], cacheEntry)
                else:
                    raise ValueError("Could not set resource instance from this data")

                tile.data[nodeid] = [{'resourceId': val}] if val else None

        if not tile or not val:
            return None

        return ResourceInstanceViewModel(val, None, None, cacheEntry)


class DateViewModel(datetime):
    """
    ViewModel for date values.

    Matches TypeScript DateViewModel extends Date.
    """

    def __new__(cls, val: str):
        try:
            dt = datetime.fromisoformat(val.replace('Z', '+00:00'))
        except:
            dt = datetime.now()
        return datetime.__new__(
            cls,
            dt.year, dt.month, dt.day,
            dt.hour, dt.minute, dt.second, dt.microsecond,
            dt.tzinfo
        )

    def __init__(self, val: str):
        self.__original = val
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['PseudoValue'] = None
        self.then = None  # Not thenable

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    def __forJsonCache(self) -> None:
        return None

    async def for_json(self) -> str:
        """Convert to ISO string"""
        try:
            return self.isoformat()
        except Exception as e:
            print(f"Warning: {e}")
            return self.__original

    def __asTileData(self) -> str:
        """Convert to ISO string for tile data"""
        return self.isoformat()

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
    ) -> Optional['DateViewModel']:
        """
        Create a DateViewModel.

        Matches TypeScript DateViewModel._create
        """
        nodeid = node.nodeid

        if hasattr(value, '__await__'):
            async def handle_promise():
                resolved = await value
                return DateViewModel._create(tile, node, resolved)
            return handle_promise()

        if tile:
            if nodeid not in tile.data:
                tile.data[nodeid] = None
            if value is not None:
                tile.data[nodeid] = value

        val = tile.data.get(nodeid)

        # Handle rendering issues (workaround for objects with 'en' key)
        if isinstance(val, dict) and 'en' in val:
            val = val['en']

        if not tile or val is None or val == '':
            return None

        if not isinstance(val, str):
            raise ValueError("Date should be a string")

        return DateViewModel(val)


class GeoJSONViewModel:
    """
    ViewModel for GeoJSON geometries.

    Matches TypeScript GeoJSONViewModel.
    """

    def __init__(self, jsonData: Dict[str, Any]):
        self._value: Dict[str, Any] = jsonData
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['PseudoValue'] = None
        self.then = None  # Not thenable

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    def __forJsonCache(self) -> None:
        return None

    def __getitem__(self, key: str) -> Any:
        """Dict-like access"""
        return self._value[key]

    def __setitem__(self, key: str, value: Any):
        """Dict-like access"""
        self._value[key] = value

    async def for_json(self) -> Dict[str, Any]:
        """Convert to JSON"""
        return self._value

    def __asTileData(self) -> Dict[str, Any]:
        """Convert to tile data"""
        return self._value

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
    ) -> Optional['GeoJSONViewModel']:
        """
        Create a GeoJSONViewModel.

        Matches TypeScript GeoJSONViewModel._create
        """
        nodeid = node.nodeid

        if hasattr(value, '__await__'):
            async def handle_promise():
                resolved = await value
                return GeoJSONViewModel._create(tile, node, resolved)
            return handle_promise()

        if tile:
            if nodeid not in tile.data:
                tile.data[nodeid] = None
            if value is not None:
                tile.data[nodeid] = value

        val = tile.data.get(nodeid)
        if not tile or val is None:
            return None

        if not isinstance(val, dict):
            raise ValueError("GeoJSON should be a JSON object")

        return GeoJSONViewModel(val)


class EDTFViewModel(str):
    """
    ViewModel for EDTF (Extended Date/Time Format) values.

    Matches TypeScript EDTFViewModel extends String.
    """

    def __new__(cls, val: str):
        return str.__new__(cls, val)

    def __init__(self, val: str):
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['PseudoValue'] = None

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    def __forJsonCache(self) -> None:
        return None

    async def for_json(self) -> str:
        """Convert to JSON string"""
        return str(self)

    def __asTileData(self) -> str:
        """Convert to tile data"""
        return str(self)

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
    ) -> Optional['EDTFViewModel']:
        """
        Create an EDTFViewModel.

        Matches TypeScript EDTFViewModel._create
        """
        nodeid = node.nodeid

        if hasattr(value, '__await__'):
            async def handle_promise():
                resolved = await value
                return EDTFViewModel._create(tile, node, resolved)
            return handle_promise()

        if tile:
            if value is not None:
                tile.data[nodeid] = value

        val = tile.data.get(nodeid)
        if not tile or val is None:
            return None

        return EDTFViewModel(val)


class NonLocalizedStringViewModel(str):
    """
    ViewModel for non-localized string values.

    Matches TypeScript NonLocalizedStringViewModel extends String.
    """

    def __new__(cls, val: str):
        return str.__new__(cls, val)

    def __init__(self, val: str):
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['PseudoValue'] = None

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    def __forJsonCache(self) -> None:
        return None

    async def for_json(self) -> str:
        """Convert to JSON string"""
        return str(self)

    def __asTileData(self) -> str:
        """Convert to tile data"""
        return str(self)

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
    ) -> Optional['NonLocalizedStringViewModel']:
        """
        Create a NonLocalizedStringViewModel.

        Matches TypeScript NonLocalizedStringViewModel._create
        """
        nodeid = node.nodeid

        if hasattr(value, '__await__'):
            async def handle_promise():
                resolved = await value
                return NonLocalizedStringViewModel._create(tile, node, resolved)
            return handle_promise()

        if tile:
            if value is not None:
                tile.data[nodeid] = value

        val = tile.data.get(nodeid)
        if not tile or val is None:
            return None

        return NonLocalizedStringViewModel(val)


class NumberViewModel(float):
    """
    ViewModel for numeric values.

    Matches TypeScript NumberViewModel extends Number.
    """

    def __new__(cls, val: Union[int, float]):
        return float.__new__(cls, val)

    def __init__(self, val: Union[int, float]):
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['PseudoValue'] = None

    def __str__(self) -> str:
        return str(float(self))

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    def __forJsonCache(self) -> None:
        return None

    async def for_json(self) -> float:
        """Convert to JSON number"""
        return float(self)

    def __asTileData(self) -> float:
        """Convert to tile data"""
        return float(self)

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
    ) -> Optional['NumberViewModel']:
        """
        Create a NumberViewModel.

        Matches TypeScript NumberViewModel._create
        """
        nodeid = node.nodeid

        if hasattr(value, '__await__'):
            async def handle_promise():
                resolved = await value
                return NumberViewModel._create(tile, node, resolved)
            return handle_promise()

        if tile:
            if value is not None:
                tile.data[nodeid] = value

        val = tile.data.get(nodeid)
        if not tile or val is None:
            return None

        return NumberViewModel(val)


class BooleanViewModel:
    """
    ViewModel for boolean values with configurable labels.

    Matches TypeScript BooleanViewModel extends Boolean.
    Note: This is a Boolean object, not a primitive boolean.
    """

    def __init__(self, value: bool, config: 'StaticNodeConfigBoolean'):
        self._value = bool(value)
        self.__config = config
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['PseudoValue'] = None

    def __bool__(self) -> bool:
        return self._value

    def __str__(self) -> str:
        return self.toString()

    def toString(self, lang: Optional[str] = None) -> str:
        """Get string representation with language-specific labels"""
        labelLang = lang or DEFAULT_LANGUAGE
        if self._value:
            return (
                self.__config.trueLabel[labelLang] if self.__config and self.__config.trueLabel else 'true'
            ) if self.__config else 'true'
        else:
            return (
                self.__config.falseLabel[labelLang] if self.__config and self.__config.falseLabel else 'false'
            ) if self.__config else 'false'

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    def __forJsonCache(self) -> None:
        return None

    async def for_json(self) -> bool:
        """Convert to JSON boolean"""
        return self._value

    def __asTileData(self) -> bool:
        """Convert to tile data"""
        return self._value

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
    ) -> Optional['BooleanViewModel']:
        """
        Create a BooleanViewModel.

        Matches TypeScript BooleanViewModel._create
        """
        from .node_config import nodeConfigManager, StaticNodeConfigBoolean

        nodeid = node.nodeid

        if hasattr(value, '__await__'):
            async def handle_promise():
                resolved = await value
                return BooleanViewModel._create(tile, node, resolved)
            return handle_promise()

        if tile:
            if value is not None:
                tile.data[nodeid] = value

        val = tile.data.get(nodeid)
        if not tile or val is None:
            return None

        config = nodeConfigManager.retrieve(node)
        if not config or not isinstance(config, StaticNodeConfigBoolean):
            raise ValueError(f"Cannot form boolean value for {node.nodeid} without config")

        if not isinstance(val, bool) and val not in (0, 1):
            raise ValueError(f"Refusing to use truthiness for value {val} in boolean")

        return BooleanViewModel(True if val else False, config)


@dataclass
class Url:
    """URL with optional label"""
    url: str
    url_label: Optional[str] = None


class UrlViewModel(str):
    """
    ViewModel for URL values with optional labels.

    Matches TypeScript UrlViewModel extends String.
    """

    def __new__(cls, value: Url):
        displayValue = value.url_label or value.url
        return str.__new__(cls, displayValue)

    def __init__(self, value: Url):
        self._value: Url = value
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['PseudoValue'] = None

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    def __forJsonCache(self) -> None:
        return None

    async def for_json(self) -> Dict[str, str]:
        """Convert to JSON object"""
        return {
            'url': self._value.url,
            'url_label': self._value.url_label or "",
        }

    def label(self) -> str:
        """Get the label"""
        return self._value.url_label or self._value.url

    def href(self) -> str:
        """Get the URL"""
        return self._value.url

    def __asTileData(self) -> Dict[str, str]:
        """Convert to tile data"""
        return self.for_json()

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
    ) -> Optional['UrlViewModel']:
        """
        Create a UrlViewModel.

        Matches TypeScript UrlViewModel._create
        """
        nodeid = node.nodeid

        if hasattr(value, '__await__'):
            async def handle_promise():
                resolved = await value
                return UrlViewModel._create(tile, node, resolved)
            return handle_promise()

        if tile:
            if nodeid not in tile.data:
                tile.data[nodeid] = {}

            if value is not None:
                if isinstance(value, UrlViewModel):
                    value = value._value
                elif isinstance(value, dict):
                    if 'url' not in value:
                        raise ValueError(f"A URL must be null or have a 'url' field: {value}")

                tile.data[nodeid] = {
                    'url': value.url if isinstance(value, Url) else value.get('url'),
                    'url_label': value.url_label if isinstance(value, Url) else value.get('url_label'),
                }

        val = tile.data.get(nodeid)
        if not tile or val is None:
            return None

        if not isinstance(val, dict):
            url = Url(url=str(val))
        elif isinstance(val, dict):
            if 'url' in val and isinstance(val['url'], str):
                url = Url(url=val['url'], url_label=val.get('url_label'))
            else:
                raise ValueError(f"Unrecognised URL type: {val}")
        else:
            raise ValueError(f"Unrecognised URL type: {val}")

        return UrlViewModel(url)


@dataclass
class StringTranslatedLanguage:
    """Single language value for translatable strings"""
    value: str = ""


class StringViewModel(str):
    """
    ViewModel for translatable string values.

    Matches TypeScript StringViewModel extends String.
    """

    def __new__(cls, value: Dict[str, Union[str, StringTranslatedLanguage]], language: Optional[str] = None):
        lang = language or DEFAULT_LANGUAGE
        lang_value = value.get(lang)

        if lang_value:
            if isinstance(lang_value, str):
                displayValue = lang_value
            else:
                displayValue = lang_value.value
        else:
            # Fallback to empty string
            displayValue = ""

        return str.__new__(cls, displayValue)

    def __init__(self, value: Dict[str, Union[str, StringTranslatedLanguage]], language: Optional[str] = None):
        self._value: Dict[str, Union[str, StringTranslatedLanguage]] = value
        self._: Optional[Union[IViewModel, Awaitable[IViewModel]]] = None
        self.__parentPseudo: Optional['PseudoValue'] = None

    def describeField(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describeFieldGroup(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    def __forJsonCache(self) -> None:
        return None

    async def for_json(self) -> str:
        """Convert to JSON string (default language)"""
        return str(self)

    def lang(self, language: str) -> Optional[str]:
        """Get value in specific language"""
        elt = self._value.get(language)
        if elt:
            if isinstance(elt, dict) or hasattr(elt, 'value'):
                return elt.value if hasattr(elt, 'value') else elt['value']
            return elt
        return None

    def __asTileData(self) -> Dict[str, Union[str, StringTranslatedLanguage]]:
        """Convert to tile data"""
        return self._value

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
    ) -> Optional['StringViewModel']:
        """
        Create a StringViewModel.

        Matches TypeScript StringViewModel._create
        """
        nodeid = node.nodeid

        if hasattr(value, '__await__'):
            async def handle_promise():
                resolved = await value
                return StringViewModel._create(tile, node, resolved)
            return handle_promise()

        if tile:
            if nodeid not in tile.data:
                tile.data[nodeid] = {}

            if value is not None:
                if isinstance(value, dict):
                    # Set each language value
                    for k, v in value.items():
                        val_data = tile.data.get(nodeid)
                        if isinstance(val_data, dict):
                            val_data[k] = v
                        elif val_data is not None:
                            raise ValueError("Malformed string in tile data")
                else:
                    # Non-dict value, use default language
                    tile.data[nodeid] = {DEFAULT_LANGUAGE: value}

        val = tile.data.get(nodeid)
        if not tile or val is None:
            return None

        # Ensure dict format
        if isinstance(val, dict):
            mapVal = val
        else:
            mapVal = {}

        return StringViewModel(mapVal)


class NodeViewModel:
    """
    ViewModel for navigating node structure (not tile data).

    Matches TypeScript NodeViewModel.
    """

    def __init__(self, parentPseudo: 'PseudoNode', parentWkrm: Optional[Any]):
        self.__parentPseudo: 'PseudoNode' = parentPseudo
        self.__parentWkrm: Optional[Any] = parentWkrm
        self.then = None  # Not thenable

    def __forJsonCache(self) -> None:
        return None

    def __str__(self) -> str:
        if not self.__parentPseudo:
            return "[NodeViewModel]"
        alias = getattr(self.__parentPseudo, 'alias', None)
        return alias or "[unnamed]"

    async def __getEdgeTo(self, key: str) -> 'StaticEdge':
        """Get edge from parent node to child node by alias"""
        from .static_types import StaticEdge

        childNode = self.__parentPseudo.childNodes.get(key)
        if not childNode:
            raise ValueError(f"Child node key {key} missing")

        domainId = self.__parentPseudo.nodeid
        rangeId = childNode.nodeid
        edges = [
            edge for edge in self.__parentWkrm.graph.edges
            if edge.domainnode_id == domainId and edge.rangenode_id == rangeId
        ]

        if len(edges) != 1:
            raise ValueError(f"Number of edges from {domainId}->{rangeId} != 1")

        return edges[0]

    async def __get(self, key: str) -> 'NodeViewModel':
        """Get child node by alias"""
        pseudo = self.__parentWkrm.createPseudoNodeChild(key, self.__parentPseudo)
        return await NodeViewModel._create(pseudo, self.__parentWkrm)

    def __getattribute__(self, key: str) -> Any:
        """Proxy attribute access"""
        # Internal attributes
        if key.startswith('_NodeViewModel__') or key.startswith('_') or key in ('then',):
            return object.__getattribute__(self, key)

        # Special handling
        if key == '_':
            return self.__parentPseudo.node
        elif key.endswith('$edge'):
            # Get edge to child
            alias = key[:-5]  # Remove '$edge' suffix
            import asyncio
            return asyncio.create_task(self.__getEdgeTo(alias))
        elif key == 'length':
            return self.__parentPseudo.size

        # Get child node
        import asyncio
        return asyncio.create_task(self.__get(key))

    @staticmethod
    async def _create(
        pseudo: 'PseudoNode',
        parent: Optional[Any],
    ) -> 'NodeViewModel':
        """
        Create a NodeViewModel.

        Matches TypeScript NodeViewModel._create
        """
        return NodeViewModel(pseudo, parent)


# ============================================================================
# View Context
# ============================================================================

class ViewContext:
    """Global context for ViewModels (mainly for GraphManager)"""

    def __init__(self):
        self.graphManager: Optional[Any] = None


viewContext = ViewContext()
view_context = viewContext  # snake_case alias


# ============================================================================
# Custom Datatypes Registry
# ============================================================================

CUSTOM_DATATYPES: Dict[str, Union[str, type]] = {}


# ============================================================================
# ViewModel Factory
# ============================================================================

async def get_view_model(
    parentPseudo: 'PseudoValue',
    tile: 'StaticTile',
    node: 'StaticNode',
    data: Any,
    parent: Optional[Any] = None,
    isInner: bool = False
) -> Optional[IViewModel]:
    """
    Factory function to create appropriate ViewModel based on datatype.

    Matches TypeScript getViewModel().
    """
    # Get cache entries if available
    cacheEntries: Optional[Dict[str, Dict[str, Any]]] = None
    if parentPseudo.parent and parentPseudo.parent._i:
        cacheEntries = await parentPseudo.parent._i.getValueCache(False, None)

    cacheEntry: Optional[Any] = None
    if cacheEntries and tile.tileid:
        cacheEntry = cacheEntries.get(tile.tileid, {}).get(node.nodeid)

    datatype = "semantic" if isInner else CUSTOM_DATATYPES.get(node.datatype, node.datatype)

    vm = None

    # Handle custom datatypes (classes with _create method)
    if not isinstance(datatype, str):
        vm = await datatype._create(tile, node, data, cacheEntry)
    else:
        # Built-in datatypes
        if datatype == "semantic":
            # SemanticViewModel imported at top of module
            vm = await SemanticViewModel._create(tile, node, data, parent)
        elif datatype == "domain-value":
            vm = await DomainValueViewModel._create(tile, node, data)
        elif datatype == "domain-value-list":
            vm = await DomainValueListViewModel._create(tile, node, data)
        elif datatype == "concept":
            # Ensure cache entry is correct type
            conceptValueCacheEntry = None
            if cacheEntry and isinstance(cacheEntry, dict) and not isinstance(cacheEntry, ConceptValueCacheEntry):
                conceptValueCacheEntry = ConceptValueCacheEntry(**cacheEntry)
            elif isinstance(cacheEntry, ConceptValueCacheEntry):
                conceptValueCacheEntry = cacheEntry
            vm = await ConceptValueViewModel._create(tile, node, data, conceptValueCacheEntry)
        elif datatype == "resource-instance":
            # Ensure cache entry is correct type
            resourceInstanceCacheEntry = None
            if cacheEntry and isinstance(cacheEntry, dict) and not isinstance(cacheEntry, ResourceInstanceCacheEntry):
                resourceInstanceCacheEntry = ResourceInstanceCacheEntry(**cacheEntry)
            elif isinstance(cacheEntry, ResourceInstanceCacheEntry):
                resourceInstanceCacheEntry = cacheEntry
            vm = await ResourceInstanceViewModel._create(tile, node, data, resourceInstanceCacheEntry)
        elif datatype == "resource-instance-list":
            # Ensure cache entry is correct type
            resourceInstanceListCacheEntry = None
            if cacheEntry and isinstance(cacheEntry, dict) and not isinstance(cacheEntry, ResourceInstanceListCacheEntry):
                resourceInstanceListCacheEntry = ResourceInstanceListCacheEntry(**cacheEntry)
            elif isinstance(cacheEntry, ResourceInstanceListCacheEntry):
                resourceInstanceListCacheEntry = cacheEntry
            vm = await ResourceInstanceListViewModel._create(tile, node, data, resourceInstanceListCacheEntry)
        elif datatype == "concept-list":
            # Ensure cache entry is correct type
            conceptCacheEntry = None
            if cacheEntry and isinstance(cacheEntry, dict) and not isinstance(cacheEntry, ConceptListCacheEntry):
                conceptCacheEntry = ConceptListCacheEntry(**cacheEntry)
            elif isinstance(cacheEntry, ConceptListCacheEntry):
                conceptCacheEntry = cacheEntry
            vm = await ConceptListViewModel._create(tile, node, data, conceptCacheEntry)
        elif datatype == "date":
            vm = DateViewModel._create(tile, node, data)
        elif datatype == "geojson-feature-collection":
            vm = GeoJSONViewModel._create(tile, node, data)
        elif datatype == "boolean":
            vm = BooleanViewModel._create(tile, node, data)
        elif datatype == "string":
            vm = StringViewModel._create(tile, node, data)
        elif datatype == "number":
            vm = NumberViewModel._create(tile, node, data)
        elif datatype == "edtf":
            vm = EDTFViewModel._create(tile, node, data)
        elif datatype == "url":
            vm = UrlViewModel._create(tile, node, data)
        elif datatype == "non-localized-string":
            vm = NonLocalizedStringViewModel._create(tile, node, data)
        else:
            print(f"Warning: Missing type for tile {tile.tileid} on node {node.alias} with type {node.datatype}")
            vm = NonLocalizedStringViewModel._create(tile, node, data)

    # Handle promises from _create
    if hasattr(vm, '__await__'):
        vm = await vm

    if vm is None:
        return None

    # Set parent pseudo
    vm.__parentPseudo = parentPseudo

    # For list ViewModels, set parent pseudo on items
    if isinstance(vm, list):
        for vme in vm:
            if hasattr(vme, '__await__'):
                async def set_parent(item):
                    resolved = await item
                    if resolved is not None:
                        resolved.__parentPseudo = parentPseudo
                import asyncio
                asyncio.create_task(set_parent(vme))
            else:
                vme.__parentPseudo = parentPseudo

    return vm
