// TODO: make this customizable.
const DEFAULT_LANGUAGE = "en";

function getCurrentLanguage(): string {
  return (navigator.language || DEFAULT_LANGUAGE).slice(0, 2);
}

class StaticTranslatableString extends String {
    translations: Map<string, string>;
    lang: string;

    constructor(s: string | StaticTranslatableString, lang: undefined | string = undefined) {
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
        if (lang === undefined || !(lang in translations)) {
          const defaultLanguage = getCurrentLanguage();
          if (!translations || defaultLanguage in translations) {
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
}

class StaticNode {
    alias: string | null
    config: {[key: string]: any}
    datatype: string
    description: string | null
    exportable: boolean
    fieldname: null | string
    graph_id: string
    hascustomalias: boolean
    is_collector: boolean
    isrequired: boolean
    issearchable: boolean
    istopnode: boolean
    name: string
    nodegroup_id: string | null
    nodeid: string
    parentproperty: string | null = null
    sortorder: number
    ontologyclass: string | null = null
    sourcebranchpublication_id: null | string = null

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
}

class StaticConstraint {
    card_id: string
    constraintid: string
    nodes: Array<string>
    uniquetoallinstances: boolean

    constructor(jsonData: StaticConstraint) {
      this.card_id = jsonData.card_id;
      this.constraintid = jsonData.constraintid;
      this.nodes = jsonData.nodes;
      this.uniquetoallinstances = jsonData.uniquetoallinstances;
    }
}

class StaticCard {
    active: boolean
    cardid: string
    component_id: string
    config: null | Object
    constraints: Array<StaticConstraint>
    cssclass: null | string
    description: string | null | StaticTranslatableString
    graph_id: string
    helpenabled: boolean
    helptext: StaticTranslatableString
    helptitle: StaticTranslatableString
    instructions: StaticTranslatableString
    is_editable: boolean
    name: StaticTranslatableString
    nodegroup_id: string
    sortorder: number | null
    visible: boolean

    constructor(jsonData: StaticCard) {
      this.active = jsonData.active;
      this.cardid = jsonData.cardid;
      this.component_id = jsonData.component_id;
      this.config = jsonData.config;
      this.constraints = jsonData.constraints.map(
        constraint => new StaticConstraint(constraint)
      );
      this.cssclass = jsonData.cssclass;
      this.description = jsonData.description && new StaticTranslatableString(jsonData.description);
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
    card_id: string
    config: Object
    id: string
    label: StaticTranslatableString
    node_id: string
    sortorder: number
    visible: boolean
    widget_id: string

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
    description: null
    domainnode_id: string
    edgeid: string
    graph_id: string
    name: null | string
    rangenode_id: string
    ontologyproperty: null | string =  null

    constructor(jsonData: StaticEdge) {
      this.description = jsonData.description;
      this.domainnode_id = jsonData.domainnode_id;
      this.edgeid = jsonData.edgeid;
      this.graph_id = jsonData.graph_id;
      this.name = jsonData.name;
      this.rangenode_id = jsonData.rangenode_id;
      this.ontologyproperty = jsonData.ontologyproperty;
    }
}

class StaticFunctionsXGraphs {
    config: Object
    function_id: string
    graph_id: string
    id: string

    constructor(jsonData: StaticFunctionsXGraphs) {
      this.config = jsonData.config;
      this.function_id = jsonData.function_id;
      this.graph_id = jsonData.graph_id;
      this.id = jsonData.id;
    }
}

class StaticPublication {
    graph_id: string
    notes: null | string
    publicationid: string
    published_time: string

    constructor(jsonData: StaticPublication) {
      this.graph_id = jsonData.graph_id;
      this.notes = jsonData.notes;
      this.publicationid = jsonData.publicationid;
      this.published_time = jsonData.published_time;
    }
}

class StaticGraph {
    author: string
    cards: Array<StaticCard> | null = null
    cards_x_nodes_x_widgets: Array<StaticCardsXNodesXWidgets> | null = null
    color: string | null
    config: Object
    deploymentdate: null | string
    deploymentfile: null | string
    description: StaticTranslatableString
    edges: Array<StaticEdge>
    functions_x_graphs: Array<StaticFunctionsXGraphs> | null = null
    graphid: string
    iconclass: string
    is_editable: boolean | null = null
    isresource: boolean
    jsonldcontext: string | null
    name: StaticTranslatableString
    nodegroups: Array<StaticNodegroup>
    nodes: Array<StaticNode>
    ontology_id: string | null
    publication: StaticPublication | null = null
    relatable_resource_model_ids: Array<string>
    resource_2_resource_constringaints: Array<any> | null = null
    slug: string | null
    subtitle: StaticTranslatableString
    template_id: string
    version: string

    constructor(jsonData: StaticGraph) {
      this.author = jsonData.author;
      this.cards = jsonData.cards && jsonData.cards.map(card => new StaticCard(card));
      this.cards_x_nodes_x_widgets = jsonData.cards_x_nodes_x_widgets && jsonData.cards_x_nodes_x_widgets.map(
        card_x_node_x_widget => new StaticCardsXNodesXWidgets(card_x_node_x_widget)
      );
      this.color = jsonData.color;
      this.config = jsonData.config;
      this.deploymentdate = jsonData.deploymentdate;
      this.deploymentfile = jsonData.deploymentfile;
      this.description = new StaticTranslatableString(jsonData.description);
      this.edges = jsonData.edges.map(edge => new StaticEdge(edge));
      this.functions_x_graphs = jsonData.functions_x_graphs && jsonData.functions_x_graphs.map(
        functions_x_graphs => new StaticFunctionsXGraphs(functions_x_graphs)
      );
      this.graphid = jsonData.graphid;
      this.iconclass = jsonData.iconclass;
      this.is_editable = jsonData.is_editable;
      this.isresource = jsonData.isresource;
      this.jsonldcontext = jsonData.jsonldcontext;
      this.name = new StaticTranslatableString(jsonData.name);
      this.nodegroups = jsonData.nodegroups.map(nodegroup => new StaticNodegroup(nodegroup));
      this.nodes = jsonData.nodes.map(node => new StaticNode(node));
      this.ontology_id = jsonData.ontology_id;
      this.publication = jsonData.publication && new StaticPublication(jsonData.publication);
      this.relatable_resource_model_ids = jsonData.relatable_resource_model_ids;
      this.resource_2_resource_constringaints = jsonData.resource_2_resource_constringaints;
      this.slug = jsonData.slug;
      this.subtitle = new StaticTranslatableString(jsonData.subtitle);
      this.template_id = jsonData.template_id;
      this.version = jsonData.version;
    }
}

/// Resources
//
type StaticProvisionalEdit = any;

class StaticValue {
  id: string
  value: string
  __concept: StaticConcept | null

  constructor(jsonData: StaticValue, concept: StaticConcept | null=null) {
    this.id = jsonData.id;
    this.value = jsonData.value;
    this.__concept = concept;
  }
}

class StaticConcept {
  id: string;
  prefLabels: {[lang: string]: StaticValue};
  source: string | null;
  sortOrder: number | null;
  children: StaticConcept[] | null;

  constructor(jsonData: StaticConcept) {
    this.id = jsonData.id;
    this.prefLabels = jsonData.prefLabels;
    for (let [lang, value] of Object.entries(this.prefLabels)) {
     if (!(value instanceof StaticValue)) {
        this.prefLabels[lang] = new StaticValue(value, this);
      }
    }
    this.source = jsonData.source;
    this.sortOrder = jsonData.sortOrder;
    this.children = jsonData.children;
    if (this.children) {
      this.children = this.children.map(child => {
        return child instanceof StaticConcept ? child : new StaticConcept(child);
      });
    }
  }

  getPrefLabel(): StaticValue {
    return this.prefLabels[getCurrentLanguage()] || Object.values(this.prefLabels)[0];
  }

  toString() {
    return this.getPrefLabel().value;
  }
}

// A prefLabel, for example, can only exist once per language.
class StaticCollection {
  id: string
  prefLabels: {[lang: string]: StaticValue}
  concepts: {[conceptId: string]: StaticConcept}
  __allConcepts: {[conceptId: string]: StaticConcept}
  __values: {[valueId: string]: StaticValue}

  constructor(jsonData: StaticCollection) {
    this.id = jsonData.id;
    this.prefLabels = jsonData.prefLabels;
    this.concepts = jsonData.concepts;
    this.__allConcepts = {};
    this.__values = {};
    const addValues = (concept: StaticConcept) => {
      this.__allConcepts[concept.id] = concept;
      for (let [_, value] of Object.entries(concept.prefLabels)) {
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
    for (let [id, concept] of Object.entries(this.concepts)) {
      if (!(concept instanceof StaticConcept)) {
        this.concepts[id] = new StaticConcept(concept);
      }
      addValues(this.concepts[id]);
    }
  }

  getConceptValue(valueId: string) {
    return this.__values[valueId];
  }

  toString() {
    this.prefLabels[getCurrentLanguage()] || Object.values(this.prefLabels)[0];
  }
}

class StaticTile {
  data: Map<string, Object | Map<string, any> | Array<any> | null | number | boolean | string>
  nodegroup_id: string
  resourceinstance_id: string
  tileid: string | null
  parenttile_id: string | null = null
  provisionaledits: null | Array<StaticProvisionalEdit> = null
  sortorder: number | null = null

  constructor(jsonData: StaticTile) {
    this.data = jsonData.data;
    if (this.data && (!(this.data instanceof Map))) {
      this.data = new Map(Object.entries(this.data));
    }
    this.nodegroup_id = jsonData.nodegroup_id;
    this.resourceinstance_id = jsonData.resourceinstance_id;
    this.tileid = jsonData.tileid;
    this.parenttile_id = jsonData.parenttile_id;
    this.provisionaledits = jsonData.provisionaledits;
    this.sortorder = jsonData.sortorder;
  }
}

class StaticResourceMetadata {
  descriptors: Map<string, any>
  graph_id: string
  name: string
  resourceinstanceid: string
  publication_id: string | null = null
  principaluser_id: number | null = null
  legacyid: null | string = null
  graph_publication_id: string | null = null

  constructor(jsonData: StaticResourceMetadata) {
    this.descriptors = jsonData.descriptors;
    if (!(this.descriptors instanceof Map)) {
      this.descriptors = new Map([...Object.entries(this.descriptors)]);
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

class StaticResource {
  resourceinstance: StaticResourceMetadata;
  tiles: Array<StaticTile> | null = null

  constructor(jsonData: StaticResource) {
    this.resourceinstance = new StaticResourceMetadata(jsonData.resourceinstance);
    this.tiles = jsonData.tiles && jsonData.tiles.map(tile => new StaticTile(tile));
  }
}

export { StaticValue, StaticTile, StaticGraph, StaticResource, StaticNode, StaticNodegroup, StaticEdge, StaticCollection, StaticConcept };
