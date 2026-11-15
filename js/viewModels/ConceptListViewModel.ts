import { IViewModel, IPseudo, GetMeta } from "../interfaces";
import { StaticTile, StaticNode } from "../static-types";
import { ConceptListCacheEntry } from "./cacheEntries";
import { ConceptValueViewModel } from "./ConceptValueViewModel";

export class ConceptListViewModel extends Array implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(ConceptValueViewModel | null)[]> | null = null;

  async forJson() {
    const value = await this._value;
    return value ? Promise.all(value.map(async (v) => (v ? v.forJson() : null))) : null;
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
    // Built from core's value for this node (the tile's data is a copy):
    // the list holds its items, resolved, and _value resolves to them.
    let items: (ConceptValueViewModel | null)[] = [];
    if (value !== null && value !== undefined) {
      if (!Array.isArray(value)) {
        throw new Error(
          `Cannot set an (entire) concept list value on node ${node.nodeid} except via an array: ${JSON.stringify(value)}`,
        );
      }
      items = await Promise.all(value.map((c, i) => {
        if (c instanceof ConceptValueViewModel) {
          return c;
        }
        return ConceptValueViewModel.__create(tile, node, c, cacheEntry && cacheEntry._[i] ? cacheEntry._[i] : null);
      }));
    }

    const list = new ConceptListViewModel();
    list.push(...items);
    list._value = Promise.resolve(items);
    return list;
  }

  async __asTileData() {
    // Each item's tile form (its value id), not the view models themselves.
    const value = await this._value;
    return value ? Promise.all(value.map(async (v) => (v ? v.__asTileData() : null))) : null;
  }
}
