import {
  IStringKeyedObject,
  IInstanceWrapper,
  IModelWrapper,
  IViewModel,
  IPseudo,
  IGraphManager,
  IRIVM,
  GetMeta,
} from "./interfaces.ts";
import { PseudoValue, PseudoList } from "./pseudos";
import { RDM } from "./rdm";
import {
  StaticNodeConfigDomain,
  StaticNodeConfigBoolean,
} from './nodeConfig';
import {
  StaticDomainValue,
  StaticTile,
  StaticNode,
  StaticValue,
  StaticConcept,
  StaticResource,
  StaticResourceReference
} from "./static-types";
import { AttrPromise } from "./utils";
import { nodeConfigManager } from './nodeConfig';

const TILE_LOADING_ERRORS = null; // "suppress" or "silence" TODO: enum

const DEFAULT_LANGUAGE = "en";

class ViewContext {
  graphManager: IGraphManager | undefined
};
const viewContext = new ViewContext();

function tileLoadingError(reason: string, exc: any) {
  if (TILE_LOADING_ERRORS !== "silence") {
    console.error(reason, exc);
    if (TILE_LOADING_ERRORS !== "suppress") {
      throw exc;
    }
  }
}

class ValueList<T extends IRIVM<T>> {
  values: Map<string, any>;
  wrapper: IInstanceWrapper<T>;
  tiles: StaticTile[] | null;
  promises: Map<string | null, Promise<boolean | IViewModel>>;
  writeLock: null | Promise<boolean | IViewModel>;

