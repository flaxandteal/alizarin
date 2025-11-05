import {
  IStringKeyedObject,
  IViewModel,
  IPseudo,
  IRIVM,
} from "./interfaces.ts";
import { AttrPromise } from "./utils";
import { PseudoValue, PseudoList } from "./pseudos";
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
  __childNodes: Map<string, StaticNode>;
  __tile: StaticTile | null;
  __node: StaticNode;

  __forJsonCache(): null {
    return null;
  }

  constructor(
    parentWkri: IRIVM<any> | null,
    childNodes: Map<string, StaticNode>,
    tile: StaticTile | null,
    node: StaticNode,
  ) {
    this.__childValues = new Map<string, any>();
    this.__parentWkri = parentWkri;
    this.__tile = tile;
    this.__node = node;
    this.__childNodes = childNodes;
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
          return object.__childNodes.size;
        }
        return new AttrPromise((resolve) => {
          object.__get(k).then(resolve);
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
    return this.__childNodes.has(key);
  }

  async __getChildTypes() {
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
    if (!this.__childNodes.has(key)) {
      throw Error(
        `Semantic node does not have this key: ${key} (${[...this.__childNodes.keys()]})`,
      );
    }

    let child;
    if (!this.__childValues.has(key)) {
      const children = await this.__getChildValues();
      child = children.get(key) || null;

      let set = true;
      if (child === null) {
        child = this.__makePseudo(key);
        set = setDefault;
      }
      if (set) {
        // This ensures that we do not set a default value in our
        // local cache simply because the node is not loaded yet.
        this.__childValues.set(key, child);
      }
      child.parentNode = this.__parentPseudo || null;
    } else {
      child = this.__childValues.get(key);
    }
    return child;
  }

  __makePseudo(key: string): IPseudo {
    const childNode = this.__childNodes.get(key);

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

    const child = this.__parentWkri.$.addPseudo(childNode, this.__tile, this.__node);
    child.parentNode = this.__parentPseudo || null;
    return child;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    parent: IRIVM<any> | null,
    childNodes: Map<string, StaticNode>,
  ): Promise<SemanticViewModel> {
    const svm = new SemanticViewModel(parent, childNodes, tile, node);
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
    const childNodes = this.__childNodes;
    const tile = this.__tile;
    const node = this.__node;
    if (!parent || !parent.$) {
      return new Map();
    }

    // Ensure lazy-loading done.
    // TODO check this does not go deeper than necessary.
    await parent.$.loadNodes([...childNodes.keys()]);

    // TODO: Why not just use edges?
    const children: Map<string, any> = new Map();
    for (const entry of [...parent.$.allEntries()]) {
      const key = entry[0];
      let values = entry[1];
      if (values instanceof Promise) {
        values = await values;
      }
      if (values === false || values === null || values === undefined) {
        continue;
      }
      const childNode = childNodes.get(key);
      if (childNode) {
        for (let value of values) {
          if (
            value !== null &&
            value.node &&
            (!(value.parentNode) ||
              value.parentNode === this.__parentPseudo)
          ) {
            // It is possible that this value has already
            // been requested, but the tile is in-flight.
            value = await value;
            if (!value.node) {
              throw Error(`Node ${childNode.alias} (${childNode.nodeid}) is unavailable`);
            }
            if (
              // value.node.nodegroup_id == node.nodeid in all cases for first possibility?
              (value.node.nodegroup_id != node.nodegroup_id && tile && value.tile && (!(value.tile.parenttile_id) || value.tile.parenttile_id == tile.tileid)) ||
              (value.node.nodegroup_id == node.nodegroup_id &&
                tile &&
                value.tile == tile &&
                !childNode.is_collector) //  # It shares a tile
              // it feels like this should be necessary, but area_assignments->area_assignment fails with null parenttile_id
              // (tile && value.tile && value.tile.parenttile_id == tile.tileid) ||
              // (value.node.nodegroup_id == node.nodegroup_id &&
              //   tile &&
              //   value.tile == tile &&
              //   !childNode.is_collector) //  # It shares a tile
            ) {
              children.set(key, value);
            } else if (
              node.nodegroup_id != value.node.nodegroup_id &&
              childNode.is_collector // It does not share a tile
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
                if (children.has(key)) {
                  children.get(key).push(...listValue);
                } else {
                  children.set(key, listValue);
                }
              } else {
                // In this case, we have a value, but the wrapper logic did not make it a PseudoList, so
                // we should treat it as singular.
                children.set(key, value);
              }
            }
          }
        }
      }
    }
    for (const [key, value] of [...children.entries()]) {
      value.parentNode = this.__parentPseudo;
      this.__childValues.set(key, value);
    }

    return children;
  }
}

export { SemanticViewModel };
