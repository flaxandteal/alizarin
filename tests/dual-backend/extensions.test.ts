/**
 * clm's `reference` and filelist's `file-list` through the ORM, on the
 * extensions' current builds: each registers as its JS package does (the
 * package's own code on WASM; the native handler pointer on NAPI, as
 * @alizarin/*-napi's register.js does), then a resource's values are read and
 * rendered for display by core through the registered handlers. Both backends
 * must give the same values. Python checks the same in
 * crates/alizarin-python/tests/test_extensions.py.
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
} from '../../js/backend';
import * as wasm from '../../pkg/alizarin';
import { ResourceModelWrapper, GraphMutator } from '../../js/graphManager';
import type { ResourceInstanceWrapper } from '../../js/graphManager';
import { ResourceInstanceViewModel } from '../../js/viewModels/ResourceInstanceViewModel';
import { createStaticGraph, StaticCollection, StaticConcept } from '../../js/static-types';
import { RDM } from '../../js/rdm';
import { nodeConfigManager } from '../../js/nodeConfig';

const CRM = "http://www.cidoc-crm.org/cidoc-crm/";
const RESOURCE_ID = "eeeeeeee-1111-2222-3333-444444444444";
const LIST_ID = "2730d609-3a8d-49dc-bf51-6ac34e80294a";

/** clm's fixture reference (ext/clm/js/tests/data). */
const REFERENCE = [{
  labels: [{
    id: "0ea39e2e-6663-467c-8707-ab492896d23e", language_id: "en",
    list_item_id: "6672d187-dfc3-4424-8c63-7a3b377b4159", value: "Item 1>1", valuetype_id: "prefLabel",
  }],
  list_id: LIST_ID,
  uri: "http://localhost:8000/plugins/controlled-list-manager/item/6672d187-dfc3-4424-8c63-7a3b377b4159",
}];
const FILES = [
  { name: "file1.pdf", url: "/files/1", file_id: "id-1", type: "application/pdf" },
  { name: "file2.jpg", url: "/files/2", file_id: "id-2", type: "image/jpeg" },
];

beforeAll(async () => {
  // Each package registers its WASM handler and view models on import.
  await import('../../ext/clm/js/src/main');
  await import('../../ext/filelist/js/src/main');
  if (getBackend() === 'napi') {
    // What importing @alizarin/clm-napi and @alizarin/filelist-napi does,
    // against this host.
    const require = createRequire(import.meta.url);
    require('../../ext/clm/napi/registration.js').register(getNapiModule());
    require('../../ext/filelist/napi/registration.js').register(getNapiModule());
  }
});

/** A record with a file-list and a reference node, and a value in each. */
async function record(kind: unknown = REFERENCE): Promise<any> {
  const mutator = new GraphMutator(createStaticGraph({ name: "Record", author: "Test" }));
  const add = (mutator as any)._addGenericNode.bind(mutator);
  // Added as strings, then given their datatypes: JS has no widget registry
  // binding, so GraphMutator cannot add extension datatypes (A-6).
  add(null, "attachments", "Attachments", "1", "string", `${CRM}E31_Document`, `${CRM}P70i_is_documented_in`);
  add(null, "kind", "Kind", "1", "string", `${CRM}E55_Type`, `${CRM}P2_has_type`);
  const raw = JSON.parse(JSON.stringify(mutator.apply()));
  const node = (alias: string) => raw.nodes.find((n: any) => n.alias === alias);
  node("attachments").datatype = "file-list";
  node("kind").datatype = "reference";
  node("kind").config = { controlledList: LIST_ID, multiValue: true };
  const graph = parseStaticGraph(JSON.stringify({ graph: [raw] }));
  // As loadGraph does: view models read node configs (a reference's
  // multiValue) from here.
  nodeConfigManager.loadFromGraph(graph);
  const wkrm = createWKRM(createStaticGraphMeta({
    graphid: graph.graphid, name: "Record", slug: "record",
    relatable_resource_model_ids: [], resource_2_resource_constraints: [], extra_fields: {},
  }));
  class Record extends ResourceInstanceViewModel<any> {
    static _: ResourceInstanceWrapper<any> | null;
    static __: ResourceModelWrapper<any> | null;
  }
  const wrapper = new ResourceModelWrapper(wkrm, graph, Record, true);
  Record.prototype.__ = wrapper;
  const tile = (alias: string, value: unknown) => ({
    tileid: `${alias}-tile`, nodegroup_id: node(alias).nodegroup_id, resourceinstance_id: RESOURCE_ID,
    parenttile_id: null, data: { [node(alias).nodeid]: value }, sortorder: 0, provisionaledits: null,
  });
  const resources = parseStaticResources(JSON.stringify({
    business_data: {
      resources: [{
        resourceinstance: {
          descriptors: { name: "Record", description: "", map_popup: "", slug: "record" },
          graph_id: graph.graphid, legacyid: null, name: "Record", resourceinstanceid: RESOURCE_ID,
        },
        tiles: [tile("attachments", FILES), tile("kind", kind)],
        tiles_loaded: true,
      }],
    },
  }));
  return wrapper.fromStaticResource(resources[0], false, false);
}

