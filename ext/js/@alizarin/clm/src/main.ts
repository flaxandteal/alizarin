import { RDM, nodeConfig, utils, interfaces, staticTypes, viewModels } from "alizarin";
type IPseudo = interfaces.IPseudo;
type IViewModel = interfaces.IViewModel;
type StaticTile = staticTypes.StaticTile;
type StaticNode = staticTypes.StaticNode;
// [{"labels": [{"id": "0ea39e2e-6663-467c-8707-ab492896d23e", "language_id": "en", "list_item_id": "6672d187-dfc3-4424-8c63-7a3b377b4159", "value": "Item 1>1", "valuetype_id": "prefLabel"}], "list_id": "2730d609-3a8d-49dc-bf51-6ac34e80294a", "uri": "http://localhost:8000/plugins/controlled-list-manager/item/6672d187-dfc3-4424-8c63-7a3b377b4159"}]

// class ReferenceListCacheEntry implements IStringKeyedObject {
//   [key: string]: any;
//   datatype: string = 'reference';
//   _: ReferenceValueCacheEntry[];
//   meta: {[key: string]: any};
// 
//   constructor({meta, _}: {meta: IStringKeyedObject | undefined, _: ReferenceValueCacheEntry[]}) {
//     this._ = _.map(instance => {
//       if (instance instanceof ReferenceValueCacheEntry) {
//         return instance;
//       } else if (instance) {
//         return new ReferenceValueCacheEntry(instance);
//       }
//       return null;
//     }).filter(cvce => cvce !== null);
//     this.meta = meta || {};
//   }
// }
// 
// class ReferenceValueCacheEntry implements IStringKeyedObject {
//   [key: string]: any
//   datatype: string = 'reference';
//   id: string;
//   value: string;
//   referenceId: string | null;
//   meta: {[key: string]: any};
// 
//   constructor({meta, id, value, referenceId}: {meta: IStringKeyedObject | undefined, id: string, value: string, referenceId: string | null}) {
//     this.id = id;
//     this.value = value;
//     this.referenceId = referenceId;
//     this.meta = meta || {};
//   }
// }

class StaticReferenceLabel {
  id: string
  language_id: string
  list_item_id: string
  value: string
  valuetype_id: string

  constructor(label: StaticReferenceLabel) {
    this.id = label.id;
    this.language_id = label.language_id;
    this.list_item_id = label.list_item_id;
    this.value = label.value;
    this.valuetype_id = label.valuetype_id;
  }
};

class StaticReference {
  labels: StaticReferenceLabel[]
  list_id: string
  uri: string

  constructor(reference: StaticReference) {
    this.list_id = reference.list_id;
    this.uri = reference.uri;
    this.labels = [];
    for (const label of reference.labels) {
      if (label instanceof StaticReferenceLabel) {
        this.labels.push(label);
      } else {
        this.labels.push(new StaticReferenceLabel(label));
      }
    }
  }
};

function referenceToString(reference: StaticReference): string {
  if (reference.labels.length == 1) {
    return reference.labels[0].value;
  }
  let prefLabel: string | undefined;
  const lang = utils.getCurrentLanguage();
  for (const label of reference.labels) {
    if (label.valuetype_id === "prefLabel") {
      prefLabel = label.value;
      if (label.language_id === lang) {
        return prefLabel;
      }
    }
  }
  return prefLabel || "(undefined)";
}

class ReferenceValueViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _ref: StaticReference | Promise<StaticReference>;

  constructor(reference: StaticReference) {
    super(referenceToString(reference));
    this._ref = reference;
  }

  async forJson(): Promise<StaticReference> {
    return this._ref;
  }

  getValue(): StaticReference | Promise<StaticReference> {
    return this._ref;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    _cacheEntry: object
  ): Promise<ReferenceValueViewModel | null> {
    const nodeid = node.nodeid;
    const collectionId = node.config?.controlledList || node.config?.rdmCollection;
    if (!collectionId) {
      throw Error(`Node ${node.alias} (${node.nodeid}) missing controlledList or rdmCollection in config`);
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (value instanceof Promise) {
          return value.then((value) => {
            return ReferenceValueViewModel.__create(tile, node, value, _cacheEntry);
          });
        } else if (typeof value == "string") {
          if (
            /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(
              value,
            )
          ) {
            const collection = RDM.retrieveCollection(collectionId);
            return collection.then((collection: staticTypes.StaticCollection) => {
              if (!collection.getReferenceValue) {
                throw Error(`Collection ${collection.id} must be a StaticCollection here, not a key/value object`);
              }
              const val = collection.getReferenceValue(value);

              if (!val) {
                console.error("Could not find reference for value", value, "for", node.alias, "in collection", collectionId);
              }

              tile.data.set(nodeid, val ? val.id : null);

              if (!tile || !val) {
                return null;
              }
              const str = new ReferenceValueViewModel(val);

              return str;
            });
          } else {
            throw Error(
              `Set references using values from collections, not strings: ${value}`,
            );
          }
        } else if (Array.isArray(value) && value.length > 0 && "labels" in value[0]) {
          // Handle array of pre-formatted reference values from business data
          // For now, just use the first value
          const ref = new StaticReference(value[0]);
          tile.data.set(nodeid, ref);
          return new ReferenceValueViewModel(ref);
        } else if (typeof value === "object" && value !== null && "labels" in value) {
          // Handle single pre-formatted reference value from business data
          const ref = new StaticReference(value);
          tile.data.set(nodeid, ref);
          return new ReferenceValueViewModel(ref);
        } else {
          throw Error("Could not set reference from this data: " + JSON.stringify(value));
        }

        if (!(value instanceof Promise)) {
          if (!value) {
            console.error("Could not find reference for value", value, "for", node.alias, "in collection", collectionId);
          }

          tile.data.set(nodeid, value || null);
        }
      }
    }

    if (!tile || !value) {
      return null;
    }
    const str = new ReferenceValueViewModel(value);
    return str;
  }

  async __asTileData() {
    const value = await this._ref;
    return value ?? null;
  }
}

class ReferenceListViewModel extends Array implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)
  _value: Promise<(ReferenceValueViewModel | null)[]> | null = null;

  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => (v ? v.forJson() : null)) : null;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    _cacheEntry: object | null = null
  ): Promise<ReferenceListViewModel> {
    const nodeid = node.nodeid;
    let val: (ReferenceValueViewModel | Promise<ReferenceValueViewModel | null> | null)[] = [];
    if (!tile.data.has(nodeid)) {
      tile.data.set(nodeid, null);
    }
    if (value !== null) {
      tile.data.set(nodeid, []);
      if (!Array.isArray(value)) {
        throw Error(
          `Cannot set an (entire) reference list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`,
        );
      }
      val = value.map((c, _i) => {
        if (c instanceof ReferenceValueViewModel) {
          return c;
        }
        return ReferenceValueViewModel.__create(tile, node, c, {});
      });
      Promise.all(val).then((vals) => {
        Promise.all(
          vals.map(async (c) => {
            const v = await c;
            return v ? (await v.getValue()) : null;
          })
        ).then((ids) => {
          tile.data.set(nodeid, ids);
        });
      });
      value = val;
    } else {
      value = [];
    }

    const str = new ReferenceListViewModel(...value);
    return str;
  }

  async __asTileData() {
    return this._value ? await this._value : null;
  }
}

class ReferenceMergedDataType {
  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    _cacheEntry: object
  ): Promise<ReferenceValueViewModel | ReferenceListViewModel | null> {
    const config = nodeConfig.nodeConfigManager.retrieve(node);
    if (config && config.multiValue) {
      return ReferenceListViewModel.__create(tile, node, value, _cacheEntry);
    }
    return ReferenceValueViewModel.__create(tile, node, value, _cacheEntry);
  }
}

viewModels.CUSTOM_DATATYPES.set("reference", ReferenceMergedDataType);
