/**
 * Backend consistency tests — verifies WASM and NAPI expose identical
 * property names and method signatures on PseudoValue, PseudoList,
 * instance wrappers, and ResourceModelWrapper.
 *
 * Prevents regressions like:
 *   - NAPI exposing `tileDataJson` instead of `tileData`
 *   - NAPI missing `setTileDataForNode` / `node` getter
 *   - ResourceModelWrapper missing `setDefaultAllowAllNodegroups`
 *
 * Runs under both backends via:
 *   npx vitest run (WASM)
 *   npx vitest run -c vitest.napi.config.js (NAPI)
 */
import { test, describe, beforeAll, expect } from "vitest";
import {
  StaticGraph,
  StaticGraphMeta,
} from '../../js/static-types';
import { ResourceModelWrapper, createWKRM } from '../../js/graphManager';
import {
  getBackend,
  createInstanceWrapperForModel,
  createResourceRegistry,
  createStaticGraphMeta,
  parseStaticGraph,
} from '../../js/backend';
import { IWKRM } from '../../js/interfaces';
import { staticStore } from '../../js/staticStore';
import * as GroupJSON from "../data/models/Group.json";
import * as GroupResourceJSON from "../definitions/resources/_07883c9e-b25c-11e9-975a-a4d18cec433a.json";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function createTestWKRM(graphData: any): IWKRM {
  const meta = createStaticGraphMeta({
    graphid: graphData.graphid,
    name: graphData.name || "Test Graph",
    slug: "test_graph",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {},
  });
  return createWKRM(meta);
}

// ---------------------------------------------------------------------------
// Test state
// ---------------------------------------------------------------------------

let wrapper: any;          // WASMResourceInstanceWrapper or NapiResourceInstanceWrapper
let modelWrapper: ResourceModelWrapper<any>;
let pseudoList: any;       // from getValuesAtPath
let pseudoValue: any;      // first value in the list
let resource: any;
let graph: any;

