/**
 * Output parity: every route that hands a static type to JS gives exactly
 * core's serialisation, as recorded in tests/data/output-parity/expected.json
 * by crates/alizarin-core/tests/output_parity_test.rs. The comparison is of
 * `JSON.stringify` output, byte for byte: key presence (`null` vs missing),
 * key order and values. Python is held to the same file in
 * crates/alizarin-python/tests/test_output_parity.py.
 *
 * Getters are checked too: none may return `undefined` (WASM's default for an
 * empty `Option`, where NAPI gives `null`), and a getter named after a field
 * returns that field's value.
 *
 * The binding parity manifest (binding-parity.json) pins which members exist;
 * this pins what they return.
 *
 * Runs under both:
 *   npx vitest run tests/dual-backend/outputParity.test.ts
 *   npx vitest run --config vitest.napi.config.js tests/dual-backend/outputParity.test.ts
 */
import { describe, expect, test, beforeAll } from "vitest";
import * as fs from "fs";
import * as path from "path";
import {
  getBackend,
  parseStaticGraph,
  createStaticGraphMeta,
  createWKRM,
  createStaticTile,
  createResourceRegistry,
  createInstanceWrapperForModel,
  parseStaticResources,
} from "../../js/backend";
import { ResourceModelWrapper } from "../../js/graphManager";
import { composeResourceLayers } from "../../js/csvModelLoader";

const DIR = path.join(__dirname, "../data/output-parity");
const read = (name: string) => fs.readFileSync(path.join(DIR, name), "utf8");
const golden: Record<string, string> = JSON.parse(read("expected.json"));
const backend = getBackend();

const graphText = read("graph.json");
const graphFixture = JSON.parse(graphText).graph[0];
const resourceFixture = JSON.parse(read("resource.json"));
const layerFixture = JSON.parse(read("resource-layer.json"));
const RESOURCE_ID = resourceFixture.resourceinstance.resourceinstanceid;

/** Keys sorted at every level: for routes that return a string serialised
 * straight from a struct (field order, identical on every backend because
 * core writes it), compared with the golden's sorted form. */
function sortKeys(value: any): any {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.keys(value).sort().map((k) => [k, sortKeys(value[k])]));
  }
  return value;
}

/**
 * Getters whose value is not the field's JSON, by class: each needs a reason.
 * A getter not listed here and named after a field must return that field.
 */
const GETTER_EXCEPTIONS: Record<string, Record<string, string>> = {
  tile: { data: "Map-like by design: tile.data.get/set (a Map on WASM, NapiTileData on NAPI)" },
};

let graph: any;
let full: any;

beforeAll(() => {
  graph = parseStaticGraph(graphText);
  const wkrm = createWKRM(
    createStaticGraphMeta({
      graphid: graph.graphid,
      name: "Parity",
      slug: "parity",
      relatable_resource_model_ids: [],
      resource_2_resource_constraints: [],
      extra_fields: {},
    }),
  );
  new ResourceModelWrapper(wkrm, graph, undefined, true);
  const registry = createResourceRegistry();
  const json = JSON.stringify([resourceFixture]);
  if (typeof registry.mergeFromResourcesJson === "function") registry.mergeFromResourcesJson(json, true);
  else registry.loadFromResourcesJson(json, true);
  full = registry.getFull(RESOURCE_ID);
  (globalThis as any).__parityRegistry = registry;
});

