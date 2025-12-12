import { IViewModel, IPseudo } from "../interfaces";
import { StaticTile, StaticNode } from "../static-types";
import { ConceptValueViewModel } from "./ConceptValueViewModel";

export class FileListViewModel extends Array implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(ConceptValueViewModel | null)[]> | null = null;

  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => (v ? v.forJson() : null)) : null;
  }

  async __forJsonCache(): Promise<null> {
    return null;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
  ): Promise<FileListViewModel> {
    const nodeid = node.nodeid;
    let val: (ConceptValueViewModel | Promise<ConceptValueViewModel | null> | null)[] = [];
    if (!tile.data.has(nodeid)) {
      tile.data.set(nodeid, null);
    }
    if (value !== null) {
      tile.data.set(nodeid, []);
      if (!Array.isArray(value)) {
        throw new Error(
          `Cannot set an (entire) file list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`,
        );
      }
      val = value.map((c) => {
        return c;
      });
      Promise.all(val).then((vals) => {
        Promise.all(
          vals.map(async (c) => {
            const v = await c;
            return v ? (await v.getValue()).id : null;
          })
        ).then((ids) => {
          tile.data.set(nodeid, ids);
        });
      });
      value = val;
    } else {
      value = [];
    }

    const str = new FileListViewModel(...value);
    return str;
  }

  async __asTileData() {
    return this._value ? await this._value : null;
  }
}
