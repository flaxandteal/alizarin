import {
  IStringKeyedObject,
  IInstanceWrapper,
  IModelWrapper,
  IViewModel,
} from "./interfaces.ts";
import { PseudoValue, PseudoList } from "./pseudos";
import { RDM } from "./rdm";
import {
  StaticDomainValue,
  StaticTile,
  StaticNode,
  StaticValue,
  StaticConcept,
  StaticResourceReference
} from "./static-types";
import { AttrPromise } from "./utils";
import { nodeConfigManager } from './nodeConfig';

const TILE_LOADING_ERRORS = null; // "suppress" or "silence" TODO: enum

const DEFAULT_LANGUAGE = "en";

function tileLoadingError(reason: string, exc: any) {
  if (TILE_LOADING_ERRORS !== "silence") {
    console.error(reason, exc);
    if (TILE_LOADING_ERRORS !== "suppress") {
      throw exc;
    }
  }
}

class ValueList {
  values: Map<string, any>;
  wrapper: IInstanceWrapper;
  tiles: StaticTile[] | null;

  constructor(
    values: Map<string, any>,
    wrapper: IInstanceWrapper,
    tiles: StaticTile[] | null,
  ) {
    this.values = values;
    this.wrapper = wrapper;
    this.tiles = tiles;
  }

  async get(key: string) {
    return this.retrieve(key, this.values.get(key), true);
  }

  set(key: string, value: any) {
    this.values.set(key, value);
  }

  async has(key: string) {
    await this.retrieve(key, null);
    return this.values.has(key);
  }

  async retrieve(key: string, dflt: any = null, raiseError: boolean = false) {
    let result: any = await this.values.get(key);
    if (result === false) {
      if (this.wrapper.resource) {
        // Will KeyError if we do not have it.
        const node = this.wrapper.model.getNodeObjectsByAlias().get(key);
        if (node === undefined) {
          throw Error(
            "Tried to retrieve a node key that does not exist on this resource",
          );
        }
        const values = new Map([...this.values.entries()]);
        const promise = new Promise((resolve) => {
          this.wrapper
            .ensureNodegroup(
              values,
              node,
              node.nodegroup_id,
              this.wrapper.model.getNodeObjects(),
              this.wrapper.model.getNodegroupObjects(),
              this.wrapper.model.getEdges(),
              false,
              this.tiles,
            )
            .then(([ngValues]) => {
              for (const [key, value] of [...ngValues.entries()]) {
                this.values.set(key, value);
              }
              resolve(null);
            });
        });
        this.values.set(key, promise);
        await promise;
      } else {
        this.values.delete(key);
      }
      result = this.values.get(key);
    }
    result = await result;
    if (result === undefined || result === false) {
      if (raiseError) {
        throw Error(`Unset key ${key}`);
      } else {
        return dflt;
      }
    }
    return result;
  }

  async setDefault(key: string, value: any) {
    const newValue = await this.retrieve(key, value, false);
    this.values.set(key, newValue);
    return newValue;
  }
}

class ResourceInstanceViewModel implements IStringKeyedObject {
  [key: string]: any;
  _: IInstanceWrapper;
  __: IModelWrapper;
  id: string;
  then: null = null;

  toString() {
    return `[${this.__.wkrm.modelClassName}:${this.id ?? "-"}]`;
  }

  async forJson(cascade: boolean=false) {
    const jsonData: {[key: string]: string} = {
      type: this.__.wkrm.modelClassName,
      graphId: this.__.wkrm.graphId,
      id: this.id,
    };
    const basic = new StaticResourceReference(jsonData);
    if (cascade) {
      const root = await (await this._.getRoot()).getValue();
      basic.root = await root.forJson();
    }
    return basic;
  }

  constructor(
    id: string,
    modelWrapper: IModelWrapper,
    instanceWrapperFactory: (
      rivm: ResourceInstanceViewModel,
    ) => IInstanceWrapper,
  ) {
    this.id = id;
    this._ = instanceWrapperFactory(this);
    this.__ = modelWrapper;
    return new Proxy(this, {
      set: (object: ResourceInstanceViewModel, key, value) => {
        const k = key.toString();
        if (k in object) {
          object[k] = value;
        } else {
          object._.setOrmAttribute(k, value);
        }
        return true;
      },
      get: (object: ResourceInstanceViewModel, key) => {
        const k = key.toString();
        if (k in object) {
          return object[k];
        }
        return new AttrPromise((resolve) => {
          return object._.getOrmAttribute(k).then((v) => {
            return resolve(v);
          });
        });
      },
    });
  }
}

