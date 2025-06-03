import { ArchesClient } from './client.ts';
import { StaticResource, StaticResourceMetadata } from './static-types';
declare class StaticStore {
    archesClient: ArchesClient;
    cache: Map<string, StaticResource | StaticResourceMetadata>;
    cacheMetadataOnly: boolean;
    constructor(archesClient: ArchesClient, cacheMetadataOnly?: boolean);
    getMeta(id: string, onlyIfCached?: boolean): Promise<StaticResourceMetadata | null>;
    loadAll(graphId: string, limit?: number | undefined): AsyncIterable<StaticResource>;
    loadOne(id: string): Promise<StaticResource>;
}
declare const staticStore: StaticStore;
export { staticStore };
