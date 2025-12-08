import init, { initSync, greet, StaticNode as WasmStaticNode, StaticGraphMeta as WasmStaticGraphMeta, StaticTranslatableString, getRscvTimings } from "../pkg/alizarin";
import { registerRustTimingGetter } from "./tracing";
import wasmURL from "../pkg/alizarin_bg.wasm?url"

let wasmInitialized = false;

/**
 * Patch a WASM class prototype to delegate unknown String methods to toString().
 * This allows `sts.trim()`, `sts.toUpperCase()`, etc. to work transparently.
 */
function patchStringLikePrototype(proto: object): void {
  const stringProto = String.prototype as Record<string, unknown>;
  const protoRecord = proto as Record<string, unknown>;

  for (const method of Object.getOwnPropertyNames(stringProto)) {
    if (typeof stringProto[method] === 'function' && !(method in protoRecord) && method !== 'constructor') {
      protoRecord[method] = function(this: { toString(): string }, ...args: unknown[]) {
        const str = this.toString();
        return (str as Record<string, unknown>)[method] instanceof Function
          ? ((str as Record<string, (...a: unknown[]) => unknown>)[method])(...args)
          : (str as Record<string, unknown>)[method];
      };
    }
  }

  // Add .length property
  Object.defineProperty(proto, 'length', {
    get(this: { toString(): string }) { return this.toString().length; },
    configurable: true,
  });

  // Add valueOf for coercion (template literals, concatenation)
  if (!('valueOf' in protoRecord)) {
    protoRecord['valueOf'] = function(this: { toString(): string }) {
      return this.toString();
    };
  }
}

/**
 * Apply prototype patches to WASM string-like classes.
 * Called once after WASM initialization.
 */
function applyPrototypePatches(): void {
  // Patch StaticTranslatableString to behave like a String
  patchStringLikePrototype(StaticTranslatableString.prototype);
}

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

    // Apply prototype patches after WASM is initialized
    applyPrototypePatches();

    // Register Rust timing getter for unified tracing
    registerRustTimingGetter(getRscvTimings);

    wasmInitialized = true;
  }
}

export async function run() {
  await initWasm();
  greet();
}

// Re-export WASM types for use in the rest of the codebase
export { WasmStaticNode, WasmStaticGraphMeta };
