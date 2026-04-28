import { test, beforeAll } from "vitest";
import { assert } from 'chai';
import { createWKRM } from '../js/graphManager';
import { StaticGraphMeta } from '../js/static-types';
import { initWasmForTests } from './wasm-init';

beforeAll(async () => {
  await initWasmForTests();
});

// ============================================================================
// WKRM Constructor Tests - Class Name Manipulation
// ============================================================================

test("WKRM > constructor > should create instance with basic meta", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.isDefined(wkrm);
  assert.equal(wkrm.graphId, "test-graph-123");
  assert.equal(wkrm.modelName, "Test Graph");
  // Compare the graphid to verify meta was properly stored (instances may differ due to serialization)
  assert.equal(wkrm.meta.graphid, meta.graphid);
});

test("WKRM > constructor > should convert slug with underscores to PascalCase", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    slug: "test_graph_model",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "TestGraphModel");
});

test("WKRM > constructor > should convert slug with hyphens to PascalCase", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    slug: "test-graph-model",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "TestGraphModel");
});

test("WKRM > constructor > should convert slug with spaces to PascalCase", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    slug: "test graph model",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "TestGraphModel");
});

test("WKRM > constructor > should handle mixed separators in slug", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    slug: "test_graph-model name",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "TestGraphModelName");
});

test("WKRM > constructor > should ensure first character is uppercase", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    slug: "myModel",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "MyModel");
});

test("WKRM > constructor > should handle single word slug", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    slug: "group",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "Group");
});

test("WKRM > constructor > should use modelName as fallback when slug is missing", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "my test graph",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "MyTestGraph");
});

test("WKRM > constructor > should use 'Unnamed' when name is missing", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelName, "Unnamed");
  assert.equal(wkrm.modelClassName, "Unnamed");
});

test("WKRM > constructor > should handle slug with leading underscore", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    slug: "_private_model",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "PrivateModel");
});

test("WKRM > constructor > should handle slug with trailing underscore", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    slug: "test_model_",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "TestModel");
});

test("WKRM > constructor > should handle slug with consecutive separators", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    slug: "test__graph--model",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  // Consecutive separators result in empty strings after split,
  // which don't get capitalized
  assert.equal(wkrm.modelClassName, "TestGraphModel");
});

test("WKRM > constructor > should preserve existing PascalCase in slug", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-graph-123",
    name: "Test Graph",
    slug: "MyCustomModel",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "MyCustomModel");
});

test("WKRM > constructor > should handle real-world example: Group", () => {
  const meta = new StaticGraphMeta({
    graphid: "07883c9e-b25c-11e9-975a-a4d18cec433a",
    name: "Group",
    slug: "group",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelName, "Group");
  assert.equal(wkrm.modelClassName, "Group");
  assert.equal(wkrm.graphId, "07883c9e-b25c-11e9-975a-a4d18cec433a");
});

test("WKRM > constructor > should handle real-world example: Historical Event", () => {
  const meta = new StaticGraphMeta({
    graphid: "abc123",
    name: "Historical Event",
    slug: "historical_event",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelName, "Historical Event");
  assert.equal(wkrm.modelClassName, "HistoricalEvent");
});

test("WKRM > constructor > should handle numbers in slug", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-123",
    name: "Test Model",
    slug: "test_model_v2",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  assert.equal(wkrm.modelClassName, "TestModelV2");
});

test("WKRM > constructor > should handle special characters that aren't underscores or hyphens", () => {
  const meta = new StaticGraphMeta({
    graphid: "test-123",
    name: "Test Model",
    slug: "test@model#name",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });

  const wkrm = createWKRM(meta);

  // Special characters other than _ and - are not replaced, just PascalCased
  assert.equal(wkrm.modelClassName, "Test@model#name");
});
