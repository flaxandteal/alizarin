import { test, beforeAll } from "vitest";
import { assert } from 'chai';
import { createStaticGraph, StaticCollection, StaticConcept, StaticNode, StaticNodegroup, StaticGraph, StaticGraphMeta } from '../js/static-types';
import { ResourceModelWrapper, GraphMutator, WKRM } from '../js/graphManager';
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

test("ResourceModelWrapper > buildNodes > should cache nodes, edges, and nodegroups", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  // Add a node to the graph
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

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  // Initially caches should be empty
  // except that Rust builds them on access

  // Build nodes
  wrapper.buildNodes();

  // Caches should be populated
  assert.isDefined(wrapper.nodes);
  assert.isDefined(wrapper.edges);
  assert.isDefined(wrapper.nodegroups);
  assert.isTrue(wrapper.nodes!.size > 0);
  assert.isTrue(wrapper.edges!.size > 0);
  assert.isTrue(wrapper.nodegroups!.size > 0);
});

test("ResourceModelWrapper > getNodeObjects > should build nodes if not cached", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    graph
  );

  const nodes = wrapper.getNodeObjects();

  assert.isDefined(wrapper.nodes);
  assert.isTrue(nodes.size > 0);
  assert.isTrue(nodes.has(graph.root.nodeid));
});

test("ResourceModelWrapper > getEdges > should return edge map", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "child",
    "Child Node",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  const edges = wrapper.getEdges();

  // Should have edge from root to child
  assert.isTrue(edges.has(mutatedGraph.root.nodeid));
  const childEdges = edges.get(mutatedGraph.root.nodeid);
  assert.isDefined(childEdges);
  assert.equal(childEdges!.length, 1);
});

test("ResourceModelWrapper > getChildNodes > should return child nodes", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "child",
    "Child Node",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  const childNodes = wrapper.getChildNodes(mutatedGraph.root.nodeid);

  assert.equal(childNodes.size, 1);
  assert.isTrue(childNodes.has("child"));
});

test("ResourceModelWrapper > getRootNode > should return root node", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    graph
  );

  const rootNode = wrapper.getRootNode();

  assert.equal(rootNode.nodeid, graph.root.nodeid);
  assert.isFalse(!!rootNode.nodegroup_id);
});

test("ResourceModelWrapper > getCollections > should return unique collection IDs", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const collection = StaticCollection.fromConceptScheme({
    conceptScheme: StaticConcept.fromValue(
      null,
      "Test Concept",
      ["Option A", "Option B"]
    )
  });

  const mutator = new GraphMutator(graph);
  mutator.addConceptNode(
    null,
    "concept1",
    "Concept Field 1",
    collection,
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  mutator.addConceptNode(
    null,
    "concept2",
    "Concept Field 2",
    collection,
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  const collections = wrapper.getCollections();

  assert.equal(collections.length, 1);
  assert.equal(collections[0], collection.id);
});

test("ResourceModelWrapper > getCollections > should filter by accessible when specified", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const collection = StaticCollection.fromConceptScheme({
    conceptScheme: StaticConcept.fromValue(
      null,
      "Test Concept",
      ["Option A", "Option B"]
    )
  });

  const mutator = new GraphMutator(graph);
  mutator.addConceptNode(
    null,
    "accessible",
    "Accessible Concept",
    collection,
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  mutator.addConceptNode(
    null,
    "restricted",
    "Restricted Concept",
    collection,
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  // Get nodegroup IDs
  const nodes = wrapper.getNodeObjects();
  const accessibleNode = [...nodes.values()].find(n => n.alias === "accessible");
  const restrictedNode = [...nodes.values()].find(n => n.alias === "restricted");

  // Set permissions - only allow accessible node
  const permissions = new Map<string | null, boolean>();
  permissions.set("", true); // root
  permissions.set(accessibleNode!.nodegroup_id || "", true);
  permissions.set(restrictedNode!.nodegroup_id || "", false);
  wrapper.setPermittedNodegroups(permissions);

  const collectionsAll = wrapper.getCollections(false);
  const collectionsAccessible = wrapper.getCollections(true);

  assert.equal(collectionsAll.length, 1);
  assert.equal(collectionsAccessible.length, 1); // Still 1 because same collection
});

test("ResourceModelWrapper > getBranchPublicationIds > should return unique branch publication IDs", () => {
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

  // Create a new node with sourcebranchpublication_id set
  const nodeToUpdate = mutatedGraph.nodes.find(n => n.alias === "field1");
  if (nodeToUpdate) {
    const updatedNode = new StaticNode({
      alias: nodeToUpdate.alias,
      config: nodeToUpdate.config,
      datatype: nodeToUpdate.datatype,
      description: nodeToUpdate.description,
      exportable: nodeToUpdate.exportable,
      fieldname: nodeToUpdate.fieldname,
      graph_id: nodeToUpdate.graph_id,
      hascustomalias: nodeToUpdate.hascustomalias,
      is_collector: nodeToUpdate.is_collector,
      isrequired: nodeToUpdate.isrequired,
      issearchable: nodeToUpdate.issearchable,
      istopnode: nodeToUpdate.istopnode,
      name: nodeToUpdate.name,
      nodegroup_id: nodeToUpdate.nodegroup_id,
      nodeid: nodeToUpdate.nodeid,
      ontologyclass: nodeToUpdate.ontologyclass,
      parentproperty: nodeToUpdate.parentproperty,
      sortorder: nodeToUpdate.sortorder,
      sourcebranchpublication_id: "branch-pub-123"
    });

    // Replace the node in the graph
    mutatedGraph.nodes = mutatedGraph.nodes.map(n =>
      n.alias === "field1" ? updatedNode : n
    );
  }

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  const branchPubIds = wrapper.getBranchPublicationIds();

  assert.equal(branchPubIds.length, 1);
  assert.equal(branchPubIds[0], "branch-pub-123");
});

test("ResourceModelWrapper > getPermittedNodegroups > should default to all permitted", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "field",
    "Field",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  const permissions = wrapper.getPermittedNodegroups();

  // Should have permissions for root and the field's nodegroup
  assert.isTrue(permissions.size >= 2);
  assert.isTrue(permissions.get("") === true); // root
});

test("ResourceModelWrapper > setPermittedNodegroups > should set permissions by nodegroup ID", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "field",
    "Field",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  const nodes = wrapper.getNodeObjects();
  const fieldNode = [...nodes.values()].find(n => n.alias === "field");

  const permissions = new Map<string | null, boolean>();
  permissions.set("", true);
  permissions.set(fieldNode!.nodegroup_id || "", false);

  wrapper.setPermittedNodegroups(permissions);

  assert.isFalse(wrapper.isNodegroupPermitted(fieldNode!.nodegroup_id || "", null));
});

