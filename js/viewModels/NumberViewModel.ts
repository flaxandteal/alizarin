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
    console.debug(`[NumberViewModel.__create] nodeid=${nodeid} alias=${node.alias} value=`, value, `type=${typeof value} tile.tileid=${tile?.tileid} tile.data keys:`, tile?.data ? [...tile.data.keys()] : 'no data');
    if (tile) {
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }

    const val = tile.data.get(nodeid);
    console.debug(`[NumberViewModel.__create] nodeid=${nodeid} val from tile.data.get=`, val, `type=${typeof val}`);
    if (!tile || val === null || val === undefined) {
      console.warn(`[NumberViewModel.__create] RETURNING NULL for nodeid=${nodeid} alias=${node.alias} tile=${!!tile} val=${val}`);
      return null;
    }
    const num = new NumberViewModel(val);
    return num;
  }

  __asTileData() {
    return this.valueOf();
  }
}