/** Each object a route returns, with the golden label it must match. */
function routes(): [string, string, () => any][] {
  const out: [string, string, () => any][] = [
    ["graph", "graph", () => graph],
    ["graph.name", "graph.name", () => graph.name],
    ["graph.root", "graph.root", () => graph.root],
  ];
  graphFixture.nodes.forEach((n: any, i: number) =>
    out.push([`graph.nodes[${i}]`, `node:${n.alias}`, () => graph.nodes[i]]),
  );
  graphFixture.nodegroups.forEach((g: any, i: number) =>
    out.push([`graph.nodegroups[${i}]`, `nodegroup:${g.nodegroupid}`, () => graph.nodegroups[i]]),
  );
  graphFixture.edges.forEach((e: any, i: number) =>
    out.push([`graph.edges[${i}]`, `edge:${e.edgeid}`, () => graph.edges[i]]),
  );
  graphFixture.cards.forEach((c: any, i: number) => {
    out.push([`graph.cards[${i}]`, `card:${c.cardid}`, () => graph.cards[i]]);
    c.constraints.forEach((k: any, j: number) =>
      out.push([`graph.cards[${i}].constraints[${j}]`, `constraint:${k.constraintid}`, () => graph.cards[i].constraints[j]]),
    );
  });
  graphFixture.cards_x_nodes_x_widgets.forEach((x: any, i: number) =>
    out.push([`graph.cards_x_nodes_x_widgets[${i}]`, `cxnxw:${x.id}`, () => graph.cards_x_nodes_x_widgets[i]]),
  );
  graphFixture.functions_x_graphs.forEach((f: any, i: number) =>
    out.push([`graph.functions_x_graphs[${i}]`, `fxg:${f.id}`, () => graph.functions_x_graphs[i]]),
  );
  out.push(["registry.getFull", "resource", () => full]);
  out.push(["registry.getSummary", "resource.summary", () => (globalThis as any).__parityRegistry.getSummary(RESOURCE_ID)]);
  resourceFixture.tiles.forEach((t: any, i: number) => {
    out.push([`getFull().tiles[${i}]`, `tile:${t.tileid}`, () => full.tiles[i]]);
    out.push([`createStaticTile(tiles[${i}])`, `tile:${t.tileid}`, () => createStaticTile(t)]);
  });
  out.push([
    "createStaticTile(empty)",
    "tile:empty",
    () => createStaticTile({ nodegroup_id: resourceFixture.tiles[0].nodegroup_id, resourceinstance_id: RESOURCE_ID, data: {} }),
  ]);
  out.push([
    "composeResourceLayers",
    "composed",
    () => composeResourceLayers([layerFixture, resourceFixture], graphFixture, [], false),
  ]);
  return out;
}

describe(`output parity [${backend} backend]`, () => {
  test("every route gives core's JSON, byte for byte", () => {
    const wrong: string[] = [];
    for (const [route, label, get] of routes()) {
      let actual: string;
      try {
        actual = JSON.stringify(get());
      } catch (e) {
        actual = `threw ${e}`;
      }
      if (actual !== golden[label]) wrong.push(`${route} (${label}):\n  got      ${actual}\n  expected ${golden[label]}`);
    }
    expect(wrong, wrong.join("\n")).toEqual([]);
  });

  test("routes returning a string give core's JSON, keys aside", () => {
    const w = createInstanceWrapperForModel(graph.graphid);
    w.loadTilesFromResource(
      backend === "napi"
        ? JSON.stringify(resourceFixture)
        : parseStaticResources(JSON.stringify({ business_data: { resources: [resourceFixture] } }))[0],
    );
    const exported = JSON.parse(w.exportTilesJson());
    resourceFixture.tiles.forEach((t: any, i: number) => {
      expect(JSON.stringify(sortKeys(exported[i])), `exportTilesJson()[${i}]`).toBe(golden[`tile:${t.tileid}`]);
    });
  });

  test("a pseudo value's tile is core's JSON", () => {
    const w = createInstanceWrapperForModel(graph.graphid);
    w.loadTilesFromResource(
      backend === "napi"
        ? JSON.stringify(resourceFixture)
        : parseStaticResources(JSON.stringify({ business_data: { resources: [resourceFixture] } }))[0],
    );
    const tile = w.getValuesAtPath("name").getValue(0).tile;
    expect(JSON.stringify(tile)).toBe(golden[`tile:${resourceFixture.tiles[0].tileid}`]);
  });

  test("no getter returns undefined, and a field's getter returns the field", () => {
    const wrong: string[] = [];
    for (const [route, label, get] of routes()) {
      let object: any;
      try {
        object = get();
      } catch {
        continue; // a missing route: the byte-for-byte test reports it
      }
      if (!object || typeof object !== "object") continue;
      const expected = JSON.parse(golden[label]);
      const kind = label.split(":")[0].replace(/^(resource|graph)\..*$/, "$1");
      const exceptions = GETTER_EXCEPTIONS[kind] ?? {};
      const getters = new Set<string>();
      for (let p = Object.getPrototypeOf(object); p && p !== Object.prototype; p = Object.getPrototypeOf(p)) {
        for (const [name, d] of Object.entries(Object.getOwnPropertyDescriptors(p))) if (d.get) getters.add(name);
      }
      for (const name of getters) {
        if (exceptions[name]) continue;
        let value: any;
        try {
          value = object[name];
        } catch (e) {
          wrong.push(`${route}.${name} threw ${e}`);
          continue;
        }
        if (value === undefined) {
          wrong.push(`${route}.${name} is undefined`);
          continue;
        }
        if (expected && typeof expected === "object" && !Array.isArray(expected) && name in expected) {
          const got = JSON.stringify(value);
          const want = JSON.stringify(expected[name]);
          if (got !== want) wrong.push(`${route}.${name}: got ${got}, field is ${want}`);
        }
      }
    }
    expect(wrong, wrong.join("\n")).toEqual([]);
  });
});
