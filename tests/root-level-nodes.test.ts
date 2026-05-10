/**
 * Regression tests: root-level nodes (direct children of root in their own nodegroup)
 * should resolve correctly via dot-access.
 *
 * Bug: matches_semantic_child returned false when parent_tile_id was None (root node)
 * and child was in a different nodegroup (non-collector). This prevented all direct
 * root-level data fields from being accessed via dot-access or SemanticViewModel.
 *
 * Variations tested:
 * 1. Single cardinality-1 string node under root
 * 2. Multiple cardinality-1 nodes under root (different datatypes)
 * 3. Cardinality-N node under root (collector)
 * 4. Nested node (under a semantic parent, not root) — should still work
 */
import { test, expect } from "vitest";
import {
  parseStaticGraph,
  parseStaticResources,
  createStaticGraphMeta,
  createWKRM,
} from '../js/backend';
import { ResourceModelWrapper, GraphMutator } from '../js/graphManager';
import { ResourceInstanceViewModel } from '../js/viewModels/ResourceInstanceViewModel';
import type { ResourceInstanceWrapper } from '../js/graphManager';
import { createStaticGraph } from '../js/static-types';

function makeModelWrapper(graph: any) {
  const meta = createStaticGraphMeta({
    graphid: graph.graphid,
    name: graph.name || "Test",
    slug: "test",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {},
  });
  const wkrm = createWKRM(meta);

  class TestModel extends ResourceInstanceViewModel<any> {
    static _: ResourceInstanceWrapper<any> | null;
    static __: ResourceModelWrapper<any> | null;
  }

  const wrapper = new ResourceModelWrapper(wkrm, graph, TestModel, true);
  TestModel.prototype.__ = wrapper;
  return wrapper;
}

function makeBusinessData(graphId: string, resourceId: string, tiles: any[]) {
  return {
    business_data: {
      resources: [{
        resourceinstance: {
          descriptors: { name: "Test", description: "", map_popup: "", slug: "test-1" },
          graph_id: graphId,
          legacyid: null,
          name: "Test",
          resourceinstanceid: resourceId,
        },
        tiles,
        tiles_loaded: true,
      }]
    }
  };
}