describe(`Backend consistency [${getBackend()}]`, () => {

  beforeAll(() => {
    // Load graph model — use backend-aware factory (no WASM constructor)
    const graphJson = JSON.stringify(GroupJSON);
    graph = parseStaticGraph(graphJson);

    // Create model wrapper (this also registers the graph for NAPI)
    const wkrm = createTestWKRM(graph);
    modelWrapper = new ResourceModelWrapper(wkrm, graph, undefined, true);

    // Load resource via the registry (handles both WASM and NAPI resource loading)
    const registry = createResourceRegistry();
    const resourcesJson = JSON.stringify((GroupResourceJSON as any).business_data.resources);
    if (typeof registry.mergeFromResourcesJson === 'function') {
      // NAPI path
      registry.mergeFromResourcesJson(resourcesJson, true);
    } else if (typeof registry.loadFromResourcesJson === 'function') {
      // WASM path
      registry.loadFromResourcesJson(resourcesJson, true);
    }

    // Create instance wrapper for the model, then load tiles
    wrapper = createInstanceWrapperForModel(graph.graphid);
    const resources = (GroupResourceJSON as any).business_data.resources;
    const tiles = resources[0]?.tiles || [];
    if (getBackend() === 'napi') {
      // NAPI: loadTilesFromResource accepts plain JSON
      wrapper.loadTilesFromResource(resources[0]);
    } else {
      // WASM: loadTiles accepts plain JS tile array
      wrapper.loadTiles(tiles);
    }

    // Get a pseudo list by resolving a known path
    pseudoList = wrapper.getValuesAtPath("basic_info.name");
    if (pseudoList.totalValues > 0) {
      pseudoValue = pseudoList.getValue(0);
    }
  });

  // =========================================================================
  // PseudoValue property names (prevents bug #1: tileDataJson vs tileData,
  //                              bug #3: missing .node getter)
  // =========================================================================

  describe("PseudoValue getters", () => {
    test("has .tileData property", () => {
      expect(pseudoValue).toBeDefined();
      // The critical regression: consumers access .tileData, not .tileDataJson
      // Both backends must expose .tileData
      expect(() => pseudoValue.tileData).not.toThrow();
    });

    test("has .tileId property", () => {
      expect(pseudoValue).toBeDefined();
      const tileId = pseudoValue.tileId;
      // tileId should be a string (UUID) or null
      if (tileId !== null && tileId !== undefined) {
        expect(typeof tileId).toBe('string');
      }
    });

    test("has .nodeId property", () => {
      expect(pseudoValue).toBeDefined();
      expect(typeof pseudoValue.nodeId).toBe('string');
    });

    test("has .nodeAlias property", () => {
      expect(pseudoValue).toBeDefined();
      // nodeAlias may be null for nodes without alias
      if (pseudoValue.nodeAlias !== null && pseudoValue.nodeAlias !== undefined) {
        expect(typeof pseudoValue.nodeAlias).toBe('string');
      }
    });

    test("has .datatype property (string)", () => {
      expect(pseudoValue).toBeDefined();
      expect(typeof pseudoValue.datatype).toBe('string');
    });

    test("has .node getter returning object with .datatype", () => {
      expect(pseudoValue).toBeDefined();
      const node = pseudoValue.node;
      expect(node).toBeDefined();
      expect(typeof node).toBe('object');
      expect(node.datatype).toBeDefined();
      expect(typeof node.datatype).toBe('string');
    });

    test("has .nodegroupId property", () => {
      expect(pseudoValue).toBeDefined();
      // nodegroupId may be null for root nodes
      if (pseudoValue.nodegroupId !== null && pseudoValue.nodegroupId !== undefined) {
        expect(typeof pseudoValue.nodegroupId).toBe('string');
      }
    });

    test("has .isCollector property (boolean)", () => {
      expect(pseudoValue).toBeDefined();
      expect(typeof pseudoValue.isCollector).toBe('boolean');
    });

    test("has .sortorder property", () => {
      expect(pseudoValue).toBeDefined();
      // sortorder can be number or null
      expect(() => pseudoValue.sortorder).not.toThrow();
    });

    test("has .valueLoaded property (boolean)", () => {
      expect(pseudoValue).toBeDefined();
      expect(typeof pseudoValue.valueLoaded).toBe('boolean');
    });
  });

  // =========================================================================
  // PseudoList property names
  // =========================================================================

  describe("PseudoList getters", () => {
    test("has .totalValues (number)", () => {
      expect(typeof pseudoList.totalValues).toBe('number');
      expect(pseudoList.totalValues).toBeGreaterThan(0);
    });

    test("has .getValue(index) method", () => {
      expect(typeof pseudoList.getValue).toBe('function');
      const val = pseudoList.getValue(0);
      expect(val).toBeDefined();
    });

    test("has .getAllValues() method returning array", () => {
      expect(typeof pseudoList.getAllValues).toBe('function');
      const vals = pseudoList.getAllValues();
      expect(Array.isArray(vals)).toBe(true);
      expect(vals.length).toBe(pseudoList.totalValues);
    });

    test("has .nodeAlias property (string)", () => {
      expect(typeof pseudoList.nodeAlias).toBe('string');
    });
  });

  // =========================================================================
  // Instance wrapper methods (prevents bug #2: missing setTileDataForNode,
  //                            bug #5: missing setLazy/tilesLoaded)
  // =========================================================================

  describe("InstanceWrapper methods", () => {
    test("has .getValuesAtPath(path) method", () => {
      expect(typeof wrapper.getValuesAtPath).toBe('function');
      const result = wrapper.getValuesAtPath("basic_info");
      expect(result).toBeDefined();
      expect(typeof result.totalValues).toBe('number');
    });

    test("has .setTileDataForNode(tileId, nodeId, value) on the wrapper", () => {
      expect(typeof wrapper.setTileDataForNode).toBe('function');
      const tileId = pseudoValue?.tileId;
      if (tileId) {
        const result = wrapper.setTileDataForNode(tileId, "fake-node-id", null);
        expect(typeof result).toBe('boolean');
      }
    });

    test("setTileDataForNode write persists through getValuesAtPath", () => {
      // Regression: in WASM, setTileDataForNode on StaticResource wrote to a
      // cloned copy, so reads via getValuesAtPath saw stale data.
      const tileId = pseudoValue.tileId;
      const nodeId = pseudoValue.nodeId;
      expect(tileId).toBeDefined();

      // Write a sentinel value
      const sentinel = { __test: "persist-check" };
      const ok = wrapper.setTileDataForNode(tileId, nodeId, sentinel);
      expect(ok).toBe(true);

      // Read it back through getValuesAtPath — must see the sentinel
      const freshList = wrapper.getValuesAtPath("basic_info.name");
      expect(freshList.totalValues).toBeGreaterThan(0);
      let found = false;
      for (let i = 0; i < freshList.totalValues; i++) {
        const pv = freshList.getValue(i);
        if (pv.tileId === tileId) {
          const td = pv.tileData;
          // WASM returns Map, NAPI returns plain object
          const value = td instanceof Map ? td.get('__test') : td?.__test;
          if (value === "persist-check") {
            found = true;
          }
          break;
        }
      }
      expect(found).toBe(true);
    });

    test("has .tilesLoaded() method returning boolean", () => {
      expect(typeof wrapper.tilesLoaded).toBe('function');
      expect(typeof wrapper.tilesLoaded()).toBe('boolean');
    });

    test("has .toJson() method", () => {
      expect(typeof wrapper.toJson).toBe('function');
    });

    test("has .setLazy(boolean) method", () => {
      expect(typeof wrapper.setLazy).toBe('function');
    });
  });

  // =========================================================================
  // ResourceModelWrapper methods (prevents bug #6: missing delegation)
  // =========================================================================

  describe("ResourceModelWrapper methods", () => {
    test("has .setDefaultAllowAllNodegroups(boolean)", () => {
      expect(typeof modelWrapper.setDefaultAllowAllNodegroups).toBe('function');
      // Should not throw
      modelWrapper.setDefaultAllowAllNodegroups(true);
    });

    test("has .setPermittedNodegroups(permissions)", () => {
      expect(typeof modelWrapper.setPermittedNodegroups).toBe('function');
    });

    test("has .isNodegroupPermitted(id)", () => {
      expect(typeof modelWrapper.isNodegroupPermitted).toBe('function');
    });
  });
});
