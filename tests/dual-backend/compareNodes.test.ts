/**
 * compareNodes: whether two nodes are the same apart from their ids, by core
 * on either backend (WASM's StaticNode.compare, NAPI's compareNodes). Python:
 * crates/alizarin-python/tests/test_compare_nodes.py, the same cases.
 *
 * Runs under both backends via:
 *   npx vitest run tests/dual-backend/compareNodes.test.ts (WASM)
 *   npx vitest run -c vitest.napi.config.js tests/dual-backend/compareNodes.test.ts (NAPI)
 */
import { describe, expect, test } from "vitest";
import { compareNodes, getBackend } from "../../js/backend";

const node = {
  nodeid: "n1", graph_id: "g1", nodegroup_id: "ng1",
  alias: "name", datatype: "string", config: {}, name: "Name",
};

describe(`compareNodes [${getBackend()} backend]`, () => {
  test.each([
    ["the same node", node, "same"],
    ["another node id", { ...node, nodeid: "n2" }, "nodeid_conflict"],
    ["another nodegroup", { ...node, nodeid: "n2", nodegroup_id: "ng2" }, "nodegroup_id_conflict"],
    ["another graph", { ...node, nodegroup_id: "ng2", graph_id: "g2" }, "graph_id_conflict"],
    ["another datatype", { ...node, datatype: "number" }, "different"],
    ["a partial node without ids", { alias: "name", datatype: "string", config: null, name: "Name" }, "same_ids_partial"],
  ])("%s", (_, other, expected) => {
    expect(compareNodes(node, other)).toBe(expected);
  });
});
