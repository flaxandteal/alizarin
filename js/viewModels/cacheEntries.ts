import { IStringKeyedObject } from "../interfaces";

export class ConceptListCacheEntry implements IStringKeyedObject {
  [key: string]: any;
  datatype: string = 'concept-list';
  _: ConceptValueCacheEntry[];
  meta: {[key: string]: any};

  constructor({meta, _}: {meta: IStringKeyedObject | undefined, _: ConceptValueCacheEntry[]}) {
    this._ = _.map(instance => {
      if (instance instanceof ConceptValueCacheEntry) {
        return instance;
      } else if (instance) {
        return new ConceptValueCacheEntry(instance);
      }
      return null;
    }).filter(cvce => cvce !== null);
    this.meta = meta || {};
  }
}

export class ConceptValueCacheEntry implements IStringKeyedObject {
  [key: string]: any
  datatype: string = 'concept';
  id: string;
  value: string;
  conceptId: string | null;
  meta: {[key: string]: any};

  constructor({meta, id, value, conceptId}: {meta: IStringKeyedObject | undefined, id: string, value: string, conceptId: string | null}) {
    this.id = id;
    this.value = value;
    this.conceptId = conceptId;
    this.meta = meta || {};
  }
}

export class ResourceInstanceListCacheEntry implements IStringKeyedObject {
  [key: string]: any;
  datatype: string = 'resource-instance-list';
  _: ResourceInstanceCacheEntry[];
  meta: {[key: string]: any};

  constructor(input: {meta?: IStringKeyedObject, _?: ResourceInstanceCacheEntry[]} | {id: string, type: string, graphId: string, title?: string | null, meta?: IStringKeyedObject}) {
    // Handle both list format {meta, _} and single entry format {id, type, graphId, title}
    // This allows wrapping a single resource-instance entry as a list
    let entries: any[];
    if ('_' in input && Array.isArray(input._)) {
      // List format: {meta, _: [...]}
      entries = input._;
      this.meta = input.meta || {};
    } else if ('id' in input) {
      // Single entry format: {id, type, graphId, title} - wrap in array
      entries = [input];
      this.meta = (input as any).meta || {};
    } else {
      // Unknown format - use empty array
      entries = [];
      this.meta = {};
    }

    this._ = entries.map(instance => {
      if (instance instanceof ResourceInstanceCacheEntry) {
        return instance;
      }
      return new ResourceInstanceCacheEntry(instance);
    });
  }
}

export class ResourceInstanceCacheEntry implements IStringKeyedObject {
  [key: string]: any
  datatype: string = 'resource-instance';
  id: string;
  type: string;
  graphId: string;
  title: string | null;
  meta: {[key: string]: any};

  constructor({meta, id, type, graphId, title}: {meta: IStringKeyedObject | undefined, id: string, type: string, graphId: string, title: string | null}) {
    this.id = id;
    this.type = type;
    this.graphId = graphId;
    this.meta = meta || {};
    this.title = this.meta.title || title;
  }
}
