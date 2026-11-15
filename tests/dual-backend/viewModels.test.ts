/**
 * Every datatype's view model, read through the ORM, against the shared
 * expectations in tests/data/view-models/expected.json, which Python's
 * tests/test_view_models_shared.py checks too: the same on every backend.
 *
 * Runs under both:
 *   npx vitest run
 *   npx vitest run --config vitest.napi.config.js
 */
import { beforeAll, describe, expect, test } from "vitest";
import {
  parseStaticGraph,
  parseStaticResources,
  createStaticGraphMeta,
  createWKRM,
  getBackend,
} from '../../js/backend';
import { ResourceModelWrapper } from '../../js/graphManager';
import { ResourceInstanceViewModel } from '../../js/viewModels/ResourceInstanceViewModel';
import { StaticCollection } from '../../js/static-types';
import { RDM } from '../../js/rdm';
import { nodeConfigManager } from '../../js/nodeConfig';
import * as GraphJSON from "../data/view-models/graph.json";
import * as ResourceJSON from "../data/view-models/resource.json";
import * as CollectionJSON from "../data/view-models/collection.json";
import * as ExpectedJSON from "../data/view-models/expected.json";

type Expectation = { forJson: any; str?: string; items?: string[] };
const expected: Record<string, Expectation> = Object.fromEntries(
  Object.entries((ExpectedJSON as any).default ?? ExpectedJSON).filter(([alias]) => !alias.startsWith("_")),
) as any;
const plain = (json: any) => JSON.parse(JSON.stringify((json as any).default ?? json));

let resource: any;

beforeAll(async () => {
  RDM.clear();
  RDM.addCollection(new StaticCollection(plain(CollectionJSON)));

  const graph = parseStaticGraph(JSON.stringify({ graph: [plain(GraphJSON)] }));
  nodeConfigManager.loadFromGraph(graph);
  const wkrm = createWKRM(createStaticGraphMeta({
    graphid: graph.graphid,
    name: "Probe",
    slug: "probe",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {},
  }));
  class Probe extends ResourceInstanceViewModel<any> {
    static _: any;
    static __: ResourceModelWrapper<any> | null;
  }
  const wrapper = new ResourceModelWrapper(wkrm, graph, Probe, true);
  Probe.prototype.__ = wrapper;
  const [statics] = parseStaticResources(JSON.stringify({
    business_data: { resources: [{ ...plain(ResourceJSON), tiles_loaded: true }] },
  }));
  resource = await wrapper.fromStaticResource(statics, false, false);
});

describe(`View models [${getBackend()} backend]`, () => {
  test.each(Object.keys(expected))("%s reads as expected", async (alias) => {
    const { forJson, str, items } = expected[alias];
    const value = await resource[alias];
    expect(value).not.toBeNull();
    // As it serialises: forJson is the form JSON is written from.
    expect(JSON.parse(JSON.stringify(await value.forJson()))).toEqual(forJson);
    if (str !== undefined) {
      expect(String(value)).toBe(str);
    }
    if (items !== undefined) {
      expect([...value].map((item: any) => String(item))).toEqual(items);
    }
  });

  test("every datatype in the fixture has an expectation", () => {
    const datatypes = new Set(plain(GraphJSON).nodes
      .filter((node: any) => !node.istopnode)
      .map((node: any) => node.alias));
    expect(new Set(Object.keys(expected))).toEqual(datatypes);
  });
});
