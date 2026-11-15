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

    gm = GraphManager()
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


def test_every_datatype_in_the_fixture_has_an_expectation():
    aliases = {node["alias"] for node in load("graph.json")["nodes"] if not node["istopnode"]}
    assert set(EXPECTED) == aliases
