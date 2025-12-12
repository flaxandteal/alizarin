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
        throw new Error(
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
