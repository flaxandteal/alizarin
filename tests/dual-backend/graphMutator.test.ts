/**
 * GraphMutator runs its changes in core on every backend, so the same
 * mutations give the same ids on WASM and NAPI, and in Python, whose test
 * (crates/alizarin-python/tests/test_graph_mutator.py) asserts the same id.
 */
import { describe, expect, test } from "vitest";
import { GraphMutator } from "../../js/graphManager";
import { getBackend, parseStaticGraph } from "../../js/backend";
import * as GroupJSON from "../data/models/Group.json";

// Core's id for node "note" on the Group graph.
const NOTE_NODE_ID = "9ab6dff1-dcfe-50a7-927e-55fd818ee70e";

describe(`GraphMutator [${getBackend()}]`, () => {
  test("a new node gets core's id, the same on every backend", () => {
    const base = parseStaticGraph(JSON.stringify(GroupJSON));
    const mutator = new GraphMutator(base);
    mutator.addStringNode(
      null,
      "note",
      "Note",
      "n",
      "http://www.cidoc-crm.org/cidoc-crm/E62_String",
      "http://www.cidoc-crm.org/cidoc-crm/P3_has_note",
    );
    const graph: any = mutator.apply();
    const nodes: any[] = [...graph.nodes];
    const note = nodes.find((node) => node.alias === "note");
    expect(note?.nodeid).toBe(NOTE_NODE_ID);
    expect(note?.nodegroup_id).toBe(NOTE_NODE_ID);
  });
});
