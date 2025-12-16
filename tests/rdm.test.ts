import { describe, it, expect, beforeEach } from "vitest";
import { RDM } from "../js/rdm";
import { createStaticGraph, StaticCollection, StaticConcept, StaticNode } from "../js/static-types";

describe("RDM.resolveLabels", () => {
  let collection: StaticCollection;
  let graph: ReturnType<typeof createStaticGraph>;

  beforeEach(() => {
    // Clear RDM cache
    RDM.clear();

    // Create a test collection with child concepts
    collection = StaticCollection.fromConceptScheme({
      collectionid: "test-collection-id",
      name: "Test Categories",
      conceptScheme: StaticConcept.fromValue(
        null,
        "Categories",
        ["Category A", "Category B", "Category C"]
      ),
    });

    // Add to RDM cache
    RDM.addCollection(collection);

    // Create a graph with concept and reference nodes
    graph = createStaticGraph({
      author: "Test",
      description: "Test graph",
      name: "Test Graph",
    });

    // Add a concept node with rdmCollection config
    const conceptNode = new StaticNode({
      nodeid: "concept-node-id",
      alias: "category",
      datatype: "concept",
      name: "Category",
      graph_id: graph.graphid,
      nodegroup_id: "concept-nodegroup-id",
      is_collector: true,
      isrequired: false,
      exportable: false,
      config: {
        rdmCollection: "test-collection-id",
      },
    });

    // Add a reference node with controlledList config
    const referenceNode = new StaticNode({
      nodeid: "reference-node-id",
      alias: "status",
      datatype: "reference",
      name: "Status",
      graph_id: graph.graphid,
      nodegroup_id: "reference-nodegroup-id",
      is_collector: true,
      isrequired: false,
      exportable: false,
      config: {
        controlledList: "test-collection-id",
      },
    });

    // Add a string node (should not be resolved)
    const stringNode = new StaticNode({
      nodeid: "string-node-id",
      alias: "name",
      datatype: "string",
      name: "Name",
      graph_id: graph.graphid,
      nodegroup_id: "string-nodegroup-id",
      is_collector: true,
      isrequired: false,
      exportable: false,
      config: {},
    });

    graph.pushNode(conceptNode);
    graph.pushNode(referenceNode);
    graph.pushNode(stringNode);
  });

  it("should resolve concept labels to UUIDs", async () => {
    const tree = {
      category: ["Category A", "Category B"],
    };

    const resolved = await RDM.resolveLabels(tree, graph);

    // Labels should be replaced with UUIDs
    expect(resolved.category[0]).not.toBe("Category A");
    expect(resolved.category[1]).not.toBe("Category B");
    // Should be valid UUIDs
    expect(resolved.category[0]).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
    );
  });

  it("should resolve reference labels using controlledList config", async () => {
    const tree = {
      status: ["Category C"],
    };

    const resolved = await RDM.resolveLabels(tree, graph);

    expect(resolved.status[0]).not.toBe("Category C");
    expect(resolved.status[0]).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
    );
  });

  it("should pass through UUIDs unchanged", async () => {
    const conceptA = collection.getConceptByValue?.("Category A");
    expect(conceptA).toBeDefined();

    const tree = {
      category: [conceptA!.id],
    };

    const resolved = await RDM.resolveLabels(tree, graph);

    // UUID should pass through unchanged
    expect(resolved.category[0]).toBe(conceptA!.id);
  });

  it("should not modify non-concept fields", async () => {
    const tree = {
      name: ["John Doe"],
      category: ["Category A"],
    };

    const resolved = await RDM.resolveLabels(tree, graph);

    // String field should be unchanged
    expect(resolved.name[0]).toBe("John Doe");
    // Concept field should be resolved
    expect(resolved.category[0]).not.toBe("Category A");
  });

  it("should handle _value wrapper", async () => {
    const tree = {
      category: [{ _value: "Category A" }],
    };

    const resolved = await RDM.resolveLabels(tree, graph);

    expect(resolved.category[0]._value).not.toBe("Category A");
    expect(resolved.category[0]._value).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
    );
  });

  it("should throw in strict mode for unknown labels", async () => {
    const tree = {
      category: ["Unknown Category"],
    };

    await expect(
      RDM.resolveLabels(tree, graph, { strict: true })
    ).rejects.toThrow("Unknown Category");
  });

  it("should pass through unknown labels in non-strict mode", async () => {
    const tree = {
      category: ["Unknown Category"],
    };

    const resolved = await RDM.resolveLabels(tree, graph, { strict: false });

    // Unknown label should pass through
    expect(resolved.category[0]).toBe("Unknown Category");
  });

  it("should only load collections that are needed", async () => {
    // Clear and don't pre-load
    RDM.clear();

    // Tree only uses 'category', not 'status'
    const tree = {
      name: ["John"],
    };

    // This should NOT try to load the collection since 'name' is a string field
    const resolved = await RDM.resolveLabels(tree, graph);

    expect(resolved.name[0]).toBe("John");
    // Collection should not have been loaded
    expect(RDM.hasCollection("test-collection-id")).toBe(false);
  });
});
