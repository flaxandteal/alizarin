import { assert, describe, beforeAll, beforeEach } from 'vitest';
import fetchMock from '@fetch-mock/vitest';
import { ArchesClientLocal } from "../js/client";
import { ResourceInstanceViewModel } from "../js/viewModels";
import { graphManager, staticStore, printWasmTimings, clearWasmTimings } from "../js/graphManager";
import { RDM } from "../js/rdm";
import { apiTest } from "./apiTest";
import { initWasmForTests } from './wasm-init';
import { printRscvTimings, clearRscvTimings } from '../pkg/alizarin';


fetchMock.mockGlobal();

class Group extends ResourceInstanceViewModel<Group> {};

const archesClient = new ArchesClientLocal();
graphManager.archesClient = archesClient;
staticStore.archesClient = archesClient;
RDM.archesClient = archesClient;

describe("testing api", () => {
  beforeAll(async () => {
    // Initialize WASM module for tests
    await initWasmForTests();
  });

  beforeEach(async () => {
    fetchMock.mockReset();
    await graphManager.initialize();
  });

  apiTest(
    "loads resource models",
    async ({ graphsResponse, graphResponses }) => {
      fetchMock.once('*', JSON.stringify(graphsResponse));
      // Order doesn't strictly matter as long as it gets all, but it would be nicer to map to the request.
      Object.values(graphResponses).forEach((response) =>
        fetchMock.once('*', JSON.stringify(response)),
      );

      const Groups = await graphManager.get(Group);

      const groups: Group[] = await Groups.all({ pruneTiles: false });

      assert(groups[0].constructor.name.toString() === "Group");
      const basic_info = await groups[0].basic_info;
      console.log("basic_info (awaited):", basic_info);
      console.log("basic_info.length:", basic_info.length);
      console.log("basic_info.constructor.name:", basic_info.constructor.name);
      console.log("basic_info contents:", [...basic_info]);
      const basic_info_0 = basic_info[0];
      console.log("basic_info[0]:", basic_info_0);
      const name = await basic_info_0.name;
      console.log("name:", name);
      assert(name.toString() == "Global Group");
      assert(name.lang("ga") === "Grúpa Domhanda");
      console.log(await groups[0].permissions[0].action.__parentPseudo.node.node.nodeid, await groups[0].permissions[0].action.__parentPseudo.tile.data, 'PERMSS');
      const action = await groups[0].permissions[0].action[0];
      assert(action.toString() == "Reading");

      const GroupsByName = await graphManager.get("Group");
      const groupsByName = await GroupsByName.all({ pruneTiles: false });
      assert((await groupsByName[0].basic_info[0].name).toString() == "Global Group");
    },
  );

  apiTest(
    "loads resource models lazily",
    async ({ graphsResponse, graphResponses }) => {
      fetchMock.once("*", JSON.stringify(graphsResponse));
      // Order doesn't strictly matter as long as it gets all, but it would be nicer to map to the request.
      Object.values(graphResponses).forEach((response) =>
        fetchMock.once("*", JSON.stringify(response)),
      );

      // Clear timings before test
      clearRscvTimings();

      const Groups = await graphManager.get(Group);
      const groups = await Groups.all({ lazy: true, pruneTiles: false });
      console.log(groups[0]);
      console.log("groups[0].$?.resource:", groups[0].$?.resource);
      console.log("groups[0].$?.resource?.tiles?.length:", groups[0].$?.resource?.tiles?.length);
      console.log("wasmWrapper.tilesLoaded():", groups[0].$?.wasmWrapper?.tilesLoaded());
      const root = await groups[0].$.getRoot();
      console.log("root:", root);
      console.log("root.tile:", root?.tile);
      const basic_info = await groups[0].basic_info;
      console.log("basic_info:", basic_info);
      console.log(basic_info);
      console.log("basic_info.length:", basic_info?.length);
      if (basic_info?.length > 0) {
        const basic_info_0 = await basic_info[0];
        const name = await basic_info_0.name;
        console.log("name:", name);
        assert(name.toString() == "Global Group");
        assert(name.lang("ga") === "Grúpa Domhanda");
      } else {
        throw new Error("basic_info is empty!");
      }
      const action = await groups[0].permissions[0].action[0];
      assert(action.toString() == "Reading");

      // Print timing summary
      printRscvTimings();
    },
  );

  apiTest(
    "WASM timing: loads resource models non-lazily",
    async ({ graphsResponse, graphResponses }) => {
      fetchMock.once("*", JSON.stringify(graphsResponse));
      Object.values(graphResponses).forEach((response) =>
        fetchMock.once("*", JSON.stringify(response)),
      );

      // Clear timings before test
      clearRscvTimings();
      clearWasmTimings();

      const Groups = await graphManager.get(Group);

      // Test with lazy=false (the slow path)
      // Simulate loading 100 resources by calling find() multiple times
      console.log("\n=== Loading 100 resources with lazy=false ===");
      const startAll = Date.now();

      const resourcePromises = [];
      for (let i = 0; i < 100; i++) {
        // Use find() to load the same resource 100 times (simulates batch loading)
        resourcePromises.push(Groups.find("d2368123-9628-49a2-b3dd-78ac6ee3e911", false, false));
      }
      const resources = await Promise.all(resourcePromises);
      console.log(`Loaded ${resources.length} resources in ${Date.now() - startAll}ms`);

      // Access data to ensure it's fully loaded
      const basic_info = await resources[0].basic_info;
      assert(basic_info?.length > 0, "basic_info should have entries");

      // Print both timing summaries
      console.log("\n=== JS-side WASM boundary timings (100 resources) ===");
      printWasmTimings();
      console.log("\n=== Rust-side timings ===");
      printRscvTimings();
    },
  );

  apiTest(
    "loads tiles on-demand via callback (true lazy loading)",
    async ({ graphsResponse, graphResponses }) => {
      fetchMock.once("*", JSON.stringify(graphsResponse));
      Object.values(graphResponses).forEach((response) =>
        fetchMock.once("*", JSON.stringify(response)),
      );

      // Clear timings before test
      clearRscvTimings();

      const Groups = await graphManager.get(Group);

      // Use summaries to get resources WITHOUT tiles - forces callback-based loading
      const groups = await Groups.allSummaries({ limit: 1 });
      const group = groups[0];

      console.log("Using summary - tilesLoaded:", group.$?.wasmWrapper?.tilesLoaded());
      console.log("resource.tiles:", group.$?.resource?.tiles);

      // Access a property that requires tiles - this should trigger the callback
      const basic_info = await group.basic_info;
      console.log("basic_info.length:", basic_info?.length);

      if (basic_info?.length > 0) {
        const name = await basic_info[0].name;
        console.log("name:", name);
        assert(name.toString() == "Global Group");
      }

      // Print timing summary - should show callback timings
      console.log("\n=== Timing Results ===");
      printRscvTimings();
    },
  );

  apiTest(
    "loads tiles on-demand for 100 resources",
    async ({ graphsResponse, graphResponses }) => {
      fetchMock.once("*", JSON.stringify(graphsResponse));
      Object.values(graphResponses).forEach((response) =>
        fetchMock.once("*", JSON.stringify(response)),
      );

      // Clear timings before test
      clearRscvTimings();

      const Groups = await graphManager.get(Group);

      // Create 100 resources from summaries (no tiles upfront)
      const allGroups: Group[] = [];
      for (let i = 0; i < 100; i++) {
        const groups = await Groups.allSummaries({ limit: 1 });
        allGroups.push(groups[0]);
      }

      console.log(`Created ${allGroups.length} resources from summaries`);

      // Access basic_info on each - should trigger callback-based tile loading
      const startTime = Date.now();
      for (const group of allGroups) {
        const basic_info = await group.basic_info;
        if (basic_info?.length > 0) {
          const name = await basic_info[0].name;
          // Just access the value to trigger loading
        }
      }
      const totalTime = Date.now() - startTime;

      console.log(`\nAccessed basic_info on 100 resources in ${totalTime}ms`);
      console.log("\n=== Timing Results (100 resources) ===");
      printRscvTimings();
    },
  );

  apiTest(
    "toJson() uses efficient Rust-side traversal",
    async ({ graphsResponse, graphResponses }) => {
      fetchMock.once("*", JSON.stringify(graphsResponse));
      Object.values(graphResponses).forEach((response) =>
        fetchMock.once("*", JSON.stringify(response)),
      );

      // Clear timings before test
      clearRscvTimings();

      const Groups = await graphManager.get(Group);
      const groups: Group[] = await Groups.all({ pruneTiles: false });
      const group = groups[0];

      // Make sure we have an instance wrapper with wasmWrapper
      assert(group.$, "group should have instance wrapper");
      assert(group.$.wasmWrapper, "instance wrapper should have wasmWrapper");

      // Verify the cache is populated by accessing a value first
      const basic_info = await group.basic_info;
      console.log("basic_info accessed, length:", basic_info?.length);

      // Call toJson() via the Rust path
      const json = group.$.wasmWrapper.toJson();
      console.log("toJson() result:", JSON.stringify(json, null, 2));

      // Verify the structure has expected fields
      assert(json !== null, "toJson() should return a value");
      assert(typeof json === 'object', "toJson() should return an object");

      // Check that we got basic_info data
      if (json.basic_info) {
        console.log("basic_info in JSON:", json.basic_info);
        // The structure should have the name data
        if (Array.isArray(json.basic_info)) {
          assert(json.basic_info.length > 0, "basic_info should have entries");
        }
      }

      // Print timing summary - should show toJson (Rust) timing
      console.log("\n=== toJson Timing Results ===");
      printRscvTimings();
    },
  );

  apiTest(
    "toTiles() builds StaticTiles from pseudo cache",
    async ({ graphsResponse, graphResponses }) => {
      fetchMock.once("*", JSON.stringify(graphsResponse));
      Object.values(graphResponses).forEach((response) =>
        fetchMock.once("*", JSON.stringify(response)),
      );

      // Clear timings before test
      clearRscvTimings();

      const Groups = await graphManager.get(Group);
      const groups: Group[] = await Groups.all({ pruneTiles: false });
      const group = groups[0];

      // Make sure we have an instance wrapper with wasmWrapper
      assert(group.$, "group should have instance wrapper");
      assert(group.$.wasmWrapper, "instance wrapper should have wasmWrapper");

      // Access a value to populate the cache
      const basic_info = await group.basic_info;
      assert(basic_info?.length > 0, "basic_info should have entries");

      // Call toTiles() via the Rust path
      const tiles = group.$.wasmWrapper.toTiles();
      console.log("toTiles() result:", JSON.stringify(tiles, null, 2));

      // Verify the result is an array of tiles
      assert(Array.isArray(tiles), "toTiles() should return an array");
      assert(tiles.length > 0, "toTiles() should return at least one tile");

      // Check that tiles have expected structure
      const firstTile = tiles[0];
      assert(firstTile.nodegroup_id, "tile should have nodegroup_id");
      assert(typeof firstTile.data === 'object', "tile should have data object");

      console.log(`\ntoTiles() returned ${tiles.length} tiles`);
      tiles.forEach((tile: any, i: number) => {
        console.log(`  tile[${i}]: nodegroup=${tile.nodegroup_id}, data keys=${Object.keys(tile.data).length}`);
      });

      // Print timing summary
      console.log("\n=== toTiles Timing Results ===");
      printRscvTimings();
    },
  );

  apiTest(
    "toResource() builds complete StaticResource",
    async ({ graphsResponse, graphResponses }) => {
      fetchMock.once("*", JSON.stringify(graphsResponse));
      Object.values(graphResponses).forEach((response) =>
        fetchMock.once("*", JSON.stringify(response)),
      );

      // Clear timings before test
      clearRscvTimings();

      const Groups = await graphManager.get(Group);
      const groups: Group[] = await Groups.all({ pruneTiles: false });
      const group = groups[0];

      // Make sure we have an instance wrapper with wasmWrapper
      assert(group.$, "group should have instance wrapper");
      assert(group.$.wasmWrapper, "instance wrapper should have wasmWrapper");

      // Access a value to populate the cache
      const basic_info = await group.basic_info;
      assert(basic_info?.length > 0, "basic_info should have entries");

      // Call toResource() via the Rust path
      const resource = group.$.wasmWrapper.toResource();
      console.log("toResource() result:", JSON.stringify(resource, null, 2).substring(0, 500) + "...");

      // Verify the result has expected structure
      assert(resource.resourceinstance, "resource should have resourceinstance");
      assert(resource.resourceinstance.resourceinstanceid, "resource should have resourceinstanceid");
      assert(Array.isArray(resource.tiles), "resource should have tiles array");
      assert(resource.tiles.length > 0, "resource should have at least one tile");

      console.log(`\ntoResource() returned resource with ${resource.tiles.length} tiles`);
      console.log(`  resourceinstanceid: ${resource.resourceinstance.resourceinstanceid}`);

      // Print timing summary
      console.log("\n=== toResource Timing Results ===");
      printRscvTimings();
    },
  );
});