class ConceptListViewModel extends Array implements IViewModel {
  __parentPseudo: PseudoValue | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(ConceptValueViewModel | null)[]> | null = null;

  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => (v ? v.forJson() : null)) : null;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): Promise<ConceptValueViewModel | null> {
    const nodeid = node.nodeid;
    let val: (ConceptValueViewModel | Promise<ConceptValueViewModel> | null)[];
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, []);
        if (!Array.isArray(value)) {
          throw Error(
            "Cannot set an (entire) concept list value except via an array",
          );
        }
        val = value.map((c) => {
          if (c instanceof ConceptValueViewModel) {
            return c;
          }
          return ConceptValueViewModel.__create(tile, node, c, RDM);
        });
        this._value = Promise.all(val).then((vals) => {
          Promise.all(
            vals.map(async (c) => {
              const v = await c;
              return v ? (await v.getValue()).id : null;
            }),
          ).then((ids) => {
            tile.data.set(nodeid, ids);
            return ids;
          });
        });
      }
    }

    if (!tile || !val) {
      return null;
    }
    const str = new ConceptListViewModel(...val);
    return str;
  }

  async __asTileData() {
    return this._value ? await this._value : null;
  }
}

class DomainValueListViewModel extends Array implements IViewModel {
  __parentPseudo: PseudoValue | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(DomainValueViewModel | null)[]> | null = null;

  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => (v ? v.forJson() : null)) : null;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): Promise<DomainValueViewModel | null> {
    const nodeid = node.nodeid;
    let val: (DomainValueViewModel | Promise<DomainValueViewModel> | null)[];
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, []);
        if (!Array.isArray(value)) {
          throw Error(
            "Cannot set an (entire) domain list value except via an array",
          );
        }
        val = value.map((c) => {
          if (c instanceof DomainValueViewModel) {
            return c;
          }
          return DomainValueViewModel.__create(tile, node, c, RDM);
        });
        this._value = Promise.all(val).then((vals) => {
          tile.data.set(nodeid, vals.map(v => v.id));
          return ids;
        });
      }
    }

    if (!tile || !val) {
      return null;
    }
    const str = new DomainValueListViewModel(...val);
    return str;
  }

  async __asTileData() {
    return this._value ? await this._value : null;
  }
}



class DomainValueViewModel extends String implements IViewModel {
  __parentPseudo: PseudoValue | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: StaticDomainValue | Promise<StaticDomainValue>;

  constructor(value: StaticDomainValue) {
    super(value.toString());
    this._value = value;
  }

  async forJson(): Promise<StaticDomainValue> {
    return this._value;
  }

  getValue(): StaticValue | Promise<StaticValue> {
    return this._value;
  }

  lang(lang: string): string | undefined {
    return this._value.lang(lang);
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): Promise<DomainValueViewModel | null> {
    const nodeid = node.nodeid;
    let val: StaticDomainValue = value;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (!value && !(value instanceof StaticDomainValue)) {
          val = null;
        } else if (value instanceof Promise) {
          return value.then((value) => {
            return DomainValueViewModel.__create(tile, node, value);
          });
        } else if (typeof value == "string") {
          if (
            /^[0-9a-f]{8}-[0-9a-f]{4}-[0-5][0-9a-f]{3}-[089ab][0-9a-f]{3}-[0-9a-f]{12}$/i.exec(
              value,
            )
          ) {
            const config = nodeConfigManager.retrieve(node);
            val = config.valueFromId(value);
          } else {
            throw Error(
              "Set domain values using values from domain lists, not strings",
            );
          }
        } else {
          throw Error("Could not set domain value from this data");
        }

        if (!(val instanceof Promise)) {
          tile.data.set(nodeid, val ? val.id : null);
        }
      }
    }

    if (!tile || !val) {
      return null;
    }
    const str = new DomainValueViewModel(val);
    return str;
  }

  __asTileData() {
    return this._value ? this._value.id : null;
  }
}

