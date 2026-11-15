"""
The core instance wrapper's methods that Python gained to match NAPI and WASM
(tests/binding-parity.json): loading a whole resource, reading its tiles,
name, descriptors and pseudos, serialising, and pruning to the model's
permissions.
"""

import json
import os
import uuid

import pytest

from alizarin import PyResourceModelWrapper, register_graph
from alizarin.instance_wrapper import RustResourceInstanceWrapperCore

MODELS_DIR = os.path.join(os.path.dirname(__file__), "../../../tests/data/models")

BASIC_INFO = "12707705-c05e-11e9-8177-a4d18cec433a"
NAME = "127095f5-c05e-11e9-bb57-a4d18cec433a"
GUIDELINE_APPROVAL = "389f0d26-027c-11ed-9dee-0242ac180008"


def group_graph() -> dict:
    with open(os.path.join(MODELS_DIR, "Group.json")) as f:
        return json.load(f)["graph"][0]


@pytest.fixture
def loaded():
    """A model wrapper and a core wrapper loaded with one Group resource;
    the model's permissions restored afterwards, as the registry is shared."""
    graph = group_graph()
    register_graph(json.dumps(graph))
    model = PyResourceModelWrapper(graph["graphid"])
    resource_id = str(uuid.uuid4())
    resource = {
        "resourceinstance": {
            "resourceinstanceid": resource_id,
            "graph_id": graph["graphid"],
            "name": "Ráth",
            "descriptors": {},
        },
        "tiles": [
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
        ],
    }
    core = RustResourceInstanceWrapperCore(graph["graphid"])
    core.load_tiles_from_resource(json.dumps(resource))
    try:
        yield model, core, resource, graph
    finally:
        model.set_permitted_nodegroups({})
        model.set_default_allow_all_nodegroups(True)


def test_a_whole_resource_loads_its_metadata_and_tiles(loaded):
    _, core, resource, _ = loaded
    assert core.tiles_loaded()
    assert core.get_resource_id() == resource["resourceinstance"]["resourceinstanceid"]
    assert core.get_name() == "Ráth"
    approval = resource["tiles"][1]
    assert core.get_tile(approval["tileid"])["data"] == {GUIDELINE_APPROVAL: True}
    with pytest.raises(KeyError):
        core.get_tile(str(uuid.uuid4()))


def test_exported_tiles_are_the_stored_ones(loaded):
    _, core, resource, _ = loaded
    exported = json.loads(core.export_tiles_json())
    assert sorted(t["tileid"] for t in exported) == sorted(t["tileid"] for t in resource["tiles"])


def test_to_json_populates_and_serialises_raw_values(loaded):
    _, core, _, _ = loaded
    out = core.to_json()
    assert out["guideline_approval"] is True
    # Populated as it serialised, so the root pseudo is cached.
    assert core.get_root_pseudo() is not None


def test_descriptors_and_pseudos_come_from_core(loaded):
    _, core, _, graph = loaded
    assert isinstance(core.get_descriptors(True), dict)

    root = next(n for n in graph["nodes"] if n.get("istopnode"))
    children = core.get_all_semantic_child_values(None, root["nodeid"], None)
    assert "guideline_approval" in children
    assert not children["guideline_approval"].is_empty

    assert core.make_pseudo_value("guideline_approval", None, True, False).result_type == "list"
    assert core.make_pseudo_value("guideline_approval", None, False, True).is_empty


def test_a_cyclic_argument_raises_rather_than_crashing(loaded):
    # Python's dict-to-JSON conversion once recursed until the process
    # overflowed its stack.
    model, core, _, _ = loaded
    rule = {"path": ".x", "allowed": []}
    rule["self"] = rule
    with pytest.raises(ValueError, match="Circular reference"):
        model.set_permitted_nodegroups({GUIDELINE_APPROVAL: rule})
    assert core.tiles_loaded()


def test_pruning_drops_tiles_the_model_no_longer_permits(loaded):
    model, core, resource, _ = loaded
    model.set_permitted_nodegroups({GUIDELINE_APPROVAL: False})
    assert model.get_permitted_nodegroups()[GUIDELINE_APPROVAL] is False

    core.prune_resource_tiles()
    exported = json.loads(core.export_tiles_json())
    assert [t["nodegroup_id"] for t in exported] == [BASIC_INFO]
    with pytest.raises(KeyError):
        core.get_tile(resource["tiles"][1]["tileid"])
