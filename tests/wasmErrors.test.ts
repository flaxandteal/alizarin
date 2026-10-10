import { test, beforeAll } from "vitest";
import { assert } from "chai";
import { buildResourcesFromBusinessCsv } from "../pkg/alizarin";
import { initWasmForTests } from "./wasm-init";

beforeAll(async () => {
  await initWasmForTests();
});

// C12: WASM data-loading functions must reject with real `Error` objects, not
// bare strings. Before the fix they threw via `JsValue::from_str`, so `e.message`
// was undefined in catch blocks and standard error handling broke.
test("buildResourcesFromBusinessCsv throws an Error (not a string) on bad input (C12)", () => {
  let caught: unknown;
  try {
    // Invalid graph JSON forces the parse-error path.
    buildResourcesFromBusinessCsv(
      "ResourceID\nx",
      "{ this is not valid json",
      "[]",
      "en",
      true,
      undefined,
    );
  } catch (e) {
    caught = e;
  }

  assert.instanceOf(caught, Error, "a thrown error must be a real Error instance");
  const message = (caught as Error).message;
  assert.isString(message, "Error.message must be a string, not undefined");
  assert.include(message, "parse graph JSON", "the message should carry the failure detail");
});
