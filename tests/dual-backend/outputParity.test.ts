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
  createStaticNode,
  createStaticNodegroup,
  createStaticEdge,
  createStaticCard,
  createStaticPublication,
  createStaticCardsXNodesXWidgets,
  createStaticResource,
  createResourceRegistry,
  createInstanceWrapperForModel,
  parseStaticResources,
  createStaticResourceFromSummary,
  summaryToMetadata,
  tilesToTree,
  cardsToTree,
  batchTilesToTrees,
  treeToTiles,
  batchTreesToTiles,
  mergeResources,
  batchMergeResources,
  buildPrebuildExport,
} from "../../js/backend";
import { ResourceModelWrapper } from "../../js/graphManager";
import { composeResourceLayers, buildGraphFromModelCsvs, buildResourcesFromBusinessCsv } from "../../js/csvModelLoader";
import { resourceLifecycleState, transitionResourceLifecycle } from "../../js/lifecycle";

const DIR = path.join(__dirname, "../data/output-parity");
const read = (name: string) => fs.readFileSync(path.join(DIR, name), "utf8");
const golden: Record<string, string> = JSON.parse(read("expected.json"));
const backend = getBackend();

const graphText = read("graph.json");
const graphFixture = JSON.parse(graphText).graph[0];
const resourceFixture = JSON.parse(read("resource.json"));
const layerFixture = JSON.parse(read("resource-layer.json"));
const RESOURCE_ID = resourceFixture.resourceinstance.resourceinstanceid;
/** Arches' Standard lifecycle, as in graph.json. */
const ACTIVE = "f75bb034-36e3-4ab4-8167-f520cf0b4c58";
const RETIRED = "d95d9c0e-0e2c-4450-93a3-d788b91abcc8";
/** Fixes the ids treeToTiles gives cardinality-n tiles. */
const TILE_NAMESPACE = "0d0d0d0d-0000-4000-8000-000000000001";
/** The RDM namespace the CSV fixtures' graph is built in. */
const CSV_NAMESPACE = "https://example.org/parity";

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
let fullByLanguage: any;

/** resource.json with its descriptors in Arches' per-language form: its own
 * under "en", a decoy under "de" (which sorts first). Read as "en", it must
 * give the same resource (as output_parity_test.rs checks in core). */
const resourceByLanguage = {
  ...resourceFixture,
  resourceinstance: {
    ...resourceFixture.resourceinstance,
    descriptors: { de: { name: "Parität eins" }, en: resourceFixture.resourceinstance.descriptors },
  },
};

