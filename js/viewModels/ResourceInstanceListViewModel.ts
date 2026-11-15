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
    return value ? Promise.all(value.map(async (v) => (v ? v.forJson() : null))) : null;
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
    // Built from core's value for this node (the tile's data is a copy):
    // the list holds its items, resolved, and _value resolves to them.
    if (!tile || value === null || value === undefined) {
      return null;
    }
    if (!Array.isArray(value)) {
      throw new Error(
        `Cannot set an (entire) resource list value on node ${node.nodeid} except via an array: ${JSON.stringify(value)}`,
      );
    }
    const items = await Promise.all(value.map((v, i) => {
      if (v instanceof ResourceInstanceViewModel) {
        return v;
      }
      return ResourceInstanceViewModel.__create(tile, node, v, cacheEntry && cacheEntry._[i] ? cacheEntry._[i] : null);
    }));

    const list = new ResourceInstanceListViewModel();
    list.push(...items);
    list._value = Promise.resolve(items);
    return list;
  }

  async __asTileData() {
    // Each item's tile form ({resourceId}), not the view models themselves.
    const value = await this._value;
    return value ? Promise.all(value.map(async (v) => (v ? v.__asTileData() : null))) : null;
  }
}
