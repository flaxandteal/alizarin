/**
 * A derive provider registered from JS (the WASM backend's
 * registerDeriveProvider): core calls it with the resource's tiles, its
 * declaration with `nodes` resolved, and a `lookup` into the graph, and stores
 * or loads what it returns. WASM-only: on NAPI and Python a provider comes
 * from an extension's native module (covered with the geo extension).
 */
import { describe, expect, test } from "vitest";
import {
  parseStaticGraph,
  parseStaticResources,
  createStaticGraphMeta,
  createWKRM,
  registerDeriveProvider,
  hasDeriveProvider,
} from '../js/backend';
import { ResourceModelWrapper, GraphMutator } from '../js/graphManager';
import type { ResourceInstanceWrapper } from '../js/graphManager';
import { ResourceInstanceViewModel } from '../js/viewModels/ResourceInstanceViewModel';
import { createStaticGraph } from '../js/static-types';

const LITERAL = "http://www.w3.org/2000/01/rdf-schema#Literal";
const NOTE = "http://www.cidoc-crm.org/cidoc-crm/P3_has_note";
const COMPUTE_TILES = "60000000-0000-0000-0000-000000000002";
const PROVIDER = "test-js-shout";
const RESOURCE_ID = "dddddddd-1111-2222-3333-444444444444";

/** What the provider saw on its last call. */
const seen: { config?: any; lookedUp?: any } = {};

registerDeriveProvider(PROVIDER, {
  // `shout` = `name` in upper case: `name` from the declared nodes, `shout`
  // looked up through the graph handoff.
  derive(resourceId: string, tiles: any[], config: any, lookup: (kind: number, key: string) => any) {
    seen.config = config;
    const name = config.resolved.source.nodeid;
    const shout = lookup(1, "shout");
    seen.lookedUp = shout;
    const value = tiles.find((t) => t.data[name])?.data[name]?.en?.value;
    if (!value) {
      return [];
    }
    return [{
      nodegroup_id: config.nodegroup,
      resourceinstance_id: resourceId,
      tileid: null,
      parenttile_id: null,
      sortorder: null,
      provisionaledits: null,
      data: { [shout.nodeid]: { en: { direction: "ltr", value: value.toUpperCase() } } },
    }];
  },
});

async function place() {
  const mutator = new GraphMutator(createStaticGraph({ name: "Place", author: "Test" }));
  mutator.addStringNode(null, "name", "Name", "1", LITERAL, NOTE);
  mutator.addStringNode(null, "shout", "Shout", "1", LITERAL, NOTE);
  const raw = JSON.parse(JSON.stringify(mutator.apply()));
  const node = (alias: string) => raw.nodes.find((n: any) => n.alias === alias);
  raw.functions_x_graphs = [{
    id: "fxg-shout",
    function_id: COMPUTE_TILES,
    graph_id: raw.graphid,
    config: {
      provider: PROVIDER,
      nodegroup: node("shout").nodegroup_id,
      member_of: "",
      nodes: { source: "name" },
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
        tiles: [{
          data: { [node("name").nodeid]: { en: { direction: "ltr", value: "Baile" } } },
          nodegroup_id: node("name").nodegroup_id, parenttile_id: null,
          resourceinstance_id: RESOURCE_ID, sortorder: 0,
          tileid: "aaaaaaaa-0000-0000-0000-000000000001",
        }],
        tiles_loaded: true,
      }],
    },
  }));
  return { asset: await wrapper.fromStaticResource(resources[0], false, false) as any, node };
}

describe("a derive provider registered from JS", () => {
  test("is registered under its id", () => {
    expect(hasDeriveProvider(PROVIDER)).toBe(true);
    expect(hasDeriveProvider("no-such-provider")).toBe(false);
  });

  test("derives on load, with resolved nodes and graph lookups", async () => {
    const { asset, node } = await place();
    expect(`${await asset.shout}`).toBe("BAILE");
    expect(seen.config.resolved.source.nodeid).toBe(node("name").nodeid);
    expect(seen.lookedUp.alias).toBe("shout");

    // On-load tiles are never exported as stored data.
    const exported = JSON.parse(asset.$.wasmWrapper.exportTilesJson());
    expect(exported.map((t: any) => t.nodegroup_id)).toEqual([node("name").nodegroup_id]);
  });
});
