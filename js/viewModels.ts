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
import { PseudoValue, PseudoList, wrapRustPseudo } from "./pseudos";
import { SemanticViewModel } from "./semantic";
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

const DEFAULT_LANGUAGE = "en";

class ViewContext {
  graphManager: IGraphManager | undefined
};
const viewContext = new ViewContext();

// ValueList has been removed - use IInstanceWrapper.retrievePseudo() instead

class ConceptListCacheEntry implements IStringKeyedObject {
  [key: string]: any;
  datatype: string = 'concept-list';
  _: ConceptValueCacheEntry[];
  meta: {[key: string]: any};

  constructor({meta, _}: {meta: IStringKeyedObject | undefined, _: ConceptValueCacheEntry[]}) {
    this._ = _.map(instance => {
      if (instance instanceof ConceptValueCacheEntry) {
        return instance;
      } else if (instance) {
        return new ConceptValueCacheEntry(instance);
      }
      return null;
    }).filter(cvce => cvce !== null);
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
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
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
            `Cannot set an (entire) resource list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`,
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
          tile.data.set(nodeid, ids.map(id => {
            return {
              resourceId: id
            };
          }));
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
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  $: IInstanceWrapper<RIVM> | null;
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
    if (!this.$) {
      return undefined;
    }
    return (await this.$.getRootViewModel() || new Map()).__has(key);
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
      if (!this.$) {
        await this.retrieve();
        if (!this.$) {
          throw Error("Could not retrieve resource");
        }
      }
      const root = await this.$.getRootViewModel();
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
      iw = replacement.$;
      // @ts-expect-error We cannot guarantee this resource is the right type...
      mw = replacement.__;
    } else {
      throw Error("Cannot traverse resource relationships without a GraphManager");
    }
    this.$ = iw;
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
    this.$ = instanceWrapperFactory ? instanceWrapperFactory(this) : null;
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
        } else if (k in object || k.startsWith('__')) {
          object[k] = value;
        } else {
          if (!object.$) {
            await this.retrieve();
            if (!object.$) {
              throw Error("Could not retrieve resource");
            }
          }
          object.$.setOrmAttribute(k, value);
        }
        return true;
      },
      get: (object: ResourceInstanceViewModel<RIVM>, key) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;
        if (key in object) {
          return object[key];
        } else if (k in object || k.startsWith('__')) {
          return object[k];
        }
        return new AttrPromise(async (resolve) => {
          if (!object.$) {
            await this.retrieve();
            if (!object.$) {
              throw Error("Could not retrieve resource");
            }
          }
          const p = object.$.getOrmAttribute(k);
          return p.then((v) => {
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
        } else if (value instanceof Map && value.get("resourceId")) {
          val = value.get("resourceId");
        } else if (value instanceof Array && value.length < 2 ) {
          if (value.length == 1) {
            return ResourceInstanceViewModel.__create(tile, node, value[0], cacheEntry);
          }
        } else {
          throw Error("Could not set resource instance from this data");
        }

        tile.data.set(nodeid, val ? [{resourceId: val}] : null);
      }
    }

    if (!tile || !val) {
      return null;
    }
    const str = new ResourceInstanceViewModel(val, null, null, cacheEntry);
    return str;
  }
}

class FileListViewModel extends Array implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(ConceptValueViewModel | null)[]> | null = null;

  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => (v ? v.forJson() : null)) : null;
  }

  async __forJsonCache(): Promise<null> {
    return null;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): Promise<FileListViewModel> {
    const nodeid = node.nodeid;
    let val: (ConceptValueViewModel | Promise<ConceptValueViewModel | null> | null)[] = [];
    if (!tile.data.has(nodeid)) {
      tile.data.set(nodeid, null);
    }
    if (value !== null) {
      tile.data.set(nodeid, []);
      if (!Array.isArray(value)) {
        throw Error(
          `Cannot set an (entire) file list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`,
        );
      }
      val = value.map((c) => {
        return c;
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

    const str = new FileListViewModel(...value);
    return str;
  }

  async __asTileData() {
    return this._value ? await this._value : null;
  }
}

class ConceptListViewModel extends Array implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
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
      _: (await Promise.all([...this.values()].map(async (rivmPromise: Promise<ConceptValueViewModel>) => {
        const rivm = await rivmPromise;
        if (rivm) {
          return await rivm.__forJsonCache(getMeta)
        }
      }))).filter(val => val !== undefined)
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
          `Cannot set an (entire) concept list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`,
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
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

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
            `Cannot set an (entire) domain list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`,
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
      } else {
        value = [];
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
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

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
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
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
      conceptId: value.__conceptId ?? null
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
    const collectionId = node.config?.rdmCollection;
    if (!collectionId) {
      throw Error(`Node ${node.alias} (${node.nodeid}) missing rdmCollection in config`);
    }
    let val: StaticValue | null = value;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (value instanceof StaticConcept) {
          if (value.getPrefLabel) {
            val = value.getPrefLabel();
          } else {
            throw Error("Recognizing value as StaticConcept, but no getPrefLabel member");
          }
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
              const collection = RDM.retrieveCollection(collectionId);
              return collection.then((collection) => {
                if (!collection.getConceptValue) {
                  throw Error(`Collection ${collection.id} must be a StaticCollection here, not a key/value object`);
                }
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
            console.error("Could not find concept for value", value, "for", node.alias, "in collection", collectionId);
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
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;
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
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;
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

class EDTFViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

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
  ): EDTFViewModel | Promise<EDTFViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) => EDTFViewModel.__create(tile, node, value));
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
    const string = new EDTFViewModel(val);
    return string;
  }

  __asTileData() {
    return `${this}`;
  }
}

class NonLocalizedStringViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

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
    return `${this}`;
  }
}

class NumberViewModel extends Number implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

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
    return this.valueOf();
  }
}

// Note that this is a Boolean _object__, not an actual boolean
class BooleanViewModel extends Boolean implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;
  __config:  StaticNodeConfigBoolean;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  constructor(value: boolean, config: StaticNodeConfigBoolean) {
    super(value);
    this.__config = config;
  }

  toString(lang?: string | undefined): string {
    const labelLang = lang || DEFAULT_LANGUAGE;
    return this.valueOf() ? (
      this.__config && this.__config.trueLabel ? this.__config.trueLabel[labelLang] || 'true' : 'true'
    ) : (
      this.__config && this.__config.trueLabel ? this.__config.falseLabel[labelLang] || 'false' : 'false'
    );
  }

  __forJsonCache(): null {
    return null;
  }

  forJson(): boolean {
    return this.valueOf();
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
    if (typeof val !== 'boolean' && val !== 0 && val !== 1) {
      throw Error(`Refusing to use truthiness for value ${val} in boolean`);
    }
    const bool = new BooleanViewModel(val ? true : false, config);
    return bool;
  }

  __asTileData() {
    return this.valueOf();
  }
}

class Url {
  url: string
  url_label?: string

  constructor(url: string, url_label?: string) {
    this.url = url;
    this.url_label = url_label;
  }
}

class UrlViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: Url;

  __forJsonCache(): null {
    return null;
  }

  constructor(value: Url) {
    const displayValue = value.url_label || value.url;
    super(displayValue);
    this._value = value;
  }

  forJson(): {[key: string]: string} {
    return {
      url: this._value.url,
      url_label: this._value.url_label || "",
    };
  }

  label() {
    return this._value.url_label || this._value.url;
  }

  href() {
    return this._value.url;
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
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, {});
      }
      if (value !== null) {
        if (value instanceof UrlViewModel) {
          value = value._value;
        } else if (value instanceof Object) {
          if (!value.url) {
            throw Error(`A URL must be null or have a 'url' field: ${value}`);
          }
        }
        tile.data.set(nodeid, {
          url: value.url,
          url_label: value.url_label,
        });
      }
    }

    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === undefined) {
      return null;
    }
    let url: Url;
    if (typeof val !== 'object') {
      url = new Url(`${val}`);
    } else if (val instanceof Map) {
      url = new Url(val.get('url'), val.get('url_label'));
    } else if ('url' in val && typeof val === 'object' && typeof val.url === 'string' && 'url_label' in val && (val.url_label === undefined || typeof val.url_label === 'string')) {
      url = new Url(val.url, val.url_label);
    } else {
      throw Error(`Unrecognised URL type: ${val}`);
    }
    const str = new UrlViewModel(url);
    return str;
  }

  __asTileData() {
    return this.forJson();
  }
}

class StringViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

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

class NodeViewModel implements IStringKeyedObject, IViewModel {
  [key: string | symbol]: any;
  then: undefined;
  [Symbol.toPrimitive]: undefined;

  __parentPseudo: PseudoNode;
  __parentWkrm: any | null;

  __forJsonCache(): null {
    return null;
  }

  constructor(
    parentPseudo: PseudoNode,
    parentWkrm: any | null,
  ) {
    this.__parentPseudo = parentPseudo;
    this.__parentWkrm = parentWkrm;
    return new Proxy(this, {
      set: (object, key, value) => {
        const k: string = typeof key === 'symbol' ? key.description || '' : key;
        if (key in object) {
          object[key] = value;
        } else if (k.startsWith("__") || k in object) {
          object[k] = value;
        } else {
          throw Error("Cannot set values on a node");
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
        } else if (k == "_") {
          return this.__parentPseudo.node;
        } else if (k.endsWith("$edge")) {
          return this.__getEdgeTo(k.substring(0, k.length - 5));
        }
        if (k == "length") {
          return object.__parentPseudo.size;
        }
        return new AttrPromise((resolve) => {
          object.__get(k).then(resolve);
        });
      },
    });
  }

