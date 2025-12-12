import { generateUuidv5 } from './utils';
import { getCurrentLanguage } from './utils';
import { StaticCollection, StaticConcept, StaticValue } from './static-types';

/**
 * Options for adding a concept to a collection
 */
export interface AddConceptOptions {
  /** The label value (string) or multilingual labels */
  label: string | { [lang: string]: string };
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
export class CollectionMutator {
  private collection: StaticCollection;

  constructor(collection: StaticCollection) {
    this.collection = collection;
  }

  /**
   * Get the underlying collection
   */
  getCollection(): StaticCollection {
    return this.collection;
  }

  /**
   * Create a new empty collection with a mutator
   */
  static createEmpty(name: string | { [lang: string]: string }, collectionId?: string): CollectionMutator {
    const lang = getCurrentLanguage();
    let prefLabels: { [lang: string]: StaticValue };
    let nameValue: string;

    if (typeof name === 'string') {
      nameValue = name;
      prefLabels = {
        [lang]: StaticValue.create('', 'prefLabel', name)
      };
    } else {
      // Multilingual name
      nameValue = name[lang] || Object.values(name)[0];
      prefLabels = {};
      for (const [l, v] of Object.entries(name)) {
        prefLabels[l] = StaticValue.create('', 'prefLabel', v);
      }
    }

    const id = collectionId || generateUuidv5(['collection'], nameValue);

    const collection = new StaticCollection({
      id,
      prefLabels,
      concepts: {},
      __allConcepts: {},
      __values: {}
    });

    return new CollectionMutator(collection);
  }

  /**
   * Add a top-level concept to the collection
   */
  addConcept(options: AddConceptOptions): AddConceptResult {
    if (options.parentId) {
      return this.addChildConcept(options.parentId, options);
    }

    const concept = this._createConcept(options);

    // Add to top-level concepts
    this.collection.concepts[concept.id] = concept;

    // Update internal indexes
    this._indexConcept(concept);

    return { concept, id: concept.id };
  }

  /**
   * Add a child concept under an existing concept
   */
  addChildConcept(parentId: string, options: Omit<AddConceptOptions, 'parentId'>): AddConceptResult {
    const parent = this._findConcept(parentId);
    if (!parent) {
      throw new Error(`Parent concept with ID ${parentId} not found`);
    }

    const concept = this._createConcept(options);

    // Add to parent's children
    if (!parent.children) {
      parent.children = [];
    }
    parent.children.push(concept);

    // Sort children by sortOrder if present
    parent.children.sort((a, b) => (a.sortOrder ?? 999) - (b.sortOrder ?? 999));

    // Update internal indexes
    this._indexConcept(concept);

    return { concept, id: concept.id };
  }

  /**
   * Add multiple concepts at once, supporting nested structure
   * @param concepts Array of concepts with optional children
   * @param parentId Optional parent ID to add all concepts under
   */
  addConcepts(
    concepts: Array<AddConceptOptions & { children?: Array<AddConceptOptions & { children?: any[] }> }>,
    parentId?: string
  ): AddConceptResult[] {
    const results: AddConceptResult[] = [];

    for (const conceptData of concepts) {
      const { children, ...conceptOptions } = conceptData;

      // Add the concept
      const result = parentId
        ? this.addChildConcept(parentId, conceptOptions)
        : this.addConcept(conceptOptions);

      results.push(result);

      // Recursively add children
      if (children && children.length > 0) {
        const childResults = this.addConcepts(children, result.id);
        results.push(...childResults);
      }
    }

    return results;
  }

  /**
   * Remove a concept by ID (and all its children)
   */
  removeConcept(conceptId: string): boolean {
    // Check if it's a top-level concept
    if (this.collection.concepts[conceptId]) {
      const concept = this.collection.concepts[conceptId];
      this._unindexConcept(concept);
      delete this.collection.concepts[conceptId];
      return true;
    }

    // Search in children
    for (const topConcept of Object.values(this.collection.concepts)) {
      if (this._removeFromChildren(topConcept, conceptId)) {
        return true;
      }
    }

    return false;
  }

  /**
   * Move a concept to a new parent (or to top level if parentId is null)
   */
  moveConcept(conceptId: string, newParentId: string | null): boolean {
    const concept = this._findConcept(conceptId);
    if (!concept) {
      return false;
    }

    // Remove from current location
    this.removeConcept(conceptId);

    // Re-index (was unindexed during removal)
    this._indexConcept(concept);

    if (newParentId === null) {
      // Move to top level
      this.collection.concepts[concept.id] = concept;
    } else {
      // Move under new parent
      const newParent = this._findConcept(newParentId);
      if (!newParent) {
        // Revert - add back to top level
        this.collection.concepts[concept.id] = concept;
        return false;
      }
      if (!newParent.children) {
        newParent.children = [];
      }
      newParent.children.push(concept);
      newParent.children.sort((a, b) => (a.sortOrder ?? 999) - (b.sortOrder ?? 999));
    }

    return true;
  }

