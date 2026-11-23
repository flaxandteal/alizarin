"""
Every datatype's view model, read through the ORM, against the shared
expectations in tests/data/view-models/expected.json, which TypeScript's
tests/dual-backend/viewModels.test.ts checks on WASM and NAPI: the same on
every backend.
"""

import inspect
import json
import os

import pytest

import alizarin
from alizarin.graph_manager import GraphManager
from alizarin.view_models import viewContext

DATA_DIR = os.path.join(os.path.dirname(__file__), "../../../tests/data/view-models")


def load(name: str):
    with open(os.path.join(DATA_DIR, name), encoding="utf-8") as f:
        return json.load(f)


EXPECTED = {alias: e for alias, e in load("expected.json").items() if not alias.startswith("_")}


async def resolved(value):
    return await value if inspect.isawaitable(value) else value


@pytest.fixture
async def resource():
    """The fixture resource, its concepts' collection in core's RDM cache."""
    collection = load("collection.json")
    cache = alizarin.RustRdmCache()
    cache.add_collection_from_json(collection["id"], json.dumps(list(collection["concepts"].values())))
    alizarin.set_global_rdm_cache(cache)

    gm = GraphManager(default_allow_all_nodegroups=True)
    gm.register_graph(load("graph.json"))
    data = load("resource.json")
    gm.register_resource(data)
    viewContext.graphManager = gm
    try:
        yield await gm.getResource(data["resourceinstance"]["resourceinstanceid"], lazy=False)
    finally:
        viewContext.graphManager = None
        alizarin.clear_global_rdm_cache()


@pytest.mark.asyncio
@pytest.mark.parametrize("alias", sorted(EXPECTED))
async def test_reads_as_expected(resource, alias):
    expected = EXPECTED[alias]
    value = await getattr(resource, alias)
    assert value is not None
    # As it serialises: for_json is the form JSON is written from.
    assert json.loads(json.dumps(await resolved(value.for_json()), default=str)) == expected["forJson"]
    if "str" in expected:
        assert str(value) == expected["str"]
    if "items" in expected:
        assert [str(item) for item in value] == expected["items"]


@pytest.mark.asyncio
@pytest.mark.parametrize("alias", sorted(EXPECTED))
async def test_get_tile_gives_the_stored_value(resource, alias):
    # The view model's tile form (its __asTileData is name-mangled, so it was
    # never found), else core's value: the tile's own, either way.
    nodes = {node["alias"]: node["nodeid"] for node in load("graph.json")["nodes"]}
    stored = next(tile["data"][nodes[alias]] for tile in load("resource.json")["tiles"] if nodes[alias] in tile["data"])
    await getattr(resource, alias)
    pseudos = await resource._i.retrieve_pseudo(alias)
    pseudo = pseudos[0] if isinstance(pseudos, list) else pseudos
    tile_value, relationships = pseudo.get_tile()
    if isinstance(stored, list) and stored and isinstance(stored[0], dict):
        # Resource references: the tile form keeps resourceId alone.
        stored = [{"resourceId": ref["resourceId"]} for ref in stored]
        tile_value = [{"resourceId": ref["resourceId"]} for ref in tile_value]
    assert tile_value == stored
    assert relationships == []


@pytest.mark.asyncio
async def test_a_subclass_view_model_has_its_base_class_tile_form():
    from alizarin.view_models import ResourceInstanceViewModel, _tile_form, as_tile_data_method

    class Place(ResourceInstanceViewModel):
        pass

    place = Place("eeeeeeee-1111-2222-3333-555555555555", None, None, None)
    assert as_tile_data_method(place) is not None
    assert await _tile_form(place) == {"resourceId": "eeeeeeee-1111-2222-3333-555555555555"}


def test_every_datatype_in_the_fixture_has_an_expectation():
    aliases = {node["alias"] for node in load("graph.json")["nodes"] if not node["istopnode"]}
    assert set(EXPECTED) == aliases
