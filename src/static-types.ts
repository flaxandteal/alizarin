import { v4 as uuidv4 } from 'uuid';
import { generateUuidv5 } from './utils';
const UUID_NAMESPACE = '1a79f1c8-9505-4bea-a18e-28a053f725ca'; // Generated for this purpose.
import { getCurrentLanguage, slugify } from './utils';

class StaticGraphMeta {
  [key: string]: any
  author: string | undefined
  cards: number | undefined
  cards_x_nodes_x_widgets: number | undefined
  color: string | undefined
  description: {[lang: string]: string} | undefined
  edges: number | undefined
  graphid: string
  iconclass: string | undefined
  is_editable: boolean | undefined
  isresource: boolean | undefined
  jsonldcontext: {[key: string]: any} | undefined
  name: {[lang: string]: string} | undefined
  nodegroups: number | undefined
  nodes: number | undefined
  ontology_id: string | undefined
  publication: {[key: string]: string | null} | undefined
  relatable_resource_model_ids: string[] = []
  resource_2_resource_constraints: any[] = []
  root: StaticNode | undefined
  slug: string | undefined
  subtitle: {[lang: string]: string} | undefined
  version: string | undefined

  constructor(jsondata: StaticGraphMeta) {
    this.graphid = jsondata.graphid;
    Object.assign(this, jsondata)
  }
}

class StaticTranslatableString extends String {
  translations: Map<string, string>;
  lang: string;

  constructor(
    s: string | StaticTranslatableString,
    lang: undefined | string = undefined,
  ) {
    let translations: Map<string, string>;
    let finalLang: string;
    if (s instanceof StaticTranslatableString) {
      translations = new Map(s.translations);
      if (lang === undefined) {
        finalLang = s.lang;
      } else {
        finalLang = lang;
      }
    } else if (typeof s === "object") {
      translations = new Map(Object.entries(s));
      if (lang === undefined || !translations.has(lang)) {
        const defaultLanguage = getCurrentLanguage();
        if (!translations || translations.has(defaultLanguage)) {
          finalLang = defaultLanguage;
        } else {
          finalLang = Object.keys(s)[0];
        }
      } else {
        finalLang = lang;
      }
    } else {
      translations = new Map();
      finalLang = lang || getCurrentLanguage();
      translations.set(finalLang, s);
    }
    s = translations.get(finalLang) || "";
    super(s);
    this.translations = translations;
    this.lang = finalLang;
  }

  copy?() {
    return new StaticTranslatableString(this, this.lang);
  }

  toString(): string {
    const current = this.lang || getCurrentLanguage();
    let asString;
    if (this.translations.size) {
      asString = this.translations.get(current) || this.translations.values().next().value
    }
    return `${asString}`;
  }
}

class StaticNodegroup {
  legacygroupid: null;
  nodegroupid: string;
  parentnodegroup_id: string | null;
  cardinality: "1" | "n" | null;

  constructor(jsonData: StaticNodegroup) {
    this.legacygroupid = jsonData.legacygroupid;
    this.nodegroupid = jsonData.nodegroupid;
    this.parentnodegroup_id = jsonData.parentnodegroup_id;
    this.cardinality = jsonData.cardinality;
  }

  copy?(): StaticNodegroup {
    return new StaticNodegroup(this);
  }
}

class StaticNode {
  alias: string | null;
  config: { [key: string]: any } | null;
  datatype: string;
  description: string | null;
  exportable: boolean;
  fieldname: null | string;
  graph_id: string;
  hascustomalias: boolean;
  is_collector: boolean;
  isrequired: boolean;
  issearchable: boolean;
  istopnode: boolean;
  name: string;
  nodegroup_id: string | null;
  nodeid: string;
  parentproperty: string | null = null;
  sortorder: number;
  ontologyclass: string | null = null;
  sourcebranchpublication_id: null | string = null;

