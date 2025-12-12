import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode, StaticDomainValue } from "../static-types";
import { StaticNodeConfigDomain, nodeConfigManager } from '../nodeConfig';

export class DomainValueViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: StaticDomainValue | Promise<StaticDomainValue>;

  constructor(value: StaticDomainValue) {
    super(value.toString());
    this._value = value;
  }

  async forJson(): Promise<StaticDomainValue> {
    return this._value;
  }

  __forJsonCache(): null {
    return null;
  }

  getValue(): StaticDomainValue | Promise<StaticDomainValue> {
    return this._value;
  }

  async lang(lang: string): Promise<string | undefined> {
    return (await this._value).lang(lang);
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): Promise<DomainValueViewModel | null> {
    const nodeid = node.nodeid;
    let val: StaticDomainValue | null = value;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (!value && !(value instanceof StaticDomainValue)) {
          val = null;
        } else if (value instanceof Promise) {
          return value.then((value) => {
            return DomainValueViewModel.__create(tile, node, value);
          });
        } else if (typeof value == "string") {
          if (
            /^[0-9a-f]{8}-[0-9a-f]{4}-[0-5][0-9a-f]{3}-[089ab][0-9a-f]{3}-[0-9a-f]{12}$/i.exec(
              value,
            )
          ) {
            const config = nodeConfigManager.retrieve(node);
            if (!config || !(config instanceof StaticNodeConfigDomain)) {
              throw new Error(`Cannot form domain value for ${node.nodeid} without config`);
            }
            val = config.valueFromId(value) || null;
          } else {
            throw new Error(
              "Set domain values using values from domain lists, not strings",
            );
          }
        } else {
          throw new Error("Could not set domain value from this data");
        }

        if (!(val instanceof Promise)) {
          tile.data.set(nodeid, val ? val.id : null);
        }
      }
    }

    if (!tile || !val) {
      return null;
    }
    const str = new DomainValueViewModel(val);
    return str;
  }

  async __asTileData() {
    const value = await this._value;
    return value ? value.id : null;
  }
}
