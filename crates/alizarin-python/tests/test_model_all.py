"""
ResourceModelWrapper.all(), as tests/dual-backend/staticStoreRegistry.test.ts
("ORM read-back") checks it on WASM and NAPI: the Group resource comes back as a
view model of the model all() was called on, and reads through to its data.
"""

import json
import os

import pytest

from alizarin.graph_manager import GraphManager
from alizarin.model_wrapper import ResourceModelWrapper
from alizarin.view_models import viewContext

DATA = os.path.join(os.path.dirname(__file__), "../../../tests")
GRAPH_ID = "07883c9e-b25c-11e9-975a-a4d18cec433a"
RESOURCE_ID = "d2368123-9628-49a2-b3dd-78ac6ee3e911"


def load(path):
    with open(os.path.join(DATA, path)) as f:
        return json.load(f)


@pytest.fixture
def manager():
    gm = GraphManager(default_allow_all_nodegroups=True)
    gm.register_graph(load("data/models/Group.json")["graph"][0])
    viewContext.graphManager = gm
    yield gm
    viewContext.graphManager = None


def with_group(gm):
    resources = load(f"definitions/resources/_{GRAPH_ID}.json")["business_data"]["resources"]
    for resource in resources:
        gm.register_resource(resource)


@pytest.mark.asyncio
async def test_all_reads_back_the_resource(manager):
    with_group(manager)
    model = ResourceModelWrapper.from_graph_id(GRAPH_ID, True)
    instances = await model.all()
    assert len(instances) == 1

    instance = instances[0]
    assert instance.id == RESOURCE_ID
    assert instance._i.model is model

    root = await instance._i.get_root_view_model()
    basic_info = await root.basic_info
    name = await basic_info[0].name
    assert str(name) == "Global Group"


@pytest.mark.asyncio
async def test_all_takes_a_limit_and_only_the_models_graph(manager):
    with_group(manager)
    manager.register_resource({
        "resourceinstance": {
            "resourceinstanceid": "ffffffff-0000-0000-0000-000000000001",
            "graph_id": "ffffffff-0000-0000-0000-00000000000a",
            "name": "Elsewhere",
        },
        "tiles": [],
    })
    model = ResourceModelWrapper.from_graph_id(GRAPH_ID, True)
    assert [i.id for i in await model.all()] == [RESOURCE_ID]
    assert await model.all({"limit": 1}) != []


@pytest.mark.asyncio
async def test_all_with_no_resources_is_empty(manager):
    assert await ResourceModelWrapper.from_graph_id(GRAPH_ID, True).all() == []