  constructor(
    values: Map<string, any>,
    wrapper: IInstanceWrapper<T>,
    tiles: StaticTile[] | null,
  ) {
    this.values = values;
    this.wrapper = wrapper;
    this.tiles = tiles;
    this.promises = new Map();
    this.writeLock = null;
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
    // console.trace();
    const node = this.wrapper.model.getNodeObjectsByAlias().get(key);
    const promise = node ? this.promises.get(node.nodegroup_id) : null;
    // When an unloaded node is found, the whole nodegroup is loaded.
    // The promises member ensures that no other node in the nodegroup
    // triggers the same nodegroup load. Note that there is _also_ the
    // individual node promise, which allows any operation to wait for
    // just that node to finish and resolve in to the approach pseudo
    // even if `retrieve` is not used (e.g. allEntries).
    // _However_, this means that allEntries will not see a promise for
    // individual nodes in the nodegroup that are _not_ the first
    // requested, until the first requested resolves and updates the
    // values map for all nodes in the nodegroup.
    if (promise) {
      await promise;
    }
    let result: any = await this.values.get(key);

    if (result === false) {
      await this.writeLock;
      if (this.wrapper.resource) {
        // Will KeyError if we do not have it.
        const node = this.wrapper.model.getNodeObjectsByAlias().get(key);
        if (node === undefined) {
          throw Error(
            "Tried to retrieve a node key that does not exist on this resource",
          );
        }
        const values = new Map([...this.values.entries()]);
        const promise: Promise<IViewModel | boolean> = new Promise((resolve) => {
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
              true
            )
            .then(([ngValues]) => {
              let original = false;
              for (const [k, value] of [...ngValues.entries()]) {
                if (key === k) {
                  // Other methods may be waiting on this specific
                  // value to resolve.
                  original = value;
                }
                this.values.set(k, value);
              }
              resolve(original);
              this.promises.delete(node.nodegroup_id);
            });
        });
        // No writes should happen until this is done
        this.writeLock = promise;
        // No reads from this nodegroup should happen
        this.promises.set(node.nodegroup_id, promise);
        // Other readers are welcome to wait for this nodegroup's read
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

class ConceptListCacheEntry implements IStringKeyedObject {
  [key: string]: any;
  datatype: string = 'concept-list';
  _: ConceptValueCacheEntry[];
  meta: {[key: string]: any};

  constructor({meta, _}: {meta: IStringKeyedObject | undefined, _: ConceptValueCacheEntry[]}) {
    this._ = _.map(instance => {
      if (instance instanceof ConceptValueCacheEntry) {
        return instance;
      }
      return new ConceptValueCacheEntry(instance);
    });
    this.meta = meta || {};
  }
}

class ConceptValueCacheEntry implements IStringKeyedObject {
  [key: string]: any
  datatype: string = 'concept';
  id: string;
  value: string;
  conceptId: string | null;
  meta: {[key: string]: any};

  constructor({meta, id, value, conceptId}: {meta: IStringKeyedObject | undefined, id: string, value: string, conceptId: string | null}) {
    this.id = id;
    this.value = value;
    this.conceptId = conceptId;
    this.meta = meta || {};
  }
}

class ResourceInstanceListCacheEntry implements IStringKeyedObject {
  [key: string]: any;
  datatype: string = 'resource-instance-list';
  _: ResourceInstanceCacheEntry[];
  meta: {[key: string]: any};

  constructor({meta, _}: {meta: IStringKeyedObject | undefined, _: ResourceInstanceCacheEntry[]}) {
    this._ = _.map(instance => {
      if (instance instanceof ResourceInstanceCacheEntry) {
        return instance;
      }
      return new ResourceInstanceCacheEntry(instance);
    });
    this.meta = meta || {};
  }
}

class ResourceInstanceCacheEntry implements IStringKeyedObject {
  [key: string]: any
  datatype: string = 'resource-instance';
  id: string;
  type: string;
  graphId: string;
  title: string | null;
  meta: {[key: string]: any};

  constructor({meta, id, type, graphId, title}: {meta: IStringKeyedObject | undefined, id: string, type: string, graphId: string, title: string | null}) {
    this.id = id;
    this.type = type;
    this.graphId = graphId;
    this.meta = meta || {};
    this.title = this.meta.title || title;
  }
}

class ResourceInstanceListViewModel extends Array implements IViewModel {
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(ResourceInstanceViewModel<any> | null)[]> | null = null;

  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => (v ? v.forJson() : null)) : null;
  }

  async __forJsonCache(getMeta: GetMeta): Promise<ResourceInstanceListCacheEntry> {
    return new ResourceInstanceListCacheEntry({
      meta: getMeta ? await getMeta(this) : getMeta,
      _: await Promise.all([...this.values()].map(async (rivmPromise: Promise<ResourceInstanceViewModel<any>>) => {
        const rivm = await rivmPromise;
        return await rivm.__forJsonCache(getMeta)
      }))
    });
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    cacheEntry: ResourceInstanceListCacheEntry | null = null
  ): Promise<ResourceInstanceListViewModel | null> {
    const nodeid = node.nodeid;
    let val: (ResourceInstanceViewModel<any> | null | Promise<ResourceInstanceViewModel<any> | null>)[];
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, []);
        if (!Array.isArray(value)) {
          throw Error(
            "Cannot set an (entire) resource list value except via an array",
          );
        }
        val = value.map((v, i) => {
          if (v instanceof ResourceInstanceViewModel) {
            return v;
          }
          return ResourceInstanceViewModel.__create(tile, node, v, cacheEntry && cacheEntry._[i] ? cacheEntry._[i] : null);
        });
        Promise.all(
          val.map(async (c) => {
            const v = await c;
            return v ? (await v).id : null;
          }),
        ).then((ids) => {
          tile.data.set(nodeid, ids);
          return ids;
        });
        value = val;
      }
    } else {
      value = [];
    }

    if (!tile || !value) {
      return null;
    }
    const str = new ResourceInstanceListViewModel(...value);
    return str;
  }

  async __asTileData() {
    return this._value ? await this._value : null;
  }
}

class ResourceInstanceViewModel<RIVM extends IRIVM<RIVM>> implements IStringKeyedObject {
  [key: string | symbol]: any;
  _: IInstanceWrapper<RIVM> | null;
  __: IModelWrapper<RIVM> | null;
  __parentPseudo: IPseudo | undefined = undefined;
  __cacheEntry: ResourceInstanceCacheEntry | null = null;
  id: string;
  then: undefined;
  [Symbol.toPrimitive]: undefined;

  gm: IGraphManager | undefined;

  toString(): string {
    if (!this.__) {
      return `[Resource:${this.id}]`;
    }
    return `[${this.__.wkrm.modelClassName}:${this.id ?? "-"}]`;
  }

