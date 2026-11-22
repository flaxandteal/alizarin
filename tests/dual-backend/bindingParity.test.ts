/**
 * Binding parity: the module exports exactly the classes and free functions
 * tests/binding-parity.json lists for this backend, each class has exactly
 * the listed methods, and everything it lacks is explained there (ecosystem
 * or gap). Anything added to one binding only fails here until it is listed.
 * Python checks the same manifest in
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
const PLUMBING = new Set([
  'constructor', 'free', '__destroy_into_raw', '__getClassname', 'toJSON', 'toString', '__wrap', '__unwrap',
]);

/** String.prototype's names, which js/_wasm.ts copies onto `stringLike` classes. */
const STRING_LIKE = new Set([...Object.getOwnPropertyNames(String.prototype), 'length']);

/** A function's own properties, which every class has (not static methods). */
const FUNCTION_OWN = new Set(['length', 'name', 'prototype', 'caller', 'arguments']);

/** A class's methods, instance and static, as Python's dir() lists both. */
function methods(cls: any, stringLike = false): string[] {
  const statics = Object.getOwnPropertyNames(cls).filter(
    (name) => !FUNCTION_OWN.has(name) && typeof cls[name] === 'function',
  );
  return [...new Set([...Object.getOwnPropertyNames(cls.prototype), ...statics])]
    .filter((name) => !PLUMBING.has(name) && !name.startsWith('__wbg'))
    .filter((name) => !(stringLike && STRING_LIKE.has(name)))
    .sort();
}

const backend = getBackend() as Binding;
const module: any = backend === 'napi' ? getNapiModule() : wasm;

describe(`binding parity [${backend} backend]`, () => {
  for (const [name, spec] of Object.entries((manifest as any).classes)) {
    const { operations } = spec as any;

    test.skipIf((spec as any)[backend] === null)(`${name}: exactly the listed methods`, () => {
      const cls = module[(spec as any)[backend]];
      expect(cls, `${(spec as any)[backend]} is exported`).toBeDefined();
      const expected = operations
        .map((op: any) => op[backend])
        .filter((m: string | null) => m !== null)
        .sort();
      expect(methods(cls, (spec as any).stringLike)).toEqual(expected);
    });

    test(`${name}: every missing method is explained`, () => {
      const unexplained = operations
        .flatMap((op: any) =>
          (['wasm', 'napi', 'python'] as Binding[])
            // A binding without the class at all explains it once, on the class.
            .filter((b) => op[b] === null && !(op.why?.[b] || op.why?.all || (spec as any).why?.[b]))
            .map((b) => `${b} lacks ${op.wasm ?? op.napi ?? op.python}`),
        );
      expect(unexplained).toEqual([]);
    });
  }

  const functions = Object.entries((manifest as any).functions ?? {}) as [string, any][];

  test('every listed function is exported, and every missing one explained', () => {
    const problems = functions.flatMap(([canonical, spec]) => {
      if (spec[backend] === null) {
        return spec.why?.[backend] ? [] : [`${canonical}: missing on ${backend}, with no why`];
      }
      return typeof module[spec[backend]] === 'function' ? [] : [`${canonical}: ${spec[backend]} is not exported`];
    });
    expect(problems).toEqual([]);
  });

  test('nothing is exported that the manifest does not list', () => {
    const listed = new Set([
      ...Object.values((manifest as any).classes).map((spec: any) => spec[backend]),
      ...functions.map(([, spec]) => spec[backend]),
    ]);
    const unlisted = Object.keys(module)
      .filter((name) => name !== 'default' && !name.startsWith('__') && typeof module[name] === 'function')
      .filter((name) => !listed.has(name))
      .sort();
    expect(unlisted).toEqual([]);
  });
});
