import { StaticCollection, StaticGraph } from "./static-types";
import { ArchesClient } from "./client";
import { isValidUuid } from "../pkg/alizarin";
export { isValidUuid };
/**
 * Register an additional datatype for label resolution.
 * Extensions should call this to include their datatypes in resolveLabels.
 * E.g. CLM registers "reference" so reference labels are resolved.
 */
declare function registerResolvableDatatype(datatype: string): void;
/**
 * Unregister a previously registered resolvable datatype.
 */
declare function unregisterResolvableDatatype(datatype: string): void;
interface ResolveLabelsOptions {
    /** If true, throw errors for unresolved labels. Default: false */
    strict?: boolean;
    /** Additional datatypes to resolve beyond the defaults and any extension-registered types */
    additionalDatatypes?: string[];
    /** Additional config keys to check for collection IDs */
    additionalConfigKeys?: string[];
}
declare class ReferenceDataManager {
    archesClient: ArchesClient;
    collections: Map<string, Promise<StaticCollection>>;
    constructor(archesClient: ArchesClient);
    retrieveCollection(id: string): Promise<StaticCollection>;
    /**
     * Add a pre-loaded collection to the cache.
     */
    addCollection(collection: StaticCollection): void;
    /**
     * Check if a collection is cached.
     */
    hasCollection(id: string): boolean;
    /**
     * Clear a specific collection from cache.
     * Also clears from the Rust RDM cache if available.
     */
    clearCollection(id: string): void;
    /**
     * Clear all cached collections.
     * Also clears the Rust RDM cache if available.
     */
    clear(): void;
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
    resolveLabels<T extends object>(tree: T, graph: StaticGraph, options?: ResolveLabelsOptions): Promise<T>;
}
declare const RDM: ReferenceDataManager;
export { StaticCollection, ReferenceDataManager, RDM, registerResolvableDatatype, unregisterResolvableDatatype };
export type { ResolveLabelsOptions };
