import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";

export class NumberViewModel extends Number implements IViewModel {
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
