#!/usr/bin/env node
/**
 * Print each JS binding's runtime surface as JSON: every export, and for
 * classes their methods (instance and static), listed exactly as
 * tests/dual-backend/bindingParity.test.ts sees them. Used to build and
 * maintain tests/binding-parity.json (see scripts/guards/binding_manifest.py).
 *
 * Usage: node scripts/guards/binding_surface.mjs   (needs pkg/ and a NAPI build)
 */
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const require = createRequire(import.meta.url);

/** Same plumbing filter as bindingParity.test.ts. */
const PLUMBING = new Set([
  'constructor', 'free', '__destroy_into_raw', '__getClassname', 'toJSON', 'toString', '__wrap', '__unwrap',
]);
/** A function's own properties, which every class has (not static methods). */
const FUNCTION_OWN = new Set(['length', 'name', 'prototype', 'caller', 'arguments']);

/** A class's methods, instance and static, as Python's dir() lists both. */
function members(cls) {
  const statics = Object.getOwnPropertyNames(cls).filter(
    (m) => !FUNCTION_OWN.has(m) && typeof cls[m] === 'function',
  );
  return [...new Set([...Object.getOwnPropertyNames(cls.prototype), ...statics])]
    .filter((m) => !PLUMBING.has(m) && !m.startsWith('__wbg'))
    .sort();
}

function surface(module, classNames) {
  const classes = {};
  const functions = [];
  for (const [name, value] of Object.entries(module)) {
    if (name === 'default' || name.startsWith('__')) continue;
    if (typeof value !== 'function') continue;
    if (classNames.has(name)) {
      classes[name] = members(value);
    } else {
      functions.push(name);
    }
  }
  return { classes, functions: functions.sort() };
}

const wasmDts = readFileSync(path.join(root, 'pkg/alizarin.d.ts'), 'utf8');
const napiDts = readFileSync(path.join(root, 'crates/alizarin-napi/index.d.ts'), 'utf8');
const wasmClasses = new Set([...wasmDts.matchAll(/^export class (\w+)/gm)].map((m) => m[1]));
const napiClasses = new Set([...napiDts.matchAll(/^export declare class (\w+)/gm)].map((m) => m[1]));

const wasm = await import(path.join(root, 'pkg/alizarin.js'));
wasm.initSync({ module: readFileSync(path.join(root, 'pkg/alizarin_bg.wasm')) });
const napi = require(path.join(root, 'crates/alizarin-napi/index.js'));

console.log(JSON.stringify({ wasm: surface(wasm, wasmClasses), napi: surface(napi, napiClasses) }, null, 2));
