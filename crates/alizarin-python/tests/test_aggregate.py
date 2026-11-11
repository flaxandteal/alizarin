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


def test_layer_extent_rules_out_a_query_box_only_when_disjoint():
    pytest.importorskip("alizarin_geo")
    expected = load("expected_geo_tiles.json")
    alizarin.register_graph(json.dumps(load("layer_geo.json")))
    registry = loaded_registry()
    registry.apply_aggregate_functions(expected["layer_id"])
    tile = layer_tiles(registry.get_full(expected["layer_id"]))["extent-buildings"]
    extent = json.dumps(tile["data"]["c423f13e-50ba-59e3-badd-a583d628defe"])

    def box(bbox):
        return alizarin.fold_may_match(
            "geojson-feature-collection", extent, json.dumps({"bbox": bbox})
        )

    assert box([-7, 54.5, -6.5, 55])  # inside
    assert box([-6, 55.5, 0, 60])  # touches a corner (inclusive)
    assert not box([100, -10, 110, 0])  # far away


def test_number_summary_may_match_a_range():
    summary = json.dumps({"count": 2, "min": 10, "max": 20})
    assert not alizarin.fold_may_match("number", summary, '{"range": {"min": 21}}')
    assert alizarin.fold_may_match("number", summary, '{"range": {"min": 20}}')
    assert not alizarin.fold_may_match("number", "null", '{"range": {}}')


def test_adding_members_in_turns_matches_a_full_run():
    registry = loaded_registry()
    # As a loop would: the ids added since the last consumer event, per call.
    for turn in (["b1"], ["b2", "b3"], ["b4"]):
        registry.apply_aggregate_additions(EXPECTED["layer_id"], turn)
    assert layer_tiles(registry.get_full(EXPECTED["layer_id"])) == EXPECTED["tiles"]


def test_adding_an_unloaded_member_is_an_error():
    registry = loaded_registry()
    with pytest.raises(ValueError, match="not loaded in full"):
        registry.apply_aggregate_additions(EXPECTED["layer_id"], ["no-such-member"])


def test_an_unknown_target_is_an_error():
    with pytest.raises(ValueError, match="not loaded in full"):
        alizarin.ResourceRegistry().apply_aggregate_functions("no-such-resource")