class ConceptValueViewModel extends String implements IViewModel {
  __parentPseudo: PseudoValue | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: StaticValue | Promise<StaticValue>;

  constructor(value: StaticValue) {
    super(value.value);
    this._value = value;
  }

  async forJson(): Promise<StaticValue> {
    return this._value;
  }

  getValue(): StaticValue | Promise<StaticValue> {
    return this._value;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): Promise<ConceptValueViewModel | null> {
    const nodeid = node.nodeid;
    let val: StaticValue = value;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (value instanceof StaticConcept) {
          val = value.getPrefLabel();
        }
        if (!value) {
          val = null;
        } else if (value instanceof StaticValue) {
        } else if (value instanceof Promise) {
          return value.then((value) => {
            return ConceptValueViewModel.__create(tile, node, value);
          });
        } else if (typeof value == "string") {
          if (
            /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(
              value,
            )
          ) {
            const collectionId = node.config["rdmCollection"];
            const collection = RDM.retrieveCollection(collectionId);
            return collection.then((collection) => {
              const val = collection.getConceptValue(value);

              if (!val) {
                console.error("Could not find concept for value", value, "for", node.alias, "in collection", collectionId);
              }

              tile.data.set(nodeid, val ? val.id : null);

              if (!tile || !val) {
                return null;
              }
              const str = new ConceptValueViewModel(val);

              return str;
            });
          } else {
            throw Error(
              `Set concepts using values from collections, not strings: ${value}`,
            );
          }
        } else {
          throw Error("Could not set concept from this data");
        }

        if (!(val instanceof Promise)) {
          if (!val) {
            console.error("Could not find concept for value", value, "for", node.alias, "in collection", node.config.get("rdmCollection"));
          }

          tile.data.set(nodeid, val ? val.id : null);
        }
      }
    }

    if (!tile || !val) {
      return null;
    }
    const str = new ConceptValueViewModel(val);
    return str;
  }

  __asTileData() {
    return this._value ? this._value.id : null;
  }
}

class GeoJSONViewModel implements IViewModel, IStringKeyedObject {
  [key: string]: any;
  __parentPseudo: PseudoValue | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: { [key: string]: any };

  constructor(jsonData: { [key: string]: any }) {
    this._value = jsonData;
    return new Proxy(this, {
      get: (object: GeoJSONViewModel, key) => {
        const k = key.toString();
        if (k in object) {
          return object[k];
        }
        return this._value[k];
      },
      set: (object: GeoJSONViewModel, key, value) => {
        const k = key.toString();
        if (k in object) {
          object[k] = value;
        } else {
          this._value[k] = value;
        }
        return true;
      },
    });
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): GeoJSONViewModel | Promise<GeoJSONViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) =>
        GeoJSONViewModel.__create(tile, node, value),
      );
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }

    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === undefined) {
      return null;
    }
    if (!(val instanceof Object)) {
      throw Error("GeoJSON should be a JSON object");
    }
    const str = new GeoJSONViewModel(val);
    return str;
  }

  async forJson() {
    return await this._value;
  }

  __asTileData() {
    return this._value;
  }
}

class StringViewModel extends String implements IViewModel {
  __parentPseudo: PseudoValue | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: Map<string, object>;

  constructor(value: Map<string, object>, language: string | null = null) {
    let displayValue = value.get(language || DEFAULT_LANGUAGE) || {
      value: "",
    };
    if (displayValue instanceof Object) {
      displayValue = displayValue.value;
    }
    super(displayValue);
    this._value = value;
  }

  forJson() {
    return `${this}`;
  }

  lang(language: string) {
    const elt = this._value.get(language);
    if (elt) {
      if (elt instanceof Object) {
        return elt.value;
      }
      return elt;
    } else {
      return undefined;
    }
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): StringViewModel | Promise<StringViewModel> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) => StringViewModel.__create(tile, node, value));
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, {});
      }
      if (value !== null) {
        if (value instanceof Object) {
          const entries =
            value instanceof Map ? value.entries() : Object.entries(value);
          for (const [k, v] of [...entries]) {
            const val = tile.data.get(nodeid);
            if (val instanceof Map) {
              val.set(k, v);
            } else if (val instanceof Object) {
              val[k] = v;
            } else if (val !== null) {
              throw Error("Malformed string in tile data");
            }
          }
        } else {
          tile.data.set(nodeid, {
            [DEFAULT_LANGUAGE]: value,
          });
        }
      }
    }

    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === undefined) {
      return null;
    }
    let mapVal;
    if (val instanceof Map) {
      mapVal = val;
    } else {
      mapVal = new Map(Object.entries(val));
    }
    const str = new StringViewModel(mapVal);
    return str;
  }

  __asTileData() {
    return this._value;
  }
}