describe(`extensions through the ORM [${getBackend()} backend]`, () => {
  test("both handlers are registered with the backend", () => {
    const native: any = getBackend() === 'napi' ? getNapiModule() : wasm;
    expect(native.hasExtensionHandler("reference")).toBe(true);
    expect(native.hasExtensionHandler("file-list")).toBe(true);
  });

  test("values read through the ORM", async () => {
    const asset = await record();
    const files = await asset.attachments;
    expect(files).toHaveLength(2);
    // Item by item: how a list view model stringifies differs per binding
    // (PARITY-RESEARCH.md, G-6).
    expect([...(await asset.kind)].map(String)).toEqual(["Item 1>1"]);
  });

  test("core renders both for display through their handlers", async () => {
    const asset = await record();
    const display = await asset.forDisplayJson(true, "en");
    // A multi-value reference displays as a list (clm's displayAsList).
    expect(display.root).toEqual({ attachments: "file1.pdf, file2.jpg", kind: ["Item 1>1"] });
  });

  test("a reference's URI is built on the base URI set through @alizarin/clm", async () => {
    // A concept with no stored URI, looked up from a bare id: its URI is built,
    // on the base set here, by clm's Rust (as Python's set_clm_base_uri sets it).
    const clm: any = await import('../../ext/clm/js/src/main');
    const collection = StaticCollection.fromConceptScheme({
      collectionid: LIST_ID, name: "Kinds",
      conceptScheme: StaticConcept.fromValue(null, "Kinds", ["Bowl"]),
    });
    RDM.addCollection(collection);
    const bowl = Object.keys((collection as any).__allConcepts).find(
      (id) => (collection as any).__allConcepts[id].prefLabels?.en?.value === "Bowl",
    )!;
    clm.setClmBaseUri("https://lists.example.org/item/");
    try {
      expect(clm.getClmBaseUri()).toBe("https://lists.example.org/item/");
      const asset = await record([bowl]);
      const [kind] = [...(await asset.kind)];
      expect((await kind.getValue()).uri).toBe(`https://lists.example.org/item/${bowl}`);
    } finally {
      clm.clearClmBaseUri();
    }
    // What keeps it for core's marker resolution: on NAPI the native ext (which
    // @alizarin/clm forwards to once it has loaded it), on WASM clm's WASM build.
    const holder = getBackend() === 'napi'
      ? createRequire(import.meta.url)('../../ext/clm/napi/registration.js').binding
      : clm;
    holder.setClmBaseUri("https://lists.example.org/item/");
    expect(holder.getClmBaseUri()).toBe("https://lists.example.org/item/");
    holder.clearClmBaseUri();
    expect(holder.getClmBaseUri()).toBeNull();
  });

  test("an edit is coerced by the handler before it is stored", async () => {
    const asset = await record();
    asset.attachments = [FILES[1]];
    const delta = asset.$commit();
    expect(delta.upserted.map((t: any) => Object.values(t.data)[0])).toEqual([[{
      accepted: false, content: null, file_id: "id-2", index: 0, last_modified: null, name: "file2.jpg",
      path: null, renderer: null, selected: false, size: null, status: null, type: "image/jpeg", url: "/files/2",
    }]]);
  });
});
