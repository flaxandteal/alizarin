/**
 * Binding parity: each wrapper class has exactly the methods
 * tests/binding-parity.json lists for this backend, and every one it lacks is
 * explained there (ecosystem or gap). A method added to one binding only fails
 * here until it is listed. Python checks the same manifest in
 * crates/alizarin-python/tests/test_binding_parity.py.
 *
 * Runs under both:
 *   npx vitest run
 *   npx vitest run --config vitest.napi.config.js
 */
import { describe, expect, test } from "vitest";
import { getBackend, getNapiModule } from '../../js/backend';
import * as wasm from '../../pkg/alizarin';
import manifest from '../binding-parity.json';

type Binding = 'wasm' | 'napi' | 'python';

/** WASM-bindgen's and NAPI's own plumbing, not part of either API. */
const PLUMBING = new Set(['constructor', 'free', '__destroy_into_raw', '__getClassname', 'toJSON', 'toString']);

function methods(cls: any): string[] {
  return Object.getOwnPropertyNames(cls.prototype)
    .filter((name) => !PLUMBING.has(name) && !name.startsWith('__wbg'))
    .sort();
}

const backend = getBackend() as Binding;
const module: any = backend === 'napi' ? getNapiModule() : wasm;

describe(`binding parity [${backend} backend]`, () => {
  for (const [name, spec] of Object.entries((manifest as any).classes)) {
    const { operations } = spec as any;

    test(`${name}: exactly the listed methods`, () => {
      const cls = module[(spec as any)[backend]];
      expect(cls, `${(spec as any)[backend]} is exported`).toBeDefined();
      const expected = operations
        .map((op: any) => op[backend])
        .filter((m: string | null) => m !== null)
        .sort();
      expect(methods(cls)).toEqual(expected);
    });

    test(`${name}: every missing method is explained`, () => {
      const unexplained = operations
        .flatMap((op: any) =>
          (['wasm', 'napi', 'python'] as Binding[])
            .filter((b) => op[b] === null && !(op.why?.[b] || op.why?.all))
            .map((b) => `${b} lacks ${op.wasm ?? op.napi ?? op.python}`),
        );
      expect(unexplained).toEqual([]);
    });
  }
});
