import {
  IStringKeyedObject,
  IViewModel,
  IPseudo,
  IRIVM,
} from "./interfaces.ts";
import { AttrPromise } from "./utils";
import { PseudoList, wrapRustPseudo } from "./pseudos";
import type { PseudoValue } from "./pseudos";
import {
  StaticTile,
  StaticNode,
} from "./static-types";

// Aggregated timing stats for profiling
const timingStats = {
  wasmCalls: 0,
  wasmTotalMs: 0,
  wrapCalls: 0,
  wrapTotalMs: 0,
  forJsonCalls: 0,
  forJsonTotalMs: 0,
};

export function resetTimingStats() {
  timingStats.wasmCalls = 0;
  timingStats.wasmTotalMs = 0;
  timingStats.wrapCalls = 0;
  timingStats.wrapTotalMs = 0;
  timingStats.forJsonCalls = 0;
  timingStats.forJsonTotalMs = 0;
}

export function getTimingStats() {
  return {
    ...timingStats,
    wasmAvgMs: timingStats.wasmCalls > 0 ? timingStats.wasmTotalMs / timingStats.wasmCalls : 0,
    wrapAvgMs: timingStats.wrapCalls > 0 ? timingStats.wrapTotalMs / timingStats.wrapCalls : 0,
    forJsonAvgMs: timingStats.forJsonCalls > 0 ? timingStats.forJsonTotalMs / timingStats.forJsonCalls : 0,
  };
}

export function logTimingStats(label: string = '') {
  const stats = getTimingStats();
  console.log(`[timing-stats] ${label} wasm: ${stats.wasmCalls} calls, ${stats.wasmTotalMs.toFixed(1)}ms total (${stats.wasmAvgMs.toFixed(2)}ms avg) | wrap: ${stats.wrapCalls} calls, ${stats.wrapTotalMs.toFixed(1)}ms total | forJson: ${stats.forJsonCalls} calls, ${stats.forJsonTotalMs.toFixed(1)}ms total`);
}

class SemanticViewModel implements IStringKeyedObject, IViewModel {
  [key: string | symbol]: any;
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  then: undefined;
  [Symbol.toPrimitive]: undefined;

  __parentPseudo: PseudoValue<any> | undefined;
  __childValues: Map<string, any>;
  __parentWkri: IRIVM<any> | null;
  __tile: StaticTile | null;
  __node: StaticNode;

  __forJsonCache(): null {
    return null;
  }

  constructor(
    parentWkri: IRIVM<any> | null,
    tile: StaticTile | null,
    node: StaticNode,
  ) {
    this.__childValues = new Map<string, any>();
    this.__parentWkri = parentWkri;
    this.__tile = tile;
    this.__node = node;
    return new Proxy(this, {
      set: (object, key, value) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;
        // Allow setting internal properties (starting with __)
        if (k.startsWith("__") || key in object) {
          object[k] = value;
          return true;
        }
        throw Error(`Setting semantic values via proxy (key: ${String(key)}) is not supported`);
      },
      get: (object, key) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;

        if (key.toString() === "Symbol.toStringTag") {
          return () => this.constructor.name;
        }

        if (key in object) {
          return object[key];
        } else if (k.startsWith("__") || k in object) {
          return object[k];
        }
        if (k == "length") {
          throw Error("TODO");
        }
        return new AttrPromise((resolve) => {
          const p = object.__get(k);
          p.then(resolve);
        });
      },
    });
  }

  async toString(): Promise<string> {
    const entries = [...this.__childValues.entries()].map(([k, v]) => `${k}: ${v}`);
    return `[[${entries.join(",")}]]`;
  }

  async toObject() {
    if (!this.__parentWkri || !this.__parentWkri.$) {
      return {};
    }
    const childAliases = this.__parentWkri.$.model.getChildNodeAliases(this.__node.nodeid);
    const entries = await Promise.all(childAliases.map(async (alias) => {
      const child = await this.__getChildValue(alias);
      if (!child) {
        return null;
      }
      const value = await child.getValue();
      return [alias, value];
    }));
    return Object.fromEntries(entries.filter(e => e !== null));
  }

  async forJson() {
    const t0 = performance.now();
    if (!this.__parentWkri || !this.__parentWkri.$) {
      return {};
    }
    const childAliases = this.__parentWkri.$.model.getChildNodeAliases(this.__node.nodeid);
    const entries = await Promise.all(childAliases.map(async (alias) => {
      const child = await this.__getChildValue(alias);
      if (!child) {
        return null;
      }
      const value = await child.forJson();
      return [alias, value];
    }));
    timingStats.forJsonCalls++;
    timingStats.forJsonTotalMs += performance.now() - t0;
    return Object.fromEntries(entries.filter(e => e !== null));
  }

  async __get(key: string) {
    const childValue = await this.__getChildValue(key);
    if (!childValue) {
      return null;
    }
    return childValue.getValue();
  }

  __has(key: string) {
    const childAliases = this.__parentWkri.$.model.getChildNodeAliases(this.__node.nodeid);
    return childAliases.includes(key);
  }

  async __getChildValue(key: string): Promise<IPseudo> | null | undefined {
    const parent = this.__parentWkri;
    const tile = this.__tile;
    const node = this.__node;
    const childAliases = parent.$.model.getChildNodeAliases(node.nodeid);

    if (!childAliases.includes(key)) {
      throw Error(
        `Semantic node does not have this key: ${key} (${childAliases.join(', ')})`,
      );
    }

    // Check cache first
    if (this.__childValues.has(key)) {
      return this.__childValues.get(key);
    }

    // Call Rust to get the semantic child value (with lazy tile loading)
    // Rust now always returns a WasmPseudoList (empty if no values found)
    const t0 = performance.now();
    const rustValue = await parent.$.wasmWrapper.retrieveSemanticChildValue(
      tile?.tileid || null,
      node.nodeid,
      node.nodegroup_id || null,
      key
    );
    timingStats.wasmCalls++;
    timingStats.wasmTotalMs += performance.now() - t0;

    // Wrap the Rust value - Rust always returns something (possibly empty PseudoList)
    const t1 = performance.now();
    const child = wrapRustPseudo(rustValue, parent, parent.$.model);
    timingStats.wrapCalls++;
    timingStats.wrapTotalMs += performance.now() - t1;

    if (child === null || child === undefined) {
      return child;
    }

    // Set parent node
    child.parentNode = this.__parentPseudo || null;

    // Cache the child
    this.__childValues.set(key, child);

    return child;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    _value: any,
    parent: IRIVM<any> | null,
  ): Promise<SemanticViewModel> {
    // Note: value parameter is ignored - semantic nodes don't have direct values,
    // their children are loaded lazily via __getChildValue
    return new SemanticViewModel(parent, tile, node);
  }

  async __asTileData(): Promise<[null, any[]]> {
    // Semantic nodes don't have direct tile values - only their children do.
    // Collect relationships from all cached child values.
    const relationships: any[] = [];
    for (const [_, child] of this.__childValues.entries()) {
      if (child && typeof child.getTile === 'function') {
        const [, childRelationships] = await child.getTile();
        relationships.push(...childRelationships);
      }
    }
    return [null, relationships];
  }
}

export { SemanticViewModel };
