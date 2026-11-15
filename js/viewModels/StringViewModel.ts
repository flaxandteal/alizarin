import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
import { DEFAULT_LANGUAGE, StringTranslatedLanguage } from "./types";
import { getCurrentLanguage } from "../utils";

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
    // The current language (what a plain string is assigned in), falling back
    // to the default where the value has none: as Python's StringViewModel.
    const lang = value.get(language || getCurrentLanguage()) ?? value.get(DEFAULT_LANGUAGE);
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
    if (value instanceof Promise) {
      return value.then((value) => StringViewModel.__create(tile, node, value));
    }

    // Core's value for this node, as BooleanViewModel's: a plain string is
    // the default language's.
    let val = value;
    if (val !== null && val !== undefined && !(val instanceof Object)) {
      val = { [DEFAULT_LANGUAGE]: val };
    }
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
