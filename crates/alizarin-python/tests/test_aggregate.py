"""Aggregate functions from Python: a Layer resource's fold Aggregate writes the
same node values as on WASM and NAPI (tests/dual-backend/aggregate.test.ts),
checked against the same fixture and expected tiles."""

import json
import os

import pytest

import alizarin

DATA = os.path.join(os.path.dirname(__file__), "../../../tests/data/aggregate")


def load(name):
    with open(os.path.join(DATA, name)) as f:
        return json.load(f)


EXPECTED = load("expected_layer_tiles.json")


@pytest.fixture(scope="module", autouse=True)
def registered_graphs():
    alizarin.register_graph(json.dumps(load("building.json")))
    alizarin.register_graph(json.dumps(load("layer.json")))


def layer_tiles(resource):
    return {
        t["tileid"]: {"nodegroup_id": t["nodegroup_id"], "data": t["data"]}
        for t in resource["tiles"]
    }


def loaded_registry():
    registry = alizarin.ResourceRegistry()
    resources = load("business_data.json")["business_data"]["resources"]
    registry.merge_from_resources(json.dumps(resources), True, False)
    return registry


def test_registry_slice_writes_the_expected_node_values():
    registry = loaded_registry()
    registry.apply_aggregate_functions(EXPECTED["layer_id"])
    assert layer_tiles(registry.get_full(EXPECTED["layer_id"])) == EXPECTED["tiles"]


def test_rerunning_gives_identical_tiles():
    registry = loaded_registry()
    registry.apply_aggregate_functions(EXPECTED["layer_id"])
    first = layer_tiles(registry.get_full(EXPECTED["layer_id"]))
    registry.apply_aggregate_functions(EXPECTED["layer_id"])
    assert layer_tiles(registry.get_full(EXPECTED["layer_id"])) == first


def test_streamed_file_slice_matches():
    resources = load("business_data.json")["business_data"]["resources"]
    layer = next(
        r for r in resources if r["resourceinstance"]["resourceinstanceid"] == EXPECTED["layer_id"]
    )
    result = json.loads(
        alizarin.apply_aggregate_functions_from_file(
            json.dumps(layer), os.path.join(DATA, "business_data.json")
        )
    )
    assert layer_tiles(result) == EXPECTED["tiles"]


def test_geometry_folds_through_geo_over_the_c_abi():
    pytest.importorskip("alizarin_geo")  # registers geo's handler (validate + fold) on import
    expected = load("expected_geo_tiles.json")
    alizarin.register_graph(json.dumps(load("layer_geo.json")))
    registry = loaded_registry()
    registry.apply_aggregate_functions(expected["layer_id"])
    assert layer_tiles(registry.get_full(expected["layer_id"])) == expected["tiles"]


def test_an_unknown_target_is_an_error():
    with pytest.raises(ValueError, match="not loaded in full"):
        alizarin.ResourceRegistry().apply_aggregate_functions("no-such-resource")