class SemanticViewModel implements IStringKeyedObject, IViewModel {
  [key: string]: any;
  then: undefined;

  __parentPseudo: PseudoValue | undefined;
  __childValues: Map<string, any>;
  __parentWkri: ResourceInstanceViewModel | null;
  __childNodes: Map<string, StaticNode>;
  __tile: StaticTile | null;
  __node: StaticNode;

  constructor(
    parentWkri: ResourceInstanceViewModel | null,
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
        const k: string = key.toString();
        if (k.startsWith("__")) {
          object[k] = value;
        } else {
          object.__set(k, value);
        }
        return true;
      },
      get: (object, key) => {
        const k: string = key.toString();
        if (k.startsWith("__") || k in object) {
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
    const entries = this.__childValues.entries().map(([k, v]) => `${k}: ${v}`);
    return `[[${entries.join(",")}]]`;
  }

  async toObject() {
    const entries = [...(await this.__getChildValues()).entries()];
    return Object.fromEntries(await Promise.all(entries.map(async ([k, vl]) => {
        return [k, (await vl).getValue()];
    })));
  }

  async forJson() {
    async function _forJson(v: IViewModel | null) {
      v = await v;
      if (!v) {
        return null;
      }
      if (v && v instanceof PseudoValue) {
        v = await v.getValue();
      }
      if (v && v instanceof Object && v.forJson) {
        return await v.forJson();
      }
      return v;
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
    if (!this.__childNodes.has(key)) {
      throw Error(
        `Semantic node does not have this key: ${key} (${[...this.__childNodes.keys()]})`,
      );
    }

    if (!this.__childValues.has(key)) {
      let child = await this.__getChildValues(key);
      if (child === null) {
        child = this.__makePseudo(key);
      }
      this.__childValues.set(key, child);
      child.parentNode = this.__parentPseudo;
    }
    this.__childValues.get(key).value = value;
  }

  async __getChildTypes() {
    const promises = [...this.__childNodes.keys()].map(async (key) => [
      key,
      await this.__getChildValue(key),
    ]);
    const entries = await Promise.all(promises);
    return new Map<string, any>(...entries);
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

  async __getChildValue(key: string) {
    if (!this.__childNodes.has(key)) {
      throw Error(
        `Semantic node does not have this key: ${key} (${[...this.__childNodes.keys()]})`,
      );
    }

    let child;
    if (!this.__childValues.has(key)) {
      child = await this.__getChildValues(key);
      if (child === null) {
        child = this.__makePseudo(key);
      } else {
        // This ensures that we do not set a default value in our
        // local cache simply because the node is not loaded yet.
        this.__childValues.set(key, child);
      }
      child.parentNode = this.__parentPseudo;
    } else {
      child = this.__childValues.get(key);
    }
    return child;
  }

  __makePseudo(key: string) {
    const childNode = this.__childNodes.get(key);

    if (!childNode) {
      throw Error(`Child node key ${key} missing`);
    }

    if (!this.__parentWkri) {
      throw Error("This semantic node is currently parentless (no WKRI)");
    }

    const child = this.__parentWkri._.model.makePseudoCls(
      key,
      false,
      !childNode.is_collector ? this.__tile : null, // Does it share a tile
      this.__parentWkri,
    );

    child.parentNode = this.__parentPseudo;
    if (this.__parentWkri) {
      const valueList: ValueList<any> = this.__parentWkri._.valueList;
      valueList.setDefault(key, []).then((val: Array<any>) => val.push(child));
    }
    return child;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    parent: ResourceInstanceViewModel | null,
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
    await svm.__getChildren();
    return svm;
  }

  async __asTileData() {
    // Ensure all nodes have populated the tile
    const relationships: any[] = [];
    for (const value of this.__getChildren(true)) {
      // We do not use tile, because a child node will ignore its tile reference.
      const [_, subrelationships] = await value.getTile();
      relationships.push(...subrelationships);
    }
    // This is none because the semantic type has no nodal value,
    // only its children have nodal values, and the nodal value of this nodeid should
    // not exist.
    return [null, relationships];
  }

  async __getChildValues(targetKey: string | null = null) {
    const parent = this.__parentWkri;
    const childNodes = this.__childNodes;
    const tile = this.__tile;
    const node = this.__node;
    if (!parent) {
      return targetKey === null ? {} : null;
    }

    // Ensure lazy-loading done.
    // TODO check this does not go deeper than necessary.
    for (const key of childNodes.keys()) {
      await parent._.valueList.retrieve(key);
    }

    const children: Map<string, any> = new Map();
    for (let [key, values] of [...parent._.valueList.values.entries()]) {
      // if (this.__parentPseudo && this.__parentPseudo.node.alias == "localities_administrative_areas") {
      //   console.log("M", key, values);
      // }
      // if (this.__node && this.__node.alias == "localities_administrative_areas" || this.__node.alias == "area_names") {
      //   console.log(items, 'CHILDREN', this.__node.alias);
      // }
      if (values instanceof Promise) {
        values = await values;
      }
      if (values === false || values === null || values === undefined) {
        continue;
      }
      const childNode = childNodes.get(key);
      for (const value of values) {
        if (
          childNode &&
          value !== null &&
          (!(value.parentNode) ||
            value.parentNode === this.__parentPseudo)
        ) {
          if (
            (tile && value.parenttile_id == tile.tileid) ||
            (value.node.nodegroup_id == node.nodegroup_id &&
              tile &&
              value.tile == tile &&
              !childNode.is_collector) //  # It shares a tile
          ) {
            children.set(key, value);
          } else if (
            node.nodegroup_id != value.node.nodegroup_id &&
            childNode.is_collector // It does not share a tile
          ) {
            // This avoids list types that have their own tiles (like resource or concept lists)
            // from appearing doubly-nested
            if (
              value instanceof PseudoList ||
              (value.value && Array.isArray(value.value))
            ) {
              if (children.has(key)) {
                children.get(key).push(...value);
              } else {
                children.set(key, value);
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
    for (const [key, value] of [...children.entries()]) {
      value.parentNode = this.__parentPseudo;
      this.__childValues.set(key, value);
    }

    if (targetKey !== null) {
      return children.get(targetKey) || null;
    }
    return children;
  }
}

async function getViewModel(
  parentPseudo: PseudoValue,
  tile: StaticTile,
  node: StaticNode,
  data: any,
  parent: ResourceInstanceViewModel | null,
  childNodes: Map<string, StaticNode>,
): Promise<[IViewModel, Function | null, string, boolean]> {
  let vm;
  switch (node.datatype) {
    case "semantic":
      vm = await SemanticViewModel.__create(
        tile,
        node,
        data,
        parent,
        childNodes,
      );
      break;
    case "domain-value":
      vm = await DomainValueViewModel.__create(tile, node, data);
      break;
    case "domain-value-list":
      vm = await DomainValueListViewModel.__create(tile, node, data);
      break;
    case "concept":
      vm = await ConceptValueViewModel.__create(tile, node, data);
      break;
    case "concept-list":
      vm = await ConceptListViewModel.__create(tile, node, data);
      break;
    case "geojson-feature-collection":
      vm = await GeoJSONViewModel.__create(tile, node, data);
      break;
    case "string":
    default:
      vm = await StringViewModel.__create(tile, node, data);
  }
  // const vm = StringViewModel.__create(tile, node, data, parent, parentCls, childNodes);
  let asTileData: Function | null = null;
  if (vm) {
    vm.__parentPseudo = parentPseudo;
    if (vm instanceof Array) {
      for (const vme of vm) {
        if (vme instanceof Promise) {
          vme.then(vmep => { vmep.__parentPseudo = parentPseudo; });
        } else {
          vme.__parentPseudo = parentPseudo;
        }
      }
    }
    asTileData = vm.__asTileData.bind(vm);
  }

  return [vm, asTileData, "string", false];
}

export { ResourceInstanceViewModel, ValueList, getViewModel, DomainValueViewModel, SemanticViewModel, StringViewModel, GeoJSONViewModel, ConceptValueViewModel };