function loadRegistry(resource: any) {
  const registry = createResourceRegistry();
  const json = JSON.stringify([resource]);
  if (typeof registry.mergeFromResourcesJson === "function") registry.mergeFromResourcesJson(json, true);
  else registry.loadFromResourcesJson(json, true);
  return registry;
}

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
  const registry = loadRegistry(resourceFixture);
  full = registry.getFull(RESOURCE_ID);
  (globalThis as any).__parityRegistry = registry;
  fullByLanguage = loadRegistry(resourceByLanguage).getFull(RESOURCE_ID);
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
  out.push(["registry.getFull(per-language descriptors)", "resource", () => fullByLanguage]);
  out.push(["registry.getSummary", "resource.summary", () => (globalThis as any).__parityRegistry.getSummary(RESOURCE_ID)]);
  out.push([
    "summaryToMetadata",
    "resource.summary.metadata",
    () => summaryToMetadata((globalThis as any).__parityRegistry.getSummary(RESOURCE_ID)),
  ]);
  out.push([
    "createStaticResourceFromSummary",
    "resource.from_summary",
    () => createStaticResourceFromSummary((globalThis as any).__parityRegistry.getSummary(RESOURCE_ID)),
  ]);
  resourceFixture.tiles.forEach((t: any, i: number) => {
    out.push([`getFull().tiles[${i}]`, `tile:${t.tileid}`, () => full.tiles[i]]);
    out.push([`createStaticTile(tiles[${i}])`, `tile:${t.tileid}`, () => createStaticTile(t)]);
  });
  const resourceText = JSON.stringify(resourceFixture);
  out.push(["tilesToTree", "tree", () => tilesToTree(resourceText, graph)]);
  out.push(["cardsToTree", "tree:cards", () => cardsToTree(resourceText, graph)]);
  out.push(["batchTilesToTrees", "trees:batch", () => batchTilesToTrees(`[${resourceText}]`, graph, true)]);
  // Trees back to tiles (a tile namespace fixes the ids), and merging.
  const treeText = () => JSON.stringify(tilesToTree(resourceText, graph));
  out.push(["treeToTiles", "tiles:tree", () => treeToTiles(treeText(), graph, false, true, null, false, TILE_NAMESPACE)]);
  out.push([
    "batchTreesToTiles",
    "tiles:batch",
    () => batchTreesToTiles(`[${treeText()}]`, graph, false, true, null, false, false, null, TILE_NAMESPACE),
  ]);
  // A graph and its business data built from CSVs.
  const csv = (name: string) => read(`csv/${name}`);
  const csvGraph = () =>
    buildGraphFromModelCsvs(csv("graph.csv"), csv("nodes.csv"), CSV_NAMESPACE, csv("collections.csv"));
  out.push(["buildGraphFromModelCsvs", "csv:graph", csvGraph]);
  out.push([
    "buildResourcesFromBusinessCsv",
    "csv:business_data",
    () => {
      const { graph: built, collections } = csvGraph();
      return buildResourcesFromBusinessCsv(csv("business.csv"), built, collections, "en", true, TILE_NAMESPACE);
    },
  ]);
  out.push([
    "buildPrebuildExport",
    "export:prebuild",
    () => buildPrebuildExport("http://localhost/", [graph.graphid]),
  ]);
  const layerText = JSON.stringify(layerFixture);
  out.push(["mergeResources", "merged", () => mergeResources(`[${resourceText},${layerText}]`)]);
  out.push([
    "batchMergeResources",
    "merged:batch",
    () => batchMergeResources(JSON.stringify([resourceText, layerText]), false, false),
  ]);
  // The factories: core builds each, on either backend.
  graphFixture.nodes.forEach((n: any) => out.push([`createStaticNode(${n.alias})`, `node:${n.alias}`, () => createStaticNode(n)]));
  graphFixture.nodegroups.forEach((g: any) =>
    out.push([`createStaticNodegroup(${g.nodegroupid})`, `nodegroup:${g.nodegroupid}`, () => createStaticNodegroup(g)]),
  );
  graphFixture.edges.forEach((e: any) => out.push([`createStaticEdge(${e.edgeid})`, `edge:${e.edgeid}`, () => createStaticEdge(e)]));
  graphFixture.cards.forEach((c: any) => out.push([`createStaticCard(${c.cardid})`, `card:${c.cardid}`, () => createStaticCard(c)]));
  graphFixture.cards_x_nodes_x_widgets.forEach((x: any) =>
    out.push([`createStaticCardsXNodesXWidgets(${x.id})`, `cxnxw:${x.id}`, () => createStaticCardsXNodesXWidgets(x)]),
  );
  out.push(["createStaticPublication", "publication", () => createStaticPublication(graphFixture.publication)]);
  out.push(["createStaticResource", "resource:constructed", () => createStaticResource(resourceFixture)]);
  out.push(["parseStaticResources", "business_data:resources", () => parseStaticResources(read("business-data.json"))]);
  out.push([
    "registry.populateCachesFromJson",
    "populate_caches",
    () => (globalThis as any).__parityRegistry.populateCachesFromJson(`[${resourceText}]`, graph, true, false, false),
  ]);
  out.push(["resolvePath", "resolve_path:part", () => createInstanceWrapperForModel(graph.graphid).resolvePath("part")]);
  out.push([
    "registry.loadFromBusinessDataBytes",
    "business_data:refs",
    () => createResourceRegistry().loadFromBusinessDataBytes(fs.readFileSync(path.join(DIR, "business-data.json")), true, true),
  ]);
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
  // Lifecycles: the base is Active and the layer copy Draft, so the composed
  // resource is a Conflict.
  const lifecycle = graphFixture.resource_instance_lifecycle;
  const composed = () => composeResourceLayers([layerFixture, resourceFixture], graphFixture, [], false).resource;
  out.push(["resourceLifecycleState", "lifecycle:state", () => resourceLifecycleState(resourceFixture, lifecycle)]);
  out.push(["resourceLifecycleState(composed)", "lifecycle:state:composed", () => resourceLifecycleState(composed(), lifecycle)]);
  out.push(["resourceLifecycleState(no lifecycle)", "lifecycle:state:none", () => resourceLifecycleState(resourceFixture, null)]);
  out.push([
    "transitionResourceLifecycle",
    "lifecycle:transition",
    () => transitionResourceLifecycle(resourceFixture, lifecycle, RETIRED),
  ]);
  out.push([
    "transitionResourceLifecycle(composed, non-strict)",
    "lifecycle:transition:resolve",
    () => transitionResourceLifecycle(composed(), lifecycle, ACTIVE, false),
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

  test("a lifecycle transition is strict unless asked otherwise", () => {
    const lifecycle = graphFixture.resource_instance_lifecycle;
    const draft = { ...resourceFixture, resourceinstance: { ...resourceFixture.resourceinstance } };
    delete draft.resourceinstance.resource_instance_lifecycle_state_id; // initial: Draft
    expect(() => transitionResourceLifecycle(draft, lifecycle, RETIRED)).toThrow(/not permitted/);
    expect(transitionResourceLifecycle(draft, lifecycle, RETIRED, false).event.to).toBe(RETIRED);
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
