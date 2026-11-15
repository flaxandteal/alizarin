"""
Editing through the ORM: assignment records edits (checked against the graph
at once), reads see them, and commit() applies them through core's
ResourceEdit, returning the tile delta. Mirrors the TypeScript dual-backend
tests/dual-backend/ormEdit.test.ts, on the same model.
"""

import json
import os

import pytest

from alizarin import GraphMutator, set_current_language
from alizarin.graph_manager import GraphManager
from alizarin.static_types import StaticGraph
from alizarin.view_models import viewContext

LITERAL = "http://www.w3.org/2000/01/rdf-schema#Literal"
NOTE = "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
RESOURCE_ID = "eeeeeeee-1111-2222-3333-444444444444"


def english(value: str) -> dict:
    return {"en": {"direction": "ltr", "value": value}}


@pytest.fixture
def town():
    """A place titled "Town" with one nickname, "The Big Smoke": `title` is
    translatable and single-valued, `nicknames` multi-valued."""
    path = os.path.join(os.path.dirname(__file__), "../../../tests/data/models/Group.json")
    with open(path) as f:
        base = StaticGraph.from_dict(json.load(f)["graph"][0])
    mutator = GraphMutator(base)
    mutator.add_string_node(None, "title", "Title", "1", LITERAL, NOTE)
    mutator.add_string_node(None, "nicknames", "Nicknames", "n", LITERAL, NOTE)
    graph = mutator.apply()
    nodes = {n.alias: n for n in graph.nodes}

    def tile(alias, tile_id, value):
        node = nodes[alias]
        return {
            "tileid": tile_id,
            "nodegroup_id": node.nodegroup_id,
            "resourceinstance_id": RESOURCE_ID,
            "parenttile_id": None,
            "data": {node.nodeid: english(value)},
            "sortorder": 0,
        }

    gm = GraphManager()
    gm.register_graph(graph.to_dict())
    gm.register_resource({
        "resourceinstance": {
            "resourceinstanceid": RESOURCE_ID,
            "graph_id": graph.graphid,
            "name": "Town",
        },
        "tiles": [
            tile("title", "aaaaaaaa-0000-0000-0000-000000000001", "Town"),
            tile("nicknames", "aaaaaaaa-0000-0000-0000-000000000002", "The Big Smoke"),
        ],
    })
    viewContext.graphManager = gm
    yield gm
    viewContext.graphManager = None
    set_current_language("en")


@pytest.mark.asyncio
async def test_a_plain_string_sets_the_current_language_keeping_the_others(town):
    place = await town.getResource(RESOURCE_ID, lazy=False)
    set_current_language("ga")
    place.title = "Baile"

    # Reads see the assignment before the commit.
    assert await place.title == "Baile"

    delta = place.commit()
    assert len(delta["upserted"]) == 1
    assert delta["removed"] == []
    title = await place.title
    assert title.lang("ga") == "Baile"
    assert title.lang("en") == "Town"


@pytest.mark.asyncio
async def test_append_and_del_add_and_remove_multi_valued_tiles(town):
    place = await town.getResource(RESOURCE_ID, lazy=False)
    nicknames = await place.nicknames
    assert len(nicknames) == 1

    nicknames.append({"nicknames": "Dublin"})
    assert len(await place.nicknames) == 2
    added = place.commit()
    assert len(added["upserted"]) == 1
    assert len(await place.nicknames) == 2

    reread = await place.nicknames
    del reread[0]
    removed = place.commit()
    assert len(removed["removed"]) == 1
    assert len(await place.nicknames) == 1


@pytest.mark.asyncio
async def test_assignments_the_graph_rules_out_raise_where_they_are_made(town):
    place = await town.getResource(RESOURCE_ID, lazy=False)
    with pytest.raises(AttributeError, match="no child no_such_node"):
        place.no_such_node = 1
    with pytest.raises(ValueError, match="multi-valued"):
        place.nicknames = "x"
    assert not place._i.has_pending_edits()


@pytest.mark.asyncio
async def test_discard_drops_what_was_assigned(town):
    place = await town.getResource(RESOURCE_ID, lazy=False)
    place.title = "Somewhere else"
    place.discard()
    assert await place.title == "Town"
    assert place.commit()["upserted"] == []
