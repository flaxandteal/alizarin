import { StaticCollection, StaticConcept, SkosNodeType } from './static-types';
/**
 * Options for adding a concept to a collection
 */
export interface AddConceptOptions {
    /** The label value (string) or multilingual labels */
    label: string | {
        [lang: string]: string;
    };
    /** Optional parent concept ID to add this as a child */
    parentId?: string;
    /** Optional source/identifier URI */
    source?: string;
    /** Optional sort order */
    sortOrder?: number;
    /** Optional explicit ID (otherwise generated from label) */
    id?: string;
    /** Optional URI for the concept */
    uri?: string;
}
/**
 * Result of adding a concept
 */
export interface AddConceptResult {
    concept: StaticConcept;
    id: string;
}
/**
 * Mutator class for StaticCollection that provides methods for adding,
 * modifying, and removing concepts including nested children.
 */
export declare class CollectionMutator {
    private collection;
    constructor(collection: StaticCollection);
    /**
     * Get the underlying collection
     */
    getCollection(): StaticCollection;
    /**
     * Create a new empty collection with a mutator.
     *
     * @param name - Collection name (string or multilingual object)
     * @param options - Optional settings
     * @param options.collectionId - Explicit ID (otherwise generated from name)
     * @param options.nodeType - 'ConceptScheme' (default) or 'Collection' for Arches-compatible
     * @param options.uri - Optional URI for the collection
     *
     * Both types support hierarchical concepts via narrower/broader relationships.
     * The difference is how membership is expressed in RDF:
     * - ConceptScheme: uses skos:inScheme on concepts
     * - Collection: uses skos:member to list all concepts (including children)
     *
     * @example
     * // Create a hierarchical ConceptScheme (default)
     * const scheme = CollectionMutator.createEmpty("Categories");
     * scheme.addConcept({ label: "Parent" });
     * scheme.addChildConcept(parentId, { label: "Child" }); // narrower/broader
     *
     * @example
     * // Create an Arches-compatible Collection with hierarchy
     * const collection = CollectionMutator.createEmpty("My Collection", { nodeType: 'Collection' });
     * const parent = collection.addConcept({ label: "Parent" });
     * collection.addChildConcept(parent.id, { label: "Child" }); // narrower/broader, all listed as members
     */
    static createEmpty(name: string | {
        [lang: string]: string;
    }, options?: string | {
        collectionId?: string;
        nodeType?: SkosNodeType;
        uri?: string;
    }): CollectionMutator;
    /**
     * Create a new Arches-compatible Collection.
     * Convenience method equivalent to createEmpty(name, { nodeType: 'Collection' })
     * Collections can have hierarchical concepts (narrower/broader) but list all
     * concepts (including children) as members via skos:member.
     */
    static createArchesCollection(name: string | {
        [lang: string]: string;
    }, options?: {
        collectionId?: string;
        uri?: string;
    }): CollectionMutator;
    /**
     * Add a top-level concept to the collection.
     * Use parentId to create hierarchy (narrower/broader relationships on concepts).
     */
    addConcept(options: AddConceptOptions): AddConceptResult;
    /**
     * Add a top-level member concept (alias for addConcept).
     * Useful when working with Collections to make intent clear.
     * Use addChildConcept() to create hierarchical concepts.
     */
    addMember(options: Omit<AddConceptOptions, 'parentId'>): AddConceptResult;
    /**
     * Add a child concept under an existing concept.
     * For both ConceptScheme and Collection types, this creates a narrower/broader hierarchy
     * on the concepts. Collections list all concepts (including children) as members.
     */
    addChildConcept(parentId: string, options: Omit<AddConceptOptions, 'parentId'>): AddConceptResult;
    /**
     * Add multiple concepts at once, supporting nested structure
     * @param concepts Array of concepts with optional children
     * @param parentId Optional parent ID to add all concepts under
     */
    addConcepts(concepts: Array<AddConceptOptions & {
        children?: Array<AddConceptOptions & {
            children?: any[];
        }>;
    }>, parentId?: string): AddConceptResult[];
    /**
     * Remove a concept by ID (and all its children)
     */
    removeConcept(conceptId: string): boolean;
    /**
     * Move a concept to a new parent (or to top level if parentId is null)
     */
    moveConcept(conceptId: string, newParentId: string | null): boolean;
    /**
     * Update a concept's properties
     */
    updateConcept(conceptId: string, updates: Partial<Omit<AddConceptOptions, 'id' | 'parentId'>>): boolean;
    /**
     * Get a concept by ID
     */
    getConcept(conceptId: string): StaticConcept | null;
    /**
     * Get all concept IDs in the collection (including nested)
     */
    getAllConceptIds(): string[];
    /**
     * Get the path from root to a concept (list of concept IDs)
     */
    getConceptPath(conceptId: string): string[] | null;
    /**
     * Create a concept object from options
     */
    private _createConcept;
    /**
     * Find a concept by ID in the entire tree
     */
    private _findConcept;
    /**
     * Index a concept and its children in __allConcepts and __values
     */
    private _indexConcept;
    /**
     * Remove a concept and its children from indexes
     */
    private _unindexConcept;
    /**
     * Remove a concept from a parent's children list
     */
    private _removeFromChildren;
}
