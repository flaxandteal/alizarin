/**
 * Backend benchmark — WASM vs NAPI comparison.
 *
 * Run with:
 *   npx vitest bench --run --config benchmarks/vitest.bench.wasm.config.js
 *   npx vitest bench --run --config benchmarks/vitest.bench.napi.config.js
 *
 * CI script: node benchmarks/compare.mjs (runs both, outputs markdown table)
 */
import { bench, describe } from "vitest";
import {
  getBackend,
  createInstanceWrapperForModel,
  createResourceRegistry,
  createStaticGraphMeta,
  parseStaticGraph,
  createStaticTile,
  loadTiles,
  loadTilesFromResource,
} from "../js/backend";
import { ResourceModelWrapper, createWKRM } from "../js/graphManager";
import { IWKRM } from "../js/interfaces";
import * as GroupJSON from "../tests/data/models/Group.json";
import * as PersonJSON from "../tests/definitions/models/22477f01-1a44-11e9-b0a9-000d3ab1e588.json";
import * as GroupResourceJSON from "../tests/definitions/resources/_07883c9e-b25c-11e9-975a-a4d18cec433a.json";

// ---------------------------------------------------------------------------
// Setup (module-level, runs once before benchmarks)
// ---------------------------------------------------------------------------

const backend = getBackend();

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

const groupGraphJson = JSON.stringify(GroupJSON);
const personGraphJson = JSON.stringify(PersonJSON);

// Pre-parse for benchmarks that need a ready graph
const groupGraph = parseStaticGraph(groupGraphJson);
const personGraph = parseStaticGraph(personGraphJson);

// Register group model
const groupWkrm = createTestWKRM(groupGraph);
const groupModelWrapper = new ResourceModelWrapper(groupWkrm, groupGraph, undefined, true);

// Set up instance wrapper with tiles
const groupWrapper = createInstanceWrapperForModel(groupGraph.graphid);
const resources = (GroupResourceJSON as any).business_data.resources;
loadTilesFromResource(groupWrapper, resources[0]);

// Find the root node alias for populate/toJson
const groupNodes = groupGraph.nodes || [];
const groupRootAlias =
  (groupNodes.find((n: any) => n.istopnode) || groupNodes[0])?.alias || "group";

// Pre-create a tile for data operations
const benchTile = createStaticTile({
  nodegroup_id: "bench-ng-001",
  tileid: "bench-tile-001",
  sortorder: 0,
  resourceinstance_id: "bench-res-001",
  parenttile_id: null,
  data: new Map<string, any>(),
});

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

describe(`Graph parsing [${backend}]`, () => {
  bench("parse Group graph (60KB, ~30 nodes)", () => {
    parseStaticGraph(groupGraphJson);
  });

  bench("parse Person graph (228KB, ~100 nodes)", () => {
    parseStaticGraph(personGraphJson);
  });
});

describe(`Model registration [${backend}]`, () => {
  bench("register Group model + build nodes", () => {
    const wkrm = createTestWKRM(groupGraph);
    new ResourceModelWrapper(wkrm, groupGraph, undefined, true);
  });

  bench("register Person model + build nodes", () => {
    const wkrm = createTestWKRM(personGraph);
    new ResourceModelWrapper(wkrm, personGraph, undefined, true);
  });
});

describe(`Tile loading [${backend}]`, () => {
  const tilesOnly = resources[0]?.tiles || [];

  bench("loadTiles (tiles array only)", () => {
    const wrapper = createInstanceWrapperForModel(groupGraph.graphid);
    loadTiles(wrapper, tilesOnly);
  });

  bench("loadTilesFromResource (full resource)", () => {
    const wrapper = createInstanceWrapperForModel(groupGraph.graphid);
    loadTilesFromResource(wrapper, resources[0]);
  });
});

describe(`Path resolution [${backend}]`, () => {
  bench("getValuesAtPath('basic_info.name')", () => {
    groupWrapper.getValuesAtPath("basic_info.name");
  });

  bench("getValuesAtPath('basic_info')", () => {
    groupWrapper.getValuesAtPath("basic_info");
  });
});

describe(`Populate + serialize [${backend}]`, () => {
  // Ensure populated before serialization benchmarks
  groupWrapper.populate(false, [], groupRootAlias);

  bench("populate (full)", () => {
    groupWrapper.populate(false, [], groupRootAlias);
  });

  bench("toJson (full serialization)", () => {
    groupWrapper.toJson();
  });
});

describe(`Tile data operations [${backend}]`, () => {
  bench("tile.data.set + get (100 keys)", () => {
    for (let i = 0; i < 100; i++) {
      benchTile.data.set(`node-${i}`, { en: { value: `value-${i}` } });
    }
    for (let i = 0; i < 100; i++) {
      benchTile.data.get(`node-${i}`);
    }
  });

  bench("tile.data.has + delete (100 keys)", () => {
    for (let i = 0; i < 100; i++) {
      benchTile.data.set(`tmp-${i}`, "x");
    }
    for (let i = 0; i < 100; i++) {
      if (benchTile.data.has(`tmp-${i}`)) {
        benchTile.data.delete(`tmp-${i}`);
      }
    }
  });
});

describe(`Resource registry [${backend}]`, () => {
  const resourcesJson = JSON.stringify(resources);

  // Simulate the raw-bytes path (as if read from disk with fs.readFile)
  const businessDataBytes = new Uint8Array(
    Buffer.from(JSON.stringify(GroupResourceJSON))
  );

  // Create a larger payload (~100KB) by duplicating resources to simulate realistic file sizes
  const bulkResources = Array.from({ length: 50 }, (_, i) => {
    const r = JSON.parse(JSON.stringify(resources[0]));
    r.resourceinstance.resourceinstanceid = `bulk-${i}-${r.resourceinstance.resourceinstanceid}`;
    return r;
  });
  const bulkBusinessData = { business_data: { resources: bulkResources } };
  const bulkBytes = new Uint8Array(Buffer.from(JSON.stringify(bulkBusinessData)));

  bench("registry: merge resources JSON (string)", () => {
    const registry = createResourceRegistry();
    if (typeof registry.mergeFromResourcesJson === "function") {
      registry.mergeFromResourcesJson(resourcesJson, true);
    } else if (typeof registry.loadFromResourcesJson === "function") {
      registry.loadFromResourcesJson(resourcesJson, true);
    }
  });

  bench("registry: loadFromBusinessDataBytes (3KB)", () => {
    const registry = createResourceRegistry();
    registry.loadFromBusinessDataBytes(businessDataBytes, true, true);
  });

  bench("registry: loadFromBusinessDataBytes (bulk ~100KB)", () => {
    const registry = createResourceRegistry();
    registry.loadFromBusinessDataBytes(bulkBytes, true, true);
  });
});
