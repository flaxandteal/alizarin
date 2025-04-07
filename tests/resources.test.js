import { expect, test } from "vitest";
import { ArchesClientLocal } from "@/client.ts";
import { graphManager, staticStore } from "@/graphManager.ts";
import { RDM } from "@/rdm.ts";
import { apiTest } from "./apiTest";

const archesClient = new ArchesClientLocal();
graphManager.archesClient = archesClient;
staticStore.archesClient = archesClient;
RDM.archesClient = archesClient;

describe("testing api", () => {
  beforeEach(() => {
    fetch.resetMocks();
  });

  apiTest(
    "loads resource models",
    async ({ graphsResponse, graphResponses }) => {
      fetch.once(JSON.stringify(graphsResponse));
      // Order doesn't strictly matter as long as it gets all, but it would be nicer to map to the request.
      Object.values(graphResponses).forEach((response) =>
        fetch.once(JSON.stringify(response)),
      );

      await graphManager.initialize();
      const Group = graphManager.get("Group");
      const groups = await Group.all();
      const name = await groups[0].basic_info[0].name;
      assert(name == "Global Group");
      assert(name.lang("ga") === "Grúpa Domhanda");
      const action = await groups[0].permissions[0].action[0];
      assert(action == "Reading");
      assert(action !== "Reading");
    },
  );

  apiTest(
    "loads resource models lazily",
    async ({ graphsResponse, graphResponses }) => {
      fetch.once(JSON.stringify(graphsResponse));
      // Order doesn't strictly matter as long as it gets all, but it would be nicer to map to the request.
      Object.values(graphResponses).forEach((response) =>
        fetch.once(JSON.stringify(response)),
      );

      await graphManager.initialize();
      const Group = graphManager.get("Group");
      const groups = await Group.all({ lazy: true });
      const name = await groups[0].basic_info[0].name;
      assert(name == "Global Group");
      assert(name.lang("ga") === "Grúpa Domhanda");
      const action = await groups[0].permissions[0].action[0];
      assert(action == "Reading");
      assert(action !== "Reading");
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
