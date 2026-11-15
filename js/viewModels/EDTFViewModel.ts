import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";

export class EDTFViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  __forJsonCache(): null {
    return null;
  }

  forJson(): string {
    return this.toString();
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): EDTFViewModel | Promise<EDTFViewModel | null> | null {
    if (value instanceof Promise) {
      return value.then((value) => EDTFViewModel.__create(tile, node, value));
    }
    // Core's value for this node, as BooleanViewModel's.
    const val = value;
    if (!tile || val === null || val === undefined) {
      return null;
    }
    const string = new EDTFViewModel(val);
    return string;
  }

  __asTileData() {
    return `${this}`;
  }
}
