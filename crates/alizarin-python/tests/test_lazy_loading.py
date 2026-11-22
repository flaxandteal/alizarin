"""
Lazy tile loading through the Python ORM, as TypeScript does on NAPI and WASM:
a lazy resource fetches each nodegroup a lookup needs (core lists them), through
the compiled-in tile source, then the tile loader, then the resource's own
tiles, and reads semantic children with core's per-tile lookup in both modes.
"""

import inspect
import json
import os
import uuid

import pytest

from alizarin.graph_manager import GraphManager
from alizarin.view_models import viewContext

MODELS_DIR = os.path.join(os.path.dirname(__file__), "../../../tests/data/models")

BASIC_INFO = "12707705-c05e-11e9-8177-a4d18cec433a"
NAME = "127095f5-c05e-11e9-bb57-a4d18cec433a"
GUIDELINE_APPROVAL = "389f0d26-027c-11ed-9dee-0242ac180008"


def group_graph() -> dict:
    with open(os.path.join(MODELS_DIR, "Group.json")) as f:
        return json.load(f)["graph"][0]


def group_tiles(resource_id: str) -> list:
    return [
        {
            "tileid": str(uuid.uuid4()),
            "nodegroup_id": BASIC_INFO,
            "resourceinstance_id": resource_id,
            "parenttile_id": None,
            "data": {NAME: {"en": {"value": "Ráth", "direction": "ltr"}}},
            "sortorder": 0,
        },
        {
            "tileid": str(uuid.uuid4()),
            "nodegroup_id": GUIDELINE_APPROVAL,
            "resourceinstance_id": resource_id,
            "parenttile_id": None,
            "data": {GUIDELINE_APPROVAL: True},
            "sortorder": 0,
        },
    ]


@pytest.fixture
def setup():
    """A GraphManager holding one Group resource registered with `tiles`."""
    managers = []

    def make(tiles_for):
        gm = GraphManager()
        graph_id = gm.register_graph(group_graph())
        resource_id = str(uuid.uuid4())
        gm.register_resource(
            {
                "resourceinstance": {
                    "resourceinstanceid": resource_id,
                    "graph_id": graph_id,
                    "name": "Group",
                },
                "tiles": tiles_for(resource_id),
            }
        )
        viewContext.graphManager = gm
        managers.append(gm)
        return gm, resource_id

    yield make
    viewContext.graphManager = None


async def plain(value):
    """A view model's JSON form, awaiting where the view model is async."""
    result = value.for_json() if hasattr(value, "for_json") else value
    return await result if inspect.isawaitable(result) else result


@pytest.mark.asyncio
async def test_a_lazy_resource_fetches_only_the_nodegroup_it_reads(setup):
    gm, resource_id = setup(lambda _: [])
    tiles = group_tiles(resource_id)
    group = await gm.getResource(resource_id, lazy=True)

    fetched = []

    async def loader(nodegroup_id):
        fetched.append(nodegroup_id)
        return [t for t in tiles if t["nodegroup_id"] == nodegroup_id]

    group._i.set_tile_loader(loader)
    assert group._i.is_lazy()

    approval = await group.guideline_approval
    assert await plain(approval) is True
    assert fetched == [GUIDELINE_APPROVAL]


@pytest.mark.asyncio
async def test_a_multi_valued_child_is_a_list_lazily_and_eagerly(setup):
    # Lazily, through the loader.
    gm, resource_id = setup(lambda _: [])
    tiles = group_tiles(resource_id)
    group = await gm.getResource(resource_id, lazy=True)

    async def loader(nodegroup_id):
        return [t for t in tiles if t["nodegroup_id"] == nodegroup_id]

    group._i.set_tile_loader(loader)
    lazy_basic_info = await group.basic_info
    assert len(lazy_basic_info) == 1

    # Eagerly, with every tile loaded up front: the same shape.
    gm, resource_id = setup(group_tiles)
    group = await gm.getResource(resource_id, lazy=False)
    eager_basic_info = await group.basic_info
    assert len(eager_basic_info) == 1
    assert not group._i.is_lazy()


@pytest.mark.asyncio
async def test_without_a_loader_the_resources_own_tiles_are_fetched(setup):
    gm, resource_id = setup(group_tiles)
    group = await gm.getResource(resource_id, lazy=True)
    full = group._i.resource

    # Start again from nothing loaded, keeping the resource's tiles to fall
    # back on.
    group._i._rust_core.release()
    group._i.resource = full

    approval = await group.guideline_approval
    assert await plain(approval) is True
    assert group._i._rust_core.is_nodegroup_loaded(GUIDELINE_APPROVAL)
    assert not group._i._rust_core.is_nodegroup_loaded(BASIC_INFO)
