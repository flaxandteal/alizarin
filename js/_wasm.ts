import init, { initSync, greet, StaticNode as WasmStaticNode, StaticGraphMeta as WasmStaticGraphMeta } from "../pkg/alizarin";
import wasmURL from "../pkg/alizarin_bg.wasm?url"

let wasmInitialized = false;

export async function initWasm() {
  if (!wasmInitialized) {
    // In Node.js environment (tests), use synchronous init with file system
    if (typeof process !== 'undefined' && process.versions?.node) {
      try {
        const fs = await import('fs');
        const path = await import('path');
        const wasmPath = path.join(process.cwd(), 'pkg', 'alizarin_bg.wasm');
        const wasmBuffer = fs.readFileSync(wasmPath);
        initSync({ module: wasmBuffer });
      } catch (error) {
        console.error('Failed to initialize WASM in Node.js:', error);
        throw error;
      }
    } else {
      // In browser environment, use async init with fetch
      await init(wasmURL);
    }
    wasmInitialized = true;
  }
}

export async function run() {
  await initWasm();
  greet();
}

// Re-export WASM types for use in the rest of the codebase
export { WasmStaticNode, WasmStaticGraphMeta };
