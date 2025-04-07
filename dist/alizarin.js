var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
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
  }
  async getCollection(collectionId) {
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
    const response = await fetch(
      `${this.archesUrl}/${this.resourceIdToFile(resourceId)}`
    );
    return await response.json();
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
      const response = await fetch(`${this.archesUrl}/${file}`);
      const resourceSet = (await response.json()).business_data.resources;
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
    this.fs = import("./__vite-browser-external-2Ng8QIWW.js").then((fs) => {
      return fs.promises;
    });
    this.allGraphFile = allGraphFile || (() => "public/models/_all.json");
    this.graphIdToGraphFile = graphIdToGraphFile || ((graphId) => `public/models/${graphId}.json`);
    this.graphIdToResourcesFiles = graphIdToResourcesFiles || ((graphId) => [`public/resources/_${graphId}.json`]);
    this.resourceIdToFile = resourceIdToFile || ((resourceId) => `public/resources/${resourceId}.json`);
    this.collectionIdToFile = collectionIdToFile || ((collectionId) => `public/collections/${collectionId}.json`);
  }
  async getGraphs() {
    const fs = await this.fs;
    const response = await fs.readFile(this.allGraphFile(), "utf8");
    return await JSON.parse(response);
  }
  async getGraph(graphId) {
    const fs = await this.fs;
    const response = await fs.readFile(
      this.graphIdToGraphFile(graphId),
      "utf8"
    );
    return await JSON.parse(response).graph[0];
  }
  async getResource(resourceId) {
    const fs = await this.fs;
    const response = await fs.readFile(
      this.resourceIdToFile(resourceId),
      "utf8"
    );
    return await JSON.parse(response);
  }
  async getCollection(collectionId) {
    const fs = await this.fs;
    const response = await fs.readFile(
      this.collectionIdToFile(collectionId),
      "utf8"
    );
    return await JSON.parse(response);
  }
  async getResources(graphId, limit) {
    const fs = await this.fs;
    const resources = [];
    for (const file of this.graphIdToResourcesFiles(graphId)) {
      const response = await fs.readFile(file, "utf8");
      const resourceSet = (await JSON.parse(response)).business_data.resources;
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
  archesClient
}, Symbol.toStringTag, { value: "Module" }));
const DEFAULT_LANGUAGE$1 = "en";
function getCurrentLanguage() {
  return (typeof navigator != "undefined" && navigator.language || DEFAULT_LANGUAGE$1).slice(0, 2);
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
      if (lang === void 0 || !(lang in translations)) {
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
    __publicField(this, "resource_2_resource_constringaints", null);
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
    this.resource_2_resource_constringaints = jsonData.resource_2_resource_constringaints;
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
    this.id = jsonData.id;
    this.value = jsonData.value;
    this.__concept = concept;
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
class StaticResource {
  constructor(jsonData) {
    __publicField(this, "resourceinstance");
    __publicField(this, "tiles", null);
    this.resourceinstance = new StaticResourceMetadata(
      jsonData.resourceinstance
    );
    this.tiles = jsonData.tiles && jsonData.tiles.map((tile) => new StaticTile(tile));
  }
}
const staticTypes = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  StaticCollection,
  StaticConcept,
  StaticEdge,
  StaticGraph,
  StaticNode,
  StaticNodegroup,
  StaticResource,
  StaticTile,
  StaticValue
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
class AttrPromise extends Promise {
  constructor(executor) {
    super(executor);
    return new Proxy(this, {
      set: (object, keyObj, value) => {
        const key = keyObj.toString();
        if (object instanceof Promise) {
          return object.then((val) => {
            val[key] = value;
            return val;
          });
        }
        object[key] = value;
        return this;
      },
      get: (object, keyObj) => {
        const key = keyObj.toString();
        if (key in object) {
          if (typeof object[key] === "function") {
            return object[key].bind(object);
          }
          return object[key];
        }
        if (object instanceof Promise) {
          return object.then((val) => {
            return val ? val[key] : val;
          });
        }
        return object[key];
      }
    });
  }
}
const utils = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  AttrPromise
}, Symbol.toStringTag, { value: "Module" }));
const DEFAULT_LANGUAGE = "en";
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
    this.values = values;
    this.wrapper = wrapper;
    this.tiles = tiles;
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
    let result = await this.values.get(key);
    if (result === false) {
      if (this.wrapper.resource) {
        const node = this.wrapper.model.getNodeObjectsByAlias().get(key);
        if (node === void 0) {
          throw Error(
            "Tried to retrieve a node key that does not exist on this resource"
          );
        }
        const values = new Map([...this.values.entries()]);
        const promise = new Promise((resolve) => {
          this.wrapper.ensureNodegroup(
            values,
            node,
            node.nodegroup_id,
            this.wrapper.model.getNodeObjects(),
            this.wrapper.model.getNodegroupObjects(),
            this.wrapper.model.getEdges(),
            false,
            this.tiles
          ).then((ngValues) => {
            for (const [key2, value] of [...ngValues.entries()]) {
              this.values.set(key2, value);
            }
            resolve(null);
          });
        });
        this.values.set(key, promise);
        await promise;
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
class ResourceInstanceViewModel {
  constructor(id, modelWrapper, instanceWrapperFactory) {
    __publicField(this, "_");
    __publicField(this, "__");
    __publicField(this, "id");
    __publicField(this, "then", null);
    this.id = id;
    this._ = instanceWrapperFactory(this);
    this.__ = modelWrapper;
    return new Proxy(this, {
      set: (object, key, value) => {
        const k = key.toString();
        if (k in object) {
          object[k] = value;
        } else {
          object._.setOrmAttribute(k, value);
        }
        return true;
      },
      get: (object, key) => {
        const k = key.toString();
        if (k in object) {
          return object[k];
        }
        return new AttrPromise((resolve) => {
          return object._.getOrmAttribute(k).then((v) => {
            return resolve(v);
          });
        });
      }
    });
  }
  toString() {
    return `[${this.__.wkrm.modelClassName}:${this.id ?? "-"}]`;
  }
  async forJson() {
    return {
      type: this.__.wkrm.modelClassName,
      id: this.id
    };
  }
}
class ConceptListViewModel extends Array {
  constructor() {
    super(...arguments);
    __publicField(this, "__parentPseudo");
    __publicField(this, "_value", null);
  }
  async forJson() {
    const value = await this._value;
    return value ? value.map((v) => v ? v.forJson() : null) : null;
  }
  static async __create(tile, node, value) {
    const nodeid = node.nodeid;
    let val;
    if (tile) {
      if (!(nodeid in tile.data)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, []);
        if (!Array.isArray(value)) {
          throw Error(
            "Cannot set an (entire) concept list value except via an array"
          );
        }
        val = value.map((c) => {
          if (c instanceof ConceptValueViewModel) {
            return c;
          }
          return ConceptValueViewModel.__create(tile, node, c, RDM);
        });
        this._value = Promise.all(val).then((vals) => {
          Promise.all(
            vals.map(async (c) => {
              const v = await c;
              return v ? (await v.getValue()).id : null;
            })
          ).then((ids) => {
            tile.data.set(nodeid, ids);
            return ids;
          });
        });
      }
    }
    if (!tile || !val) {
      return null;
    }
    const str = new ConceptListViewModel(...val);
    return str;
  }
  async __asTileData() {
    return this._value ? await this._value : null;
  }
}
class ConceptValueViewModel extends String {
  constructor(value) {
    super(value.value);
    __publicField(this, "__parentPseudo");
    __publicField(this, "_value");
    this._value = value;
  }
  async forJson() {
    return `${await this._value}`;
  }
  getValue() {
    return this._value;
  }
  static async __create(tile, node, value) {
    const nodeid = node.nodeid;
    let val = value;
    if (tile) {
      if (!(nodeid in tile.data)) {
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
            return ConceptValueViewModel.__create(tile, node, value2);
          });
        } else if (typeof value == "string") {
          if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-5][0-9a-f]{3}-[089ab][0-9a-f]{3}-[0-9a-f]{12}$/i.exec(
            value
          )) {
            const collectionId = node.config["rdmCollection"];
            const collection = RDM.retrieveCollection(collectionId);
            return collection.then((collection2) => {
              const val2 = collection2.getConceptValue(value);
              tile.data.set(nodeid, val2 ? val2.id : null);
              if (!tile || !val2) {
                return null;
              }
              const str2 = new ConceptValueViewModel(val2);
              return str2;
            });
          } else {
            throw Error(
              "Set concepts using values from collections, not strings"
            );
          }
        } else {
          throw Error("Could not set concept from this data");
        }
        if (!(val instanceof Promise)) {
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
  __asTileData() {
    return this._value ? this._value.id : null;
  }
}
class GeoJSONViewModel {
  constructor(jsonData) {
    __publicField(this, "__parentPseudo");
    __publicField(this, "_value");
    this._value = jsonData;
    return new Proxy(this, {
      get: (object, key) => {
        const k = key.toString();
        if (k in object) {
          return object[k];
        }
        return this._value[k];
      },
      set: (object, key, value) => {
        const k = key.toString();
        if (k in object) {
          object[k] = value;
        } else {
          this._value[k] = value;
        }
        return true;
      }
    });
  }
  static __create(tile, node, value) {
    const nodeid = node.nodeid;
    if (value instanceof Promise) {
      return value.then(
        (value2) => GeoJSONViewModel.__create(tile, node, value2)
      );
    }
    if (tile) {
      if (!(nodeid in tile.data)) {
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
    const str = new GeoJSONViewModel(val);
    return str;
  }
  async forJson() {
    return await this._value;
  }
  __asTileData() {
    return this._value;
  }
}
class StringViewModel extends String {
  constructor(value, language = null) {
    const displayValue = value.get(language || DEFAULT_LANGUAGE) || {
      value: ""
    };
    super(displayValue.value);
    __publicField(this, "__parentPseudo");
    __publicField(this, "_value");
    this._value = value;
  }
  forJson() {
    return `${this}`;
  }
  lang(language) {
    const elt = this._value.get(language);
    if (elt) {
      return elt.value;
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
      if (!(nodeid in tile.data)) {
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
class SemanticViewModel extends Map {
  constructor(parentWkri, childNodes, tile, node) {
    super();
    __publicField(this, "then");
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
        const k = key.toString();
        if (k.startsWith("__")) {
          object[k] = value;
        } else {
          object.__set(k, value);
        }
        return true;
      },
      get: (object, key) => {
        const k = key.toString();
        if (k.startsWith("__") || k in object) {
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
  async toString() {
    const entries = this.__childValues.entries().map(([k, v]) => `${k}: ${v}`);
    return `[[${entries.join(",")}]]`;
  }
  async forJson() {
    const values = new Object(
      await Promise.all([...(await this.__getChildren(true)).entries()])
    );
    return values;
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
    if (!this.__childValues.has(key)) {
      let child = await this.__getChildValues(key);
      if (child === null) {
        child = this.__makePseudo(key);
      }
      this.__childValues.set(key, child);
      child.parentNode = this;
    }
    this.__childValues.get(key).value = value;
  }
  async __getChildTypes() {
    const promises = [...this.__childNodes.keys()].map(async (key) => [
      key,
      await this.__getChildValue(key)
    ]);
    const entries = await Promise.all(promises);
    return new Map(...entries);
  }
  async __getChildren(direct = null) {
    const items = /* @__PURE__ */ new Map();
    for (const [key, value] of [...await this.__getChildValues()]) {
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
  async __getChildValue(key) {
    if (!this.__childNodes.has(key)) {
      throw Error(
        `Semantic node does not have this key: ${key} (${[...this.__childNodes.keys()]})`
      );
    }
    let child;
    if (!this.__childValues.has(key)) {
      child = await this.__getChildValues(key);
      if (child === null) {
        child = this.__makePseudo(key);
      } else {
        this.__childValues.set(key, child);
      }
      child.parentNode = this;
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
    const child = this.__parentWkri._.model.makePseudoCls(
      key,
      false,
      !childNode.is_collector ? this.__tile : null,
      // Does it share a tile
      this.__parentWkri
    );
    child.parentNode = this.__parentPseudo;
    if (this.__parentWkri) {
      const valueList = this.__parentWkri._.valueList;
      valueList.setDefault(key, []).then((val) => val.push(child));
    }
    return child;
  }
  static async __create(tile, node, value, parent, childNodes) {
    const svm = new SemanticViewModel(parent, childNodes, tile, node);
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
    await svm.__getChildren();
    return svm;
  }
  async __asTileData() {
    const relationships = [];
    for (const value of this.__getChildren(true)) {
      const [_, subrelationships] = await value.getTile();
      relationships.push(...subrelationships);
    }
    return [null, relationships];
  }
  async __getChildValues(targetKey = null) {
    const parent = this.__parentWkri;
    const childNodes = this.__childNodes;
    const tile = this.__tile;
    const node = this.__node;
    if (!parent) {
      return targetKey === null ? {} : null;
    }
    for (const key of childNodes.keys()) {
      await parent._.valueList.retrieve(key);
    }
    const children = /* @__PURE__ */ new Map();
    for (let [key, values] of [...parent._.valueList.values.entries()]) {
      if (values instanceof Promise) {
        values = await values;
      }
      if (values === false || values === null || values === void 0) {
        continue;
      }
      const childNode = childNodes.get(key);
      for (const value of values) {
        if (childNode && value !== null && (value.parentNode === null || value.parentNode === this.__parentPseudo)) {
          if (tile && value.parenttile_id == tile.tileid || value.node.nodegroup_id == node.nodeid && tile && value.tile == tile && !childNode.is_collector) {
            children.set(key, value);
          } else if (node.nodegroup_id != value.node.nodegroup_id && childNode.is_collector) {
            if (value instanceof PseudoList || value.value && Array.isArray(value.value)) {
              if (children.has(key)) {
                children.get(key).push(...value);
              } else {
                children.set(key, value);
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
    if (targetKey !== null) {
      return children.get(targetKey) || null;
    }
    return children;
  }
}
async function getViewModel(parentPseudo, tile, node, data, parent, childNodes) {
  let vm;
  switch (node.datatype) {
    case "semantic":
      vm = await SemanticViewModel.__create(
        tile,
        node,
        data,
        parent,
        childNodes
      );
      break;
    case "concept":
      vm = await ConceptValueViewModel.__create(tile, node, data);
      break;
    case "concept-list":
      vm = await ConceptListViewModel.__create(tile, node, data);
      break;
    case "geojson-feature-collection":
      vm = await GeoJSONViewModel.__create(tile, node, data);
      break;
    case "string":
    default:
      vm = await StringViewModel.__create(tile, node, data);
  }
  let asTileData = null;
  if (vm) {
    vm.__parentPseudo = parentPseudo;
    asTileData = vm.__asTileData.bind(vm);
  }
  return [vm, asTileData, "string", false];
}
const viewModels = /* @__PURE__ */ Object.freeze(/* @__PURE__ */ Object.defineProperty({
  __proto__: null,
  ResourceInstanceViewModel,
  ValueList,
  getViewModel
}, Symbol.toStringTag, { value: "Module" }));
class PseudoUnavailable {
  constructor() {
    __publicField(this, "parentNode", null);
  }
  async getValue() {
    console.warn("Tried to get value of unavailable node");
    return null;
  }
  getLength() {
    return 0;
  }
  getChildren(_ = false) {
    return [];
  }
}
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
    __publicField(this, "multiple", false);
    __publicField(this, "asTileData", null);
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
    if (this.asTileData && this.value !== null) {
      tileValue = await this.asTileData(this.value);
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
    if (this.tile && this.tile.data && this.node.nodeid in this.tile.data) {
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
        provisionaledits: null
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
        let value;
        [value, this.asTileData, this.datatype, this.multiple] = vm2;
        if (value !== null && this.value instanceof Object) {
          value.__parentPseudo = this;
        }
        if (value !== null) {
          this.valueLoaded = true;
        }
        return value;
      };
      this.value = new AttrPromise((resolve) => {
        vm.then((vm2) => resolve(resolveAttr(vm2)));
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
  async getType() {
    await this.updateValue();
    return [this.datatype, this.multiple];
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
  getValue() {
    return this;
  }
  toString() {
    return `<PL: ${this.length}>`;
  }
}
function makePseudoCls(key, single, tile = null, wkri = null) {
  const nodeObj = this.getNodeObjectsByAlias().get(key);
  if (!nodeObj) {
    throw Error("Could not find node by alias");
  }
  const nodegroups = this.getNodegroupObjects();
  const nodegroup = nodegroups.get(nodeObj.nodegroup_id || "");
  const permitted = this.getPermittedNodegroups();
  let value = null;
  if (nodeObj.nodegroup_id && nodeObj.is_collector && nodegroup && nodegroup.cardinality == "n" && !single) {
    value = new PseudoList();
    value.initialize(nodeObj, wkri);
  }
  if (value === null || tile) {
    let nodeValue;
    if (nodeObj.nodegroup_id && !permitted.get(nodeObj.nodegroup_id)) {
      nodeValue = new PseudoUnavailable();
    } else {
      const childNodes = this.getChildNodes(nodeObj.nodeid);
      nodeValue = new PseudoValue(nodeObj, tile, null, wkri, childNodes);
    }
    if (value) {
      value.push(nodeValue.getValue());
    } else {
      value = nodeValue;
    }
  }
  return value;
}
class WKRM {
  constructor(modelName, graphId) {
    __publicField(this, "modelName");
    __publicField(this, "modelClassName");
    __publicField(this, "graphId");
    this.modelName = modelName;
    this.graphId = graphId;
    this.modelClassName = modelName.replace(/\s(.)/g, (c) => c.toUpperCase()).replace(/\s/g, "");
  }
}
class ConfigurationOptions {
  constructor() {
    __publicField(this, "graphs");
    this.graphs = null;
  }
}
class StaticStore {
  constructor(archesClient2) {
    __publicField(this, "archesClient");
    this.archesClient = archesClient2;
  }
  async loadAll(graphId, limit = void 0) {
    const resourcesJSON = await this.archesClient.getResources(graphId, limit || 0);
    resourcesJSON.length;
    return [...resourcesJSON.entries()].map(
      ([num, resourceJSON]) => {
        return new StaticResource(resourceJSON);
      }
    );
  }
  async loadOne(id) {
    const resourceJSON = await this.archesClient.getResource(id);
    return new StaticResource(resourceJSON);
  }
}
class ResourceInstanceWrapper {
  constructor(wkri, model, resource) {
    __publicField(this, "wkri");
    __publicField(this, "model");
    __publicField(this, "resource");
    __publicField(this, "valueList");
    this.wkri = wkri;
    this.model = model;
    this.resource = resource;
    this.valueList = new ValueList(/* @__PURE__ */ new Map(), this);
  }
  async keys() {
    return (await this.getRoot()).keys();
  }
  async values() {
    return (await this.getRoot()).values();
  }
  async entries() {
    return (await this.getRoot()).entries();
  }
  async getOrmAttribute(key) {
    const root = await this.getRoot();
    if (root) {
      const value = root.getValue();
      return value[key];
    } else {
      throw Error(`Tried to get ${key} on ${this}, which has no root`);
    }
  }
  async getRoot() {
    const values = this.valueList;
    const node = this.model.getRootNode();
    if (node) {
      let value;
      const alias = node.alias;
      if (!alias) {
        throw Error(`Alias missing for node ${node.nodeid}`);
      }
      await values.setDefault(alias, []);
      const nodeValues = await values.get(alias);
      if (nodeValues.length > 1) {
        throw Error("Cannot have multiple root tiles");
      } else if (nodeValues.length == 1) {
        value = nodeValues[0];
      } else {
        value = this.model.makePseudoCls(alias, false, null, this.wkri);
        values.set(alias, [value]);
      }
      return value;
    }
  }
  setOrmAttribute(key, value) {
    const root = this.getRoot();
    if (root) {
      root.value[key] = value;
    } else {
      throw Error(`Tried to set ${key} on ${self}, which has no root`);
    }
  }
  async ensureNodegroup(allValues, node, nodegroupId, nodeObjs, nodegroupObjs, edges, addIfMissing, tiles) {
    if (!node) {
      return allValues;
    }
    const alias = node.alias || "";
    const impliedNodegroups = /* @__PURE__ */ new Set();
    const value = node && await allValues.get(alias);
    let newAllValues = allValues;
    if (value === false || addIfMissing && value === void 0) {
      if (alias in newAllValues) {
        newAllValues.delete(alias);
      }
      let nodegroupTiles;
      if (tiles === null) {
        nodegroupTiles = [];
        console.error("Tiles must be provided and cannot be lazy-loaded yet");
      } else {
        nodegroupTiles = tiles.filter(
          (tile) => tile.nodegroup_id == nodegroupId
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
          newAllValues.set(entry[0], entry[1]);
        });
        [...newImpliedNodegroups.keys()].forEach((k) => {
          impliedNodegroups.add(k);
        });
      }
    }
    for (const nodegroupId2 of impliedNodegroups) {
      newAllValues = await this.ensureNodegroup(
        newAllValues,
        node,
        nodegroupId2,
        nodeObjs,
        nodegroupObjs,
        edges,
        true,
        tiles
        // RMV different from Python
      );
    }
    return newAllValues;
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
    if (!lazy && this.resource) {
      const tiles = this.resource.tiles;
      for (const [ng, _] of nodegroupObjs) {
        for (const [key, value] of await this.ensureNodegroup(
          allValues,
          nodeObjs.get(ng) || rootNode,
          // should be the only missing ID
          ng,
          nodeObjs,
          nodegroupObjs,
          edges,
          true,
          // RMV: check vs python
          tiles
        )) {
          allValues.set(key, value);
        }
      }
    }
    this.valueList = new ValueList(
      allValues,
      this,
      this.resource ? this.resource.tiles : null
    );
  }
  async valuesFromResourceNodegroup(existingValues, nodegroupTiles, nodegroupId, nodeObjs, edges) {
    const allValues = /* @__PURE__ */ new Map();
    const impliedNodegroups = /* @__PURE__ */ new Set();
    const nodesUnseen = new Set(
      [...nodeObjs.values()].filter((node) => node.nodegroup_id == nodegroupId).map((node) => node.alias)
    );
    const _addNode = async (node, tile) => {
      const key = node.alias || "";
      nodesUnseen.delete(node.alias);
      let existing = existingValues.get(key);
      if (existing instanceof Promise) {
        existing = await existing;
      }
      if (existing !== false && existing !== void 0) {
        throw Error(`Tried to load node twice: ${key} ${existing}`);
      }
      if (!(key in allValues)) {
        allValues.set(key, []);
      }
      const pseudoNode = this.model.makePseudoCls(key, false, tile, this.wkri);
      for (const [domain, ranges] of edges) {
        if (node.nodegroup_id && ranges.includes(node.nodegroup_id)) {
          const domainNode = nodeObjs.get(domain);
          if (!domainNode) {
            throw Error("Edge error in graph");
          }
          const toAdd = domainNode.nodegroup_id ? domainNode.nodegroup_id : domainNode.nodeid;
          impliedNodegroups.add(toAdd);
          break;
        }
      }
      if (Array.isArray(pseudoNode)) {
        const value = allValues.get(key);
        if (value !== void 0 && value !== false) {
          for (const pseudoNodeList of allValues.get(key)) {
            if (!Array.isArray(pseudoNodeList)) {
              throw Error(`Should be all lists not ${typeof pseudoNodeList}`);
            }
            if (pseudoNodeList.parentNode == pseudoNode.parentNode) {
              for (const ps of pseudoNode) {
                pseudoNodeList.push(ps);
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
      await _addNode(parentNode, tile);
      if (tile) {
        const tileNodes = /* @__PURE__ */ new Map();
        for (const [key, value] of [...tile.data.entries()]) {
          tileNodes.set(key, value);
        }
        if (!(tile.nodegroup_id in tileNodes)) {
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
            await _addNode(node, tile);
          }
        }
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
  constructor(wkrm, graph) {
    __publicField(this, "wkrm");
    __publicField(this, "graph");
    __publicField(this, "viewModelClass", ResourceInstanceViewModel);
    __publicField(this, "makePseudoCls");
    __publicField(this, "edges");
    __publicField(this, "nodes");
    __publicField(this, "nodegroups");
    __publicField(this, "nodesByAlias");
    this.wkrm = wkrm;
    this.graph = graph;
    this.makePseudoCls = makePseudoCls.bind(this);
  }
  async all(params) {
    return Promise.all(
      (await staticStore.loadAll(this.wkrm.graphId, params.limit)).map(
        (resource) => {
          return this.fromStaticResource(resource, params.lazy);
        }
      )
    );
  }
  async find(id, lazy = true) {
    const rivm = await staticStore.loadOne(id);
    return this.fromStaticResource(rivm, lazy);
  }
  getPermittedNodegroups() {
    return this.getNodegroupObjects();
  }
  makeInstance(id, resource) {
    const instance = new this.viewModelClass(
      id,
      this.viewModelClass.prototype.__,
      (rivm) => new ResourceInstanceWrapper(rivm, this, resource)
    );
    return instance;
  }
  getChildNodes(nodeId) {
    const childNodes = /* @__PURE__ */ new Map();
    const edges = this.getEdges().get(nodeId);
    if (edges) {
      for (const [_, n] of this.getNodeObjects()) {
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
    const graph = graphManager.getGraph(this.wkrm.graphId);
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
    return rootNode;
  }
  fromStaticResource(resource, lazy = false) {
    const wkri = this.makeInstance(
      resource.resourceinstance.resourceinstanceid,
      resource
    );
    return wkri._.populate(lazy).then(() => wkri);
  }
}
function makeResourceModelWrapper(wkrm, graph) {
  var _a;
  const viewModelClass = {
    [wkrm.modelClassName]: (_a = class extends ResourceInstanceViewModel {
      static async all(params = {
        limit: null,
        lazy: false
      }) {
        return viewModelClass.prototype.__.all({
          limit: params.limit,
          lazy: params.lazy
        });
      }
      static async find(id, lazy = false) {
        return viewModelClass.prototype.__.find(id, lazy);
      }
      constructor(id, wrapper2, instanceWrapperFactory) {
        super(id, wrapper2, instanceWrapperFactory);
      }
    }, __publicField(_a, "_"), __publicField(_a, "__"), _a)
  }[wkrm.modelClassName];
  const wrapper = new ResourceModelWrapper(wkrm, graph);
  wrapper.viewModelClass = viewModelClass;
  viewModelClass.prototype.__ = wrapper;
  return wrapper;
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
  async initialize(configurationOptions) {
    if (this._initialized) {
      return;
    }
    if (configurationOptions === void 0) {
      configurationOptions = new ConfigurationOptions();
    }
    const graphJsons = await this.archesClient.getGraphs();
    let graphs = Object.keys(graphJsons["models"]);
    if (configurationOptions.graphs !== null) {
      graphs = graphs.filter(
        (graphId) => graphId in configurationOptions
      );
    }
    await Promise.all(
      graphs.map(async (graphId) => {
        const bodyJson = await this.archesClient.getGraph(graphId);
        const graph = new StaticGraph(bodyJson);
        const wkrm = new WKRM(graph.name.toString(), graph.graphid);
        this.wkrms.set(wkrm.modelClassName, wkrm);
        this.graphs.set(graph.graphid, makeResourceModelWrapper(wkrm, graph));
      })
    );
    this._initialized = true;
  }
  get(modelClassName) {
    this.initialize(void 0);
    const wkrm = this.wkrms.get(modelClassName);
    if (wkrm === void 0) {
      throw Error(`Cannot find model requested: ${modelClassName}`);
    }
    const wrapper = this.graphs.get(wkrm.graphId);
    if (wrapper === void 0) {
      throw Error(`Cannot find graph requested: ${modelClassName}`);
    }
    return wrapper.viewModelClass;
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
const staticStore = new StaticStore(archesClient);
export {
  RDM,
  client,
  graphManager,
  staticStore,
  staticTypes,
  utils,
  viewModels
};
