import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";

export class DateViewModel extends Date implements IViewModel {
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
    if (val && typeof val == "object" && val['en'] !== undefined) {
      val = val.en;
    }
    if (!tile || val === null || val === undefined || val === '') {
      return null;
    }
    if (typeof val != "string") {
      throw new Error("Date should be a string");
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