test("ResourceModelWrapper > setPermittedNodegroups > should accept node alias for semantic nodes", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addSemanticNode(
    null,
    "semantic",
    "Semantic Node",
    "n",
    "http://www.cidoc-crm.org/cidoc-crm/E1_CRM_Entity",
    "http://www.cidoc-crm.org/cidoc-crm/P2_has_type"
  );
  mutator.addStringNode(
    "semantic",
    "child",
    "Child Field",
    "1",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  // Set permission using node alias instead of nodegroup ID
  const permissions = new Map<string | null, boolean>();
  permissions.set("", true);
  permissions.set("semantic", false); // Use alias

  wrapper.setPermittedNodegroups(permissions);

  const nodes = wrapper.getNodeObjects();
  const semanticNode = [...nodes.values()].find(n => n.alias === "semantic");

  // The child's nodegroup_id is the semantic node's nodeid
  assert.isFalse(wrapper.isNodegroupPermitted(semanticNode!.nodeid, null));
});

test("ResourceModelWrapper > isNodegroupPermitted > should support function-based permissions", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "field",
    "Field",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  const nodes = wrapper.getNodeObjects();
  const fieldNode = [...nodes.values()].find(n => n.alias === "field");

  const permissions = new Map<string | null, boolean | ((nodegroupId: string, tile: any, nodes: Map<string, StaticNode>) => boolean)>();
  permissions.set("", true);

  // Use a function that always returns false
  permissions.set(fieldNode!.nodegroup_id || "", (nodegroupId, tile, nodes) => {
    return false;
  });

  wrapper.setPermittedNodegroups(permissions);

  assert.isFalse(wrapper.isNodegroupPermitted(fieldNode!.nodegroup_id || "", null));
});

