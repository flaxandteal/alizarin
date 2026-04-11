import { v4 as uuidv4 } from 'uuid';
import { generateUuidv5 } from './utils';
import { getCurrentLanguage, slugify } from './utils';
import { getGlobalWasmRdmCache } from './_wasm';
import {
  StaticGraphMeta,
  StaticNode,
  StaticTranslatableString,
  StaticNodegroup,
  StaticConstraint,
  StaticCard,
  StaticCardsXNodesXWidgets,
  StaticFunctionsXGraphs,
  StaticPublication,
  StaticTile,
  StaticGraph,
  StaticEdge,
  StaticResourceDescriptors,
  StaticResourceMetadata,
  StaticResourceSummary,
  StaticResourceReference,
  StaticResource,
  StaticResourceRegistry,
} from '../pkg/alizarin';

interface IStaticDescriptorConfig {
  descriptor_types: {
    nodegroup_id: string,
    string_template: string
  }[],
};

// Factory function for creating StaticGraph instances (kept in TypeScript)
function createStaticGraph(props: {
    author?: string,
    color?: string | null,
    config?: object,
    deploymentdate?: null | string,
    deploymentfile?: null | string,
    description?: string | StaticTranslatableString,
    graphid?: string,
    iconclass?: string,
    is_editable?: boolean | null,
    isresource?: boolean,
    jsonldcontext?: string | {[key: string]: any} | null,
    name: string | StaticTranslatableString,
    ontology_id?: string | string[] | null,
    relatable_resource_model_ids?: Array<string>,
    resource_2_resource_constraints?: Array<any> | null,
    slug?: string | null,
    subtitle?: string | StaticTranslatableString,
    template_id?: string,
    version?: string
  }, published: boolean=true): StaticGraph {
    const graphid = props.graphid || uuidv4();
    const publication = published ? new StaticPublication({
      graph_id: graphid,
      notes: null,
      publicationid: uuidv4(),
      published_time: (new Date()).toISOString()
    }) : null;
    // TODO: check name is not just string in upstream
    // Convert name to plain JS object for Rust deserialization
    let nameForRust;
    if (props.name instanceof StaticTranslatableString) {
      nameForRust = props.name.toJSON();
    } else if (typeof props.name === 'string') {
      nameForRust = props.name;
    } else {
      throw Error(`Name of graph must be string or StaticTranslatableString, not ${props.name}`);
    }
    const alias = slugify(typeof nameForRust === 'string' ? nameForRust : JSON.stringify(nameForRust));
    const root = new StaticNode({
      "alias": alias,
      "config": {},
      "datatype": "semantic",
      "description": "",
      "exportable": false,
      "fieldname": "",
      "graph_id": graphid,
      "hascustomalias": false,
      "is_collector": false,
      "isrequired": false,
      "issearchable": true,
      "istopnode": true,
      "name": typeof nameForRust === 'string' ? nameForRust : (nameForRust.en || Object.values(nameForRust)[0] || ''),
      "nodegroup_id": null,
      "nodeid": graphid,
      "ontologyclass": props.ontology_id ?? null,
      "parentproperty": null,
      "sortorder": 0,
      "sourcebranchpublication_id": null
    });

    let subtitleForRust;
    if (props.subtitle) {
      if (props.subtitle instanceof StaticTranslatableString) {
        subtitleForRust = props.subtitle.toJSON();
      } else {
        subtitleForRust = props.subtitle;
      }
    } else {
      subtitleForRust = '';
    }

    return new StaticGraph({
      author: props.author || '',
      cards: null,
      cards_x_nodes_x_widgets: null,
      color: props.color || null,
      config: props.config || {},
      deploymentdate: props.deploymentdate || null,
      deploymentfile: props.deploymentfile || null,
      description: props.description ? (
        props.description instanceof StaticTranslatableString ?
        props.description : new StaticTranslatableString(props.description)
      ) : new StaticTranslatableString(''),
      edges: [],
      functions_x_graphs: [],
      graphid: graphid,
      iconclass: props.iconclass || '',
      is_editable: props.is_editable || null,
      isresource: props.isresource ?? false,
      jsonldcontext: props.jsonldcontext || null,
      name: nameForRust,
      nodegroups: [],
      nodes: root ? [root.copy()] : [],
      ontology_id: props.ontology_id || null,
      publication: publication,
      relatable_resource_model_ids: props.relatable_resource_model_ids || [],
      resource_2_resource_constraints: props.resource_2_resource_constraints || null,
      root: root,
      slug: props.slug || null,
      subtitle: subtitleForRust,
      template_id: props.template_id || '',
      version: props.version || ''
    });
}

