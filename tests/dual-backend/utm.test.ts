/**
 * UTM through the derive-provider bridge, end to end: geo registers its UTM
 * provider as its JS package does (a native DeriveProviderInfo pointer on NAPI,
 * a JS callback over its WASM build on WASM), a graph declares it in stored
 * mode with named nodes, and editing the geometry through the ORM reruns it on
 * $commit(), returning the derived tile in the delta. Python checks the same
 * in crates/alizarin-python/tests/test_utm.py.
 *
 * Runs under both:
 *   npx vitest run
 *   npx vitest run --config vitest.napi.config.js
 */
import { beforeAll, describe, expect, test } from "vitest";
import { createRequire } from "module";
import {
  parseStaticGraph,
  parseStaticResources,
  createStaticGraphMeta,
  createWKRM,
  getBackend,
  getNapiModule,
  registerDeriveProvider,
  hasDeriveProvider,
} from '../../js/backend';
import { ResourceModelWrapper, GraphMutator } from '../../js/graphManager';
import type { ResourceInstanceWrapper } from '../../js/graphManager';
import { ResourceInstanceViewModel } from '../../js/viewModels/ResourceInstanceViewModel';
import { createStaticGraph } from '../../js/static-types';
import { getCurrentLanguage, setCurrentLanguage } from '../../js/utils';

const CRM = "http://www.cidoc-crm.org/cidoc-crm/";
const COMPUTE_TILES = "60000000-0000-0000-0000-000000000002";
const RESOURCE_ID = "ffffffff-1111-2222-3333-444444444444";

/** London (51.5074°N, 0.1278°W): zone 30N, easting ≈ 699 320 m. */
const LONDON = {
  type: "FeatureCollection",
  features: [{ type: "Feature", properties: {}, geometry: { type: "Point", coordinates: [-0.1278, 51.5074] } }],
};

beforeAll(async () => {
  if (getBackend() === 'napi') {
    // As @alizarin/geo does on import, for the NAPI backend.
    const geoNapi = createRequire(import.meta.url)('../../ext/geo/napi/index.js');
    getNapiModule().registerDeriveProvider(geoNapi.utmDeriveProviderPtr());
  } else {
    // As @alizarin/geo does on import, for the WASM backend.
    const geo = await import('../../ext/geo/wasm/pkg/alizarin_geo_wasm');
    geo.initSync((await import('../../ext/geo/wasm/pkg/alizarin_geo_wasm_bg.wasm')).default);
    registerDeriveProvider('utm', {
      derive: (resourceId: string, tiles: unknown, config: unknown, lookup: any) =>
        geo.utmDerive(resourceId, tiles, config, lookup),
    });
  }
});

/** A place with a `location` geometry and a node for UTM's `role` output, of
 *  `datatype`, declared as stored. */
async function place(
  role = "easting", datatype = "number",
): Promise<{ asset: any; node: (alias: string) => any }> {
  const mutator = new GraphMutator(createStaticGraph({ name: "Place", author: "Test" }));
  const add = (mutator as any)._addGenericNode.bind(mutator);
  add(null, "location", "Location", "1", "geojson-feature-collection", `${CRM}E53_Place`, `${CRM}P168_place_is_defined_by`);
  add(null, role, role, "1", datatype, `${CRM}E54_Dimension`, `${CRM}P43_has_dimension`);
  const raw = JSON.parse(JSON.stringify(mutator.apply()));
  const node = (alias: string) => raw.nodes.find((n: any) => n.alias === alias);
  raw.functions_x_graphs = [{
    id: "fxg-utm",
    function_id: COMPUTE_TILES,
    graph_id: raw.graphid,
    config: {
      provider: "utm",
      nodegroup: node(role).nodegroup_id,
      mode: "stored",
      sources: [node("location").nodegroup_id],
      nodes: { geometry: "location", [role]: role },
    },
  }];
  const graph = parseStaticGraph(JSON.stringify({ graph: [raw] }));
  const wkrm = createWKRM(createStaticGraphMeta({
    graphid: graph.graphid, name: "Place", slug: "place",
    relatable_resource_model_ids: [], resource_2_resource_constraints: [], extra_fields: {},
  }));
  class Place extends ResourceInstanceViewModel<any> {
    static _: ResourceInstanceWrapper<any> | null;
    static __: ResourceModelWrapper<any> | null;
  }
  const wrapper = new ResourceModelWrapper(wkrm, graph, Place, true);
  Place.prototype.__ = wrapper;
  const resources = parseStaticResources(JSON.stringify({
    business_data: {
      resources: [{
        resourceinstance: {
          descriptors: { name: "Town", description: "", map_popup: "", slug: "town" },
          graph_id: graph.graphid, legacyid: null, name: "Town", resourceinstanceid: RESOURCE_ID,
        },
        tiles: [],
        tiles_loaded: true,
      }],
    },
  }));
  return { asset: await wrapper.fromStaticResource(resources[0], false, false), node };
}

describe(`UTM through the derive bridge [${getBackend()} backend]`, () => {
  test("geo's provider is registered", () => {
    expect(hasDeriveProvider("utm")).toBe(true);
  });

  test("editing the geometry reruns UTM in the commit", async () => {
    const { asset, node } = await place();
    asset.location = LONDON;
    const delta = asset.$commit();

    const easting = delta.upserted.find((t: any) => t.nodegroup_id === node("easting").nodegroup_id);
    expect(easting).toBeDefined();
    expect(Math.abs(easting.data[node("easting").nodeid] - 699_320)).toBeLessThan(50);
    // Stored with the resource, so it reads back like any value.
    expect(Math.abs(Number(await asset.easting) - 699_320)).toBeLessThan(50);
  });

  test("a derived string is in the host's language", async () => {
    // The provider runs in geo's own module (or JS); the value is still keyed
    // by the language set here, in the host.
    const previous = getCurrentLanguage();
    setCurrentLanguage("ga");
    try {
      const { asset, node } = await place("hemisphere", "string");
      asset.location = LONDON;
      const delta = asset.$commit();
      const tile = delta.upserted.find((t: any) => t.nodegroup_id === node("hemisphere").nodegroup_id);
      expect(tile.data[node("hemisphere").nodeid]).toEqual({ ga: { value: "N", direction: "ltr" } });
    } finally {
      setCurrentLanguage(previous);
    }
  });

  test("clearing the geometry removes the derived tile", async () => {
    const { asset } = await place();
    asset.location = LONDON;
    asset.$commit();
    asset.location = null;
    const delta = asset.$commit();
    expect(delta.removed).toHaveLength(1);
  });
});
