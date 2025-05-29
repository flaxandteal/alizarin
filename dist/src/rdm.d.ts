import { StaticCollection } from './static-types';
import { ArchesClient } from './client';
declare class ReferenceDataManager {
    archesClient: ArchesClient;
    collections: Map<string, Promise<StaticCollection>>;
    constructor(archesClient: ArchesClient);
    retrieveCollection(id: string): Promise<StaticCollection>;
}
declare const RDM: ReferenceDataManager;
export { StaticCollection, ReferenceDataManager, RDM };
