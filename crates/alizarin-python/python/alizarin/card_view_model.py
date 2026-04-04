"""
CardViewModel for card-tree traversal.

Provides __getattr__-based navigation over the card/widget hierarchy
(instead of the node/edge hierarchy used by SemanticViewModel).

A CardViewModel wraps a serialized card dict and exposes:
- Widget values as attributes (by node alias)
- Child cards as nested CardViewModels (by slugified card name)

Cardinality-n cards expose a list of CardInstanceViewModels.
"""

from __future__ import annotations

import re
from typing import Any, Dict, List, Optional


def _slugify(name: str) -> str:
    """Convert a translatable name dict or string to a Python-friendly attribute name."""
    if isinstance(name, dict):
        # Pick 'en' or first available language
        name = name.get("en") or next(iter(name.values()), "")
    if not isinstance(name, str):
        return ""
    # lowercase, replace non-alnum with underscore, collapse runs
    slug = re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")
    return slug


class CardViewModel:
    """
    ViewModel for a single card in the card hierarchy.

    Provides attribute access to:
    - Widgets by node_alias (returns the serialized value)
    - Child cards by slugified card name (returns CardViewModel or list)
    """

    __slots__ = ("_card", "_widgets_by_alias", "_child_cards_by_slug", "_child_cards_by_id")

    def __init__(self, card_dict: Dict[str, Any]) -> None:
        self._card = card_dict

        # Index widgets by node_alias for O(1) lookup
        self._widgets_by_alias: Dict[str, Any] = {}
        for w in card_dict.get("widgets", []):
            alias = w.get("node_alias", "")
            if alias:
                self._widgets_by_alias[alias] = w.get("value")

        # Index child cards by slugified name
        self._child_cards_by_slug: Dict[str, Any] = {}
        self._child_cards_by_id: Dict[str, Any] = {}
        for child in card_dict.get("cards", []):
            child_vm = _wrap_card(child)
            slug = _slugify(child.get("name", ""))
            if slug:
                self._child_cards_by_slug[slug] = child_vm
            card_id = child.get("card_id", "")
            if card_id:
                self._child_cards_by_id[card_id] = child_vm

    @property
    def card_id(self) -> str:
        return self._card.get("card_id", "")

    @property
    def name(self) -> Any:
        return self._card.get("name")

    @property
    def tile_id(self) -> Optional[str]:
        return self._card.get("tile_id")

    @property
    def cardinality(self) -> str:
        return self._card.get("cardinality", "1")

    @property
    def visible(self) -> bool:
        return self._card.get("visible", True)

    @property
    def active(self) -> bool:
        return self._card.get("active", True)

    @property
    def widgets(self) -> Dict[str, Any]:
        """All widget values keyed by node alias."""
        return dict(self._widgets_by_alias)

    @property
    def cards(self) -> Dict[str, Any]:
        """Child cards keyed by slug."""
        return dict(self._child_cards_by_slug)

    def __getattr__(self, name: str) -> Any:
        if name.startswith("_"):
            raise AttributeError(f"'{type(self).__name__}' has no attribute '{name}'")

        # 1. Try widget by node_alias
        if name in self._widgets_by_alias:
            return self._widgets_by_alias[name]

        # 2. Try child card by slug
        if name in self._child_cards_by_slug:
            return self._child_cards_by_slug[name]

        # 3. Try child card by card_id
        if name in self._child_cards_by_id:
            return self._child_cards_by_id[name]

        available = sorted(set(
            list(self._widgets_by_alias.keys()) +
            list(self._child_cards_by_slug.keys())
        ))
        raise AttributeError(
            f"Card '{_slugify(self._card.get('name', ''))}' has no widget or child card '{name}'. "
            f"Available: {', '.join(available)}"
        )

    def __repr__(self) -> str:
        slug = _slugify(self._card.get("name", ""))
        return f"<CardViewModel '{slug}' widgets={list(self._widgets_by_alias.keys())}>"

    def to_dict(self) -> Dict[str, Any]:
        """Return the raw serialized card dict."""
        return self._card


class CardInstanceViewModel(CardViewModel):
    """A single instance within a cardinality-n card."""
    pass


class CardListViewModel:
    """
    ViewModel for a cardinality-n card (wraps multiple instances).

    Supports indexing and iteration.
    """

    __slots__ = ("_card", "_instances")

    def __init__(self, card_dict: Dict[str, Any]) -> None:
        self._card = card_dict
        self._instances: List[CardInstanceViewModel] = [
            CardInstanceViewModel(inst)
            for inst in card_dict.get("instances", [])
        ]

    @property
    def card_id(self) -> str:
        return self._card.get("card_id", "")

    @property
    def name(self) -> Any:
        return self._card.get("name")

    @property
    def cardinality(self) -> str:
        return "n"

    def __len__(self) -> int:
        return len(self._instances)

    def __iter__(self):
        return iter(self._instances)

    def __getitem__(self, index: int) -> CardInstanceViewModel:
        return self._instances[index]

    def __bool__(self) -> bool:
        return len(self._instances) > 0

    def __repr__(self) -> str:
        slug = _slugify(self._card.get("name", ""))
        return f"<CardListViewModel '{slug}' instances={len(self._instances)}>"

    def to_dict(self) -> Dict[str, Any]:
        return self._card


class RootCardViewModel:
    """
    Root of card-tree traversal. Wraps the list of root cards.

    Provides __getattr__ to access root cards by slugified name.
    """

    __slots__ = ("_cards_by_slug", "_cards_by_id", "_all_cards")

    def __init__(self, root_cards: List[Dict[str, Any]]) -> None:
        self._all_cards: List[Any] = []
        self._cards_by_slug: Dict[str, Any] = {}
        self._cards_by_id: Dict[str, Any] = {}

        for card_dict in root_cards:
            vm = _wrap_card(card_dict)
            self._all_cards.append(vm)
            slug = _slugify(card_dict.get("name", ""))
            if slug:
                self._cards_by_slug[slug] = vm
            card_id = card_dict.get("card_id", "")
            if card_id:
                self._cards_by_id[card_id] = vm

    def __getattr__(self, name: str) -> Any:
        if name.startswith("_"):
            raise AttributeError(f"'{type(self).__name__}' has no attribute '{name}'")

        if name in self._cards_by_slug:
            return self._cards_by_slug[name]

        if name in self._cards_by_id:
            return self._cards_by_id[name]

        available = sorted(self._cards_by_slug.keys())
        raise AttributeError(
            f"No root card '{name}'. Available: {', '.join(available)}"
        )

    def __iter__(self):
        return iter(self._all_cards)

    def __len__(self) -> int:
        return len(self._all_cards)

    def __repr__(self) -> str:
        return f"<RootCardViewModel cards={list(self._cards_by_slug.keys())}>"


def _wrap_card(card_dict: Dict[str, Any]) -> Any:
    """Wrap a serialized card dict as the appropriate ViewModel type."""
    if card_dict.get("cardinality") == "n":
        return CardListViewModel(card_dict)
    return CardViewModel(card_dict)