  async __has(key: string): Promise<boolean | undefined> {
    // There is a catch here, that because we lazy-load, we do not
    // know, hence three possible return values.
    if (!this._) {
      return undefined;
    }
    return (await this._.getRootViewModel() || new Map()).__has(key);
  }

  async __asTileData(): Promise<IStringKeyedObject> {
    return {
      resourceId: this.id
    };
  }

  async __forJsonCache(getMeta: GetMeta): Promise<ResourceInstanceCacheEntry> {
    // TODO should we re-use the cacheEntry and rely on it being expired correctly?
    let wrapper: IModelWrapper<RIVM>;
    if (!this.__) {
      if (this.__cacheEntry) {
        return this.__cacheEntry;
      } else {
        [, wrapper] = await this.retrieve();
      }
    } else {
      wrapper = this.__;
    }
    this.__cacheEntry = new ResourceInstanceCacheEntry({
      meta: getMeta ? await getMeta(this) : undefined,
      id: this.id,
      type: wrapper.wkrm.modelClassName,
      graphId: wrapper.wkrm.graphId,
      title: null,
    });
    return this.__cacheEntry;
  }

  async forJson(cascade: boolean=false) {
    let jsonData: StaticResourceReference;
    if (!cascade && this.__cacheEntry) {
      jsonData = {
        type: this.__cacheEntry.type,
        graphId: this.__cacheEntry.graphId,
        id: this.__cacheEntry.id,
        title: this.__cacheEntry.title || undefined,
        meta: this.__cacheEntry.meta || undefined,
        root: null
      };
    } else if (this.__) {
      jsonData = {
        type: this.__.wkrm.modelClassName,
        graphId: this.__.wkrm.graphId,
        id: this.id,
        title: undefined,
        meta: undefined,
        root: null
      };
    } else {
      jsonData = {
        type: "(unknown)",
        graphId: "",
        id: this.id,
        title: undefined,
        meta: undefined,
        root: null
      };
    }
    const basic = new StaticResourceReference(jsonData);
    if (cascade) {
      if (!this._) {
        await this.retrieve();
        if (!this._) {
          throw Error("Could not retrieve resource");
        }
      }
      const root = await this._.getRootViewModel();
      basic.root = await root.forJson();
    }
    return basic;
  }

  async retrieve(): Promise<[IInstanceWrapper<RIVM>, IModelWrapper<RIVM>]> {
    let iw: IInstanceWrapper<RIVM>;
    let mw: IModelWrapper<RIVM>;
    if (viewContext.graphManager) {
      const replacement = await viewContext.graphManager.getResource(this.id, true);

      // @ts-expect-error We cannot guarantee this resource is the right type...
      iw = replacement._;
      // @ts-expect-error We cannot guarantee this resource is the right type...
      mw = replacement.__;
    } else {
      throw Error("Cannot traverse resource relationships without a GraphManager");
    }
    this._ = iw;
    this.__ = mw;
    return [iw, mw];
  }

