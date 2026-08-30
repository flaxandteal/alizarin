// Doctest for the runnable documentation examples (docs/example/example-*.tsx).
//
// These modules power the interactive "▶ Run this example" demos in the docs.
// Here we execute each one against the bundled sample data so a broken demo
// fails CI instead of silently rendering an error box in the browser.
//
// The examples return React JSX (esbuild's default classic transform emits
// `React.createElement`). The docs site renders these with React; the test only
// needs them to *execute*, so we install a minimal `React.createElement` shim
// that returns plain nodes — no react dependency required.

import { describe, it, beforeAll, expect } from 'vitest';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

import { ArchesClientRemoteStatic } from '../js/client';
import { graphManager, staticStore } from '../js/graphManager';
import { RDM } from '../js/rdm';
import { initWasmForTests } from './wasm-init';

// Runnable examples — same modules the docs embed via <AlizarinComponent>.
import example1 from '../docs/example/example-1';
import example2 from '../docs/example/example-2';
import example3 from '../docs/example/example-3';
import example4 from '../docs/example/example-4';
import example5 from '../docs/example/example-5';
import example6 from '../docs/example/example-6';
import example7 from '../docs/example/example-7';
import example8 from '../docs/example/example-8';

const EXAMPLES: Record<string, { run: () => Promise<unknown> }> = {
  'example-1': example1,
  'example-2': example2,
  'example-3': example3,
  'example-4': example4,
  'example-5': example5,
  'example-6': example6,
  'example-7': example7,
  'example-8': example8,
};

// --- React.createElement shim (returns plain nodes, no react) ---------------
type ShimNode = { type: unknown; props: Record<string, unknown>; children: unknown[] };
(globalThis as unknown as { React: unknown }).React = {
  createElement: (type: unknown, props: Record<string, unknown> | null, ...children: unknown[]): ShimNode => ({
    type,
    props: props ?? {},
    children: children.flat(Infinity),
  }),
  Fragment: Symbol.for('react.fragment'),
};

// --- Serve docs/example/*.json from the filesystem for the client's fetches --
const realFetch = globalThis.fetch;
(globalThis as unknown as { fetch: typeof fetch }).fetch = async (
  input: RequestInfo | URL,
  init?: RequestInit,
): Promise<Response> => {
  const url = typeof input === 'string' ? input : (input as Request).url ?? String(input);
  if (typeof url === 'string' && !/^https?:\/\//.test(url)) {
    const filePath = resolve(process.cwd(), url.replace(/^\.?\/+/, ''));
    try {
      const body = await readFile(filePath, 'utf8');
      return new Response(body, { status: 200, headers: { 'content-type': 'application/json' } });
    } catch {
      return new Response('not found', { status: 404 });
    }
  }
  return realFetch(input, init);
};

// Same static-data resolver config the docs harness (alizarin-docs/lib/alizarin.ts) uses.
function exampleClient() {
  return new ArchesClientRemoteStatic('', {
    allGraphFile: () => 'docs/example/resource_models/_all.json',
    graphIdToGraphFile: (graphId: string) => `docs/example/resource_models/${graphId}.json`,
    graphIdToResourcesFiles: (graphId: string) => [`docs/example/business_data/_${graphId}.json`],
    resourceIdToFile: (resourceId: string) => `docs/example/business_data/${resourceId}.json`,
    collectionIdToFile: (collectionId: string) => `docs/example/collections/${collectionId}.json`,
  });
}

// An example's run() returns a deeply-nested tree of Promises/arrays/nodes.
// Resolve it fully and collect its text so we can assert the query produced output.
async function renderToText(node: unknown): Promise<string> {
  const v = await node;
  if (v == null || v === false) return '';
  if (Array.isArray(v)) return (await Promise.all(v.map(renderToText))).join('');
  if (typeof v === 'object' && v !== null && 'children' in v) {
    const children = (v as ShimNode).children ?? [];
    return (await Promise.all(children.map(renderToText))).join('');
  }
  return String(v);
}

describe('doc examples execute against the sample data', () => {
  beforeAll(async () => {
    await initWasmForTests();
    const client = exampleClient();
    graphManager.archesClient = client;
    staticStore.archesClient = client;
    RDM.archesClient = client;
    await graphManager.initialize();
  });

  for (const [name, mod] of Object.entries(EXAMPLES)) {
    it(`${name} runs and produces output`, async () => {
      const result = await mod.run();
      const text = await renderToText(result);
      // Each example's run() catches errors and renders "Error: …" — fail on that.
      expect(text).not.toMatch(/Error:/);
      expect(text.trim().length).toBeGreaterThan(0);
    });
  }
});
