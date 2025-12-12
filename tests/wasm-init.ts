import { initWasm } from '../js/_wasm';

// Custom WASM initialization for tests
// This uses the same initialization path as the main code to ensure
// the wasmInitialized flag is set properly and prevent duplicate initialization
export async function initWasmForTests() {
  await initWasm();
}