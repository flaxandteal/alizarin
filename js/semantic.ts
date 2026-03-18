import {
  IStringKeyedObject,
  IViewModel,
  IPseudo,
  IRIVM,
} from "./interfaces.ts";
import { AttrPromise } from "./utils";
import { wrapRustPseudo } from "./pseudos";
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
};

export function resetTimingStats() {
  timingStats.wasmCalls = 0;
  timingStats.wasmTotalMs = 0;
  timingStats.wrapCalls = 0;
  timingStats.wrapTotalMs = 0;
}

export function getTimingStats() {
  return {
    ...timingStats,
    wasmAvgMs: timingStats.wasmCalls > 0 ? timingStats.wasmTotalMs / timingStats.wasmCalls : 0,
    wrapAvgMs: timingStats.wrapCalls > 0 ? timingStats.wrapTotalMs / timingStats.wrapCalls : 0,
  };
}

export function logTimingStats(label: string = '') {
  const stats = getTimingStats();
  console.log(`[timing-stats] ${label} wasm: ${stats.wasmCalls} calls, ${stats.wasmTotalMs.toFixed(1)}ms total (${stats.wasmAvgMs.toFixed(2)}ms avg) | wrap: ${stats.wrapCalls} calls, ${stats.wrapTotalMs.toFixed(1)}ms total`);
}

class SemanticViewModel implements IStringKeyedObject, IViewModel {
  [key: string | symbol]: any;
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  then: undefined;
  [Symbol.toPrimitive]: undefined;

  get [Symbol.toStringTag]() { return 'SemanticViewModel'; }

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

    const wasmWrapper = this.__parentWkri.$.wasmWrapper;
    const model = this.__parentWkri.$.model;

    // Step 1: Check which nodegroups need tiles loaded (sync call)
    const t0 = performance.now();
    const missingNodegroups = wasmWrapper.getMissingNodegroupsForChildren(this.__node.nodeid);
    timingStats.wasmCalls++;
    timingStats.wasmTotalMs += performance.now() - t0;

    // Step 2: Load any missing tiles (async - the only async part)
    if (missingNodegroups.length > 0) {
      const tileLoader = wasmWrapper.getTileLoader();
      if (tileLoader) {
        // Load tiles for each missing nodegroup
        for (const nodegroupId of missingNodegroups) {
          const t1 = performance.now();
          const tiles = await tileLoader(nodegroupId);
          timingStats.wasmTotalMs += performance.now() - t1;
          if (tiles && tiles.length > 0) {
            const t2 = performance.now();
            wasmWrapper.appendTiles(tiles);
            timingStats.wasmCalls++;
            timingStats.wasmTotalMs += performance.now() - t2;
          }
        }
      }
    }

    // Step 3: Get all child values in a single sync batch call (no async boundary crossings)
    const t3 = performance.now();
    const childValuesMap: Map<string, any> = wasmWrapper.getAllSemanticChildValues(
      this.__tile?.tileid || null,
      this.__node.nodeid,
      this.__node.nodegroup_id || null,
    );
    timingStats.wasmCalls++;
    timingStats.wasmTotalMs += performance.now() - t3;

    // Step 4: Wrap all values and build result object
    // Use Promise.all to batch all getValue() calls - reduces promise callback overhead
    const t4 = performance.now();
    const wrappedChildren: Array<[string, any]> = [];
    for (const [alias, rustValue] of childValuesMap.entries()) {
      if (rustValue === null || rustValue === undefined) {
        continue;
      }
      const child = wrapRustPseudo(rustValue, this.__parentWkri, model);
      timingStats.wrapCalls++;
      if (child === null || child === undefined) {
        continue;
      }
      child.parentNode = this.__parentPseudo || null;
      // Cache the wrapped child for future use
      this.__childValues.set(alias, child);
      wrappedChildren.push([alias, child]);
    }

    // Batch all getValue() calls with Promise.all instead of sequential awaits
    const valuePromises = wrappedChildren.map(([alias, child]) =>
      child.getValue().then((value: any) => [alias, value] as [string, any])
    );
    const resolvedValues = await Promise.all(valuePromises);

    const result: Record<string, any> = {};
    for (const [alias, value] of resolvedValues) {
      result[alias] = value;
    }
    timingStats.wrapTotalMs += performance.now() - t4;

    return result;
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
    const wasmWrapper = parent.$.wasmWrapper;
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

    // Try sync path first (avoids async boundary crossing overhead)
    let rustValue: any;
    const t0 = performance.now();
    try {
      // Sync call - returns immediately if tiles loaded, throws if not
      rustValue = wasmWrapper.getSemanticChildValue(
        tile?.tileid || null,
        node.nodeid,
        node.nodegroup_id || null,
        key
      );
      timingStats.wasmCalls++;
      timingStats.wasmTotalMs += performance.now() - t0;
    } catch (e: any) {
      const errorStr = e?.message || String(e);
      if (errorStr.startsWith('TILES_NOT_LOADED:')) {
        // Tiles not loaded - load them and retry
        const nodegroupId = errorStr.split(':')[1].split(' ')[0]; // Extract nodegroup ID
        const tileLoader = wasmWrapper.getTileLoader();
        if (tileLoader) {
          const t1 = performance.now();
          const tiles = await tileLoader(nodegroupId);
          timingStats.wasmTotalMs += performance.now() - t1;
          if (tiles && tiles.length > 0) {
            const t2 = performance.now();
            wasmWrapper.appendTiles(tiles);
            timingStats.wasmCalls++;
            timingStats.wasmTotalMs += performance.now() - t2;
          }
        }
        // Retry sync call after loading tiles
        const t3 = performance.now();
        rustValue = wasmWrapper.getSemanticChildValue(
          tile?.tileid || null,
          node.nodeid,
          node.nodegroup_id || null,
          key
        );
        timingStats.wasmCalls++;
        timingStats.wasmTotalMs += performance.now() - t3;
      } else {
        throw e; // Re-throw non-tile-loading errors
      }
    }

    // Wrap the Rust value - Rust always returns something (possibly empty PseudoList)
    const t4 = performance.now();
    const child = wrapRustPseudo(rustValue, parent, parent.$.model);
    timingStats.wrapCalls++;
    timingStats.wrapTotalMs += performance.now() - t4;

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