/// Resources
//
class StaticValue {
  id: string;
  value: string;
  __concept?: StaticConcept | null;
  __conceptId?: string | null;

  constructor(jsonData: StaticValue, concept: StaticConcept | string | null = null) {
    this.id = jsonData.id;
    this.value = jsonData.value;
    if (concept instanceof StaticConcept) {
      this.__concept = concept;
      this.__conceptId = concept ? concept.id : null;
    } else {
      this.__concept = null;
      this.__conceptId = concept;
    }
  }

  toJSON() {
    return {id: this.id, value: this.value};
  }

  toString() {
    return this.value;
  }

  static create(referent: string | StaticConcept, valueType: string, value: string, language?: string) {
    const lang = language || getCurrentLanguage();
    const referentId = referent instanceof StaticConcept ? referent.id : referent;
    // NB: this means passing an ID of a concept, and a concept, have different ID-creating behaviour.
    const concept = referent instanceof StaticConcept ? referent : null;
    const id = generateUuidv5(
      ['value'],
      `${referentId}/${valueType}/${value}/${lang}`
    );
    return new StaticValue(
      {
        id: id,
        value: value
      },
      concept
    );
  }
}

class StaticConcept {
  id: string;
  prefLabels: { [lang: string]: StaticValue };
  source: string | null;
  sortOrder: number | null;
  children: StaticConcept[] | null;

  constructor(jsonData: StaticConcept) {
    this.id = jsonData.id;
    this.prefLabels = jsonData.prefLabels;
    for (const [lang, value] of Object.entries(this.prefLabels)) {
      if (!(value instanceof StaticValue)) {
        this.prefLabels[lang] = new StaticValue(value, this);
      } else {
        // Set backlink even if already a StaticValue
        value.__concept = this;
        value.__conceptId = this.id;
      }
    }
    this.source = jsonData.source;
    this.sortOrder = jsonData.sortOrder;
    this.children = jsonData.children;
    if (this.children) {
      this.children = this.children.map((child) => {
        return child instanceof StaticConcept
          ? child
          : new StaticConcept(child);
      });
    }
  }

  getPrefLabel?(): StaticValue {
    return (
      this.prefLabels[getCurrentLanguage()] || Object.values(this.prefLabels)[0]
    );
  }

  toString() {
    if (!this.getPrefLabel) {
      return this.constructor(this).getPrefLabel().value;
    }
    return this.getPrefLabel().value;
  }

  // NB: copies value, does not make it a child
  static fromValue(conceptScheme: StaticConcept | null, value: string | StaticValue | { [lang: string]: StaticValue }, children?: (string | StaticValue | StaticConcept)[], config: {baseLanguage?: string, source?: string | null, sortOrder?: number | null} = {}): StaticConcept {
    // TODO make sure that children are in the same concept scheme so that deterministic IDs are preserved.
    let lang = config?.baseLanguage || getCurrentLanguage();
    let tmpValue: string;
    let prefLabels: { [lang: string]: StaticValue } | null = null;

    if (typeof value === 'string') {
      tmpValue = value;
      // prefLabels will be created after we have the concept ID
    } else if (value instanceof StaticValue) {
      tmpValue = value.value;
      prefLabels = {[lang]: value};
    } else if (lang in value) {
      tmpValue = value[lang].value;
      prefLabels = value;
    } else {
      const firstValue = Object.entries(value).sort()[0];
      if (firstValue === undefined) {
        throw Error("Cannot create a concept from values without a non-empty value");
      }
      lang = firstValue[0];
      tmpValue = firstValue[1].value;
      prefLabels = value;
    }

    const conceptId = generateUuidv5(
      ['concept'],
      `${conceptScheme?.id || '(none)'}/${tmpValue}`
    );

    // Create prefLabels with proper value ID if not already set
    if (prefLabels === null) {
      const valueId = generateUuidv5(
        ['value'],
        `${conceptId}/prefLabel/${tmpValue}/${lang}`
      );
      prefLabels = {[lang]: new StaticValue({id: valueId, value: tmpValue})};
    }

    const childConcepts = (children || []).map(child => {
      if (!(child instanceof StaticConcept)) {
        return StaticConcept.fromValue(conceptScheme, child, [], {baseLanguage: config.baseLanguage});
      }
      return child;
    });

    return new StaticConcept({
      id: conceptId,
      prefLabels,
      source: config.source || null,
      sortOrder: config.sortOrder || null,
      children: childConcepts
    });
  }
}

