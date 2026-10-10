import { test, beforeAll } from "vitest";
import { assert } from "chai";
import { StaticGraph } from "../pkg/alizarin";
import { initWasmForTests } from "./wasm-init";
import * as GroupJSON from "./data/models/Group.json";

beforeAll(async () => {
  await initWasmForTests();
});

// C11: graph-level `config` must round-trip as a plain JS object, preserving
// arbitrary host keys. The macro-generated getter used serde_wasm_bindgen's
// default serializer, which turned the config Object into a JS Map — so custom
// keys were unreachable and it inspected as `{}`.
test("StaticGraph preserves custom config keys as a plain object (C11)", () => {
  const raw = JSON.parse(JSON.stringify(GroupJSON)) as {
    graph: Array<Record<string, unknown>>;
  };
  raw.graph[0].config = { cairn: { scope: "campaign" }, something: 42 };

  const graph = StaticGraph.fromJsonString(JSON.stringify(raw));
  const config = graph.config as Record<string, unknown>;

  // Plain object, not a Map: own keys are directly accessible.
  assert.isObject(config, "config must be a plain object");
  assert.isFalse(config instanceof Map, "config must not be a Map (C11)");
  const cairn = config.cairn as Record<string, unknown> | undefined;
  assert.isDefined(cairn, "custom config.cairn key must survive the round-trip");
  assert.equal(cairn!.scope, "campaign");
  assert.equal(config.something, 42);
});
