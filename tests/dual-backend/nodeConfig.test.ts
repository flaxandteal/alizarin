/**
 * Dual-backend node configs: the shared TypeScript NodeConfigManager must read
 * boolean and domain configs identically on WASM and NAPI.
 *
 * Runs under both backends via:
 *   npx vitest run tests/dual-backend/nodeConfig.test.ts (WASM)
 *   npx vitest run -c vitest.napi.config.js tests/dual-backend/nodeConfig.test.ts (NAPI)
 */
import { test, describe, expect } from "vitest";
import { getBackend, parseStaticGraph } from '../../js/backend';
import { NodeConfigManager } from '../../js/nodeConfig';
import * as GroupModel from "../definitions/models/07883c9e-b25c-11e9-975a-a4d18cec433a.json";

const BOOLEAN = { nodeid: '389f0d26-027c-11ed-9dee-0242ac180008', datatype: 'boolean' } as any;
const DOMAIN = { nodeid: '82c9eef6-2d5a-11ef-8722-0242ac120006', datatype: 'domain-value' } as any;
const ACCEPTED = '6dbde4c3-eeab-4fab-9106-718af3bba238';
const RETURNED = '7c10f127-3358-4cc9-b4df-75cd6c5f7cd7';

function loadedManager(): NodeConfigManager {
  const manager = new NodeConfigManager();
  manager.loadFromGraph(parseStaticGraph(JSON.stringify(GroupModel)));
  return manager;
}

describe(`Node configs [${getBackend()} backend]`, () => {
  test("boolean config gives its labels", () => {
    const config: any = loadedManager().retrieve(BOOLEAN);
    expect(config).not.toBeNull();
    expect(config.getLabel(true, 'en')).toBe('Yes');
    expect(config.getLabel(false, 'ga')).toBe('No');
    expect(config.trueLabel).toEqual({ en: 'Yes' });
    expect(config.falseLabel).toEqual({ en: 'No' });
  });

  test("domain config gives its options and selection", () => {
    const config: any = loadedManager().retrieve(DOMAIN);
    expect(config).not.toBeNull();
    expect(config.options.map((o: any) => o.id)).toEqual([ACCEPTED, RETURNED]);
    expect(config.getSelected()?.id).toBe(RETURNED);
    expect(config.valueFromId(ACCEPTED)?.text).toEqual({ en: 'Accepted' });
    expect(config.valueFromId('no-such-option')).toBeUndefined();
  });

  test("a node with no config gives null", () => {
    const manager = loadedManager();
    expect(manager.retrieve({ nodeid: 'not-a-node', datatype: 'boolean' } as any)).toBeNull();
  });
});
