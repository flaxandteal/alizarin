import { test, beforeAll } from "vitest";
import { assert } from 'chai';
import { createStaticGraph, StaticNode, StaticNodegroup, StaticGraph, StaticGraphMeta } from '../js/static-types';
import { WASMResourceModelWrapper, GraphMutator, WKRM } from '../js/graphManager';
import { initWasmForTests } from './wasm-init';
import * as GroupJSON from "./data/models/Group.json";

beforeAll(async () => {
  await initWasmForTests();
});

// Helper function to create a WKRM instance for testing
function createTestWKRM(graph: StaticGraph): WKRM {
  const meta = new StaticGraphMeta({
    graphid: graph.graphid,
    name: graph.name || "Test Graph",
    slug: "test_graph",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });
  return new WKRM(meta);
}

// ============================================================================
// Core Functionality Tests
// ============================================================================

test("WASMResourceModelWrapper > constructor > should initialize with wkrm and graph", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    graph,
    true
  );

  assert.isDefined(wrapper);
  assert.equal(wrapper.wkrm.graphId, graph.graphid);
  assert.equal(wrapper.graph.graphid, graph.graphid);
});

test("WASMResourceModelWrapper > buildNodesForGraph > should populate all caches", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "test_field",
    "Test Field",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note",
    "A test field"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph,
    true
  );

  // Initially caches should be empty
  // However, these getters in Rust will populate them.
  // assert.isUndefined(wrapper.nodes);
  // assert.isUndefined(wrapper.edges);
  // assert.isUndefined(wrapper.nodegroups);
  // assert.isUndefined(wrapper.nodesByAlias);

  // Build nodes
  wrapper.buildNodesForGraph(mutatedGraph);

  // All caches should be populated
  assert.isDefined(wrapper.nodes);
  assert.isDefined(wrapper.edges);
  assert.isDefined(wrapper.nodegroups);
  assert.isDefined(wrapper.nodesByAlias);
  assert.isTrue(wrapper.nodes!.size > 0);
  assert.isTrue(wrapper.nodegroups!.size > 0);
});

test("WASMResourceModelWrapper > buildNodesForGraph > should build edges correctly", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "child1",
    "Child 1",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  mutator.addStringNode(
    null,
    "child2",
    "Child 2",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph,
    true
  );

  wrapper.buildNodesForGraph(mutatedGraph);

  // Root should have edges to both children
  const rootEdges = wrapper.edges!.get(mutatedGraph.root.nodeid);
  assert.isDefined(rootEdges);
  assert.equal(rootEdges!.length, 2);
});

test("WASMResourceModelWrapper > buildNodesForGraph > should create nodesByAlias map", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "my_alias",
    "My Field",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph,
    true
  );

  wrapper.buildNodesForGraph(mutatedGraph);

  assert.isDefined(wrapper.nodesByAlias);
  assert.isTrue(wrapper.nodesByAlias!.has("my_alias"));
  const node = wrapper.nodesByAlias!.get("my_alias");
  assert.isDefined(node);
  assert.equal(node!.alias, "my_alias");
  assert.equal(node!.name, "My Field");
});

test("WASMResourceModelWrapper > buildNodesForGraph > should error when trying to rebuild", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    graph,
    true
  );

  wrapper.buildNodesForGraph(graph);

  // Try to build again should throw
  assert.throws(() => {
    wrapper.buildNodesForGraph(graph);
  }, "Cache should never try and rebuild nodes when non-empty");
});

// ============================================================================
// getNodeObjects Tests
// ============================================================================

test("WASMResourceModelWrapper > getNodeObjects > should return all nodes", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "field1",
    "Field 1",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph,
    true
  );

  wrapper.buildNodesForGraph(mutatedGraph);
  const nodes = wrapper.getNodeObjects();

  assert.isDefined(nodes);
  assert.isTrue(nodes.size > 0);
  assert.isTrue(nodes.has(mutatedGraph.root.nodeid));
});

// ============================================================================
// getRootNode Tests
// ============================================================================

