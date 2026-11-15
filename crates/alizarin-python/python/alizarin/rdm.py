"""
Reference data for the ORM's concept view models: collections by id, as
TypeScript's ReferenceDataManager (js/rdm.ts).

A collection added here (``RDM.add_collection``) is used as it is. Any other
id resolves through core's global RDM cache, where Python loads collections
(``set_global_rdm_cache``, the ``add_*_to_global_cache`` functions), so a
concept reads with the labels core serialises it with.
"""

from __future__ import annotations

from typing import Any, Dict, Optional

from .alizarin import (
    global_rdm_concept_pref_label,
    global_rdm_get_parent_id,
    global_rdm_has_collection,
    global_rdm_lookup_value,
)
from .static_types import StaticValue


def _static_value(found: Optional[Dict[str, Any]]) -> Optional[StaticValue]:
    if not found:
        return None
    return StaticValue(id=found["id"], value=found["value"], _concept_id=found.get("conceptId"))


class CoreCollection:
    """A collection in core's global RDM cache, looked up there in place."""

    def __init__(self, collection_id: str):
        self.id = collection_id

    def get_concept_value(self, value_id: str) -> Optional[StaticValue]:
        """The value `value_id`, with its concept's id, or None."""
        return _static_value(global_rdm_lookup_value(self.id, value_id))

    def get_parent_id(self, concept_id: str) -> Optional[str]:
        """The parent of concept `concept_id`, or None at the top."""
        return global_rdm_get_parent_id(self.id, concept_id)

    def get_pref_label(self, concept_id: str, language: str = "en") -> Optional[StaticValue]:
        """Concept `concept_id`'s preferred label in `language` (else its first)."""
        return _static_value(global_rdm_concept_pref_label(self.id, concept_id, language))


class ReferenceDataManager:
    """Collections by id: those added here, else core's global RDM cache."""

    def __init__(self) -> None:
        self._collections: Dict[str, Any] = {}

    def add_collection(self, collection: Any) -> None:
        """Use `collection` (a StaticCollection) for its id."""
        self._collections[collection.id] = collection

    def has_collection(self, collection_id: str) -> bool:
        return collection_id in self._collections or global_rdm_has_collection(collection_id)

    def clear(self) -> None:
        """Forget the collections added here (core's cache is untouched)."""
        self._collections.clear()

    async def retrieve_collection(self, collection_id: str) -> Any:
        """The collection `collection_id`, as added here or in core's cache."""
        if collection_id in self._collections:
            return self._collections[collection_id]
        if global_rdm_has_collection(collection_id):
            return CoreCollection(collection_id)
        raise KeyError(
            f"Collection {collection_id} not found: add it with RDM.add_collection, "
            "or load it into core's RDM cache (set_global_rdm_cache)"
        )

    # TypeScript's names, as the view models were written with them.
    addCollection = add_collection
    hasCollection = has_collection
    retrieveCollection = retrieve_collection


RDM = ReferenceDataManager()