  /**
   * Update a concept's properties
   */
  updateConcept(conceptId: string, updates: Partial<Omit<AddConceptOptions, 'id' | 'parentId'>>): boolean {
    const concept = this._findConcept(conceptId);
    if (!concept) {
      return false;
    }

    if (updates.label !== undefined) {
      // Unindex old values
      for (const value of Object.values(concept.prefLabels)) {
        delete this.collection.__values[value.id];
      }

      // Update labels - create plain objects that will be converted to StaticValue
      if (typeof updates.label === 'string') {
        const lang = getCurrentLanguage();
        const valueId = generateUuidv5(['value'], `${concept.id}/prefLabel/${updates.label}/${lang}`);
        const newValue = new StaticValue({ id: valueId, value: updates.label }, concept);
        concept.prefLabels = { [lang]: newValue };
      } else {
        concept.prefLabels = {};
        for (const [lang, value] of Object.entries(updates.label)) {
          const valueId = generateUuidv5(['value'], `${concept.id}/prefLabel/${value}/${lang}`);
          const newValue = new StaticValue({ id: valueId, value: value }, concept);
          concept.prefLabels[lang] = newValue;
        }
      }

      // Re-index new values
      for (const value of Object.values(concept.prefLabels)) {
        this.collection.__values[value.id] = value;
      }
    }

    if (updates.source !== undefined) {
      concept.source = updates.source;
    }

    if (updates.sortOrder !== undefined) {
      concept.sortOrder = updates.sortOrder;
    }

    if (updates.uri !== undefined) {
      // StaticConcept doesn't have uri field, but we can store in source
      concept.source = updates.uri;
    }

    return true;
  }

  /**
   * Get a concept by ID
   */
  getConcept(conceptId: string): StaticConcept | null {
    return this._findConcept(conceptId);
  }

  /**
   * Get all concept IDs in the collection (including nested)
   */
  getAllConceptIds(): string[] {
    return Object.keys(this.collection.__allConcepts);
  }

  /**
   * Get the path from root to a concept (list of concept IDs)
   */
  getConceptPath(conceptId: string): string[] | null {
    const path: string[] = [];

    const findPath = (concepts: StaticConcept[], targetId: string): boolean => {
      for (const concept of concepts) {
        if (concept.id === targetId) {
          path.push(concept.id);
          return true;
        }
        if (concept.children) {
          path.push(concept.id);
          if (findPath(concept.children, targetId)) {
            return true;
          }
          path.pop();
        }
      }
      return false;
    };

    if (findPath(Object.values(this.collection.concepts), conceptId)) {
      return path;
    }
    return null;
  }

  /**
   * Create a concept object from options
   */
  private _createConcept(options: Omit<AddConceptOptions, 'parentId'>): StaticConcept {
    const lang = getCurrentLanguage();
    let labelValue: string;

    if (typeof options.label === 'string') {
      labelValue = options.label;
    } else {
      labelValue = options.label[lang] || Object.values(options.label)[0];
    }

    // Generate ID if not provided
    const conceptId = options.id || generateUuidv5(
      ['concept'],
      `${this.collection.id}/${labelValue}`
    );

    // Create prefLabels as plain objects with id and value
    // StaticConcept constructor will convert them to StaticValue with proper __concept reference
    let prefLabels: { [lang: string]: { id: string; value: string } } = {};
    if (typeof options.label === 'string') {
      const valueId = generateUuidv5(['value'], `${conceptId}/prefLabel/${options.label}/${lang}`);
      prefLabels[lang] = { id: valueId, value: options.label };
    } else {
      for (const [l, v] of Object.entries(options.label)) {
        const valueId = generateUuidv5(['value'], `${conceptId}/prefLabel/${v}/${l}`);
        prefLabels[l] = { id: valueId, value: v };
      }
    }

    return new StaticConcept({
      id: conceptId,
      prefLabels: prefLabels as any,
      source: options.source || options.uri || null,
      sortOrder: options.sortOrder ?? null,
      children: null
    });
  }

  /**
   * Find a concept by ID in the entire tree
   */
  private _findConcept(conceptId: string): StaticConcept | null {
    return this.collection.__allConcepts[conceptId] || null;
  }

  /**
   * Index a concept and its children in __allConcepts and __values
   */
  private _indexConcept(concept: StaticConcept): void {
    this.collection.__allConcepts[concept.id] = concept;
    for (const value of Object.values(concept.prefLabels)) {
      this.collection.__values[value.id] = value;
    }
    if (concept.children) {
      for (const child of concept.children) {
        this._indexConcept(child);
      }
    }
  }

  /**
   * Remove a concept and its children from indexes
   */
  private _unindexConcept(concept: StaticConcept): void {
    delete this.collection.__allConcepts[concept.id];
    for (const value of Object.values(concept.prefLabels)) {
      delete this.collection.__values[value.id];
    }
    if (concept.children) {
      for (const child of concept.children) {
        this._unindexConcept(child);
      }
    }
  }

  /**
   * Remove a concept from a parent's children list
   */
  private _removeFromChildren(parent: StaticConcept, conceptId: string): boolean {
    if (!parent.children) {
      return false;
    }

    const index = parent.children.findIndex(c => c.id === conceptId);
    if (index !== -1) {
      const removed = parent.children[index];
      this._unindexConcept(removed);
      parent.children.splice(index, 1);
      if (parent.children.length === 0) {
        parent.children = null;
      }
      return true;
    }

    // Search deeper
    for (const child of parent.children) {
      if (this._removeFromChildren(child, conceptId)) {
        return true;
      }
    }

    return false;
  }
}

