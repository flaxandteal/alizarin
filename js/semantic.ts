import {
  IStringKeyedObject,
  IViewModel,
  IPseudo,
  IRIVM,
} from "./interfaces.ts";
import { AttrPromise } from "./utils";
import { PseudoValue, PseudoList, wrapRustPseudo } from "./pseudos";
import {
  StaticTile,
  StaticNode,
} from "./static-types";

const TILE_LOADING_ERRORS = null; // "suppress" or "silence" TODO: enum

function tileLoadingError(reason: string, exc: any) {
  if (TILE_LOADING_ERRORS !== "silence") {
    console.error(reason, exc);
    if (TILE_LOADING_ERRORS !== "suppress") {
      throw exc;
    }
  }
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
        if (key in object) {
          object[key] = value;
        } else if (k.startsWith("__") || k in object) {
          object[k] = value;
        } else {
          object.__set(k, value);
        }
        return true;
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

  async __update(map: Map<string, any>) {
    return Promise.all(
      [...map.entries()].map(([k, v]) => {
        this.__set(k, v);
      }),
    );
  }

  async __get(key: string) {
    const childValue = await this.__getChildValue(key);
    return childValue.getValue();
  }

  async __set(key: string, value: any) {
    throw Error("TODO");
  // async __set(key: string, value: any) {
    if (!this.__childNodes.has(key)) {
      throw Error(
        `Semantic node does not have this key: ${key} (${[...this.__childNodes.keys()]})`,
      );
    }

    throw Error(`Setting semantic keys (${key} = ${value}) is not implemented yet in Javascript`);
    // const child = await this.__getChildValue(key, true);
    // child.value = value;
  }

  __has(key: string) {
    const childAliases = this.__parentWkri.$.model.getChildNodeAliases(this.__node.nodeid);
    return childAliases.includes(key);
  }

  async __getChildTypes() {
    throw Error("TODO");
    const promises = [...this.__childNodes.keys()].map(async (key): Promise<[string, IPseudo]> => [
      key,
      await this.__getChildValue(key),
    ]);
    const entries: Array<[string, IPseudo]> = await Promise.all(promises);
    return new Map<string, any>([...entries]);
  }

  async __getChildren(direct: null | boolean = null) {
    const items = new Map<string, any>();
    for (const [key, value] of [...(await this.__getChildValues()).entries()]) {
      items.set(key, value);
    }
    const children = [...items.entries()]
      .filter((entry) => {
    throw Error("TODO");
        const child = this.__childNodes.get(entry[0]);
        if (!child) {
          throw Error("Child key is not in child nodes");
        }
        return (
          (direct === null || direct === !child.is_collector) &&
          entry[1] !== null
        );
      })
      .map((entry) => entry[1]);
    return children;
  }

  async __getChildValue(key: string, setDefault: boolean = false): Promise<IPseudo> {
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

    // Call Rust to get the semantic child value
    const rustValue = parent.$.wasmWrapper.getSemanticChildValue(
      tile?.tileid || null,
      node.nodeid,
      node.nodegroup_id || "",
      key
    );

    let child;
    if (rustValue === null || rustValue === undefined) {
      // Child not found - always create pseudo
      child = this.__makePseudo(key);
    } else {
      // Wrap the Rust value
      child = wrapRustPseudo(rustValue, parent, parent.$.model);
    }

    // Set parent node
    child.parentNode = this.__parentPseudo || null;

    // Cache the child
    this.__childValues.set(key, child);

    return child;
  }

  __makePseudo(key: string): IPseudo {
    const parent = this.__parentWkri;
    const childNode = parent.$.model.getNodeObjectFromAlias(key);

    if (!childNode) {
      throw Error(`Child node key ${key} missing`);
    }

    if (!this.__parentWkri) {
      throw Error("This semantic node is currently parentless (no WKRI)");
    }

    if (!this.__parentWkri.$) {
      // Could autoretreive?
      throw Error("This semantic node is currently on an unloaded WKRI");
    }

    const tile = (!childNode.is_collector && childNode.nodegroup_id === this.__node.nodegroup_id) ? this.__tile : null; // Does it share a tile
    const child = this.__parentWkri.$.addPseudo(childNode, tile, this.__node);
    child.parentNode = this.__parentPseudo || null;
    return child;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    parent: IRIVM<any> | null,
  ): Promise<SemanticViewModel> {
    const svm = new SemanticViewModel(parent, tile, node);
    if (value) {
      try {
        await svm.__update(value);
      } catch (e) {
        tileLoadingError(
          `
          Suppressed a tile loading error: ${e}: ${typeof e} (tile: ${tile}; node: ${node}) - ${value}
        `,
          e,
        );
      }
    }
    // await svm.__getChildren();
    return svm;
  }

  async __asTileData() {
    // Ensure all nodes have populated the tile
    const relationships: any[] = [];
    for (const value of [...await this.__getChildren(true)]) {
      // We do not use tile, because a child node will ignore its tile reference.
      const [, subrelationships] = await value.getTile();
      relationships.push(...subrelationships);
    }
    // This is none because the semantic type has no nodal value,
    // only its children have nodal values, and the nodal value of this nodeid should
    // not exist.
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

      for (let value of values) {
        // It is possible that this value has already been requested, but the tile is in-flight.
        value = await value;
        if (
          value !== null &&
          value.node &&
          (!(value.parentNode) || value.parentNode === this.__parentPseudo)
        ) {
          if (!value.node) {
            throw Error(`Node ${childNode.alias} (${childNode.nodeid}) is unavailable`);
          }

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