  constructor(
    id: string,
    modelWrapper: IModelWrapper<RIVM> | null,
    instanceWrapperFactory: ((
      rivm: RIVM,
    ) => IInstanceWrapper<RIVM>) | null,
    cacheEntry: object | null,
  ) {
    this.id = id;
    // @ts-expect-error I believe some deep type magic would be required to
    // convince TS that `this` is a valid RIVM.
    this._ = instanceWrapperFactory ? instanceWrapperFactory(this) : null;
    this.__ = modelWrapper;
    if (cacheEntry instanceof ResourceInstanceCacheEntry) {
      this.__cacheEntry = cacheEntry;
    }

    return new Proxy(this, {
      // NOTE: set should not return a promise, so could cause a race
      // condition with a subsequent read.
      // @ts-expect-error Returning a promise for set
      set: async (object: ResourceInstanceViewModel<RIVM>, key, value): Promise<boolean> => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;
        if (key in object) {
          object[key] = value;
        } else if (k in object) {
          object[k] = value;
        } else {
          if (!object._) {
            await this.retrieve();
            if (!object._) {
              throw Error("Could not retrieve resource");
            }
          }
          object._.setOrmAttribute(k, value);
        }
        return true;
      },
      get: (object: ResourceInstanceViewModel<RIVM>, key) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;
        if (key in object) {
          return object[key];
        } else if (k in object) {
          return object[k];
        }
        return new AttrPromise(async (resolve) => {
          if (!object._) {
            await this.retrieve();
            if (!object._) {
              throw Error("Could not retrieve resource");
            }
          }
          return object._.getOrmAttribute(k).then((v) => {
            return resolve(v);
          });
        });
      },
    });
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    cacheEntry: ResourceInstanceCacheEntry | null
  ): Promise<ResourceInstanceViewModel<any> | null> {
    const nodeid = node.nodeid;
    let val: string | null = value;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (!value && !(value instanceof StaticResource) && !(value instanceof StaticResourceReference)) {
          val = null;
        } else if (value instanceof Promise) {
          return value.then((value) => {
            return ResourceInstanceViewModel.__create(tile, node, value, cacheEntry);
          });
        } else if (typeof value == "string") {
          if (
            /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(
              value,
            )
          ) {
            val = value;
          } else {
            throw Error(
              `Set resource instances using id, not strings in node ${node.alias}: ${value}`,
            );
          }
        } else if (value instanceof Object && value.resourceId) {
          val = value.resourceId;
        } else if (value instanceof Array && value.length < 2 ) {
          if (value.length == 1) {
            return ResourceInstanceViewModel.__create(tile, node, value[0], cacheEntry);
          }
        } else {
          throw Error("Could not set resource instance from this data");
        }

        tile.data.set(nodeid, val ? val : null);
      }
    }

    if (!tile || !val) {
      return null;
    }
    const str = new ResourceInstanceViewModel(val, null, null, cacheEntry);
    return str;
  }
}

