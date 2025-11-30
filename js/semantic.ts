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
    const entries = [...(await this.__getChildValues()).entries()];
    return Object.fromEntries(await Promise.all(entries.map(async ([k, vl]) => {
        return [k, (await vl).getValue()];
    })));
  }

  async forJson() {
    async function _forJson(v: IPseudo | IViewModel | null) {
      v = await v;
      if (!v) {
        return null;
      }
      return await v.forJson();
    };
    const entries = [...(await this.__getChildValues()).entries()];
    return Object.fromEntries(await Promise.all(entries.map(async ([k, vl]) => {
        return [k, vl ? await _forJson(vl) : vl];
    })));
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
    const rustValue = await parent.$.wasmWrapper.retrieveSemanticChildValue(
      tile?.tileid || null,
      node.nodeid,
      node.nodegroup_id || null,
      key
    );

    // Wrap the Rust value - Rust always returns something (possibly empty PseudoList)
    const child = wrapRustPseudo(rustValue, parent, parent.$.model);

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

  async __getChildValues(): Promise<Map<string, IPseudo>> {
    const parent = this.__parentWkri;
    const tile = this.__tile;
    const node = this.__node;
    const childAliases = parent.$.model.getChildNodeAliases(node.nodeid);
    if (!parent || !parent.$) {
      return new Map();
    }

    // Ensure lazy-loading done.
    // TODO check this does not go deeper than necessary.
    await parent.$.loadNodes(childAliases);

    // Use Rust implementation to find which tiles contain semantic children
    // PORT: Replaces lines 163-233 with Rust findSemanticChildren
    const matchingTiles: Map<string, string[]> = parent.$.wasmWrapper.findSemanticChildren(
      tile?.tileid || null,
      node.nodeid,
      node.nodegroup_id || "",
      []
    );

    const children: Map<string, any> = new Map();
    const allEntries = [...parent.$.allEntries()];

    // Process results from Rust - for each child alias that has matching tiles
    for (const [childAlias, tileIds] of matchingTiles.entries()) {
      // Get the PseudoValue(s) from allEntries
      const entry = allEntries.find(e => e[0] === childAlias);
      if (!entry) {
        throw Error("Missing entry!");
      }

      let values = entry[1];
      if (values instanceof Promise) {
        values = await values;
      }
      if (values === false || values === null || values === undefined) {
        continue;
      }

      // Filter values to only those matching the tileIds from Rust
      const tileIdSet = new Set(tileIds);

      const childNode = parent.$.model.getNodeObjectFromAlias(childAlias);
      if (!childNode) {
        throw Error(`Child node alias ${childAlias} not found in model`);
      }

      for (let value of values) {
        // It is possible that this value has already been requested, but the tile is in-flight.
        value = await value;
        if (
          value !== null &&
          value.node &&
          (!(value.parentNode) || value.parentNode === this.__parentPseudo)
        ) {
          // Check if this value's tile matches one of the tiles Rust identified
          const valueTileId = value.tile?.tileid;
          if (!valueTileId || !tileIdSet.has(valueTileId)) {
            continue;
          }

          // Handle collector nodes (Branch 3 from Rust implementation)
          if (
            node.nodegroup_id != value.node.nodegroup_id &&
            childNode.is_collector
          ) {
            // This avoids list types that have their own tiles (like resource or concept lists)
            // from appearing doubly-nested
            const childValue = value instanceof PseudoList ? value : (value.isIterable() ? await value.getValue() : null);
            let listValue: PseudoList | Array<any> | null;
            if (childValue && Array.isArray(childValue)) {
              listValue = childValue;
            } else {
              listValue = null;
            }
            if (listValue !== null) {
              if (children.has(childAlias)) {
                children.get(childAlias).push(...listValue);
              } else {
                children.set(childAlias, listValue);
              }
            } else {
              // In this case, we have a value, but the wrapper logic did not make it a PseudoList, so
              // we should treat it as singular.
              children.set(childAlias, value);
            }
          } else {
            // Branch 1 and Branch 2 cases
            children.set(childAlias, value);
          }
        }
      }
    }

    // Set parent node for all children and cache
    for (const [key, value] of [...children.entries()]) {
      value.parentNode = this.__parentPseudo;
      this.__childValues.set(key, value);
    }

    return children;
  }
}

export { SemanticViewModel };
