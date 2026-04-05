import { ArchesClient } from "./client.ts";
import { StaticResource, StaticResourceSummary, StaticResourceMetadata, StaticResourceRegistry, StaticTile } from "./static-types";
/**
 * StaticStore - Thin wrapper around StaticResourceRegistry with optional archesClient fallback
 *
 * Primary mode: Resources are pre-loaded into the registry before use.
 * Fallback mode: If archesClient is set and resource not in registry, loads on-demand.
 *
 * Methods return sync results wrapped in Promises for API compatibility.
 */
declare class StaticStore {
    private _registry;
    archesClient: ArchesClient | null;
    constructor(registry?: StaticResourceRegistry);
    /**
     * Get or create the registry (lazy initialization for WASM timing)
     */
    get registry(): StaticResourceRegistry;
    /**
     * Set the registry (allows replacing with a pre-populated registry)
     */
    set registry(registry: StaticResourceRegistry);
    /**
     * Get metadata for a resource from the registry (or load via archesClient if available).
     */
    getMeta(id: string, onlyIfCached?: boolean): Promise<StaticResourceMetadata | null>;
    /**
     * Get all full resources for a graph.
     * First yields from registry, then loads remaining via archesClient if available.
     */
    loadAll(graphId: string, limit?: number | undefined, useCache?: boolean): AsyncIterable<StaticResource>;
    /**
     * Get a full resource from the registry (or load via archesClient if available).
     */
    loadOne(id: string): Promise<StaticResource>;
    /**
     * Get all summaries for a graph.
     */
    loadAllSummaries(graphId: string, limit?: number | undefined): AsyncIterable<StaticResourceSummary>;
    /**
     * Get tiles for a resource.
     */
    loadTiles(id: string, nodegroupId?: string | null): Promise<StaticTile[]>;
    /**
     * Get a full resource with tiles loaded.
     */
    ensureFullResource(id: string): Promise<StaticResource>;
    /**
     * Check if a resource exists in the registry.
     */
    contains(id: string): boolean;
    /**
     * Check if a full resource (with tiles) exists in the registry.
     */
    hasFull(id: string): boolean;
}
declare const staticStore: StaticStore;
export { staticStore, StaticStore };
