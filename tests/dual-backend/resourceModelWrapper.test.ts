/**
 * Dual-backend ResourceModelWrapper tests.
 *
 * These tests exercise the TS ResourceModelWrapper abstraction, which
 * delegates to either the WASM or NAPI backend depending on the active
 * setup file. They never import WASM- or NAPI-specific classes directly.
 *
 * Runs under both:
 *   npx vitest run --project wasm-dual
 *   npx vitest run --project napi
 */
import { test, describe, beforeAll } from "vitest";
import { assert } from 'chai';
import {
  createStaticGraph,
  StaticGraph,
} from '../../js/static-types';
import { ResourceModelWrapper, GraphMutator, createWKRM } from '../../js/graphManager';
import { IWKRM } from '../../js/interfaces';
import { getBackend, createStaticGraphMeta, createStaticGraph as createStaticGraphRaw } from '../../js/backend';
import * as GroupJSON from "../data/models/Group.json";

// Helper: create a WKRM for a graph (backend-agnostic)
function createTestWKRM(graph: StaticGraph): IWKRM {
  const meta = createStaticGraphMeta({
    graphid: graph.graphid,
    name: graph.name || "Test Graph",
    slug: "test_graph",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });
  return createWKRM(meta);
}

// Helper: create a simple graph with one string child node
function createGraphWithChild(alias: string = "child", name: string = "Child Node") {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });
  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    alias,
    name,
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  return { graph, mutatedGraph: mutator.apply() };
}

// Helper: wrap a graph in a ResourceModelWrapper
function wrapGraph(graph: StaticGraph, defaultAllow: boolean = true): ResourceModelWrapper<any> {
  return new ResourceModelWrapper(createTestWKRM(graph), graph, undefined, defaultAllow);
}

