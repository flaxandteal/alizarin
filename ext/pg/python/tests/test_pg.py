"""
batch_trees_to_pg_copy converts through alizarin's own registries (the graph
registered with alizarin.register_graph, its extension handlers and RDM
cache) and writes what batch_trees_to_tiles would give, as COPY rows.
"""

import json
from pathlib import Path

import pytest

import alizarin
from alizarin_pg import batch_trees_to_pg_copy

GROUP = Path(__file__).parents[4] / "tests" / "data" / "models" / "Group.json"
RESOURCE_ID = "11111111-1111-1111-1111-111111111111"


@pytest.fixture(scope="module")
def graph_id():
    with open(GROUP) as f:
        graph = json.load(f)["graph"][0]
    alizarin.register_graph(json.dumps(graph))
    return graph["graphid"]


def trees(*names):
    return json.dumps([
        {"resourceinstanceid": f"{i + 1:08d}-1111-1111-1111-111111111111",
         "basic_info": [{"name": {"en": name}}]}
        for i, name in enumerate(names)
    ])


def rows(copy):
    return [line.split("\t") for line in copy.splitlines()]


def test_tiles_are_alizarins_conversion(graph_id):
    result = batch_trees_to_pg_copy(trees("Test Group"), graph_id)
    expected = alizarin.batch_trees_to_tiles(trees("Test Group"), graph_id)
    tiles = expected["business_data"]["resources"][0]["tiles"]

    assert result["error_count"] == 0
    # Tile ids of cardinality-n tiles are random per call, so compare the rest.
    assert sorted(
        (r[1], r[2], json.loads(r[3]), r[4], r[5], r[6]) for r in rows(result["tiles_copy"])
    ) == sorted(
        (t["resourceinstance_id"], t["parenttile_id"] or "\\N", t["data"], t["nodegroup_id"],
         str(t["sortorder"] or 0), "\\N")
        for t in tiles
    )


def test_resource_rows_take_the_copy_options(graph_id):
    result = batch_trees_to_pg_copy(
        trees("A", "B"), graph_id,
        graph_publication_id="pub-1", lifecycle_state_id="lc-1",
        created_time="2026-01-01T00:00:00Z",
    )
    resources = rows(result["resources_copy"])
    assert [r[0] for r in resources] == [
        RESOURCE_ID.replace("11111111", "00000001", 1),
        RESOURCE_ID.replace("11111111", "00000002", 1),
    ]
    assert {(r[1], r[2], r[6], r[7]) for r in resources} == {
        (graph_id, "pub-1", "2026-01-01T00:00:00Z", "lc-1"),
    }


# Not a tree: an error per tree, not a warning.
BAD_TREE = "not a tree"


def test_non_strict_reports_errors_and_writes_the_rest(graph_id):
    batch = json.dumps([BAD_TREE] + json.loads(trees("Fine")))
    result = batch_trees_to_pg_copy(batch, graph_id, strict=False)
    assert result["error_count"] == 1
    assert result["errors"][0].startswith("Tree 0:")
    assert len(rows(result["resources_copy"])) == 1


def test_strict_raises(graph_id):
    with pytest.raises(ValueError):
        batch_trees_to_pg_copy(json.dumps([BAD_TREE]), graph_id)


def test_an_unregistered_graph_is_alizarins_error():
    with pytest.raises(KeyError):
        batch_trees_to_pg_copy(trees("X"), "00000000-0000-0000-0000-000000000000")
