"""Output parity: Python's serialisation of every static type is core's.

tests/data/output-parity/expected.json is core's serialisation of the fixture
in that directory (crates/alizarin-core/tests/output_parity_test.rs); WASM and
NAPI are held to it in tests/dual-backend/outputParity.test.ts, and Python here.

Each Python route's result is dumped as compact JSON and compared with the
golden entry, so key presence (``None`` written vs left out) and values must
match exactly. Keys are compared sorted: a Python dict's order is not part of
its value, and ``json.dumps(..., sort_keys=True)`` gives the golden's order.
"""

import json
import os

import pytest

import alizarin
from alizarin.static_types import (
    StaticGraph,
    StaticPublication,
    StaticResource,
    StaticTile,
)

DATA_DIR = os.path.join(os.path.dirname(__file__), "../../../tests/data/output-parity")


def load(name: str):
    with open(os.path.join(DATA_DIR, name), encoding="utf-8") as f:
        return json.load(f)


GOLDEN = load("expected.json")
GRAPH = load("graph.json")["graph"][0]
RESOURCE = load("resource.json")
LAYER = load("resource-layer.json")


def dumped(value) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def routes():
    graph = StaticGraph.from_dict(GRAPH)
    out = [
        ("StaticGraph.to_dict", "graph", lambda: graph.to_dict()),
        ("graph.name", "graph.name", lambda: graph.name),
        ("graph.root.to_dict", "graph.root", lambda: graph.root.to_dict()),
    ]
    for i, node in enumerate(GRAPH["nodes"]):
        out.append((f"graph.nodes[{i}]", f"node:{node['alias']}", lambda i=i: graph.nodes[i].to_dict()))
    for i, ng in enumerate(GRAPH["nodegroups"]):
        out.append(
            (f"graph.nodegroups[{i}]", f"nodegroup:{ng['nodegroupid']}", lambda i=i: graph.nodegroups[i].to_dict())
        )
    for i, edge in enumerate(GRAPH["edges"]):
        out.append((f"graph.edges[{i}]", f"edge:{edge['edgeid']}", lambda i=i: graph.edges[i].to_dict()))
    for i, card in enumerate(GRAPH["cards"]):
        out.append((f"graph.cards[{i}]", f"card:{card['cardid']}", lambda i=i: graph.cards[i].to_dict()))
    for i, row in enumerate(GRAPH["cards_x_nodes_x_widgets"]):
        out.append(
            (
                f"graph.cards_x_nodes_x_widgets[{i}]",
                f"cxnxw:{row['id']}",
                lambda i=i: graph.cards_x_nodes_x_widgets[i].to_dict(),
            )
        )
    for i, row in enumerate(GRAPH["functions_x_graphs"]):
        out.append(
            (f"graph.functions_x_graphs[{i}]", f"fxg:{row['id']}", lambda i=i: graph.functions_x_graphs[i].to_dict())
        )
    out.append(
        ("StaticPublication", "publication", lambda: StaticPublication.from_dict(GRAPH["publication"]).to_dict())
    )
    out.append(("StaticResource", "resource", lambda: StaticResource.from_dict(RESOURCE).to_dict()))
    for tile in RESOURCE["tiles"]:
        out.append(
            (
                f"StaticTile({tile['tileid']})",
                f"tile:{tile['tileid']}",
                lambda tile=tile: StaticTile.from_dict(tile).to_dict(),
            )
        )
    out.append(
        (
            "StaticTile(empty)",
            "tile:empty",
            lambda: StaticTile(
                tileid=None,
                nodegroup_id=RESOURCE["tiles"][0]["nodegroup_id"],
                resourceinstance_id=RESOURCE["resourceinstance"]["resourceinstanceid"],
                data={},
            ).to_dict(),
        )
    )
    out.append(
        (
            "compose_resource_layers",
            "composed",
            lambda: alizarin.compose_resource_layers(json.dumps([LAYER, RESOURCE]), json.dumps(GRAPH), "[]", False),
        )
    )
    return out


@pytest.mark.parametrize("route,label,get", routes(), ids=[r[0] for r in routes()])
def test_route_gives_cores_json(route, label, get):
    assert dumped(get()) == GOLDEN[label]