  async toString(): Promise<string> {
    if (!this.__parentPseudo) {
      return "[NodeViewModel]";
    }
    const alias = this.__parentPseudo.alias;
    return alias || "[unnamed]";
  }

  async __getEdgeTo(key: string) {
    const childNode = this.__parentPseudo.childNodes.get(key);

    if (!childNode) {
      throw Error(`Child node key ${key} missing`);
    }

    const domainId = this.__parentPseudo.nodeid;
    const rangeId = childNode.nodeid;
    const edges = this.__parentWkrm.graph.edges.filter(
      (edge: StaticEdge) => edge.domainnode_id === domainId && edge.rangenode_id === rangeId
    );
    if (edges.length !== 1) {
      throw Error(`Number of edges from ${domainId}->${rangeId} != 1`);
    }
    return edges[0];
  }

  async __get(key: string) {
    const pseudo = this.__parentWkrm.createPseudoNodeChild(key, this.__parentPseudo);
    return NodeViewModel.__create(pseudo, this.__parentWkrm);
  }

  static async __create(
    pseudo: PseudoNode,
    parent: any | null,
  ): Promise<NodeViewModel> {
    const node = new NodeViewModel(pseudo, parent);
    return node;
  }
}

const CUSTOM_DATATYPES: Map<string, string | IViewModel> = new Map();

async function getViewModel<RIVM extends IRIVM<RIVM>>(
  parentPseudo: PseudoValue<any>,
  tile: StaticTile,
  node: StaticNode,
  data: any,
  parent: IRIVM<RIVM> | null,
  isInner: boolean = false
): Promise<IViewModel | null> {
  let vm;
  // TODO: should parentPseudo.parent.$ trigger a retrieve if missing?
  const cacheEntries: {[tileId: string]: {[nodeId: string]: IStringKeyedObject}} | undefined = parentPseudo.parent && parentPseudo.parent.$ ? await parentPseudo.parent.$.getValueCache(false, undefined) : undefined;
  let cacheEntry: IStringKeyedObject | null = null;
  if (cacheEntries) {
    cacheEntry = (tile.tileid ? (cacheEntries[tile.tileid] ?? {}) : {})[node.nodeid]
  };
  const datatype = isInner ? "semantic" : CUSTOM_DATATYPES.get(node.datatype) ?? node.datatype;

  // TODO: find a neater way.
  let conceptCacheEntry: ConceptListCacheEntry | null;
  let conceptValueCacheEntry: ConceptValueCacheEntry | null;
  let resourceInstanceCacheEntry: ResourceInstanceCacheEntry | null;
  let resourceInstanceListCacheEntry: ResourceInstanceListCacheEntry | null;

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
          conceptValueCacheEntry = new ConceptValueCacheEntry(cacheEntry);
        } else {
          conceptValueCacheEntry = cacheEntry;
        }
        vm = await ConceptValueViewModel.__create(tile, node, data, conceptValueCacheEntry);
        break;
      case "resource-instance":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          resourceInstanceCacheEntry = new ResourceInstanceCacheEntry(cacheEntry);
        } else {
          resourceInstanceCacheEntry = cacheEntry;
        }
        vm = await ResourceInstanceViewModel.__create(tile, node, data, resourceInstanceCacheEntry);
        break;
      case "resource-instance-list":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceListCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          resourceInstanceListCacheEntry = new ResourceInstanceListCacheEntry(cacheEntry);
        } else {
          resourceInstanceListCacheEntry = cacheEntry;
        }
        vm = await ResourceInstanceListViewModel.__create(tile, node, data, resourceInstanceListCacheEntry);
        break;
      case "concept-list":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptListCacheEntry)) {
          // @ts-expect-error We do not know the cache entry is structured correctly, and any such checks are in the constructor.
          conceptCacheEntry = new ConceptListCacheEntry(cacheEntry);
        } else {
          conceptCacheEntry = cacheEntry;
        }
        vm = await ConceptListViewModel.__create(tile, node, data, conceptCacheEntry);
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
      case "file-list":
        vm = await FileListViewModel.__create(tile, node, data);
        break;
      case "edtf":
        vm = await EDTFViewModel.__create(tile, node, data);
        break;
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
        vme.then(vmep => { if (vmep !== null) vmep.__parentPseudo = parentPseudo; });
      } else {
        vme.__parentPseudo = parentPseudo;
      }
    }
  }

  return vm;
}

export { ResourceInstanceCacheEntry, DEFAULT_LANGUAGE, ResourceInstanceViewModel, getViewModel, DomainValueViewModel, SemanticViewModel, StringViewModel, DateViewModel, GeoJSONViewModel, ConceptValueViewModel, viewContext, NonLocalizedStringViewModel, CUSTOM_DATATYPES, BooleanViewModel, NumberViewModel, UrlViewModel, NodeViewModel };
