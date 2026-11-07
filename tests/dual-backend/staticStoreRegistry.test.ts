/**
 * Dual-backend StaticStore tests: the registry calls StaticStore makes, and the
 * ORM read-back path built on them (`all()` -> `getRootViewModel` -> `toSnapshot`).
 *
 * The native registry returns plain JSON where wasm returns class instances, so
 * everything here goes through StaticStore / ResourceModelWrapper and never
 * touches backend-specific classes.
 *
 * Runs under both backends via:
 *   npx vitest run tests/dual-backend/staticStoreRegistry.test.ts (WASM)
 *   npx vitest run -c vitest.napi.config.js tests/dual-backend/staticStoreRegistry.test.ts (NAPI)
 */
import { test, describe, expect } from "vitest";
import { ResourceModelWrapper, createWKRM } from '../../js/graphManager';
import {
  getBackend,
  createResourceRegistry,
  createStaticGraphMeta,
  parseStaticGraph,
} from '../../js/backend';
import { StaticStore } from '../../js/staticStore';
import { ResourceInstanceViewModel } from '../../js/viewModels';
import * as GroupJSON from "../data/models/Group.json";
import * as GroupResourceJSON from "../definitions/resources/_07883c9e-b25c-11e9-975a-a4d18cec433a.json";

class Group extends ResourceInstanceViewModel<Group> {}

const GRAPH_ID = '07883c9e-b25c-11e9-975a-a4d18cec433a';
const RESOURCE_ID = 'd2368123-9628-49a2-b3dd-78ac6ee3e911';
const TILE_COUNT = 3;

function businessDataBytes(): Uint8Array {
  const business_data = (GroupResourceJSON as any).business_data;
  return new TextEncoder().encode(JSON.stringify({ business_data }));
}

function loadedRegistry(storeFull: boolean = true): any {
  const registry = createResourceRegistry();
  registry.loadFromBusinessDataBytes(businessDataBytes(), storeFull, true);
  return registry;
}

function loadedStore(storeFull: boolean = true): StaticStore {
  return new StaticStore(loadedRegistry(storeFull));
}

/** An archesClient serving from a registry the store under test can't see. */
function remoteClient(): any {
  const source = loadedRegistry();
  return {
    registry: null,
    getResources: async () => [source.getFull(RESOURCE_ID)],
    getResource: async () => source.getFull(RESOURCE_ID),
    getResourceSummaries: async () => [source.getSummary(RESOURCE_ID)],
    getResourceTiles: async () => source.getFull(RESOURCE_ID).tiles,
  };
}

function modelFor(store: StaticStore): ResourceModelWrapper<any> {
  const graph = parseStaticGraph(JSON.stringify(GroupJSON));
  const wkrm = createWKRM(createStaticGraphMeta({
    graphid: GRAPH_ID,
    name: "Group",
    slug: "group",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {},
  }));
  return new ResourceModelWrapper(wkrm, graph, Group, true, store);
}

async function collect<T>(items: AsyncIterable<T>): Promise<T[]> {
  const out: T[] = [];
  for await (const item of items) out.push(item);
  return out;
}