test("WASMResourceModelWrapper > getRootNode > should return root node", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    graph,
    true
  );

  wrapper.buildNodesForGraph(graph);
  const rootNode = wrapper.getRootNode();

  assert.isDefined(rootNode);
  assert.equal(rootNode.nodeid, graph.root.nodeid);
  assert.isString(rootNode.alias); // Should have alias set (empty string if not present)
});

test("WASMResourceModelWrapper > getRootNode > should ensure alias is set", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  // Set root alias to undefined
  const originalAlias = graph.root.alias;
  (graph.root as any).alias = undefined;

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    graph,
    true
  );

  wrapper.buildNodesForGraph(graph);
  const rootNode = wrapper.getRootNode();

  // getRootNode sets alias to empty string if it's undefined
  assert.isDefined(rootNode.alias);
  assert.isString(rootNode.alias);

  // Restore
  graph.root.alias = originalAlias;
});

// This test is skipped because it's testing an implementation detail that doesn't
// work with WASM encapsulation. You cannot modify the internal Rust HashMap by
// deleting from the JavaScript Map returned by the getter.
test.skip("WASMResourceModelWrapper > getRootNode > should throw if no root found", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    graph,
    true
  );

  wrapper.buildNodesForGraph(graph);

  // Manually remove root node to test error
  const rootId = graph.root.nodeid;
  wrapper.nodes!.delete(rootId);

  assert.throws(() => {
    wrapper.getRootNode();
  }, /COULD NOT FIND ROOT NODE/);
});

// ============================================================================
// createPseudoNode Tests
// ============================================================================

test("WASMResourceModelWrapper > createPseudoNode > should create pseudo node", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "test_field",
    "Test Field",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph,
    true
  );

  wrapper.buildNodesForGraph(mutatedGraph);

  const testNode = [...wrapper.nodes!.values()].find(n => n.alias === "test_field");
  assert.isDefined(testNode);

  const pseudoNode = wrapper.createPseudoNode(testNode!.alias);
  assert.isDefined(pseudoNode);
  assert.isDefined(pseudoNode.node);
  assert.equal(pseudoNode.node.alias, "test_field");
});

// ============================================================================
// Complex Graph Tests with Real Data
// ============================================================================

test("WASMResourceModelWrapper > Group model > should build nodes correctly", () => {
  const groupModel = GroupJSON["graph"][0];
  const graph = new StaticGraph(groupModel);

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    graph,
    true
  );

  wrapper.buildNodesForGraph(graph);

  // Check all caches are populated
  assert.isDefined(wrapper.nodes);
  assert.isDefined(wrapper.edges);
  assert.isDefined(wrapper.nodegroups);
  assert.isDefined(wrapper.nodesByAlias);

  // Check specific known nodes
  assert.isTrue(wrapper.nodesByAlias!.has("group"));
  assert.isTrue(wrapper.nodesByAlias!.has("basic_info"));
  assert.isTrue(wrapper.nodesByAlias!.has("name"));
  assert.isTrue(wrapper.nodesByAlias!.has("source"));
  assert.isTrue(wrapper.nodesByAlias!.has("formation"));
  assert.isTrue(wrapper.nodesByAlias!.has("formation_time"));
});

test("WASMResourceModelWrapper > Group model > should get root node correctly", () => {
  const groupModel = GroupJSON["graph"][0];
  const graph = new StaticGraph(groupModel);

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    graph,
    true
  );

  wrapper.buildNodesForGraph(graph);
  const rootNode = wrapper.getRootNode();

  assert.isDefined(rootNode);
  assert.equal(rootNode.alias, "group");
  assert.equal(rootNode.ontologyclass, "http://www.cidoc-crm.org/cidoc-crm/E74_Group");
});

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

test("WASMResourceModelWrapper > should handle graph with no edges", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    graph,
    true
  );

  wrapper.buildNodesForGraph(graph);

  assert.isDefined(wrapper.edges);
  // Root should have no edges
  const rootEdges = wrapper.edges!.get(graph.root.nodeid);
  assert.isUndefined(rootEdges);
});

test("WASMResourceModelWrapper > should handle nodes without nodegroups", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    graph,
    true
  );

  wrapper.buildNodesForGraph(graph);

  // Root node has no nodegroup (can be null or undefined)
  assert.isFalse(!!graph.root.nodegroup_id);
  assert.isDefined(wrapper.nodegroups);
});