class ConceptListViewModel extends Array implements IViewModel {
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(ConceptValueViewModel | null)[]> | null = null;

  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => (v ? v.forJson() : null)) : null;
  }

  async __forJsonCache(getMeta: GetMeta): Promise<ConceptListCacheEntry> {
    return new ConceptListCacheEntry({
      meta: getMeta ? await getMeta(this) : getMeta,
      _: await Promise.all([...this.values()].map(async (rivmPromise: Promise<ConceptValueViewModel>) => {
        const rivm = await rivmPromise;
        return await rivm.__forJsonCache(getMeta)
      }))
    });
  }
  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    cacheEntry: ConceptListCacheEntry | null = null
  ): Promise<ConceptListViewModel> {
    const nodeid = node.nodeid;
    let val: (ConceptValueViewModel | Promise<ConceptValueViewModel | null> | null)[] = [];
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
      val = value.map((c, i) => {
        if (c instanceof ConceptValueViewModel) {
          return c;
        }
        return ConceptValueViewModel.__create(tile, node, c, cacheEntry && cacheEntry._[i] ? cacheEntry._[i] : null);
      });
      Promise.all(val).then((vals) => {
        Promise.all(
          vals.map(async (c) => {
            const v = await c;
            return v ? (await v.getValue()).id : null;
          })
        ).then((ids) => {
          tile.data.set(nodeid, ids);
        });
      });
      value = val;
    } else {
      value = [];
    }

    const str = new ConceptListViewModel(...value);
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

  // No point in caching something that is on the graph.
  __forJsonCache(): null {
    return null;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): Promise<DomainValueListViewModel> {
    const nodeid = node.nodeid;
    let val: (DomainValueViewModel | Promise<DomainValueViewModel | null> | null)[];
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
          return DomainValueViewModel.__create(tile, node, c);
        });
        Promise.all(val).then(async (vals) => {
          const ids = Promise.all(vals.map(async val => val === null ? val : (await val._value).id));
          ids.then(ids => {
            tile.data.set(nodeid, ids);
          });
        });
      }
    } else {
      value = [];
    }

    const str = new DomainValueListViewModel(...value);
    return str;
  }

  async __asTileData() {
    const value = await this._value;
    return value ?? null;
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

  // No point in caching something that is on the graph.
  __forJsonCache(): null {
    return null;
  }

  getValue(): StaticDomainValue | Promise<StaticDomainValue> {
    return this._value;
  }

  async lang(lang: string): Promise<string | undefined> {
    return (await this._value).lang(lang);
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): Promise<DomainValueViewModel | null> {
    const nodeid = node.nodeid;
    let val: StaticDomainValue | null = value;
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
            if (!config || !(config instanceof StaticNodeConfigDomain)) {
              throw Error(`Cannot form domain value for ${node.nodeid} without config`);
            }
            val = config.valueFromId(value) || null;
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

  async __asTileData() {
    const value = await this._value;
    return value ? value.id : null;
  }
}

class ConceptValueViewModel extends String implements IViewModel {
  __parentPseudo: IPseudo | undefined;

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

  async __forJsonCache(getMeta: GetMeta): Promise<ConceptValueCacheEntry> {
    const value = await this._value;
    return new ConceptValueCacheEntry({
      meta: getMeta ? await getMeta(this) : undefined,
      id: value.id,
      value: value.value,
      conceptId: value.__conceptId
    });
  }

  getValue(): StaticValue | Promise<StaticValue> {
    return this._value;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    cacheEntry: ConceptValueCacheEntry | null
  ): Promise<ConceptValueViewModel | null> {
    const nodeid = node.nodeid;
    let val: StaticValue | null = value;
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
          // No change needed.
        } else if (value instanceof Promise) {
          return value.then((value) => {
            return ConceptValueViewModel.__create(tile, node, value, cacheEntry);
          });
        } else if (typeof value == "string") {
          if (
            /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(
              value,
            )
          ) {
            if (cacheEntry) {
              val = new StaticValue({
                id: cacheEntry.id,
                value: cacheEntry.value,
                __concept: null,
                __conceptId: cacheEntry.conceptId,
              }, cacheEntry.conceptId);
              return new ConceptValueViewModel(val);
            } else {
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
            }
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

  async __asTileData() {
    const value = await this._value;
    return value ? value.id : null;
  }
}

class DateViewModel extends Date implements IViewModel {
  __parentPseudo: PseudoValue | undefined;
  __original: string;
  then: undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  __forJsonCache(): null {
    return null;
  }

  constructor(val: string) {
    super(val);
    this.__original = val;
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): DateViewModel | Promise<DateViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) =>
        DateViewModel.__create(tile, node, value),
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

    let val: string | {[key: string]: string} | any = tile.data.get(nodeid);
    // TODO: catch rendering issues - this workaround should be removed
    // as it is overly tolerant of input issues.
    if (typeof val == "object" && val['en'] !== undefined) {
      val = val.en;
    }
    if (!tile || val === null || val === undefined || val === '') {
      return null;
    }
    if (typeof val != "string") {
      throw Error("Date should be a string");
    }
    const str = new DateViewModel(val);
    return str;
  }

  async forJson() {
    try {
      return this.toISOString();
    } catch (e) {
      console.warn(e);
      return this.__original;
    }
  }

  __asTileData() {
    return this.toISOString();
  }
}

class GeoJSONViewModel implements IViewModel, IStringKeyedObject {
  [key: string | symbol]: any;
  __parentPseudo: PseudoValue | undefined;
  then: undefined;
  [Symbol.toPrimitive]: undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: { [key: string]: any };

  __forJsonCache(): null {
    return null;
  }

  constructor(jsonData: { [key: string]: any }) {
    this._value = jsonData;
    return new Proxy(this, {
      get: (object: GeoJSONViewModel, key) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;
        if (key in object) {
          return object[key];
        } else if (k in object) {
          return object[k];
        }
        return this._value[k];
      },
      set: (object: GeoJSONViewModel, key, value) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;
        if (key in object) {
          object[key] = value;
        } else if (k in object) {
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

class StringTranslatedLanguage {
  value: string = ""
}

class NonLocalizedStringViewModel extends String implements IViewModel {
  __parentPseudo: PseudoValue | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  __forJsonCache(): null {
    return null;
  }

  forJson(): string {
    return this.toString();
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): NonLocalizedStringViewModel | Promise<NonLocalizedStringViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) => NonLocalizedStringViewModel.__create(tile, node, value));
    }
    if (tile) {
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }

    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === undefined) {
      return null;
    }
    const string = new NonLocalizedStringViewModel(val);
    return string;
  }

  __asTileData() {
    return this ? true : false;
  }
}

