/**
 * Dual-backend RDM namespace: core keeps the namespace and the string it was
 * set from, so each backend gives the same UUID and the same raw string (the
 * base URI ArchesClientLocal parses SKOS XML against; NAPI had none and fell
 * back to http://localhost/). Python: crates/alizarin-python/tests/test_rdm_namespace.py.
 *
 * Runs under both backends via:
 *   npx vitest run tests/dual-backend/rdmNamespace.test.ts (WASM)
 *   npx vitest run -c vitest.napi.config.js tests/dual-backend/rdmNamespace.test.ts (NAPI)
 */
import { afterEach, describe, expect, test } from "vitest";
import { getBackend, getNapiModule, getRdmNamespaceRaw, setRdmNamespace } from '../../js/backend';
import * as wasm from '../../pkg/alizarin';

const module: any = getBackend() === 'napi' ? getNapiModule() : wasm;
const URL = 'http://example.org/rdm/';
// uuid5(NAMESPACE_URL, URL)
const URL_UUID = '81adf959-2b61-5009-8011-c017660f4f1f';

describe(`RDM namespace [${getBackend()} backend]`, () => {
  afterEach(() => module.clearRdmNamespace());

  test("keeps the URL it was set from, and its UUID", () => {
    setRdmNamespace(URL);
    expect(getRdmNamespaceRaw()).toBe(URL);
    expect(module.getRdmNamespace()).toBe(URL_UUID);
    expect(module.hasRdmNamespace()).toBe(true);
  });

  test("clearing forgets both", () => {
    setRdmNamespace(URL);
    module.clearRdmNamespace();
    expect(getRdmNamespaceRaw()).toBeNull();
    expect(module.getRdmNamespace()).toBeNull();
    expect(module.hasRdmNamespace()).toBe(false);
  });
});
