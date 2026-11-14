import { describe, it, expect, beforeEach } from "vitest";
import { RDM } from "../js/rdm";
import { createStaticGraph, StaticCollection, StaticConcept } from "../js/static-types";
import { GraphMutator } from "../js/graphManager";

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

    // A graph with a concept node (rdmCollection config), a reference node
    // (controlledList config) and a string node (not resolved), built by core.
    // No auto cards: the reference datatype has no default widget in core.
    const E55 = "http://www.cidoc-crm.org/cidoc-crm/E55_Type";
    const P2 = "http://www.cidoc-crm.org/cidoc-crm/P2_has_type";
    graph = new GraphMutator(
      createStaticGraph({
        author: "Test",
        description: "Test graph",
        name: "Test Graph",
      }),
      { autocreateCard: false },
    )
      .addConceptNode(null, "category", "Category", collection, "n", E55, P2)
      ._addGenericNode(null, "status", "Status", "n", "reference", E55, P2, undefined, {}, {
        controlledList: "test-collection-id",
      })
      .addStringNode(null, "name", "Name", "n",
        "http://www.cidoc-crm.org/cidoc-crm/E41_Appellation",
        "http://www.cidoc-crm.org/cidoc-crm/P1_is_identified_by")
      .apply();
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

    // "reference" is an extension type (registered by CLM), not a core default,
    // and core does not know its config key. Without CLM loaded, we must pass
    // both; with CLM, its handler names the collection.
    const resolved = await RDM.resolveLabels(tree, graph, {
      additionalDatatypes: ["reference"],
      additionalConfigKeys: ["controlledList"],
    });

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
