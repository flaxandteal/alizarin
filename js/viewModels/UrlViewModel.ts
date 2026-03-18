import { IViewModel } from "../interfaces";
import { PseudoValue } from "../pseudos";
import { StaticTile, StaticNode } from "../static-types";
import { Url } from "./types";

export class UrlViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: PseudoValue<any> | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: Url;

  __forJsonCache(): null {
    return null;
  }

  constructor(value: Url) {
    const displayValue = value.url_label || value.url;
    super(displayValue);
    this._value = value;
  }

  forJson(): {[key: string]: string} {
    return {
      url: this._value.url,
      url_label: this._value.url_label || "",
    };
  }

  label() {
    return this._value.url_label || this._value.url;
  }

  href() {
    return this._value.url;
  }

  static __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): UrlViewModel | Promise<UrlViewModel | null> | null {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value) => UrlViewModel.__create(tile, node, value));
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, {});
      }
      if (value !== null) {
        if (value instanceof UrlViewModel) {
          value = value._value;
        } else if (value instanceof Object) {
          if (!value.url) {
            throw new Error(`A URL must be null or have a 'url' field: ${value}`);
          }
        }
        tile.data.set(nodeid, {
          url: value.url,
          url_label: value.url_label,
        });
      }
    }

    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === undefined) {
      return null;
    }
    let url: Url;
    if (typeof val !== 'object') {
      url = new Url(`${val}`);
    } else if (val instanceof Map) {
      url = new Url(val.get('url'), val.get('url_label'));
    } else if ('url' in val && typeof val === 'object' && typeof val.url === 'string' && 'url_label' in val && (val.url_label === undefined || typeof val.url_label === 'string')) {
      url = new Url(val.url, val.url_label);
    } else {
      throw new Error(`Unrecognised URL type: ${val}`);
    }
    const str = new UrlViewModel(url);
    return str;
  }

  __asTileData() {
    return this.forJson();
  }
}