  constructor(jsonData: StaticNode) {
    this.alias = jsonData.alias;
    this.config = jsonData.config;
    this.datatype = jsonData.datatype;
    this.description = jsonData.description;
    this.exportable = jsonData.exportable;
    this.fieldname = jsonData.fieldname;
    this.graph_id = jsonData.graph_id;
    this.hascustomalias = jsonData.hascustomalias;
    this.is_collector = jsonData.is_collector;
    this.isrequired = jsonData.isrequired;
    this.issearchable = jsonData.issearchable;
    this.istopnode = jsonData.istopnode;
    this.name = jsonData.name;
    this.nodegroup_id = jsonData.nodegroup_id;
    this.nodeid = jsonData.nodeid;
    this.parentproperty = jsonData.parentproperty;
    this.sortorder = jsonData.sortorder;
    this.ontologyclass = jsonData.ontologyclass;
    this.sourcebranchpublication_id = jsonData.sourcebranchpublication_id;
  }

  copy?(): StaticNode {
    // TODO: config should be deep copied
    return new StaticNode(this);
  }

  // true -- same object
  // 2 -- identical
  // 1 -- identical not counting falsey nodeid, nodegroupid and/or graphid
  // -1 -- identical up to nodeid
  // -2 -- identical up to nodeid, nodegroupid
  // -3 -- identical up to nodeid, nodegroupid and graphid
  // false -- different
  // for <2, falsey nodeid, nodegroupid and graphid count as matches
  // and copy/compare are ignored.
  static compare(nodeA: StaticNode | {[key: string]: any}, nodeB: StaticNode | {[key: string]: any}): number | boolean {
    if (nodeA === nodeB) {
      return true;
    }
    const keys = [...Object.keys(nodeA), ...Object.keys(nodeB)].filter(key => ![
      'compare',
      'copy',
      'nodeid',
      'graph_id',
      'nodegroup_id'
    ].includes(key));
    // doubles keys...
    function compareEntries(entriesA: [string, any][], entriesB: [string, any][]) {
      const entryPairs: {[key: string]: any} = {};
      for (const [key, value] of [...entriesA, ...entriesB]) {
        entryPairs[key] = entryPairs[key] || [];
        entryPairs[key].push(value);
      }
      for (const [_, [valA, valB]] of Object.entries(entryPairs)) {
        if (valA && valB && typeof valA === 'object' && typeof valB === 'object') {
          if (!compareEntries(Object.entries(valA), Object.entries(valB))) {
            return false;
          }
        }
        if (Array.isArray(valA) && Array.isArray(valB)) {
          if (!compareEntries(Object.entries(valA), Object.entries(valB))) {
            return false;
          }
        }
        if (valA !== valB) {
          return false;
        }
      }
      return true;
    }
    if (!compareEntries(
      // @ts-expect-error Expecting values to be symbols
      keys.map((k: string): [string, any][] => [k, nodeA[k]]),
      // @ts-expect-error Expecting values to be symbols
      keys.map((k: string): [string, any][] => [k, nodeB[k]])
    )) {
      return false;
    }

    // We know these are the same up to the IDs
    if (nodeA.graph_id && nodeB.graph_id && nodeA.graph_id !== nodeB.graph_id) {
      return -3;
    }
    if (nodeA.nodegroup_id && nodeB.nodegroup_id && nodeA.nodegroup_id !== nodeB.nodegroup_id) {
      return -2;
    }
    if (nodeA.nodeid && nodeB.nodeid && nodeA.nodeid !== nodeB.nodeid) {
      return -1;
    }
    if (
      (nodeA.graph_id && nodeB.graph_id || nodeA.graph_id === nodeB.graph_id) &&
      (nodeA.nodegroup_id && nodeB.nodegroup_id || nodeA.nodegroup_id === nodeB.nodegroup_id) &&
      (nodeA.nodeid && nodeB.nodeid || nodeA.nodeid === nodeB.nodeid)
    ) {
      return 2;
    }
    return 1;
  }
}

class StaticConstraint {
  card_id: string;
  constraintid: string;
  nodes: Array<string>;
  uniquetoallinstances: boolean;

  constructor(jsonData: StaticConstraint) {
    this.card_id = jsonData.card_id;
    this.constraintid = jsonData.constraintid;
    this.nodes = jsonData.nodes;
    this.uniquetoallinstances = jsonData.uniquetoallinstances;
  }
}

