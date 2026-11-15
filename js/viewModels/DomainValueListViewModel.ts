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
    return value ? Promise.all(value.map(async (v) => (v ? v.forJson() : null))) : null;
  }

  __forJsonCache(): null {
    return null;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): Promise<DomainValueListViewModel> {
    // Built from core's value for this node (the tile's data is a copy):
    // the list holds its items, resolved, and _value resolves to them.
    let items: (DomainValueViewModel | null)[] = [];
    if (tile && value !== null && value !== undefined) {
      if (!Array.isArray(value)) {
        throw new Error(
          `Cannot set an (entire) domain list value on node ${node.nodeid} except via an array: ${JSON.stringify(value)}`,
        );
      }
      items = await Promise.all(value.map((c) => {
        if (c instanceof DomainValueViewModel) {
          return c;
        }
        return DomainValueViewModel.__create(tile, node, c);
      }));
    }

    const list = new DomainValueListViewModel();
    list.push(...items);
    list._value = Promise.resolve(items);
    return list;
  }

  async __asTileData() {
    // Each item's tile form (its option id), not the view models themselves.
    const value = await this._value;
    return value ? Promise.all(value.map(async (v) => (v ? v.__asTileData() : null))) : null;
  }
}