describe(`ResourceModelWrapper [${getBackend()} backend]`, () => {

  // =========================================================================
  // buildNodes
  // =========================================================================

  test("buildNodes > should populate nodes, edges, and nodegroups caches", () => {
    const { graph, mutatedGraph } = createGraphWithChild("test_field", "Test Field");
    const wrapper = wrapGraph(mutatedGraph);

    wrapper.buildNodes();

    assert.isDefined(wrapper.nodes);
    assert.isDefined(wrapper.edges);
    assert.isDefined(wrapper.nodegroups);
    assert.isTrue(wrapper.nodes!.size > 0);
    assert.isTrue(wrapper.nodegroups!.size > 0);
  });

  // =========================================================================
  // getNodeObjects
  // =========================================================================

  test("getNodeObjects > should auto-build nodes if not cached", () => {
    const graph = createStaticGraph({ name: "Test Graph", author: "Test Author" });
    const wrapper = wrapGraph(graph);

    const nodes = wrapper.getNodeObjects();

    assert.isDefined(nodes);
    assert.isTrue(nodes.size > 0);
    assert.isTrue(nodes.has(graph.root.nodeid));
  });

  test("getNodeObjects > should contain all graph nodes", () => {
    const { mutatedGraph } = createGraphWithChild();
    const wrapper = wrapGraph(mutatedGraph);

    const nodes = wrapper.getNodeObjects();

    // Root + child = 2 nodes minimum
    assert.isTrue(nodes.size >= 2);
  });

  // =========================================================================
  // getEdges
  // =========================================================================

  test("getEdges > should return edge map with root->child edges", () => {
    const { mutatedGraph } = createGraphWithChild();
    const wrapper = wrapGraph(mutatedGraph);

    const edges = wrapper.getEdges();

    assert.isTrue(edges.has(mutatedGraph.root.nodeid));
    const childEdges = edges.get(mutatedGraph.root.nodeid);
    assert.isDefined(childEdges);
    assert.equal(childEdges!.length, 1);
  });

  test("getEdges > should handle graph with no child edges", () => {
    const graph = createStaticGraph({ name: "Test Graph", author: "Test Author" });
    const wrapper = wrapGraph(graph);

    const edges = wrapper.getEdges();

    // Root with no children should have empty edges or undefined entry
    assert.isDefined(edges);
  });

  // =========================================================================
  // getRootNode
  // =========================================================================

  test("getRootNode > should return root node with correct ID", () => {
    const graph = createStaticGraph({ name: "Test Graph", author: "Test Author" });
    const wrapper = wrapGraph(graph);

    const rootNode = wrapper.getRootNode();

    assert.equal(rootNode.nodeid, graph.root.nodeid);
  });

  test("getRootNode > should have istopnode true", () => {
    const graph = createStaticGraph({ name: "Test Graph", author: "Test Author" });
    const wrapper = wrapGraph(graph);

    const rootNode = wrapper.getRootNode();

    assert.isTrue(rootNode.istopnode);
  });

  // =========================================================================
  // getChildNodes
  // =========================================================================

  test("getChildNodes > should return children keyed by alias", () => {
    const { mutatedGraph } = createGraphWithChild("my_child");
    const wrapper = wrapGraph(mutatedGraph);

    const childNodes = wrapper.getChildNodes(mutatedGraph.root.nodeid);

    assert.equal(childNodes.size, 1);
    assert.isTrue(childNodes.has("my_child"));
  });

  test("getChildNodes > should return empty map for leaf node", () => {
    const { mutatedGraph } = createGraphWithChild("leaf");
    const wrapper = wrapGraph(mutatedGraph);

    const leafNode = wrapper.getNodeObjectFromAlias("leaf");
    const children = wrapper.getChildNodes(leafNode.nodeid);

    assert.equal(children.size, 0);
  });

  // =========================================================================
  // getChildNodeAliases
  // =========================================================================

  test("getChildNodeAliases > should return alias strings", () => {
    const graph = createStaticGraph({ name: "Test Graph", author: "Test Author" });
    const mutator = new GraphMutator(graph);
    mutator.addStringNode(null, "alpha", "Alpha", "n",
      "http://www.w3.org/2000/01/rdf-schema#Literal",
      "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
    );
    mutator.addStringNode(null, "beta", "Beta", "n",
      "http://www.w3.org/2000/01/rdf-schema#Literal",
      "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
    );
    const mutatedGraph = mutator.apply();
    const wrapper = wrapGraph(mutatedGraph);

    const aliases = wrapper.getChildNodeAliases(mutatedGraph.root.nodeid);

    assert.isArray(aliases);
    assert.equal(aliases.length, 2);
    assert.isTrue(aliases.includes("alpha"));
    assert.isTrue(aliases.includes("beta"));
  });

  // =========================================================================
  // getNodeObjectFromAlias / getNodeObjectFromId / getNodeIdFromAlias
  // =========================================================================

  test("getNodeObjectFromAlias > should return correct node", () => {
    const { mutatedGraph } = createGraphWithChild("my_field", "My Field");
    const wrapper = wrapGraph(mutatedGraph);

    const node = wrapper.getNodeObjectFromAlias("my_field");

    assert.equal(node.alias, "my_field");
    assert.equal(node.name, "My Field");
  });

  test("getNodeObjectFromId > should return correct node", () => {
    const { mutatedGraph } = createGraphWithChild();
    const wrapper = wrapGraph(mutatedGraph);

    const rootNode = wrapper.getRootNode();
    const node = wrapper.getNodeObjectFromId(rootNode.nodeid);

    assert.equal(node.nodeid, rootNode.nodeid);
  });

  test("getNodeIdFromAlias > should return correct ID", () => {
    const { mutatedGraph } = createGraphWithChild("lookup_test");
    const wrapper = wrapGraph(mutatedGraph);

    const nodeId = wrapper.getNodeIdFromAlias("lookup_test");
    const node = wrapper.getNodeObjectFromId(nodeId);

    assert.equal(node.alias, "lookup_test");
  });

  // =========================================================================
  // Nodegroup accessors
  // =========================================================================

  test("getNodegroupObjects > should return nodegroups", () => {
    const { mutatedGraph } = createGraphWithChild();
    const wrapper = wrapGraph(mutatedGraph);

    const nodegroups = wrapper.getNodegroupObjects();

    assert.isDefined(nodegroups);
    assert.isTrue(nodegroups.size > 0);
  });

  test("getNodegroupIds > should return string array", () => {
    const { mutatedGraph } = createGraphWithChild();
    const wrapper = wrapGraph(mutatedGraph);

    const ids = wrapper.getNodegroupIds();

    assert.isArray(ids);
    assert.isTrue(ids.length > 0);
    ids.forEach(id => assert.isString(id));
  });

  test("getNodegroupName > should return name string", () => {
    const { mutatedGraph } = createGraphWithChild("named_ng", "Named Nodegroup");
    const wrapper = wrapGraph(mutatedGraph);

    const node = wrapper.getNodeObjectFromAlias("named_ng");
    const name = wrapper.getNodegroupName(node.nodegroup_id!);

    assert.isString(name);
  });

  // =========================================================================
  // Permissions
  // =========================================================================

  test("getPermittedNodegroups > should default to all permitted with defaultAllow=true", () => {
    const { mutatedGraph } = createGraphWithChild();
    const wrapper = wrapGraph(mutatedGraph, true);

    const permissions = wrapper.getPermittedNodegroups();

    assert.isTrue(permissions.size >= 1);
    // All should be true with default allow
    for (const [, value] of permissions) {
      assert.isTrue(value === true);
    }
  });

  test("setPermittedNodegroups > should set and check permissions", () => {
    const { mutatedGraph } = createGraphWithChild("field");
    const wrapper = wrapGraph(mutatedGraph, true);

    const nodes = wrapper.getNodeObjects();
    const fieldNode = [...nodes.values()].find(n => n.alias === "field")!;

    const permissions = new Map<string | null, boolean>();
    permissions.set("", true);
    permissions.set(fieldNode.nodegroup_id || "", false);

    wrapper.setPermittedNodegroups(permissions);

    assert.isFalse(wrapper.isNodegroupPermitted(fieldNode.nodegroup_id || "", null));
  });

  test("isNodegroupPermitted > should deny after explicit setPermittedNodegroups with false", () => {
    const { mutatedGraph } = createGraphWithChild("denied_field");
    const wrapper = wrapGraph(mutatedGraph, false);

    const nodes = wrapper.getNodeObjects();
    const childNode = [...nodes.values()].find(n => n.alias === "denied_field")!;

    // Explicitly set the nodegroup as not permitted
    const permissions = new Map<string | null, boolean>();
    permissions.set("", true); // root
    permissions.set(childNode.nodegroup_id!, false);
    wrapper.setPermittedNodegroups(permissions);

    assert.isFalse(wrapper.isNodegroupPermitted(childNode.nodegroup_id!, null));
  });

  // =========================================================================
  // Graph pruning
  // =========================================================================

  test("pruneGraph > should remove unpermitted nodegroups", () => {
    const graph = createStaticGraph({ name: "Test Graph", author: "Test Author" });
    const mutator = new GraphMutator(graph);
    mutator.addStringNode(null, "keep", "Keep Me", "n",
      "http://www.w3.org/2000/01/rdf-schema#Literal",
      "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
    );
    mutator.addStringNode(null, "remove", "Remove Me", "n",
      "http://www.w3.org/2000/01/rdf-schema#Literal",
      "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
    );
    const mutatedGraph = mutator.apply();
    const wrapper = wrapGraph(mutatedGraph, false);

    const nodes = wrapper.getNodeObjects();
    const keepNode = [...nodes.values()].find(n => n.alias === "keep")!;
    const removeNode = [...nodes.values()].find(n => n.alias === "remove")!;

    const permissions = new Map<string | null, boolean>();
    permissions.set("", true); // root always permitted
    permissions.set(keepNode.nodegroup_id!, true);
    permissions.set(removeNode.nodegroup_id!, false);
    wrapper.setPermittedNodegroups(permissions);

    wrapper.pruneGraph();

    // After pruning, only "keep" should remain (plus root)
    const prunedNodes = wrapper.getNodeObjects();
    const prunedAliases = [...prunedNodes.values()].map(n => n.alias);
    assert.isTrue(prunedAliases.includes("keep"));
    assert.isFalse(prunedAliases.includes("remove"));
  });

  // =========================================================================
  // Real data: Group model
  // =========================================================================

  test("Group model > should build and query nodes correctly", () => {
    const groupModel = (GroupJSON as any)["graph"][0];
    const graph = createStaticGraphRaw(groupModel);
    const wrapper = wrapGraph(graph);

    // Verify key aliases exist
    assert.isDefined(wrapper.getNodeObjectFromAlias("group"));
    assert.isDefined(wrapper.getNodeObjectFromAlias("basic_info"));
    assert.isDefined(wrapper.getNodeObjectFromAlias("name"));

    // Root should be "group"
    const root = wrapper.getRootNode();
    assert.equal(root.alias, "group");
  });

  test("Group model > should return correct child nodes for root", () => {
    const groupModel = (GroupJSON as any)["graph"][0];
    const graph = createStaticGraphRaw(groupModel);
    const wrapper = wrapGraph(graph);

    const root = wrapper.getRootNode();
    const children = wrapper.getChildNodes(root.nodeid);

    // Group root has several children
    assert.isTrue(children.size > 0);
    // "basic_info" is a known direct child
    assert.isTrue(children.has("basic_info"));
  });

  // =========================================================================
  // GraphMutator
  // =========================================================================

  test("GraphMutator > addStringNode > should add node to graph", () => {
    const graph = createStaticGraph({ name: "Test Graph", author: "Test Author" });
    const mutator = new GraphMutator(graph);

    mutator.addStringNode(
      null,
      "new_field",
      "New Field",
      "1",
      "http://www.w3.org/2000/01/rdf-schema#Literal",
      "http://www.cidoc-crm.org/cidoc-crm/P3_has_note",
      "A description"
    );

    const result = mutator.apply();

    const hasNewField = result.nodes.some(n => n.alias === "new_field");
    assert.isTrue(hasNewField);
  });

  test("GraphMutator > addSemanticNode > should add semantic node to graph", () => {
    const graph = createStaticGraph({ name: "Test Graph", author: "Test Author" });
    const mutator = new GraphMutator(graph);

    mutator.addSemanticNode(
      null,
      "semantic_group",
      "Semantic Group",
      "n",
      "http://www.cidoc-crm.org/cidoc-crm/E1_CRM_Entity",
      "http://www.cidoc-crm.org/cidoc-crm/P2_has_type"
    );

    const result = mutator.apply();

    const semanticNode = result.nodes.find(n => n.alias === "semantic_group");
    assert.isDefined(semanticNode);
    assert.equal(semanticNode!.datatype, "semantic");
  });

  test("GraphMutator > nested nodes > should create parent-child edges", () => {
    const graph = createStaticGraph({ name: "Test Graph", author: "Test Author" });
    const mutator = new GraphMutator(graph);

    mutator.addSemanticNode(
      null,
      "parent",
      "Parent",
      "n",
      "http://www.cidoc-crm.org/cidoc-crm/E1_CRM_Entity",
      "http://www.cidoc-crm.org/cidoc-crm/P2_has_type"
    );
    mutator.addStringNode(
      "parent",
      "nested_child",
      "Nested Child",
      "1",
      "http://www.w3.org/2000/01/rdf-schema#Literal",
      "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
    );

    const result = mutator.apply();
    const wrapper = wrapGraph(result);

    const parentNode = wrapper.getNodeObjectFromAlias("parent");
    const children = wrapper.getChildNodes(parentNode.nodeid);

    assert.isTrue(children.has("nested_child"));
  });

  // =========================================================================
  // WKRM (via createWKRM)
  // =========================================================================

  test("createWKRM > should create WKRM with correct properties", () => {
    const meta = createStaticGraphMeta({
      graphid: "test-graph-123",
      name: "Test Graph",
      slug: "test_graph",
      relatable_resource_model_ids: [],
      resource_2_resource_constraints: [],
      extra_fields: {}
    });

    const wkrm = createWKRM(meta);

    assert.equal(wkrm.graphId, "test-graph-123");
    assert.equal(wkrm.modelName, "Test Graph");
    assert.equal(wkrm.modelClassName, "TestGraph");
  });

  test("createWKRM > should convert slug with underscores to PascalCase", () => {
    const meta = createStaticGraphMeta({
      graphid: "test-graph-123",
      name: "Test Graph",
      slug: "historical_event",
      relatable_resource_model_ids: [],
      resource_2_resource_constraints: [],
      extra_fields: {}
    });

    const wkrm = createWKRM(meta);

    assert.equal(wkrm.modelClassName, "HistoricalEvent");
  });
});