class StaticCard {
  active: boolean;
  cardid: string;
  component_id: string;
  config: null | object;
  constraints: Array<StaticConstraint>;
  cssclass: null | string;
  description: string | null | StaticTranslatableString;
  graph_id: string;
  helpenabled: boolean;
  helptext: StaticTranslatableString;
  helptitle: StaticTranslatableString;
  instructions: StaticTranslatableString;
  is_editable: boolean;
  name: StaticTranslatableString;
  nodegroup_id: string;
  sortorder: number | null;
  visible: boolean;

  constructor(jsonData: StaticCard) {
    this.active = jsonData.active;
    this.cardid = jsonData.cardid;
    this.component_id = jsonData.component_id;
    this.config = jsonData.config;
    this.constraints = jsonData.constraints.map(
      (constraint) => new StaticConstraint(constraint),
    );
    this.cssclass = jsonData.cssclass;
    this.description =
      jsonData.description &&
      new StaticTranslatableString(jsonData.description);
    this.graph_id = jsonData.graph_id;
    this.helpenabled = jsonData.helpenabled;
    this.helptext = new StaticTranslatableString(jsonData.helptext);
    this.helptitle = new StaticTranslatableString(jsonData.helptitle);
    this.instructions = new StaticTranslatableString(jsonData.instructions);
    this.is_editable = jsonData.is_editable;
    this.name = new StaticTranslatableString(jsonData.name);
    this.nodegroup_id = jsonData.nodegroup_id;
    this.sortorder = jsonData.sortorder;
    this.visible = jsonData.visible;
  }
}

class StaticCardsXNodesXWidgets {
  card_id: string;
  config: object;
  id: string;
  label: StaticTranslatableString;
  node_id: string;
  sortorder: number;
  visible: boolean;
  widget_id: string;

  constructor(jsonData: StaticCardsXNodesXWidgets) {
    this.card_id = jsonData.card_id;
    this.config = jsonData.config;
    this.id = jsonData.id;
    this.label = new StaticTranslatableString(jsonData.label);
    this.node_id = jsonData.node_id;
    this.sortorder = jsonData.sortorder;
    this.visible = jsonData.visible;
    this.widget_id = jsonData.widget_id;
  }
}

class StaticEdge {
  description: string | null;
  domainnode_id: string;
  edgeid: string;
  graph_id: string;
  name: null | string;
  rangenode_id: string;
  ontologyproperty: null | string = null;

  constructor(jsonData: StaticEdge) {
    this.description = jsonData.description;
    this.domainnode_id = jsonData.domainnode_id;
    this.edgeid = jsonData.edgeid;
    this.graph_id = jsonData.graph_id;
    this.name = jsonData.name;
    this.rangenode_id = jsonData.rangenode_id;
    this.ontologyproperty = jsonData.ontologyproperty;
  }

  copy?(): StaticEdge {
    return new StaticEdge(this);
  }
}

interface IStaticDescriptorConfig {
  descriptor_types: {
    nodegroup_id: string,
    string_template: string
  }[],
};

class StaticFunctionsXGraphs {
  config: IStaticDescriptorConfig;
  function_id: string;
  graph_id: string;
  id: string;

  constructor(jsonData: StaticFunctionsXGraphs) {
    this.config = jsonData.config;
    this.function_id = jsonData.function_id;
    this.graph_id = jsonData.graph_id;
    this.id = jsonData.id;
  }

  copy(): StaticFunctionsXGraphs {
    return new StaticFunctionsXGraphs(this);
  }
}

class StaticPublication {
  graph_id: string;
  notes: null | string;
  publicationid: string;
  published_time: string;

  constructor(jsonData: StaticPublication) {
    this.graph_id = jsonData.graph_id;
    this.notes = jsonData.notes;
    this.publicationid = jsonData.publicationid;
    this.published_time = jsonData.published_time;
  }

  copy?(): StaticPublication {
    return new StaticPublication(this);
  }
}

