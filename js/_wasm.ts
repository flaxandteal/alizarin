import init, { initSync, greet, StaticNode as WasmStaticNode, StaticGraphMeta as WasmStaticGraphMeta } from "../pkg/alizarin";
import wasmURL from "../pkg/alizarin_bg.wasm?url"

let wasmInitialized = false;

export async function initWasm() {
  if (!wasmInitialized) {
    // In Node.js environment (tests), use synchronous init with file system
    if (typeof process !== 'undefined' && process.versions?.node) {
      try {
        console.log('[alizarin] Initializing WASM in Node.js environment');
        // Import Node modules only when in Node.js environment
        const fs = await import('fs');
        const path = await import('path');
        const { fileURLToPath } = await import('url');

        // Resolve relative to this module's location, not process.cwd()
        const moduleDir = path.dirname(fileURLToPath(import.meta.url));
        const wasmPath = path.join(moduleDir, '../pkg', 'alizarin_bg.wasm');
        const wasmBuffer = fs.readFileSync(wasmPath);
        initSync({ module: wasmBuffer });
        console.log('[alizarin] WASM initialized successfully in Node.js');
      } catch (error) {
        console.error('Failed to initialize WASM in Node.js:', error);
        throw error;
      }
    } else {
      // In browser environment, use async init with fetch
      console.log('[alizarin] Initializing WASM in browser environment', { init, wasmURL });
      try {
        await init(wasmURL);
        console.log('[alizarin] WASM initialized successfully in browser');
      } catch (error) {
        console.error('[alizarin] Failed to initialize WASM in browser:', error);
        throw error;
      }
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
