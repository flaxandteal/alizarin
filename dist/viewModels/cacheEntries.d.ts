import { IStringKeyedObject } from "../interfaces";
export declare class ConceptListCacheEntry implements IStringKeyedObject {
    [key: string]: any;
    datatype: string;
    _: ConceptValueCacheEntry[];
    meta: {
        [key: string]: any;
    };
    constructor({ meta, _ }: {
        meta: IStringKeyedObject | undefined;
        _: ConceptValueCacheEntry[];
    });
}
export declare class ConceptValueCacheEntry implements IStringKeyedObject {
    [key: string]: any;
    datatype: string;
    id: string;
    value: string;
    conceptId: string | null;
    meta: {
        [key: string]: any;
    };
    constructor({ meta, id, value, conceptId }: {
        meta: IStringKeyedObject | undefined;
        id: string;
        value: string;
        conceptId: string | null;
    });
}
export declare class ResourceInstanceListCacheEntry implements IStringKeyedObject {
    [key: string]: any;
    datatype: string;
    _: ResourceInstanceCacheEntry[];
    meta: {
        [key: string]: any;
    };
    constructor(input: {
        meta?: IStringKeyedObject;
        _?: ResourceInstanceCacheEntry[];
    } | {
        id: string;
        type: string;
        graphId: string;
        title?: string | null;
        meta?: IStringKeyedObject;
    });
}
export interface ResourceDescriptors {
    name?: string | null;
    description?: string | null;
    map_popup?: string | null;
    slug?: string | null;
}
export declare class ResourceInstanceCacheEntry implements IStringKeyedObject {
    [key: string]: any;
    datatype: string;
    id: string;
    type: string;
    graphId: string;
    title: string | null;
    descriptors: ResourceDescriptors | null;
    meta: {
        [key: string]: any;
    };
    constructor({ meta, id, type, graphId, title, descriptors }: {
        meta: IStringKeyedObject | undefined;
        id: string;
        type: string;
        graphId: string;
        title: string | null;
        descriptors?: ResourceDescriptors | null;
    });
}