class StaticGraph {
  author: string;
  cards: Array<StaticCard> | null = null;
  cards_x_nodes_x_widgets: Array<StaticCardsXNodesXWidgets> | null = null;
  color: string | null;
  config: object;
  deploymentdate: null | string = null;
  deploymentfile: null | string = null;
  description: StaticTranslatableString;
  edges: Array<StaticEdge>;
  functions_x_graphs: Array<StaticFunctionsXGraphs> | null = null;
  graphid: string;
  iconclass: string;
  is_editable: boolean | null = null;
  isresource: boolean;
  jsonldcontext: string | null = null;
  name: StaticTranslatableString;
  nodegroups: Array<StaticNodegroup>;
  nodes: Array<StaticNode>;
  ontology_id: string | null = null;
  publication: StaticPublication | null = null;
  relatable_resource_model_ids: Array<string>;
  resource_2_resource_constraints: Array<any> | null = null;
  root: StaticNode;
  slug: string | null = null;
  subtitle: StaticTranslatableString;
  template_id: string;
  version: string;

  constructor(jsonData: StaticGraph) {
    this.author = jsonData.author;
    this.cards =
      jsonData.cards && jsonData.cards.map((card) => new StaticCard(card));
    this.cards_x_nodes_x_widgets =
      jsonData.cards_x_nodes_x_widgets &&
      jsonData.cards_x_nodes_x_widgets.map(
        (card_x_node_x_widget) =>
          new StaticCardsXNodesXWidgets(card_x_node_x_widget),
      );
    this.color = jsonData.color;
    this.config = jsonData.config;
    this.deploymentdate = jsonData.deploymentdate;
    this.deploymentfile = jsonData.deploymentfile;
    this.description = new StaticTranslatableString(jsonData.description);
    this.edges = jsonData.edges.map((edge) => new StaticEdge(edge));
    this.functions_x_graphs =
      jsonData.functions_x_graphs &&
      jsonData.functions_x_graphs.map(
        (functions_x_graphs) => new StaticFunctionsXGraphs(functions_x_graphs),
      );
    this.graphid = jsonData.graphid;
    this.iconclass = jsonData.iconclass;
    this.is_editable = jsonData.is_editable;
    this.isresource = jsonData.isresource;
    this.jsonldcontext = jsonData.jsonldcontext;
    this.name = new StaticTranslatableString(jsonData.name);
    this.nodegroups = jsonData.nodegroups.map(
      (nodegroup) => new StaticNodegroup(nodegroup),
    );
    this.nodes = jsonData.nodes.map((node) => new StaticNode(node));
    this.ontology_id = jsonData.ontology_id;
    this.publication =
      jsonData.publication && new StaticPublication(jsonData.publication);
    this.relatable_resource_model_ids = jsonData.relatable_resource_model_ids;
    this.resource_2_resource_constraints =
      jsonData.resource_2_resource_constraints;
    this.root = jsonData.root;
    this.slug = jsonData.slug;
    this.subtitle = new StaticTranslatableString(jsonData.subtitle);
    this.template_id = jsonData.template_id;
    this.version = jsonData.version;
  }

  // TODO: complete deepcopy
  copy?(): StaticGraph {
    const newGraph = new StaticGraph(this);
    Object.assign(newGraph, {
      author: this.author,
      cards: this.cards?.map(card => new StaticCard(card)) || [],
      cards_x_nodes_x_widgets: this.cards_x_nodes_x_widgets?.map(cnw => new StaticCardsXNodesXWidgets(cnw)) || [],
      color: this.color,
      config: Object.assign({}, this.config), // TODO: deepcopy;
      deploymentdate: this.deploymentdate,
      deploymentfile: this.deploymentfile,
      description: this.description.copy && this.description.copy() || this.description,
      edges: this.edges.map(edge => edge.copy && edge.copy() || edge),
      functions_x_graphs: this.functions_x_graphs?.map(fxg => fxg.copy()) || [],
      graphid: this.graphid,
      iconclass: this.iconclass,
      is_editable: this.is_editable,
      isresource: this.isresource,
      jsonldcontext: this.jsonldcontext,
      name: this.name.copy && this.name.copy() || this.name,
      nodegroups: this.nodegroups?.map(ng => ng.copy && ng.copy() || ng),
      nodes: this.nodes?.map(n => n.copy && n.copy() || n),
      ontology_id: this.ontology_id,
      publication: this.publication?.copy && this.publication.copy() || null,
      relatable_resource_model_ids: [...this.relatable_resource_model_ids || []],
      resource_2_resource_constraints: [...this.resource_2_resource_constraints || []],
      root: this.root.copy && this.root.copy() || this.root,
      slug: this.slug,
      subtitle: this.subtitle.copy && this.subtitle.copy(),
      template_id: this.template_id,
      version: this.version
    });
    return newGraph;
  }

