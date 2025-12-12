import init, { initSync, greet, StaticNode as WasmStaticNode, StaticGraphMeta as WasmStaticGraphMeta, StaticTranslatableString, getRscvTimings, parseSkosXml, parseSkosXmlToCollection, collectionToSkosXml, collectionsToSkosXml } from "../pkg/alizarin";
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
    // Check if WASM is already available (initialized by another module instance)
    try {
      // Try to use a WASM export to see if it's already initialized
      greet();
      console.log('[alizarin] WASM already available from another module instance');
      wasmInitialized = true;
      return;
    } catch (e) {
      // WASM not initialized yet, continue with normal initialization
    }

    // In Node.js environment (tests), use synchronous init with file system
    if (typeof process !== 'undefined' && process.versions?.node) {
      try {
        console.log('[alizarin] Initializing WASM in Node.js environment');
        // Import Node modules only when in Node.js environment
        const fs = await import('fs');
        const path = await import('path');
        const { fileURLToPath } = await import('url');

        // Try multiple possible locations for the WASM file
        // This handles different import contexts (direct vs through extensions)
        const moduleDir = path.dirname(fileURLToPath(import.meta.url));
        const possiblePaths = [
          path.join(moduleDir, '../pkg', 'alizarin_bg.wasm'),      // Normal: js/_wasm.ts -> pkg/
          path.join(process.cwd(), 'pkg', 'alizarin_bg.wasm'),     // Working directory
          path.join(moduleDir, 'alizarin/pkg', 'alizarin_bg.wasm'),// From parent dir (extension context)
          path.join(moduleDir, '../../pkg', 'alizarin_bg.wasm'),   // From dist/bundled location
          path.join(moduleDir, '../../../pkg', 'alizarin_bg.wasm'),// From deeply nested imports
        ];

        let wasmPath: string | undefined;
        for (const candidate of possiblePaths) {
          if (fs.existsSync(candidate)) {
            wasmPath = candidate;
            break;
          }
        }

        if (!wasmPath) {
          const error = new Error(
            `Could not find alizarin_bg.wasm in any of these locations:\n${possiblePaths.map(p => `  - ${p}`).join('\n')}`
          );
          console.error('[alizarin]', error.message);
          throw error;
        }

        const wasmBuffer = fs.readFileSync(wasmPath);
        try {
          initSync({ module: wasmBuffer });
          wasmInitialized = true;
          console.log('[alizarin] WASM initialized successfully in Node.js');
        } catch (initError) {
          const initMsg = initError instanceof Error ? initError.message : String(initError);
          if (initMsg.includes('memory already initialized') || initMsg.includes('unreachable')) {
            console.log('[alizarin] WASM already initialized (detected during initSync), continuing');
            wasmInitialized = true;
            return;
          }
          throw initError;
        }
      } catch (error) {
        // Check if this is a "WASM already initialized" error
        const errorMsg = error instanceof Error ? error.message : String(error);
        if (errorMsg.includes('memory already initialized') || errorMsg.includes('unreachable')) {
          // WASM is already initialized from another import context - this is fine
          console.log('[alizarin] WASM already initialized (from another import), continuing');
          wasmInitialized = true;
          return;
        }
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

// Re-export SKOS parsing and serialization functions
export { parseSkosXml, parseSkosXmlToCollection, collectionToSkosXml, collectionsToSkosXml };
