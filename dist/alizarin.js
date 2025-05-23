var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
var _a, _b, _c, _d, _e;
const DEFAULT_LANGUAGE$1 = "en";
function getCurrentLanguage() {
  return (typeof navigator != "undefined" && navigator.language || DEFAULT_LANGUAGE$1).slice(0, 2);
}
class AttrPromise extends (_b = Promise, _a = Symbol.toPrimitive, _b) {
  constructor(executor) {
    super(executor);
    __publicField(this, _a);
    return new Proxy(this, {
      set: (object, keyObj, value) => {
        if (object instanceof Promise) {
          return object.then((val) => {
            val[keyObj] = value;
            return val;
          });
        }
        object[keyObj] = value;
        return this;
      },
      get: (object, keyObj) => {
        if (keyObj in object) {
          if (typeof object[keyObj] === "function") {
            return object[keyObj].bind(object);
          }
          return object[keyObj];
        }
        const key = keyObj.toString();
        if (key in object) {
          if (typeof object[key] === "function") {
            return object[key].bind(object);
          }
          return object[key];
        }
        if (object instanceof Promise) {
          return object.then((val) => {
            return val ? val[keyObj] : val;
          });
        }
        return object[keyObj];
      }
    });
  }
}
const utils = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  AttrPromise,
  getCurrentLanguage
}, Symbol.toStringTag, { value: "Module" }));
class StaticGraphMeta {
  constructor(jsondata) {
    __publicField(this, "author");
    __publicField(this, "cards");
    __publicField(this, "cards_x_nodes_x_widgets");
    __publicField(this, "color");
    __publicField(this, "description");
    __publicField(this, "edges");
    __publicField(this, "graphid");
    __publicField(this, "iconclass");
    __publicField(this, "is_editable");
    __publicField(this, "isresource");
    __publicField(this, "jsonldcontext");
    __publicField(this, "name");
    __publicField(this, "nodegroups");
    __publicField(this, "nodes");
    __publicField(this, "ontology_id");
    __publicField(this, "publication");
    __publicField(this, "relatable_resource_model_ids", []);
    __publicField(this, "resource_2_resource_constraints", []);
    __publicField(this, "root");
    __publicField(this, "slug");
    __publicField(this, "subtitle");
    __publicField(this, "version");
    this.graphid = jsondata.graphid;
    Object.assign(this, jsondata);
  }
}
class StaticTranslatableString extends String {
  constructor(s, lang = void 0) {
    let translations;
    let finalLang;
    if (s instanceof StaticTranslatableString) {
      translations = new Map(s.translations);
      if (lang === void 0) {
        finalLang = s.lang;
      } else {
        finalLang = lang;
      }
    } else if (typeof s === "object") {
      translations = new Map(Object.entries(s));
      if (lang === void 0 || !translations.has(lang)) {
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
      translations = /* @__PURE__ */ new Map();
      finalLang = lang || getCurrentLanguage();
      translations.set(finalLang, s);
    }
    s = translations.get(finalLang) || "";
    super(s);
    __publicField(this, "translations");
    __publicField(this, "lang");
    this.translations = translations;
    this.lang = finalLang;
  }
}
class StaticNodegroup {
  constructor(jsonData) {
    __publicField(this, "legacygroupid");
    __publicField(this, "nodegroupid");
    __publicField(this, "parentnodegroup_id");
    __publicField(this, "cardinality");
    this.legacygroupid = jsonData.legacygroupid;
    this.nodegroupid = jsonData.nodegroupid;
    this.parentnodegroup_id = jsonData.parentnodegroup_id;
    this.cardinality = jsonData.cardinality;
  }
}
class StaticNode {
  constructor(jsonData) {
    __publicField(this, "alias");
    __publicField(this, "config");
    __publicField(this, "datatype");
    __publicField(this, "description");
    __publicField(this, "exportable");
    __publicField(this, "fieldname");
    __publicField(this, "graph_id");
    __publicField(this, "hascustomalias");
    __publicField(this, "is_collector");
    __publicField(this, "isrequired");
    __publicField(this, "issearchable");
    __publicField(this, "istopnode");
    __publicField(this, "name");
    __publicField(this, "nodegroup_id");
    __publicField(this, "nodeid");
    __publicField(this, "parentproperty", null);
    __publicField(this, "sortorder");
    __publicField(this, "ontologyclass", null);
    __publicField(this, "sourcebranchpublication_id", null);
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
  constructor(jsonData) {
    __publicField(this, "card_id");
    __publicField(this, "constraintid");
    __publicField(this, "nodes");
    __publicField(this, "uniquetoallinstances");
    this.card_id = jsonData.card_id;
    this.constraintid = jsonData.constraintid;
    this.nodes = jsonData.nodes;
    this.uniquetoallinstances = jsonData.uniquetoallinstances;
  }
}
class StaticCard {
  constructor(jsonData) {
    __publicField(this, "active");
    __publicField(this, "cardid");
    __publicField(this, "component_id");
    __publicField(this, "config");
    __publicField(this, "constraints");
    __publicField(this, "cssclass");
    __publicField(this, "description");
    __publicField(this, "graph_id");
    __publicField(this, "helpenabled");
    __publicField(this, "helptext");
    __publicField(this, "helptitle");
    __publicField(this, "instructions");
    __publicField(this, "is_editable");
    __publicField(this, "name");
    __publicField(this, "nodegroup_id");
    __publicField(this, "sortorder");
    __publicField(this, "visible");
    this.active = jsonData.active;
    this.cardid = jsonData.cardid;
    this.component_id = jsonData.component_id;
    this.config = jsonData.config;
    this.constraints = jsonData.constraints.map(
      (constraint) => new StaticConstraint(constraint)
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
  constructor(jsonData) {
    __publicField(this, "card_id");
    __publicField(this, "config");
    __publicField(this, "id");
    __publicField(this, "label");
    __publicField(this, "node_id");
    __publicField(this, "sortorder");
    __publicField(this, "visible");
    __publicField(this, "widget_id");
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
  constructor(jsonData) {
    __publicField(this, "description");
    __publicField(this, "domainnode_id");
    __publicField(this, "edgeid");
    __publicField(this, "graph_id");
    __publicField(this, "name");
    __publicField(this, "rangenode_id");
    __publicField(this, "ontologyproperty", null);
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
  constructor(jsonData) {
    __publicField(this, "config");
    __publicField(this, "function_id");
    __publicField(this, "graph_id");
    __publicField(this, "id");
    this.config = jsonData.config;
    this.function_id = jsonData.function_id;
    this.graph_id = jsonData.graph_id;
    this.id = jsonData.id;
  }
}
class StaticPublication {
  constructor(jsonData) {
    __publicField(this, "graph_id");
    __publicField(this, "notes");
    __publicField(this, "publicationid");
    __publicField(this, "published_time");
    this.graph_id = jsonData.graph_id;
    this.notes = jsonData.notes;
    this.publicationid = jsonData.publicationid;
    this.published_time = jsonData.published_time;
  }
}
class StaticGraph {
  constructor(jsonData) {
    __publicField(this, "author");
    __publicField(this, "cards", null);
    __publicField(this, "cards_x_nodes_x_widgets", null);
    __publicField(this, "color");
    __publicField(this, "config");
    __publicField(this, "deploymentdate");
    __publicField(this, "deploymentfile");
    __publicField(this, "description");
    __publicField(this, "edges");
    __publicField(this, "functions_x_graphs", null);
    __publicField(this, "graphid");
    __publicField(this, "iconclass");
    __publicField(this, "is_editable", null);
    __publicField(this, "isresource");
    __publicField(this, "jsonldcontext");
    __publicField(this, "name");
    __publicField(this, "nodegroups");
    __publicField(this, "nodes");
    __publicField(this, "ontology_id");
    __publicField(this, "publication", null);
    __publicField(this, "relatable_resource_model_ids");
    __publicField(this, "resource_2_resource_constraints", null);
    __publicField(this, "root");
    __publicField(this, "slug");
    __publicField(this, "subtitle");
    __publicField(this, "template_id");
    __publicField(this, "version");
    this.author = jsonData.author;
    this.cards = jsonData.cards && jsonData.cards.map((card) => new StaticCard(card));
    this.cards_x_nodes_x_widgets = jsonData.cards_x_nodes_x_widgets && jsonData.cards_x_nodes_x_widgets.map(
      (card_x_node_x_widget) => new StaticCardsXNodesXWidgets(card_x_node_x_widget)
    );
    this.color = jsonData.color;
    this.config = jsonData.config;
    this.deploymentdate = jsonData.deploymentdate;
    this.deploymentfile = jsonData.deploymentfile;
    this.description = new StaticTranslatableString(jsonData.description);
    this.edges = jsonData.edges.map((edge) => new StaticEdge(edge));
    this.functions_x_graphs = jsonData.functions_x_graphs && jsonData.functions_x_graphs.map(
      (functions_x_graphs) => new StaticFunctionsXGraphs(functions_x_graphs)
    );
    this.graphid = jsonData.graphid;
    this.iconclass = jsonData.iconclass;
    this.is_editable = jsonData.is_editable;
    this.isresource = jsonData.isresource;
    this.jsonldcontext = jsonData.jsonldcontext;
    this.name = new StaticTranslatableString(jsonData.name);
    this.nodegroups = jsonData.nodegroups.map(
      (nodegroup) => new StaticNodegroup(nodegroup)
    );
    this.nodes = jsonData.nodes.map((node) => new StaticNode(node));
    this.ontology_id = jsonData.ontology_id;
    this.publication = jsonData.publication && new StaticPublication(jsonData.publication);
    this.relatable_resource_model_ids = jsonData.relatable_resource_model_ids;
    this.resource_2_resource_constraints = jsonData.resource_2_resource_constraints;
    this.root = jsonData.root;
    this.slug = jsonData.slug;
    this.subtitle = new StaticTranslatableString(jsonData.subtitle);
    this.template_id = jsonData.template_id;
    this.version = jsonData.version;
  }
}
class StaticValue {
  constructor(jsonData, concept = null) {
    __publicField(this, "id");
    __publicField(this, "value");
    __publicField(this, "__concept");
    __publicField(this, "__conceptId");
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
}
class StaticConcept {
  constructor(jsonData) {
    __publicField(this, "id");
    __publicField(this, "prefLabels");
    __publicField(this, "source");
    __publicField(this, "sortOrder");
    __publicField(this, "children");
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
        return child instanceof StaticConcept ? child : new StaticConcept(child);
      });
    }
  }
  getPrefLabel() {
    return this.prefLabels[getCurrentLanguage()] || Object.values(this.prefLabels)[0];
  }
  toString() {
    return this.getPrefLabel().value;
  }
}
class StaticCollection {
  constructor(jsonData) {
    __publicField(this, "id");
    __publicField(this, "prefLabels");
    __publicField(this, "concepts");
    __publicField(this, "__allConcepts");
    __publicField(this, "__values");
    this.id = jsonData.id;
    this.prefLabels = jsonData.prefLabels;
    this.concepts = jsonData.concepts;
    this.__allConcepts = {};
    this.__values = {};
    const addValues = (concept) => {
      this.__allConcepts[concept.id] = concept;
      for (const [_, value] of Object.entries(concept.prefLabels)) {
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
  getConceptValue(valueId) {
    return this.__values[valueId];
  }
  toString() {
    return this.prefLabels[getCurrentLanguage()] || Object.values(this.prefLabels)[0];
  }
}
class StaticTile {
  constructor(jsonData) {
    __publicField(this, "data");
    __publicField(this, "nodegroup_id");
    __publicField(this, "resourceinstance_id");
    __publicField(this, "tileid");
    __publicField(this, "parenttile_id", null);
    __publicField(this, "provisionaledits", null);
    __publicField(this, "sortorder", null);
    this.data = jsonData.data;
    if (this.data && !(this.data instanceof Map)) {
      this.data = new Map(Object.entries(this.data));
    }
    this.nodegroup_id = jsonData.nodegroup_id;
    this.resourceinstance_id = jsonData.resourceinstance_id;
    this.tileid = jsonData.tileid;
    this.parenttile_id = jsonData.parenttile_id;
    this.provisionaledits = jsonData.provisionaledits;
    this.sortorder = jsonData.sortorder;
  }
  ensureId() {
    if (!this.tileid) {
      this.tileid = crypto.randomUUID();
    }
    return this.tileid;
  }
}
class StaticResourceMetadata {
  constructor(jsonData) {
    __publicField(this, "descriptors");
    __publicField(this, "graph_id");
    __publicField(this, "name");
    __publicField(this, "resourceinstanceid");
    __publicField(this, "publication_id", null);
    __publicField(this, "principaluser_id", null);
    __publicField(this, "legacyid", null);
    __publicField(this, "graph_publication_id", null);
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
class StaticDomainValue {
  constructor(jsonData) {
    __publicField(this, "id");
    __publicField(this, "selected");
    __publicField(this, "text");
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
  lang(lang) {
    return this.text[lang];
  }
  async forJson() {
    return {
      id: this.id,
      selected: this.selected,
      text: this.text
    };
  }
}
class StaticResourceReference {
  constructor(jsonData) {
    __publicField(this, "id");
    __publicField(this, "type");
    __publicField(this, "graphId");
    __publicField(this, "title");
    __publicField(this, "root");
    __publicField(this, "meta");
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
  constructor(jsonData) {
    __publicField(this, "resourceinstance");
    __publicField(this, "tiles", null);
    __publicField(this, "metadata");
    __publicField(this, "__cache");
    __publicField(this, "__source");
    __publicField(this, "__scopes");
    this.resourceinstance = new StaticResourceMetadata(
      jsonData.resourceinstance
    );
    this.tiles = jsonData.tiles && jsonData.tiles.map((tile) => new StaticTile(tile));
    this.metadata = jsonData.metadata || {};
    this.__cache = jsonData.__cache;
    this.__scopes = jsonData.__scopes;
  }
}
const staticTypes = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  StaticCard,
  StaticCardsXNodesXWidgets,
  StaticCollection,
  StaticConcept,
  StaticDomainValue,
  StaticEdge,
  StaticFunctionsXGraphs,
  StaticGraph,
  StaticGraphMeta,
  StaticNode,
  StaticNodegroup,
  StaticResource,
  StaticResourceMetadata,
  StaticResourceReference,
  StaticTile,
  StaticValue
}, Symbol.toStringTag, { value: "Module" }));
class GraphResult {
  constructor(jsonData) {
    __publicField(this, "models");
    this.models = Object.fromEntries(
      Object.entries(jsonData.models).map(([k, v]) => [k, new StaticGraphMeta(v)])
    );
  }
}
class ArchesClient {
}
class ArchesClientRemote extends ArchesClient {
  constructor(archesUrl) {
    super();
    __publicField(this, "archesUrl");
    this.archesUrl = archesUrl;
  }
  async getGraphs() {
    const response = await fetch(
      `${this.archesUrl}/api/arches/graphs?format=arches-json&hide_empty_nodes=false&compact=false`
    );
    return await response.json();
  }
  async getGraph(graphId) {
    const response = await fetch(
      `${this.archesUrl}/graphs/${graphId}?format=arches-json&gen=`
    );
    return await response.json();
  }
  async getResource(resourceId) {
    throw Error(`Not implemented yet: getResource(${resourceId}`);
  }
  async getCollection(collectionId) {
    throw Error(`Not implemented yet: getCollection(${collectionId}`);
  }
  async getResources(graphId, limit) {
    const response = await fetch(
      `${this.archesUrl}/resources?graph_uuid=${graphId}&format=arches-json&hide_empty_nodes=false&compact=false&limit=${limit}`
    );
    return await response.json();
  }
}
class ArchesClientRemoteStatic extends ArchesClient {
  constructor(archesUrl, {
    allGraphFile,
    graphIdToGraphFile,
    graphIdToResourcesFiles,
    resourceIdToFile,
    collectionIdToFile
  } = {}) {
    super();
    __publicField(this, "archesUrl");
    __publicField(this, "allGraphFile");
    __publicField(this, "graphIdToGraphFile");
    __publicField(this, "graphIdToResourcesFiles");
    __publicField(this, "resourceIdToFile");
    __publicField(this, "collectionIdToFile");
    this.archesUrl = archesUrl;
    this.allGraphFile = allGraphFile || (() => "resource_models/_all.json");
    this.graphIdToGraphFile = graphIdToGraphFile || ((graphId) => `resource_models/${graphId}.json`);
    this.graphIdToResourcesFiles = graphIdToResourcesFiles || ((graphId) => [`business_data/_${graphId}.json`]);
    this.resourceIdToFile = resourceIdToFile || ((resourceId) => `business_data/${resourceId}.json`);
    this.collectionIdToFile = collectionIdToFile || ((collectionId) => `collections/${collectionId}.json`);
  }
  async getGraphs() {
    const response = await fetch(`${this.archesUrl}/${this.allGraphFile()}`);
    return await response.json();
  }
  async getGraph(graphId) {
    const response = await fetch(
      `${this.archesUrl}/${this.graphIdToGraphFile(graphId)}`
    );
    return (await response.json()).graph[0];
  }
  async getResource(resourceId) {
    const source = `${this.archesUrl}/${this.resourceIdToFile(resourceId)}`;
    const response = await fetch(source);
    return response.json().then((response2) => {
      response2.__source = source;
      return response2;
    });
  }
  async getCollection(collectionId) {
    const response = await fetch(
      `${this.archesUrl}/${this.collectionIdToFile(collectionId)}`
    );
    return await response.json();
  }
  async getResources(graphId, limit) {
    const resources = [];
    for (const file of this.graphIdToResourcesFiles(graphId)) {
      const source = `${this.archesUrl}/${file}`;
      const response = await fetch(source);
      const resourceSet = (await response.json()).business_data.resources;
      for (const resource of resourceSet) {
        resource.__source = source;
      }
      resources.push(...limit ? resourceSet.slice(0, limit) : resourceSet);
      if (limit && resources.length > limit) {
        break;
      }
    }
    return resources;
  }
}
class ArchesClientLocal extends ArchesClient {
  constructor({
    allGraphFile,
    graphIdToGraphFile,
    graphIdToResourcesFiles,
    resourceIdToFile,
    collectionIdToFile
  } = {}) {
    super();
    __publicField(this, "fs");
    __publicField(this, "allGraphFile");
    __publicField(this, "graphIdToGraphFile");
    __publicField(this, "graphIdToResourcesFiles");
    __publicField(this, "resourceIdToFile");
    __publicField(this, "collectionIdToFile");
    this.fs = import("./__vite-browser-external-2Ng8QIWW.js");
    this.allGraphFile = allGraphFile || (() => "tests/definitions/models/_all.json");
    this.graphIdToGraphFile = graphIdToGraphFile || ((graphId) => `tests/definitions/models/${graphId}.json`);
    this.graphIdToResourcesFiles = graphIdToResourcesFiles || ((graphId) => [`tests/definitions/resources/_${graphId}.json`]);
    this.resourceIdToFile = resourceIdToFile || ((resourceId) => `tests/definitions/resources/${resourceId}.json`);
    this.collectionIdToFile = collectionIdToFile || ((collectionId) => `tests/definitions/collections/${collectionId}.json`);
  }
  async getGraphs() {
    const fs = await this.fs;
    const response = await fs.promises.readFile(this.allGraphFile(), "utf8");
    return new GraphResult(await JSON.parse(response));
  }
  async getGraph(graphId) {
    const fs = await this.fs;
    const graphFile = this.graphIdToGraphFile(graphId);
    if (!graphFile) {
      return null;
    }
    const response = await fs.promises.readFile(
      graphFile,
      "utf8"
    );
    return await JSON.parse(response).graph[0];
  }
  async getResource(resourceId) {
    const fs = await this.fs;
    const source = this.resourceIdToFile(resourceId);
    const response = await fs.promises.readFile(
      source,
      "utf8"
    );
    return JSON.parse(response).then((resource) => {
      resource.__source = source;
      return resource;
    });
  }
  async getCollection(collectionId) {
    const fs = await this.fs;
    const response = await fs.promises.readFile(
      this.collectionIdToFile(collectionId),
      "utf8"
    );
    return await JSON.parse(response);
  }
  async getResources(graphId, limit) {
    const fs = await this.fs;
    const resources = [];
    for (const file of this.graphIdToResourcesFiles(graphId)) {
      const response = JSON.parse(await fs.promises.readFile(file, "utf8"));
      const source = file;
      const resourceSet = response.business_data.resources.filter(
        (resource) => graphId === resource.resourceinstance.graph_id
      );
      for (const resource of resourceSet) {
        resource.__source = source;
      }
      resources.push(...limit ? resourceSet.slice(0, limit) : resourceSet);
      if (limit && resources.length > limit) {
        break;
      }
    }
    return resources;
  }
}
const archesClient = new ArchesClientRemote("http://localhost:8000");
const client = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  ArchesClient,
  ArchesClientLocal,
  ArchesClientRemote,
  ArchesClientRemoteStatic,
  GraphResult,
  archesClient
}, Symbol.toStringTag, { value: "Module" }));
const interfaces = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null
}, Symbol.toStringTag, { value: "Module" }));
class ReferenceDataManager {
  constructor(archesClient2) {
    __publicField(this, "archesClient");
    __publicField(this, "collections");
    this.archesClient = archesClient2;
    this.collections = /* @__PURE__ */ new Map();
  }
  retrieveCollection(id) {
    let collection = this.collections.get(id);
    if (collection !== void 0) {
      return collection;
    }
    collection = this.archesClient.getCollection(id).then((jsonData) => new StaticCollection(jsonData));
    this.collections.set(id, collection);
    return collection;
  }
}
const RDM = new ReferenceDataManager(archesClient);
class StaticStore {
  constructor(archesClient2, cacheMetadataOnly = true) {
    __publicField(this, "archesClient");
    __publicField(this, "cache");
    __publicField(this, "cacheMetadataOnly");
    this.archesClient = archesClient2;
    this.cache = /* @__PURE__ */ new Map();
    this.cacheMetadataOnly = cacheMetadataOnly;
  }
  async getMeta(id, onlyIfCached = true) {
    if (this.cache.has(id)) {
      const resource = this.cache.get(id);
      if (resource instanceof StaticResource) {
        return resource.resourceinstance;
      }
      return resource;
    }
    if (!onlyIfCached) {
      const resource = await this.loadOne(id);
      return resource.resourceinstance;
    }
    return null;
  }
  async *loadAll(graphId, limit = void 0) {
    const resourcesJSON = await this.archesClient.getResources(graphId, limit || 0);
    for (const resourceJSON of resourcesJSON.values()) {
      const resource = new StaticResource(resourceJSON);
      this.cache.set(
        resource.resourceinstance.resourceinstanceid,
        this.cacheMetadataOnly ? resource.resourceinstance : resource
      );
      yield resource;
    }
  }
  async loadOne(id) {
    if (this.cache.has(id)) {
      const resource2 = this.cache.get(id);
      if (resource2 instanceof StaticResource) {
        return resource2;
      }
    }
    const resourceJSON = await this.archesClient.getResource(id);
    const resource = new StaticResource(resourceJSON);
    if (this.cacheMetadataOnly) {
      this.cache.set(id, this.cacheMetadataOnly ? resource.resourceinstance : resource);
    }
    return resource;
  }
}
const staticStore = new StaticStore(archesClient);
class StaticNodeConfigBoolean {
  constructor(jsonData) {
    __publicField(this, "i18n_properties");
    __publicField(this, "falseLabel");
    __publicField(this, "trueLabel");
    this.i18n_properties = jsonData.i18n_properties;
    this.falseLabel = jsonData.falseLabel;
    this.trueLabel = jsonData.trueLabel;
  }
}
class StaticNodeConfigDomain {
  constructor(jsonData) {
    __publicField(this, "i18n_config");
    __publicField(this, "options");
    this.i18n_config = jsonData.i18n_config;
    this.options = jsonData.options;
    if (this.options) {
      this.options = this.options.map((sdv) => {
        if (!(sdv instanceof StaticDomainValue)) {
          return new StaticDomainValue(sdv);
        }
        return sdv;
      });
    }
  }
  getSelected() {
    return this.options.find((option) => option.selected);
  }
  valueFromId(id) {
    return this.options.find((option) => option.id == id);
  }
}
const _NodeConfigManager = class _NodeConfigManager {
  constructor(cache = void 0) {
    __publicField(this, "cache");
    if (!cache) {
      cache = _NodeConfigManager._cache || /* @__PURE__ */ new Map();
    }
    this.cache = cache;
  }
  retrieve(node) {
    if (this.cache.has(node.nodeid)) {
      return this.cache.get(node.nodeid);
    }
    let nodeConfig2 = null;
    switch (node.datatype) {
      case "boolean":
        nodeConfig2 = new StaticNodeConfigBoolean(node.config);
        break;
      case "domain-value-list":
      case "domain-value":
        nodeConfig2 = new StaticNodeConfigDomain(node.config);
        break;
    }
    this.cache.set(node.nodeid, nodeConfig2);
    return nodeConfig2;
  }
};
__publicField(_NodeConfigManager, "_cache");
let NodeConfigManager = _NodeConfigManager;
const nodeConfigManager = new NodeConfigManager();
const nodeConfig = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  StaticNodeConfigBoolean,
  StaticNodeConfigDomain,
  nodeConfigManager
}, Symbol.toStringTag, { value: "Module" }));
const DEFAULT_LANGUAGE = "en";
class ViewContext {
  constructor() {
    __publicField(this, "graphManager");
  }
}
const viewContext = new ViewContext();
function tileLoadingError(reason, exc) {
  {
    console.error(reason, exc);
    {
      throw exc;
    }
  }
}
class ValueList {
  constructor(values, wrapper, tiles) {
    __publicField(this, "values");
    __publicField(this, "wrapper");
    __publicField(this, "tiles");
    __publicField(this, "promises");
    __publicField(this, "writeLock");
    this.values = values;
    this.wrapper = wrapper;
    this.tiles = tiles;
    this.promises = /* @__PURE__ */ new Map();
    this.writeLock = null;
  }
  async get(key) {
    return this.retrieve(key, this.values.get(key), true);
  }
  set(key, value) {
    this.values.set(key, value);
  }
  async has(key) {
    await this.retrieve(key, null);
    return this.values.has(key);
  }
  async retrieve(key, dflt = null, raiseError = false) {
    const node = this.wrapper.model.getNodeObjectsByAlias().get(key);
    const promise = node ? this.promises.get(node.nodegroup_id) : null;
    if (promise) {
      await promise;
    }
    let result = await this.values.get(key);
    if (result === false) {
      await this.writeLock;
      if (this.wrapper.resource) {
        const node2 = this.wrapper.model.getNodeObjectsByAlias().get(key);
        if (node2 === void 0) {
          throw Error(
            "Tried to retrieve a node key that does not exist on this resource"
          );
        }
        const values = new Map([...this.values.entries()]);
        const promise2 = new Promise((resolve) => {
          this.wrapper.ensureNodegroup(
            values,
            node2,
            node2.nodegroup_id,
            this.wrapper.model.getNodeObjects(),
            this.wrapper.model.getNodegroupObjects(),
            this.wrapper.model.getEdges(),
            false,
            this.tiles,
            true
          ).then(([ngValues]) => {
            let original = false;
            for (const [k, value] of [...ngValues.entries()]) {
              if (key === k) {
                original = value;
              }
              if (value !== false) {
                this.values.set(k, value);
              }
            }
            resolve(original);
            this.promises.delete(node2.nodegroup_id);
          });
        });
        this.writeLock = promise2;
        this.promises.set(node2.nodegroup_id, promise2);
        this.values.set(key, promise2);
        await promise2;
      } else {
        this.values.delete(key);
      }
      result = this.values.get(key);
    }
    result = await result;
    if (result === void 0 || result === false) {
      if (raiseError) {
        throw Error(`Unset key ${key}`);
      } else {
        return dflt;
      }
    }
    return result;
  }
  async setDefault(key, value) {
    const newValue = await this.retrieve(key, value, false);
    this.values.set(key, newValue);
    return newValue;
  }
}
class ConceptListCacheEntry {
  constructor({ meta, _ }) {
    __publicField(this, "datatype", "concept-list");
    __publicField(this, "_");
    __publicField(this, "meta");
    this._ = _.map((instance) => {
      if (instance instanceof ConceptValueCacheEntry) {
        return instance;
      }
      return new ConceptValueCacheEntry(instance);
    });
    this.meta = meta || {};
  }
}
class ConceptValueCacheEntry {
  constructor({ meta, id, value, conceptId }) {
    __publicField(this, "datatype", "concept");
    __publicField(this, "id");
    __publicField(this, "value");
    __publicField(this, "conceptId");
    __publicField(this, "meta");
    this.id = id;
    this.value = value;
    this.conceptId = conceptId;
    this.meta = meta || {};
  }
}
class ResourceInstanceListCacheEntry {
  constructor({ meta, _ }) {
    __publicField(this, "datatype", "resource-instance-list");
    __publicField(this, "_");
    __publicField(this, "meta");
    this._ = _.map((instance) => {
      if (instance instanceof ResourceInstanceCacheEntry) {
        return instance;
      }
      return new ResourceInstanceCacheEntry(instance);
    });
    this.meta = meta || {};
  }
}
class ResourceInstanceCacheEntry {
  constructor({ meta, id, type, graphId, title }) {
    __publicField(this, "datatype", "resource-instance");
    __publicField(this, "id");
    __publicField(this, "type");
    __publicField(this, "graphId");
    __publicField(this, "title");
    __publicField(this, "meta");
    this.id = id;
    this.type = type;
    this.graphId = graphId;
    this.meta = meta || {};
    this.title = this.meta.title || title;
  }
}
class ResourceInstanceListViewModel extends Array {
  constructor() {
    super(...arguments);
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_value", null);
  }
  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => v ? v.forJson() : null) : null;
  }
  async __forJsonCache(getMeta) {
    return new ResourceInstanceListCacheEntry({
      meta: getMeta ? await getMeta(this) : getMeta,
      _: await Promise.all([...this.values()].map(async (rivmPromise) => {
        const rivm = await rivmPromise;
        return await rivm.__forJsonCache(getMeta);
      }))
    });
  }
  static async __create(tile, node, value, cacheEntry = null) {
    const nodeid = node.nodeid;
    let val;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, []);
        if (!Array.isArray(value)) {
          throw Error(
            "Cannot set an (entire) resource list value except via an array"
          );
        }
        val = value.map((v, i) => {
          if (v instanceof ResourceInstanceViewModel) {
            return v;
          }
          return ResourceInstanceViewModel.__create(tile, node, v, cacheEntry && cacheEntry._[i] ? cacheEntry._[i] : null);
        });
        Promise.all(
          val.map(async (c) => {
            const v = await c;
            return v ? (await v).id : null;
          })
        ).then((ids) => {
          tile.data.set(nodeid, ids);
          return ids;
        });
        value = val;
      }
    } else {
      value = [];
    }
    if (!tile || !value) {
      return null;
    }
    const str = new ResourceInstanceListViewModel(...value);
    return str;
  }
  async __asTileData() {
    return this._value ? await this._value : null;
  }
}
_c = Symbol.toPrimitive;
const _ResourceInstanceViewModel = class _ResourceInstanceViewModel {
  constructor(id, modelWrapper, instanceWrapperFactory, cacheEntry) {
    __publicField(this, "_");
    __publicField(this, "__");
    __publicField(this, "__parentPseudo");
    __publicField(this, "__cacheEntry", null);
    __publicField(this, "id");
    __publicField(this, "then");
    __publicField(this, _c);
    __publicField(this, "gm");
    this.id = id;
    this._ = instanceWrapperFactory ? instanceWrapperFactory(this) : null;
    this.__ = modelWrapper;
    if (cacheEntry instanceof ResourceInstanceCacheEntry) {
      this.__cacheEntry = cacheEntry;
    }
    return new Proxy(this, {
      // NOTE: set should not return a promise, so could cause a race
      // condition with a subsequent read.
      // @ts-expect-error Returning a promise for set
      set: async (object, key, value) => {
        const k = typeof key === "symbol" ? key.description || "" : key;
        if (key in object) {
          object[key] = value;
        } else if (k in object) {
          object[k] = value;
        } else {
          if (!object._) {
            await this.retrieve();
            if (!object._) {
              throw Error("Could not retrieve resource");
            }
          }
          object._.setOrmAttribute(k, value);
        }
        return true;
      },
      get: (object, key) => {
        const k = typeof key === "symbol" ? key.description || "" : key;
        if (key in object) {
          return object[key];
        } else if (k in object) {
          return object[k];
        }
        return new AttrPromise(async (resolve) => {
          if (!object._) {
            await this.retrieve();
            if (!object._) {
              throw Error("Could not retrieve resource");
            }
          }
          return object._.getOrmAttribute(k).then((v) => {
            return resolve(v);
          });
        });
      }
    });
  }
  toString() {
    if (!this.__) {
      return `[Resource:${this.id}]`;
    }
    return `[${this.__.wkrm.modelClassName}:${this.id ?? "-"}]`;
  }
  async __has(key) {
    if (!this._) {
      return void 0;
    }
    return (await this._.getRootViewModel() || /* @__PURE__ */ new Map()).__has(key);
  }
  async __asTileData() {
    return {
      resourceId: this.id
    };
  }
  async __forJsonCache(getMeta) {
    let wrapper;
    if (!this.__) {
      if (this.__cacheEntry) {
        return this.__cacheEntry;
      } else {
        [, wrapper] = await this.retrieve();
      }
    } else {
      wrapper = this.__;
    }
    this.__cacheEntry = new ResourceInstanceCacheEntry({
      meta: getMeta ? await getMeta(this) : void 0,
      id: this.id,
      type: wrapper.wkrm.modelClassName,
      graphId: wrapper.wkrm.graphId,
      title: null
    });
    return this.__cacheEntry;
  }
  async forJson(cascade = false) {
    let jsonData;
    if (!cascade && this.__cacheEntry) {
      jsonData = {
        type: this.__cacheEntry.type,
        graphId: this.__cacheEntry.graphId,
        id: this.__cacheEntry.id,
        title: this.__cacheEntry.title || void 0,
        meta: this.__cacheEntry.meta || void 0,
        root: null
      };
    } else if (this.__) {
      jsonData = {
        type: this.__.wkrm.modelClassName,
        graphId: this.__.wkrm.graphId,
        id: this.id,
        title: void 0,
        meta: void 0,
        root: null
      };
    } else {
      jsonData = {
        type: "(unknown)",
        graphId: "",
        id: this.id,
        title: void 0,
        meta: void 0,
        root: null
      };
    }
    const basic = new StaticResourceReference(jsonData);
    if (cascade) {
      if (!this._) {
        await this.retrieve();
        if (!this._) {
          throw Error("Could not retrieve resource");
        }
      }
      const root = await this._.getRootViewModel();
      basic.root = await root.forJson();
    }
    return basic;
  }
  async retrieve() {
    let iw;
    let mw;
    if (viewContext.graphManager) {
      const replacement = await viewContext.graphManager.getResource(this.id, true);
      iw = replacement._;
      mw = replacement.__;
    } else {
      throw Error("Cannot traverse resource relationships without a GraphManager");
    }
    this._ = iw;
    this.__ = mw;
    return [iw, mw];
  }
  static async __create(tile, node, value, cacheEntry) {
    const nodeid = node.nodeid;
    let val = value;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (!value && !(value instanceof StaticResource) && !(value instanceof StaticResourceReference)) {
          val = null;
        } else if (value instanceof Promise) {
          return value.then((value2) => {
            return _ResourceInstanceViewModel.__create(tile, node, value2, cacheEntry);
          });
        } else if (typeof value == "string") {
          if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(
            value
          )) {
            val = value;
          } else {
            throw Error(
              `Set resource instances using id, not strings in node ${node.alias}: ${value}`
            );
          }
        } else if (value instanceof Object && value.resourceId) {
          val = value.resourceId;
        } else if (value instanceof Array && value.length < 2) {
          if (value.length == 1) {
            return _ResourceInstanceViewModel.__create(tile, node, value[0], cacheEntry);
          }
        } else {
          throw Error("Could not set resource instance from this data");
        }
        tile.data.set(nodeid, val ? val : null);
      }
    }
    if (!tile || !val) {
      return null;
    }
    const str = new _ResourceInstanceViewModel(val, null, null, cacheEntry);
    return str;
  }
};
let ResourceInstanceViewModel = _ResourceInstanceViewModel;
class ConceptListViewModel extends Array {
  constructor() {
    super(...arguments);
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_value", null);
  }
  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => v ? v.forJson() : null) : null;
  }
  async __forJsonCache(getMeta) {
    return new ConceptListCacheEntry({
      meta: getMeta ? await getMeta(this) : getMeta,
      _: await Promise.all([...this.values()].map(async (rivmPromise) => {
        const rivm = await rivmPromise;
        return await rivm.__forJsonCache(getMeta);
      }))
    });
  }
  static async __create(tile, node, value, cacheEntry = null) {
    const nodeid = node.nodeid;
    let val = [];
    if (!tile.data.has(nodeid)) {
      tile.data.set(nodeid, null);
    }
    if (value !== null) {
      tile.data.set(nodeid, []);
      if (!Array.isArray(value)) {
        throw Error(
          "Cannot set an (entire) concept list value except via an array"
        );
      }
      val = value.map((c, i) => {
        if (c instanceof ConceptValueViewModel) {
          return c;
        }
        return ConceptValueViewModel.__create(tile, node, c, cacheEntry && cacheEntry._[i] ? cacheEntry._[i] : null);
      });
      Promise.all(val).then((vals) => {
        Promise.all(
          vals.map(async (c) => {
            const v = await c;
            return v ? (await v.getValue()).id : null;
          })
        ).then((ids) => {
          tile.data.set(nodeid, ids);
        });
      });
      value = val;
    } else {
      value = [];
    }
    const str = new ConceptListViewModel(...value);
    return str;
  }
  async __asTileData() {
    return this._value ? await this._value : null;
  }
}
class DomainValueListViewModel extends Array {
  constructor() {
    super(...arguments);
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_value", null);
  }
  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => v ? v.forJson() : null) : null;
  }
  // No point in caching something that is on the graph.
  __forJsonCache() {
    return null;
  }
  static async __create(tile, node, value) {
    const nodeid = node.nodeid;
    let val;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, []);
        if (!Array.isArray(value)) {
          throw Error(
            "Cannot set an (entire) domain list value except via an array"
          );
        }
        val = value.map((c) => {
          if (c instanceof DomainValueViewModel) {
            return c;
          }
          return DomainValueViewModel.__create(tile, node, c);
        });
        Promise.all(val).then(async (vals) => {
          const ids = Promise.all(vals.map(async (val2) => val2 === null ? val2 : (await val2._value).id));
          ids.then((ids2) => {
            tile.data.set(nodeid, ids2);
          });
        });
      }
    } else {
      value = [];
    }
    const str = new DomainValueListViewModel(...value);
    return str;
  }
  async __asTileData() {
    const value = await this._value;
    return value ?? null;
  }
}
class DomainValueViewModel extends String {
  constructor(value) {
    super(value.toString());
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_value");
    this._value = value;
  }
  async forJson() {
    return this._value;
  }
  // No point in caching something that is on the graph.
  __forJsonCache() {
    return null;
  }
  getValue() {
    return this._value;
  }
  async lang(lang) {
    return (await this._value).lang(lang);
  }
  static async __create(tile, node, value) {
    const nodeid = node.nodeid;
    let val = value;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (!value && !(value instanceof StaticDomainValue)) {
          val = null;
        } else if (value instanceof Promise) {
          return value.then((value2) => {
            return DomainValueViewModel.__create(tile, node, value2);
          });
        } else if (typeof value == "string") {
          if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-5][0-9a-f]{3}-[089ab][0-9a-f]{3}-[0-9a-f]{12}$/i.exec(
            value
          )) {
            const config = nodeConfigManager.retrieve(node);
            if (!config || !(config instanceof StaticNodeConfigDomain)) {
              throw Error(`Cannot form domain value for ${node.nodeid} without config`);
            }
            val = config.valueFromId(value) || null;
          } else {
            throw Error(
              "Set domain values using values from domain lists, not strings"
            );
          }
        } else {
          throw Error("Could not set domain value from this data");
        }
        if (!(val instanceof Promise)) {
          tile.data.set(nodeid, val ? val.id : null);
        }
      }
    }
    if (!tile || !val) {
      return null;
    }
    const str = new DomainValueViewModel(val);
    return str;
  }
  async __asTileData() {
    const value = await this._value;
    return value ? value.id : null;
  }
}
class ConceptValueViewModel extends String {
  constructor(value) {
    super(value.value);
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_value");
    this._value = value;
  }
  async forJson() {
    return this._value;
  }
  async __forJsonCache(getMeta) {
    const value = await this._value;
    return new ConceptValueCacheEntry({
      meta: getMeta ? await getMeta(this) : void 0,
      id: value.id,
      value: value.value,
      conceptId: value.__conceptId
    });
  }
  getValue() {
    return this._value;
  }
  static async __create(tile, node, value, cacheEntry) {
    const nodeid = node.nodeid;
    let val = value;
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        if (value instanceof StaticConcept) {
          val = value.getPrefLabel();
        }
        if (!value) {
          val = null;
        } else if (value instanceof StaticValue) ;
        else if (value instanceof Promise) {
          return value.then((value2) => {
            return ConceptValueViewModel.__create(tile, node, value2, cacheEntry);
          });
        } else if (typeof value == "string") {
          if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(
            value
          )) {
            if (cacheEntry) {
              val = new StaticValue({
                id: cacheEntry.id,
                value: cacheEntry.value,
                __concept: null,
                __conceptId: cacheEntry.conceptId
              }, cacheEntry.conceptId);
              return new ConceptValueViewModel(val);
            } else {
              const collectionId = node.config["rdmCollection"];
              const collection = RDM.retrieveCollection(collectionId);
              return collection.then((collection2) => {
                const val2 = collection2.getConceptValue(value);
                if (!val2) {
                  console.error("Could not find concept for value", value, "for", node.alias, "in collection", collectionId);
                }
                tile.data.set(nodeid, val2 ? val2.id : null);
                if (!tile || !val2) {
                  return null;
                }
                const str2 = new ConceptValueViewModel(val2);
                return str2;
              });
            }
          } else {
            throw Error(
              `Set concepts using values from collections, not strings: ${value}`
            );
          }
        } else {
          throw Error("Could not set concept from this data");
        }
        if (!(val instanceof Promise)) {
          if (!val) {
            console.error("Could not find concept for value", value, "for", node.alias, "in collection", node.config.get("rdmCollection"));
          }
          tile.data.set(nodeid, val ? val.id : null);
        }
      }
    }
    if (!tile || !val) {
      return null;
    }
    const str = new ConceptValueViewModel(val);
    return str;
  }
  async __asTileData() {
    const value = await this._value;
    return value ? value.id : null;
  }
}
class DateViewModel extends Date {
  constructor(val) {
    super(val);
    __publicField(this, "__parentPseudo");
    __publicField(this, "__original");
    __publicField(this, "then");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    this.__original = val;
  }
  __forJsonCache() {
    return null;
  }
  static __create(tile, node, value) {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then(
        (value2) => DateViewModel.__create(tile, node, value2)
      );
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }
    let val = tile.data.get(nodeid);
    if (typeof val == "object" && val["en"] !== void 0) {
      val = val.en;
    }
    if (!tile || val === null || val === void 0 || val === "") {
      return null;
    }
    if (typeof val != "string") {
      throw Error("Date should be a string");
    }
    const str = new DateViewModel(val);
    return str;
  }
  async forJson() {
    try {
      return this.toISOString();
    } catch (e) {
      console.warn(e);
      return this.__original;
    }
  }
  __asTileData() {
    return this.toISOString();
  }
}
_d = Symbol.toPrimitive;
const _GeoJSONViewModel = class _GeoJSONViewModel {
  constructor(jsonData) {
    __publicField(this, "__parentPseudo");
    __publicField(this, "then");
    __publicField(this, _d);
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_value");
    this._value = jsonData;
    return new Proxy(this, {
      get: (object, key) => {
        const k = typeof key === "symbol" ? key.description || "" : key;
        if (key in object) {
          return object[key];
        } else if (k in object) {
          return object[k];
        }
        return this._value[k];
      },
      set: (object, key, value) => {
        const k = typeof key === "symbol" ? key.description || "" : key;
        if (key in object) {
          object[key] = value;
        } else if (k in object) {
          object[k] = value;
        } else {
          this._value[k] = value;
        }
        return true;
      }
    });
  }
  __forJsonCache() {
    return null;
  }
  static __create(tile, node, value) {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then(
        (value2) => _GeoJSONViewModel.__create(tile, node, value2)
      );
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }
    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === void 0) {
      return null;
    }
    if (!(val instanceof Object)) {
      throw Error("GeoJSON should be a JSON object");
    }
    const str = new _GeoJSONViewModel(val);
    return str;
  }
  async forJson() {
    return await this._value;
  }
  __asTileData() {
    return this._value;
  }
};
let GeoJSONViewModel = _GeoJSONViewModel;
class NonLocalizedStringViewModel extends String {
  constructor() {
    super(...arguments);
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
  }
  __forJsonCache() {
    return null;
  }
  forJson() {
    return this.toString();
  }
  static __create(tile, node, value) {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value2) => NonLocalizedStringViewModel.__create(tile, node, value2));
    }
    if (tile) {
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }
    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === void 0) {
      return null;
    }
    const string = new NonLocalizedStringViewModel(val);
    return string;
  }
  __asTileData() {
    return this ? true : false;
  }
}
class NumberViewModel extends Number {
  constructor() {
    super(...arguments);
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
  }
  toString() {
    return `${this.valueOf()}`;
  }
  __forJsonCache() {
    return null;
  }
  forJson() {
    return this.valueOf();
  }
  static __create(tile, node, value) {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value2) => NumberViewModel.__create(tile, node, value2));
    }
    if (tile) {
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }
    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === void 0) {
      return null;
    }
    const num = new NumberViewModel(val);
    return num;
  }
  __asTileData() {
    return this ? true : false;
  }
}
class UrlViewModel extends String {
  constructor() {
    super(...arguments);
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
  }
  __forJsonCache() {
    return null;
  }
  forJson() {
    return this ? true : false;
  }
  static __create(tile, node, value) {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value2) => UrlViewModel.__create(tile, node, value2));
    }
    if (tile) {
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }
    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === void 0) {
      return null;
    }
    const url = new UrlViewModel(val);
    return url;
  }
  __asTileData() {
    return this ? true : false;
  }
}
class BooleanViewModel extends Boolean {
  constructor(value, config) {
    super(value);
    __publicField(this, "__parentPseudo");
    __publicField(this, "__config");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    this.__config = config;
  }
  toString(lang) {
    const labelLang = lang || DEFAULT_LANGUAGE;
    const isTrue = this;
    return isTrue ? this.__config && this.__config.trueLabel ? this.__config.trueLabel[labelLang] || "true" : "true" : this.__config && this.__config.trueLabel ? this.__config.falseLabel[labelLang] || "false" : "false";
  }
  __forJsonCache() {
    return null;
  }
  forJson() {
    return this ? true : false;
  }
  static __create(tile, node, value) {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value2) => BooleanViewModel.__create(tile, node, value2));
    }
    if (tile) {
      if (value !== null) {
        tile.data.set(nodeid, value);
      }
    }
    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === void 0) {
      return null;
    }
    const config = nodeConfigManager.retrieve(node);
    if (!config || !(config instanceof StaticNodeConfigBoolean)) {
      throw Error(`Cannot form boolean value for ${node.nodeid} without config`);
    }
    const bool = new BooleanViewModel(val, config);
    return bool;
  }
  __asTileData() {
    return this ? true : false;
  }
}
class StringViewModel extends String {
  constructor(value, language = null) {
    const lang = value.get(language || DEFAULT_LANGUAGE);
    let displayValue;
    if (lang) {
      if (typeof lang == "string") {
        displayValue = lang;
      } else {
        displayValue = lang.value;
      }
    } else {
      displayValue = "";
    }
    super(displayValue);
    __publicField(this, "__parentPseudo");
    __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
    __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
    __publicField(this, "_value");
    this._value = value;
  }
  __forJsonCache() {
    return null;
  }
  forJson() {
    return `${this}`;
  }
  lang(language) {
    const elt = this._value.get(language);
    if (elt) {
      if (elt instanceof Object) {
        return elt.value;
      }
      return elt;
    } else {
      return void 0;
    }
  }
  static __create(tile, node, value) {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then((value2) => StringViewModel.__create(tile, node, value2));
    }
    if (tile) {
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, {});
      }
      if (value !== null) {
        if (value instanceof Object) {
          const entries = value instanceof Map ? value.entries() : Object.entries(value);
          for (const [k, v] of [...entries]) {
            const val2 = tile.data.get(nodeid);
            if (val2 instanceof Map) {
              val2.set(k, v);
            } else if (val2 instanceof Object) {
              val2[k] = v;
            } else if (val2 !== null) {
              throw Error("Malformed string in tile data");
            }
          }
        } else {
          tile.data.set(nodeid, {
            [DEFAULT_LANGUAGE]: value
          });
        }
      }
    }
    const val = tile.data.get(nodeid);
    if (!tile || val === null || val === void 0) {
      return null;
    }
    let mapVal;
    if (val instanceof Map) {
      mapVal = val;
    } else {
      mapVal = new Map(Object.entries(val));
    }
    const str = new StringViewModel(mapVal);
    return str;
  }
  __asTileData() {
    return this._value;
  }
}
_e = Symbol.toPrimitive;
const _SemanticViewModel = class _SemanticViewModel {
  constructor(parentWkri, childNodes, tile, node) {
    __publicField(this, "then");
    __publicField(this, _e);
    __publicField(this, "__parentPseudo");
    __publicField(this, "__childValues");
    __publicField(this, "__parentWkri");
    __publicField(this, "__childNodes");
    __publicField(this, "__tile");
    __publicField(this, "__node");
    this.__childValues = /* @__PURE__ */ new Map();
    this.__parentWkri = parentWkri;
    this.__tile = tile;
    this.__node = node;
    this.__childNodes = childNodes;
    return new Proxy(this, {
      set: (object, key, value) => {
        const k = typeof key === "symbol" ? key.description || "" : key;
        if (key in object) {
          object[key] = value;
        } else if (k.startsWith("__") || k in object) {
          object[k] = value;
        } else {
          object.__set(k, value);
        }
        return true;
      },
      get: (object, key) => {
        const k = typeof key === "symbol" ? key.description || "" : key;
        if (key in object) {
          return object[key];
        } else if (k.startsWith("__") || k in object) {
          return object[k];
        }
        if (k == "length") {
          return object.__childNodes.size;
        }
        return new AttrPromise((resolve) => {
          object.__get(k).then(resolve);
        });
      }
    });
  }
  __forJsonCache() {
    return null;
  }
  async toString() {
    const entries = [...this.__childValues.entries()].map(([k, v]) => `${k}: ${v}`);
    return `[[${entries.join(",")}]]`;
  }
  async toObject() {
    const entries = [...(await this.__getChildValues()).entries()];
    return Object.fromEntries(await Promise.all(entries.map(async ([k, vl]) => {
      return [k, (await vl).getValue()];
    })));
  }
  async forJson() {
    async function _forJson(v) {
      v = await v;
      if (!v) {
        return null;
      }
      return await v.forJson();
    }
    const entries = [...(await this.__getChildValues()).entries()];
    return Object.fromEntries(await Promise.all(entries.map(async ([k, vl]) => {
      return [k, vl ? await _forJson(vl) : vl];
    })));
  }
  async __update(map) {
    return Promise.all(
      [...map.entries()].map(([k, v]) => {
        this.__set(k, v);
      })
    );
  }
  async __get(key) {
    const childValue = await this.__getChildValue(key);
    return childValue.getValue();
  }
  async __set(key, value) {
    if (!this.__childNodes.has(key)) {
      throw Error(
        `Semantic node does not have this key: ${key} (${[...this.__childNodes.keys()]})`
      );
    }
    throw Error(`Setting semantic keys (${key} = ${value}) is not implemented yet in Javascript`);
  }
  __has(key) {
    return this.__childNodes.has(key);
  }
  async __getChildTypes() {
    const promises = [...this.__childNodes.keys()].map(async (key) => [
      key,
      await this.__getChildValue(key)
    ]);
    const entries = await Promise.all(promises);
    return new Map([...entries]);
  }
  async __getChildren(direct = null) {
    const items = /* @__PURE__ */ new Map();
    for (const [key, value] of [...(await this.__getChildValues()).entries()]) {
      items.set(key, value);
    }
    const children = [...items.entries()].filter((entry) => {
      const child = this.__childNodes.get(entry[0]);
      if (!child) {
        throw Error("Child key is not in child nodes");
      }
      return (direct === null || direct === !child.is_collector) && entry[1] !== null;
    }).map((entry) => entry[1]);
    return children;
  }
  async __getChildValue(key, setDefault = false) {
    if (!this.__childNodes.has(key)) {
      throw Error(
        `Semantic node does not have this key: ${key} (${[...this.__childNodes.keys()]})`
      );
    }
    let child;
    if (!this.__childValues.has(key)) {
      const children = await this.__getChildValues();
      child = children.get(key) || null;
      let set = true;
      if (child === null) {
        child = this.__makePseudo(key);
        set = setDefault;
      }
      if (set) {
        this.__childValues.set(key, child);
      }
      child.parentNode = this.__parentPseudo || null;
    } else {
      child = this.__childValues.get(key);
    }
    return child;
  }
  __makePseudo(key) {
    const childNode = this.__childNodes.get(key);
    if (!childNode) {
      throw Error(`Child node key ${key} missing`);
    }
    if (!this.__parentWkri) {
      throw Error("This semantic node is currently parentless (no WKRI)");
    }
    if (!this.__parentWkri._) {
      throw Error("This semantic node is currently on an unloaded WKRI");
    }
    const child = this.__parentWkri._.addPseudo(childNode, this.__tile);
    child.parentNode = this.__parentPseudo || null;
    return child;
  }
  static async __create(tile, node, value, parent, childNodes) {
    const svm = new _SemanticViewModel(parent, childNodes, tile, node);
    if (value) {
      try {
        await svm.__update(value);
      } catch (e) {
        tileLoadingError(
          `
          Suppressed a tile loading error: ${e}: ${typeof e} (tile: ${tile}; node: ${node}) - ${value}
        `,
          e
        );
      }
    }
    return svm;
  }
  async __asTileData() {
    const relationships = [];
    for (const value of [...await this.__getChildren(true)]) {
      const [, subrelationships] = await value.getTile();
      relationships.push(...subrelationships);
    }
    return [null, relationships];
  }
  async __getChildValues() {
    const parent = this.__parentWkri;
    const childNodes = this.__childNodes;
    const tile = this.__tile;
    const node = this.__node;
    if (!parent || !parent._) {
      return /* @__PURE__ */ new Map();
    }
    await parent._.loadNodes([...childNodes.keys()]);
    const children = /* @__PURE__ */ new Map();
    for (const entry of [...parent._.allEntries()]) {
      const key = entry[0];
      let values = entry[1];
      if (values instanceof Promise) {
        values = await values;
      }
      if (values === false || values === null || values === void 0) {
        continue;
      }
      const childNode = childNodes.get(key);
      for (let value of values) {
        if (childNode && value.node && value !== null && (!value.parentNode || value.parentNode === this.__parentPseudo)) {
          value = await value;
          if (!value.node) {
            throw Error(`Node ${childNode.alias} (${childNode.nodeid}) is unavailable`);
          }
          if (value.node.nodegroup_id != node.nodegroup_id && tile && value.tile && (!value.tile.parenttile_id || value.tile.parenttile_id == tile.tileid) || value.node.nodegroup_id == node.nodegroup_id && tile && value.tile == tile && !childNode.is_collector) {
            children.set(key, value);
          } else if (node.nodegroup_id != value.node.nodegroup_id && childNode.is_collector) {
            const childValue = value instanceof PseudoList ? value : value.isIterable() ? await value.getValue() : null;
            let listValue;
            if (childValue && Array.isArray(childValue)) {
              listValue = childValue;
            } else {
              listValue = null;
            }
            if (listValue !== null) {
              if (children.has(key)) {
                children.get(key).push(...listValue);
              } else {
                children.set(key, listValue);
              }
            } else {
              children.set(key, value);
            }
          }
        }
      }
    }
    for (const [key, value] of [...children.entries()]) {
      value.parentNode = this.__parentPseudo;
      this.__childValues.set(key, value);
    }
    return children;
  }
};
let SemanticViewModel = _SemanticViewModel;
const CUSTOM_DATATYPES = /* @__PURE__ */ new Map();
async function getViewModel(parentPseudo, tile, node, data, parent, childNodes) {
  let vm;
  const cacheEntries = parentPseudo.parent && parentPseudo.parent._ ? await parentPseudo.parent._.getValueCache(false, void 0) : void 0;
  let cacheEntry = null;
  if (cacheEntries) {
    cacheEntry = (tile.tileid ? cacheEntries[tile.tileid] ?? {} : {})[node.nodeid];
  }
  const datatype = CUSTOM_DATATYPES.get(node.datatype) ?? node.datatype;
  if (!(typeof datatype == "string")) {
    vm = await datatype.__create(tile, node, data, cacheEntry);
  } else {
    switch (datatype) {
      case "semantic":
        vm = await SemanticViewModel.__create(
          tile,
          node,
          data,
          parent,
          childNodes
        );
        break;
      case "domain-value":
        vm = await DomainValueViewModel.__create(tile, node, data);
        break;
      case "domain-value-list":
        vm = await DomainValueListViewModel.__create(tile, node, data);
        break;
      case "concept":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptValueCacheEntry)) {
          cacheEntry = new ConceptValueCacheEntry(cacheEntry);
        }
        vm = await ConceptValueViewModel.__create(tile, node, data, cacheEntry);
        break;
      case "resource-instance":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceCacheEntry)) {
          cacheEntry = new ResourceInstanceCacheEntry(cacheEntry);
        }
        vm = await ResourceInstanceViewModel.__create(tile, node, data, cacheEntry);
        break;
      case "resource-instance-list":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceListCacheEntry)) {
          cacheEntry = new ResourceInstanceListCacheEntry(cacheEntry);
        }
        vm = await ResourceInstanceListViewModel.__create(tile, node, data, cacheEntry);
        break;
      case "concept-list":
        if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptListCacheEntry)) {
          cacheEntry = new ConceptListCacheEntry(cacheEntry);
        }
        vm = await ConceptListViewModel.__create(tile, node, data, cacheEntry);
        break;
      case "date":
        vm = await DateViewModel.__create(tile, node, data);
        break;
      case "geojson-feature-collection":
        vm = await GeoJSONViewModel.__create(tile, node, data);
        break;
      case "boolean":
        vm = await BooleanViewModel.__create(tile, node, data);
        break;
      case "string":
        vm = await StringViewModel.__create(tile, node, data);
        break;
      case "number":
        vm = await NumberViewModel.__create(tile, node, data);
        break;
      case "url":
        vm = await UrlViewModel.__create(tile, node, data);
        break;
      case "non-localized-string":
        vm = await NonLocalizedStringViewModel.__create(tile, node, data);
        break;
      default:
        console.warn("Missing type for tile", tile.tileid, "on node", node.alias, "with type", node.datatype);
        vm = await NonLocalizedStringViewModel.__create(tile, node, data);
    }
  }
  if (vm === null) {
    return null;
  }
  vm.__parentPseudo = parentPseudo;
  if (vm instanceof Array) {
    for (const vme of vm) {
      if (vme instanceof Promise) {
        vme.then((vmep) => {
          vmep.__parentPseudo = parentPseudo;
        });
      } else {
        vme.__parentPseudo = parentPseudo;
      }
    }
  }
  return vm;
}
const viewModels = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  BooleanViewModel,
  CUSTOM_DATATYPES,
  ConceptValueViewModel,
  DEFAULT_LANGUAGE,
  DateViewModel,
  DomainValueViewModel,
  GeoJSONViewModel,
  NonLocalizedStringViewModel,
  NumberViewModel,
  ResourceInstanceCacheEntry,
  ResourceInstanceViewModel,
  SemanticViewModel,
  StringViewModel,
  UrlViewModel,
  ValueList,
  getViewModel,
  viewContext
}, Symbol.toStringTag, { value: "Module" }));
class PseudoUnavailable {
  constructor(node) {
    __publicField(this, "parentNode", null);
    __publicField(this, "tile", null);
    __publicField(this, "node");
    this.node = node;
  }
  async forJson() {
    return null;
  }
  describeField() {
    return "Unavailable field";
  }
  describeFieldGroup() {
    return "Unavailable field";
  }
  async getValue() {
    return null;
  }
  getLength() {
    return 0;
  }
  getChildren(_ = false) {
    return [];
  }
  isIterable() {
    return false;
  }
}
const ITERABLE_DATATYPES = [
  "concept-list",
  "resource-instance-list",
  "domain-value-list"
];
class PseudoValue {
  constructor(node, tile, value, parent, childNodes) {
    __publicField(this, "node");
    __publicField(this, "tile");
    __publicField(this, "value");
    __publicField(this, "parent");
    __publicField(this, "parentNode");
    __publicField(this, "valueLoaded", false);
    __publicField(this, "datatype", null);
    __publicField(this, "originalTile");
    __publicField(this, "accessed");
    __publicField(this, "childNodes");
    this.node = node;
    this.tile = tile;
    if (!parent) {
      throw Error("Must have a parent or parent class for a pseudo-node");
    }
    this.parent = parent;
    this.parentNode = null;
    this.childNodes = childNodes;
    this.value = value;
    this.accessed = false;
    this.originalTile = tile;
    this.datatype = node.datatype;
  }
  isIterable() {
    return this.datatype !== null && ITERABLE_DATATYPES.includes(this.datatype);
  }
  describeField() {
    let fieldName = this.node.name;
    if (this.parent && this.parent.__) {
      fieldName = `${this.parent.__.wkrm.modelName} - ${fieldName}`;
    }
    return fieldName;
  }
  describeFieldGroup() {
    let fieldName = this.node.name;
    if (this.parent && this.node.nodegroup_id && this.parent._) {
      const nodegroup = this.parent._.model.getNodeObjects().get(this.node.nodegroup_id);
      if (nodegroup && this.parent.__) {
        fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroup.name}`;
      }
    }
    return fieldName;
  }
  // TODO deepcopy
  //
  getParentTileId() {
    return this.tile ? this.tile.parenttile_id : null;
  }
  async getTile() {
    await this.updateValue();
    const relationships = [];
    let tileValue;
    if (this.value !== null) {
      tileValue = await this.value.__asTileData();
    } else {
      tileValue = this.value;
    }
    if (!this.tile) {
      throw Error();
    }
    this.tile.data = this.tile.data || {};
    if (tileValue === null) {
      if (this.tile.data.get(this.node.nodeid)) {
        this.tile.data.delete(this.node.nodeid);
      }
    } else {
      this.tile.data.set(this.node.nodeid, tileValue);
    }
    const tile = this.node.is_collector ? this.tile : null;
    return [tile, relationships];
  }
  clear() {
    this.value = null;
    if (this.tile && this.tile.data && this.tile.data.has(this.node.nodeid)) {
      this.tile.data.delete(this.node.nodeid);
    }
  }
  updateValue() {
    this.accessed = true;
    if (!this.tile) {
      if (!this.node) {
        throw Error("Empty tile");
      }
      this.tile = new StaticTile({
        nodegroup_id: this.node.nodegroup_id || "",
        tileid: null,
        data: /* @__PURE__ */ new Map(),
        sortorder: this.node.sortorder,
        resourceinstance_id: "",
        parenttile_id: null,
        provisionaledits: null,
        ensureId: () => ""
      });
    }
    if (this.valueLoaded === false) {
      this.valueLoaded = void 0;
      let data;
      if (this.value === null && this.tile.data !== null && this.tile.data.has(this.node.nodeid)) {
        data = this.tile.data.get(this.node.nodeid);
      } else {
        data = this.value;
      }
      const vm = getViewModel(
        this,
        this.tile,
        this.node,
        data,
        this.parent,
        this.childNodes
      );
      const resolveAttr = (vm2) => {
        if (vm2 !== null && vm2 instanceof Object) {
          vm2.__parentPseudo = this;
          this.valueLoaded = true;
        }
        return vm2;
      };
      this.value = new AttrPromise((resolve) => {
        vm.then((vm2) => resolve(vm2 ? resolveAttr(vm2) : vm2));
      });
    }
    return this.value;
  }
  getValue() {
    return this.updateValue();
  }
  // @value.setter
  // def value(this, value):
  getLength() {
    return this.getChildren().length;
  }
  async getChildTypes() {
    await this.updateValue();
    if (this.value instanceof Object) {
      try {
        return this.value.get_child_types();
      } catch (AttributeError) {
      }
    }
    return {};
  }
  getChildren(direct = null) {
    if (this.value) {
      try {
        return this.value.getChildren(direct);
      } catch (AttributeError) {
      }
    }
    return [];
  }
  async forJson() {
    const value = await this.getValue();
    return value instanceof Object ? value.forJson() : value;
  }
}
class PseudoList extends Array {
  constructor() {
    super(...arguments);
    __publicField(this, "node");
    __publicField(this, "parent");
    __publicField(this, "parentNode", null);
    __publicField(this, "tile");
    __publicField(this, "parenttileId");
    __publicField(this, "ghostChildren", null);
  }
  isIterable() {
    return true;
  }
  async sorted() {
    const resolved = await Promise.all(this.map(async (pn) => await pn));
    const sorted = resolved.sort((a, b) => {
      const vals = [a, b].map((val) => {
        if (val && a.__parentPseudo && a.__parentPseudo.tile) {
          if (val.__parentPseudo.tile.sortorder > 0) ;
          return val.__parentPseudo.tile.sortorder;
        } else {
          return 0;
        }
      });
      return vals[0] - vals[1];
    });
    return sorted;
  }
  describeField() {
    if (!this.node) {
      return "[(uninitialized node)]";
    }
    let fieldName = this.node.name;
    if (this.parent && this.parent.__) {
      fieldName = `${this.parent.__.wkrm.modelName} - ${fieldName}`;
    }
    return `[${fieldName}]`;
  }
  describeFieldGroup() {
    if (!this.node) {
      return "[(uninitialized node)]";
    }
    let fieldName = this.node.name;
    if (this.parent && this.node.nodegroup_id && this.parent._) {
      const nodegroup = this.parent._.model.getNodeObjects().get(this.node.nodegroup_id);
      if (nodegroup && this.parent.__) {
        fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroup.name}`;
      }
    }
    return `[${fieldName}]`;
  }
  // Otherwise interferes with Array methods;
  initialize(node, parent) {
    this.node = node;
    if (Array.isArray(this.node)) {
      throw Error("Cannot make a list of lists");
    }
    if (!parent) {
      throw Error("Must have a parent or parent class for a pseudo-node");
    }
    this.parent = parent;
    this.tile = void 0;
    this.parenttileId = void 0;
    this.ghostChildren = /* @__PURE__ */ new Set();
  }
  async forJson() {
    const array = Array.from(
      (await this.sorted()).map(
        async (entry) => {
          const value = await entry;
          return value && value instanceof Object && value.forJson ? value.forJson() : value;
        }
      )
    );
    return Promise.all(array);
  }
  getValue() {
    return new AttrPromise((resolve) => resolve(this));
  }
  toString() {
    return `<PL: ${this.length}>`;
  }
}
function makePseudoCls(model, key, single, tile = null, wkri = null) {
  const nodeObjs = model.getNodeObjectsByAlias();
  const nodeObj = nodeObjs.get(key);
  if (!nodeObj) {
    throw Error("Could not find node by alias");
  }
  const nodegroups = model.getNodegroupObjects();
  const nodegroup = nodegroups.get(nodeObj.nodegroup_id || "");
  let value = null;
  if (nodeObj.nodegroup_id && nodeObj.is_collector && nodegroup && nodegroup.cardinality == "n" && true) {
    value = new PseudoList();
    value.initialize(nodeObj, wkri);
  }
  if (value === null || tile) {
    let nodeValue;
    const isPermitted = model.isNodegroupPermitted(nodeObj.nodegroup_id || "", tile, nodeObjs);
    if (isPermitted) {
      const childNodes = model.getChildNodes(nodeObj.nodeid);
      nodeValue = new PseudoValue(nodeObj, tile, null, wkri, childNodes);
    } else {
      nodeValue = new PseudoUnavailable(nodeObj);
    }
    if (value) {
      value.push(nodeValue.getValue());
    } else {
      value = nodeValue;
    }
  }
  return value;
}
const MAX_GRAPH_DEPTH = 100;
class WKRM {
  constructor(meta) {
    __publicField(this, "modelName");
    __publicField(this, "modelClassName");
    __publicField(this, "graphId");
    __publicField(this, "meta");
    let name;
    if (meta.name instanceof Object) {
      name = meta.name[DEFAULT_LANGUAGE].toString();
    } else {
      name = meta.name;
    }
    this.modelName = name || "Unnamed";
    this.graphId = meta.graphid;
    this.modelClassName = (meta.slug || this.modelName).replace(/[_-]/g, " ").replace(/\s(.)/g, (c) => c.toUpperCase()).replace(/\s/g, "");
    this.modelClassName = this.modelClassName[0].toUpperCase() + this.modelClassName.slice(1);
    this.meta = meta;
  }
}
class ConfigurationOptions {
  constructor() {
    __publicField(this, "graphs");
    this.graphs = null;
  }
}
class ResourceInstanceWrapper {
  constructor(wkri, model, resource) {
    __publicField(this, "wkri");
    __publicField(this, "model");
    __publicField(this, "resource");
    __publicField(this, "valueList");
    __publicField(this, "cache");
    this.wkri = wkri;
    this.model = model;
    if (resource) {
      this.model.stripTiles(resource);
    }
    this.resource = resource;
    this.valueList = new ValueList(/* @__PURE__ */ new Map(), this, []);
    this.cache = resource ? resource.__cache : void 0;
    this.scopes = resource ? resource.__scopes : void 0;
    this.metadata = resource ? resource.metadata : void 0;
  }
  async loadNodes(aliases) {
    for (const key of aliases) {
      await this.valueList.retrieve(key);
    }
  }
  addPseudo(childNode, tile) {
    const key = childNode.alias;
    if (!key) {
      throw Error(`Cannot add a pseudo node with no alias ${childNode.nodeid}`);
    }
    const child = makePseudoCls(
      this.model,
      key,
      false,
      !childNode.is_collector ? tile : null,
      // Does it share a tile
      this.wkri
    );
    const valueList = this.valueList;
    valueList.setDefault(key, []).then((val) => val.push(child));
    return child;
  }
  allEntries() {
    return this.valueList.values.entries();
  }
  async keys() {
    return (await this.getRootViewModel()).keys();
  }
  async values() {
    return (await this.getRootViewModel()).values();
  }
  async entries() {
    return (await this.getRootViewModel()).entries();
  }
  async getRootViewModel() {
    const root = await this.getRoot();
    let value = null;
    if (root) {
      const rootValue = await root.getValue();
      if (!Array.isArray(rootValue)) {
        value = rootValue;
      }
    }
    if (!value || !(value instanceof SemanticViewModel)) {
      throw Error(`Tried to get root on ${this}, which has no root`);
    }
    return value;
  }
  getOrmAttribute(key) {
    let promise;
    if (this.resource === null) {
      promise = this.model.findStatic(this.wkri.id).then((resource) => {
        this.resource = resource;
      }).then(() => this.populate(true));
    } else {
      promise = new Promise((resolve) => {
        resolve();
      });
    }
    return promise.then(() => this.getRootViewModel()).then((root) => root[key]);
  }
  async getRoot() {
    const values = this.valueList;
    const node = this.model.getRootNode();
    if (node) {
      let value;
      const alias = node.alias;
      if (!(typeof alias == "string")) {
        throw Error(`Alias missing for node ${node.nodeid}`);
      }
      await values.setDefault(alias, []);
      const nodeValues = await values.get(alias);
      if (nodeValues.length > 1) {
        throw Error("Cannot have multiple root tiles");
      } else if (nodeValues.length == 1) {
        value = nodeValues[0];
      } else {
        value = makePseudoCls(this.model, alias, false, null, this.wkri);
        values.set(alias, [value]);
      }
      return value;
    }
  }
  setOrmAttribute(key, value) {
    return this.getRootViewModel().then((root) => {
      if (root) {
        root[key] = value;
      } else {
        throw Error(`Tried to set ${key} on ${self}, which has no root`);
      }
    });
  }
  async ensureNodegroup(allValues, node, nodegroupId, nodeObjs, nodegroupObjs, edges, addIfMissing, tiles, doImpliedNodegroups = true) {
    const alias = node.alias || "";
    const impliedNodegroups = /* @__PURE__ */ new Map();
    if (!node) {
      return [allValues, impliedNodegroups];
    }
    const value = node && await allValues.get(alias);
    let newAllValues = allValues;
    if (value === false || addIfMissing && value === void 0) {
      if (newAllValues.has(alias)) {
        newAllValues.delete(alias);
      }
      let nodegroupTiles;
      if (tiles === null) {
        nodegroupTiles = [];
        console.error("Tiles must be provided and cannot be lazy-loaded yet");
      } else {
        nodegroupTiles = tiles.filter(
          (tile) => tile.nodegroup_id == nodegroupId && this.model.isNodegroupPermitted(nodegroupId, tile)
        );
        if (nodegroupTiles.length == 0 && addIfMissing) {
          nodegroupTiles = [null];
        }
        const rgValues = await this.valuesFromResourceNodegroup(
          newAllValues,
          nodegroupTiles,
          nodegroupId,
          nodeObjs,
          edges
        );
        const newValues = rgValues[0];
        const newImpliedNodegroups = rgValues[1];
        [...newValues.entries()].forEach((entry) => {
          if (entry[1] !== void 0) {
            newAllValues.set(entry[0], entry[1]);
          }
        });
        [...newImpliedNodegroups.entries()].forEach(([k, v]) => {
          impliedNodegroups.set(k, v);
        });
      }
    }
    if (doImpliedNodegroups) {
      for (const [nodegroupId2, node2] of [...impliedNodegroups.entries()]) {
        const [impliedValues] = await this.ensureNodegroup(
          newAllValues,
          node2,
          nodegroupId2,
          nodeObjs,
          nodegroupObjs,
          edges,
          true,
          tiles,
          // RMV different from Python
          true
        );
        newAllValues = impliedValues;
      }
      impliedNodegroups.clear();
    }
    return [newAllValues, impliedNodegroups];
  }
  async populate(lazy) {
    const nodeObjs = this.model.getNodeObjects();
    const nodegroupObjs = this.model.getNodegroupObjects();
    const edges = this.model.getEdges();
    const allValues = new Map(
      [...nodegroupObjs.keys()].map((id) => {
        const node = nodeObjs.get(id);
        if (!node) {
          throw Error(`Could not find node for nodegroup ${id}`);
        }
        return [node.alias || "", false];
      })
    );
    const rootNode = this.model.getRootNode();
    if (rootNode.alias === null) {
      throw Error("Cannot populate a model with no proper root node");
    }
    allValues.set(rootNode.alias, false);
    let tiles = null;
    if (!lazy && this.resource) {
      tiles = this.resource.tiles;
      let impliedNodegroups = /* @__PURE__ */ new Map();
      for (const [ng] of nodegroupObjs) {
        const [values, newImpliedNodegroups] = await this.ensureNodegroup(
          allValues,
          nodeObjs.get(ng) || rootNode,
          // should be the only missing ID
          ng,
          nodeObjs,
          nodegroupObjs,
          edges,
          true,
          // RMV: check vs python
          tiles,
          false
        );
        for (const [key, value] of [...values.entries()]) {
          allValues.set(key, value);
        }
        for (const [impliedNodegroup, impliedNode] of [...newImpliedNodegroups.entries()]) {
          impliedNodegroups.set(impliedNodegroup, impliedNode);
        }
        impliedNodegroups.delete(ng);
      }
      while (impliedNodegroups.size) {
        const newImpliedNodegroups = /* @__PURE__ */ new Map();
        for (const [nodegroupId, impliedNode] of [...impliedNodegroups.entries()]) {
          const currentValue = allValues.get(impliedNode.nodeid);
          if (currentValue === false || currentValue === void 0) {
            const [impliedValues, newImpliedNodegroups2] = await this.ensureNodegroup(
              allValues,
              impliedNode,
              nodegroupId,
              nodeObjs,
              nodegroupObjs,
              edges,
              true,
              tiles,
              // RMV different from Python
              true
            );
            for (const [impliedNodegroup, impliedNode2] of [...newImpliedNodegroups2.entries()]) {
              newImpliedNodegroups2.set(impliedNodegroup, impliedNode2);
            }
            for (const [key, value] of [...impliedValues.entries()]) {
              allValues.set(key, value);
            }
          }
        }
        impliedNodegroups = newImpliedNodegroups;
      }
    } else if (this.resource) {
      this.model.stripTiles(this.resource);
    }
    this.valueList = new ValueList(
      allValues,
      this,
      this.resource ? this.resource.tiles : null
    );
  }
  async getValueCache(build = true, getMeta = void 0) {
    if (build) {
      this.cache = await this.buildValueCache(getMeta);
    }
    return this.cache;
  }
  async buildValueCache(getMeta) {
    const cacheByTile = {};
    for (let pseudos of this.valueList.values.values()) {
      pseudos = await pseudos;
      if (pseudos) {
        await Promise.all(pseudos.map(async (pseudo) => {
          const value = await pseudo.getValue();
          if (pseudo.tile && value && !Array.isArray(pseudo)) {
            const cacheJson = await value.__forJsonCache(getMeta);
            if (cacheJson) {
              const tileId = pseudo.tile.ensureId();
              const nodeId = pseudo.node.nodeid;
              if (!(tileId in cacheByTile)) {
                cacheByTile[tileId] = {};
              }
              if (!(nodeId in cacheByTile[tileId])) {
                cacheByTile[tileId][nodeId] = {};
              }
              cacheByTile[tileId][nodeId] = cacheJson;
            }
          }
        }));
      }
    }
    return cacheByTile;
  }
  async valuesFromResourceNodegroup(existingValues, nodegroupTiles, nodegroupId, nodeObjs, edges) {
    const allValues = /* @__PURE__ */ new Map();
    const impliedNodegroups = /* @__PURE__ */ new Map();
    const impliedNodes = /* @__PURE__ */ new Map();
    const nodesUnseen = new Set(
      [...nodeObjs.values()].filter((node) => node.nodegroup_id == nodegroupId).map((node) => node.alias)
    );
    const tileNodesSeen = /* @__PURE__ */ new Set();
    const _addPseudo = async (node, tile) => {
      const key = node.alias || "";
      nodesUnseen.delete(node.alias);
      const tileid = tile && tile.tileid;
      if (tileid) {
        tileNodesSeen.add([node.nodeid, tileid]);
      }
      let existing = existingValues.get(key);
      if (existing instanceof Promise) {
        existing = await existing;
      }
      if (existing !== false && existing !== void 0) {
        allValues.set(key, existing);
      }
      if (!allValues.has(key)) {
        allValues.set(key, []);
      }
      const pseudoNode = makePseudoCls(this.model, key, false, tile, this.wkri);
      for (const [domain, ranges] of edges) {
        if (ranges.includes(node.nodeid)) {
          const domainNode = nodeObjs.get(domain);
          if (!domainNode) {
            throw Error("Edge error in graph");
          }
          const toAdd = domainNode.nodegroup_id ? domainNode.nodegroup_id : domainNode.nodeid;
          if (toAdd && toAdd !== nodegroupId) {
            impliedNodegroups.set(toAdd, domainNode);
          }
          if (domainNode.nodegroup_id && domainNode.nodegroup_id !== domainNode.nodeid && tileid && !impliedNodes.has(domainNode.nodeid + tileid)) {
            impliedNodes.set(domainNode.nodeid + tileid, [domainNode, tile]);
          }
          break;
        }
      }
      if (Array.isArray(pseudoNode)) {
        const value = allValues.get(key);
        if (value !== void 0 && value !== false) {
          for (const pseudoList of allValues.get(key)) {
            if (!(pseudoList instanceof PseudoList) || !(pseudoNode instanceof PseudoList)) {
              throw Error(`Should be all lists not ${typeof pseudoList} and ${typeof pseudoNode}`);
            }
            if (pseudoList.parentNode == pseudoNode.parentNode) {
              for (const ps of pseudoNode) {
                pseudoList.push(ps);
              }
              return;
            }
          }
        }
      }
      allValues.get(key).push(pseudoNode);
    };
    for (const tile of nodegroupTiles) {
      const parentNode = nodeObjs.get(nodegroupId);
      if (parentNode === void 0) {
        continue;
      }
      if (!parentNode.nodegroup_id || parentNode.nodegroup_id == nodegroupId) {
        await _addPseudo(parentNode, tile);
      }
      if (tile) {
        const tileNodes = /* @__PURE__ */ new Map();
        for (const [key, value] of [...tile.data.entries()]) {
          tileNodes.set(key, value);
        }
        if (!tileNodes.has(tile.nodegroup_id)) {
          tileNodes.set(tile.nodegroup_id, {});
        }
        for (const [nodeid, nodeValue] of [...tileNodes.entries()]) {
          if (nodeid == nodegroupId) {
            continue;
          }
          const node = nodeObjs.get(nodeid);
          if (!node) {
            throw Error("Unknown node in nodegroup");
          }
          if (nodeValue !== null) {
            await _addPseudo(node, tile);
          }
        }
      }
    }
    while (impliedNodes.size > 0) {
      const value = impliedNodes.entries().next().value;
      if (value) {
        const [node, tile] = value[1];
        if (tile.tileid && !tileNodesSeen.has([node.nodeid, tile.tileid])) {
          await _addPseudo(node, tile);
        }
        impliedNodes.delete(value[0]);
      }
    }
    [...nodesUnseen.keys()].forEach((nodeUnseen) => {
      if (nodeUnseen) {
        allValues.set(nodeUnseen, void 0);
      }
    });
    return [allValues, impliedNodegroups];
  }
}
class ResourceModelWrapper {
  constructor(wkrm, graph, viewModelClass) {
    __publicField(this, "wkrm");
    __publicField(this, "graph");
    __publicField(this, "viewModelClass");
    __publicField(this, "permittedNodegroups");
    __publicField(this, "edges");
    __publicField(this, "nodes");
    __publicField(this, "nodegroups");
    __publicField(this, "nodesByAlias");
    this.wkrm = wkrm;
    this.graph = graph;
    this.viewModelClass = viewModelClass;
  }
  // TODO: Switch to getBranches
  getBranchPublicationIds(accessible) {
    const accessibleOnly = accessible || false;
    const nodes = [...this.graph.nodes.values()];
    return [...nodes.reduce(
      (acc, node) => {
        if (node.sourcebranchpublication_id) {
          if (accessibleOnly) {
            if (this.isNodegroupPermitted(node.nodegroup_id || "", null)) {
              acc.add(node.config.rdmCollection);
            }
          } else {
            acc.add(node.config.rdmCollection);
          }
        }
        return acc;
      },
      /* @__PURE__ */ new Set()
    )];
  }
  getCollections(accessible) {
    const accessibleOnly = accessible || false;
    const nodes = [...this.graph.nodes.values()];
    return [...nodes.reduce(
      (acc, node) => {
        var _a2;
        if (["concept", "concept-list"].includes(node.datatype) && ((_a2 = node.config) == null ? void 0 : _a2.rdmCollection)) {
          if (accessibleOnly) {
            if (this.isNodegroupPermitted(node.nodegroup_id || "", null)) {
              acc.add(node.config.rdmCollection);
            }
          } else {
            acc.add(node.config.rdmCollection);
          }
        }
        return acc;
      },
      /* @__PURE__ */ new Set()
    )];
  }
  pruneGraph(keepFunctions) {
    const allNodegroups = this.getNodegroupObjects();
    const root = this.graph.root.nodeid;
    const allowedNodegroups = new Map([...allNodegroups.values()].filter((nodegroup) => {
      return this.isNodegroupPermitted(nodegroup.nodegroupid || "", null);
    }).map((nodegroup) => [nodegroup.nodegroupid, nodegroup.nodegroupid === null || nodegroup.nodegroupid === "" || nodegroup.nodegroupid === root]));
    const backedges = /* @__PURE__ */ new Map();
    for (const [d, rs] of this.getEdges()) {
      for (const r of rs) {
        if (backedges.has(r)) {
          throw Error(`Graph is malformed, node ${r} has multiple parents, ${backedges.get(r)} and ${d} at least`);
        }
        backedges.set(r, d);
      }
    }
    let loops = 0;
    allowedNodegroups.set(root, true);
    while (loops < MAX_GRAPH_DEPTH) {
      const unrooted = [...allowedNodegroups.entries()].filter(([_, rooted]) => !rooted);
      if (unrooted.length === 0) {
        break;
      }
      for (const [ng] of unrooted) {
        if (ng === root) {
          continue;
        }
        const next = backedges.get(ng);
        if (!next) {
          throw Error(`Graph does not have a parent for ${ng}`);
        }
        allowedNodegroups.set(ng, true);
        if (!allowedNodegroups.has(next)) {
          allowedNodegroups.set(next, false);
        }
      }
      loops += 1;
    }
    if (loops >= MAX_GRAPH_DEPTH) {
      throw Error("Hit edge traversal limit when pruning, is the graph well-formed without cycles?");
    }
    const allowedNodes = new Set([...this.getNodeObjects().values()].filter((node) => {
      return node.nodegroup_id && allowedNodegroups.get(node.nodegroup_id) || node.nodeid === root;
    }).map((node) => node.nodeid));
    this.graph.cards = (this.graph.cards || []).filter((card) => allowedNodegroups.get(card.nodegroup_id));
    this.graph.cards_x_nodes_x_widgets = (this.graph.cards_x_nodes_x_widgets || []).filter((card) => allowedNodes.has(card.node_id));
    this.graph.edges = (this.graph.edges || []).filter((edge) => (edge.domainnode_id === root || allowedNodes.has(edge.domainnode_id)) && allowedNodes.has(edge.rangenode_id));
    this.graph.nodegroups = (this.graph.nodegroups || []).filter((ng) => allowedNodegroups.has(ng.nodegroupid));
    this.graph.nodes = (this.graph.nodes || []).filter((node) => allowedNodes.has(node.nodeid));
    if (Array.isArray(keepFunctions) && this.graph.functions_x_graphs) {
      this.graph.functions_x_graphs = this.graph.functions_x_graphs.filter((fxg) => keepFunctions.includes(fxg.function_id));
    } else {
      this.graph.functions_x_graphs = [];
    }
  }
  exportGraph() {
    const graph = this.graph;
    return new StaticGraph({
      author: graph.author,
      cards: graph.cards,
      cards_x_nodes_x_widgets: graph.cards_x_nodes_x_widgets,
      color: graph.color,
      config: graph.config,
      deploymentdate: graph.deploymentdate,
      deploymentfile: graph.deploymentfile,
      description: graph.description,
      edges: graph.edges,
      functions_x_graphs: graph.functions_x_graphs,
      graphid: graph.graphid,
      iconclass: graph.iconclass,
      is_editable: graph.is_editable,
      isresource: graph.isresource,
      jsonldcontext: graph.jsonldcontext,
      name: graph.name,
      nodegroups: graph.nodegroups,
      nodes: graph.nodes,
      ontology_id: graph.ontology_id,
      publication: graph.publication,
      relatable_resource_model_ids: graph.relatable_resource_model_ids,
      resource_2_resource_constraints: graph.resource_2_resource_constraints,
      root: graph.root,
      slug: graph.slug,
      subtitle: graph.subtitle,
      template_id: graph.template_id,
      version: graph.version
    });
  }
  async all(params = void 0) {
    const paramObj = params || { limit: void 0, lazy: void 0 };
    const promises = [];
    for await (const resource of this.iterAll(paramObj)) {
      promises.push(resource);
    }
    return Promise.all(promises);
  }
  stripTiles(resource) {
    if (resource.tiles) {
      const nodes = this.getNodeObjects();
      resource.tiles = resource.tiles.filter((tile) => {
        const node = nodes.get(tile.nodegroup_id);
        if (!node) {
          throw Error(`Tile ${tile.tileid} has nodegroup ${tile.nodegroup_id} that is not on the model ${this.graph.graphid}`);
        }
        return this.isNodegroupPermitted(tile.nodegroup_id || "", tile);
      });
    }
  }
  async *resourceGenerator(staticResources, lazy = false) {
    for await (const staticResource of staticResources) {
      yield this.fromStaticResource(staticResource, lazy);
    }
  }
  async *iterAll(params) {
    yield* this.resourceGenerator(staticStore.loadAll(this.wkrm.graphId, params.limit), params.lazy);
  }
  async findStatic(id) {
    return await staticStore.loadOne(id);
  }
  async find(id, lazy = true) {
    const rivm = await this.findStatic(id);
    return this.fromStaticResource(rivm, lazy);
  }
  setPermittedNodegroups(permissions) {
    const nodegroups = this.getNodegroupObjects();
    const nodes = this.getNodeObjectsByAlias();
    const nodesById = this.getNodeObjects();
    this.permittedNodegroups = new Map([...permissions].map(([key, value]) => {
      const k = key || "";
      if (nodegroups.has(k) || k === "") {
        return [key, value];
      } else {
        const node = nodes.get(k);
        if (node) {
          if (node.nodeid !== node.nodegroup_id) {
            const nodegroup = nodesById.has(node.nodegroup_id || "") ? `${nodesById.get(node.nodegroup_id).alias} (${node.nodegroup_id})` : node.nodegroup_id;
            console.warn(`Applying a permissions rule to a nodegroup ${nodegroup} that contains, but is not the same as, the node you requested: ${node.alias} (${node.nodeid})`);
          }
          return [node.nodegroup_id, value];
        } else {
          throw Error(`Could not find ${key} in nodegroups for permissions`);
        }
      }
    }));
  }
  // Defaults to visible, which helps reduce the risk of false sense of security
  // from front-end filtering masking the presence of data transferred to it.
  getPermittedNodegroups() {
    if (!this.permittedNodegroups) {
      const permissions = new Map([...this.getNodegroupObjects()].map(
        ([k, _]) => [k, true]
      ));
      permissions.set("", true);
      this.setPermittedNodegroups(permissions);
    }
    const permittedNodegroups = this.permittedNodegroups;
    if (permittedNodegroups === void 0) {
      throw Error("Could not set permitted nodegroups");
    }
    return permittedNodegroups;
  }
  isNodegroupPermitted(nodegroupId, tile) {
    let permitted = this.getPermittedNodegroups().get(nodegroupId);
    if (permitted && typeof permitted == "function") {
      const nodes = this.getNodeObjectsByAlias();
      permitted = permitted(nodegroupId, tile, nodes);
    }
    if (!permitted) {
      return false;
    }
    if (permitted === true) {
      return true;
    }
    throw Error(`Ambiguous permission state: ${permitted} for nodegroup ${nodegroupId}`);
  }
  makeInstance(id, resource) {
    if (!this.viewModelClass) {
      throw Error(`Cannot instantiate without a viewModelClass in ${this.wkrm.modelClassName}`);
    }
    const instance = new this.viewModelClass(
      id,
      this.viewModelClass.prototype.__,
      (rivm) => new ResourceInstanceWrapper(rivm, this, resource),
      null
    );
    return instance;
  }
  getChildNodes(nodeId) {
    const childNodes = /* @__PURE__ */ new Map();
    const edges = this.getEdges().get(nodeId);
    if (edges) {
      for (const [, n] of this.getNodeObjects()) {
        if (edges.includes(n.nodeid)) {
          if (n.alias) {
            childNodes.set(n.alias, n);
          }
        }
      }
    }
    return childNodes;
  }
  buildNodes() {
    if (this.nodes || this.nodegroups) {
      throw Error("Cache should never try and rebuild nodes when non-empty");
    }
    this.edges = /* @__PURE__ */ new Map();
    this.nodes = /* @__PURE__ */ new Map();
    this.nodegroups = /* @__PURE__ */ new Map();
    const graph = this.graph ?? graphManager.getGraph(this.wkrm.graphId);
    if (!graph) {
      throw Error(`Could not find graph ${this.wkrm.graphId} for resource`);
    }
    const nodes = new Map(graph.nodes.map((node) => [node.nodeid, node]));
    const nodegroups = new Map(
      graph.nodes.filter((node) => node.nodegroup_id).map((node) => [
        node.nodegroup_id || "",
        new StaticNodegroup({
          cardinality: "n",
          legacygroupid: null,
          nodegroupid: node.nodegroup_id || "",
          parentnodegroup_id: null
        })
      ])
    );
    for (const nodegroup of graph.nodegroups) {
      nodegroups.set(nodegroup.nodegroupid, nodegroup);
    }
    const edgePairs = graph.edges.map((edge) => [
      edge.domainnode_id,
      edge.rangenode_id
    ]);
    const edges = edgePairs.reduce((edges2, dr) => {
      const range = edges2.get(dr[0]) || [];
      range.push(dr[1]);
      edges2.set(dr[0], range);
      return edges2;
    }, /* @__PURE__ */ new Map());
    this.nodes = nodes;
    this.nodegroups = nodegroups;
    this.edges = edges;
    this.nodesByAlias = new Map(
      [...nodes.values()].map((node) => [node.alias || "", node])
    );
  }
  getNodeObjectsByAlias() {
    if (!this.nodesByAlias) {
      this.buildNodes();
    }
    if (!this.nodesByAlias) {
      throw Error("Could not build nodes");
    }
    return this.nodesByAlias;
  }
  getEdges() {
    if (!this.edges) {
      this.buildNodes();
    }
    if (!this.edges) {
      throw Error("Could not build edges");
    }
    return this.edges;
  }
  getNodeObjects() {
    if (!this.nodes) {
      this.buildNodes();
    }
    if (!this.nodes) {
      throw Error("Could not build nodes");
    }
    return this.nodes;
  }
  getNodegroupObjects() {
    if (!this.nodegroups) {
      this.buildNodes();
    }
    if (!this.nodegroups) {
      throw Error("Could not build nodegroups");
    }
    return this.nodegroups;
  }
  getRootNode() {
    const nodes = this.getNodeObjects();
    const rootNode = [...nodes.values()].find((n) => !n.nodegroup_id);
    if (!rootNode) {
      throw Error(
        `COULD NOT FIND ROOT NODE FOR ${this.wkrm.modelClassName}. Does the graph ${this.graph.graphid} still exist?`
      );
    }
    rootNode.alias = rootNode.alias || "";
    return rootNode;
  }
  fromStaticResource(resource, lazy = false) {
    const wkri = this.makeInstance(
      resource.resourceinstance.resourceinstanceid,
      resource
    );
    if (!wkri._) {
      throw Error("Could not load resource from static definition");
    }
    return wkri._.populate(lazy).then(() => wkri);
  }
}
function makeResourceModelWrapper(viewModelClass, wkrm, graph) {
  var _a2;
  let vmc;
  if (!viewModelClass) {
    const viewModelClassObj = {
      [wkrm.modelClassName]: (_a2 = class extends ResourceInstanceViewModel {
      }, __publicField(_a2, "_"), __publicField(_a2, "__"), _a2)
    };
    vmc = viewModelClassObj[wkrm.modelClassName];
  } else {
    vmc = viewModelClass;
  }
  const wrapper = new ResourceModelWrapper(wkrm, graph, vmc);
  vmc.prototype.__ = wrapper;
  return vmc;
}
class GraphManager {
  constructor(archesClient2) {
    __publicField(this, "_initialized", false);
    __publicField(this, "archesClient");
    __publicField(this, "graphs");
    __publicField(this, "wkrms");
    this.archesClient = archesClient2;
    this.graphs = /* @__PURE__ */ new Map();
    this.wkrms = /* @__PURE__ */ new Map();
  }
  async initialize(configurationOptions = void 0) {
    if (this._initialized) {
      return;
    }
    if (configurationOptions === void 0) {
      configurationOptions = new ConfigurationOptions();
    }
    const graphJsons = await this.archesClient.getGraphs();
    Object.entries(graphJsons["models"]).forEach(([graphId, meta]) => {
      meta.graphid = meta.graphid || graphId;
      const wkrm = new WKRM(meta);
      this.wkrms.set(wkrm.modelClassName, wkrm);
    });
    let graphs = Object.keys(graphJsons["models"]);
    const allowedGraphs = configurationOptions.graphs;
    if (allowedGraphs !== null) {
      if (allowedGraphs === false) {
        throw Error("No current meaning of allowedGraphs === false");
      } else if (allowedGraphs !== true) {
        graphs = graphs.filter(
          (graphId) => allowedGraphs.includes(graphId)
        );
      }
      await Promise.all(graphs.map((g) => this.loadGraph(g)));
    }
    this._initialized = true;
  }
  async loadGraph(modelClass) {
    let modelClassName;
    if (typeof modelClass == "string") {
      modelClassName = modelClass;
    } else {
      modelClassName = modelClass.name;
    }
    const wkrm = this.wkrms.get(modelClassName);
    if (wkrm === void 0) {
      throw Error(`Only loading graphs for which metadata is present, not ${modelClassName}`);
    }
    const bodyJson = await this.archesClient.getGraph(wkrm.graphId);
    if (!bodyJson) {
      throw Error(`Could not load graph ${wkrm.graphId}`);
    }
    const graph = new StaticGraph(bodyJson);
    let model;
    if (typeof modelClass == "string") {
      modelClassName = modelClass;
      model = makeResourceModelWrapper(void 0, wkrm, graph);
    } else {
      modelClassName = modelClass.name;
      model = makeResourceModelWrapper(modelClass, wkrm, graph);
    }
    this.graphs.set(graph.graphid, model.prototype.__);
    return model.prototype.__;
  }
  async get(modelClass) {
    let modelClassName;
    if (typeof modelClass == "string") {
      modelClassName = modelClass;
    } else {
      modelClassName = modelClass.name;
    }
    this.initialize(void 0);
    const wkrm = this.wkrms.get(modelClassName);
    if (wkrm === void 0) {
      throw Error(`Cannot find model requested: ${modelClassName}`);
    }
    const wrapper = this.graphs.get(wkrm.graphId);
    if (wrapper === void 0) {
      return this.loadGraph(modelClass);
    }
    return wrapper;
  }
  async getResource(resourceId, lazy = true) {
    const rivm = await staticStore.loadOne(resourceId);
    const graph = this.graphs.get(rivm.resourceinstance.graph_id);
    if (!graph) {
      throw Error(`Graph not found for resource ${resourceId}`);
    }
    return graph.fromStaticResource(rivm, lazy);
  }
  getGraph(graphId) {
    const wrapper = this.graphs.get(graphId);
    if (wrapper === void 0) {
      throw Error(`Cannot find graph requested: ${graphId}`);
    }
    return wrapper.graph;
  }
}
const graphManager = new GraphManager(archesClient);
viewContext.graphManager = graphManager;
class Cleanable extends String {
  constructor() {
    super(...arguments);
    __publicField(this, "__clean");
  }
}
class BaseRenderer {
  async render(asset) {
    if (!asset._) {
      throw Error("Cannot render unloaded asset - do you want to await asset.retrieve()?");
    }
    const root = await await asset._.getRootViewModel();
    return this.renderValue(root, 0);
  }
  async renderValue(value, depth) {
    let newValue;
    if (value instanceof Promise) {
      value = await value;
    }
    if (value instanceof DomainValueViewModel) {
      newValue = this.renderDomainValue(value, depth);
    } else if (value instanceof DateViewModel) {
      newValue = this.renderDate(value, depth);
    } else if (value instanceof ConceptValueViewModel) {
      newValue = this.renderConceptValue(value, depth);
    } else if (value instanceof ResourceInstanceViewModel) {
      newValue = this.renderResourceReference(value, depth);
    } else if (value instanceof SemanticViewModel) {
      newValue = this.renderSemantic(value, depth);
    } else if (value instanceof Array) {
      newValue = this.renderArray(value, depth);
    } else if (value instanceof StringViewModel || value instanceof NonLocalizedStringViewModel || typeof value === "string") {
      newValue = this.renderString(value, depth);
    } else if (value instanceof BooleanViewModel) {
      newValue = this.renderBoolean(value, depth);
    } else if (value instanceof NumberViewModel) {
      newValue = this.renderNumber(value, depth);
    } else if (value instanceof GeoJSONViewModel) {
      newValue = this.renderBlock(await value.forJson(), depth);
    } else if (value instanceof UrlViewModel) {
      newValue = this.renderUrl(await value, depth);
    } else if (value instanceof Object) {
      newValue = this.renderBlock(value, depth);
    } else {
      newValue = value;
    }
    return newValue;
  }
}
class Renderer extends BaseRenderer {
  async renderDomainValue(value, _depth) {
    return value;
  }
  async renderString(value, _depth) {
    return `${value}`;
  }
  async renderNumber(value, _depth) {
    return `${value}`;
  }
  async renderBoolean(value, _depth) {
    return value.toString();
  }
  async renderDate(value, _depth) {
    return value;
  }
  async renderConceptValue(value, _depth) {
    return value;
  }
  async renderResourceReference(value, _depth) {
    return value;
  }
  async renderSemantic(value, depth) {
    return this.renderBlock(await value.toObject(), depth);
  }
  async renderUrl(value, _depth) {
    return value;
  }
  renderBlock(block, depth) {
    const renderedBlock = {};
    const promises = [];
    for (const [key, value] of Object.entries(block)) {
      promises.push(
        this.renderValue(value, depth + 1).then((val) => {
          renderedBlock[key] = val;
        })
      );
    }
    return Promise.all(promises).then(() => renderedBlock);
  }
  async renderArray(value, depth) {
    return Promise.all(value.map((val) => this.renderValue(val, depth + 1)));
  }
}
class MarkdownRenderer extends Renderer {
  constructor(callbacks) {
    super();
    __publicField(this, "conceptValueToUrl");
    __publicField(this, "dateToText");
    __publicField(this, "domainValueToUrl");
    __publicField(this, "resourceReferenceToUrl");
    __publicField(this, "nodeToUrl");
    this.conceptValueToUrl = callbacks.conceptValueToUrl;
    this.dateToText = callbacks.dateToText;
    this.domainValueToUrl = callbacks.domainValueToUrl;
    this.resourceReferenceToUrl = callbacks.resourceReferenceToUrl;
    this.nodeToUrl = callbacks.nodeToUrl;
  }
  async renderUrl(value, _depth) {
    return `[${value}](${value})`;
  }
  async renderDomainValue(domainValue, _) {
    const value = await domainValue.getValue();
    const url = this.domainValueToUrl ? await this.domainValueToUrl(domainValue) : null;
    const text = url ? `[${value.toString()}](${url.trim()})` : value.toString();
    const wrapper = new Cleanable(`
    <span
      class='alizarin-domain-value' data-id='${value.id}'
    >
      ${text}
    </span>`.replace(/\n/g, " ").trim());
    wrapper.__clean = domainValue.toString();
    return wrapper;
  }
  async renderDate(date, _) {
    const value = await date;
    const text = this.dateToText ? await this.dateToText(value) : value.toISOString();
    const wrapper = new Cleanable(`
    <time datetime='${text}'>
      ${text}
    </time>`.replace(/\n/g, " ").trim());
    wrapper.__clean = text;
    return wrapper;
  }
  async renderConceptValue(conceptValue, _) {
    const value = await conceptValue.getValue();
    const url = this.conceptValueToUrl ? await this.conceptValueToUrl(conceptValue) : null;
    const text = url ? `[${value.value}](${url.trim()})` : value.value;
    const wrapper = new Cleanable(`
    <span
      class='alizarin-concept-value' data-id='${value.id}'
      data-concept-id='${value.__concept ? value.__concept.id : ""}'
      data-concept-ref='$${value.__concept ? value.__concept.source : ""}'
    >
      ${text}
    </span>`.replace(/\n/g, " ").trim());
    wrapper.__clean = conceptValue.toString();
    return wrapper;
  }
  async renderResourceReference(rivm, _) {
    const value = await rivm.forJson(false);
    const url = this.resourceReferenceToUrl ? await this.resourceReferenceToUrl(rivm) : null;
    let title = value.title || value.type || "Resource";
    const text = url ? `[${title}](${url.trim()})` : title;
    const resourceMetadata = await staticStore.getMeta(value.id);
    if (resourceMetadata) {
      title = resourceMetadata.name;
    }
    const wrapper = new Cleanable(`
    <span
      class='alizarin-resource-instance alizarin-related-resource' data-id='${value.id}'
      data-graph-id='${value.graphId}'
    >
      ${text}
    </span>`.replace(/\n/g, " ").trim());
    wrapper.__clean = rivm.toString();
    return wrapper;
  }
}
class FlatMarkdownRenderer extends MarkdownRenderer {
  async renderSemantic(vm, depth) {
    const children = [...(await vm.__getChildValues()).entries()].map(([_, v]) => [v.node.alias, v.node]);
    const nodes = Object.fromEntries(await Promise.all(children));
    return super.renderSemantic(vm, depth).then(async (block) => {
      const text = [
        `* <span class='node-type'>${vm.__node.name}</span> &rarr;`,
        ...Object.entries(await block).map(([key, value]) => {
          const node = nodes[key];
          let nodeName = node.name;
          if (this.nodeToUrl) {
            nodeName = `[${node.name}](${this.nodeToUrl(node)})`;
          }
          if ((typeof value == "string" || value instanceof String) && value.indexOf("\n") != -1) {
            return `  * <span class='node-name'>${nodeName}</span> <span class='node-alias'>[*${node.alias}*]</span>:<span class='node-value'>
${value.split("\n").map((x) => `    ${x}`).join("\n")}
    </span>`;
          } else {
            return `  * <span class='node-name'>${nodeName}</span> <span class='node-alias'>[*${node.alias}*]</span>: <span class='node-value'>${value}</span>`;
          }
        }).join("\n").split("\n")
      ];
      if (text[1] == "") {
        text[0] += `<span class='node-empty'>&lt;empty&gt;</span>`;
        text.pop();
      }
      return text.map((line) => `  ${line}`).join("\n");
    });
  }
  async renderArray(value, depth) {
    const rows = await super.renderArray(value, depth);
    if (value instanceof PseudoList || value.indexOf("\n") != -1) {
      return rows.map((x) => `${x}`).join("\n");
    } else {
      return rows.join(", ");
    }
  }
  async renderString(value, _depth) {
    if (value.indexOf("\n") != -1) {
      value = "\n    " + value.split("\n").join("\n    ");
    }
    return value;
  }
}
class JsonRenderer extends Renderer {
  async renderDate(value, _depth) {
    return value.forJson();
  }
  async renderConceptValue(value, _depth) {
    return value.forJson();
  }
  async renderDomainValue(value, _depth) {
    return value.forJson();
  }
  async renderResourceReference(value, _depth) {
    return value.forJson();
  }
}
const renderers = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  Cleanable,
  FlatMarkdownRenderer,
  JsonRenderer,
  MarkdownRenderer
}, Symbol.toStringTag, { value: "Module" }));
const AlizarinModel = ResourceInstanceViewModel;
export {
  AlizarinModel,
  GraphManager,
  RDM,
  ResourceModelWrapper,
  WKRM,
  client,
  graphManager,
  interfaces,
  nodeConfig,
  renderers,
  staticStore,
  staticTypes,
  utils,
  viewModels
};
//# sourceMappingURL=alizarin.js.map