test("ResourceModelWrapper > pruneGraph > should remove unpermitted nodes", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addStringNode(
    null,
    "keep",
    "Keep Field",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  mutator.addStringNode(
    null,
    "remove",
    "Remove Field",
    "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  const nodes = wrapper.getNodeObjects();
  const keepNode = [...nodes.values()].find(n => n.alias === "keep");
  const removeNode = [...nodes.values()].find(n => n.alias === "remove");

  const permissions = new Map<string | null, boolean>();
  permissions.set("", true);
  permissions.set(keepNode!.nodegroup_id || "", true);
  permissions.set(removeNode!.nodegroup_id || "", false);

  wrapper.setPermittedNodegroups(permissions);

  const nodeCountBefore = wrapper.graph.nodes.length;
  wrapper.pruneGraph();
  const nodeCountAfter = wrapper.graph.nodes.length;

  // Should have removed the "remove" node
  assert.isTrue(nodeCountAfter < nodeCountBefore);
  assert.isTrue(wrapper.graph.nodes.some(n => n.alias === "keep"));
  assert.isFalse(wrapper.graph.nodes.some(n => n.alias === "remove"));
});

test("ResourceModelWrapper > pruneGraph > should preserve parent nodes of permitted children", () => {
  const graph = createStaticGraph({
    name: "Test Graph",
    author: "Test Author",
  });

  const mutator = new GraphMutator(graph);
  mutator.addSemanticNode(
    null,
    "parent",
    "Parent Node",
    "n",
    "http://www.cidoc-crm.org/cidoc-crm/E1_CRM_Entity",
    "http://www.cidoc-crm.org/cidoc-crm/P2_has_type"
  );
  mutator.addStringNode(
    "parent",
    "child",
    "Child Field",
    "1",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
  );
  const mutatedGraph = mutator.apply();

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    mutatedGraph
  );

  const nodes = wrapper.getNodeObjects();
  const parentNode = [...nodes.values()].find(n => n.alias === "parent");
  const childNode = [...nodes.values()].find(n => n.alias === "child");

  // Only permit the child, not the parent nodegroup
  const permissions = new Map<string | null, boolean>();
  permissions.set("", true);
  permissions.set(childNode!.nodegroup_id || "", true);

  wrapper.setPermittedNodegroups(permissions);

  wrapper.pruneGraph();

  // Parent should still exist because child is permitted
  assert.isTrue(wrapper.graph.nodes.some(n => n.alias === "parent"));
  assert.isTrue(wrapper.graph.nodes.some(n => n.alias === "child"));
});

// TODO: exportGraph has a bug - it tries to pass WASM array instances directly to StaticGraph constructor
// which causes "Reflect.get called on non-object" error. This needs to be fixed by serializing
// the arrays to plain JS objects first, or by using a different approach.
// test("ResourceModelWrapper > exportGraph > should create new StaticGraph with same properties", () => {
//   const graph = createStaticGraph({
//     name: "Test Graph",
//     author: "Test Author",
//   });
//
//   const wrapper = new ResourceModelWrapper(
//     createTestWKRM(graph),
//     graph
//   );
//
//   const exported = wrapper.exportGraph();
//
//   // Should have same core properties
//   assert.equal(exported.graphid, wrapper.graph.graphid);
//   assert.isDefined(exported.name);
//   assert.isDefined(exported.author);
// });

test("ResourceModelWrapper > getRoot > should allow navigation through NodeViewModel", async () => {
  // Load Group model from test data
  const groupModel = GroupJSON["graph"][0];
  const graph = new StaticGraph(groupModel);

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    graph
  );

  // Get root NodeViewModel
  const root = wrapper.getRoot();
  assert.isDefined(root);
  assert.isDefined(root.__parentPseudo);
  assert.isDefined(root.__parentWkrm);

  // Test navigation doesn't throw
  // Note: The navigation returns AttrPromises that resolve to pseudo nodes
  assert.isDefined(root.basic_info);

  // Verify the proxy returns AttrPromise for non-existent properties
  const basicInfo = root.basic_info;
  assert.isDefined(basicInfo);

  // Verify toString works
  const rootStr = await root.toString();
  assert.equal(rootStr, "group");
});

test("ResourceModelWrapper > NodeViewModel > should allow access to node properties via ._", async () => {
  // Load Group model from test data
  const groupModel = GroupJSON["graph"][0];
  const graph = new StaticGraph(groupModel);

  const wrapper = new ResourceModelWrapper(
    createTestWKRM(graph),
    graph
  );

  // Get root NodeViewModel
  const root = wrapper.getRoot();

  // Test accessing the source node's config via ._.config
  const sourceNode = await root.basic_info.source._;
  assert.isDefined(sourceNode);
  assert.equal(sourceNode.alias, "source");
  assert.isDefined(sourceNode.config);
  assert.isDefined(sourceNode.config.graphs);
  assert.isArray(sourceNode.config.graphs);
  assert.equal(sourceNode.config.graphs.length, 3);

  // Test accessing the formation_time node's ontologyclass via ._.ontologyclass
  const formationTimeNode = await root.formation.formation_time._;
  assert.isDefined(formationTimeNode);
  assert.equal(formationTimeNode.alias, "formation_time");
  assert.equal(formationTimeNode.ontologyclass, "http://www.cidoc-crm.org/cidoc-crm/E52_Time-Span");

  const formationEdge = await root.formation$edge;
  assert.isDefined(formationEdge);
  assert.equal(formationEdge.ontologyproperty, "http://www.cidoc-crm.org/cidoc-crm/P95i_was_formed_by");
});
