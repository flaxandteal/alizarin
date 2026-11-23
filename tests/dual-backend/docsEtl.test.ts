/**
 * The "Batch and ETL" guide (docs/batch-etl.mdx), run as written against
 * docs/example. The client serves graphs only: every resource comes from
 * business data loaded into the registry, so finding and following
 * references must not need the client. If this fails, the docs are wrong.
 *
 * Runs under both backends via:
 *   npx vitest run tests/dual-backend/docsEtl.test.ts (WASM)
 *   npx vitest run -c vitest.napi.config.js tests/dual-backend/docsEtl.test.ts (NAPI)
 */
import { beforeAll, describe, expect, test } from "vitest";
import * as fs from "fs";
import * as path from "path";
import { client, graphManager, staticStore, RDM } from "../../js/main";

const EXAMPLE = path.join(__dirname, "../../docs/example");
const PERSON_ID = "193750d0-e4b8-527d-8dc2-cbfb4cd31e72";
const PERSON_GRAPH = "91e13029-1637-5fbb-ac8a-2dd0c6502cc7";

describe("docs: batch and ETL", () => {
  let refs: any[] = [];

  beforeAll(async () => {
    // --- docs/batch-etl.mdx: graphs from the client, nothing else ---
    const local = new client.ArchesClientLocal({
      allGraphFile: () => `${EXAMPLE}/resource_models/_all.json`,
      graphIdToGraphFile: (graphId) => `${EXAMPLE}/resource_models/${graphId}.json`,
      graphIdToResourcesFiles: () => [],
      resourceIdToFile: () => `${EXAMPLE}/no-such-file.json`,
      collectionIdToFile: (collectionId) => `${EXAMPLE}/collections/${collectionId}.json`,
    });
    graphManager.archesClient = local;
    staticStore.archesClient = local;
    RDM.archesClient = local;
    await graphManager.initialize({ defaultAllowAllNodegroups: true });

    // --- every business data file, straight into the registry ---
    const dir = `${EXAMPLE}/business_data`;
    for (const file of fs.readdirSync(dir).filter((f) => f.endsWith(".json")).sort()) {
      const bytes = fs.readFileSync(`${dir}/${file}`);
      refs.push(...staticStore.registry.loadFromBusinessDataBytes(bytes, true, true));
    }
  });

  test("loading returns a ref per resource, public only with a public __scopes", () => {
    expect(refs).toHaveLength(64);
    const person = refs.find((r) => r.resourceinstanceid === PERSON_ID);
    // The example's resources have no __scopes, so none is public.
    expect(person).toEqual({ resourceinstanceid: PERSON_ID, graph_id: PERSON_GRAPH, isPublic: false });
  });

  test("the registry answers directly", () => {
    const registry = staticStore.registry;
    expect(registry.length).toBe(64);
    expect(registry.contains(PERSON_ID)).toBe(true);
    expect(registry.hasFull(PERSON_ID)).toBe(true);
    const full = registry.getFull(PERSON_ID);
    expect(full.resourceinstance.graph_id).toBe(PERSON_GRAPH);
    expect(full.tiles.length).toBeGreaterThan(0);
    expect(registry.getSummary(PERSON_ID)).not.toBeNull();
    expect(registry.getAllFullForGraph(PERSON_GRAPH)).toHaveLength(15);
    expect(registry.getFull("00000000-0000-4000-8000-000000000000")).toBeNull();
  });

  test("find() and references read the registry, not the client", async () => {
    const Person = await graphManager.loadGraph("Person");
    const person = await Person.find(PERSON_ID);
    expect(String(await person.name)).toBe("Oisín Ó Dálaigh");
    const site = await person.primary_site;
    const org = await site.maintained_by;
    expect(String(await org.name)).toBe("City Council -- Planning & GIS Division");
  });

  test("a resource in no loaded file falls back to the client, and says so", async () => {
    await expect(staticStore.loadOne("00000000-0000-4000-8000-000000000000")).rejects.toThrow(
      /resourceIdToFile/,
    );
  });
});
