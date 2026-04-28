/**
 * Backend abstraction for WASM vs NAPI.
 *
 * When running in Node.js CLI environments (e.g., starches-builder), the NAPI
 * backend avoids WASM linear memory limits. In browsers or when NAPI is
 * unavailable, the WASM backend is used.
 *
 * Usage:
 *   import { setBackend, createInstanceWrapper } from './backend';
 *   setBackend('napi');  // before any wrapper creation
 */
import type { StaticResource } from "./static-types";
export type BackendType = 'wasm' | 'napi';
/**
 * Register the loaded WASM module. Called by _wasm.ts after initialization.
 * This avoids require() of ESM packages in factory functions.
 */
export declare function setWasmModule(mod: any): void;
/**
 * Set the active backend. Must be called before creating any instance wrappers.
 */
export declare function setBackend(backend: BackendType): void;
export declare function getBackend(): BackendType;
/**
 * Register a pre-loaded NAPI module (for when require() path resolution
 * doesn't work, e.g., in bundled environments).
 */
export declare function setNapiModule(mod: any): void;
/**
 * Create an instance wrapper for a resource.
 * Returns either a WASMResourceInstanceWrapper or NapiResourceInstanceWrapper,
 * both of which share the same method interface.
 */
export declare function createInstanceWrapperForResource(resource: StaticResource): any;
/**
 * Create an instance wrapper for a model (no resource data yet).
 */
export declare function createInstanceWrapperForModel(graphId: string): any;
/**
 * Create a resource registry (for staticStore).
 */
export declare function createResourceRegistry(): any;
/**
 * Get or create a backend-appropriate RDM cache.
 * In WASM mode returns WasmRdmCache; in NAPI mode returns NapiRdmCache.
 */
export declare function getRdmCache(): any;
/**
 * Get or create a backend-appropriate node config manager.
 */
export declare function getNodeConfigManager(): any;
/**
 * Create a model wrapper backend (graph schema operations).
 * Returns either a WASMResourceModelWrapper or NapiResourceModelWrapper.
 *
 * In WASM mode, the wkrm is passed to the constructor (WASM stores it).
 * In NAPI mode, only the graph and defaultAllow are needed — wkrm is
 * stored in the TS ResourceModelWrapper directly.
 */
export declare function createResourceModelWrapper(wkrm: any, graph: any, defaultAllow: boolean): any;
/**
 * Create a WKRM (Well-Known Resource Model) from graph metadata.
 * In WASM mode uses the Rust WKRM class; in NAPI mode uses a pure TS equivalent.
 */
export declare function createWKRM(meta: any): any;
/**
 * Get the WKRM class itself (for instanceof checks or direct construction).
 */
export declare function getWKRMClass(): any;
/**
 * Auto-detect the best backend for the current environment.
 * Prefers NAPI in Node.js when available, falls back to WASM.
 */
export declare function autoDetectBackend(): BackendType;
