"""
Editing through the ORM: assignment records edits (checked against the graph
at once), reads see them, and commit() applies them through core's
ResourceEdit, returning the tile delta. Mirrors the TypeScript dual-backend
tests/dual-backend/ormEdit.test.ts, on the same model.
"""

import inspect
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

    # Reads show the current language, falling back to the default.
    assert await place.title == "Baile"
    set_current_language("en")
    assert await place.title == "Town"
    set_current_language("fr")
    assert await place.title == "Town"


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


@pytest.mark.asyncio
async def test_deleting_an_item_appended_since_the_last_commit_raises(town):
    place = await town.getResource(RESOURCE_ID, lazy=False)
    nicknames = await place.nicknames
    nicknames.append({"nicknames": "Dublin"})
    with pytest.raises(ValueError, match="added since the last commit"):
        del nicknames[1]
    place.discard()


@pytest.mark.asyncio
async def test_extend_iadd_pop_and_remove_record_edits(town):
    place = await town.getResource(RESOURCE_ID, lazy=False)
    nicknames = await place.nicknames
    nicknames.extend([{"nicknames": "Dublin"}])
    nicknames += [{"nicknames": "Áth Cliath"}]
    assert len(place.commit()["upserted"]) == 2

    nicknames = await place.nicknames
    assert len(nicknames) == 3
    nicknames.pop()
    nicknames.remove(nicknames[0])
    assert len(place.commit()["removed"]) == 2
    assert len(await place.nicknames) == 1


def test_assigning_before_the_resource_is_retrieved_raises():
    from alizarin.view_models import ResourceInstanceViewModel

    unretrieved = ResourceInstanceViewModel(id=RESOURCE_ID)
    with pytest.raises(ValueError, match="retrieve this resource first"):
        unretrieved.title = "Baile"


EVENT_1 = "bbbbbbbb-0000-0000-0000-000000000001"
EVENT_2 = "bbbbbbbb-0000-0000-0000-000000000002"
E5 = "http://www.cidoc-crm.org/cidoc-crm/E5_Event"


@pytest.fixture
def with_events():
    """A place with `events` (semantic, n), each with `notes` (n): the first
    event has one note, the second none."""
    path = os.path.join(os.path.dirname(__file__), "../../../tests/data/models/Group.json")
    with open(path) as f:
        base = StaticGraph.from_dict(json.load(f)["graph"][0])
    mutator = GraphMutator(base)
    mutator.add_semantic_node(None, "events", "Events", "n", E5, NOTE)
    mutator.add_string_node("events", "notes", "Notes", "n", LITERAL, NOTE)
    graph = mutator.apply()
    nodes = {n.alias: n for n in graph.nodes}

    def tile(alias, tile_id, parent, data, sortorder):
        return {
            "tileid": tile_id,
            "nodegroup_id": nodes[alias].nodegroup_id,
            "resourceinstance_id": RESOURCE_ID,
            "parenttile_id": parent,
            "data": data,
            "sortorder": sortorder,
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
            tile("events", EVENT_1, None, {}, 0),
            tile("events", EVENT_2, None, {}, 1),
            tile("notes", "cccccccc-0000-0000-0000-000000000001", EVENT_1,
                 {nodes["notes"].nodeid: english("Founded")}, 0),
        ],
    })
    viewContext.graphManager = gm
    yield gm, nodes
    viewContext.graphManager = None


@pytest.mark.asyncio
async def test_append_under_an_item_adds_a_tile_with_that_item_as_its_parent(with_events):
    gm, _ = with_events
    place = await gm.getResource(RESOURCE_ID, lazy=False)
    events = await place.events
    notes = await events[0].notes
    assert len(notes) == 1
    notes.append({"notes": "Renamed"})
    delta = place.commit()
    assert len(delta["upserted"]) == 1
    assert delta["upserted"][0]["parenttile_id"] == EVENT_1


@pytest.mark.asyncio
async def test_append_on_an_empty_nested_list_uses_its_container_as_the_parent(with_events):
    gm, _ = with_events
    place = await gm.getResource(RESOURCE_ID, lazy=False)
    events = await place.events
    notes = await events[1].notes
    assert notes == []
    notes.append({"notes": "Planned"})
    delta = place.commit()
    assert len(delta["upserted"]) == 1
    assert delta["upserted"][0]["parenttile_id"] == EVENT_2


@pytest.mark.asyncio
async def test_a_value_wrapped_from_core_finds_its_node_on_the_model(with_events):
    gm, nodes = with_events
    place = await gm.getResource(RESOURCE_ID, lazy=False)
    events = await place.events
    notes = await events[0].notes
    note = notes[0]
    # get_view_model gave the view model its pseudo, where its class reads it.
    parent_pseudo = note._StringViewModel__parentPseudo
    assert parent_pseudo is not None
    assert parent_pseudo.node.nodeid == nodes["notes"].nodeid
    assert parent_pseudo.node is place._i.model.get_node_objects()[nodes["notes"].nodeid]


@pytest.fixture
def with_kind():
    """A place, with no tiles yet, whose `kind` (a string) has a child
    `kind_note`, as the TypeScript test's."""
    path = os.path.join(os.path.dirname(__file__), "../../../tests/data/models/Group.json")
    with open(path) as f:
        base = StaticGraph.from_dict(json.load(f)["graph"][0])
    mutator = GraphMutator(base)
    mutator.add_string_node(None, "kind", "Kind", "1", LITERAL, NOTE)
    mutator.add_string_node("kind", "kind_note", "Kind note", "1", LITERAL, NOTE)
    graph = mutator.apply()

    gm = GraphManager()
    gm.register_graph(graph.to_dict())
    gm.register_resource(
        {
            "resourceinstance": {"resourceinstanceid": RESOURCE_ID, "graph_id": graph.graphid, "name": "Town"},
            "tiles": [],
        }
    )
    viewContext.graphManager = gm
    yield gm
    viewContext.graphManager = None


@pytest.mark.asyncio
async def test_assigning_an_outer_nodes_tree_form_sets_its_value_and_children(with_kind):
    place = await with_kind.getResource(RESOURCE_ID, lazy=False)
    place.kind = {"_": "Fort", "kind_note": "Old"}
    delta = place.commit()
    assert len(delta["upserted"]) > 0

    kind = await place.kind
    assert kind == "Fort"
    inner = kind._
    inner = await inner if inspect.isawaitable(inner) else inner
    assert await inner.kind_note == "Old"
