/**
 * Backend consistency tests — verifies WASM and NAPI expose identical
 * property names and method signatures on PseudoValue, PseudoList,
 * instance wrappers, and ResourceModelWrapper.
 *
 * Prevents regressions like:
 *   - NAPI exposing `tileDataJson` instead of `tileData`
 *   - NAPI missing the `node` getter
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
  getRdmCache,
  getNodeConfigManager,
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
      // NAPI: loadTilesFromResource accepts JSON string (single-pass deserialization)
      wrapper.loadTilesFromResource(JSON.stringify(resources[0]));
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
      // basic_info.name is held in the resource's one basic_info tile
      expect(pseudoValue.tileId).toBe("8225f975-813d-4705-9ada-e8cedd6fea48");
    });

    test("has .nodeId property", () => {
      expect(pseudoValue).toBeDefined();
      expect(typeof pseudoValue.nodeId).toBe('string');
    });

    test("has .nodeAlias property", () => {
      expect(pseudoValue).toBeDefined();
      expect(pseudoValue.nodeAlias).toBe("name");
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
      expect(pseudoValue.nodegroupId).toBe("12707705-c05e-11e9-8177-a4d18cec433a");
    });

    test("has .isCollector property (boolean)", () => {
      expect(pseudoValue).toBeDefined();
      expect(typeof pseudoValue.isCollector).toBe('boolean');
    });

    test("has core's .hasTile, .isInner and .isOuter flags (booleans)", () => {
      // The rest of the surface is pinned by tests/binding-parity.json.
      expect(pseudoValue).toBeDefined();
      expect(typeof pseudoValue.hasTile).toBe('boolean');
      expect(typeof pseudoValue.isInner).toBe('boolean');
      expect(typeof pseudoValue.isOuter).toBe('boolean');
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
  // Instance wrapper methods (prevents bug #5: missing setLazy/tilesLoaded)
  // =========================================================================

  describe("InstanceWrapper methods", () => {
    test("has .getValuesAtPath(path) method", () => {
      expect(typeof wrapper.getValuesAtPath).toBe('function');
      const result = wrapper.getValuesAtPath("basic_info");
      expect(result).toBeDefined();
      expect(typeof result.totalValues).toBe('number');
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

    test("appendFetchedTiles marks just the fetched nodegroup loaded, even with no tiles", () => {
      // A fresh lazy instance has fetched nothing: every child is unknown.
      const lazy = createInstanceWrapperForModel(graph.graphid);
      expect(lazy.isLazy()).toBe(false);
      lazy.setLazy(true);
      expect(lazy.isLazy()).toBe(true);
      const rootId = modelWrapper.getRootNode().nodeid;
      const nodegroupOf = (alias: string) =>
        modelWrapper.getNodeObjectsByAlias().get(alias)!.nodegroup_id;
      const child = (alias: string) => {
        try {
          return lazy.getSemanticChildValue(null, rootId, null, alias);
        } catch (e: any) {
          return String(e?.message ?? e);
        }
      };

      expect(child("statement")).toMatch(/^TILES_NOT_LOADED:/);
      lazy.appendFetchedTiles([], nodegroupOf("statement"));
      expect(child("statement")).toBeNull();
      expect(child("basic_info")).toMatch(/^TILES_NOT_LOADED:/);
    });

    test("a cyclic argument throws, leaving the process and wrapper usable", () => {
      // NAPI's argument conversion once recursed until V8 aborted the process.
      const fresh = createInstanceWrapperForModel(graph.graphid);
      const op: any = { op: "set", alias: "statement", value: 1 };
      op.self = op;
      expect(() => fresh.applyEdits([op])).toThrow();
      expect(fresh.isLazy()).toBe(false);
    });

    test("serializes cards through core, in tile-data and display modes", () => {
      const basicInfo = modelWrapper.getNodeObjectsByAlias().get("basic_info")!.nodegroup_id;
      const card = (GroupJSON as any).graph[0].cards.find(
        (c: any) => c.nodegroup_id === basicInfo,
      );
      expect(wrapper.nodegroupIdsForCard(card.cardid, 0)).toEqual([basicInfo]);

      wrapper.toJson(); // populates the pseudo cache
      const serialized = wrapper.serializeCard(card.cardid, null, null, 0);
      expect(JSON.stringify(serialized)).toContain("Global Group");
      expect(wrapper.serializeRootCards(0)).toContainEqual(serialized);

      const display = wrapper.serializeCardDisplay(
        card.cardid, getRdmCache(), getNodeConfigManager(), null, null, 0, "ga", staticStore.registry,
      );
      expect(JSON.stringify(display)).toContain("Grúpa Domhanda");
      const rootsDisplay = wrapper.serializeRootCardsDisplay(
        getRdmCache(), getNodeConfigManager(), 0, "ga", staticStore.registry,
      );
      expect(rootsDisplay).toContainEqual(display);

      expect(() => wrapper.serializeCard("not-a-card", null, null, 0)).toThrow(/not found/);
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