class NumberViewModel extends Number implements IViewModel {
  __parentPseudo: PseudoValue | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  toString(): string {
    return `${this.valueOf()}`;
  }

  __forJsonCache(): null {
    return null;
  }

  forJson(): number {
    return this.valueOf();
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): NumberViewModel | Promise<NumberViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) => NumberViewModel.__create(tile, node, value));
    }
    if (tile) {
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }

    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === undefined) {
      return null;
    }
    const num = new NumberViewModel(val);
    return num;
  }

  __asTileData() {
    return this ? true : false;
  }
}

class UrlViewModel extends String implements IViewModel {
  __parentPseudo: PseudoValue | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  __forJsonCache(): null {
    return null;
  }

  forJson(): boolean {
    return this ? true : false;
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): UrlViewModel | Promise<UrlViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) => UrlViewModel.__create(tile, node, value));
    }
    if (tile) {
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }

    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === undefined) {
      return null;
    }
    const url = new UrlViewModel(val);
    return url;
  }

  __asTileData() {
    return this ? true : false;
  }
}

class BooleanViewModel extends Boolean implements IViewModel {
  __parentPseudo: PseudoValue | undefined;
  __config:  StaticNodeConfigBoolean;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  constructor(value: boolean, config: StaticNodeConfigBoolean) {
    super(value);
    this.__config = config;
  }

  toString(lang?: string | undefined): string {
    const labelLang = lang || DEFAULT_LANGUAGE;
    const isTrue: Boolean = this;
    return isTrue ? (
      this.__config && this.__config.trueLabel ? this.__config.trueLabel[labelLang] || 'true' : 'true'
    ) : (
      this.__config && this.__config.trueLabel ? this.__config.falseLabel[labelLang] || 'false' : 'false'
    );
  }

  __forJsonCache(): null {
    return null;
  }

  forJson(): boolean {
    return this ? true : false;
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): BooleanViewModel | Promise<BooleanViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) => BooleanViewModel.__create(tile, node, value));
    }
    if (tile) {
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }

    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === undefined) {
      return null;
    }
    const config = nodeConfigManager.retrieve(node);
    if (!config || !(config instanceof StaticNodeConfigBoolean)) {
      throw Error(`Cannot form boolean value for ${node.nodeid} without config`);
    }
    const bool = new BooleanViewModel(val, config);
    return bool;
  }

  __asTileData() {
    return this ? true : false;
  }
}

class StringViewModel extends String implements IViewModel {
  __parentPseudo: PseudoValue | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: Map<string, StringTranslatedLanguage>;

  __forJsonCache(): null {
    return null;
  }

  constructor(value: Map<string, StringTranslatedLanguage>, language: string | null = null) {
    const lang = value.get(language || DEFAULT_LANGUAGE);
    let displayValue: string;
    if (lang) {
      if (typeof lang == "string") {
        displayValue = lang;
      } else {
        displayValue = lang.value;
      }
    } else {
      // TODO: allow fallback
      displayValue = "";
    }
    super(displayValue);
    this._value = value;
  }

