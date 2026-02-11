import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
import { DEFAULT_LANGUAGE, StringTranslatedLanguage } from "./types";

export class StringViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: Map<string, StringTranslatedLanguage>;

  __forJsonCache(): null {
    return null;
  }

  constructor(value: Map<string, StringTranslatedLanguage>, language: string | null = null) {
    const lang = value.get(language || DEFAULT_LANGUAGE);
    let displayValue: string;
    if (lang) {
      if (typeof lang == "string") {
        displayValue = lang;
      } else {
        displayValue = lang.value;
      }
    } else {
      displayValue = "";
    }
    super(displayValue);
    this._value = value;
  }

  forJson(): string {
    return this.toString();
  }

  lang(language: string) {
    const elt = this._value.get(language);
    if (elt) {
      if (elt instanceof Object) {
        return elt.value;
      }
      return elt;
    } else {
      return undefined;
    }
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): StringViewModel | Promise<StringViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) => StringViewModel.__create(tile, node, value));
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, {});
      }
      if (value !== null) {
        if (value instanceof Object) {
          const entries =
            value instanceof Map ? value.entries() : Object.entries(value);
          for (const [k, v] of [...entries]) {
            const val = tile.data.get(nodeid);
            if (val instanceof Map) {
              val.set(k, v);
            } else if (val instanceof Object) {
              // @ts-expect-error Need better typing of data to make this settable.
              val[k] = v;
            } else if (val !== null) {
              throw new Error("Malformed string in tile data");
            }
          }
        } else {
          tile.data.set(nodeid, {
            [DEFAULT_LANGUAGE]: value,
          });
        }
      }
    }

    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === undefined) {
      return null;
    }
    let mapVal;
    if (val instanceof Map) {
      mapVal = val;
    } else {
      mapVal = new Map(Object.entries(val));
    }
    const str = new StringViewModel(mapVal);
    return str;
  }

  __asTileData() {
    return this._value;
  }
}
