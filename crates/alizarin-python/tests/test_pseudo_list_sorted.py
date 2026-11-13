"""PseudoList.sorted() orders values by tile sortorder as core orders semantic
children: tiles without one after the numbered ones, ties in load order."""

import asyncio
from types import SimpleNamespace

from alizarin.pseudos import PseudoList


class _Value:
    def __init__(self, name, sortorder):
        self.name = name
        self.tile = SimpleNamespace(sortorder=sortorder)

    async def get_value(self):
        return self.name


def test_sorted_puts_unnumbered_tiles_last_in_load_order():
    values = [
        _Value("a", None),
        _Value("b", 2),
        _Value("c", 1),
        _Value("d", None),
        _Value("e", 1),
    ]
    pseudo_list = PseudoList("alias", values)
    assert asyncio.run(pseudo_list.sorted()) == ["c", "e", "b", "a", "d"]


def test_sorted_treats_a_value_without_a_tile_as_unnumbered():
    no_tile = _Value("x", None)
    no_tile.tile = None
    pseudo_list = PseudoList("alias", [no_tile, _Value("y", 0)])
    assert asyncio.run(pseudo_list.sorted()) == ["y", "x"]
