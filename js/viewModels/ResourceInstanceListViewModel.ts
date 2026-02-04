import { IViewModel, IPseudo, GetMeta } from "../interfaces";
import { StaticTile, StaticNode } from "../static-types";
import { ResourceInstanceListCacheEntry } from "./cacheEntries";
import { ResourceInstanceViewModel } from "./ResourceInstanceViewModel";

export class ResourceInstanceListViewModel extends Array implements IViewModel {
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
          throw new Error(
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
