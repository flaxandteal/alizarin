#!/usr/bin/env node

/**
 * Post-build script: creates dist/alizarin.inline.js as a thin wrapper
 * that re-exports everything from alizarin.js after setting the WASM URL
 * to an inline base64 data URI.
 *
 * This approach ensures alizarin and alizarin/inline share the same module
 * instance — so extensions like @alizarin/clm that import from 'alizarin'
 * register into the same state that consumers of 'alizarin/inline' see.
 *
 * The microtask deferral in main.ts ensures setWasmURL() runs before initWasm().
 */

import { readFileSync, writeFileSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const distDir = resolve(__dirname, '../dist');

const wasmPath = resolve(distDir, 'alizarin_bg.wasm');
const outPath = resolve(distDir, 'alizarin.inline.js');

if (!existsSync(wasmPath)) {
  console.error(`[create-inline] ${wasmPath} not found — run vite build first`);
  process.exit(1);
}

const wasm = readFileSync(wasmPath);
const dataUri = `data:application/wasm;base64,${wasm.toString('base64')}`;

const wrapper = `// Auto-generated wrapper — sets inline WASM URL then re-exports from alizarin.js.
// Both 'alizarin' and 'alizarin/inline' share the same module instance.
import { setWasmURL } from './alizarin.js';
setWasmURL('${dataUri}');
export * from './alizarin.js';
`;

writeFileSync(outPath, wrapper);

const wasmSizeMB = (wasm.length / 1024 / 1024).toFixed(2);
const wrapperSizeKB = (wrapper.length / 1024).toFixed(0);
console.log(`[create-inline] Created alizarin.inline.js wrapper (${wrapperSizeKB} KB, WASM ${wasmSizeMB} MB inline)`);
