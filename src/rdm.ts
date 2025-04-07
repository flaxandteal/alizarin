import { StaticCollection, StaticConcept } from "./static-types";
import { ArchesClient, archesClient } from "./client";

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
}

const RDM = new ReferenceDataManager(archesClient);

export { StaticCollection, ReferenceDataManager, RDM };
