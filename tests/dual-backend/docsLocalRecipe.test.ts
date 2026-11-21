/**
 * The "Local testing" recipe and the resource-reference walkthrough in
 * docs/local-testing.mdx and docs/resource-references.mdx, run as written
 * against docs/example (the prebuild the docs site serves). If this fails,
 * the docs are wrong.
 *
 * Runs under both backends via:
 *   npx vitest run tests/dual-backend/docsLocalRecipe.test.ts (WASM)
 *   npx vitest run -c vitest.napi.config.js tests/dual-backend/docsLocalRecipe.test.ts (NAPI)
 */
import { beforeAll, describe, expect, test } from "vitest";
import * as path from "path";
import { client, graphManager, staticStore, RDM } from "../../js/main";

const EXAMPLE = path.join(__dirname, "../../docs/example");
const PERSON_ID = "193750d0-e4b8-527d-8dc2-cbfb4cd31e72";
const SITE_ID = "bb8ec53b-0d84-5d36-95a9-e02fdd046f24";

describe("docs: local testing recipe", () => {
  beforeAll(async () => {
    // --- docs/local-testing.mdx: setup ---
    const local = new client.ArchesClientLocal({
      allGraphFile: () => `${EXAMPLE}/resource_models/_all.json`,
      graphIdToGraphFile: (graphId) => `${EXAMPLE}/resource_models/${graphId}.json`,
      graphIdToResourcesFiles: (graphId) => [`${EXAMPLE}/business_data/_${graphId}.json`],
      resourceIdToFile: () => `${EXAMPLE}/business_data/_91e13029-1637-5fbb-ac8a-2dd0c6502cc7.json`,
      collectionIdToFile: (collectionId) => `${EXAMPLE}/collections/${collectionId}.json`,
    });
    graphManager.archesClient = local;
    staticStore.archesClient = local;
    RDM.archesClient = local;
    await graphManager.initialize();

    // Load every graph's resources into the registry, so references between
    // them resolve without going back to the client.
    for (const name of ["Person", "Site", "Organisation"]) {
      await (await graphManager.loadGraph(name)).all();
    }
  });

  test("loadGraph takes the model's class name, its display name or its graph id", async () => {
    const byClass = await graphManager.loadGraph("Person");
    expect(await graphManager.loadGraph("person")).toBe(byClass);
    expect(await graphManager.loadGraph("91e13029-1637-5fbb-ac8a-2dd0c6502cc7")).toBe(byClass);
  });

  test("initialize takes a plain options object", async () => {
    const { GraphManager } = await import("../../js/graphManager");
    const only = new GraphManager(graphManager.archesClient);
    await only.initialize({ graphs: ["91e13029-1637-5fbb-ac8a-2dd0c6502cc7"] });
    await expect(only.loadGraph("Person")).resolves.toBeDefined();
    await expect(only.loadGraph("Site")).rejects.toThrow(/known: Person$/);
  });

  test("an unknown model name says which names it knows", async () => {
    await expect(graphManager.loadGraph("Nope")).rejects.toThrow(/Person/);
  });

  test("a resource missing from resourceIdToFile's file says so", async () => {
    await expect(staticStore.loadOne("00000000-0000-4000-8000-000000000000")).rejects.toThrow(
      /is not in .*resourceIdToFile/,
    );
  });

  test("a reference is a stub until read, then hydrates through the client", async () => {
    const Person = await graphManager.loadGraph("Person");
    const person = await Person.find(PERSON_ID);

    // --- docs/resource-references.mdx ---
    const site = await person.primary_site;
    expect(site.id).toBe(SITE_ID);
    expect(site.$).toBeNull(); // a stub: id only, nothing retrieved yet

    // Reading an attribute retrieves the resource (registry first, else the client).
    const siteName = await site.name;
    expect(String(siteName)).toBeTruthy();
    expect(site.$).not.toBeNull();

    // And on through the next reference.
    const org = await site.maintained_by;
    expect(String(await org.name)).toBe("City Council -- Planning & GIS Division");

    // Descriptors come from the graph's descriptor function; Organisation has
    // none, so getName() is null (on either backend) and the name is an attribute.
    expect(await org.getName(true)).toBeNull();
  });
});
