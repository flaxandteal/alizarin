"""StaticGraph round-trips whole: Python reads a graph and writes it back
without losing anything core reads (cards, widgets, functions, publication,
and fields Python doesn't model)."""

import json
import os

import pytest

from alizarin import apply_mutations_from_json
from alizarin.static_types import StaticGraph

MODELS = os.path.join(os.path.dirname(__file__), "../../../tests/data/models")
FIXTURES = ["Group.json", "Person.json", "Heritage Item.json"]


def load(fixture: str) -> dict:
    with open(os.path.join(MODELS, fixture)) as f:
        return json.load(f)["graph"][0]


def core_view(graph: dict) -> dict:
    """The graph as core reads and writes it (no mutations applied)."""
    return json.loads(apply_mutations_from_json(json.dumps(graph), json.dumps({"mutations": []})))


@pytest.mark.parametrize("fixture", FIXTURES)
def test_a_python_round_trip_loses_nothing_core_reads(fixture):
    source = load(fixture)
    round_tripped = StaticGraph.from_dict(source).to_dict()
    assert core_view(round_tripped) == core_view(source)


def test_fields_the_fixtures_lack_survive_too():
    source = load("Group.json")
    source["nodes"][0].update({"source_identifier_id": "src-node", "is_immutable": True})
    source["edges"][0]["source_identifier_id"] = "src-edge"
    source["cards"][0]["source_identifier_id"] = "src-card"
    source["is_active"] = True  # not modelled: kept in `extra`
    round_tripped = StaticGraph.from_dict(source).to_dict()
    assert core_view(round_tripped) == core_view(source)
    assert round_tripped["nodes"][0]["is_immutable"] is True
    assert round_tripped["is_active"] is True


@pytest.mark.parametrize("fixture", FIXTURES)
def test_widgets_and_functions_survive_and_round_trips_are_stable(fixture):
    source = load(fixture)
    graph = StaticGraph.from_dict(source)
    assert len(graph.cards_x_nodes_x_widgets or []) == len(source.get("cards_x_nodes_x_widgets") or [])
    assert len(graph.functions_x_graphs) == len(source.get("functions_x_graphs") or [])
    assert StaticGraph.from_dict(graph.to_dict()).to_dict() == graph.to_dict()