  static create(props: {
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
    jsonldcontext?: string | null,
    name?: string | StaticTranslatableString,
    ontology_id?: string | null,
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
    const name = props.name ? (
      props.name instanceof StaticTranslatableString ?
      props.name : new StaticTranslatableString(props.name)
    ) : new StaticTranslatableString('');
    const alias = slugify(name);
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
      "name": name.toString(),
      "nodegroup_id": null,
      "nodeid": graphid,
      "ontologyclass": props.ontology_id || null,
      "parentproperty": null,
      "sortorder": 0,
      "sourcebranchpublication_id": null
    });
    return new StaticGraph({
      author: props.author,
      cards: null,
      cards_x_nodes_x_widgets: null,
      color: props.color || null,
      config: props.config || {},
      deploymentdate: props.deploymentdate || null,
      deploymentfile: props.deploymentfile || null,
      description: props.description ? (
        props.description instanceof StaticTranslatableString ?
        props.description : new StaticTranslatableString(props.description)
      ) : null,
      edges: [],
      functions_x_graphs: [],
      graphid: graphid,
      iconclass: props.iconclass || '',
      is_editable: props.is_editable || null,
      isresource: props.isresource || null,
      jsonldcontext: props.jsonldcontext || null,
      name: name,
      nodegroups: [],
      nodes: [root.copy()],
      ontology_id: props.ontology_id || null,
      publication: publication,
      relatable_resource_model_ids: props.relatable_resource_model_ids || [],
      resource_2_resource_constraints: props.resource_2_resource_constraints || null,
      root: root,
      slug: props.slug || null,
      subtitle: props.subtitle ? (
        props.subtitle instanceof StaticTranslatableString ?
        props.subtitle : new StaticTranslatableString(props.subtitle)
      ) : new StaticTranslatableString(''),
      template_id: props.template_id || '',
      version: props.version || ''
    });
  }
}

