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
export function getNapiModule(): any {
  if (_napiModule) return _napiModule;
  // Check globalThis in case setNapiModule was called from another module instance
  if ((globalThis as any).__alizarin_napi) {
    _napiModule = (globalThis as any).__alizarin_napi;
    return _napiModule;
  }
  return null;
}

/**
 * Register a pre-loaded NAPI module (for when require() path resolution
 * doesn't work, e.g., in bundled environments).
 */
export function setNapiModule(mod: any) {
  _napiModule = mod;
  _backend = 'napi';
  // Expose for modules that can't import backend.ts without circular deps
  (globalThis as any).__alizarin_napi = mod;
}

// ============================================================================
// Instance wrapper factories
// ============================================================================

/**
 * Create an instance wrapper for a resource.
 * Returns either a WASMResourceInstanceWrapper or NapiResourceInstanceWrapper,
 * both of which share the same method interface.
 *
 * If registry is provided in NAPI mode, tiles are loaded directly from Rust
 * (avoids the JS→Rust serialization round-trip for tile data).
 */
export function createInstanceWrapperForResource(resource: StaticResource, registry?: any): any {
  if (_backend === 'napi') {
    const napi = getNapiModule();
    if (!napi) throw new Error('NAPI backend selected but @alizarin/napi not available');
    const wrapper = new napi.NapiResourceInstanceWrapper(resource.resourceinstance.graph_id);
    // Try direct registry load (avoids re-serializing all tile data through JS)
    const resourceId = resource.resourceinstance.resourceinstanceid;
    if (registry && typeof wrapper.loadFromRegistry === 'function') {
      const loaded = wrapper.loadFromRegistry(resourceId, registry);
      if (!loaded) {
        // Fallback to JS round-trip if not found in registry
        wrapper.loadTilesFromResource(resource);
      }
    } else {
      wrapper.loadTilesFromResource(resource);
    }
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
// RDM cache factory
// ============================================================================

let _rdmCache: any = null;

/**
 * Get or create a backend-appropriate RDM cache.
 * In WASM mode returns WasmRdmCache; in NAPI mode returns NapiRdmCache.
 */
export function getRdmCache(): any {
  if (_rdmCache) return _rdmCache;
  if (_backend === 'napi') {
    const napi = getNapiModule();
    if (!napi) throw new Error('NAPI backend selected but @alizarin/napi not available');
    _rdmCache = new napi.NapiRdmCache();
  } else {
    const { RdmCache } = getWasmModule();
    _rdmCache = new RdmCache();
  }
  return _rdmCache;
}

let _nodeConfigManager: any = null;

/**
 * Get or create a backend-appropriate node config manager.
 */
export function getNodeConfigManager(): any {
  if (_nodeConfigManager) return _nodeConfigManager;
  if (_backend === 'napi') {
    const napi = getNapiModule();
    if (!napi) throw new Error('NAPI backend selected but @alizarin/napi not available');
    _nodeConfigManager = new napi.NapiNodeConfigManager();
  } else {
    const { NodeConfigManager } = getWasmModule();
    _nodeConfigManager = new NodeConfigManager();
  }
  return _nodeConfigManager;
}

// ============================================================================
// Extension handler registration
// ============================================================================

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
export function registerExtensionHandler(datatype: string, handler: any): void {
  if (_backend === 'napi') {
    // NAPI: handlers are compiled into the native binary. Verify it's known.
    const napi = getNapiModule();
    if (napi && typeof napi.hasExtensionHandler === 'function') {
      if (!napi.hasExtensionHandler(datatype)) {
        console.warn(`[alizarin] Extension handler for '${datatype}' not found in NAPI binary`);
      }
    }
    return;
  }
  // WASM: register JS handler callbacks into Rust via wasm-bindgen
  const mod = getWasmModule();
  mod.registerExtensionHandler(datatype, handler);
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
// Static type factories (WASM ↔ plain objects for NAPI)
// ============================================================================

/**
 * Generic factory: create a WASM type instance, or return plain object in NAPI mode.
 *
 * ALL WASM types (StaticGraph, StaticNode, StaticTile, etc.) are serde wrappers
 * that take a JS object, deserialize into Rust, and provide getters. In NAPI mode,
 * we just use plain objects — NAPI stringifies them when it needs them in Rust.
 */
export function createWasmType(className: string, data: any): any {
  if (_backend === 'napi') {
    return data;
  }
  const Cls = getWasmModule()[className];
  if (!Cls) throw new Error(`Unknown WASM type: ${className}`);
  return new Cls(data);
}

// Convenience wrappers for commonly-used types:

export function createStaticGraphMeta(data: any): any {
  return createWasmType('StaticGraphMeta', data);
}

export function createStaticGraph(data: any): any {
  if (_backend === 'napi') {
    // Add mutation methods that mirror the WASM StaticGraph API
    const graph = { ...data };
    graph.nodes = graph.nodes ? [...graph.nodes] : [];
    graph.edges = graph.edges ? [...graph.edges] : [];
    graph.nodegroups = graph.nodegroups ? [...graph.nodegroups] : [];
    graph.cards = graph.cards ? [...(graph.cards || [])] : [];
    graph.cards_x_nodes_x_widgets = graph.cards_x_nodes_x_widgets ? [...graph.cards_x_nodes_x_widgets] : [];
    graph.pushNode = function(node: any) { this.nodes.push(node); };
    graph.pushEdge = function(edge: any) { this.edges.push(edge); };
    graph.pushNodegroup = function(ng: any) { this.nodegroups.push(ng); };
    graph.pushCard = function(card: any) { this.cards.push(card); };
    graph.pushCardXNodeXWidget = function(cxnxw: any) { this.cards_x_nodes_x_widgets.push(cxnxw); };
    graph.copy = function() { return createStaticGraph(this); };
    return graph;
  }
  const Cls = getWasmModule()['StaticGraph'];
  if (!Cls) throw new Error('Unknown WASM type: StaticGraph');
  return new Cls(data);
}

export function createStaticNode(data: any): any {
  return createWasmType('StaticNode', data);
}

export function createStaticTile(data: any): any {
  return createWasmType('StaticTile', data);
}

export function createStaticEdge(data: any): any {
  return createWasmType('StaticEdge', data);
}

export function createStaticResource(data: any): any {
  return createWasmType('StaticResource', data);
}

export function createStaticNodegroup(data: any): any {
  return createWasmType('StaticNodegroup', data);
}

export function createStaticCard(data: any): any {
  return createWasmType('StaticCard', data);
}

export function createStaticPublication(data: any): any {
  return createWasmType('StaticPublication', data);
}

export function createStaticCardsXNodesXWidgets(data: any): any {
  return createWasmType('StaticCardsXNodesXWidgets', data);
}

export function createStaticTranslatableString(data: any): any {
  if (_backend === 'napi') {
    // Plain object or string — mimic the interface consumers expect
    if (typeof data === 'string') {
      return { en: data, toString() { return data; }, toJSON() { return data; }, toStringDefault() { return data; } };
    }
    if (typeof data === 'object' && data !== null) {
      const first = Object.values(data)[0] as string || '';
      return { ...data, toString() { return (data as any)[Object.keys(data)[0]] || ''; }, toJSON() { return data; }, toStringDefault() { return first; } };
    }
    return { toString() { return String(data ?? ''); }, toJSON() { return data; }, toStringDefault() { return String(data ?? ''); } };
  }
  const { StaticTranslatableString } = getWasmModule();
  return new StaticTranslatableString(data);
}

/**
 * Parse a JSON string into a StaticGraph.
 * In WASM mode uses the Rust parser; in NAPI mode just JSON.parse (NAPI
 * stringifies again when it needs the data in Rust).
 */
export function parseStaticGraph(jsonText: string): any {
  if (_backend === 'napi') {
    const parsed = JSON.parse(jsonText);
    // Handle the { graph: [...] } wrapper format
    if (parsed.graph && Array.isArray(parsed.graph) && parsed.graph.length === 1) {
      return parsed.graph[0];
    }
    return parsed;
  }
  const { StaticGraph } = getWasmModule();
  return StaticGraph.fromJsonString(jsonText);
}

/**
 * Parse a business_data JSON string into StaticResource[].
 * In WASM mode uses the Rust bulk parser; in NAPI mode JSON.parse.
 */
export function parseStaticResources(jsonText: string): any[] {
  if (_backend === 'napi') {
    const parsed = JSON.parse(jsonText);
    const resources = parsed?.business_data?.resources;
    return Array.isArray(resources) ? resources : [];
  }
  const { StaticResource } = getWasmModule();
  return StaticResource.fromBusinessDataJsonString(jsonText);
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
