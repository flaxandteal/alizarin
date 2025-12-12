import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
import { DomainValueViewModel } from "./DomainValueViewModel";

export class DomainValueListViewModel extends Array implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(DomainValueViewModel | null)[]> | null = null;

  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => (v ? v.forJson() : null)) : null;
  }

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
          throw new Error(
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