/// Resources
//
type StaticProvisionalEdit = any;

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
    let tmpValue;
    let prefLabels: { [lang: string]: StaticValue };
    if (typeof value === 'string') {
      tmpValue = value;
      prefLabels = {[lang]: new StaticValue({id: '', value: tmpValue})};
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
    const childConcepts = (children || []).map(child => {
      if (!(child instanceof StaticConcept)) {
        return StaticConcept.fromValue(conceptScheme, value, [], {baseLanguage: config.baseLanguage});
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

// A prefLabel, for example, can only exist once per language.
class StaticCollection {
  id: string;
  prefLabels: { [lang: string]: StaticValue };
  concepts: { [conceptId: string]: StaticConcept };
  __allConcepts: { [conceptId: string]: StaticConcept };
  __values: { [valueId: string]: StaticValue };

  static fromConceptScheme(props: {
    collectionid?: string,
    name?: string | { [lang: string]: StaticValue } | StaticValue;
    conceptScheme: StaticConcept
  }): StaticCollection {
    const collectionName = props.name ?? props.conceptScheme.toString();
    return StaticCollection.create({
      name: collectionName,
      concepts: props.conceptScheme.children || []
    })
  }

  static create(props: {
    collectionid?: string,
    name: string | { [lang: string]: StaticValue } | StaticValue;
    concepts: StaticConcept[] | { [conceptId: string]: StaticConcept }
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
      prefLabels: prefLabels,
      concepts: concepts,
      __allConcepts: {},
      __values: {}
    });
  }

  constructor(jsonData: StaticCollection) {
    this.id = jsonData.id;
    this.prefLabels = jsonData.prefLabels;
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

  toString(): string {
    return (this.prefLabels[getCurrentLanguage()] || Object.values(this.prefLabels)[0] || '').toString();
  }
}

class StaticTile {
  data: Map<
    string,
    object | Map<string, any> | Array<any> | null | number | boolean | string
  >;
  nodegroup_id: string;
  resourceinstance_id: string;
  tileid: string | null;
  parenttile_id: string | null = null;
  provisionaledits: null | Array<StaticProvisionalEdit> = null;
  sortorder: number | null = null;

  constructor(jsonData: StaticTile) {
    this.data = jsonData.data;
    if (typeof this.data === 'object' && !(this.data instanceof Map)) {
      this.data = new Map(Object.entries(this.data));
    }
    this.nodegroup_id = jsonData.nodegroup_id;
    this.resourceinstance_id = jsonData.resourceinstance_id;
    this.tileid = jsonData.tileid;
    this.parenttile_id = jsonData.parenttile_id;
    this.provisionaledits = jsonData.provisionaledits;
    this.sortorder = jsonData.sortorder;
  }

  ensureId(): string {
    if (!this.tileid) {
      this.tileid = crypto.randomUUID();
    }
    return this.tileid;
  }
}

class StaticResourceDescriptors {
  [key: string]: (string | undefined | (() => boolean));
  name?: string;
  map_popup?: string;
  description?: string;

  constructor(jsonData?: StaticResourceDescriptors) {
    if (jsonData) {
      this.name = jsonData.name;
      this.map_popup = jsonData.map_popup;
      this.description = jsonData.description;
    }
  }

  isEmpty(): boolean {
    return !(this.name || this.map_popup || this.description);
  }
}

class StaticResourceMetadata {
  descriptors: StaticResourceDescriptors;
  graph_id: string;
  name: string;
  resourceinstanceid: string;
  publication_id: string | null = null;
  principaluser_id: number | null = null;
  legacyid: null | string = null;
  graph_publication_id: string | null = null;

  constructor(jsonData: StaticResourceMetadata) {
    this.descriptors = jsonData.descriptors;
    if (!(this.descriptors instanceof StaticResourceDescriptors)) {
      if (jsonData.descriptors instanceof Map) {
        this.descriptors = new StaticResourceDescriptors(Object.fromEntries(jsonData.descriptors.entries()));
      } else {
        this.descriptors = new StaticResourceDescriptors(this.descriptors);
      }
    }
    this.graph_id = jsonData.graph_id;
    this.name = jsonData.name;
    this.resourceinstanceid = jsonData.resourceinstanceid;
    this.publication_id = jsonData.publication_id;
    this.principaluser_id = jsonData.principaluser_id;
    this.legacyid = jsonData.legacyid;
    this.graph_publication_id = jsonData.graph_publication_id;
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

class StaticResourceReference {
  id: string;
  type: string | undefined;
  graphId: string;
  title: string | undefined;
  root: any | undefined;
  meta?: {[key: string]: any};

  constructor(jsonData: StaticResourceReference) {
    this.id = jsonData.id;
    this.type = jsonData.type;
    this.graphId = jsonData.graphId;
    this.root = jsonData.root;
    this.title = jsonData.title;
    if (jsonData.meta) {
      this.meta = jsonData.meta;
    }
  }
}

class StaticResource {
  resourceinstance: StaticResourceMetadata;
  tiles: Array<StaticTile> | null = null;
  metadata: {[key: string]: string};
  __cache: {[tileId: string]: {[nodeId: string]: {[key: string]: string}}} | undefined = undefined;
  __source: string | undefined = undefined;
  __scopes: string[] | undefined = undefined;

  constructor(jsonData: StaticResource) {
    this.resourceinstance = new StaticResourceMetadata(
      jsonData.resourceinstance,
    );
    this.tiles =
      jsonData.tiles && jsonData.tiles.map((tile) => new StaticTile(tile));
    this.metadata = jsonData.metadata || {};
    this.__cache = jsonData.__cache;
    this.__scopes = jsonData.__scopes;
  }
}

export {
  StaticValue,
  StaticTile,
  StaticGraph,
  StaticResource,
  StaticResourceMetadata,
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
  type IStaticDescriptorConfig
};
