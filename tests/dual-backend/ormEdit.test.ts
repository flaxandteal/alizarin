/**
 * Editing through the ORM: assignment records edits (checked against the graph
 * at once), reads see them, and $commit() applies them through core's
 * ResourceEdit, returning the tile delta. The same on WASM and NAPI, as in
 * Python's tests/test_orm_edit.py.
 *
 * Runs under both:
 *   npx vitest run
 *   npx vitest run --config vitest.napi.config.js
 */
import { afterEach, describe, expect, test } from "vitest";
import {
  parseStaticGraph,
  parseStaticResources,
  createStaticGraphMeta,
  createWKRM,
  getBackend,
} from '../../js/backend';
import { ResourceModelWrapper, GraphMutator } from '../../js/graphManager';
import type { ResourceInstanceWrapper } from '../../js/graphManager';
import { ResourceInstanceViewModel } from '../../js/viewModels/ResourceInstanceViewModel';
import { createStaticGraph } from '../../js/static-types';
import { setCurrentLanguage } from '../../js/utils';

const LITERAL = "http://www.w3.org/2000/01/rdf-schema#Literal";
const NOTE = "http://www.cidoc-crm.org/cidoc-crm/P3_has_note";
const RESOURCE_ID = "eeeeeeee-1111-2222-3333-444444444444";

/** A model with `title` (translatable, cardinality 1) and `nicknames` (n). */
function model() {
  const mutator = new GraphMutator(createStaticGraph({ name: "Place", author: "Test" }));
  mutator.addStringNode(null, "title", "Title", "1", LITERAL, NOTE);
  mutator.addStringNode(null, "nicknames", "Nicknames", "n", LITERAL, NOTE);
  const graph = parseStaticGraph(JSON.stringify({ graph: [mutator.apply()] }));
  const wkrm = createWKRM(createStaticGraphMeta({
    graphid: graph.graphid,
    name: "Place",
    slug: "place",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {},
  }));
  class Place extends ResourceInstanceViewModel<any> {
    static _: ResourceInstanceWrapper<any> | null;
    static __: ResourceModelWrapper<any> | null;
  }
  const wrapper = new ResourceModelWrapper(wkrm, graph, Place, true);
  Place.prototype.__ = wrapper;
  return { graph, wrapper };
}

const english = (value: string) => ({ en: { direction: "ltr", value } });

/** A place titled "Town" with one nickname, "The Big Smoke". */
async function place(): Promise<any> {
  const { graph, wrapper } = model();
  const nodes = wrapper.getNodeObjectsByAlias();
  const title = nodes.get("title");
  const nicknames = nodes.get("nicknames");
  const tile = (node: any, id: string, value: string) => ({
    data: { [node.nodeid]: english(value) },
    nodegroup_id: node.nodegroup_id,
    parenttile_id: null,
    resourceinstance_id: RESOURCE_ID,
    sortorder: 0,
    tileid: id,
  });
  const resources = parseStaticResources(JSON.stringify({
    business_data: {
      resources: [{
        resourceinstance: {
          descriptors: { name: "Town", description: "", map_popup: "", slug: "town" },
          graph_id: graph.graphid,
          legacyid: null,
          name: "Town",
          resourceinstanceid: RESOURCE_ID,
        },
        tiles: [
          tile(title, "aaaaaaaa-0000-0000-0000-000000000001", "Town"),
          tile(nicknames, "aaaaaaaa-0000-0000-0000-000000000002", "The Big Smoke"),
        ],
        tiles_loaded: true,
      }],
    },
  }));
  return wrapper.fromStaticResource(resources[0], false, false);
}

describe(`ORM editing [${getBackend()} backend]`, () => {
  afterEach(() => setCurrentLanguage("en"));

  test("a plain string sets the current language, keeping the others", async () => {
    const town = await place();
    setCurrentLanguage("ga");
    town.title = "Baile";

    // Reads see the assignment before the commit.
    expect(await town.title).toBe("Baile");

    const delta = town.$commit();
    expect(delta.upserted).toHaveLength(1);
    expect(delta.removed).toHaveLength(0);
    const title = await town.title;
    expect(title.lang("ga")).toBe("Baile");
    expect(title.lang("en")).toBe("Town");

    // Reads show the current language, falling back to the default.
    expect(`${await town.title}`).toBe("Baile");
    setCurrentLanguage("en");
    expect(`${await town.title}`).toBe("Town");
    setCurrentLanguage("fr");
    expect(`${await town.title}`).toBe("Town");
  });

  test("push and splice add and remove multi-valued tiles", async () => {
    const town = await place();
    const nicknames = await town.nicknames;
    expect(nicknames).toHaveLength(1);

    nicknames.push({ nicknames: "Dublin" });
    expect(await town.nicknames).toHaveLength(2);
    const added = town.$commit();
    expect(added.upserted).toHaveLength(1);
    expect(await town.nicknames).toHaveLength(2);

    const reread = await town.nicknames;
    reread.splice(0, 1);
    const removed = town.$commit();
    expect(removed.removed).toHaveLength(1);
    expect(await town.nicknames).toHaveLength(1);
  });

  test("assignments the graph rules out throw where they're made", async () => {
    const town = await place();
    expect(() => { town.no_such_node = 1; }).toThrow(/no child no_such_node/);
    expect(() => { town.nicknames = "x"; }).toThrow(/multi-valued/);
    expect(town.$.hasPendingEdits()).toBe(false);
  });

  test("splicing an item pushed since the last commit throws", async () => {
    const town = await place();
    const nicknames = await town.nicknames;
    nicknames.push({ nicknames: "Dublin" });
    expect(() => nicknames.splice(1, 1)).toThrow(/added since the last \$commit/);
    town.$discard();
  });

  test("$discard drops what was assigned", async () => {
    const town = await place();
    town.title = "Somewhere else";
    town.$discard();
    expect(`${await town.title}`).toBe("Town");
    expect(town.$commit().upserted).toHaveLength(0);
  });
});

