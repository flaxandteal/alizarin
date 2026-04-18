#!/usr/bin/env node

/**
 * Post-build script: creates inline wrappers that set the WASM URL to a
 * base64 data URI, then re-export from the corresponding non-inline build.
 *
 * Produces:
 *   dist/alizarin.inline.js      — core only (re-exports alizarin.js)
 *   dist/alizarin.inline-full.js — core + extensions (re-exports alizarin.full.js)
 *
 * The microtask deferral in main.ts ensures setWasmURL() runs before initWasm().
 */

import { readFileSync, writeFileSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const distDir = resolve(__dirname, '../dist');

const wasmPath = resolve(distDir, 'alizarin_bg.wasm');
if (!existsSync(wasmPath)) {
  console.error(`[create-inline] ${wasmPath} not found — run vite build first`);
  process.exit(1);
}

const wasm = readFileSync(wasmPath);
const dataUri = `data:application/wasm;base64,${wasm.toString('base64')}`;
const wasmSizeMB = (wasm.length / 1024 / 1024).toFixed(2);

const variants = [
  { source: 'alizarin.js',      out: 'alizarin.inline.js',      label: 'core' },
  { source: 'alizarin.full.js', out: 'alizarin.inline-full.js', label: 'core + extensions' },
];

for (const { source, out, label } of variants) {
  const sourcePath = resolve(distDir, source);
  if (!existsSync(sourcePath)) {
    console.warn(`[create-inline] ${source} not found, skipping ${out}`);
    continue;
  }

  const wrapper = `// Auto-generated wrapper (${label}) — sets inline WASM URL then re-exports.
import { setWasmURL } from './${source}';
setWasmURL('${dataUri}');
export * from './${source}';
`;

  writeFileSync(resolve(distDir, out), wrapper);
  const sizeKB = (wrapper.length / 1024).toFixed(0);
  console.log(`[create-inline] Created ${out} (${sizeKB} KB, WASM ${wasmSizeMB} MB inline)`);
}
