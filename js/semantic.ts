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
    if (!this.__parentWkri || !this.__parentWkri.$) {
      return {};
    }
    const childAliases = this.__parentWkri.$.model.getChildNodeAliases(this.__node.nodeid);
    // DEBUG: Log childAliases for location_data node
    if (this.__node.alias === "location_data" || childAliases.includes("geometry")) {
      console.log(`[SVM.forJson] node=${this.__node.alias}(${this.__node.nodeid}), childAliases=${childAliases.join(',')}`);
    }
    const entries = await Promise.all(childAliases.map(async (alias) => {
      const child = await this.__getChildValue(alias);
      if (!child) {
        // DEBUG
        if (alias === "geometry" || alias === "location_data") {
          console.log(`[SVM.forJson] __getChildValue('${alias}') returned null/falsy`);
        }
        return null;
      }
      const value = await child.forJson();
      // DEBUG
      if (alias === "geometry" || alias === "location_data") {
        console.log(`[SVM.forJson] '${alias}' forJson() =`, value);
      }
      return [alias, value];
    }));
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

    // DEBUG: Log for location_data/geometry
    if (key === "geometry" || key === "location_data") {
      console.log(`[SVM.__getChildValue] key='${key}', tile=${tile?.tileid}, node=${node.nodeid}, childAliases=${childAliases.join(',')}`);
    }

    if (!childAliases.includes(key)) {
      throw Error(
        `Semantic node does not have this key: ${key} (${childAliases.join(', ')})`,
      );
    }

    // Check cache first
    if (this.__childValues.has(key)) {
      if (key === "geometry" || key === "location_data") {
        console.log(`[SVM.__getChildValue] '${key}' found in cache`);
      }
      return this.__childValues.get(key);
    }

    if (key === "geometry" || key === "location_data") {
      console.log(`[SVM.__getChildValue] '${key}' not in cache, calling retrieveSemanticChildValue`);
    }

    // Call Rust to get the semantic child value (with lazy tile loading)
    // Rust now always returns a WasmPseudoList (empty if no values found)
    const rustValue = await parent.$.wasmWrapper.retrieveSemanticChildValue(
      tile?.tileid || null,
      node.nodeid,
      node.nodegroup_id || null,
      key
    );

    if (key === "geometry" || key === "location_data") {
      console.log(`[SVM.__getChildValue] '${key}' rustValue=`, rustValue, 'length=', rustValue?.length);
    }

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
}

export { SemanticViewModel };
