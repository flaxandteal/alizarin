/**
 * Dual-backend Aggregate functions: a Layer resource's fold Aggregate, run over a
 * resource registry, writes the same node values on WASM and NAPI. Python checks
 * the same fixture against the same expected tiles
 * (crates/alizarin-python/tests/test_aggregate.py), so all three agree.
 *
 * Runs under both backends via:
 *   npx vitest run tests/dual-backend/aggregate.test.ts (WASM)
 *   npx vitest run -c vitest.napi.config.js tests/dual-backend/aggregate.test.ts (NAPI)
 */
import { test, describe, expect, beforeAll } from "vitest";
import { ResourceModelWrapper, createWKRM } from '../../js/graphManager';
import {
  getBackend,
  createResourceRegistry,
  createStaticGraphMeta,
  parseStaticGraph,
  registerExtensionHandler,
} from '../../js/backend';
import BuildingGraph from "../data/aggregate/building.json";
import LayerGraph from "../data/aggregate/layer.json";
import LayerGeoGraph from "../data/aggregate/layer_geo.json";
import BusinessData from "../data/aggregate/business_data.json";
import Expected from "../data/aggregate/expected_layer_tiles.json";
import ExpectedGeo from "../data/aggregate/expected_geo_tiles.json";

beforeAll(async () => {
  if (getBackend() !== 'napi') {
    // As @alizarin/geo does: register its WASM build's fold as the handler's.
    const geo = await import('../../ext/geo/wasm/pkg/alizarin_geo_wasm');
    const bytes = (await import('../../ext/geo/wasm/pkg/alizarin_geo_wasm_bg.wasm')).default;
    geo.initSync(bytes);
    registerExtensionHandler('geojson-feature-collection', {
      foldIdentity: () => geo.geoFoldIdentity(),
      fold: (acc: unknown, values: unknown[]) => geo.geoFold(acc, values),
      foldMerge: (a: unknown, b: unknown) => geo.geoFoldMerge(a, b),
      foldRemove: (acc: unknown, values: unknown[]) => geo.geoFoldRemove(acc, values),
    });
  }
});

/** Register a graph with core, the way loading a model does. */
function register(graphJson: any): void {
  const graph = parseStaticGraph(JSON.stringify({ graph: [graphJson] }));
  const wkrm = createWKRM(
    createStaticGraphMeta({
      graphid: graph.graphid,
      name: graphJson.name?.en ?? graph.graphid,
      slug: graph.graphid,
      relatable_resource_model_ids: [],
      resource_2_resource_constraints: [],
      extra_fields: {},
    }),
  );
  new ResourceModelWrapper(wkrm, graph, undefined, true);
}

/** A Layer resource's tiles as plain JSON, keyed by tile id. */
function layerTiles(registry: any, layerId: string = Expected.layer_id): Record<string, any> {
  const full = JSON.parse(JSON.stringify(registry.getFull(layerId)));
  return Object.fromEntries(
    full.tiles.map((t: any) => [t.tileid, { nodegroup_id: t.nodegroup_id, data: t.data }]),
  );
}

describe(`Aggregate functions [${getBackend()} backend]`, () => {
  register(BuildingGraph);
  register(LayerGraph);
  register(LayerGeoGraph);

  // NAPI reaches geo's handler over the C ABI, which has no fold pointers yet,
  // so geometry folds on WASM only for now.
  test.skipIf(getBackend() === 'napi')(
    "geometry folds through geo's handler into an extent",
    () => {
      const registry = createResourceRegistry();
      registry.loadFromBusinessDataBytes(
        new TextEncoder().encode(JSON.stringify(BusinessData)),
        true,
        true,
      );
      registry.applyAggregateFunctions(ExpectedGeo.layer_id);
      expect(layerTiles(registry, ExpectedGeo.layer_id)).toEqual(ExpectedGeo.tiles);
    },
  );

  test("the fold Aggregate writes the expected node values into the Layer's tiles", () => {
    const registry = createResourceRegistry();
    registry.loadFromBusinessDataBytes(
      new TextEncoder().encode(JSON.stringify(BusinessData)),
      true,
      true,
    );
    registry.applyAggregateFunctions(Expected.layer_id);
    expect(layerTiles(registry)).toEqual(Expected.tiles);
  });

  test("re-running gives identical tiles", () => {
    const registry = createResourceRegistry();
    registry.loadFromBusinessDataBytes(
      new TextEncoder().encode(JSON.stringify(BusinessData)),
      true,
      true,
    );
    registry.applyAggregateFunctions(Expected.layer_id);
    const first = layerTiles(registry);
    registry.applyAggregateFunctions(Expected.layer_id);
    expect(layerTiles(registry)).toEqual(first);
  });

  test("an unknown target is an error", () => {
    const registry = createResourceRegistry();
    expect(() => registry.applyAggregateFunctions("no-such-resource")).toThrow(/not loaded in full/);
  });
});