test("Root-level cardinality-1 string node resolves via dot-access", async () => {
  // Build a minimal graph with one string node under root
  const graph = createStaticGraph({ name: "Test Graph", author: "Test" });
  const mutator = new GraphMutator(graph);
  mutator.addStringNode(null, "simple_field", "Simple Field", "1",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note");
  const mutatedGraph = mutator.apply();

  const parsedGraph = parseStaticGraph(JSON.stringify({ graph: [mutatedGraph] }));
  const modelWrapper = makeModelWrapper(parsedGraph);

  // Find the generated nodegroup ID for simple_field
  const nodes = modelWrapper.getNodeObjectsByAlias();
  const simpleFieldNode = nodes.get("simple_field");
  expect(simpleFieldNode).toBeDefined();
  const ngId = simpleFieldNode.nodegroup_id;

  // Create resource data with a tile for that nodegroup
  const resourceId = "aaaaaaaa-1111-2222-3333-444444444444";
  const businessData = makeBusinessData(parsedGraph.graphid, resourceId, [{
    data: { [simpleFieldNode.nodeid]: { en: { direction: "ltr", value: "hello world" } } },
    nodegroup_id: ngId,
    parenttile_id: null,
    resourceinstance_id: resourceId,
    sortorder: 0,
    tileid: "bbbbbbbb-1111-2222-3333-444444444444",
  }]);

  const staticResources = parseStaticResources(JSON.stringify(businessData));
  const asset = await modelWrapper.fromStaticResource(staticResources[0], false, false);

  const value = await (asset as any).simple_field;
  expect(value).not.toBeNull();
  expect(value).not.toBeUndefined();
  expect(value.toString()).toBe("hello world");
});

test("Multiple root-level cardinality-1 nodes all resolve", async () => {
  const graph = createStaticGraph({ name: "Multi Graph", author: "Test" });
  const mutator = new GraphMutator(graph);
  mutator.addStringNode(null, "field_a", "Field A", "1",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note");
  mutator.addStringNode(null, "field_b", "Field B", "1",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note");
  const mutatedGraph = mutator.apply();

  const parsedGraph = parseStaticGraph(JSON.stringify({ graph: [mutatedGraph] }));
  const modelWrapper = makeModelWrapper(parsedGraph);

  const nodes = modelWrapper.getNodeObjectsByAlias();
  const nodeA = nodes.get("field_a");
  const nodeB = nodes.get("field_b");

  const resourceId = "cccccccc-1111-2222-3333-444444444444";
  const businessData = makeBusinessData(parsedGraph.graphid, resourceId, [
    {
      data: { [nodeA.nodeid]: { en: { direction: "ltr", value: "alpha" } } },
      nodegroup_id: nodeA.nodegroup_id,
      parenttile_id: null,
      resourceinstance_id: resourceId,
      sortorder: 0,
      tileid: "dddddddd-1111-2222-3333-444444444444",
    },
    {
      data: { [nodeB.nodeid]: { en: { direction: "ltr", value: "beta" } } },
      nodegroup_id: nodeB.nodegroup_id,
      parenttile_id: null,
      resourceinstance_id: resourceId,
      sortorder: 0,
      tileid: "eeeeeeee-1111-2222-3333-444444444444",
    },
  ]);

  const staticResources = parseStaticResources(JSON.stringify(businessData));
  const asset = await modelWrapper.fromStaticResource(staticResources[0], false, false);

  const valueA = await (asset as any).field_a;
  const valueB = await (asset as any).field_b;
  expect(valueA?.toString()).toBe("alpha");
  expect(valueB?.toString()).toBe("beta");
});

test("Root-level cardinality-N collector node resolves multiple tiles", async () => {
  const graph = createStaticGraph({ name: "Collector Graph", author: "Test" });
  const mutator = new GraphMutator(graph);
  mutator.addStringNode(null, "multi_field", "Multi Field", "n",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note");
  const mutatedGraph = mutator.apply();

  const parsedGraph = parseStaticGraph(JSON.stringify({ graph: [mutatedGraph] }));
  const modelWrapper = makeModelWrapper(parsedGraph);

  const nodes = modelWrapper.getNodeObjectsByAlias();
  const multiNode = nodes.get("multi_field");

  const resourceId = "ffffffff-1111-2222-3333-444444444444";
  const businessData = makeBusinessData(parsedGraph.graphid, resourceId, [
    {
      data: { [multiNode.nodeid]: { en: { direction: "ltr", value: "first" } } },
      nodegroup_id: multiNode.nodegroup_id,
      parenttile_id: null,
      resourceinstance_id: resourceId,
      sortorder: 0,
      tileid: "11111111-aaaa-bbbb-cccc-dddddddddddd",
    },
    {
      data: { [multiNode.nodeid]: { en: { direction: "ltr", value: "second" } } },
      nodegroup_id: multiNode.nodegroup_id,
      parenttile_id: null,
      resourceinstance_id: resourceId,
      sortorder: 1,
      tileid: "22222222-aaaa-bbbb-cccc-dddddddddddd",
    },
  ]);

  const staticResources = parseStaticResources(JSON.stringify(businessData));
  const asset = await modelWrapper.fromStaticResource(staticResources[0], false, false);

  const value = await (asset as any).multi_field;
  // Cardinality-N returns PseudoList (array-like)
  expect(value).not.toBeNull();
  expect(value.length).toBe(2);
});

test("Nested node under semantic parent also resolves (not a root-level regression)", async () => {
  const graph = createStaticGraph({ name: "Nested Graph", author: "Test" });
  const mutator = new GraphMutator(graph);
  // Add a semantic node (group), then a string node under it
  mutator.addSemanticNode(null, "group_node", "Group Node", "1",
    "http://www.cidoc-crm.org/cidoc-crm/E55_Type",
    "http://www.cidoc-crm.org/cidoc-crm/P2_has_type");
  mutator.addStringNode("group_node", "nested_field", "Nested Field", "1",
    "http://www.w3.org/2000/01/rdf-schema#Literal",
    "http://www.cidoc-crm.org/cidoc-crm/P3_has_note");
  const mutatedGraph = mutator.apply();

  const parsedGraph = parseStaticGraph(JSON.stringify({ graph: [mutatedGraph] }));
  const modelWrapper = makeModelWrapper(parsedGraph);

  const nodes = modelWrapper.getNodeObjectsByAlias();
  const groupNode = nodes.get("group_node");
  const nestedNode = nodes.get("nested_field");

  // Nested field shares the nodegroup with group_node (cardinality 1, parent not root)
  const resourceId = "99999999-1111-2222-3333-444444444444";
  const groupNgId = groupNode.nodegroup_id;

  const businessData = makeBusinessData(parsedGraph.graphid, resourceId, [
    {
      data: {
        [groupNode.nodeid]: null,
        [nestedNode.nodeid]: { en: { direction: "ltr", value: "nested value" } },
      },
      nodegroup_id: groupNgId,
      parenttile_id: null,
      resourceinstance_id: resourceId,
      sortorder: 0,
      tileid: "77777777-aaaa-bbbb-cccc-dddddddddddd",
    },
  ]);

  const staticResources = parseStaticResources(JSON.stringify(businessData));
  const asset = await modelWrapper.fromStaticResource(staticResources[0], false, false);

  // Access: asset.group_node.nested_field
  const group = await (asset as any).group_node;
  expect(group).not.toBeNull();
  const nested = await group.nested_field;
  expect(nested).not.toBeNull();
  expect(nested.toString()).toBe("nested value");
});