// from pathlib import Path
// import json
// from urllib.parse import urlparse, parse_qs
// import httpx
// from arches_orm.adapter import context_free, get_adapter
//
// @context_free
// def test_can_get_collection():
//     StatusEnum = get_adapter().get_collection("7849cd3c-3f0d-454d-aaea-db9164629641")
//     assert StatusEnum.BacklogDashSkeleton
//
// @context_free
// def test_can_make_collection():
//     rdm = get_adapter().get_rdm()
//     concept_1 = rdm.make_simple_concept("My Status", "Backlog - Nothing")
//     concept_2 = rdm.make_simple_concept("My Status", "Backlog - Everything")
//     my_status = rdm.make_simple_concept("My Status", children=[concept_1, concept_2])
//     MyStatusEnum = rdm.concept_to_collection(my_status)
//     assert MyStatusEnum.BacklogDashEverything
//
// @context_free
// def test_can_save_collection():
//     rdm = get_adapter().get_rdm()
//     concept_1 = rdm.make_simple_concept("My Status", "Backlog - Nothing")
//     concept_2 = rdm.make_simple_concept("My Status", "Backlog - Everything")
//     my_status = rdm.make_simple_concept("My Status", children=[concept_1, concept_2])
//     MyStatusEnum = rdm.concept_to_collection(my_status)
//     rdm.save_concept(my_status, "/tmp/test.xml")
//     rdm.update_collections(MyStatusEnum, Path("/tmp/collections.xml"))
//
// @context_free
// def test_can_load_resource_models(arches_orm):
//     from arches_orm.models import Group
//     Group.all()
//
// @context_free
// def test_can_load_a_resource(arches_orm):
//     from arches_orm.models import Group
//     groups = Group.all()
//     assert str(groups[0]) == "Global Group"
//
// @context_free
// def test_can_create_a_resource(arches_orm):
//     from arches_orm.models import Person
//     ash = Person()
//     name = ash.name.append()
//     name.full_name = "Ash"
//     assert name.full_name._value == {"en": {"direction": "ltr", "value": "Ash"}} # type: ignore
//
// @context_free
// def test_can_search_for_a_resource(arches_orm):
//     from arches_orm.models import Group
//     groups = list(Group._.where(name=".*Global Group.*"))
//     assert len(groups) == 1
//     for group in groups:
//         assert group.basic_info[0].name == "Global Group"
//         assert group.statement[0].description == "Global root of group hierarchy."
//
// @context_free
// def test_find_resource(arches_orm):
//     from arches_orm.models import Group
//     group = Group.find("31170363-6328-4839-99c1-23b60e4bfa98")
//     assert group.basic_info[0].name == "Global Group"
//     assert group.statement[0].description == "Global root of group hierarchy."
//
// @context_free
// def test_can_get_text_in_language(arches_orm):
//     from arches_orm.models import Group
//     adapter = get_adapter("resource_api")
//
//     def _handler(request):
//         path = request.url.path
//         query = parse_qs(request.url.query.decode("utf-8"))
//         if path == "/search/resources":
//             with (Path(__file__).parent / "_responses" / "search.json").open() as f:
//                 data = json.load(f)
//         elif path == "/api/tiles":
//             resource_ids = json.loads(query["resource_ids"][0])
//             nodegroup_ids = json.loads(query["nodegroup_ids"][0])
//             with (Path(__file__).parent / "_responses" / "tiles.json").open() as f:
//                 data = [
//                     tile for tile in json.load(f)
//                     if (not resource_ids or tile["resourceinstance_id"] in resource_ids)
//                     and (not nodegroup_ids or tile["nodegroup_id"] in nodegroup_ids)
//                 ]
//         return httpx.Response(200, json=data)
//
//     adapter.config["client"]["transport"] = httpx.MockTransport(_handler)
//     groups = list(Group.where(name="Group")) # not a real query
//     assert len(groups) == 3
//     group_names = {
//         "Enforcement Admin Group",
//         "Global Group",
//         "Enforcement Management Group",
//     }
//     for group in groups:
//         if group.basic_info:
//             group_names.remove(group.basic_info[0].name)
//     assert not group_names