/**
 * Type of SKOS grouping structure.
 * - ConceptScheme: Uses narrower/broader hierarchy (default)
 * - Collection: Uses flat member relationships (Arches-compatible)
 */
export type SkosNodeType = 'ConceptScheme' | 'Collection';

// A prefLabel, for example, can only exist once per language.
class StaticCollection {
  id: string;
  uri?: string;
  prefLabels: { [lang: string]: StaticValue };
  altLabels?: { [lang: string]: StaticValue[] };
  scopeNotes?: { [lang: string]: StaticValue };
  nodeType: SkosNodeType;
  concepts: { [conceptId: string]: StaticConcept };
  __allConcepts: { [conceptId: string]: StaticConcept };
  __values: { [valueId: string]: StaticValue };

  static fromConceptScheme(props: {
    collectionid?: string,
    name?: string | { [lang: string]: StaticValue } | StaticValue;
    conceptScheme: StaticConcept;
    nodeType?: SkosNodeType;
  }): StaticCollection {
    const collectionName = props.name ?? props.conceptScheme.toString();
    return StaticCollection.create({
      collectionid: props.collectionid,
      name: collectionName,
      concepts: props.conceptScheme.children || [],
      nodeType: props.nodeType
    })
  }

  /**
   * Create a new StaticCollection.
   * @param props.nodeType - 'ConceptScheme' (default) for hierarchical, 'Collection' for flat Arches-compatible
   */
  static create(props: {
    collectionid?: string,
    uri?: string,
    name: string | { [lang: string]: StaticValue } | StaticValue;
    concepts: StaticConcept[] | { [conceptId: string]: StaticConcept };
    nodeType?: SkosNodeType;
  }): StaticCollection {
    let concepts = props.concepts;
    if (Array.isArray(concepts)) {
      concepts = concepts.reduce(
        (acc: { [conceptId: string]: StaticConcept }, c: StaticConcept) => {
          acc[c.id] = c;
          return acc;
        },
      {});
    }
    const name: StaticValue | { [lang: string]: StaticValue } = (
      typeof props.name === 'string' ?
        StaticValue.create('', 'prefLabel', props.name) :
        props.name
      );
    let collectionid = props.collectionid;
    if (!collectionid) {
      if (typeof name === 'string') {
        collectionid = generateUuidv5(
          ['collection'],
          name
        );
      } else if (name instanceof StaticValue) {
        collectionid = generateUuidv5(
          ['collection'],
          name.value
        );
      } else {
        throw Error("Must have a unique name to create a collection ID");
      }
    }
    const prefLabels: { [lang: string]: StaticValue } = name instanceof StaticValue ? {
      [getCurrentLanguage()]: name
    } : name;
    return new StaticCollection({
      id: collectionid,
      uri: props.uri,
      prefLabels: prefLabels,
      nodeType: props.nodeType || 'ConceptScheme',
      concepts: concepts,
      __allConcepts: {},
      __values: {}
    });
  }

  constructor(jsonData: StaticCollection) {
    this.id = jsonData.id;
    this.uri = jsonData.uri;
    this.prefLabels = jsonData.prefLabels;
    this.altLabels = jsonData.altLabels;
    this.scopeNotes = jsonData.scopeNotes;
    this.nodeType = jsonData.nodeType || 'ConceptScheme';
    this.concepts = jsonData.concepts;
    this.__allConcepts = {};
    this.__values = {};
    const addValues = (concept: StaticConcept) => {
      this.__allConcepts[concept.id] = concept;
      for (const [, value] of Object.entries(concept.prefLabels)) {
        this.__values[value.id] = value;
      }
      if (concept.children) {
        for (let child of concept.children) {
          if (!(child instanceof StaticConcept)) {
            child = new StaticConcept(child);
          }
          addValues(child);
        }
      }
    };
    for (const [id, concept] of Object.entries(this.concepts)) {
      if (!(concept instanceof StaticConcept)) {
        this.concepts[id] = new StaticConcept(concept);
      }
      addValues(this.concepts[id]);
    }
  }

  getConceptValue?(valueId: string) {
    return this.__values[valueId];
  }

  getConceptByValue?(label: string) {
    return Object.values(this.__values).find(value => value.value == label)?.__concept;
  }

  // Track whether this collection has been synced to Rust cache
  private _syncedToRustCache: boolean = false;

  /**
   * Ensure this collection is synced to the Rust RDM cache.
   * This is called automatically by hierarchy methods, but can be called
   * explicitly to pre-sync collections before label resolution.
   * @throws Error if WASM is not initialized
   */
  ensureInCache?(): void {
    getGlobalWasmRdmCache(); // Ensures WASM is initialized and cache exists
    this._ensureInRustCache();
  }

