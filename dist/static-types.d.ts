import { StaticGraphMeta, StaticNode, StaticTranslatableString, StaticNodegroup, StaticConstraint, StaticCard, StaticCardsXNodesXWidgets, StaticFunctionsXGraphs, StaticPublication, StaticTile, StaticGraph, StaticEdge, StaticResourceDescriptors, StaticResourceMetadata, StaticResourceSummary, StaticResourceReference, StaticResource, StaticResourceRegistry } from '../pkg/alizarin';
interface IStaticDescriptorConfig {
    descriptor_types: {
        nodegroup_id: string;
        string_template: string;
    }[];
}
declare function createStaticGraph(props: {
    author?: string;
    color?: string | null;
    config?: object;
    deploymentdate?: null | string;
    deploymentfile?: null | string;
    description?: string | StaticTranslatableString;
    graphid?: string;
    iconclass?: string;
    is_editable?: boolean | null;
    isresource?: boolean;
    jsonldcontext?: string | {
        [key: string]: any;
    } | null;
    name: string | StaticTranslatableString;
    ontology_id?: string | null;
    relatable_resource_model_ids?: Array<string>;
    resource_2_resource_constraints?: Array<any> | null;
    slug?: string | null;
    subtitle?: string | StaticTranslatableString;
    template_id?: string;
    version?: string;
}, published?: boolean): StaticGraph;
declare class StaticValue {
    id: string;
    value: string;
    __concept?: StaticConcept | null;
    __conceptId?: string | null;
    constructor(jsonData: StaticValue, concept?: StaticConcept | string | null);
    toJSON(): {
        id: string;
        value: string;
    };
    toString(): string;
    static create(referent: string | StaticConcept, valueType: string, value: string, language?: string): StaticValue;
}
declare class StaticConcept {
    id: string;
    prefLabels: {
        [lang: string]: StaticValue;
    };
    source: string | null;
    sortOrder: number | null;
    children: StaticConcept[] | null;
    constructor(jsonData: StaticConcept);
    getPrefLabel?(): StaticValue;
    toString(): any;
    static fromValue(conceptScheme: StaticConcept | null, value: string | StaticValue | {
        [lang: string]: StaticValue;
    }, children?: (string | StaticValue | StaticConcept)[], config?: {
        baseLanguage?: string;
        source?: string | null;
        sortOrder?: number | null;
    }): StaticConcept;
}
/**
 * Type of SKOS grouping structure.
 * - ConceptScheme: Uses narrower/broader hierarchy (default)
 * - Collection: Uses flat member relationships (Arches-compatible)
 */
export type SkosNodeType = 'ConceptScheme' | 'Collection';
declare class StaticCollection {
    id: string;
    uri?: string;
    prefLabels: {
        [lang: string]: StaticValue;
    };
    altLabels?: {
        [lang: string]: StaticValue[];
    };
    scopeNotes?: {
        [lang: string]: StaticValue;
    };
    nodeType: SkosNodeType;
    concepts: {
        [conceptId: string]: StaticConcept;
    };
    __allConcepts: {
        [conceptId: string]: StaticConcept;
    };
    __values: {
        [valueId: string]: StaticValue;
    };
    static fromConceptScheme(props: {
        collectionid?: string;
        name?: string | {
            [lang: string]: StaticValue;
        } | StaticValue;
        conceptScheme: StaticConcept;
        nodeType?: SkosNodeType;
    }): StaticCollection;
    /**
     * Create a new StaticCollection.
     * @param props.nodeType - 'ConceptScheme' (default) for hierarchical, 'Collection' for flat Arches-compatible
     */
    static create(props: {
        collectionid?: string;
        uri?: string;
        name: string | {
            [lang: string]: StaticValue;
        } | StaticValue;
        concepts: StaticConcept[] | {
            [conceptId: string]: StaticConcept;
        };
        nodeType?: SkosNodeType;
    }): StaticCollection;
    constructor(jsonData: StaticCollection);
    getConceptValue?(valueId: string): StaticValue;
    getConceptByValue?(label: string): StaticConcept;
    private _syncedToRustCache;
    /**
     * Ensure this collection is synced to the Rust RDM cache.
     * This is called automatically by hierarchy methods, but can be called
     * explicitly to pre-sync collections before label resolution.
     * @throws Error if WASM is not initialized
     */
    ensureInCache?(): void;
    /**
     * Get the parent concept ID for a given concept.
     * Uses the Rust RDM cache for hierarchy lookups.
     * @returns The parent concept ID, or null if no parent (top-level concept)
     * @throws Error if WASM is not initialized
     */
    getParentId?(conceptId: string): string | null;
    /**
     * Get the parent concept for a given concept.
     * @returns The parent concept, or null if no parent
     */
    getParent?(conceptId: string): StaticConcept | null;
    /**
     * Get all ancestor concepts for a given concept (parent, grandparent, etc).
     * Returns in order from immediate parent to root.
     */
    getAncestors?(conceptId: string): StaticConcept[];
    /**
     * Ensure this collection's data is in the Rust RDM cache.
     * Called lazily when hierarchy lookups are needed.
     * @internal
     */
    private _ensureInRustCache;
    /**
     * Convert collection concepts to the format expected by Rust cache.
     * @internal
     */
    private _toRustCacheFormat;
    toString(): string;
}
declare class StaticDomainValue {
    id: string;
    selected: boolean;
    text: {
        [lang: string]: string;
    };
    constructor(jsonData: StaticDomainValue);
    toString(): string;
    lang(lang: string): string | undefined;
    forJson(): Promise<{
        id: string;
        selected: boolean;
        text: {
            [lang: string]: string;
        };
    }>;
}
export { StaticValue, StaticTile, StaticGraph, createStaticGraph, StaticResource, StaticResourceSummary, StaticResourceMetadata, StaticResourceRegistry, StaticNode, StaticNodegroup, StaticEdge, StaticCard, StaticCardsXNodesXWidgets, StaticCollection, StaticConcept, StaticDomainValue, StaticResourceReference, StaticGraphMeta, StaticFunctionsXGraphs, StaticResourceDescriptors, StaticTranslatableString, StaticConstraint, StaticPublication, type IStaticDescriptorConfig, type SkosNodeType };
