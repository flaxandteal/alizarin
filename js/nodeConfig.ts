/**
 * Node configuration management.
 *
 * TypeScript wrappers around the Rust/WASM NodeConfigManager.
 * Config parsing and storage is done in Rust, shared with Python bindings.
 */

import { INodeConfig } from './interfaces';
import { StaticNode, StaticDomainValue, StaticGraph } from './static-types';
import {
  WasmNodeConfigManager,
  WasmNodeConfigBoolean,
  WasmNodeConfigConcept,
  WasmNodeConfigDomain,
  WasmNodeConfigReference,
  WasmStaticDomainValue,
} from '../pkg/alizarin';

// =============================================================================
// Config Type Adapters
// =============================================================================

class StaticNodeConfigBoolean implements INodeConfig {
  private _wasm: WasmNodeConfigBoolean;

  constructor(wasmConfig: WasmNodeConfigBoolean) {
    this._wasm = wasmConfig;
  }

  get i18n_properties(): string[] {
    return [];
  }

  get falseLabel(): {[key: string]: string} {
    return this._wasm.falseLabel as {[key: string]: string};
  }

  get trueLabel(): {[key: string]: string} {
    return this._wasm.trueLabel as {[key: string]: string};
  }

  getLabel(value: boolean, language: string): string | undefined {
    return this._wasm.getLabel(value, language) ?? undefined;
  }
}

class StaticNodeConfigConcept implements INodeConfig {
  private _wasm: WasmNodeConfigConcept;

  constructor(wasmConfig: WasmNodeConfigConcept) {
    this._wasm = wasmConfig;
  }

  get rdmCollection(): string {
    return this._wasm.rdmCollection;
  }
}

class StaticNodeConfigReference implements INodeConfig {
  private _wasm: WasmNodeConfigReference;

  constructor(wasmConfig: WasmNodeConfigReference) {
    this._wasm = wasmConfig;
  }

  get controlledList(): string {
    return this._wasm.controlledList;
  }

  get rdmCollection(): string {
    return this._wasm.rdmCollection;
  }

  get multiValue(): boolean {
    return this._wasm.multiValue;
  }

  getCollectionId(): string | undefined {
    return this._wasm.getCollectionId() ?? undefined;
  }
}

class StaticNodeConfigDomain implements INodeConfig {
  private _wasm: WasmNodeConfigDomain;
  private _optionsCache: StaticDomainValue[] | null = null;

  constructor(wasmConfig: WasmNodeConfigDomain) {
    this._wasm = wasmConfig;
  }

  get i18n_config(): {[key: string]: string} {
    return {};
  }

  get options(): StaticDomainValue[] {
    if (!this._optionsCache) {
      this._optionsCache = this._wasm.options.map((opt: WasmStaticDomainValue) =>
        new StaticDomainValue({
          id: opt.id,
          selected: opt.selected,
          text: opt.text as {[key: string]: string},
        })
      );
    }
    return this._optionsCache;
  }

  getSelected(): StaticDomainValue | undefined {
    const wasmSelected = this._wasm.getSelected();
    if (!wasmSelected) return undefined;
    return new StaticDomainValue({
      id: wasmSelected.id,
      selected: wasmSelected.selected,
      text: wasmSelected.text as {[key: string]: string},
    });
  }

  valueFromId(id: string): StaticDomainValue | undefined {
    const wasmValue = this._wasm.valueFromId(id);
    if (!wasmValue) return undefined;
    return new StaticDomainValue({
      id: wasmValue.id,
      selected: wasmValue.selected,
      text: wasmValue.text as {[key: string]: string},
    });
  }
}

// =============================================================================
// Node Config Manager
// =============================================================================

type NodeConfigType = StaticNodeConfigBoolean | StaticNodeConfigConcept |
                      StaticNodeConfigDomain | StaticNodeConfigReference;

class NodeConfigManager {
  // Lazy initialization - WASM manager created on first use (after WASM is ready)
  private _wasmManager: WasmNodeConfigManager | null = null;
  private _cache = new Map<string, NodeConfigType | null>();
  private _graphsLoaded = new Set<string>();

  private getWasmManager(): WasmNodeConfigManager {
    if (!this._wasmManager) {
      this._wasmManager = new WasmNodeConfigManager();
    }
    return this._wasmManager;
  }

  loadFromGraph(graph: StaticGraph): void {
    const graphId = graph.graphid;
    if (this._graphsLoaded.has(graphId)) return;

    this.getWasmManager().fromGraphJson(JSON.stringify(graph));
    this._graphsLoaded.add(graphId);
  }

  retrieve(node: StaticNode): NodeConfigType | null {
    const { nodeid, datatype } = node;

    if (this._cache.has(nodeid)) {
      return this._cache.get(nodeid) ?? null;
    }

    const config = this.getWasmManager().hasConfig(nodeid)
      ? this.getConfigFromWasm(nodeid, datatype)
      : null;

    this._cache.set(nodeid, config);
    return config;
  }

  private getConfigFromWasm(nodeid: string, datatype: string): NodeConfigType | null {
    const wasmManager = this.getWasmManager();
    switch (datatype) {
      case 'boolean': {
        const wasm = wasmManager.getBoolean(nodeid);
        return wasm ? new StaticNodeConfigBoolean(wasm) : null;
      }
      case 'domain-value':
      case 'domain-value-list': {
        const wasm = wasmManager.getDomain(nodeid);
        return wasm ? new StaticNodeConfigDomain(wasm) : null;
      }
      case 'concept':
      case 'concept-list': {
        const wasm = wasmManager.getConcept(nodeid);
        return wasm ? new StaticNodeConfigConcept(wasm) : null;
      }
      case 'reference': {
        const wasm = wasmManager.getReference(nodeid);
        return wasm ? new StaticNodeConfigReference(wasm) : null;
      }
      default:
        return null;
    }
  }

  clear(): void {
    this._cache.clear();
    this._graphsLoaded.clear();
    if (this._wasmManager) {
      this._wasmManager.clear();
    }
  }

  hasGraph(graphId: string): boolean {
    return this._graphsLoaded.has(graphId);
  }
}

const nodeConfigManager = new NodeConfigManager();

export {
  nodeConfigManager,
  StaticNodeConfigDomain,
  StaticNodeConfigBoolean,
  StaticNodeConfigConcept,
  StaticNodeConfigReference,
  NodeConfigManager,
};
