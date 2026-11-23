"""Model navigation: NodeViewModel over core's pseudo nodes, matching
TypeScript's ResourceModelWrapper.getRoot on WASM and NAPI."""

import asyncio
import json
import os

from alizarin.graph_manager import WKRM
from alizarin.model_wrapper import ResourceModelWrapper
from alizarin.static_types import StaticGraph
from alizarin.view_models import NodeViewModel


def make_wrapper() -> ResourceModelWrapper:
    path = os.path.join(os.path.dirname(__file__), "../../../tests/data/models/Group.json")
    with open(path) as f:
        graph = StaticGraph.from_dict(json.load(f)["graph"][0])
    wkrm = WKRM({
        "graphid": graph.graphid,
        "name": {"en": "Group"},
        "slug": "group",
        "relatable_resource_model_ids": [],
    })
    return ResourceModelWrapper(wkrm, graph, True)


def test_pseudo_nodes_record_the_path_from_the_root():
    wrapper = make_wrapper()
    root = wrapper.create_pseudo_node()
    assert root.get_node_placeholder() == ".group"

    basic_info = wrapper.create_pseudo_node_child("basic_info", root)
    assert basic_info.get_node_placeholder() == ".group.basic_info"
    assert sorted(basic_info.child_node_aliases) == ["image", "name", "source"]

    members = wrapper.create_pseudo_node_child("members", root)
    assert members.is_iterable()
    assert members.get_node_placeholder() == ".group.members[*]"


def test_node_view_model_navigates_children_and_exposes_nodes_and_edges():
    async def run():
        wrapper = make_wrapper()
        root = wrapper.get_root()
        assert isinstance(root, NodeViewModel)
        assert str(root) == "group"

        basic_info = await root.basic_info
        source = await basic_info.source
        node = source._
        assert node["alias"] == "source"
        assert len(node["config"]["graphs"]) == 3

        edge = await getattr(root, "basic_info$edge")
        assert edge.domainnode_id == wrapper.get_root_node().nodeid

    asyncio.run(run())
