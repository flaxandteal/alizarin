import init, { initSync, StaticNode as WasmStaticNode, StaticGraphMeta as WasmStaticGraphMeta, StaticTranslatableString, WasmRdmCache, getRscvTimings, parseSkosXml, parseSkosXmlToCollection, collectionToSkosXml, collectionsToSkosXml, registerDisplaySerializer, hasDisplaySerializer, unregisterDisplaySerializer, getRegisteredDisplaySerializers } from "../pkg/alizarin";
import { registerRustTimingGetter } from "./tracing";
import wasmURL from "../pkg/alizarin_bg.wasm?url"

let wasmInitialized = false;

// Global WASM RDM cache singleton for concept parent lookups
let _globalWasmRdmCache: WasmRdmCache | null = null;

/**
 * Get the global WASM RDM cache singleton.
 * Creates the cache on first access (after WASM initialization).
 * @throws Error if WASM is not yet initialized
 */
export function getGlobalWasmRdmCache(): WasmRdmCache {
  if (!wasmInitialized) {
    throw new Error("WASM not initialized. Call initWasm() first.");
  }
  if (!_globalWasmRdmCache) {
    _globalWasmRdmCache = new WasmRdmCache();
  }
  return _globalWasmRdmCache;
}

/**
 * Check if the global WASM RDM cache is available.
 */
export function hasGlobalWasmRdmCache(): boolean {
  return wasmInitialized && _globalWasmRdmCache !== null;
}

/**
 * Ensure the WASM RDM cache is initialized and ready.
 * This can be awaited early in the application lifecycle to ensure
 * the cache is available before any hierarchy methods are called.
 * @returns The initialized WasmRdmCache
 */
export async function ensureWasmRdmCache(): Promise<WasmRdmCache> {
  if (!wasmInitialized) {
    await initWasm();
  }
  return getGlobalWasmRdmCache();
}

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
      new StaticTranslatableString('test');
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

// Re-export WASM types for use in the rest of the codebase
export { WasmStaticNode, WasmStaticGraphMeta, WasmRdmCache };

// Re-export SKOS parsing and serialization functions
export { parseSkosXml, parseSkosXmlToCollection, collectionToSkosXml, collectionsToSkosXml };

// Re-export display serializer extension registry functions
export { registerDisplaySerializer, hasDisplaySerializer, unregisterDisplaySerializer, getRegisteredDisplaySerializers };
