import { IViewModel, IPseudo, GetMeta } from "../interfaces";
import { StaticTile, StaticNode, StaticValue, StaticConcept } from "../static-types";
import { RDM } from "../rdm";
import { ConceptValueCacheEntry } from "./cacheEntries";

export class ConceptValueViewModel extends String implements IViewModel {
  _: IViewModel | Promise<IViewModel> | undefined = undefined;
  __parentPseudo: IPseudo | undefined;

  describeField = () => (this.__parentPseudo ? this.__parentPseudo.describeField() : null)
  describeFieldGroup = () => (this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null)

  _value: StaticValue | Promise<StaticValue>;

  constructor(value: StaticValue) {
    super(value.value);
    this._value = value;
  }

  async forJson(): Promise<StaticValue> {
    return this._value;
  }

  async __forJsonCache(getMeta: GetMeta): Promise<ConceptValueCacheEntry> {
    const value = await this._value;
    return new ConceptValueCacheEntry({
      meta: getMeta ? await getMeta(this) : undefined,
      id: value.id,
      value: value.value,
      conceptId: value.__conceptId ?? null
    });
  }

  getValue(): StaticValue | Promise<StaticValue> {
    return this._value;
  }

  static async __create(
    tile: StaticTile,
    node: StaticNode,
    value: any,
    cacheEntry: ConceptValueCacheEntry | null
  ): Promise<ConceptValueViewModel | null> {
    const nodeid = node.nodeid;
    const collectionId = node.config?.rdmCollection;
    if (!collectionId) {
      throw new Error(`Node ${node.alias} (${node.nodeid}) missing rdmCollection in config`);
    }
    let val: StaticValue | null = value;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (value instanceof StaticConcept) {
          if (value.getPrefLabel) {
            val = value.getPrefLabel();
          } else {
            throw new Error("Recognizing value as StaticConcept, but no getPrefLabel member");
          }
        }
        if (!value) {
          val = null;
        } else if (value instanceof StaticValue) {
          // No change needed.
        } else if (value instanceof Promise) {
          return value.then((value) => {
            return ConceptValueViewModel.__create(tile, node, value, cacheEntry);
          });
        } else if (typeof value == "string") {
          if (
            /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(
              value,
            )
          ) {
            if (cacheEntry) {
              val = new StaticValue({
                id: cacheEntry.id,
                value: cacheEntry.value,
                __concept: null,
                __conceptId: cacheEntry.conceptId,
              }, cacheEntry.conceptId);
              return new ConceptValueViewModel(val);
            } else {
              const collection = RDM.retrieveCollection(collectionId);
              return collection.then((collection) => {
                if (!collection.getConceptValue) {
                  throw new Error(`Collection ${collection.id} must be a StaticCollection here, not a key/value object`);
                }
                const val = collection.getConceptValue(value);

                if (!val) {
                  console.error("Could not find concept for value", value, "for", node.alias, "in collection", collectionId);
                }

                tile.data.set(nodeid, val ? val.id : null);

                if (!tile || !val) {
                  return null;
                }
                const str = new ConceptValueViewModel(val);

                return str;
              });
            }
          } else {
            throw new Error(
              `Set concepts using values from collections, not strings: ${value}`,
            );
          }
        } else {
          throw new Error("Could not set concept from this data");
        }

        if (!(val instanceof Promise)) {
          if (!val) {
            console.error("Could not find concept for value", value, "for", node.alias, "in collection", collectionId);
          }

          tile.data.set(nodeid, val ? val.id : null);
        }
      }
    }

    if (!tile || !val) {
      return null;
    }
    const str = new ConceptValueViewModel(val);
    return str;
  }

  async __asTileData() {
    const value = await this._value;
    return value ? value.id : null;
  }
}
