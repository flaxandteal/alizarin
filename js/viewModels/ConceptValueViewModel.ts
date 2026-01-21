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
  _collectionId: string | null = null;

  constructor(value: StaticValue, collectionId?: string) {
    super(value.value);
    this._value = value;
    this._collectionId = collectionId ?? null;
  }

  /**
   * Get the parent concept value, if this concept has a parent in the hierarchy.
   * @returns A new ConceptValueViewModel for the parent, or null if no parent
   * @throws Error if the collection doesn't support hierarchy lookups
   */
  async parent(): Promise<ConceptValueViewModel | null> {
    const value = await this._value;
    const conceptId = value.__conceptId;
    if (!conceptId || !this._collectionId) {
      return null;
    }

    const collection = await RDM.retrieveCollection(this._collectionId);
    if (!collection.getParentId) {
      throw new Error(
        `Collection ${this._collectionId} does not support hierarchy lookups. ` +
        'Ensure WASM is initialized and the collection is a StaticCollection.'
      );
    }

    const parentId = collection.getParentId(conceptId);
    if (!parentId) {
      return null; // Top-level concept
    }

    const parentConcept = collection.__allConcepts[parentId];
    if (!parentConcept?.getPrefLabel) {
      return null;
    }

    const parentValue = parentConcept.getPrefLabel();
    return new ConceptValueViewModel(parentValue, this._collectionId);
  }

  /**
   * Get all ancestor concept values, from immediate parent to root.
   * @returns Array of ConceptValueViewModels for ancestors
   */
  async ancestors(): Promise<ConceptValueViewModel[]> {
    const result: ConceptValueViewModel[] = [];
    let current: ConceptValueViewModel | null = this;

    while ((current = await current.parent()) !== null) {
      result.push(current);
    }

    return result;
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
              return new ConceptValueViewModel(val, collectionId);
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
                const str = new ConceptValueViewModel(val, collectionId);

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
    const str = new ConceptValueViewModel(val, collectionId);
    return str;
  }

  async __asTileData() {
    const value = await this._value;
    return value ? value.id : null;
  }
}
