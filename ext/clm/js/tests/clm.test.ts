import { describe, test, assert, beforeAll } from 'vitest';

describe("CLM Extension - Reference Datatype", () => {
  beforeAll(async () => {
    // Initialize WASM via alizarin's wasmReady promise
    const { wasmReady } = await import("alizarin");
    await wasmReady;
  });

  test("CLM extension module can be imported and registers reference datatype", async () => {
    // Import the CLM extension which should register the reference datatype
    await import("../src/main");

    // Import viewModels from the same "alizarin" package the extension uses
    // This ensures we're checking the same CUSTOM_DATATYPES Map instance
    const alizarinModule = await import("alizarin");
    const CUSTOM_DATATYPES = alizarinModule.viewModels.CUSTOM_DATATYPES;

    // Verify the reference datatype is now registered
    const referenceDatatype = CUSTOM_DATATYPES.get("reference");
    assert.isDefined(referenceDatatype, "Reference datatype should be registered by CLM extension");
    assert.isDefined(referenceDatatype.__create, "Reference datatype should have __create method");
  });

  test("Test data files exist and are valid JSON", async () => {
    const PersonGraph = await import("./data/graphs/resource_models/Person.json");
    const PersonBusinessData = await import("./data/business_data/Person_2025-12-11_05-36-18.json");

    assert.isDefined(PersonGraph, "Person graph JSON should be importable");
    assert.isDefined(PersonBusinessData, "Person business data JSON should be importable");

    // Verify the graph has a reference node with controlledList
    const graph = PersonGraph["graph"][0];
    assert.isDefined(graph, "Graph data should exist");

    const testNode = graph.nodes.find((n: any) => n.datatype === "reference");
    assert.isDefined(testNode, "Graph should have a reference datatype node");
    assert.equal(testNode.alias, "test", "Reference node should be the 'test' node");
    assert.isDefined(testNode.config.controlledList, "Reference node should have controlledList config");

    // Verify business data has reference values
    const businessData = PersonBusinessData["business_data"];
    assert.isDefined(businessData, "Business data should exist");
    const resource = businessData.resources[0];
    const tile = resource.tiles[0];
    assert.isDefined(tile.data["3ade323c-376c-433e-bf07-e776546a562b"],
      "Tile should have data for the reference node");

    const referenceValues = tile.data["3ade323c-376c-433e-bf07-e776546a562b"];
    assert(Array.isArray(referenceValues), "Reference values should be an array");
    assert.equal(referenceValues.length, 1, "Should have one reference value");

    const refValue = referenceValues[0];
    assert.equal(refValue.list_id, "2730d609-3a8d-49dc-bf51-6ac34e80294a", "Reference should point to correct list");
    assert(Array.isArray(refValue.labels), "Reference should have labels array");
    assert.equal(refValue.labels[0].value, "Item 1>1", "Reference label should match");
  });
});
