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

let _backend: BackendType = 'wasm';
let _napiModule: any = null;
let _wasmModule: any = null;

/**
 * Register the loaded WASM module. Called by _wasm.ts after initialization.
 * This avoids require() of ESM packages in factory functions.
 */
export function setWasmModule(mod: any) {
  _wasmModule = mod;
}

function getWasmModule(): any {
  if (!_wasmModule) {
    throw new Error('WASM module not loaded. Call initWasm() first, or use setBackend("napi").');
  }
  return _wasmModule;
}

/**
 * Set the active backend. Must be called before creating any instance wrappers.
 */
export function setBackend(backend: BackendType) {
  _backend = backend;
  if (backend === 'napi') {
    _napiModule = getNapiModule();
  }
}

export function getBackend(): BackendType {
  return _backend;
}

/**
 * Try to load the NAPI module. Returns null if unavailable.
 */
function getNapiModule(): any {
  if (_napiModule) return _napiModule;
  try {
    // Native addons need CJS require() — ESM import() is async and callers are sync
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    return require('@alizarin/napi');
  } catch {
    return null;
  }
}

/**
 * Register a pre-loaded NAPI module (for when require() path resolution
 * doesn't work, e.g., in bundled environments).
 */
export function setNapiModule(mod: any) {
  _napiModule = mod;
  _backend = 'napi';
}

// ============================================================================
// Instance wrapper factories
// ============================================================================

/**
 * Create an instance wrapper for a resource.
 * Returns either a WASMResourceInstanceWrapper or NapiResourceInstanceWrapper,
 * both of which share the same method interface.
 */
export function createInstanceWrapperForResource(resource: StaticResource): any {
  if (_backend === 'napi') {
    const napi = getNapiModule();
    if (!napi) throw new Error('NAPI backend selected but @alizarin/napi not available');
    const wrapper = new napi.NapiResourceInstanceWrapper(resource.resourceinstance.graph_id);
    wrapper.loadTilesFromResource(resource);
    return wrapper;
  }
  const { newWASMResourceInstanceWrapperForResource } = getWasmModule();
  return newWASMResourceInstanceWrapperForResource(resource);
}

/**
 * Create an instance wrapper for a model (no resource data yet).
 */
export function createInstanceWrapperForModel(graphId: string): any {
  if (_backend === 'napi') {
    const napi = getNapiModule();
    if (!napi) throw new Error('NAPI backend selected but @alizarin/napi not available');
    return new napi.NapiResourceInstanceWrapper(graphId);
  }
  const { newWASMResourceInstanceWrapperForModel } = getWasmModule();
  return newWASMResourceInstanceWrapperForModel(graphId);
}

// ============================================================================
// Resource registry factory
// ============================================================================

/**
 * Create a resource registry (for staticStore).
 */
export function createResourceRegistry(): any {
  if (_backend === 'napi') {
    const napi = getNapiModule();
    if (!napi) throw new Error('NAPI backend selected but @alizarin/napi not available');
    return new napi.NapiStaticResourceRegistry();
  }
  const { StaticResourceRegistry } = getWasmModule();
  return new StaticResourceRegistry();
}

// ============================================================================
// Model wrapper factory
// ============================================================================

/**
 * Create a model wrapper backend (graph schema operations).
 * Returns either a WASMResourceModelWrapper or NapiResourceModelWrapper.
 *
 * In WASM mode, the wkrm is passed to the constructor (WASM stores it).
 * In NAPI mode, only the graph and defaultAllow are needed — wkrm is
 * stored in the TS ResourceModelWrapper directly.
 */
export function createResourceModelWrapper(wkrm: any, graph: any, defaultAllow: boolean): any {
  if (_backend === 'napi') {
    const napi = getNapiModule();
    if (!napi) throw new Error('NAPI backend selected but @alizarin/napi not available');
    // NapiResourceModelWrapper takes graph JSON string + defaultAllow
    const graphJson = typeof graph === 'string' ? graph : JSON.stringify(graph);
    return new napi.NapiResourceModelWrapper(graphJson, defaultAllow);
  }
  const { WASMResourceModelWrapper } = getWasmModule();
  return new WASMResourceModelWrapper(wkrm, graph, defaultAllow);
}

// ============================================================================
// WKRM factory
// ============================================================================

/**
 * Convert a string with underscores, hyphens, or spaces to PascalCase.
 * Matches the Rust WKRM::to_pascal_case implementation.
 */
function toPascalCase(s: string): string {
  return s
    .replace(/[_-]/g, ' ')
    .split(/\s+/)
    .filter(word => word.length > 0)
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join('');
}

/**
 * Extract a display string from a StaticTranslatableString (or plain string/object).
 */
function extractTranslatableName(name: any): string {
  if (!name) return 'Unnamed';
  if (typeof name === 'string') return name;
  // WASM StaticTranslatableString — has toString or toStringDefault
  if (typeof name.toStringDefault === 'function') return name.toStringDefault();
  if (typeof name.toString === 'function' && name.toString !== Object.prototype.toString) {
    return name.toString();
  }
  // Plain object with language keys, e.g. { "en": "Name" }
  if (typeof name === 'object') {
    const values = Object.values(name);
    if (values.length > 0 && typeof values[0] === 'string') return values[0] as string;
  }
  return 'Unnamed';
}

/**
 * Pure TS WKRM (Well-Known Resource Model) — metadata container.
 * Provides the same interface as the WASM WKRM class.
 */
class TSWkrm {
  graphId: string;
  modelName: string;
  modelClassName: string;
  meta: any;

  constructor(meta: any) {
    this.meta = meta;
    this.graphId = meta.graphid;
    this.modelName = extractTranslatableName(meta.name);
    const baseName = meta.slug || this.modelName;
    this.modelClassName = toPascalCase(baseName);
  }
}

/**
 * Create a WKRM (Well-Known Resource Model) from graph metadata.
 * In WASM mode uses the Rust WKRM class; in NAPI mode uses a pure TS equivalent.
 */
export function createWKRM(meta: any): any {
  if (_backend === 'napi') {
    return new TSWkrm(meta);
  }
  const { WKRM } = getWasmModule();
  return new WKRM(meta);
}

/**
 * Get the WKRM class itself (for instanceof checks or direct construction).
 */
export function getWKRMClass(): any {
  if (_backend === 'napi') return TSWkrm;
  const { WKRM } = getWasmModule();
  return WKRM;
}

// ============================================================================
// Auto-detection
// ============================================================================

/**
 * Auto-detect the best backend for the current environment.
 * Prefers NAPI in Node.js when available, falls back to WASM.
 */
export function autoDetectBackend(): BackendType {
  // Check environment variable override
  if (typeof process !== 'undefined' && process.env?.ALIZARIN_BACKEND) {
    const env = process.env.ALIZARIN_BACKEND.toLowerCase();
    if (env === 'napi' || env === 'wasm') return env as BackendType;
  }

  // In Node.js, try NAPI first
  if (typeof process !== 'undefined' && process.versions?.node) {
    const napi = getNapiModule();
    if (napi) return 'napi';
  }

  return 'wasm';
}
