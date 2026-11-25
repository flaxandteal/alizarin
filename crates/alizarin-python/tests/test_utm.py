"""
UTM through the derive-provider bridge, end to end, as
tests/dual-backend/utm.test.ts checks on WASM and NAPI: importing alizarin_geo
registers its UTM provider (a capsule over its DeriveProviderInfo), a graph
declares it in stored mode with named nodes, and editing the geometry through
the ORM reruns it on commit(), returning the derived tile in the delta.
"""

import json
import os

import pytest

import alizarin
import alizarin_geo  # noqa: F401  (registers its handler and the UTM provider)
from alizarin import GraphMutator
from alizarin.graph_manager import GraphManager
from alizarin.static_types import StaticGraph
from alizarin.view_models import viewContext

CRM = "http://www.cidoc-crm.org/cidoc-crm/"
COMPUTE_TILES = "60000000-0000-0000-0000-000000000002"
RESOURCE_ID = "ffffffff-1111-2222-3333-444444444444"

# London (51.5074°N, 0.1278°W): zone 30N, easting ≈ 699 320 m.
LONDON = {
    "type": "FeatureCollection",
    "features": [{
        "type": "Feature",
        "properties": {},
        "geometry": {"type": "Point", "coordinates": [-0.1278, 51.5074]},
    }],
}


def _place(role, datatype):
    """A place with a `location` geometry and a node for UTM's `role` output,
    of `datatype`, declared in stored mode, with no tiles yet."""
    path = os.path.join(os.path.dirname(__file__), "../../../tests/data/models/Group.json")
    with open(path) as f:
        base = StaticGraph.from_dict(json.load(f)["graph"][0])
    mutator = GraphMutator(base)
    mutator._add_node(None, "location", "Location", "1", "geojson-feature-collection",
                      f"{CRM}E53_Place", f"{CRM}P168_place_is_defined_by", None, None, None)
    mutator._add_node(None, role, role.title(), "1", datatype,
                      f"{CRM}E54_Dimension", f"{CRM}P43_has_dimension", None, None, None)
    graph = mutator.apply().to_dict()
    nodes = {n["alias"]: n for n in graph["nodes"] if n.get("alias")}
    graph["functions_x_graphs"] = (graph.get("functions_x_graphs") or []) + [{
        "id": "fxg-utm",
        "function_id": COMPUTE_TILES,
        "graph_id": graph["graphid"],
        "config": {
            "provider": "utm",
            "nodegroup": nodes[role]["nodegroup_id"],
            "mode": "stored",
            "sources": [nodes["location"]["nodegroup_id"]],
            "nodes": {"geometry": "location", role: role},
        },
    }]

    gm = GraphManager(default_allow_all_nodegroups=True)
    gm.register_graph(graph)
    gm.register_resource({
        "resourceinstance": {
            "resourceinstanceid": RESOURCE_ID,
            "graph_id": graph["graphid"],
            "name": "Town",
        },
        "tiles": [],
    })
    viewContext.graphManager = gm
    return gm, nodes


@pytest.fixture
def place():
    """A place whose `easting` (a number) UTM fills."""
    yield _place("easting", "number")
    viewContext.graphManager = None


@pytest.fixture
def irish_place():
    """A place whose `hemisphere` (a translatable string) UTM fills, with the
    host's language Irish."""
    previous = alizarin.get_current_language()
    alizarin.set_current_language("ga")
    yield _place("hemisphere", "string")
    alizarin.set_current_language(previous)
    viewContext.graphManager = None


def test_importing_alizarin_geo_registers_the_utm_provider():
    assert alizarin.has_derive_provider("utm")


@pytest.mark.asyncio
async def test_editing_the_geometry_reruns_utm_in_the_commit(place):
    gm, nodes = place
    town = await gm.getResource(RESOURCE_ID, lazy=False)
    town.location = LONDON
    delta = town.commit()

    easting_ng = nodes["easting"]["nodegroup_id"]
    easting = next(t for t in delta["upserted"] if t["nodegroup_id"] == easting_ng)
    assert abs(easting["data"][nodes["easting"]["nodeid"]] - 699_320) < 50
    # Stored with the resource, so it reads back like any value.
    assert abs(float(await town.easting) - 699_320) < 50


@pytest.mark.asyncio
async def test_clearing_the_geometry_removes_the_derived_tile(place):
    gm, _ = place
    town = await gm.getResource(RESOURCE_ID, lazy=False)
    town.location = LONDON
    town.commit()
    town.location = None
    delta = town.commit()
    assert len(delta["removed"]) == 1


@pytest.mark.asyncio
async def test_a_derived_string_is_in_the_hosts_language(irish_place):
    # The provider runs in geo's own module; the value is still keyed by the
    # language set here, in the host.
    gm, nodes = irish_place
    town = await gm.getResource(RESOURCE_ID, lazy=False)
    town.location = LONDON
    delta = town.commit()

    hemisphere_ng = nodes["hemisphere"]["nodegroup_id"]
    tile = next(t for t in delta["upserted"] if t["nodegroup_id"] == hemisphere_ng)
    assert tile["data"][nodes["hemisphere"]["nodeid"]] == {"ga": {"value": "N", "direction": "ltr"}}