  /**
   * Get the parent concept ID for a given concept.
   * Uses the Rust RDM cache for hierarchy lookups.
   * @returns The parent concept ID, or null if no parent (top-level concept)
   * @throws Error if WASM is not initialized
   */
  getParentId?(conceptId: string): string | null {
    const cache = getGlobalWasmRdmCache();
    this._ensureInRustCache();
    return cache.getParentId(this.id, conceptId) ?? null;
  }

  /**
   * Get the parent concept for a given concept.
   * @returns The parent concept, or null if no parent
   */
  getParent?(conceptId: string): StaticConcept | null {
    const parentId = this.getParentId?.(conceptId);
    return parentId ? this.__allConcepts[parentId] ?? null : null;
  }

  /**
   * Get all ancestor concepts for a given concept (parent, grandparent, etc).
   * Returns in order from immediate parent to root.
   */
  getAncestors?(conceptId: string): StaticConcept[] {
    const ancestors: StaticConcept[] = [];
    let currentId: string | null = conceptId;

    while (currentId) {
      const parentId = this.getParentId?.(currentId);
      if (!parentId) break;
      const parent = this.__allConcepts[parentId];
      if (parent) {
        ancestors.push(parent);
      }
      currentId = parentId;
    }

    return ancestors;
  }

  /**
   * Ensure this collection's data is in the Rust RDM cache.
   * Called lazily when hierarchy lookups are needed.
   * @internal
   */
  private _ensureInRustCache(): void {
    if (this._syncedToRustCache) return;

    const cache = getGlobalWasmRdmCache();
    if (cache.hasCollection(this.id)) {
      this._syncedToRustCache = true;
      return;
    }

    // Convert to the format expected by the Rust cache
    const concepts = this._toRustCacheFormat();
    cache.addCollectionFromJson(this.id, JSON.stringify(concepts));
    this._syncedToRustCache = true;
  }

  /**
   * Convert collection concepts to the format expected by Rust cache.
   * @internal
   */
  private _toRustCacheFormat(): object[] {
    const result: object[] = [];

    const addConcept = (concept: StaticConcept, parentId: string | null) => {
      const prefLabels: { [lang: string]: { id: string; value: string } } = {};
      for (const [lang, value] of Object.entries(concept.prefLabels)) {
        prefLabels[lang] = { id: value.id, value: value.value };
      }

      const broader = parentId ? [parentId] : [];
      const narrower = concept.children?.map(c => c.id) || [];

      result.push({
        id: concept.id,
        prefLabels,
        broader,
        narrower,
      });

      // Recursively add children
      if (concept.children) {
        for (const child of concept.children) {
          addConcept(child, concept.id);
        }
      }
    };

    // Add all top-level concepts
    for (const concept of Object.values(this.concepts)) {
      addConcept(concept, null);
    }

    return result;
  }

  toString(): string {
    return (this.prefLabels[getCurrentLanguage()] || Object.values(this.prefLabels)[0] || '').toString();
  }
}

class StaticDomainValue {
  id: string
  selected: boolean
  text: {[lang: string]: string}

  constructor(jsonData: StaticDomainValue) {
    this.id = jsonData.id;
    this.selected = jsonData.selected;
    this.text = jsonData.text;
  }

  toString() {
    const lang = getCurrentLanguage();
    let localized = this.text[lang];
    if (typeof localized !== "string") {
      localized = Object.values(this.text)[0];
    }
    if (!localized) {
      throw Error(`Could not render domain value ${this.id} in language ${lang}`);
    }
    return localized;
  }

  lang(lang: string): string | undefined {
    return this.text[lang];
  }

  async forJson() {
    return {
      id: this.id,
      selected: this.selected,
      text: this.text
    }
  }
}

export {
  StaticValue,
  StaticTile,
  StaticGraph,
  createStaticGraph,
  StaticResource,
  StaticResourceSummary,
  StaticResourceMetadata,
  StaticResourceRegistry,
  StaticNode,
  StaticNodegroup,
  StaticEdge,
  StaticCard,
  StaticCardsXNodesXWidgets,
  StaticCollection,
  StaticConcept,
  StaticDomainValue,
  StaticResourceReference,
  StaticGraphMeta,
  StaticFunctionsXGraphs,
  StaticResourceDescriptors,
  StaticTranslatableString,
  StaticConstraint,
  StaticPublication,
  type IStaticDescriptorConfig,
  type SkosNodeType
};
