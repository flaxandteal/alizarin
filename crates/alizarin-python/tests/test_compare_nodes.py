"""compare_nodes: whether two nodes are the same apart from their ids, by core,
as WASM's StaticNode.compare and NAPI's compareNodes
(tests/dual-backend/compareNodes.test.ts, the same cases)."""

import pytest

import alizarin

NODE = {
    "nodeid": "n1", "graph_id": "g1", "nodegroup_id": "ng1",
    "alias": "name", "datatype": "string", "config": {}, "name": "Name",
}


@pytest.mark.parametrize(
    "other, expected",
    [
        (NODE, "same"),
        ({**NODE, "nodeid": "n2"}, "nodeid_conflict"),
        ({**NODE, "nodeid": "n2", "nodegroup_id": "ng2"}, "nodegroup_id_conflict"),
        ({**NODE, "nodegroup_id": "ng2", "graph_id": "g2"}, "graph_id_conflict"),
        ({**NODE, "datatype": "number"}, "different"),
        ({"alias": "name", "datatype": "string", "config": None, "name": "Name"}, "same_ids_partial"),
    ],
)
def test_compare_nodes(other, expected):
    assert alizarin.compare_nodes(NODE, other) == expected