/** A place, with no tiles yet, whose `kind` (a string) has a child `kind_note`. */
async function placeWithKind(): Promise<any> {
  const mutator = new GraphMutator(createStaticGraph({ name: "Place", author: "Test" }));
  mutator.addStringNode(null, "kind", "Kind", "1", LITERAL, NOTE);
  mutator.addStringNode("kind", "kind_note", "Kind note", "1", LITERAL, NOTE);
  const graph = parseStaticGraph(JSON.stringify({ graph: [mutator.apply()] }));
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
  return wrapper.fromStaticResource(resources[0], false, false);
}

describe(`ORM editing an outer node [${getBackend()} backend]`, () => {
  test("assigning its tree form sets its own value from _ and its children", async () => {
    const town = await placeWithKind();
    town.kind = { _: "Fort", kind_note: "Old" };
    const delta = town.$commit();
    expect(delta.upserted.length).toBeGreaterThan(0);

    const kind = await town.kind;
    expect(`${kind}`).toBe("Fort");
    expect(`${await kind._.kind_note}`).toBe("Old");
  });
});

const EVENT_1 = "bbbbbbbb-0000-0000-0000-000000000001";
const EVENT_2 = "bbbbbbbb-0000-0000-0000-000000000002";

/**
 * A place with `events` (semantic, n), each with `notes` (n): the first event
 * has one note, the second none. `tileIds: false` loads the tiles without ids.
 */
async function placeWithEvents(tileIds: boolean = true): Promise<any> {
  const mutator = new GraphMutator(createStaticGraph({ name: "Place", author: "Test" }));
  mutator.addSemanticNode(null, "events", "Events", "n", "http://www.cidoc-crm.org/cidoc-crm/E5_Event", NOTE);
  mutator.addStringNode("events", "notes", "Notes", "n", LITERAL, NOTE);
  const graph = parseStaticGraph(JSON.stringify({ graph: [mutator.apply()] }));
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
  const nodes = wrapper.getNodeObjectsByAlias();
  const events = nodes.get("events");
  const notes = nodes.get("notes");
  const tile = (nodegroup: string, id: string, parent: string | null, data: any, sortorder: number) => ({
    data, nodegroup_id: nodegroup, parenttile_id: parent, resourceinstance_id: RESOURCE_ID,
    sortorder, ...(tileIds ? { tileid: id } : {}),
  });
  const resources = parseStaticResources(JSON.stringify({
    business_data: {
      resources: [{
        resourceinstance: {
          descriptors: { name: "Town", description: "", map_popup: "", slug: "town" },
          graph_id: graph.graphid, legacyid: null, name: "Town", resourceinstanceid: RESOURCE_ID,
        },
        tiles: [
          tile(events.nodegroup_id, EVENT_1, null, {}, 0),
          tile(events.nodegroup_id, EVENT_2, null, {}, 1),
          tile(notes.nodegroup_id, "cccccccc-0000-0000-0000-000000000001", EVENT_1,
            { [notes.nodeid]: english("Founded") }, 0),
        ],
        tiles_loaded: true,
      }],
    },
  }));
  return wrapper.fromStaticResource(resources[0], false, false);
}

describe(`ORM editing nested lists [${getBackend()} backend]`, () => {
  test("push under an item adds a tile with that item as its parent", async () => {
    const town = await placeWithEvents();
    const events = await town.events;
    const first = await events[0];
    const notes = await first.notes;
    expect(notes).toHaveLength(1);
    notes.push({ notes: "Renamed" });
    const delta = town.$commit();
    expect(delta.upserted).toHaveLength(1);
    expect(delta.upserted[0].parenttile_id).toBe(EVENT_1);
  });

  test("push on an empty nested list uses its container as the parent", async () => {
    const town = await placeWithEvents();
    const second = await (await town.events)[1];
    const notes = await second.notes;
    expect(notes).toHaveLength(0);
    notes.push({ notes: "Planned" });
    const delta = town.$commit();
    expect(delta.upserted).toHaveLength(1);
    expect(delta.upserted[0].parenttile_id).toBe(EVENT_2);
  });

  test("tiles without ids load the same way on every backend", async () => {
    const town = await placeWithEvents(false);
    const tiles = JSON.parse(town.$.wasmWrapper.exportTilesJson());
    expect(tiles).toHaveLength(3);
    expect(tiles.map((t: any) => t.tileid ?? null)).toEqual([null, null, null]);
  });
});
