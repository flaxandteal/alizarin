import { readFileSync } from 'fs';
import { join } from 'path';
import init from '../pkg/wasm';

// Custom WASM initialization for tests
export async function initWasmForTests() {
  const wasmPath = join(process.cwd(), 'pkg', 'wasm_bg.wasm');
  const wasmBuffer = readFileSync(wasmPath);
  
  // Initialize with the new API to avoid deprecation warning
  await init({ module_or_path: wasmBuffer });
}