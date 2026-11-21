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
# resource.json with its descriptors in Arches' per-language form: its own under
# "en", a decoy under "de" (which sorts first). Read as "en", it must give the
# same resource (as output_parity_test.rs checks in core).
RESOURCE_BY_LANGUAGE = {
    **RESOURCE,
    "resourceinstance": {
        **RESOURCE["resourceinstance"],
        "descriptors": {"de": {"name": "Parität eins"}, "en": RESOURCE["resourceinstance"]["descriptors"]},
    },
}

# Arches' Standard lifecycle, as in graph.json.
ACTIVE = "f75bb034-36e3-4ab4-8167-f520cf0b4c58"
RETIRED = "d95d9c0e-0e2c-4450-93a3-d788b91abcc8"


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
    out.append(
        (
            "StaticResource(per-language descriptors)",
            "resource",
            lambda: StaticResource.from_dict(RESOURCE_BY_LANGUAGE).to_dict(),
        )
    )

    def summary():
        registry = alizarin.ResourceRegistry()
        registry.merge_from_resources(json.dumps([RESOURCE]))
        return registry.get_summary(RESOURCE["resourceinstance"]["resourceinstanceid"])

    out.append(("summary_to_metadata", "resource.summary.metadata", lambda: alizarin.summary_to_metadata(summary())))
    out.append(
        ("static_resource_from_summary", "resource.from_summary", lambda: alizarin.static_resource_from_summary(summary()))
    )
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
    # Lifecycles: the base is Active and the layer copy Draft, so the composed
    # resource is a Conflict.
    lifecycle = json.dumps(GRAPH["resource_instance_lifecycle"])

    def composed():
        return json.dumps(
            alizarin.compose_resource_layers(json.dumps([LAYER, RESOURCE]), json.dumps(GRAPH), "[]", False)["resource"]
        )

    out += [
        (
            "resource_lifecycle_state",
            "lifecycle:state",
            lambda: alizarin.resource_lifecycle_state(json.dumps(RESOURCE), lifecycle),
        ),
        (
            "resource_lifecycle_state(composed)",
            "lifecycle:state:composed",
            lambda: alizarin.resource_lifecycle_state(composed(), lifecycle),
        ),
        (
            "resource_lifecycle_state(no lifecycle)",
            "lifecycle:state:none",
            lambda: alizarin.resource_lifecycle_state(json.dumps(RESOURCE), "null"),
        ),
        (
            "transition_resource_lifecycle",
            "lifecycle:transition",
            lambda: alizarin.transition_resource_lifecycle(json.dumps(RESOURCE), lifecycle, RETIRED),
        ),
        (
            "transition_resource_lifecycle(composed, non-strict)",
            "lifecycle:transition:resolve",
            lambda: alizarin.transition_resource_lifecycle(composed(), lifecycle, ACTIVE, strict=False),
        ),
    ]
    return out


@pytest.mark.parametrize("route,label,get", routes(), ids=[r[0] for r in routes()])
def test_route_gives_cores_json(route, label, get):
    assert dumped(get()) == GOLDEN[label]


def test_lifecycle_transition_is_strict_unless_asked_otherwise():
    lifecycle = json.dumps(GRAPH["resource_instance_lifecycle"])
    draft = json.loads(json.dumps(RESOURCE))
    del draft["resourceinstance"]["resource_instance_lifecycle_state_id"]  # initial: Draft
    with pytest.raises(ValueError, match="not permitted"):
        alizarin.transition_resource_lifecycle(json.dumps(draft), lifecycle, RETIRED)
    result = alizarin.transition_resource_lifecycle(json.dumps(draft), lifecycle, RETIRED, strict=False)
    assert result["event"]["to"] == RETIRED
