/**
 * Dual-backend extension node configs: core models no `reference` config; CLM
 * supplies it at runtime (native handler on NAPI, its WASM build on WASM), and
 * the shared NodeConfigManager returns the same config on both.
 *
 * Runs under both backends via:
 *   npx vitest run tests/dual-backend/extensionConfig.test.ts (WASM)
 *   npx vitest run -c vitest.napi.config.js tests/dual-backend/extensionConfig.test.ts (NAPI)
 */
import { test, describe, expect, beforeAll } from "vitest";
import { createRequire } from "module";
import { getBackend, getNapiModule, parseStaticGraph, registerExtensionHandler } from '../../js/backend';
import { NodeConfigManager, registerNodeConfigAdapter } from '../../js/nodeConfig';
import * as HeritageItem from "../data/models/Heritage Item.json";

const MULTI = { nodeid: '46cd4b7e-049d-11eb-ba3a-f875a44e0e11', datatype: 'reference' } as any;
const SINGLE = { nodeid: '46f25cdc-b6c7-11ea-a6f4-f875a44e0e11', datatype: 'reference' } as any;

beforeAll(async () => {
  if (getBackend() === 'napi') {
    // As @alizarin/clm-napi does on import: hand CLM's native handler to the host.
    const clmNapi = createRequire(import.meta.url)('../../ext/clm/napi/index.js');
    getNapiModule().registerExtensionHandler(clmNapi.referenceHandlerPtr());
  } else {
    // As @alizarin/clm does: register its WASM build's functions as the handler.
    const clm = await import('../../ext/clm/wasm/pkg/alizarin_clm_wasm');
    const bytes = (await import('../../ext/clm/wasm/pkg/alizarin_clm_wasm_bg.wasm')).default;
    clm.initSync(bytes);
    registerExtensionHandler('reference', {
      nodeConfig: (config: unknown) => clm.referenceNodeConfig(config),
      displayAsList: (config: unknown) => clm.referenceDisplayAsList(config),
    });
  }
  // The extension's adapter; identity here so the test sees the raw config.
  registerNodeConfigAdapter('reference', (config) => config);
});

function loadedManager(): NodeConfigManager {
  const manager = new NodeConfigManager();
  manager.loadFromGraph(parseStaticGraph(JSON.stringify(HeritageItem)));
  return manager;
}

describe(`Extension node configs [${getBackend()} backend]`, () => {
  test("the reference config comes from CLM's handler", () => {
    expect(loadedManager().retrieve(MULTI)).toEqual({
      controlledList: '1a161d2b-5887-4ad2-800e-ffa073f3b2bd',
      rdmCollection: null,
      multiValue: true,
      collectionId: '1a161d2b-5887-4ad2-800e-ffa073f3b2bd',
    });
    expect((loadedManager().retrieve(SINGLE) as any).multiValue).toBe(false);
  });

  test("a datatype with no adapter gives null", () => {
    expect(loadedManager().retrieve({ ...MULTI, datatype: 'some-other-extension' })).toBeNull();
  });
});
