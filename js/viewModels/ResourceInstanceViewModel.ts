import {
  IStringKeyedObject,
  IInstanceWrapper,
  IModelWrapper,
  IViewModel,
  IPseudo,
  IRIVM,
  GetMeta,
} from "../interfaces";
import { StaticTile, StaticNode, StaticResource, StaticResourceReference } from "../static-types";
import { AttrPromise } from "../utils";
import { recordWasmTiming } from '../wasmTiming';
import { viewContext, DEFAULT_LANGUAGE } from "./types";
import { ResourceInstanceCacheEntry, ResourceDescriptors } from "./cacheEntries";

export class ResourceInstanceViewModel<RIVM extends IRIVM<RIVM>> implements IStringKeyedObject {
  [key: string | symbol]: any;
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  $: IInstanceWrapper<RIVM> | null;
  __: IModelWrapper<RIVM> | null;
  __parentPseudo: IPseudo | undefined = undefined;
  __cacheEntry: ResourceInstanceCacheEntry | null = null;
  id: string;
  then: undefined;
  [Symbol.toPrimitive]: undefined;

  gm: typeof viewContext.graphManager;

  toString(): string {
    if (!this.__) {
      return `[Resource:${this.id}]`;
    }
    return `[${this.__.wkrm.modelClassName}:${this.id ?? "-"}]`;
  }

  async getName(retrieveIfNeeded: boolean=false): Promise<string> {
    return (await this.getDescriptors(retrieveIfNeeded))?.name;
  }

  async getSlug(retrieveIfNeeded: boolean=false): Promise<string> {
    return (await this.getDescriptors(retrieveIfNeeded))?.slug;
  }

  async getDescription(retrieveIfNeeded: boolean=false): Promise<string> {
    return (await this.getDescriptors(retrieveIfNeeded))?.description;
  }

  async getMapPopup(retrieveIfNeeded: boolean=false): Promise<string> {
    return (await this.getDescriptors(retrieveIfNeeded))?.map_popup;
  }

  async getDescriptors(retrieveIfNeeded: boolean=false): Promise<ResourceDescriptors | undefined> {
    if (this.__cacheEntry?.descriptors) {
      return this.__cacheEntry.descriptors;
    }
    if (retrieveIfNeeded) {
      if (!this.$) {
        await this.retrieve();
      }
      if (this.$) {
        return this.$.getDescriptors();
      }
    }
    return undefined;
  }

  async __has(key: string): Promise<boolean | undefined> {
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
    const forJsonStart = performance.now();
    let rootJson = null;
    if (cascade) {
      let t0 = performance.now();
      if (!this.$) {
        await this.retrieve();
        if (!this.$) {
          throw new Error("Could not retrieve resource");
        }
      }
      recordWasmTiming("forJson: retrieve check", performance.now() - t0);

      t0 = performance.now();
      await this.$.populate(false);
      recordWasmTiming("forJson: populate", performance.now() - t0);

      t0 = performance.now();
      rootJson = this.$.wasmWrapper.toJson();
      recordWasmTiming("forJson: toJson", performance.now() - t0);
    }
    recordWasmTiming("forJson total (viewModels)", performance.now() - forJsonStart);

    if (!cascade && this.__cacheEntry) {
      return {
        type: this.__cacheEntry.type,
        graphId: this.__cacheEntry.graphId,
        id: this.__cacheEntry.id,
        title: this.__cacheEntry.title || undefined,
        descriptors: this.__cacheEntry.descriptors || undefined,
        meta: this.__cacheEntry.meta || undefined,
        root: rootJson
      };
    } else if (this.__) {
      return {
        type: this.__.wkrm.modelClassName,
        graphId: this.__.wkrm.graphId,
        id: this.id,
        title: undefined,
        descriptors: undefined,
        meta: undefined,
        root: rootJson
      };
    } else {
      return {
        type: "(unknown)",
        graphId: "",
        id: this.id,
        title: undefined,
        descriptors: undefined,
        meta: undefined,
        root: rootJson
      };
    }
  }

  /**
   * Get JSON representation with display-friendly values.
   *
   * Unlike forJson() which returns tile data format (language maps, StaticReference objects),
   * this returns human-readable strings using registered display serializers.
   *
   * Use this for ETL/export/indexing where you want display strings instead of structured data.
   *
   * @param cascade - If true, ensures the resource is fully populated before serializing
   * @param language - Language code for display strings (defaults to current language)
   * @returns JSON with display-friendly values
   */
  async forDisplayJson(cascade: boolean=false, language?: string) {
    const forJsonStart = performance.now();
    let rootJson = null;
    if (cascade) {
      let t0 = performance.now();
      if (!this.$) {
        await this.retrieve();
        if (!this.$) {
          throw new Error("Could not retrieve resource");
        }
      }
      recordWasmTiming("forDisplayJson: retrieve check", performance.now() - t0);

      t0 = performance.now();
      await this.$.populate(false);
      recordWasmTiming("forDisplayJson: populate", performance.now() - t0);

      t0 = performance.now();
      const lang = language || DEFAULT_LANGUAGE;
      rootJson = this.$.wasmWrapper.toDisplayJsonSimple(lang);
      recordWasmTiming("forDisplayJson: toDisplayJsonSimple", performance.now() - t0);
    }
    recordWasmTiming("forDisplayJson total (viewModels)", performance.now() - forJsonStart);

    if (!cascade && this.__cacheEntry) {
      return {
        type: this.__cacheEntry.type,
        graphId: this.__cacheEntry.graphId,
        id: this.__cacheEntry.id,
        title: this.__cacheEntry.title || undefined,
        descriptors: this.__cacheEntry.descriptors || undefined,
        meta: this.__cacheEntry.meta || undefined,
        root: rootJson
      };
    } else if (this.__) {
      return {
        type: this.__.wkrm.modelClassName,
        graphId: this.__.wkrm.graphId,
        id: this.id,
        title: undefined,
        descriptors: undefined,
        meta: undefined,
        root: rootJson
      };
    } else {
      return {
        type: "(unknown)",
        graphId: "",
        id: this.id,
        title: undefined,
        descriptors: undefined,
        meta: undefined,
        root: rootJson
      };
    }
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
      throw new Error("Cannot traverse resource relationships without a GraphManager");
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
              throw new Error("Could not retrieve resource");
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
              throw new Error("Could not retrieve resource");
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
            throw new Error(
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
          throw new Error("Could not set resource instance from this data");
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
