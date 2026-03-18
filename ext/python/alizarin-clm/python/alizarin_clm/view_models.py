"""
View Models for CLM (Controlled List Manager) references.

These ViewModels match the TypeScript ReferenceValueViewModel and ReferenceListViewModel.
"""

from __future__ import annotations

import json
import re
from typing import Any, Awaitable, Dict, List, Optional, TYPE_CHECKING, Union

if TYPE_CHECKING:
    from alizarin import StaticTile, StaticNode
    from alizarin.view_models import IViewModel
    from alizarin.pseudos import IPseudo

from .static_types import StaticReference, StaticReferenceLabel


# UUID regex pattern
UUID_PATTERN = re.compile(
    r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',
    re.IGNORECASE
)


class ReferenceValueViewModel(str):
    """
    ViewModel for a single reference value.

    Extends str to display the reference label.
    Matches TypeScript ReferenceValueViewModel extends String.
    """

    def __new__(cls, reference: StaticReference, collection_id: Optional[str] = None):
        return str.__new__(cls, reference.to_display_string())

    def __init__(self, reference: StaticReference, collection_id: Optional[str] = None):
        self._ref: StaticReference = reference
        self._: Optional[Union['IViewModel', Awaitable['IViewModel']]] = None
        self.__parentPseudo: Optional['IPseudo'] = None
        self._collection_id: Optional[str] = collection_id

    def describe_field(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describe_field_group(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    async def for_json(self) -> Dict[str, Any]:
        """Convert to JSON (StaticReference dict)."""
        return self._ref.to_dict()

    def get_value(self) -> StaticReference:
        """Get the underlying StaticReference."""
        return self._ref

    def lang(self, language: str) -> Optional[str]:
        """Get label in specific language."""
        return self._ref.lang(language)

    async def parent(self) -> Optional['ReferenceValueViewModel']:
        """
        Get the parent reference value, if this reference has a parent in the hierarchy.

        Returns a new ReferenceValueViewModel for the parent, or None if no parent.
        Raises RuntimeError if the collection doesn't support hierarchy lookups.
        """
        from alizarin.rdm import RDM

        if not self._ref or not self._collection_id:
            return None

        # Get the concept ID from the first label
        if not self._ref.labels:
            return None
        concept_id = self._ref.labels[0].list_item_id
        if not concept_id:
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

        # Get the parent reference - need to construct it from the concept
        parent_concept = collection.__allConcepts.get(parent_id)
        if not parent_concept:
            return None

        # Convert concept to StaticReference format
        labels = []
        for lang_id, pref_label in (parent_concept.prefLabels or {}).items():
            labels.append(StaticReferenceLabel(
                id=pref_label.get('id', ''),
                language_id=lang_id,
                list_item_id=parent_concept.id,
                value=pref_label.get('value', '') if isinstance(pref_label, dict) else str(pref_label),
                valuetype_id='prefLabel'
            ))

        if not labels:
            return None

        parent_ref = StaticReference(
            labels=labels,
            list_id=self._collection_id,
            uri=getattr(parent_concept, 'source', '') or f"http://localhost:8000/plugins/controlled-list-manager/item/{parent_id}"
        )

        return ReferenceValueViewModel(parent_ref, self._collection_id)

    async def ancestors(self) -> List['ReferenceValueViewModel']:
        """
        Get all ancestor reference values, from immediate parent to root.

        Returns a list of ReferenceValueViewModels for ancestors.
        """
        result: List['ReferenceValueViewModel'] = []
        current: Optional['ReferenceValueViewModel'] = self

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
        cache_entry: Optional[Dict[str, Any]] = None,
    ) -> Optional['ReferenceValueViewModel']:
        """
        Create a ReferenceValueViewModel.

        Matches TypeScript ReferenceValueViewModel.__create
        """
        # RDM is optional - only needed for UUID string lookups
        RDM = None
        try:
            from alizarin.rdm import RDM as _RDM
            RDM = _RDM
        except ImportError:
            pass

        nodeid = node.nodeid
        config = node.config or {}
        collection_id = config.get('controlledList') or config.get('rdmCollection')

        if not collection_id:
            raise ValueError(
                f"Node {node.alias} ({node.nodeid}) missing controlledList or rdmCollection in config"
            )

        val: Optional[StaticReference] = None

        if tile:
            if nodeid not in tile.data:
                tile.data[nodeid] = None

            if value is not None:
                # Handle Promise/awaitable
                if hasattr(value, '__await__'):
                    resolved = await value
                    return await ReferenceValueViewModel._create(tile, node, resolved, cache_entry)

                # Handle UUID string - needs RDM lookup
                if isinstance(value, str):
                    if UUID_PATTERN.match(value):
                        if RDM is None:
                            raise ValueError(
                                f"Cannot resolve UUID {value} - RDM module not available. "
                                f"Pass pre-resolved reference objects instead."
                            )
                        collection = await RDM.retrieveCollection(collection_id)
                        if not hasattr(collection, 'getReferenceValue'):
                            raise ValueError(
                                f"Collection {collection.id} must be a StaticCollection, not a key/value object"
                            )

                        ref_value = collection.getReferenceValue(value)
                        if not ref_value:
                            print(
                                f"ERROR: Could not find reference for value {value} "
                                f"for {node.alias} in collection {collection_id}"
                            )

                        tile.data[nodeid] = ref_value.to_dict() if ref_value else None

                        if not tile or not ref_value:
                            return None

                        val = ref_value
                    else:
                        raise ValueError(
                            f"Set references using values from collections, not strings: {value}"
                        )

                # Handle array with single element
                elif isinstance(value, list) and len(value) > 0:
                    if isinstance(value[0], dict) and 'labels' in value[0]:
                        # Array of pre-formatted reference values - use first
                        val = StaticReference.from_dict(value[0])
                        tile.data[nodeid] = val.to_dict()
                    else:
                        # Single-element array
                        return await ReferenceValueViewModel._create(tile, node, value[0], cache_entry)

                # Handle pre-formatted reference object
                elif isinstance(value, dict) and 'labels' in value:
                    val = StaticReference.from_dict(value)
                    tile.data[nodeid] = val.to_dict()

                # Handle StaticReference directly
                elif isinstance(value, StaticReference):
                    val = value
                    tile.data[nodeid] = val.to_dict()

                else:
                    raise ValueError(f"Could not set reference from this data: {json.dumps(value)}")

        if not tile or not val:
            return None

        return ReferenceValueViewModel(val, collection_id)

    async def __asTileData(self) -> Optional[Dict[str, Any]]:
        """Convert to tile data format."""
        return self._ref.to_dict() if self._ref else None


class ReferenceListViewModel(list):
    """
    ViewModel for a list of reference values.

    Extends list to hold ReferenceValueViewModels.
    Matches TypeScript ReferenceListViewModel extends Array.
    """

    def __init__(self, *items):
        super().__init__(items)
        self._: Optional[Union['IViewModel', Awaitable['IViewModel']]] = None
        self.__parentPseudo: Optional['IPseudo'] = None
        self._value: Optional[Awaitable[List[Optional[ReferenceValueViewModel]]]] = None

    def describe_field(self) -> Optional[Any]:
        return self.__parentPseudo.describeField() if self.__parentPseudo else None

    def describe_field_group(self) -> Optional[Any]:
        return self.__parentPseudo.describeFieldGroup() if self.__parentPseudo else None

    async def for_json(self) -> Optional[List[Dict[str, Any]]]:
        """Convert to JSON array."""
        result = []
        for item in self:
            if hasattr(item, '__await__'):
                item = await item
            if item:
                result.append(await item.for_json())
        return result if result else None

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
        cache_entry: Optional[Dict[str, Any]] = None,
    ) -> 'ReferenceListViewModel':
        """
        Create a ReferenceListViewModel.

        Matches TypeScript ReferenceListViewModel.__create
        """
        nodeid = node.nodeid
        val: List[Union[ReferenceValueViewModel, Awaitable[Optional[ReferenceValueViewModel]]]] = []

        if nodeid not in tile.data:
            tile.data[nodeid] = None

        if value is not None:
            tile.data[nodeid] = []

            if not isinstance(value, list):
                raise ValueError(
                    f"Cannot set reference list value on node {nodeid} except via array: {json.dumps(value)}"
                )

            # Create ReferenceValueViewModel for each item
            val = []
            for item in value:
                if isinstance(item, ReferenceValueViewModel):
                    val.append(item)
                else:
                    # Create and await immediately
                    vm = await ReferenceValueViewModel._create(tile, node, item, cache_entry)
                    val.append(vm)

            # Update tile data with resolved values
            refs = []
            for v in val:
                if v:
                    refs.append(v.get_value().to_dict())
            tile.data[nodeid] = refs

            value = val
        else:
            value = []

        return ReferenceListViewModel(*value)

    async def __asTileData(self) -> Optional[List[Dict[str, Any]]]:
        """Convert to tile data format."""
        return await self.for_json()


