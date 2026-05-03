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
 * Try to load the NAPI module. Returns null if unavailable.
 */
export declare function getNapiModule(): any;
/**
 * Register a pre-loaded NAPI module (for when require() path resolution
 * doesn't work, e.g., in bundled environments).
 */
export declare function setNapiModule(mod: any): void;
/**
 * Create an instance wrapper for a resource.
 * Returns either a WASMResourceInstanceWrapper or NapiResourceInstanceWrapper,
 * both of which share the same method interface.
 *
 * If registry is provided in NAPI mode, tiles are loaded directly from Rust
 * (avoids the JS→Rust serialization round-trip for tile data).
 */
export declare function createInstanceWrapperForResource(resource: StaticResource, registry?: any): any;
/**
 * Load tiles onto an instance wrapper from a resource object.
 * In NAPI mode, stringifies the full resource for single-pass Rust deserialization.
 * In WASM mode, if `resource` is a WasmStaticResource, passes it directly;
 * otherwise falls back to loadTiles with the tiles array.
 */
export declare function loadTilesFromResource(wrapper: any, resource: any, assumeComprehensive?: boolean): void;
/**
 * Load tiles onto an instance wrapper from a tiles array.
 * In NAPI mode, stringifies first for single-pass deserialization in Rust.
 * In WASM mode, passes the JS array directly (serde_wasm_bindgen handles it).
 */
export declare function loadTiles(wrapper: any, tiles: any): void;
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
 * Register an extension handler for a custom datatype.
 *
 * In WASM mode, the handler (JS object with coerce/renderDisplay/resolveMarkers)
 * is passed to Rust via wasm-bindgen.
 *
 * In NAPI mode, extension handlers are compiled into the native binary (Rust
 * crates linked at build time). The JS call serves as activation — the NAPI
 * module already has the Rust implementation, so the JS handler object is not
 * needed. This keeps the hot path entirely in Rust without FFI boundary crossings.
 */
export declare function registerExtensionHandler(datatype: string, handler: any): void;
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
 * Generic factory: create a WASM type instance, or return plain object in NAPI mode.
 *
 * ALL WASM types (StaticGraph, StaticNode, StaticTile, etc.) are serde wrappers
 * that take a JS object, deserialize into Rust, and provide getters. In NAPI mode,
 * we just use plain objects — NAPI stringifies them when it needs them in Rust.
 */
export declare function createWasmType(className: string, data: any): any;
export declare function createStaticGraphMeta(data: any): any;
export declare function createStaticGraph(data: any): any;
export declare function createStaticNode(data: any): any;
export declare function createStaticTile(data: any): any;
export declare function createStaticEdge(data: any): any;
export declare function createStaticResource(data: any): any;
export declare function createStaticNodegroup(data: any): any;
export declare function createStaticCard(data: any): any;
export declare function createStaticPublication(data: any): any;
export declare function createStaticCardsXNodesXWidgets(data: any): any;
export declare function createStaticTranslatableString(data: any): any;
/**
 * Parse a JSON string into a StaticGraph.
 * In WASM mode uses the Rust parser; in NAPI mode just JSON.parse (NAPI
 * stringifies again when it needs the data in Rust).
 */
export declare function parseStaticGraph(jsonText: string): any;
/**
 * Parse a business_data JSON string into StaticResource[].
 * In WASM mode uses the Rust bulk parser; in NAPI mode JSON.parse.
 */
export declare function parseStaticResources(jsonText: string): any[];
/**
 * Auto-detect the best backend for the current environment.
 * Prefers NAPI in Node.js when available, falls back to WASM.
 */
export declare function autoDetectBackend(): BackendType;
