import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
import { StaticNodeConfigBoolean, nodeConfigManager } from '../nodeConfig';
import { DEFAULT_LANGUAGE } from "./types";

export class BooleanViewModel extends Boolean implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;
  __config: StaticNodeConfigBoolean;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  constructor(value: boolean, config: StaticNodeConfigBoolean) {
    super(value);
    this.__config = config;
  }

  toString(lang?: string | undefined): string {
    const labelLang = lang || DEFAULT_LANGUAGE;
    return this.valueOf() ? (
      this.__config && this.__config.trueLabel ? this.__config.trueLabel[labelLang] || 'true' : 'true'
    ) : (
      this.__config && this.__config.trueLabel ? this.__config.falseLabel[labelLang] || 'false' : 'false'
    );
  }

  __forJsonCache(): null {
    return null;
  }

  forJson(): boolean {
    return this.valueOf();
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): BooleanViewModel | Promise<BooleanViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) => BooleanViewModel.__create(tile, node, value));
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
    const config = nodeConfigManager.retrieve(node);
    if (!config || !(config instanceof StaticNodeConfigBoolean)) {
      throw new Error(`Cannot form boolean value for ${node.nodeid} without config`);
    }
    if (typeof val !== 'boolean' && val !== 0 && val !== 1) {
      throw new Error(`Refusing to use truthiness for value ${val} in boolean`);
    }
    const bool = new BooleanViewModel(val ? true : false, config);
    return bool;
  }

  __asTileData() {
    return this.valueOf();
  }
}
