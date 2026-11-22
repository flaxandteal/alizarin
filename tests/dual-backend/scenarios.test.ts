/**
 * Scenarios: sequences of ORM steps (read, assign, commit), as data in
 * tests/data/scenarios/*.json, each checked as it runs. Python plays the
 * same files in crates/alizarin-python/tests/test_scenarios.py, so the ORMs
 * agree step by step. The vocabulary is in tests/data/scenarios/README.md.
 *
 * Runs under both:
 *   npx vitest run tests/dual-backend/scenarios.test.ts
 *   npx vitest run --config vitest.napi.config.js tests/dual-backend/scenarios.test.ts
 */
import { describe, expect, test } from "vitest";
import * as fs from "fs";
import * as path from "path";
import {
  parseStaticGraph,
  parseStaticResources,
  createStaticGraphMeta,
  createWKRM,
  getBackend,
} from "../../js/backend";
import { ResourceModelWrapper } from "../../js/graphManager";
import { ResourceInstanceViewModel } from "../../js/viewModels/ResourceInstanceViewModel";
import { StaticCollection } from "../../js/static-types";
import { RDM } from "../../js/rdm";
import { nodeConfigManager } from "../../js/nodeConfig";

const DATA = path.join(__dirname, "../data");
const SCENARIOS = path.join(DATA, "scenarios");
const load = (...parts: string[]) => JSON.parse(fs.readFileSync(path.join(DATA, ...parts), "utf8"));
const plain = (value: any) => JSON.parse(JSON.stringify(value));

/** A fixture's resource, opened through the ORM, and its graph's nodes by alias. */
async function open(fixture: string): Promise<{ resource: any; nodeIds: Record<string, string> }> {
  RDM.clear();
  const collectionFile = path.join(DATA, fixture, "collection.json");
  if (fs.existsSync(collectionFile)) {
    RDM.addCollection(new StaticCollection(load(fixture, "collection.json")));
  }
  const graphJson = load(fixture, "graph.json");
  const graph = parseStaticGraph(JSON.stringify({ graph: [graphJson] }));
  nodeConfigManager.loadFromGraph(graph);
  const wkrm = createWKRM(createStaticGraphMeta({
    graphid: graph.graphid,
    name: "Scenario",
    slug: "scenario",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {},
  }));
  class Scenario extends ResourceInstanceViewModel<any> {
    static _: any;
    static __: ResourceModelWrapper<any> | null;
  }
  const wrapper = new ResourceModelWrapper(wkrm, graph, Scenario, true);
  Scenario.prototype.__ = wrapper;
  const [statics] = parseStaticResources(JSON.stringify({
    business_data: { resources: [{ ...load(fixture, "resource.json"), tiles_loaded: true }] },
  }));
  const resource = await wrapper.fromStaticResource(statics, false, false);
  const nodeIds = Object.fromEntries(graphJson.nodes.map((n: any) => [n.alias, n.nodeid]));
  return { resource, nodeIds };
}

/** Each upserted tile's value for an alias, as JSON. */
function upsertedByAlias(upserted: any[], nodeIds: Record<string, string>, aliases: string[]): Record<string, any> {
  const out: Record<string, any> = {};
  for (const alias of aliases) {
    const tile = upserted.map(plain).find((t: any) => t.data && nodeIds[alias] in t.data);
    out[alias] = tile ? tile.data[nodeIds[alias]] : "(no upserted tile)";
  }
  return out;
}

/** A read's checks, by the key a step names them with. */
const READS: Record<string, (value: any) => Promise<any> | any> = {
  forJson: async (value) => plain(await value.forJson()),
  str: (value) => String(value),
};

type Context = { resource: any; nodeIds: Record<string, string>; at: string };

/** One handler per step, keyed by the step's kind (its first key). */
const STEPS: Record<string, (step: any, ctx: Context) => Promise<void> | void> = {
  async read(step, { resource, at }) {
    const value = await resource[step.read];
    expect(typeof value?.forJson, `${at}: a view model (got ${value})`).toBe("function");
    const checks = Object.keys(READS).filter((key) => key in step);
    const actual = Object.fromEntries(await Promise.all(checks.map(async (key) => [key, await READS[key](value)])));
    expect(actual, at).toEqual(Object.fromEntries(checks.map((key) => [key, step[key]])));
  },
  async pending(step, { resource, at }) {
    // Assigned and not yet committed: the value as given.
    expect(plain(await resource[step.pending]), at).toEqual(step.value);
  },
  assign(step, { resource }) {
    resource[step.assign] = step.value;
  },
  assign_fails(step, { resource, at }) {
    expect(() => {
      resource[step.assign_fails] = step.value;
    }, at).toThrow(step.error);
  },
  commit(step, { resource, nodeIds, at }) {
    const { upserted, removed } = resource.$commit();
    const aliases = Object.keys(step.commit.upserted);
    expect(upsertedByAlias(upserted, nodeIds, aliases), at).toEqual(step.commit.upserted);
    expect(upserted.length, `${at} (upserted tiles)`).toBe(aliases.length);
    expect(removed, at).toEqual(step.commit.removed);
  },
  commit_fails(step, { resource, at }) {
    expect(() => resource.$commit(), at).toThrow(step.commit_fails);
  },
};

const files = fs.readdirSync(SCENARIOS).filter((f) => f.endsWith(".json")).sort();

describe(`Scenarios [${getBackend()} backend]`, () => {
  test.each(files)("%s", async (file) => {
    const scenario = JSON.parse(fs.readFileSync(path.join(SCENARIOS, file), "utf8"));
    const { resource, nodeIds } = await open(scenario.fixture);
    for (const [i, step] of scenario.steps.entries()) {
      const at = `${file} step ${i}: ${JSON.stringify(step)}`;
      const kind = Object.keys(step)[0];
      expect(Object.keys(STEPS), `${at}: a known step`).toContain(kind);
      await STEPS[kind](step, { resource, nodeIds, at });
    }
  });
});