class ReferenceMergedDataType:
    """
    Factory that creates either ReferenceValueViewModel or ReferenceListViewModel
    based on the node config's multiValue setting.

    Matches TypeScript ReferenceMergedDataType.
    """

    @staticmethod
    async def _create(
        tile: 'StaticTile',
        node: 'StaticNode',
        value: Any,
        cache_entry: Optional[Dict[str, Any]] = None,
    ) -> Optional[Union[ReferenceValueViewModel, ReferenceListViewModel]]:
        """
        Create appropriate ViewModel based on config.

        If config.multiValue is true, creates ReferenceListViewModel.
        Otherwise creates ReferenceValueViewModel.
        """
        from alizarin.node_config import nodeConfigManager

        config = nodeConfigManager.retrieve(node)

        # Check if multiValue is set in config
        multi_value = False
        if config and hasattr(config, 'multi_value'):
            multi_value = config.multi_value
        elif node.config and node.config.get('multiValue'):
            multi_value = True

        if multi_value:
            return await ReferenceListViewModel._create(tile, node, value, cache_entry)
        return await ReferenceValueViewModel._create(tile, node, value, cache_entry)


__all__ = [
    "ReferenceValueViewModel",
    "ReferenceListViewModel",
    "ReferenceMergedDataType",
]
