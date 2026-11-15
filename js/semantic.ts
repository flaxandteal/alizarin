import {
  IStringKeyedObject,
  IViewModel,
  IPseudo,
  IRIVM,
} from "./interfaces.ts";
import { AttrPromise } from "./utils";
import { wrapRustPseudo, PseudoList } from "./pseudos";
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
        // A pending edit, applied by the resource's $commit()
        const wrapper = object.__parentWkri?.$;
        if (!wrapper) {
          throw Error(`Cannot set ${k}: this value has no resource to edit`);
        }
        wrapper.assign(object.__tile, object.__node, k, value);
        return true;
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
        return new AttrPromise((resolve, reject) => {
          const p = object.__get(k);
          p.then(resolve, reject);
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

    // Step 2: Load any missing tiles (async - the only async part); each
    // fetch marks its nodegroup loaded, even when it has no tiles
    const t1 = performance.now();
    await this.__parentWkri.$.fetchNodegroups(missingNodegroups);
    timingStats.wasmCalls++;
    timingStats.wasmTotalMs += performance.now() - t1;

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
      // An empty list can't tell its parent tile from its items: it is ours.
      if (Array.isArray(child) && (child as any).parenttileId === undefined) {
        (child as any).parenttileId = this.__tile?.tileid ?? undefined;
      }
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

  /**
   * An empty, editable list for child `key` when it is multi-valued (its own
   * cardinality-n nodegroup) and has no values yet, so it can be pushed to;
   * null for any other child. As Python's SemanticViewModel.
   */
  __emptyChildList(key: string): PseudoList | null {
    const wrapper = this.__parentWkri?.$;
    const child = wrapper?.model.getChildNodes(this.__node.nodeid).get(key);
    if (!wrapper || !child?.nodegroup_id || child.nodegroup_id === this.__tile?.nodegroup_id) {
      return null;
    }
    if (wrapper.model.getNodegroupObjects().get(child.nodegroup_id)?.cardinality !== 'n') {
      return null;
    }
    const list = new PseudoList();
    list.initialize(child, this.__parentWkri);
    list.__alias = key;
    list.parenttileId = this.__tile?.tileid ?? undefined;
    list.__editable = true;
    return list;
  }

  async __get(key: string) {
    // A value assigned since the last $commit()
    const pending = this.__parentWkri?.$?.pendingValue(this.__tile?.tileid, key);
    if (pending) {
      return pending.value;
    }
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
        // Tiles not loaded - load every listed nodegroup, then retry
        const nodegroupIds = errorStr
          .slice('TILES_NOT_LOADED:'.length)
          .split(' ')[0]
          .split(',');
        const t1 = performance.now();
        await parent.$.fetchNodegroups(nodegroupIds);
        timingStats.wasmCalls++;
        timingStats.wasmTotalMs += performance.now() - t1;
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
      return this.__emptyChildList(key) ?? child;
    }

    // Set parent node
    child.parentNode = this.__parentPseudo || null;
    // An empty list can't tell its parent tile from its items: it is ours.
    if (Array.isArray(child) && (child as any).parenttileId === undefined) {
      (child as any).parenttileId = this.__tile?.tileid ?? undefined;
    }

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
