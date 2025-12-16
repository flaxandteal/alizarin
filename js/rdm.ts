import { StaticCollection, StaticGraph } from "./static-types";
import { ArchesClient, archesClient } from "./client";
import {
  buildAliasToCollectionMap,
  findNeededCollections,
  resolveLabelsWithLookup,
  isValidUuid,
  getDefaultResolvableDatatypes,
  getDefaultConfigKeys,
} from "../pkg/alizarin";

// Re-export WASM utilities
export { isValidUuid };

// Lazy-loaded defaults from Rust (single source of truth)
// Deferred to avoid calling WASM functions before initialization
let _labelResolvableDatatypes: Set<string> | null = null;
let _collectionConfigKeys: string[] | null = null;

function getLabelResolvableDatatypes(): Set<string> {
  if (_labelResolvableDatatypes === null) {
    _labelResolvableDatatypes = new Set(getDefaultResolvableDatatypes());
  }
  return _labelResolvableDatatypes;
}

function getCollectionConfigKeys(): string[] {
  if (_collectionConfigKeys === null) {
    _collectionConfigKeys = getDefaultConfigKeys();
  }
  return _collectionConfigKeys;
}

interface ResolveLabelsOptions {
  /** If true, throw errors for unresolved labels. Default: false */
  strict?: boolean;
  /** Additional datatypes to resolve (beyond concept/concept-list/reference) */
  additionalDatatypes?: string[];
  /** Additional config keys to check for collection IDs */
  additionalConfigKeys?: string[];
}

class ReferenceDataManager {
  archesClient: ArchesClient;
  collections: Map<string, Promise<StaticCollection>>;

  constructor(archesClient: ArchesClient) {
    this.archesClient = archesClient;
    this.collections = new Map<string, Promise<StaticCollection>>();
  }

  retrieveCollection(id: string): Promise<StaticCollection> {
    let collection = this.collections.get(id);
    if (collection !== undefined) {
      return collection;
    }
    collection = this.archesClient
      .getCollection(id)
      .then((jsonData) => new StaticCollection(jsonData));
    this.collections.set(id, collection);
    return collection;
  }

  /**
   * Add a pre-loaded collection to the cache.
   */
  addCollection(collection: StaticCollection): void {
    this.collections.set(collection.id, Promise.resolve(collection));
  }

  /**
   * Check if a collection is cached.
   */
  hasCollection(id: string): boolean {
    return this.collections.has(id);
  }

  /**
   * Clear a specific collection from cache.
   */
  clearCollection(id: string): void {
    this.collections.delete(id);
  }

  /**
   * Clear all cached collections.
   */
  clear(): void {
    this.collections.clear();
  }

  /**
   * Resolve label strings to UUIDs in a JSON tree.
   *
   * Uses centralized Rust/WASM implementation for tree traversal and resolution.
   * Only loads collections that are actually needed (lazy loading).
   *
   * @param tree - The data tree to resolve
   * @param graph - The graph definition (for node configs)
   * @param options - Resolution options
   * @returns The tree with labels resolved to UUIDs
   *
   * @example
   * ```typescript
   * const tree = { status: ["Active", "Pending"] };
   * const resolved = await RDM.resolveLabels(tree, graph);
   * // resolved = { status: ["uuid-1", "uuid-2"] }
   * ```
   */
  async resolveLabels<T extends object>(
    tree: T,
    graph: StaticGraph,
    options: ResolveLabelsOptions = {}
  ): Promise<T> {
    const { strict = false, additionalDatatypes = [], additionalConfigKeys = [] } = options;

    // Build datatypes and config keys arrays (lazy-loaded from Rust)
    const resolvableDatatypes = [
      ...getLabelResolvableDatatypes(),
      ...additionalDatatypes,
    ];
    const configKeys = [...getCollectionConfigKeys(), ...additionalConfigKeys];

    // Serialize tree and graph for WASM
    const treeJson = JSON.stringify(tree);
    const graphJson = JSON.stringify({ graph: [graph] });

    // Use Rust to build alias -> collection mapping
    // WASM returns a Map, convert to plain object for consistency
    const aliasToCollectionMap = buildAliasToCollectionMap(
      graphJson,
      resolvableDatatypes,
      configKeys
    ) as Map<string, string>;
    const aliasToCollection: Record<string, string> = Object.fromEntries(aliasToCollectionMap);

    if (Object.keys(aliasToCollection).length === 0) {
      // No resolvable nodes, return tree unchanged
      return tree;
    }

    // Use Rust to find which collections are actually needed
    const neededCollectionIds = findNeededCollections(treeJson, aliasToCollection);

    // Load needed collections (in parallel)
    const collectionPromises = neededCollectionIds.map((id: string) =>
      this.retrieveCollection(id).then((coll) => [id, coll] as const)
    );
    const loadedCollections = new Map(await Promise.all(collectionPromises));

    // Build lookup table for Rust: Map<collectionId, Map<lowercase_label, conceptId>>
    const lookupTable: Record<string, Record<string, string>> = {};
    for (const [collectionId, collection] of loadedCollections) {
      const labelToId: Record<string, string> = {};
      // Iterate through all values in the collection
      for (const value of Object.values(collection.__values || {})) {
        if (value.value && value.__concept) {
          labelToId[value.value.toLowerCase()] = value.__concept.id;
        }
      }
      lookupTable[collectionId] = labelToId;
    }

    // Use Rust to resolve labels
    const resolvedJson = resolveLabelsWithLookup(
      treeJson,
      aliasToCollection,
      lookupTable,
      strict
    );

    return JSON.parse(resolvedJson) as T;
  }
}

const RDM = new ReferenceDataManager(archesClient);

export { StaticCollection, ReferenceDataManager, RDM };
export type { ResolveLabelsOptions };