  forJson(): string {
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
  ): StringViewModel | Promise<StringViewModel | null> | null {
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
              // @ts-expect-error Need better typing of data to make this settable.
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
  [key: string | symbol]: any;
  then: undefined;
  [Symbol.toPrimitive]: undefined;

  __parentPseudo: PseudoValue | undefined;
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

    if (!this.__parentWkri._) {
      // Could autoretreive?
      throw Error("This semantic node is currently on an unloaded WKRI");
    }

    const child = this.__parentWkri._.addPseudo(childNode, this.__tile);
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
    if (!parent || !parent._) {
      return new Map();
    }

    // Ensure lazy-loading done.
    // TODO check this does not go deeper than necessary.
    await parent._.loadNodes([...childNodes.keys()]);

    const children: Map<string, any> = new Map();
    for (const entry of [...parent._.allEntries()]) {
      const key = entry[0];
      let values = entry[1];
      if (values instanceof Promise) {
        values = await values;
      }
      if (values === false || values === null || values === undefined) {
        continue;
      }
      const childNode = childNodes.get(key);
      for (let value of values) {
        if (
          childNode &&
          value.node &&
          value !== null &&
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
    for (const [key, value] of [...children.entries()]) {
      value.parentNode = this.__parentPseudo;
      this.__childValues.set(key, value);
    }

    return children;
  }
}

const CUSTOM_DATATYPES: Map<string, string | IViewModel> = new Map();

async function getViewModel<RIVM extends IRIVM<RIVM>>(
  parentPseudo: PseudoValue,
  tile: StaticTile,
  node: StaticNode,
  data: any,
  parent: IRIVM<RIVM> | null,
  childNodes: Map<string, StaticNode>,
): Promise<IViewModel | null> {
  let vm;
  // TODO: should parentPseudo.parent._ trigger a retrieve if missing?
  const cacheEntries: {[tileId: string]: {[nodeId: string]: IStringKeyedObject}} | undefined = parentPseudo.parent && parentPseudo.parent._ ? await parentPseudo.parent._.getValueCache(false, undefined) : undefined;
  let cacheEntry: IStringKeyedObject | null = null;
  if (cacheEntries) {
    cacheEntry = (tile.tileid ? (cacheEntries[tile.tileid] ?? {}) : {})[node.nodeid]
  };
  const datatype = CUSTOM_DATATYPES.get(node.datatype) ?? node.datatype;
  if (!(typeof datatype == "string")) {
    // @ts-expect-error Cannot make a static member part of the interface
    vm = await datatype.__create(tile, node, data, cacheEntry);
  } else {
    switch (datatype) {
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
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptValueCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          cacheEntry = new ConceptValueCacheEntry(cacheEntry);
          // console.warn(`Cache entry for tile ${tile.tileid} on node ${node.nodeid} is not of type ConceptValueCacheEntry`);
        }
        vm = await ConceptValueViewModel.__create(tile, node, data, cacheEntry);
        break;
      case "resource-instance":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          cacheEntry = new ResourceInstanceCacheEntry(cacheEntry);
          // console.warn(`Cache entry for tile ${tile.tileid} on node ${node.nodeid} is not of type ResourceInstanceCacheEntry`);
        }
        vm = await ResourceInstanceViewModel.__create(tile, node, data, cacheEntry);
        break;
      case "resource-instance-list":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceListCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          cacheEntry = new ResourceInstanceListCacheEntry(cacheEntry);
          // console.warn(`Cache entry for tile ${tile.tileid} on node ${node.nodeid} is not of type ResourceInstanceListCacheEntry`);
        }
        vm = await ResourceInstanceListViewModel.__create(tile, node, data, cacheEntry);
        break;
      case "concept-list":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptListCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          cacheEntry = new ConceptListCacheEntry(cacheEntry);
          // console.warn(`Cache entry for tile ${tile.tileid} on node ${node.nodeid} is not of type ConceptListCacheEntry`);
        }
        vm = await ConceptListViewModel.__create(tile, node, data, cacheEntry);
        break;
      case "date":
        vm = await DateViewModel.__create(tile, node, data);
        break;
      case "geojson-feature-collection":
        vm = await GeoJSONViewModel.__create(tile, node, data);
        break;
      case "boolean":
        vm = await BooleanViewModel.__create(tile, node, data);
        break;
      case "string":
        vm = await StringViewModel.__create(tile, node, data);
        break
      case "number":
        vm = await NumberViewModel.__create(tile, node, data);
        break
      case "url":
        vm = await UrlViewModel.__create(tile, node, data);
        break;
      case "non-localized-string":
        vm = await NonLocalizedStringViewModel.__create(tile, node, data);
        break;
      default:
        console.warn("Missing type for tile", tile.tileid, "on node", node.alias, "with type", node.datatype);
        vm = await NonLocalizedStringViewModel.__create(tile, node, data);
    }
  }

  if (vm === null) {
    return null;
  }

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

  return vm;
}

export { ResourceInstanceCacheEntry, DEFAULT_LANGUAGE, ResourceInstanceViewModel, ValueList, getViewModel, DomainValueViewModel, SemanticViewModel, StringViewModel, DateViewModel, GeoJSONViewModel, ConceptValueViewModel, viewContext, NonLocalizedStringViewModel, CUSTOM_DATATYPES, BooleanViewModel, NumberViewModel, UrlViewModel };