describe(`StaticStore registry [${getBackend()} backend]`, () => {
  describe("registry surface", () => {
    test("getAllFullForGraph returns full resources for the graph only", () => {
      const resources = loadedRegistry().getAllFullForGraph(GRAPH_ID);
      expect(resources).toHaveLength(1);
      expect(resources[0].resourceinstance.resourceinstanceid).toBe(RESOURCE_ID);
      expect(resources[0].tiles).toHaveLength(TILE_COUNT);
    });

    test("getAllFullForGraph is empty for an unknown or missing id", () => {
      const registry = loadedRegistry();
      expect(registry.getAllFullForGraph('no-such-graph')).toEqual([]);
      expect(registry.getAllFullForGraph()).toEqual([]);
      expect(registry.getAllFullForGraph(null)).toEqual([]);
    });

    test("getAllFullForGraph skips summary-only entries", () => {
      expect(loadedRegistry(false).getAllFullForGraph(GRAPH_ID)).toEqual([]);
    });

    test("mergeFromResources stores full by flag and summaries otherwise", () => {
      const source = loadedRegistry();
      const full = createResourceRegistry();
      full.mergeFromResources([source.getFull(RESOURCE_ID)], true, true);
      expect(full.hasFull(RESOURCE_ID)).toBe(true);

      const summaries = createResourceRegistry();
      summaries.mergeFromResources([source.getFull(RESOURCE_ID)]);
      expect(summaries.contains(RESOURCE_ID)).toBe(true);
      expect(summaries.hasFull(RESOURCE_ID)).toBe(false);
    });

    test("insert adds a summary without tiles", () => {
      const registry = createResourceRegistry();
      registry.insert(loadedRegistry().getSummary(RESOURCE_ID));
      expect(registry.contains(RESOURCE_ID)).toBe(true);
      expect(registry.hasFull(RESOURCE_ID)).toBe(false);
    });
  });

  describe("StaticStore from a loaded registry", () => {
    test("getMeta returns the resource metadata", async () => {
      const meta = await loadedStore().getMeta(RESOURCE_ID);
      expect(meta?.resourceinstanceid).toBe(RESOURCE_ID);
      expect(meta?.graph_id).toBe(GRAPH_ID);
      expect(meta?.name).toBe('Global Group');
    });

    test("getMeta returns null for an unknown id", async () => {
      expect(await loadedStore().getMeta('no-such-resource')).toBeNull();
    });

    test("loadAll yields only the requested graph", async () => {
      const store = loadedStore();
      expect(await collect(store.loadAll(GRAPH_ID))).toHaveLength(1);
      expect(await collect(store.loadAll('no-such-graph'))).toHaveLength(0);
    });

    test("loadOne returns the resource", async () => {
      const resource = await loadedStore().loadOne(RESOURCE_ID);
      expect(resource.resourceinstance.name).toBe('Global Group');
    });

    test("loadTiles returns tiles, filtered by nodegroup", async () => {
      const store = loadedStore();
      const tiles = await store.loadTiles(RESOURCE_ID);
      expect(tiles).toHaveLength(TILE_COUNT);
      const filtered = await store.loadTiles(RESOURCE_ID, tiles[0].nodegroup_id);
      expect(filtered.length).toBeGreaterThan(0);
      expect(filtered.every((tile: any) => tile.nodegroup_id === tiles[0].nodegroup_id)).toBe(true);
    });

    test("ensureFullResource returns the resource with tiles", async () => {
      const resource = await loadedStore().ensureFullResource(RESOURCE_ID);
      expect(resource.tiles).toHaveLength(TILE_COUNT);
    });

    test("loadAllSummaries yields summaries for the graph", async () => {
      const summaries = await collect(loadedStore().loadAllSummaries(GRAPH_ID));
      expect(summaries).toHaveLength(1);
      expect(summaries[0].resourceinstanceid).toBe(RESOURCE_ID);
      expect(summaries[0].name).toBe('Global Group');
    });
  });

  describe("StaticStore falling back to archesClient", () => {
    function remoteStore(): StaticStore {
      const store = new StaticStore(createResourceRegistry());
      store.archesClient = remoteClient();
      return store;
    }

    test("loadAll merges remote resources into the registry", async () => {
      const store = remoteStore();
      const resources = await collect(store.loadAll(GRAPH_ID));
      expect(resources).toHaveLength(1);
      expect(store.hasFull(RESOURCE_ID)).toBe(true);
    });

    test("loadOne merges the remote resource into the registry", async () => {
      const store = remoteStore();
      const resource = await store.loadOne(RESOURCE_ID);
      expect(resource.tiles).toHaveLength(TILE_COUNT);
      expect(store.hasFull(RESOURCE_ID)).toBe(true);
    });

    test("ensureFullResource merges the remote resource into the registry", async () => {
      const store = remoteStore();
      const resource = await store.ensureFullResource(RESOURCE_ID);
      expect(resource.tiles).toHaveLength(TILE_COUNT);
      expect(store.hasFull(RESOURCE_ID)).toBe(true);
    });

    test("loadAllSummaries inserts remote summaries into the registry", async () => {
      const store = remoteStore();
      const summaries = await collect(store.loadAllSummaries(GRAPH_ID));
      expect(summaries).toHaveLength(1);
      expect(store.contains(RESOURCE_ID)).toBe(true);
      expect(store.hasFull(RESOURCE_ID)).toBe(false);
    });
  });

  describe("ORM read-back", () => {
    test("all() -> getRootViewModel -> toSnapshot", async () => {
      const instances = await modelFor(loadedStore()).all();
      expect(instances).toHaveLength(1);

      const instance = instances[0];
      const root = await instance.$.getRootViewModel();

      const pseudo = await instance.$.getRoot();
      const snapshot = pseudo.getSnapshot();
      expect(snapshot.datatype).toBe('semantic');

      const basicInfo = await root.basic_info;
      const name = await (await basicInfo[0]).name;
      expect(String(await name)).toBe('Global Group');
    });

    test("all() on an empty registry is empty", async () => {
      const empty = modelFor(new StaticStore(createResourceRegistry()));
      expect(await empty.all()).toHaveLength(0);
    });

    test("allSummaries() -> loadFullResource reads the same data", async () => {
      const model = modelFor(loadedStore());
      const summaries = await model.allSummaries();
      expect(summaries).toHaveLength(1);
      expect(summaries[0].id).toBe(RESOURCE_ID);

      const full = await model.loadFullResource(RESOURCE_ID);
      const root = await full.$.getRootViewModel();
      const basicInfo = await root.basic_info;
      expect(String(await (await basicInfo[0]).name)).toBe('Global Group');
    });

    test("find() reads through the registry", async () => {
      const instance = await modelFor(loadedStore()).find(RESOURCE_ID, false);
      const root = await instance.$.getRootViewModel();
      const basicInfo = await root.basic_info;
      expect(String(await (await basicInfo[0]).name)).toBe('Global Group');
    });
  });
});
