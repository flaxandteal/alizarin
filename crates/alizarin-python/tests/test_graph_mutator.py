"""GraphMutator runs its changes in core, matching TypeScript's on WASM and
NAPI: the same mutations give the same graph, ids included."""

import json
import os

from alizarin import GraphMutator, generate_uuid_v5
from alizarin.static_types import StaticGraph

E62 = "http://www.cidoc-crm.org/cidoc-crm/E62_String"
P3 = "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
# Core's id for node "note" on the Group graph; the TypeScript dual-backend
# test asserts the same literal, so all three bindings agree.
NOTE_NODE_ID = "9ab6dff1-dcfe-50a7-927e-55fd818ee70e"


def group_graph() -> StaticGraph:
    path = os.path.join(os.path.dirname(__file__), "../../../tests/data/models/Group.json")
    with open(path) as f:
        return StaticGraph.from_dict(json.load(f)["graph"][0])


def test_a_new_node_gets_cores_id_with_its_nodegroup_card_and_widget():
    base = group_graph()
    mutator = GraphMutator(base)
    mutator.add_string_node(None, "note", "Note", "n", E62, P3)
    graph = mutator.apply()

    note = next(n for n in graph.nodes if n.alias == "note")
    assert note.nodeid == generate_uuid_v5("graph", "node-note", base.graphid)
    assert note.nodeid == NOTE_NODE_ID
    assert note.nodegroup_id == note.nodeid, "cardinality n: its own nodegroup"
    assert any(ng.nodegroupid == note.nodeid for ng in graph.nodegroups)
    assert any(c.nodegroup_id == note.nodeid for c in graph.cards or [])
    assert any(w.node_id == note.nodeid for w in graph.cards_x_nodes_x_widgets or [])

    assert not any(n.alias == "note" for n in base.nodes), "the base is untouched"
    second = GraphMutator(base).add_string_node(None, "note", "Note", "n", E62, P3).apply()
    assert next(n for n in second.nodes if n.alias == "note").nodeid == note.nodeid


def test_a_concept_node_records_its_collection_and_list_flag():
    mutator = GraphMutator(group_graph())
    mutator.add_concept_node(None, "kinds", "Kinds", "a-collection-id", "1", E62, P3,
                             options={"is_list": True})
    kinds = next(n for n in mutator.apply().nodes if n.alias == "kinds")
    assert kinds.datatype == "concept-list"
    assert kinds.config["rdmCollection"] == "a-collection-id"
