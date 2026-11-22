/**
 * NAPI's importPrebuild and loadPrebuildBusinessData on docs/example laid out
 * as a prebuild: core's summary, as Python's import_prebuild gives it
 * (crates/alizarin-python/tests/test_load_prebuild.py). WASM has no
 * filesystem, so on WASM the check is that it has neither.
 *   npx vitest run tests/dual-backend/prebuildImport.test.ts
 *   npx vitest run --config vitest.napi.config.js tests/dual-backend/prebuildImport.test.ts
 */
import { afterAll, describe, expect, test } from "vitest";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as wasm from "../../pkg/alizarin";
import { getBackend, getNapiModule } from "../../js/backend";

const EXAMPLE = path.join(__dirname, "../../docs/example");
const PERSON_ID = "193750d0-e4b8-527d-8dc2-cbfb4cd31e72";

/** docs/example's graphs and business data in the prebuild layout. */
function prebuild(): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "alizarin-prebuild-"));
  const models = path.join(dir, "graphs", "resource_models");
  fs.mkdirSync(models, { recursive: true });
  for (const name of fs.readdirSync(path.join(EXAMPLE, "resource_models"))) {
    if (!name.startsWith("_")) fs.copyFileSync(path.join(EXAMPLE, "resource_models", name), path.join(models, name));
  }
  fs.cpSync(path.join(EXAMPLE, "business_data"), path.join(dir, "business_data"), { recursive: true });
  return dir;
}

const dir = prebuild();
afterAll(() => fs.rmSync(dir, { recursive: true, force: true }));

/** What each backend can do with a prebuild directory. */
const CHECKS: Record<string, [string, () => void][]> = {
  napi: [
    ["importPrebuild gives core's summary", () => {
      const result = getNapiModule().importPrebuild(dir, "http://localhost/");
      expect(Object.keys(result).sort()).toEqual(["collectionIds", "collections", "graphIds", "ontologyConfigs"]);
      expect(result.graphIds).toHaveLength(6);
    }],
    ["a registry loads its business data", () => {
      const registry = new (getNapiModule().NapiStaticResourceRegistry)();
      const refs = registry.loadPrebuildBusinessData(dir);
      expect(refs.map((r: any) => r.resourceinstanceid)).toContain(PERSON_ID);
      expect(registry.getFull(PERSON_ID)).not.toBeNull();
    }],
  ],
  wasm: [
    ["has no filesystem import, by design", () => {
      expect((wasm as any).importPrebuild).toBeUndefined();
      expect((wasm.StaticResourceRegistry.prototype as any).loadPrebuildBusinessData).toBeUndefined();
    }],
  ],
};

describe(`prebuild directories [${getBackend()} backend]`, () => {
  test.each(CHECKS[getBackend()])("%s", (_, check) => check());
});
