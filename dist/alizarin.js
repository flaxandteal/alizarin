var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
let AlizarinModel, GraphManager, GraphMutator, RDM, ResourceModelWrapper, WKRM, client, getCurrentLanguage, graphManager, interfaces, nodeConfig, renderers, run, setCurrentLanguage, staticStore, staticTypes, utils, viewModels;
let __tla = (async () => {
  var _a, _b, _c, _d, _e;
  const REGEX = /^(?:[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}|00000000-0000-0000-0000-000000000000|ffffffff-ffff-ffff-ffff-ffffffffffff)$/i;
  function validate(uuid) {
    return typeof uuid === "string" && REGEX.test(uuid);
  }
  function parse(uuid) {
    if (!validate(uuid)) {
      throw TypeError("Invalid UUID");
    }
    let v;
    return Uint8Array.of((v = parseInt(uuid.slice(0, 8), 16)) >>> 24, v >>> 16 & 255, v >>> 8 & 255, v & 255, (v = parseInt(uuid.slice(9, 13), 16)) >>> 8, v & 255, (v = parseInt(uuid.slice(14, 18), 16)) >>> 8, v & 255, (v = parseInt(uuid.slice(19, 23), 16)) >>> 8, v & 255, (v = parseInt(uuid.slice(24, 36), 16)) / 1099511627776 & 255, v / 4294967296 & 255, v >>> 24 & 255, v >>> 16 & 255, v >>> 8 & 255, v & 255);
  }
  const byteToHex = [];
  for (let i = 0; i < 256; ++i) {
    byteToHex.push((i + 256).toString(16).slice(1));
  }
  function unsafeStringify(arr, offset = 0) {
    return (byteToHex[arr[offset + 0]] + byteToHex[arr[offset + 1]] + byteToHex[arr[offset + 2]] + byteToHex[arr[offset + 3]] + "-" + byteToHex[arr[offset + 4]] + byteToHex[arr[offset + 5]] + "-" + byteToHex[arr[offset + 6]] + byteToHex[arr[offset + 7]] + "-" + byteToHex[arr[offset + 8]] + byteToHex[arr[offset + 9]] + "-" + byteToHex[arr[offset + 10]] + byteToHex[arr[offset + 11]] + byteToHex[arr[offset + 12]] + byteToHex[arr[offset + 13]] + byteToHex[arr[offset + 14]] + byteToHex[arr[offset + 15]]).toLowerCase();
  }
  let getRandomValues;
  const rnds8 = new Uint8Array(16);
  function rng() {
    if (!getRandomValues) {
      if (typeof crypto === "undefined" || !crypto.getRandomValues) {
        throw new Error("crypto.getRandomValues() not supported. See https://github.com/uuidjs/uuid#getrandomvalues-not-supported");
      }
      getRandomValues = crypto.getRandomValues.bind(crypto);
    }
    return getRandomValues(rnds8);
  }
  function stringToBytes(str) {
    str = unescape(encodeURIComponent(str));
    const bytes = new Uint8Array(str.length);
    for (let i = 0; i < str.length; ++i) {
      bytes[i] = str.charCodeAt(i);
    }
    return bytes;
  }
  const DNS = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
  const URL$1 = "6ba7b811-9dad-11d1-80b4-00c04fd430c8";
  function v35(version, hash, value, namespace, buf, offset) {
    const valueBytes = typeof value === "string" ? stringToBytes(value) : value;
    const namespaceBytes = typeof namespace === "string" ? parse(namespace) : namespace;
    if (typeof namespace === "string") {
      namespace = parse(namespace);
    }
    if ((namespace == null ? void 0 : namespace.length) !== 16) {
      throw TypeError("Namespace must be array-like (16 iterable integer values, 0-255)");
    }
    let bytes = new Uint8Array(16 + valueBytes.length);
    bytes.set(namespaceBytes);
    bytes.set(valueBytes, namespaceBytes.length);
    bytes = hash(bytes);
    bytes[6] = bytes[6] & 15 | version;
    bytes[8] = bytes[8] & 63 | 128;
    return unsafeStringify(bytes);
  }
  const randomUUID = typeof crypto !== "undefined" && crypto.randomUUID && crypto.randomUUID.bind(crypto);
  const native = {
    randomUUID
  };
  function v4(options, buf, offset) {
    var _a2;
    if (native.randomUUID && true && !options) {
      return native.randomUUID();
    }
    options = options || {};
    const rnds = options.random ?? ((_a2 = options.rng) == null ? void 0 : _a2.call(options)) ?? rng();
    if (rnds.length < 16) {
      throw new Error("Random bytes length must be >= 16");
    }
    rnds[6] = rnds[6] & 15 | 64;
    rnds[8] = rnds[8] & 63 | 128;
    return unsafeStringify(rnds);
  }
  function f(s, x, y, z) {
    switch (s) {
      case 0:
        return x & y ^ ~x & z;
      case 1:
        return x ^ y ^ z;
      case 2:
        return x & y ^ x & z ^ y & z;
      case 3:
        return x ^ y ^ z;
    }
  }
  function ROTL(x, n) {
    return x << n | x >>> 32 - n;
  }
  function sha1(bytes) {
    const K = [
      1518500249,
      1859775393,
      2400959708,
      3395469782
    ];
    const H = [
      1732584193,
      4023233417,
      2562383102,
      271733878,
      3285377520
    ];
    const newBytes = new Uint8Array(bytes.length + 1);
    newBytes.set(bytes);
    newBytes[bytes.length] = 128;
    bytes = newBytes;
    const l = bytes.length / 4 + 2;
    const N = Math.ceil(l / 16);
    const M = new Array(N);
    for (let i = 0; i < N; ++i) {
      const arr = new Uint32Array(16);
      for (let j = 0; j < 16; ++j) {
        arr[j] = bytes[i * 64 + j * 4] << 24 | bytes[i * 64 + j * 4 + 1] << 16 | bytes[i * 64 + j * 4 + 2] << 8 | bytes[i * 64 + j * 4 + 3];
      }
      M[i] = arr;
    }
    M[N - 1][14] = (bytes.length - 1) * 8 / Math.pow(2, 32);
    M[N - 1][14] = Math.floor(M[N - 1][14]);
    M[N - 1][15] = (bytes.length - 1) * 8 & 4294967295;
    for (let i = 0; i < N; ++i) {
      const W = new Uint32Array(80);
      for (let t = 0; t < 16; ++t) {
        W[t] = M[i][t];
      }
      for (let t = 16; t < 80; ++t) {
        W[t] = ROTL(W[t - 3] ^ W[t - 8] ^ W[t - 14] ^ W[t - 16], 1);
      }
      let a = H[0];
      let b = H[1];
      let c = H[2];
      let d = H[3];
      let e = H[4];
      for (let t = 0; t < 80; ++t) {
        const s = Math.floor(t / 20);
        const T = ROTL(a, 5) + f(s, b, c, d) + e + K[s] + W[t] >>> 0;
        e = d;
        d = c;
        c = ROTL(b, 30) >>> 0;
        b = a;
        a = T;
      }
      H[0] = H[0] + a >>> 0;
      H[1] = H[1] + b >>> 0;
      H[2] = H[2] + c >>> 0;
      H[3] = H[3] + d >>> 0;
      H[4] = H[4] + e >>> 0;
    }
    return Uint8Array.of(H[0] >> 24, H[0] >> 16, H[0] >> 8, H[0], H[1] >> 24, H[1] >> 16, H[1] >> 8, H[1], H[2] >> 24, H[2] >> 16, H[2] >> 8, H[2], H[3] >> 24, H[3] >> 16, H[3] >> 8, H[3], H[4] >> 24, H[4] >> 16, H[4] >> 8, H[4]);
  }
  function v5(value, namespace, buf, offset) {
    return v35(80, sha1, value, namespace);
  }
  v5.DNS = DNS;
  v5.URL = URL$1;
  const DEFAULT_LANGUAGE$1 = "en";
  const SLUG_LENGTH = 20;
  const UUID_NAMESPACE = "1a79f1c8-9505-4bea-a18e-28a053f725ca";
  const UUID_NAMESPACE_COMPRESSION = v5("compression", "1a79f1c8-9505-4bea-a18e-28a053f725ca");
  let currentLanguage;
  function slugify(original) {
    return `${original}`.replaceAll(/[^A-Za-z0-9_]/g, "").slice(0, SLUG_LENGTH);
  }
  function getCurrentLanguage$1() {
    return currentLanguage || (typeof navigator != "undefined" && navigator.language || DEFAULT_LANGUAGE$1).slice(0, 2);
  }
  function setCurrentLanguage$1(lang) {
    currentLanguage = lang;
  }
  class AttrPromise extends (_b = Promise, _a = Symbol.toPrimitive, _b) {
    constructor(executor) {
      super(executor);
      __publicField(this, _a);
      const proxy = new Proxy(this, {
        set: (object, keyObj, value) => {
          object.then((val) => {
            val[keyObj] = value;
            return val;
          });
          return true;
        },
        get: (object, keyObj) => {
          if (keyObj in object) {
            const value = object[keyObj];
            if (typeof value === "function") {
              return value.bind(object);
            }
            return value;
          }
          const key = keyObj.toString();
          if (key in object) {
            const value = object[key];
            if (typeof value === "function") {
              return value.bind(object);
            }
            return value;
          }
          if (object instanceof Promise) {
            return object.then((val) => {
              return val ? val[keyObj] : val;
            });
          }
          return object[keyObj];
        }
      });
      return proxy;
    }
  }
  const KEY_COMPRESSION_LENGTH = 1e3;
  function generateUuidv5(group, key) {
    if (Array.isArray(key)) {
      let shortKey = "";
      let keyTracker = "";
      key.forEach((k) => {
        if (keyTracker.length + k.length + 1 > KEY_COMPRESSION_LENGTH) {
          shortKey = v5(shortKey + ">" + keyTracker, UUID_NAMESPACE_COMPRESSION);
          keyTracker = k;
        } else {
          keyTracker += ";" + k;
        }
      });
    }
    return v5(`${group[0]}:${group[1]}:${key}`, UUID_NAMESPACE);
  }
  utils = Object.freeze(Object.defineProperty({
    __proto__: null,
    AttrPromise,
    generateUuidv5,
    getCurrentLanguage: getCurrentLanguage$1,
    setCurrentLanguage: setCurrentLanguage$1,
    slugify
  }, Symbol.toStringTag, {
    value: "Module"
  }));
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
      __publicField(this, "ontologyclass", null);
      __publicField(this, "parentproperty", null);
      __publicField(this, "sortorder");
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
    copy() {
      return new StaticNode(this);
    }
    static compare(nodeA, nodeB) {
      if (nodeA === nodeB) {
        return true;
      }
      const keys = [
        ...Object.keys(nodeA),
        ...Object.keys(nodeB)
      ].filter((key) => ![
        "compare",
        "copy",
        "nodeid",
        "graph_id",
        "nodegroup_id"
      ].includes(key));
      function compareEntries(entriesA, entriesB) {
        const entryPairs = {};
        for (const [key, value] of [
          ...entriesA,
          ...entriesB
        ]) {
          entryPairs[key] = entryPairs[key] || [];
          entryPairs[key].push(value);
        }
        for (const [_, [valA, valB]] of Object.entries(entryPairs)) {
          if (valA && valB && typeof valA === "object" && typeof valB === "object") {
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
      if (!compareEntries(keys.map((k) => [
        k,
        nodeA[k]
      ]), keys.map((k) => [
        k,
        nodeB[k]
      ]))) {
        return false;
      }
      if (nodeA.graph_id && nodeB.graph_id && nodeA.graph_id !== nodeB.graph_id) {
        return -3;
      }
      if (nodeA.nodegroup_id && nodeB.nodegroup_id && nodeA.nodegroup_id !== nodeB.nodegroup_id) {
        return -2;
      }
      if (nodeA.nodeid && nodeB.nodeid && nodeA.nodeid !== nodeB.nodeid) {
        return -1;
      }
      if ((nodeA.graph_id && nodeB.graph_id || nodeA.graph_id === nodeB.graph_id) && (nodeA.nodegroup_id && nodeB.nodegroup_id || nodeA.nodegroup_id === nodeB.nodegroup_id) && (nodeA.nodeid && nodeB.nodeid || nodeA.nodeid === nodeB.nodeid)) {
        return 2;
      }
      return 1;
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
          const defaultLanguage = getCurrentLanguage$1();
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
        finalLang = lang || getCurrentLanguage$1();
        translations.set(finalLang, s);
      }
      s = translations.get(finalLang) || "";
      super(s);
      __publicField(this, "translations");
      __publicField(this, "lang");
      this.translations = translations;
      this.lang = finalLang;
    }
    copy() {
      return new StaticTranslatableString(this, this.lang);
    }
    toString() {
      const current = this.lang || getCurrentLanguage$1();
      let asString;
      if (this.translations.size) {
        asString = this.translations.get(current) || this.translations.values().next().value;
      }
      return `${asString}`;
    }
    toJSON() {
      return Object.fromEntries(this.translations);
    }
  }
  class StaticNodegroup {
    constructor(jsonData) {
      __publicField(this, "cardinality");
      __publicField(this, "legacygroupid");
      __publicField(this, "nodegroupid");
      __publicField(this, "parentnodegroup_id");
      this.legacygroupid = jsonData.legacygroupid;
      this.nodegroupid = jsonData.nodegroupid;
      this.parentnodegroup_id = jsonData.parentnodegroup_id;
      this.cardinality = jsonData.cardinality;
    }
    copy() {
      return new StaticNodegroup(this);
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
      this.constraints = jsonData.constraints.map((constraint) => new StaticConstraint(constraint));
      this.cssclass = jsonData.cssclass;
      this.description = jsonData.description;
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
      __publicField(this, "ontologyproperty", null);
      __publicField(this, "rangenode_id");
      this.description = jsonData.description;
      this.domainnode_id = jsonData.domainnode_id;
      this.edgeid = jsonData.edgeid;
      this.graph_id = jsonData.graph_id;
      this.name = jsonData.name;
      this.rangenode_id = jsonData.rangenode_id;
      this.ontologyproperty = jsonData.ontologyproperty;
    }
    copy() {
      return new StaticEdge(this);
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
    copy() {
      return new StaticFunctionsXGraphs(this);
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
    copy() {
      return new StaticPublication(this);
    }
  }
  class StaticGraph {
    constructor(jsonData) {
      __publicField(this, "author");
      __publicField(this, "cards", null);
      __publicField(this, "cards_x_nodes_x_widgets", null);
      __publicField(this, "color");
      __publicField(this, "config");
      __publicField(this, "deploymentdate", null);
      __publicField(this, "deploymentfile", null);
      __publicField(this, "description");
      __publicField(this, "edges");
      __publicField(this, "functions_x_graphs", null);
      __publicField(this, "graphid");
      __publicField(this, "iconclass");
      __publicField(this, "is_editable", null);
      __publicField(this, "isresource");
      __publicField(this, "jsonldcontext", null);
      __publicField(this, "name");
      __publicField(this, "nodegroups");
      __publicField(this, "nodes");
      __publicField(this, "ontology_id", null);
      __publicField(this, "publication", null);
      __publicField(this, "relatable_resource_model_ids");
      __publicField(this, "resource_2_resource_constraints", null);
      __publicField(this, "root");
      __publicField(this, "slug", null);
      __publicField(this, "subtitle");
      __publicField(this, "template_id");
      __publicField(this, "version");
      this.author = jsonData.author;
      this.cards = jsonData.cards && jsonData.cards.map((card) => new StaticCard(card));
      this.cards_x_nodes_x_widgets = jsonData.cards_x_nodes_x_widgets && jsonData.cards_x_nodes_x_widgets.map((card_x_node_x_widget) => new StaticCardsXNodesXWidgets(card_x_node_x_widget));
      this.color = jsonData.color;
      this.config = jsonData.config;
      this.deploymentdate = jsonData.deploymentdate;
      this.deploymentfile = jsonData.deploymentfile;
      this.description = new StaticTranslatableString(jsonData.description);
      this.edges = jsonData.edges.map((edge) => new StaticEdge(edge));
      this.functions_x_graphs = jsonData.functions_x_graphs && jsonData.functions_x_graphs.map((functions_x_graphs) => new StaticFunctionsXGraphs(functions_x_graphs));
      this.graphid = jsonData.graphid;
      this.iconclass = jsonData.iconclass;
      this.is_editable = jsonData.is_editable;
      this.isresource = jsonData.isresource;
      this.jsonldcontext = jsonData.jsonldcontext;
      this.name = new StaticTranslatableString(jsonData.name);
      this.nodegroups = jsonData.nodegroups.map((nodegroup) => new StaticNodegroup(nodegroup));
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
    copy() {
      var _a2, _b2, _c2, _d2, _e2, _f;
      const newGraph = new StaticGraph(this);
      Object.assign(newGraph, {
        author: this.author,
        cards: ((_a2 = this.cards) == null ? void 0 : _a2.map((card) => new StaticCard(card))) || [],
        cards_x_nodes_x_widgets: ((_b2 = this.cards_x_nodes_x_widgets) == null ? void 0 : _b2.map((cnw) => new StaticCardsXNodesXWidgets(cnw))) || [],
        color: this.color,
        config: Object.assign({}, this.config),
        deploymentdate: this.deploymentdate,
        deploymentfile: this.deploymentfile,
        description: this.description.copy && this.description.copy() || this.description,
        edges: this.edges.map((edge) => edge.copy && edge.copy() || edge),
        functions_x_graphs: ((_c2 = this.functions_x_graphs) == null ? void 0 : _c2.map((fxg) => fxg.copy())) || [],
        graphid: this.graphid,
        iconclass: this.iconclass,
        is_editable: this.is_editable,
        isresource: this.isresource,
        jsonldcontext: this.jsonldcontext,
        name: this.name.copy && this.name.copy() || this.name,
        nodegroups: (_d2 = this.nodegroups) == null ? void 0 : _d2.map((ng) => ng.copy && ng.copy() || ng),
        nodes: (_e2 = this.nodes) == null ? void 0 : _e2.map((n) => n.copy && n.copy() || n),
        ontology_id: this.ontology_id,
        publication: ((_f = this.publication) == null ? void 0 : _f.copy) && this.publication.copy() || null,
        relatable_resource_model_ids: [
          ...this.relatable_resource_model_ids || []
        ],
        resource_2_resource_constraints: [
          ...this.resource_2_resource_constraints || []
        ],
        root: this.root.copy && this.root.copy() || this.root,
        slug: this.slug,
        subtitle: this.subtitle.copy && this.subtitle.copy(),
        template_id: this.template_id,
        version: this.version
      });
      return newGraph;
    }
    static create(props, published = true) {
      const graphid = props.graphid || v4();
      const publication = published ? new StaticPublication({
        graph_id: graphid,
        notes: null,
        publicationid: v4(),
        published_time: (/* @__PURE__ */ new Date()).toISOString()
      }) : null;
      const name = props.name ? props.name instanceof StaticTranslatableString ? props.name : new StaticTranslatableString(props.name) : new StaticTranslatableString("");
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
        description: props.description ? props.description instanceof StaticTranslatableString ? props.description : new StaticTranslatableString(props.description) : null,
        edges: [],
        functions_x_graphs: [],
        graphid,
        iconclass: props.iconclass || "",
        is_editable: props.is_editable || null,
        isresource: props.isresource || null,
        jsonldcontext: props.jsonldcontext || null,
        name,
        nodegroups: [],
        nodes: [
          root.copy()
        ],
        ontology_id: props.ontology_id || null,
        publication,
        relatable_resource_model_ids: props.relatable_resource_model_ids || [],
        resource_2_resource_constraints: props.resource_2_resource_constraints || null,
        root,
        slug: props.slug || null,
        subtitle: props.subtitle ? props.subtitle instanceof StaticTranslatableString ? props.subtitle : new StaticTranslatableString(props.subtitle) : new StaticTranslatableString(""),
        template_id: props.template_id || "",
        version: props.version || ""
      });
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
    static create(referent, valueType, value, language) {
      const lang = language || getCurrentLanguage$1();
      const referentId = referent instanceof StaticConcept ? referent.id : referent;
      const concept = referent instanceof StaticConcept ? referent : null;
      const id = generateUuidv5([
        "value"
      ], `${referentId}/${valueType}/${value}/${lang}`);
      return new StaticValue({
        id,
        value
      }, concept);
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
      return this.prefLabels[getCurrentLanguage$1()] || Object.values(this.prefLabels)[0];
    }
    toString() {
      if (!this.getPrefLabel) {
        return this.constructor(this).getPrefLabel().value;
      }
      return this.getPrefLabel().value;
    }
    static fromValue(conceptScheme, value, children, config = {}) {
      let lang = (config == null ? void 0 : config.baseLanguage) || getCurrentLanguage$1();
      let tmpValue;
      let prefLabels;
      if (typeof value === "string") {
        tmpValue = value;
        prefLabels = {
          [lang]: new StaticValue({
            id: "",
            value: tmpValue
          })
        };
      } else if (value instanceof StaticValue) {
        tmpValue = value.value;
        prefLabels = {
          [lang]: value
        };
      } else if (lang in value) {
        tmpValue = value[lang].value;
        prefLabels = value;
      } else {
        const firstValue = Object.entries(value).sort()[0];
        if (firstValue === void 0) {
          throw Error("Cannot create a concept from values without a non-empty value");
        }
        lang = firstValue[0];
        tmpValue = firstValue[1].value;
        prefLabels = value;
      }
      const conceptId = generateUuidv5([
        "concept"
      ], `${(conceptScheme == null ? void 0 : conceptScheme.id) || "(none)"}/${tmpValue}`);
      const childConcepts = (children || []).map((child) => {
        if (!(child instanceof StaticConcept)) {
          return StaticConcept.fromValue(conceptScheme, value, [], {
            baseLanguage: config.baseLanguage
          });
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
    static fromConceptScheme(props) {
      const collectionName = props.name ?? props.conceptScheme.toString();
      return StaticCollection.create({
        name: collectionName,
        concepts: props.conceptScheme.children || []
      });
    }
    static create(props) {
      let concepts = props.concepts;
      if (Array.isArray(concepts)) {
        concepts = concepts.reduce((acc, c) => {
          acc[c.id] = c;
          return acc;
        }, {});
      }
      const name = typeof props.name === "string" ? StaticValue.create("", "prefLabel", props.name) : props.name;
      let collectionid = props.collectionid;
      if (!collectionid) {
        if (typeof name === "string") {
          collectionid = generateUuidv5([
            "collection"
          ], name);
        } else if (name instanceof StaticValue) {
          collectionid = generateUuidv5([
            "collection"
          ], name.value);
        } else {
          throw Error("Must have a unique name to create a collection ID");
        }
      }
      const prefLabels = name instanceof StaticValue ? {
        [getCurrentLanguage$1()]: name
      } : name;
      return new StaticCollection({
        id: collectionid,
        prefLabels,
        concepts,
        __allConcepts: {},
        __values: {}
      });
    }
    getConceptValue(valueId) {
      return this.__values[valueId];
    }
    getConceptByValue(label) {
      var _a2;
      return (_a2 = Object.values(this.__values).find((value) => value.value == label)) == null ? void 0 : _a2.__concept;
    }
    toString() {
      return (this.prefLabels[getCurrentLanguage$1()] || Object.values(this.prefLabels)[0] || "").toString();
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
      if (typeof this.data === "object" && !(this.data instanceof Map)) {
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
  class StaticResourceDescriptors {
    constructor(jsonData) {
      __publicField(this, "name");
      __publicField(this, "map_popup");
      __publicField(this, "description");
      if (jsonData) {
        this.name = jsonData.name;
        this.map_popup = jsonData.map_popup;
        this.description = jsonData.description;
      }
    }
    isEmpty() {
      return !(this.name || this.map_popup || this.description);
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
    constructor(jsonData) {
      __publicField(this, "id");
      __publicField(this, "selected");
      __publicField(this, "text");
      this.id = jsonData.id;
      this.selected = jsonData.selected;
      this.text = jsonData.text;
    }
    toString() {
      const lang = getCurrentLanguage$1();
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
      this.resourceinstance = new StaticResourceMetadata(jsonData.resourceinstance);
      this.tiles = jsonData.tiles && jsonData.tiles.map((tile) => new StaticTile(tile));
      this.metadata = jsonData.metadata || {};
      this.__cache = jsonData.__cache;
      this.__scopes = jsonData.__scopes;
    }
  }
  staticTypes = Object.freeze(Object.defineProperty({
    __proto__: null,
    StaticCard,
    StaticCardsXNodesXWidgets,
    StaticCollection,
    StaticConcept,
    StaticConstraint,
    StaticDomainValue,
    StaticEdge,
    StaticFunctionsXGraphs,
    StaticGraph,
    StaticGraphMeta,
    StaticNode,
    StaticNodegroup,
    StaticResource,
    StaticResourceDescriptors,
    StaticResourceMetadata,
    StaticResourceReference,
    StaticTile,
    StaticTranslatableString,
    StaticValue
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  class GraphResult {
    constructor(jsonData) {
      __publicField(this, "models");
      this.models = Object.fromEntries(Object.entries(jsonData.models).map(([k, v]) => [
        k,
        new StaticGraphMeta(v)
      ]));
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
      const response = await fetch(`${this.archesUrl}/api/arches/graphs?format=arches-json&hide_empty_nodes=false&compact=false`);
      return await response.json();
    }
    async getGraph(graph) {
      return this.getGraphByIdOnly(graph.graphid);
    }
    async getGraphByIdOnly(graphId) {
      const response = await fetch(`${this.archesUrl}/graphs/${graphId}?format=arches-json&gen=`);
      return await response.json();
    }
    async getResource(resourceId) {
      throw Error(`Not implemented yet: getResource(${resourceId}`);
    }
    async getCollection(collectionId) {
      throw Error(`Not implemented yet: getCollection(${collectionId}`);
    }
    async getResources(graphId, limit) {
      const response = await fetch(`${this.archesUrl}/resources?graph_uuid=${graphId}&format=arches-json&hide_empty_nodes=false&compact=false&limit=${limit}`);
      return await response.json();
    }
  }
  class ArchesClientRemoteStatic extends ArchesClient {
    constructor(archesUrl, { allGraphFile, graphToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, graphIdToGraphFile } = {}) {
      super();
      __publicField(this, "archesUrl");
      __publicField(this, "allGraphFile");
      __publicField(this, "graphToGraphFile");
      __publicField(this, "graphIdToGraphFile");
      __publicField(this, "graphIdToResourcesFiles");
      __publicField(this, "resourceIdToFile");
      __publicField(this, "collectionIdToFile");
      this.archesUrl = archesUrl;
      this.allGraphFile = allGraphFile || (() => "resource_models/_all.json");
      this.graphToGraphFile = graphToGraphFile;
      this.graphIdToGraphFile = graphIdToGraphFile || ((graphId) => `resource_models/${graphId}.json`);
      this.graphIdToResourcesFiles = graphIdToResourcesFiles || ((graphId) => [
        `business_data/_${graphId}.json`
      ]);
      this.resourceIdToFile = resourceIdToFile || ((resourceId) => `business_data/${resourceId}.json`);
      this.collectionIdToFile = collectionIdToFile || ((collectionId) => `collections/${collectionId}.json`);
    }
    async getGraphs() {
      const response = await fetch(`${this.archesUrl}/${this.allGraphFile()}`);
      return await response.json();
    }
    async getGraph(graph) {
      if (!this.graphToGraphFile) {
        return this.getGraphByIdOnly(graph.graphid);
      }
      const response = await fetch(`${this.archesUrl}/${this.graphToGraphFile(graph)}`);
      return (await response.json()).graph[0];
    }
    async getGraphByIdOnly(graphId) {
      const response = await fetch(`${this.archesUrl}/${this.graphIdToGraphFile(graphId)}`);
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
      const response = await fetch(`${this.archesUrl}/${this.collectionIdToFile(collectionId)}`);
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
    constructor({ allGraphFile, graphToGraphFile, graphIdToResourcesFiles, resourceIdToFile, collectionIdToFile, graphIdToGraphFile } = {}) {
      super();
      __publicField(this, "fs");
      __publicField(this, "allGraphFile");
      __publicField(this, "graphToGraphFile");
      __publicField(this, "graphIdToGraphFile");
      __publicField(this, "graphIdToResourcesFiles");
      __publicField(this, "resourceIdToFile");
      __publicField(this, "collectionIdToFile");
      this.fs = import("./__vite-browser-external-DGN5jhtd.js");
      this.allGraphFile = allGraphFile || (() => "tests/definitions/models/_all.json");
      this.graphToGraphFile = graphToGraphFile || ((graph) => `tests/definitions/models/${graph.graphid}.json`);
      this.graphIdToGraphFile = graphIdToGraphFile || ((graphId) => `tests/definitions/models/${graphId}.json`);
      this.graphIdToResourcesFiles = graphIdToResourcesFiles || ((graphId) => [
        `tests/definitions/resources/_${graphId}.json`
      ]);
      this.resourceIdToFile = resourceIdToFile || ((resourceId) => `tests/definitions/resources/${resourceId}.json`);
      this.collectionIdToFile = collectionIdToFile || ((collectionId) => `tests/definitions/collections/${collectionId}.json`);
    }
    async getGraphs() {
      const fs = await this.fs;
      const response = await fs.promises.readFile(this.allGraphFile(), "utf8");
      return new GraphResult(await JSON.parse(response));
    }
    async getGraph(graph) {
      const fs = await this.fs;
      const graphFile = this.graphToGraphFile ? this.graphToGraphFile(graph) : this.graphIdToGraphFile(graph.graphid);
      if (!graphFile) {
        return null;
      }
      const response = await fs.promises.readFile(graphFile, "utf8");
      return await JSON.parse(response).graph[0];
    }
    async getGraphByIdOnly(graphId) {
      const fs = await this.fs;
      const graphFile = this.graphIdToGraphFile(graphId);
      if (!graphFile) {
        return null;
      }
      const response = await fs.promises.readFile(graphFile, "utf8");
      return await JSON.parse(response).graph[0];
    }
    async getResource(resourceId) {
      const fs = await this.fs;
      const source = this.resourceIdToFile(resourceId);
      const response = await fs.promises.readFile(source, "utf8");
      return JSON.parse(response).then((resource) => {
        resource.__source = source;
        return resource;
      });
    }
    async getCollection(collectionId) {
      const fs = await this.fs;
      const response = await fs.promises.readFile(this.collectionIdToFile(collectionId), "utf8");
      return await JSON.parse(response);
    }
    async getResources(graphId, limit) {
      const fs = await this.fs;
      const resources = [];
      for (const file of this.graphIdToResourcesFiles(graphId)) {
        const response = JSON.parse(await fs.promises.readFile(file, "utf8"));
        const source = file;
        const resourceSet = response.business_data.resources.filter((resource) => graphId === resource.resourceinstance.graph_id);
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
  client = Object.freeze(Object.defineProperty({
    __proto__: null,
    ArchesClient,
    ArchesClientLocal,
    ArchesClientRemote,
    ArchesClientRemoteStatic,
    GraphResult,
    archesClient
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  interfaces = Object.freeze(Object.defineProperty({
    __proto__: null
  }, Symbol.toStringTag, {
    value: "Module"
  }));
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
  RDM = new ReferenceDataManager(archesClient);
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
        return resource || null;
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
        if (resource.resourceinstance.graph_id !== graphId) {
          continue;
        }
        this.cache.set(resource.resourceinstance.resourceinstanceid, this.cacheMetadataOnly ? resource.resourceinstance : resource);
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
  staticStore = new StaticStore(archesClient);
  class CardComponent {
    constructor(id, name) {
      __publicField(this, "id");
      __publicField(this, "name");
      this.id = id;
      this.name = name;
    }
  }
  class Widget {
    constructor(id, name, datatype, defaultConfig) {
      __publicField(this, "id");
      __publicField(this, "name");
      __publicField(this, "datatype");
      __publicField(this, "defaultConfig");
      this.id = id;
      this.name = name;
      this.datatype = datatype;
      this.defaultConfig = defaultConfig;
    }
    getDefaultConfig() {
      return JSON.parse(this.defaultConfig);
    }
  }
  const DEFAULT_CARD_COMPONENT = new CardComponent("f05e4d3a-53c1-11e8-b0ea-784f435179ea", "Default Card");
  const _WIDGET_VALUES = [
    [
      "10000000-0000-0000-0000-000000000001",
      "text-widget",
      "string",
      '{ "placeholder": "Enter text", "width": "100%", "maxLength": null}'
    ],
    [
      "10000000-0000-0000-0000-000000000002",
      "concept-select-widget",
      "concept",
      '{ "placeholder": "Select an option", "options": [] }'
    ],
    [
      "10000000-0000-0000-0000-000000000012",
      "concept-multiselect-widget",
      "concept-list",
      '{ "placeholder": "Select an option", "options": [] }'
    ],
    [
      "10000000-0000-0000-0000-000000000015",
      "domain-select-widget",
      "domain-value",
      '{ "placeholder": "Select an option" }'
    ],
    [
      "10000000-0000-0000-0000-000000000016",
      "domain-multiselect-widget",
      "domain-value-list",
      '{ "placeholder": "Select an option" }'
    ],
    [
      "10000000-0000-0000-0000-000000000003",
      "switch-widget",
      "boolean",
      '{ "subtitle": "Click to switch"}'
    ],
    [
      "10000000-0000-0000-0000-000000000004",
      "datepicker-widget",
      "date",
      `{
      "placeholder": "Enter date",
      "viewMode": "days",
      "dateFormat": "YYYY-MM-DD",
      "minDate": false,
      "maxDate": false
    }`
    ],
    [
      "10000000-0000-0000-0000-000000000005",
      "rich-text-widget",
      "string",
      "{}"
    ],
    [
      "10000000-0000-0000-0000-000000000006",
      "radio-boolean-widget",
      "boolean",
      '{"trueLabel": "Yes", "falseLabel": "No"}'
    ],
    [
      "10000000-0000-0000-0000-000000000007",
      "map-widget",
      "geojson-feature-collection",
      `{
      "basemap": "streets",
      "geometryTypes": [{"text":"Point", "id":"Point"}, {"text":"Line", "id":"Line"}, {"text":"Polygon", "id":"Polygon"}],
      "overlayConfigs": [],
      "overlayOpacity": 0.0,
      "geocodeProvider": "MapzenGeocoder",
      "zoom": 0,
      "maxZoom": 20,
      "minZoom": 0,
      "centerX": 0,
      "centerY": 0,
      "pitch": 0.0,
      "bearing": 0.0,
      "geocodePlaceholder": "Search",
      "geocoderVisible": true,
      "featureColor": null,
      "featureLineWidth": null,
      "featurePointSize": null
    }`
    ],
    [
      "10000000-0000-0000-0000-000000000008",
      "number-widget",
      "number",
      '{ "placeholder": "Enter number", "width": "100%", "min":"", "max":""}'
    ],
    [
      "10000000-0000-0000-0000-000000000009",
      "concept-radio-widget",
      "concept",
      '{ "options": [] }'
    ],
    [
      "10000000-0000-0000-0000-000000000013",
      "concept-checkbox-widget",
      "concept-list",
      '{ "options": [] }'
    ],
    [
      "10000000-0000-0000-0000-000000000017",
      "domain-radio-widget",
      "domain-value",
      "{}"
    ],
    [
      "10000000-0000-0000-0000-000000000018",
      "domain-checkbox-widget",
      "domain-value-list",
      "{}"
    ],
    [
      "10000000-0000-0000-0000-000000000019",
      "file-widget",
      "file-list",
      '{"acceptedFiles": "", "maxFilesize": "200"}'
    ]
  ];
  const WIDGETS = Object.fromEntries(_WIDGET_VALUES.map((constructor) => [
    constructor[1],
    new Widget(...constructor)
  ]));
  function getDefaultWidgetForNode(node, preferences = {}) {
    const datatype = node.datatype;
    if (datatype in preferences) {
      return WIDGETS[preferences[datatype]];
    }
    if (datatype === "semantic") {
      throw Error("Not default widget for a semantic node");
    } else if (datatype === "number") {
      return WIDGETS["number-widget"];
    } else if (datatype === "string") {
      return WIDGETS["text-widget"];
    } else if (datatype === "concept") {
      return WIDGETS["concept-select-widget"];
    } else if (datatype === "concept-list") {
      return WIDGETS["concept-multiselect-widget"];
    } else if (datatype === "domain-value") {
      return WIDGETS["domain-select-widget"];
    } else if (datatype === "domain-value-list") {
      return WIDGETS["domain-multiselect-widget"];
    } else if (datatype === "geojson-feature-collection") {
      return WIDGETS["geojson-feature-collection"];
    } else if (datatype === "boolean") {
      return WIDGETS["switch-widget"];
    } else if (datatype === "date") {
      return WIDGETS["datepicker-widget"];
    } else {
      throw Error(`No default widget for ${datatype} datatype - perhaps you could supply a manual preference`);
    }
  }
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
  class StaticNodeConfigConcept {
    constructor(jsonData) {
      __publicField(this, "rdmCollection");
      this.rdmCollection = jsonData.rdmCollection;
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
  nodeConfig = Object.freeze(Object.defineProperty({
    __proto__: null,
    StaticNodeConfigBoolean,
    StaticNodeConfigConcept,
    StaticNodeConfigDomain,
    nodeConfigManager
  }, Symbol.toStringTag, {
    value: "Module"
  }));
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
    constructor(values, allNodegroups, wrapper, tiles) {
      __publicField(this, "values");
      __publicField(this, "wrapper");
      __publicField(this, "tiles");
      __publicField(this, "promises");
      __publicField(this, "writeLock");
      this.values = values;
      this.wrapper = wrapper;
      this.tiles = tiles;
      this.promises = allNodegroups;
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
      let result = this.values.get(key);
      if (Array.isArray(result)) {
        return result;
      }
      const node = this.wrapper.model.getNodeObjectsByAlias().get(key);
      result = await result;
      if (!node) {
        throw Error(`This key ${key} has no corresponding node`);
      }
      const nodegroupId = node.nodegroup_id || "";
      const promise = node ? await this.promises.get(nodegroupId) : false;
      if (promise === false) {
        await this.writeLock;
        if (this.wrapper.resource) {
          const node2 = this.wrapper.model.getNodeObjectsByAlias().get(key);
          if (node2 === void 0) {
            throw Error("Tried to retrieve a node key that does not exist on this resource");
          }
          const values = new Map([
            ...this.values.entries()
          ]);
          const promise2 = new Promise((resolve) => {
            return this.wrapper.ensureNodegroup(values, this.promises, nodegroupId, this.wrapper.model.getNodeObjects(), this.wrapper.model.getNodegroupObjects(), this.wrapper.model.getEdges(), false, this.tiles, true).then(async ([ngValues]) => {
              let original = false;
              const processValue = (k, concreteValue) => {
                if (key === k) {
                  original = concreteValue;
                }
                if (concreteValue !== false) {
                  this.values.set(k, concreteValue);
                }
              };
              return Promise.all([
                ...ngValues.entries()
              ].map(([k, value]) => {
                if (value instanceof Promise) {
                  return value.then((concreteValue) => processValue(k, concreteValue));
                }
                processValue(k, value);
              })).then(() => {
                resolve(original);
              });
            });
          });
          this.writeLock = promise2;
          this.promises.set(nodegroupId, promise2);
          this.values.set(key, promise2);
          await promise2;
          this.promises.set(nodegroupId, true);
        } else {
          this.values.delete(key);
        }
        result = await this.values.get(key);
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
        } else if (instance) {
          return new ConceptValueCacheEntry(instance);
        }
        return null;
      }).filter((cvce) => cvce !== null);
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
      __publicField(this, "_");
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
        _: await Promise.all([
          ...this.values()
        ].map(async (rivmPromise) => {
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
            throw Error(`Cannot set an (entire) resource list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
          }
          val = value.map((v, i) => {
            if (v instanceof ResourceInstanceViewModel) {
              return v;
            }
            return ResourceInstanceViewModel.__create(tile, node, v, cacheEntry && cacheEntry._[i] ? cacheEntry._[i] : null);
          });
          Promise.all(val.map(async (c) => {
            const v = await c;
            return v ? (await v).id : null;
          })).then((ids) => {
            tile.data.set(nodeid, ids.map((id) => {
              return {
                resourceId: id
              };
            }));
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
      __publicField(this, "$");
      __publicField(this, "__");
      __publicField(this, "__parentPseudo");
      __publicField(this, "__cacheEntry", null);
      __publicField(this, "id");
      __publicField(this, "then");
      __publicField(this, _c);
      __publicField(this, "gm");
      this.id = id;
      this.$ = instanceWrapperFactory ? instanceWrapperFactory(this) : null;
      this.__ = modelWrapper;
      if (cacheEntry instanceof ResourceInstanceCacheEntry) {
        this.__cacheEntry = cacheEntry;
      }
      return new Proxy(this, {
        set: async (object, key, value) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
          if (key in object) {
            object[key] = value;
          } else if (k in object || k.startsWith("__")) {
            object[k] = value;
          } else {
            if (!object.$) {
              await this.retrieve();
              if (!object.$) {
                throw Error("Could not retrieve resource");
              }
            }
            object.$.setOrmAttribute(k, value);
          }
          return true;
        },
        get: (object, key) => {
          const k = typeof key === "symbol" ? key.description || "" : key;
          if (key in object) {
            return object[key];
          } else if (k in object || k.startsWith("__")) {
            return object[k];
          }
          return new AttrPromise(async (resolve) => {
            if (!object.$) {
              await this.retrieve();
              if (!object.$) {
                throw Error("Could not retrieve resource");
              }
            }
            return object.$.getOrmAttribute(k).then((v) => {
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
      if (!this.$) {
        return void 0;
      }
      return (await this.$.getRootViewModel() || /* @__PURE__ */ new Map()).__has(key);
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
        if (!this.$) {
          await this.retrieve();
          if (!this.$) {
            throw Error("Could not retrieve resource");
          }
        }
        const root = await this.$.getRootViewModel();
        basic.root = await root.forJson();
      }
      return basic;
    }
    async retrieve() {
      let iw;
      let mw;
      if (viewContext.graphManager) {
        const replacement = await viewContext.graphManager.getResource(this.id, true);
        iw = replacement.$;
        mw = replacement.__;
      } else {
        throw Error("Cannot traverse resource relationships without a GraphManager");
      }
      this.$ = iw;
      this.__ = mw;
      return [
        iw,
        mw
      ];
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
            if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(value)) {
              val = value;
            } else {
              throw Error(`Set resource instances using id, not strings in node ${node.alias}: ${value}`);
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
          tile.data.set(nodeid, val ? [
            {
              resourceId: val
            }
          ] : null);
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
  class FileListViewModel extends Array {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value", null);
    }
    async forJson() {
      const value = await this._value;
      return value ? value.map((v) => v ? v.forJson() : null) : null;
    }
    async __forJsonCache() {
      return null;
    }
    static async __create(tile, node, value) {
      const nodeid = node.nodeid;
      let val = [];
      if (!tile.data.has(nodeid)) {
        tile.data.set(nodeid, null);
      }
      if (value !== null) {
        tile.data.set(nodeid, []);
        if (!Array.isArray(value)) {
          throw Error(`Cannot set an (entire) file list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
        }
        val = value.map((c) => {
          return c;
        });
        Promise.all(val).then((vals) => {
          Promise.all(vals.map(async (c) => {
            const v = await c;
            return v ? (await v.getValue()).id : null;
          })).then((ids) => {
            tile.data.set(nodeid, ids);
          });
        });
        value = val;
      } else {
        value = [];
      }
      const str = new FileListViewModel(...value);
      return str;
    }
    async __asTileData() {
      return this._value ? await this._value : null;
    }
  }
  class ConceptListViewModel extends Array {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
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
        _: (await Promise.all([
          ...this.values()
        ].map(async (rivmPromise) => {
          const rivm = await rivmPromise;
          if (rivm) {
            return await rivm.__forJsonCache(getMeta);
          }
        }))).filter((val) => val !== void 0)
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
          throw Error(`Cannot set an (entire) concept list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
        }
        val = value.map((c, i) => {
          if (c instanceof ConceptValueViewModel) {
            return c;
          }
          return ConceptValueViewModel.__create(tile, node, c, cacheEntry && cacheEntry._[i] ? cacheEntry._[i] : null);
        });
        Promise.all(val).then((vals) => {
          Promise.all(vals.map(async (c) => {
            const v = await c;
            return v ? (await v.getValue()).id : null;
          })).then((ids) => {
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
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value", null);
    }
    async forJson() {
      const value = await this._value;
      return value ? value.map((v) => v ? v.forJson() : null) : null;
    }
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
            throw Error(`Cannot set an (entire) domain list value on node ${nodeid} except via an array: ${JSON.stringify(value)}`);
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
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      __publicField(this, "_value");
      this._value = value;
    }
    async forJson() {
      return this._value;
    }
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
            if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-5][0-9a-f]{3}-[089ab][0-9a-f]{3}-[0-9a-f]{12}$/i.exec(value)) {
              const config = nodeConfigManager.retrieve(node);
              if (!config || !(config instanceof StaticNodeConfigDomain)) {
                throw Error(`Cannot form domain value for ${node.nodeid} without config`);
              }
              val = config.valueFromId(value) || null;
            } else {
              throw Error("Set domain values using values from domain lists, not strings");
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
      __publicField(this, "_");
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
      var _a2;
      const nodeid = node.nodeid;
      const collectionId = (_a2 = node.config) == null ? void 0 : _a2.rdmCollection;
      if (!collectionId) {
        throw Error(`Node ${node.alias} (${node.nodeid}) missing rdmCollection in config`);
      }
      let val = value;
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, null);
        }
        if (value !== null) {
          if (value instanceof StaticConcept) {
            if (value.getPrefLabel) {
              val = value.getPrefLabel();
            } else {
              throw Error("Recognizing value as StaticConcept, but no getPrefLabel member");
            }
          }
          if (!value) {
            val = null;
          } else if (value instanceof StaticValue) ;
          else if (value instanceof Promise) {
            return value.then((value2) => {
              return ConceptValueViewModel.__create(tile, node, value2, cacheEntry);
            });
          } else if (typeof value == "string") {
            if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.exec(value)) {
              if (cacheEntry) {
                val = new StaticValue({
                  id: cacheEntry.id,
                  value: cacheEntry.value,
                  __concept: null,
                  __conceptId: cacheEntry.conceptId
                }, cacheEntry.conceptId);
                return new ConceptValueViewModel(val);
              } else {
                const collection = RDM.retrieveCollection(collectionId);
                return collection.then((collection2) => {
                  if (!collection2.getConceptValue) {
                    throw Error(`Collection ${collection2.id} must be a StaticCollection here, not a key/value object`);
                  }
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
              throw Error(`Set concepts using values from collections, not strings: ${value}`);
            }
          } else {
            throw Error("Could not set concept from this data");
          }
          if (!(val instanceof Promise)) {
            if (!val) {
              console.error("Could not find concept for value", value, "for", node.alias, "in collection", collectionId);
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
      __publicField(this, "_");
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
        return value.then((value2) => DateViewModel.__create(tile, node, value2));
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
      __publicField(this, "_");
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
        return value.then((value2) => _GeoJSONViewModel.__create(tile, node, value2));
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
  class EDTFViewModel extends String {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
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
        return value.then((value2) => EDTFViewModel.__create(tile, node, value2));
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
      const string = new EDTFViewModel(val);
      return string;
    }
    __asTileData() {
      return `${this}`;
    }
  }
  class NonLocalizedStringViewModel extends String {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
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
      return `${this}`;
    }
  }
  class NumberViewModel extends Number {
    constructor() {
      super(...arguments);
      __publicField(this, "_");
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
      return this.valueOf();
    }
  }
  class BooleanViewModel extends Boolean {
    constructor(value, config) {
      super(value);
      __publicField(this, "_");
      __publicField(this, "__parentPseudo");
      __publicField(this, "__config");
      __publicField(this, "describeField", () => this.__parentPseudo ? this.__parentPseudo.describeField() : null);
      __publicField(this, "describeFieldGroup", () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null);
      this.__config = config;
    }
    toString(lang) {
      const labelLang = lang || DEFAULT_LANGUAGE;
      return this.valueOf() ? this.__config && this.__config.trueLabel ? this.__config.trueLabel[labelLang] || "true" : "true" : this.__config && this.__config.trueLabel ? this.__config.falseLabel[labelLang] || "false" : "false";
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
      if (typeof val !== "boolean" && val !== 0 && val !== 1) {
        throw Error(`Refusing to use truthiness for value ${val} in boolean`);
      }
      const bool = new BooleanViewModel(val ? true : false, config);
      return bool;
    }
    __asTileData() {
      return this.valueOf();
    }
  }
  class Url {
    constructor(url, url_label) {
      __publicField(this, "url");
      __publicField(this, "url_label");
      this.url = url;
      this.url_label = url_label;
    }
  }
  class UrlViewModel extends String {
    constructor(value) {
      const displayValue = value.url_label || value.url;
      super(displayValue);
      __publicField(this, "_");
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
      return {
        url: this._value.url,
        url_label: this._value.url_label || ""
      };
    }
    label() {
      return this._value.url_label || this._value.url;
    }
    href() {
      return this._value.url;
    }
    static __create(tile, node, value) {
      const nodeid = node.nodeid;
      if (value instanceof Promise) {
        return value.then((value2) => UrlViewModel.__create(tile, node, value2));
      }
      if (tile) {
        if (!tile.data.has(nodeid)) {
          tile.data.set(nodeid, {});
        }
        if (value !== null) {
          if (value instanceof UrlViewModel) {
            value = value._value;
          } else if (value instanceof Object) {
            if (!value.url) {
              throw Error(`A URL must be null or have a 'url' field: ${value}`);
            }
          }
          tile.data.set(nodeid, {
            url: value.url,
            url_label: value.url_label
          });
        }
      }
      const val = tile.data.get(nodeid);
      if (!tile || val === null || val === void 0) {
        return null;
      }
      let url;
      if (typeof val !== "object") {
        url = new Url(`${val}`);
      } else if (val instanceof Map) {
        url = new Url(val.get("url"), val.get("url_label"));
      } else if ("url" in val && typeof val === "object" && typeof val.url === "string" && "url_label" in val && (val.url_label === void 0 || typeof val.url_label === "string")) {
        url = new Url(val.url, val.url_label);
      } else {
        throw Error(`Unrecognised URL type: ${val}`);
      }
      const str = new UrlViewModel(url);
      return str;
    }
    __asTileData() {
      return this.forJson();
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
      __publicField(this, "_");
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
            for (const [k, v] of [
              ...entries
            ]) {
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
      __publicField(this, "_");
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
          if (key.toString() === "Symbol.toStringTag") {
            return () => this.constructor.name;
          }
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
      const entries = [
        ...this.__childValues.entries()
      ].map(([k, v]) => `${k}: ${v}`);
      return `[[${entries.join(",")}]]`;
    }
    async toObject() {
      const entries = [
        ...(await this.__getChildValues()).entries()
      ];
      return Object.fromEntries(await Promise.all(entries.map(async ([k, vl]) => {
        return [
          k,
          (await vl).getValue()
        ];
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
      const entries = [
        ...(await this.__getChildValues()).entries()
      ];
      return Object.fromEntries(await Promise.all(entries.map(async ([k, vl]) => {
        return [
          k,
          vl ? await _forJson(vl) : vl
        ];
      })));
    }
    async __update(map) {
      return Promise.all([
        ...map.entries()
      ].map(([k, v]) => {
        this.__set(k, v);
      }));
    }
    async __get(key) {
      const childValue = await this.__getChildValue(key);
      return childValue.getValue();
    }
    async __set(key, value) {
      if (!this.__childNodes.has(key)) {
        throw Error(`Semantic node does not have this key: ${key} (${[
          ...this.__childNodes.keys()
        ]})`);
      }
      throw Error(`Setting semantic keys (${key} = ${value}) is not implemented yet in Javascript`);
    }
    __has(key) {
      return this.__childNodes.has(key);
    }
    async __getChildTypes() {
      const promises = [
        ...this.__childNodes.keys()
      ].map(async (key) => [
        key,
        await this.__getChildValue(key)
      ]);
      const entries = await Promise.all(promises);
      return new Map([
        ...entries
      ]);
    }
    async __getChildren(direct = null) {
      const items = /* @__PURE__ */ new Map();
      for (const [key, value] of [
        ...(await this.__getChildValues()).entries()
      ]) {
        items.set(key, value);
      }
      const children = [
        ...items.entries()
      ].filter((entry) => {
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
        throw Error(`Semantic node does not have this key: ${key} (${[
          ...this.__childNodes.keys()
        ]})`);
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
      if (!this.__parentWkri.$) {
        throw Error("This semantic node is currently on an unloaded WKRI");
      }
      const child = this.__parentWkri.$.addPseudo(childNode, this.__tile, this.__node);
      child.parentNode = this.__parentPseudo || null;
      return child;
    }
    static async __create(tile, node, value, parent, childNodes) {
      const svm = new _SemanticViewModel(parent, childNodes, tile, node);
      if (value) {
        try {
          await svm.__update(value);
        } catch (e) {
          tileLoadingError(`
          Suppressed a tile loading error: ${e}: ${typeof e} (tile: ${tile}; node: ${node}) - ${value}
        `, e);
        }
      }
      return svm;
    }
    async __asTileData() {
      const relationships = [];
      for (const value of [
        ...await this.__getChildren(true)
      ]) {
        const [, subrelationships] = await value.getTile();
        relationships.push(...subrelationships);
      }
      return [
        null,
        relationships
      ];
    }
    async __getChildValues() {
      const parent = this.__parentWkri;
      const childNodes = this.__childNodes;
      const tile = this.__tile;
      const node = this.__node;
      if (!parent || !parent.$) {
        return /* @__PURE__ */ new Map();
      }
      await parent.$.loadNodes([
        ...childNodes.keys()
      ]);
      const children = /* @__PURE__ */ new Map();
      for (const entry of [
        ...parent.$.allEntries()
      ]) {
        const key = entry[0];
        let values = entry[1];
        if (values instanceof Promise) {
          values = await values;
        }
        if (values === false || values === null || values === void 0) {
          continue;
        }
        const childNode = childNodes.get(key);
        if (childNode) {
          for (let value of values) {
            if (value !== null && value.node && (!value.parentNode || value.parentNode === this.__parentPseudo)) {
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
      }
      for (const [key, value] of [
        ...children.entries()
      ]) {
        value.parentNode = this.__parentPseudo;
        this.__childValues.set(key, value);
      }
      return children;
    }
  };
  let SemanticViewModel = _SemanticViewModel;
  const CUSTOM_DATATYPES = /* @__PURE__ */ new Map();
  async function getViewModel(parentPseudo, tile, node, data, parent, childNodes, isInner = false) {
    let vm;
    const cacheEntries = parentPseudo.parent && parentPseudo.parent.$ ? await parentPseudo.parent.$.getValueCache(false, void 0) : void 0;
    let cacheEntry = null;
    if (cacheEntries) {
      cacheEntry = (tile.tileid ? cacheEntries[tile.tileid] ?? {} : {})[node.nodeid];
    }
    const datatype = isInner ? "semantic" : CUSTOM_DATATYPES.get(node.datatype) ?? node.datatype;
    let conceptCacheEntry;
    let conceptValueCacheEntry;
    let resourceInstanceCacheEntry;
    let resourceInstanceListCacheEntry;
    if (!(typeof datatype == "string")) {
      vm = await datatype.__create(tile, node, data, cacheEntry);
    } else {
      switch (datatype) {
        case "semantic":
          vm = await SemanticViewModel.__create(tile, node, data, parent, childNodes);
          break;
        case "domain-value":
          vm = await DomainValueViewModel.__create(tile, node, data);
          break;
        case "domain-value-list":
          vm = await DomainValueListViewModel.__create(tile, node, data);
          break;
        case "concept":
          if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptValueCacheEntry)) {
            conceptValueCacheEntry = new ConceptValueCacheEntry(cacheEntry);
          } else {
            conceptValueCacheEntry = cacheEntry;
          }
          vm = await ConceptValueViewModel.__create(tile, node, data, conceptValueCacheEntry);
          break;
        case "resource-instance":
          if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceCacheEntry)) {
            resourceInstanceCacheEntry = new ResourceInstanceCacheEntry(cacheEntry);
          } else {
            resourceInstanceCacheEntry = cacheEntry;
          }
          vm = await ResourceInstanceViewModel.__create(tile, node, data, resourceInstanceCacheEntry);
          break;
        case "resource-instance-list":
          if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ResourceInstanceListCacheEntry)) {
            resourceInstanceListCacheEntry = new ResourceInstanceListCacheEntry(cacheEntry);
          } else {
            resourceInstanceListCacheEntry = cacheEntry;
          }
          vm = await ResourceInstanceListViewModel.__create(tile, node, data, resourceInstanceListCacheEntry);
          break;
        case "concept-list":
          if (cacheEntry && typeof cacheEntry === "object" && !(cacheEntry instanceof ConceptListCacheEntry)) {
            conceptCacheEntry = new ConceptListCacheEntry(cacheEntry);
          } else {
            conceptCacheEntry = cacheEntry;
          }
          vm = await ConceptListViewModel.__create(tile, node, data, conceptCacheEntry);
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
        case "file-list":
          vm = await FileListViewModel.__create(tile, node, data);
          break;
        case "edtf":
          vm = await EDTFViewModel.__create(tile, node, data);
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
            if (vmep !== null) vmep.__parentPseudo = parentPseudo;
          });
        } else {
          vme.__parentPseudo = parentPseudo;
        }
      }
    }
    return vm;
  }
  viewModels = Object.freeze(Object.defineProperty({
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
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  class PseudoUnavailable {
    constructor(node) {
      __publicField(this, "parentNode", null);
      __publicField(this, "tile", null);
      __publicField(this, "node");
      __publicField(this, "isOuter", false);
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
    getValue() {
      return new AttrPromise((resolve) => resolve(null));
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
    constructor(node, tile, value, parent, childNodes, inner) {
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
      __publicField(this, "isOuter", false);
      __publicField(this, "isInner", false);
      __publicField(this, "inner", null);
      __publicField(this, "independent");
      this.node = node;
      this.tile = tile;
      this.independent = tile === null;
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
      if (inner instanceof PseudoValue) {
        this.isOuter = true;
        this.inner = inner;
      }
      if (inner === true) {
        this.isInner = true;
        this.datatype = "semantic";
      }
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
      if (this.parent && this.node.nodegroup_id && this.parent.$) {
        const nodegroup = this.parent.$.model.getNodeObjects().get(this.node.nodegroup_id);
        if (nodegroup && this.parent.__) {
          fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroup.name}`;
        }
      }
      return fieldName;
    }
    getParentTileId() {
      return this.tile ? this.tile.parenttile_id : null;
    }
    async getTile() {
      await this.updateValue();
      let relationships = [];
      if (this.inner) {
        [this.tile, relationships] = await this.inner.getTile();
      }
      let tileValue;
      if (this.value !== null) {
        const [newTileValue, ownRelationships] = await (await this.value).__asTileData();
        tileValue = newTileValue;
        relationships = [
          ...relationships,
          ...ownRelationships
        ];
      } else {
        tileValue = null;
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
      const tile = this.independent ? this.tile : null;
      return [
        tile,
        relationships
      ];
    }
    clear() {
      this.value = null;
      if (this.tile && this.tile.data && this.tile.data.has(this.node.nodeid)) {
        this.tile.data.delete(this.node.nodeid);
      }
    }
    updateValue(tile) {
      if (tile) {
        this.tile = tile;
      }
      this.accessed = true;
      if (this.inner) {
        this.inner.accessed = true;
      }
      if (!this.tile) {
        if (!this.node) {
          throw Error("Empty tile");
        }
        if (this.inner) {
          return new AttrPromise(async (resolve) => {
            var _a2;
            const tile2 = await ((_a2 = this.inner) == null ? void 0 : _a2.getTile());
            resolve(this.updateValue(tile2 ? tile2[0] : void 0));
          });
        }
        if (!this.tile) {
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
      }
      if (this.valueLoaded === false) {
        this.valueLoaded = void 0;
        let data;
        if (this.value === null && this.tile.data !== null && this.tile.data.has(this.node.nodeid) && this.datatype !== "semantic") {
          data = this.tile.data.get(this.node.nodeid);
        } else {
          data = this.value;
        }
        if (this.isOuter && typeof data === "object" && this.inner && data) {
          let outerData = void 0;
          if ("_" in data && !data.constructor) {
            outerData = data["_"];
            delete data["_"];
            this.inner.getValue().then((v) => v && v.update(data));
            data = outerData;
          } else if (data instanceof Map && data.has("_")) {
            outerData = data.get("_");
            data.delete("_");
            this.inner.getValue().then((v) => v && v.update(data));
            data = outerData;
          }
        }
        const vm = getViewModel(this, this.tile, this.node, data, this.parent, this.childNodes, this.isInner);
        const resolveAttr = (vm2) => {
          if (vm2 !== null && vm2 instanceof Object) {
            vm2.__parentPseudo = this;
            if (this.isOuter && this.inner) {
              vm2._ = this.inner.getValue();
            }
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
    getLength() {
      return this.getChildren().length;
    }
    async getChildTypes() {
      await this.updateValue();
      let childTypes = {};
      if (this.value && this.value instanceof Object && "getChildTypes" in this.value && typeof this.value.getChildTypes === "function") {
        childTypes = this.value.getChildTypes();
      }
      if (this.inner) {
        Object.assign(childTypes, this.inner.getChildTypes());
      }
      return childTypes;
    }
    getChildren(direct = null) {
      let children = [];
      if (this.value && this.value instanceof Object && "getChildren" in this.value && typeof this.value.getChildren === "function") {
        children = this.value.getChildren(direct);
      }
      if (this.inner) {
        children = [
          ...children,
          ...this.inner.getChildren(direct)
        ];
      }
      return children;
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
      __publicField(this, "isOuter", false);
    }
    isIterable() {
      return true;
    }
    async sorted() {
      const resolved = await Promise.all(this.map(async (pn) => await pn));
      const sorted = resolved.sort((a, b) => {
        const vals = [
          a,
          b
        ].map((val) => {
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
      if (this.parent && this.node.nodegroup_id && this.parent.$) {
        const nodegroup = this.parent.$.model.getNodeObjects().get(this.node.nodegroup_id);
        if (nodegroup && this.parent.__) {
          fieldName = `${this.parent.__.wkrm.modelName} - ${nodegroup.name}`;
        }
      }
      return `[${fieldName}]`;
    }
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
      const array = Array.from((await this.sorted()).map(async (entry) => {
        const value = await entry;
        return value && value instanceof Object && value.forJson ? value.forJson() : value;
      }));
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
        let inner = false;
        if (childNodes && childNodes.size && nodeObj.datatype !== "semantic") {
          inner = new PseudoValue(nodeObj, tile, null, wkri, childNodes, true);
        }
        nodeValue = new PseudoValue(nodeObj, tile, null, wkri, inner !== false ? /* @__PURE__ */ new Map() : childNodes, inner);
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
  const DESCRIPTOR_FUNCTION_ID = "60000000-0000-0000-0000-000000000001";
  WKRM = class {
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
  };
  class ConfigurationOptions {
    constructor() {
      __publicField(this, "graphs");
      __publicField(this, "eagerLoadGraphs", false);
      this.graphs = null;
    }
  }
  class ResourceInstanceWrapper {
    constructor(wkri, model, resource, pruneTiles = true) {
      __publicField(this, "wkri");
      __publicField(this, "model");
      __publicField(this, "resource");
      __publicField(this, "valueList");
      __publicField(this, "cache");
      __publicField(this, "scopes");
      __publicField(this, "metadata");
      this.wkri = wkri;
      this.model = model;
      if (resource) {
        this.model.stripTiles(resource);
      }
      this.resource = resource;
      this.valueList = new ValueList(/* @__PURE__ */ new Map(), /* @__PURE__ */ new Map(), this, []);
      this.cache = resource ? resource.__cache : void 0;
      this.scopes = resource ? resource.__scopes : void 0;
      this.metadata = resource ? resource.metadata : void 0;
      if (pruneTiles && this.resource) {
        this.pruneResourceTiles();
      }
    }
    pruneResourceTiles() {
      if (!this.resource) {
        console.warn("Trying to prune tiles for an empty resource", this.wkri.modelClassName);
        return;
      }
      this.resource.tiles = (this.resource.tiles || []).filter((tile) => {
        return this.model.isNodegroupPermitted(tile.nodegroup_id || "", tile);
      });
    }
    async loadNodes(aliases) {
      for (const key of aliases) {
        await this.valueList.retrieve(key);
      }
    }
    async getName(update = false) {
      let resourceName = this.resource && this.resource.resourceinstance.name;
      if (update || !resourceName) {
        const descriptors = await this.getDescriptors(update);
        resourceName = descriptors && descriptors.name || resourceName || "<Unnamed>";
        if (this.resource && this.resource.resourceinstance) {
          this.resource.resourceinstance.name = resourceName;
        }
      }
      return resourceName;
    }
    async getDescriptors(update = false) {
      let descriptors = this.resource && this.resource.resourceinstance.descriptors;
      if (update || !descriptors || descriptors.isEmpty()) {
        descriptors = new StaticResourceDescriptors();
        let descriptorConfig = void 0;
        if (this.model.graph.functions_x_graphs) {
          const descriptorNode = this.model.graph.functions_x_graphs.find((node) => node.function_id === DESCRIPTOR_FUNCTION_ID);
          if (descriptorNode) {
            descriptorConfig = descriptorNode.config;
          }
        }
        const nodes = this.model.getNodeObjects();
        if (descriptorConfig) {
          for (const [descriptor, config] of Object.entries(descriptorConfig.descriptor_types)) {
            const semanticNode = nodes.get(config.nodegroup_id);
            let description = config.string_template;
            if (!description) {
              continue;
            }
            let requestedNodes = description.match(/<[A-Za-z _-]*>/g) || [];
            const relevantNodes = [
              ...nodes.values()
            ].filter((node) => node.nodegroup_id === config.nodegroup_id && [
              ...requestedNodes
            ].includes(`<${node.name}>`)).map((node) => [
              node.name,
              node.alias || ""
            ]);
            let relevantValues = [];
            if (semanticNode) {
              let semanticValue = await (await this.valueList.retrieve(semanticNode.alias || ""))[0];
              if (semanticValue instanceof PseudoList) {
                semanticValue = await semanticValue[0];
              } else if (semanticValue.inner) {
                relevantValues.push([
                  semanticNode.name || "",
                  await semanticValue.getValue()
                ]);
                semanticValue = await semanticValue.inner.getValue();
              } else {
                semanticValue = await semanticValue.getValue();
              }
              if (semanticValue) {
                relevantValues = [
                  ...relevantValues,
                  ...await Promise.all(relevantNodes.filter(([_, alias]) => semanticValue.__has(alias)).map(([name, alias]) => semanticValue[alias].then((value) => [
                    name,
                    value
                  ])))
                ];
              }
            }
            if (relevantValues) {
              description = relevantValues.reduce((desc, [name, value]) => value ? desc.replace(`<${name}>`, value) : desc, description);
            }
            requestedNodes = description.match(/<[A-Za-z _-]*>/g) || [];
            if (requestedNodes.length) {
              relevantValues = await Promise.all(relevantNodes.map(([name, alias]) => this.valueList.retrieve(alias).then((values) => [
                name,
                values ? values[0] : void 0
              ])));
              if (relevantValues) {
                description = relevantValues.reduce((desc, [name, value]) => value ? desc.replace(`<${name}>`, value) : desc, description);
              }
            }
            descriptors[descriptor] = description;
          }
        }
      }
      if (this.resource && this.resource.resourceinstance) {
        this.resource.resourceinstance.descriptors = descriptors;
        if (descriptors.name) {
          this.resource.resourceinstance.descriptors.name = descriptors.name;
        }
      }
      return descriptors;
    }
    addPseudo(childNode, tile, node) {
      const key = childNode.alias;
      if (!key) {
        throw Error(`Cannot add a pseudo node with no alias ${childNode.nodeid}`);
      }
      const child = makePseudoCls(this.model, key, false, !childNode.is_collector && childNode.nodegroup_id === node.nodegroup_id ? tile : null, this.wkri);
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
        throw Error(`Tried to get root on ${this.model.wkrm.modelClassName}, which has no root`);
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
      return new AttrPromise((resolve) => {
        promise.then(() => this.getRootViewModel()).then((root) => resolve(root[key]));
      });
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
          values.set(alias, [
            value
          ]);
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
    async ensureNodegroup(allValues, allNodegroups, nodegroupId, nodeObjs, nodegroupObjs, edges, addIfMissing, tiles, doImpliedNodegroups = true) {
      const impliedNodegroups = /* @__PURE__ */ new Set();
      const sentinel = allNodegroups.get(nodegroupId);
      let newValues = /* @__PURE__ */ new Map();
      if (sentinel === false || addIfMissing && sentinel === void 0) {
        [
          ...nodeObjs.values()
        ].filter((node) => {
          return node.nodegroup_id === nodegroupId;
        }).forEach((node) => allValues.delete(node.alias || ""));
        let nodegroupTiles;
        if (tiles === null) {
          nodegroupTiles = [];
          console.error("Tiles must be provided and cannot be lazy-loaded yet");
        } else {
          nodegroupTiles = tiles.filter((tile) => tile.nodegroup_id == nodegroupId && this.model.isNodegroupPermitted(nodegroupId, tile));
          if (nodegroupTiles.length == 0 && addIfMissing) {
            nodegroupTiles = [
              null
            ];
          }
          const rgValues = await this.valuesFromResourceNodegroup(allValues, nodegroupTiles, nodegroupId, nodeObjs, edges);
          newValues = rgValues[0];
          const newImpliedNodegroups = rgValues[1];
          [
            ...newValues.entries()
          ].forEach((entry) => {
            if (entry[1] !== void 0) {
              allValues.set(entry[0], entry[1]);
            }
          });
          [
            ...newImpliedNodegroups
          ].forEach((v) => {
            impliedNodegroups.add(v);
          });
          allNodegroups.set(nodegroupId, true);
        }
      }
      if (doImpliedNodegroups) {
        for (const nodegroupId2 of [
          ...impliedNodegroups
        ]) {
          const [impliedValues] = await this.ensureNodegroup(allValues, allNodegroups, nodegroupId2, nodeObjs, nodegroupObjs, edges, true, tiles, true);
          for (const [key, value] of impliedValues) {
            newValues.set(key, value);
          }
        }
        impliedNodegroups.clear();
      }
      return [
        newValues,
        impliedNodegroups
      ];
    }
    async populate(lazy) {
      const nodeObjs = this.model.getNodeObjects();
      const nodegroupObjs = this.model.getNodegroupObjects();
      const edges = this.model.getEdges();
      const allValues = /* @__PURE__ */ new Map();
      const allNodegroups = new Map([
        ...nodegroupObjs.keys()
      ].map((id) => {
        return [
          id || "",
          false
        ];
      }));
      const rootNode = this.model.getRootNode();
      if (rootNode.alias === null) {
        throw Error("Cannot populate a model with no proper root node");
      }
      allValues.set(rootNode.alias, false);
      let tiles = null;
      if (!lazy && this.resource) {
        tiles = this.resource.tiles;
        let impliedNodegroups = /* @__PURE__ */ new Set();
        for (const [ng] of nodegroupObjs) {
          const [_, newImpliedNodegroups] = await this.ensureNodegroup(allValues, allNodegroups, ng, nodeObjs, nodegroupObjs, edges, true, tiles, false);
          for (const impliedNodegroup of [
            ...newImpliedNodegroups
          ]) {
            impliedNodegroups.add(impliedNodegroup);
          }
          impliedNodegroups.delete(ng);
        }
        while (impliedNodegroups.size) {
          const newImpliedNodegroups = /* @__PURE__ */ new Set();
          for (const nodegroupId of [
            ...impliedNodegroups
          ]) {
            const currentValue = allNodegroups.get(nodegroupId);
            if (currentValue === false || currentValue === void 0) {
              const [_, newImpliedNodegroups2] = await this.ensureNodegroup(allValues, allNodegroups, nodegroupId, nodeObjs, nodegroupObjs, edges, true, tiles, true);
              for (const impliedNodegroup of [
                ...newImpliedNodegroups2
              ]) {
                newImpliedNodegroups2.add(impliedNodegroup);
              }
            }
          }
          impliedNodegroups = newImpliedNodegroups;
        }
      } else if (this.resource) {
        this.model.stripTiles(this.resource);
      }
      this.valueList = new ValueList(allValues, allNodegroups, this, this.resource ? this.resource.tiles : null);
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
      const impliedNodegroups = /* @__PURE__ */ new Set();
      const impliedNodes = /* @__PURE__ */ new Map();
      const nodesUnseen = new Set([
        ...nodeObjs.values()
      ].filter((node) => node.nodegroup_id == nodegroupId).map((node) => node.alias));
      const tileNodesSeen = /* @__PURE__ */ new Set();
      const _addPseudo = async (node, tile) => {
        const key = node.alias || "";
        nodesUnseen.delete(node.alias);
        const tileid = tile && tile.tileid;
        if (tileid) {
          tileNodesSeen.add([
            node.nodeid,
            tileid
          ]);
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
            const toAdd = domainNode.nodegroup_id ? domainNode.nodegroup_id : "";
            if (toAdd && toAdd !== nodegroupId) {
              impliedNodegroups.add(toAdd);
            }
            if (domainNode.nodegroup_id && tile && domainNode.nodegroup_id === tile.nodegroup_id && domainNode.nodegroup_id !== domainNode.nodeid && tileid && !impliedNodes.has(domainNode.nodeid + tileid)) {
              impliedNodes.set(domainNode.nodeid + tileid, [
                domainNode,
                tile
              ]);
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
          for (const [key, value] of [
            ...tile.data.entries()
          ]) {
            tileNodes.set(key, value);
          }
          [
            ...nodeObjs.values()
          ].filter((node) => {
            return node.nodegroup_id === nodegroupId && !tileNodes.get(node.nodeid) && node.datatype === "semantic";
          }).forEach((node) => tileNodes.set(node.nodeid, {}));
          if (!tileNodes.has(tile.nodegroup_id)) {
            tileNodes.set(tile.nodegroup_id, {});
          }
          for (const [nodeid, nodeValue] of [
            ...tileNodes.entries()
          ]) {
            if (nodeid == nodegroupId) {
              continue;
            }
            const node = nodeObjs.get(nodeid);
            if (!node) {
              throw Error(`Unknown node in nodegroup: ${nodeid} in ${nodegroupId}`);
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
          if (tile.tileid && !tileNodesSeen.has([
            node.nodeid,
            tile.tileid
          ])) {
            await _addPseudo(node, tile);
          }
          impliedNodes.delete(value[0]);
        }
      }
      [
        ...nodesUnseen.keys()
      ].forEach((nodeUnseen) => {
        if (nodeUnseen) {
          allValues.set(nodeUnseen, void 0);
        }
      });
      return [
        allValues,
        impliedNodegroups
      ];
    }
  }
  GraphMutator = class {
    constructor(baseGraph, options = {}) {
      __publicField(this, "baseGraph");
      __publicField(this, "mutations");
      __publicField(this, "autocreateCard");
      this.baseGraph = baseGraph;
      this.mutations = [];
      this.autocreateCard = options.autocreateCard === void 0 || options.autocreateCard;
    }
    _generateUuidv5(key) {
      return generateUuidv5([
        "graph",
        this.baseGraph.graphid
      ], key);
    }
    _generateEdge(fromNode, toNode, ontologyProperty, name, description) {
      const edgeId = this._generateUuidv5(`node-${fromNode}-${toNode}`);
      return new StaticEdge({
        description: description || null,
        domainnode_id: fromNode,
        edgeid: edgeId,
        graph_id: this.baseGraph.graphid,
        name: name || null,
        rangenode_id: toNode,
        ontologyproperty: ontologyProperty
      });
    }
    addSemanticNode(parentAlias, alias, name, cardinality, ontologyClass, parentProperty, description, options = {}, config) {
      return this._addGenericNode(parentAlias, alias, name, cardinality, "semantic", ontologyClass, parentProperty, description, options, config);
    }
    addConceptNode(parentAlias, alias, name, collection, cardinality, ontologyClass, parentProperty, description, options = {}, config) {
      config = config || {};
      if (collection == null ? void 0 : collection.id) {
        config["rdmCollection"] = collection.id;
      }
      return this._addGenericNode(parentAlias, alias, name, cardinality, options.is_list ? "concept-list" : "concept", ontologyClass, parentProperty, description, options, config);
    }
    addCard(nodegroup, name, component, options = {}, config) {
      const nodegroupId = typeof nodegroup === "string" ? nodegroup : nodegroup.nodegroupid;
      const cardName = name instanceof StaticTranslatableString ? name : new StaticTranslatableString(name);
      const cardComponent = component || DEFAULT_CARD_COMPONENT;
      const helptext = (options == null ? void 0 : options.helptext) && (options.helptext instanceof StaticTranslatableString ? options.helptext : new StaticTranslatableString(options.helptext));
      const helptitle = (options == null ? void 0 : options.helptitle) && (options.helptitle instanceof StaticTranslatableString ? options.helptitle : new StaticTranslatableString(options.helptitle));
      const instructions = (options == null ? void 0 : options.instructions) && (options.instructions instanceof StaticTranslatableString ? options.instructions : new StaticTranslatableString(options.instructions));
      this.mutations.push((graph) => {
        graph.cards = graph.cards || [];
        if (graph.cards.filter((card2) => card2.nodegroup_id === nodegroup).length > 0) {
          throw Error(`This nodegroup, ${nodegroupId}, already has a card`);
        }
        const cardId = this._generateUuidv5(`card-ng-${nodegroupId}`);
        const card = new StaticCard({
          active: options.active === void 0 ? true : options.active,
          cardid: cardId,
          component_id: cardComponent.id,
          config: config || null,
          constraints: options.constraints || [],
          cssclass: options.cssclass || null,
          description: options.description || null,
          graph_id: graph.graphid,
          helpenabled: !!(options.helpenabled || options.helpenabled === void 0 && (helptext || helptitle)),
          helptext: helptext || new StaticTranslatableString(""),
          helptitle: helptitle || new StaticTranslatableString(""),
          instructions: instructions || new StaticTranslatableString(""),
          is_editable: options.is_editable === void 0 ? true : options.is_editable,
          name: cardName,
          nodegroup_id: nodegroupId,
          sortorder: options.sortorder || null,
          visible: options.visible === void 0 ? true : options.visible
        });
        graph.cards.push(card);
        return graph;
      });
    }
    addStringNode(parentAlias, alias, name, cardinality, ontologyClass, parentProperty, description, options = {}, config) {
      return this._addGenericNode(parentAlias, alias, name, cardinality, "string", ontologyClass, parentProperty, description, options, config);
    }
    _addNodegroup(parentAlias, nodegroupId, cardinality, name) {
      this.mutations.push((graph) => {
        const prnt = parentAlias === null ? graph.root : graph.nodes.find((node) => node.alias === parentAlias);
        if (!prnt) {
          throw Error(`Missing parent for nodegroup: ${parentAlias}`);
        }
        const nodegroup = new StaticNodegroup({
          cardinality,
          legacygroupid: null,
          nodegroupid: nodegroupId,
          parentnodegroup_id: prnt.nodegroup_id
        });
        graph.nodegroups.push(nodegroup);
        return graph;
      });
      if (this.autocreateCard) {
        this.addCard(nodegroupId, name || "(unnamed)");
      }
      return this;
    }
    _addGenericNode(parentAlias, alias, name, cardinality, datatype, ontologyClass, parentProperty, description, options = {}, config) {
      const nodeId = this._generateUuidv5(`node-${alias}`);
      const node = {
        alias,
        config: config || {},
        datatype,
        description: description || null,
        exportable: options.exportable || false,
        fieldname: options.fieldname || null,
        graph_id: this.baseGraph.graphid,
        hascustomalias: options.hascustomalias || false,
        is_collector: options.is_collector || false,
        isrequired: options.isrequired || false,
        issearchable: options.issearchable || true,
        istopnode: options.istopnode || false,
        name,
        nodegroup_id: "",
        nodeid: nodeId,
        parentproperty: parentProperty,
        sortorder: options.sortorder || 0,
        ontologyclass: ontologyClass,
        sourcebranchpublication_id: null
      };
      if (cardinality === "n" || parentAlias === null) {
        node.nodegroup_id = nodeId;
        this._addNodegroup(parentAlias, node.nodegroup_id, cardinality, new StaticTranslatableString(name));
      }
      this.mutations.push((graph) => {
        const prnt = parentAlias === null ? graph.root : graph.nodes.find((node2) => node2.alias === parentAlias);
        if (!prnt) {
          throw Error(`Parent node does not exist: ${parentAlias}`);
        }
        node.nodegroup_id = node.nodegroup_id !== "" ? node.nodegroup_id : prnt.nodegroup_id || "";
        const newNode = new StaticNode(node);
        graph.nodes.push(newNode);
        const edge = this._generateEdge(prnt.nodeid, nodeId, parentProperty);
        graph.edges.push(edge);
        return graph;
      });
      if (this.autocreateCard && datatype !== "semantic") {
        const widget = getDefaultWidgetForNode(node);
        const config2 = widget.getDefaultConfig();
        config2.label = name;
        this.addWidgetToCard(nodeId, widget, name, config2, {
          sortorder: node.sortorder,
          silentSkip: true
        });
      }
      return this;
    }
    addWidgetToCard(nodeId, widget, name, config, options = {}) {
      this.mutations.push((graph) => {
        var _a2;
        const node = graph.nodes.find((node2) => node2.nodeid === nodeId);
        if (!node) {
          throw Error(`Tried to add card to graph ${graph.graphid} for node ${nodeId} but it was not found.`);
        }
        const card = (_a2 = graph.cards) == null ? void 0 : _a2.find((card2) => card2.nodegroup_id === node.nodegroup_id);
        if (card) {
          const cardXNodeXWidgetId = this._generateUuidv5(`cxnxw-${nodeId}-${widget.id}`);
          const cardXNodeXWidget = new StaticCardsXNodesXWidgets({
            card_id: card.cardid,
            config,
            id: cardXNodeXWidgetId,
            label: new StaticTranslatableString(name),
            node_id: nodeId,
            sortorder: options.sortorder || 0,
            visible: options.visible === void 0 || options.visible,
            widget_id: widget.id
          });
          graph.cards_x_nodes_x_widgets = graph.cards_x_nodes_x_widgets || [];
          graph.cards_x_nodes_x_widgets.push(cardXNodeXWidget);
        } else if (!options.silentSkip) {
          throw Error(`Failed adding widget for ${nodeId} to card for ${node.nodegroup_id} on graph ${graph.graphid}, as no card for this nodegroup (yet?)`);
        }
        return graph;
      });
      return this;
    }
    apply() {
      if (!this.baseGraph.copy) {
        throw Error("Attempt to build a mutator without a proper StaticGraph base graph");
      }
      const graph = this.baseGraph.copy();
      return this.mutations.reduce((graph2, mutation) => mutation(graph2), graph);
    }
  };
  ResourceModelWrapper = class {
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
    getBranchPublicationIds(accessible) {
      const accessibleOnly = accessible || false;
      const nodes = [
        ...this.graph.nodes.values()
      ];
      return [
        ...nodes.reduce((acc, node) => {
          if (node.sourcebranchpublication_id) {
            if (accessibleOnly) {
              if (this.isNodegroupPermitted(node.nodegroup_id || "", null)) {
                acc.add(node.sourcebranchpublication_id);
              }
            } else {
              acc.add(node.sourcebranchpublication_id);
            }
          }
          return acc;
        }, /* @__PURE__ */ new Set())
      ];
    }
    getCollections(accessible) {
      const accessibleOnly = accessible || false;
      const nodes = [
        ...this.graph.nodes.values()
      ];
      return [
        ...nodes.reduce((acc, node) => {
          var _a2;
          if ([
            "concept",
            "concept-list"
          ].includes(node.datatype) && ((_a2 = node.config) == null ? void 0 : _a2.rdmCollection)) {
            if (accessibleOnly) {
              if (this.isNodegroupPermitted(node.nodegroup_id || "", null)) {
                acc.add(node.config.rdmCollection);
              }
            } else {
              acc.add(node.config.rdmCollection);
            }
          }
          return acc;
        }, /* @__PURE__ */ new Set())
      ];
    }
    pruneGraph(keepFunctions) {
      const allNodegroups = this.getNodegroupObjects();
      const root = this.graph.root.nodeid;
      const allowedNodegroups = new Map([
        ...allNodegroups.values()
      ].filter((nodegroup) => {
        return this.isNodegroupPermitted(nodegroup.nodegroupid || "", null);
      }).map((nodegroup) => [
        nodegroup.nodegroupid,
        nodegroup.nodegroupid === null || nodegroup.nodegroupid === "" || nodegroup.nodegroupid === root
      ]));
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
        const unrooted = [
          ...allowedNodegroups.entries()
        ].filter(([_, rooted]) => !rooted);
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
      const allowedNodes = new Set([
        ...this.getNodeObjects().values()
      ].filter((node) => {
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
      const paramObj = params || {
        limit: void 0,
        lazy: void 0
      };
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
    async *resourceGenerator(staticResources, lazy = false, pruneTiles = true) {
      for await (const staticResource of staticResources) {
        yield this.fromStaticResource(staticResource, lazy, pruneTiles);
      }
    }
    async *iterAll(params) {
      yield* this.resourceGenerator(staticStore.loadAll(this.wkrm.graphId, params.limit), params.lazy);
    }
    async findStatic(id) {
      return await staticStore.loadOne(id);
    }
    async find(id, lazy = true, pruneTiles = true) {
      const rivm = await this.findStatic(id);
      return this.fromStaticResource(rivm, lazy, pruneTiles);
    }
    setPermittedNodegroups(permissions) {
      const nodegroups = this.getNodegroupObjects();
      const nodes = this.getNodeObjectsByAlias();
      this.permittedNodegroups = new Map([
        ...permissions
      ].map(([key, value]) => {
        const k = key || "";
        if (nodegroups.has(k) || k === "") {
          return [
            key,
            value
          ];
        } else {
          const node = nodes.get(k);
          if (node) {
            return [
              node.nodeid,
              value
            ];
          } else {
            throw Error(`Could not find ${key} in nodegroups for permissions`);
          }
        }
      }));
    }
    getPermittedNodegroups() {
      if (!this.permittedNodegroups) {
        const permissions = new Map([
          ...this.getNodegroupObjects()
        ].map(([k, _]) => [
          k,
          true
        ]));
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
    makeInstance(id, resource, pruneTiles = true) {
      if (!this.viewModelClass) {
        throw Error(`Cannot instantiate without a viewModelClass in ${this.wkrm.modelClassName}`);
      }
      const instance = new this.viewModelClass(id, this.viewModelClass.prototype.__, (rivm) => new ResourceInstanceWrapper(rivm, this, resource, pruneTiles), null);
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
      const nodes = new Map(graph.nodes.map((node) => [
        node.nodeid,
        node
      ]));
      const nodegroups = new Map(graph.nodes.filter((node) => node.nodegroup_id).map((node) => [
        node.nodegroup_id || "",
        new StaticNodegroup({
          cardinality: "n",
          legacygroupid: null,
          nodegroupid: node.nodegroup_id || "",
          parentnodegroup_id: null
        })
      ]));
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
      this.nodesByAlias = new Map([
        ...nodes.values()
      ].map((node) => [
        node.alias || "",
        node
      ]));
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
      const rootNode = [
        ...nodes.values()
      ].find((n) => !n.nodegroup_id);
      if (!rootNode) {
        throw Error(`COULD NOT FIND ROOT NODE FOR ${this.wkrm.modelClassName}. Does the graph ${this.graph.graphid} still exist?`);
      }
      rootNode.alias = rootNode.alias || "";
      return rootNode;
    }
    fromStaticResource(resource, lazy = false, pruneTiles = true) {
      const wkri = this.makeInstance(resource.resourceinstance.resourceinstanceid, resource, pruneTiles);
      if (!wkri.$) {
        throw Error("Could not load resource from static definition");
      }
      return wkri.$.populate(lazy).then(() => wkri);
    }
    asTree() {
      const root = this.getRootNode();
      const nodegroups = this.getNodegroupObjects();
      const addChildren = (node) => {
        const branch = {};
        const children = this.getChildNodes(node.nodeid);
        if (!children.size) {
          return false;
        }
        for (const child of children.values()) {
          const nodegroup = nodegroups.get(child.nodegroup_id || "");
          const multiple = child.nodegroup_id && child.is_collector && nodegroup && nodegroup.cardinality == "n" && node.nodegroup_id !== child.nodegroup_id || child.datatype.endsWith("-list");
          const childBranch = addChildren(child);
          const alias = child.alias || "";
          if (childBranch === false) {
            branch[alias] = child.datatype;
          } else {
            branch[alias] = childBranch;
            if (child.datatype !== "semantic") {
              branch[alias]["_"] = child.datatype;
            }
          }
          if (multiple) {
            branch[alias] = [
              branch[alias]
            ];
          }
        }
        return branch;
      };
      return addChildren(root) || {};
    }
  };
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
  GraphManager = class {
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
      let graphs = Object.entries(graphJsons["models"]);
      const allowedGraphs = configurationOptions.graphs;
      if (allowedGraphs !== null) {
        if (allowedGraphs === false) {
          throw Error("No current meaning of allowedGraphs === false");
        } else if (allowedGraphs !== true) {
          graphs = graphs.filter(([graphId, _]) => allowedGraphs.includes(graphId));
        }
      }
      graphs.forEach(([graphId, meta]) => {
        meta.graphid = meta.graphid || graphId;
        const wkrm = new WKRM(meta);
        this.wkrms.set(wkrm.modelClassName, wkrm);
      });
      if (configurationOptions.eagerLoadGraphs) {
        await Promise.all(graphs.map(([g]) => this.loadGraph(g)));
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
      let wkrm = this.wkrms.get(modelClassName);
      if (wkrm === void 0) {
        wkrm = [
          ...this.wkrms.values()
        ].find((wkrm2) => wkrm2.graphId === modelClassName);
        if (wkrm === void 0) {
          throw Error(`Only loading graphs for which metadata is present, not ${modelClassName}`);
        }
        modelClass = wkrm.modelClassName;
      }
      const wrapper = this.graphs.get(wkrm.graphId);
      if (wrapper !== void 0) {
        return wrapper;
      }
      const bodyJson = await this.archesClient.getGraph(wkrm.meta);
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
      let wkrm = this.wkrms.get(modelClassName);
      if (wkrm === void 0) {
        wkrm = [
          ...this.wkrms.values()
        ].find((w) => w.graphId === modelClassName);
        if (wkrm === void 0) {
          throw Error(`Cannot find model requested: ${modelClassName}`);
        }
      }
      const wrapper = this.graphs.get(wkrm.graphId);
      if (wrapper === void 0) {
        return this.loadGraph(modelClass);
      }
      return wrapper;
    }
    async getResource(resourceId, lazy = true, pruneTiles = true) {
      const rivm = await staticStore.loadOne(resourceId);
      let graph = this.graphs.get(rivm.resourceinstance.graph_id);
      if (!graph) {
        graph = await this.loadGraph(rivm.resourceinstance.graph_id);
        if (!graph) {
          throw Error(`Graph not found for resource ${resourceId}`);
        }
      }
      return graph.fromStaticResource(rivm, lazy, pruneTiles);
    }
    getGraph(graphId) {
      const wrapper = this.graphs.get(graphId);
      if (wrapper === void 0) {
        throw Error(`Cannot find graph requested: ${graphId}`);
      }
      return wrapper.graph;
    }
  };
  graphManager = new GraphManager(archesClient);
  viewContext.graphManager = graphManager;
  class Cleanable extends String {
    constructor() {
      super(...arguments);
      __publicField(this, "__clean");
    }
  }
  class BaseRenderer {
    async render(asset) {
      if (!asset.$) {
        throw Error("Cannot render unloaded asset - do you want to await asset.retrieve()?");
      }
      const root = await await asset.$.getRootViewModel();
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
        promises.push(this.renderValue(value, depth + 1).then((val) => {
          renderedBlock[key] = val;
        }));
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
      const text = `[${value}](${value})`;
      const wrapper = new Cleanable(text);
      wrapper.__clean = value.href();
      return wrapper;
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
      const children = [
        ...(await vm.__getChildValues()).entries()
      ].map(([_, v]) => [
        v.node.alias,
        v.node
      ]);
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
    async renderBoolean(value, _depth) {
      return typeof value === "boolean" ? value : value.forJson();
    }
    async renderConceptValue(value, _depth) {
      return value.forJson();
    }
    async renderDomainValue(value, _depth) {
      return value.forJson();
    }
    async renderResourceReference(value, _depth) {
      const val = value.forJson();
      return val;
    }
  }
  renderers = Object.freeze(Object.defineProperty({
    __proto__: null,
    Cleanable,
    FlatMarkdownRenderer,
    JsonRenderer,
    MarkdownRenderer
  }, Symbol.toStringTag, {
    value: "Module"
  }));
  let wasm;
  function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_2.set(idx, obj);
    return idx;
  }
  function handleError(f2, args) {
    try {
      return f2.apply(this, args);
    } catch (e) {
      const idx = addToExternrefTable0(e);
      wasm.__wbindgen_exn_store(idx);
    }
  }
  const cachedTextDecoder = typeof TextDecoder !== "undefined" ? new TextDecoder("utf-8", {
    ignoreBOM: true,
    fatal: true
  }) : {
    decode: () => {
      throw Error("TextDecoder not available");
    }
  };
  if (typeof TextDecoder !== "undefined") {
    cachedTextDecoder.decode();
  }
  let cachedUint8ArrayMemory0 = null;
  function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
      cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
  }
  function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
  }
  let WASM_VECTOR_LEN = 0;
  const cachedTextEncoder = typeof TextEncoder !== "undefined" ? new TextEncoder("utf-8") : {
    encode: () => {
      throw Error("TextEncoder not available");
    }
  };
  const encodeString = typeof cachedTextEncoder.encodeInto === "function" ? function(arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
  } : function(arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
      read: arg.length,
      written: buf.length
    };
  };
  function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === void 0) {
      const buf = cachedTextEncoder.encode(arg);
      const ptr2 = malloc(buf.length, 1) >>> 0;
      getUint8ArrayMemory0().subarray(ptr2, ptr2 + buf.length).set(buf);
      WASM_VECTOR_LEN = buf.length;
      return ptr2;
    }
    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;
    const mem = getUint8ArrayMemory0();
    let offset = 0;
    for (; offset < len; offset++) {
      const code = arg.charCodeAt(offset);
      if (code > 127) break;
      mem[ptr + offset] = code;
    }
    if (offset !== len) {
      if (offset !== 0) {
        arg = arg.slice(offset);
      }
      ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
      const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
      const ret = encodeString(arg, view);
      offset += ret.written;
      ptr = realloc(ptr, len, offset, 1) >>> 0;
    }
    WASM_VECTOR_LEN = offset;
    return ptr;
  }
  let cachedDataViewMemory0 = null;
  function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || cachedDataViewMemory0.buffer.detached === void 0 && cachedDataViewMemory0.buffer !== wasm.memory.buffer) {
      cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
  }
  function isLikeNone(x) {
    return x === void 0 || x === null;
  }
  function debugString(val) {
    const type = typeof val;
    if (type == "number" || type == "boolean" || val == null) {
      return `${val}`;
    }
    if (type == "string") {
      return `"${val}"`;
    }
    if (type == "symbol") {
      const description = val.description;
      if (description == null) {
        return "Symbol";
      } else {
        return `Symbol(${description})`;
      }
    }
    if (type == "function") {
      const name = val.name;
      if (typeof name == "string" && name.length > 0) {
        return `Function(${name})`;
      } else {
        return "Function";
      }
    }
    if (Array.isArray(val)) {
      const length = val.length;
      let debug = "[";
      if (length > 0) {
        debug += debugString(val[0]);
      }
      for (let i = 1; i < length; i++) {
        debug += ", " + debugString(val[i]);
      }
      debug += "]";
      return debug;
    }
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
      className = builtInMatches[1];
    } else {
      return toString.call(val);
    }
    if (className == "Object") {
      try {
        return "Object(" + JSON.stringify(val) + ")";
      } catch (_) {
        return "Object";
      }
    }
    if (val instanceof Error) {
      return `${val.name}: ${val.message}
${val.stack}`;
    }
    return className;
  }
  function greet() {
    wasm.greet();
  }
  typeof FinalizationRegistry === "undefined" ? {} : new FinalizationRegistry((ptr) => wasm.__wbg_staticgraphmeta_free(ptr >>> 0, 1));
  typeof FinalizationRegistry === "undefined" ? {} : new FinalizationRegistry((ptr) => wasm.__wbg_staticnode_free(ptr >>> 0, 1));
  async function __wbg_load(module, imports) {
    if (typeof Response === "function" && module instanceof Response) {
      if (typeof WebAssembly.instantiateStreaming === "function") {
        try {
          return await WebAssembly.instantiateStreaming(module, imports);
        } catch (e) {
          if (module.headers.get("Content-Type") != "application/wasm") {
            console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
          } else {
            throw e;
          }
        }
      }
      const bytes = await module.arrayBuffer();
      return await WebAssembly.instantiate(bytes, imports);
    } else {
      const instance = await WebAssembly.instantiate(module, imports);
      if (instance instanceof WebAssembly.Instance) {
        return {
          instance,
          module
        };
      } else {
        return instance;
      }
    }
  }
  function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_buffer_609cc3eee51ed158 = function(arg0) {
      const ret = arg0.buffer;
      return ret;
    };
    imports.wbg.__wbg_call_672a4d21634d4a24 = function() {
      return handleError(function(arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_done_769e5ede4b31c67b = function(arg0) {
      const ret = arg0.done;
      return ret;
    };
    imports.wbg.__wbg_entries_3265d4158b33e5dc = function(arg0) {
      const ret = Object.entries(arg0);
      return ret;
    };
    imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
      let deferred0_0;
      let deferred0_1;
      try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
      } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
      }
    };
    imports.wbg.__wbg_get_67b2ba62fc30de12 = function() {
      return handleError(function(arg0, arg1) {
        const ret = Reflect.get(arg0, arg1);
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_get_b9b93047fe3cf45b = function(arg0, arg1) {
      const ret = arg0[arg1 >>> 0];
      return ret;
    };
    imports.wbg.__wbg_getwithrefkey_1dc361bd10053bfe = function(arg0, arg1) {
      const ret = arg0[arg1];
      return ret;
    };
    imports.wbg.__wbg_instanceof_ArrayBuffer_e14585432e3737fc = function(arg0) {
      let result;
      try {
        result = arg0 instanceof ArrayBuffer;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    };
    imports.wbg.__wbg_instanceof_Map_f3469ce2244d2430 = function(arg0) {
      let result;
      try {
        result = arg0 instanceof Map;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    };
    imports.wbg.__wbg_instanceof_Uint8Array_17156bcf118086a9 = function(arg0) {
      let result;
      try {
        result = arg0 instanceof Uint8Array;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    };
    imports.wbg.__wbg_isArray_a1eab7e0d067391b = function(arg0) {
      const ret = Array.isArray(arg0);
      return ret;
    };
    imports.wbg.__wbg_isSafeInteger_343e2beeeece1bb0 = function(arg0) {
      const ret = Number.isSafeInteger(arg0);
      return ret;
    };
    imports.wbg.__wbg_iterator_9a24c88df860dc65 = function() {
      const ret = Symbol.iterator;
      return ret;
    };
    imports.wbg.__wbg_length_a446193dc22c12f8 = function(arg0) {
      const ret = arg0.length;
      return ret;
    };
    imports.wbg.__wbg_length_e2d2a49132c1b256 = function(arg0) {
      const ret = arg0.length;
      return ret;
    };
    imports.wbg.__wbg_new_405e22f390576ce2 = function() {
      const ret = new Object();
      return ret;
    };
    imports.wbg.__wbg_new_5e0be73521bc8c17 = function() {
      const ret = /* @__PURE__ */ new Map();
      return ret;
    };
    imports.wbg.__wbg_new_78feb108b6472713 = function() {
      const ret = new Array();
      return ret;
    };
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
      const ret = new Error();
      return ret;
    };
    imports.wbg.__wbg_new_a12002a7f91c75be = function(arg0) {
      const ret = new Uint8Array(arg0);
      return ret;
    };
    imports.wbg.__wbg_next_25feadfc0913fea9 = function(arg0) {
      const ret = arg0.next;
      return ret;
    };
    imports.wbg.__wbg_next_6574e1a8a62d1055 = function() {
      return handleError(function(arg0) {
        const ret = arg0.next();
        return ret;
      }, arguments);
    };
    imports.wbg.__wbg_set_37837023f3d740e8 = function(arg0, arg1, arg2) {
      arg0[arg1 >>> 0] = arg2;
    };
    imports.wbg.__wbg_set_3f1d0b984ed272ed = function(arg0, arg1, arg2) {
      arg0[arg1] = arg2;
    };
    imports.wbg.__wbg_set_65595bdd868b3009 = function(arg0, arg1, arg2) {
      arg0.set(arg1, arg2 >>> 0);
    };
    imports.wbg.__wbg_set_8fc6bf8a5b1071d1 = function(arg0, arg1, arg2) {
      const ret = arg0.set(arg1, arg2);
      return ret;
    };
    imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
      const ret = arg1.stack;
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_cd1ffa7b1ab794f1 = function(arg0) {
      const ret = arg0.value;
      return ret;
    };
    imports.wbg.__wbindgen_as_number = function(arg0) {
      const ret = +arg0;
      return ret;
    };
    imports.wbg.__wbindgen_bigint_from_i64 = function(arg0) {
      const ret = arg0;
      return ret;
    };
    imports.wbg.__wbindgen_bigint_from_u64 = function(arg0) {
      const ret = BigInt.asUintN(64, arg0);
      return ret;
    };
    imports.wbg.__wbindgen_bigint_get_as_i64 = function(arg0, arg1) {
      const v = arg1;
      const ret = typeof v === "bigint" ? v : void 0;
      getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbindgen_boolean_get = function(arg0) {
      const v = arg0;
      const ret = typeof v === "boolean" ? v ? 1 : 0 : 2;
      return ret;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
      const ret = debugString(arg1);
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
      const ret = new Error(getStringFromWasm0(arg0, arg1));
      return ret;
    };
    imports.wbg.__wbindgen_in = function(arg0, arg1) {
      const ret = arg0 in arg1;
      return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
      const table = wasm.__wbindgen_export_2;
      const offset = table.grow(4);
      table.set(0, void 0);
      table.set(offset + 0, void 0);
      table.set(offset + 1, null);
      table.set(offset + 2, true);
      table.set(offset + 3, false);
    };
    imports.wbg.__wbindgen_is_bigint = function(arg0) {
      const ret = typeof arg0 === "bigint";
      return ret;
    };
    imports.wbg.__wbindgen_is_function = function(arg0) {
      const ret = typeof arg0 === "function";
      return ret;
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
      const val = arg0;
      const ret = typeof val === "object" && val !== null;
      return ret;
    };
    imports.wbg.__wbindgen_is_string = function(arg0) {
      const ret = typeof arg0 === "string";
      return ret;
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
      const ret = arg0 === void 0;
      return ret;
    };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
      const ret = arg0 === arg1;
      return ret;
    };
    imports.wbg.__wbindgen_jsval_loose_eq = function(arg0, arg1) {
      const ret = arg0 == arg1;
      return ret;
    };
    imports.wbg.__wbindgen_memory = function() {
      const ret = wasm.memory;
      return ret;
    };
    imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
      const obj = arg1;
      const ret = typeof obj === "number" ? obj : void 0;
      getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
      const ret = arg0;
      return ret;
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
      const obj = arg1;
      const ret = typeof obj === "string" ? obj : void 0;
      var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      var len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
      const ret = getStringFromWasm0(arg0, arg1);
      return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
      throw new Error(getStringFromWasm0(arg0, arg1));
    };
    return imports;
  }
  function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
  }
  async function __wbg_init(module_or_path) {
    if (wasm !== void 0) return wasm;
    if (typeof module_or_path !== "undefined") {
      if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
        ({ module_or_path } = module_or_path);
      } else {
        console.warn("using deprecated parameters for the initialization function; pass a single object instead");
      }
    }
    if (typeof module_or_path === "undefined") {
      module_or_path = new URL("data:application/wasm;base64,AGFzbQEAAAAB9AI5YAAAYAABf2AAAW9gAX8AYAF/AX9gAX8Cf39gAX8Df39/YAF/AXxgAn9/AGACf38Bf2ACf38BfmACf38Bb2ADf39/AGADf39/AX9gA39/fwF+YAR/f39/AGAEf39/fwF/YAV/f39/fwBgBX9/f39/AX9gBn9/f39/fwBgBn9/f39/fwF/YAd/f39/f39/AGAHf39/f39/fwF/YAl/f39/f39+fn4AYAN/f34AYAV/f35/fwBgBX9/fX9/AGADf398AGAFf398f38AYAN/f28AYAJ/fgBgA39+fwF/YAR/fn9/AGAEf31/fwBgAn98AGAEf3x/fwBgBH98f38Bf2ACf28AYAJ/bwJ/f2ADf29vAGABfgF/YAF+AW9gA35/fwF/YAF8AX9gAXwBb2ABbwF/YAFvA39/f2ABbwF8YAFvAW9gAm9/AW9gA29/bwBgAm9vAX9gAm9vA39/f2ACb28Bb2ADb29/AGADb29vAGADb29vAW8Clw0zA3diZxdfX3diaW5kZ2VuX2lzX3VuZGVmaW5lZAAtA3diZw1fX3diaW5kZ2VuX2luADMDd2JnFl9fd2JpbmRnZW5fYm9vbGVhbl9nZXQALQN3YmcUX193YmluZGdlbl9pc19iaWdpbnQALQN3YmcVX193YmluZGdlbl9udW1iZXJfZ2V0ACUDd2JnGl9fd2JpbmRnZW5fYmlnaW50X2Zyb21faTY0ACkDd2JnE19fd2JpbmRnZW5fanN2YWxfZXEAMwN3YmcVX193YmluZGdlbl9zdHJpbmdfZ2V0ACUDd2JnFF9fd2JpbmRnZW5faXNfb2JqZWN0AC0Dd2JnGl9fd2JpbmRnZW5fYmlnaW50X2Zyb21fdTY0ACkDd2JnFF9fd2JpbmRnZW5fZXJyb3JfbmV3AAsDd2JnGV9fd2JpbmRnZW5fanN2YWxfbG9vc2VfZXEAMwN3YmcUX193YmluZGdlbl9hc19udW1iZXIALwN3YmcVX193YmluZGdlbl9udW1iZXJfbmV3ACwDd2JnFV9fd2JpbmRnZW5fc3RyaW5nX25ldwALA3diZyRfX3diZ19nZXR3aXRocmVma2V5XzFkYzM2MWJkMTAwNTNiZmUANQN3YmcaX193Ymdfc2V0XzNmMWQwYjk4NGVkMjcyZWQANwN3YmcaX193YmdfZ2V0X2I5YjkzMDQ3ZmUzY2Y0NWIAMQN3YmcdX193YmdfbGVuZ3RoX2UyZDJhNDkxMzJjMWIyNTYALQN3YmcaX193YmdfbmV3Xzc4ZmViMTA4YjY0NzI3MTMAAgN3YmcWX193YmluZGdlbl9pc19mdW5jdGlvbgAtA3diZxpfX3diZ19uZXdfNWUwYmU3MzUyMWJjOGMxNwACA3diZxtfX3diZ19uZXh0XzI1ZmVhZGZjMDkxM2ZlYTkAMAN3YmcbX193YmdfbmV4dF82NTc0ZTFhOGE2MmQxMDU1ADADd2JnG19fd2JnX2RvbmVfNzY5ZTVlZGU0YjMxYzY3YgAtA3diZxxfX3diZ192YWx1ZV9jZDFmZmE3YjFhYjc5NGYxADADd2JnH19fd2JnX2l0ZXJhdG9yXzlhMjRjODhkZjg2MGRjNjUAAgN3YmcaX193YmdfZ2V0XzY3YjJiYTYyZmMzMGRlMTIANQN3YmcbX193YmdfY2FsbF82NzJhNGQyMTYzNGQ0YTI0ADUDd2JnGl9fd2JnX25ld180MDVlMjJmMzkwNTc2Y2UyAAIDd2JnFF9fd2JpbmRnZW5faXNfc3RyaW5nAC0Dd2JnGl9fd2JnX3NldF8zNzgzNzAyM2YzZDc0MGU4ADIDd2JnHl9fd2JnX2lzQXJyYXlfYTFlYWI3ZTBkMDY3MzkxYgAtA3diZy1fX3diZ19pbnN0YW5jZW9mX0FycmF5QnVmZmVyX2UxNDU4NTQzMmUzNzM3ZmMALQN3YmclX193YmdfaW5zdGFuY2VvZl9NYXBfZjM0NjljZTIyNDRkMjQzMAAtA3diZxpfX3diZ19zZXRfOGZjNmJmOGE1YjEwNzFkMQA4A3diZyRfX3diZ19pc1NhZmVJbnRlZ2VyXzM0M2UyYmVlZWVjZTFiYjAALQN3YmceX193YmdfZW50cmllc18zMjY1ZDQxNThiMzNlNWRjADADd2JnHV9fd2JnX2J1ZmZlcl82MDljYzNlZWU1MWVkMTU4ADADd2JnGl9fd2JnX25ld19hMTIwMDJhN2Y5MWM3NWJlADADd2JnGl9fd2JnX3NldF82NTU5NWJkZDg2OGIzMDA5ADYDd2JnHV9fd2JnX2xlbmd0aF9hNDQ2MTkzZGMyMmMxMmY4AC0Dd2JnLF9fd2JnX2luc3RhbmNlb2ZfVWludDhBcnJheV8xNzE1NmJjZjExODA4NmE5AC0Dd2JnGl9fd2JnX25ld184YTZmMjM4YTZlY2U4NmVhAAIDd2JnHF9fd2JnX3N0YWNrXzBlZDc1ZDY4NTc1YjBmM2MAJQN3YmccX193YmdfZXJyb3JfNzUzNGI4ZTlhMzZmMWFiNAAIA3diZxxfX3diaW5kZ2VuX2JpZ2ludF9nZXRfYXNfaTY0ACUDd2JnF19fd2JpbmRnZW5fZGVidWdfc3RyaW5nACUDd2JnEF9fd2JpbmRnZW5fdGhyb3cACAN3YmcRX193YmluZGdlbl9tZW1vcnkAAgN3YmcfX193YmluZGdlbl9pbml0X2V4dGVybnJlZl90YWJsZQAAA/0E+wQTDxMRBA8IEQgIDAkMEQkRDA8NCQgMCBAkEREIDREMFAMIDyQMCQ8PDw0JCQgJDQ0NDAwICgwTCAkIDAwMDAwXDAgIEQkJKg0ICAwEDBUVExYMCAgMCQ8JDAwMCAgJCQkTEwkMAwkDDA8ICAgJCAkJCAgfEQgMCA8TExAECQkICAgICAgICBERBAgICBEICAwMBAgJDwgIDAgPCAkRBBERCAgIAwgICA8ICAwMDAwMDAwMCA8PAwMDEREICAwIDAwMDAkICBgNAwgPAxgMCQgREQwJEREREREMAwMICQgIAwwRCAgICAgMAwwMCAgMCAgIDAgMDAgTCAwMDAwEDAwIAwMDCAgIDQ0ICBEMBA8PDw8REQwIERERCQkMCAgICAgICAgIDwMEDA8DCAgIDw8PDwwNAwMNCQkIDQgDCAgIBAMACBERBAgIDAkEBAwJAwgDAwgMBAQEBAQECA8DEgMDAwwEDBEEAwgQAwMDAwMICQkIAwgDCDQICCIPLgYuBgYDCAMDCSYPCAUFBQUFBQUFBQUFBQUbFAkDAw4ODg4ODgMRERoSERkcEgEBAxAPDwMICQ0NCAknAwQPCB4DAwgJAwwJDAkJAwMDAwgICQkMAwwJCQQEBAQEBBAMDA8NDQ0MAwgJDSgoKwwdCBsNCQkRAQEBAQEBJSUJCQkMBAwRCQwMCQkJDAgICQkICQkJCQkEDAwMDAQEBAQEBA0MDAkDCQkJCQkJCQkICQkDCQMJCQMJCQMDCQkJCQkIAwMAAAkICQkIDQgICAgICQkJCQkJCQQECAMBBAQEBAcEBAQEBAQEBAQECAgIBAEAAAABAQEBBAQEBAEACQADAwQJAnABY2NvAIABBQMBABEGCQF/AUGAgMAACwehDT8GbWVtb3J5AgAaX193Ymdfc3RhdGljZ3JhcGhtZXRhX2ZyZWUAhQETc3RhdGljZ3JhcGhtZXRhX25ldwCuAxtzdGF0aWNncmFwaG1ldGFfZ2V0X2dyYXBoaWQAuwMbc3RhdGljZ3JhcGhtZXRhX3NldF9ncmFwaGlkAO8BFnN0YXRpY2dyYXBobWV0YV90b0pTT04ArwMZc3RhdGljZ3JhcGhtZXRhX2dldEF1dGhvcgC8AxlzdGF0aWNncmFwaG1ldGFfc2V0QXV0aG9yAN0BHXN0YXRpY2dyYXBobWV0YV9nZXRJc1Jlc291cmNlAPICHXN0YXRpY2dyYXBobWV0YV9zZXRJc1Jlc291cmNlALwCFV9fd2JnX3N0YXRpY25vZGVfZnJlZQDHAg5zdGF0aWNub2RlX25ldwCwAw9zdGF0aWNub2RlX2NvcHkA2QIRc3RhdGljbm9kZV90b0pTT04AsQMUc3RhdGljbm9kZV9nZXRfYWxpYXMAvQMUc3RhdGljbm9kZV9zZXRfYWxpYXMA3gEXc3RhdGljbm9kZV9nZXRfZGF0YXR5cGUAvgMXc3RhdGljbm9kZV9zZXRfZGF0YXR5cGUA8QEac3RhdGljbm9kZV9nZXRfZGVzY3JpcHRpb24AvwMac3RhdGljbm9kZV9zZXRfZGVzY3JpcHRpb24A3wEZc3RhdGljbm9kZV9nZXRfZXhwb3J0YWJsZQCHAxlzdGF0aWNub2RlX3NldF9leHBvcnRhYmxlAM8CGHN0YXRpY25vZGVfZ2V0X2ZpZWxkbmFtZQDAAxhzdGF0aWNub2RlX3NldF9maWVsZG5hbWUA4AEXc3RhdGljbm9kZV9nZXRfZ3JhcGhfaWQAwQMXc3RhdGljbm9kZV9zZXRfZ3JhcGhfaWQA8gEdc3RhdGljbm9kZV9nZXRfaGFzY3VzdG9tYWxpYXMAiAMdc3RhdGljbm9kZV9zZXRfaGFzY3VzdG9tYWxpYXMA0AIbc3RhdGljbm9kZV9nZXRfaXNfY29sbGVjdG9yAIkDG3N0YXRpY25vZGVfc2V0X2lzX2NvbGxlY3RvcgDRAhlzdGF0aWNub2RlX2dldF9pc3JlcXVpcmVkAIoDGXN0YXRpY25vZGVfc2V0X2lzcmVxdWlyZWQA0gIbc3RhdGljbm9kZV9nZXRfaXNzZWFyY2hhYmxlAIsDG3N0YXRpY25vZGVfc2V0X2lzc2VhcmNoYWJsZQDTAhhzdGF0aWNub2RlX2dldF9pc3RvcG5vZGUAjAMYc3RhdGljbm9kZV9zZXRfaXN0b3Bub2RlANQCE3N0YXRpY25vZGVfZ2V0X25hbWUAwgMTc3RhdGljbm9kZV9zZXRfbmFtZQDzARtzdGF0aWNub2RlX2dldF9ub2RlZ3JvdXBfaWQAwwMbc3RhdGljbm9kZV9zZXRfbm9kZWdyb3VwX2lkAOEBFXN0YXRpY25vZGVfZ2V0X25vZGVpZADEAxVzdGF0aWNub2RlX3NldF9ub2RlaWQA9AEcc3RhdGljbm9kZV9nZXRfb250b2xvZ3ljbGFzcwDFAxxzdGF0aWNub2RlX3NldF9vbnRvbG9neWNsYXNzAOIBHXN0YXRpY25vZGVfZ2V0X3BhcmVudHByb3BlcnR5AMYDHXN0YXRpY25vZGVfc2V0X3BhcmVudHByb3BlcnR5AOMBGHN0YXRpY25vZGVfZ2V0X3NvcnRvcmRlcgCYAxhzdGF0aWNub2RlX3NldF9zb3J0b3JkZXIA1gIpc3RhdGljbm9kZV9nZXRfc291cmNlYnJhbmNocHVibGljYXRpb25faWQAxwMpc3RhdGljbm9kZV9zZXRfc291cmNlYnJhbmNocHVibGljYXRpb25faWQA5AEUc3RhdGljbm9kZV9nZXRDb25maWcAsgMUc3RhdGljbm9kZV9zZXRDb25maWcAuAMSc3RhdGljbm9kZV9jb21wYXJlAKkDBG1haW4A9AIFZ3JlZXQAnwUUX193YmluZGdlbl9leG5fc3RvcmUAlAQXX19leHRlcm5yZWZfdGFibGVfYWxsb2MAnAUTX193YmluZGdlbl9leHBvcnRfMgEBD19fd2JpbmRnZW5fZnJlZQCTBBFfX3diaW5kZ2VuX21hbGxvYwCAAxJfX3diaW5kZ2VuX3JlYWxsb2MAmwMZX19leHRlcm5yZWZfdGFibGVfZGVhbGxvYwCPAxBfX3diaW5kZ2VuX3N0YXJ0AJ4FCc4BAgBBAQtigAX/BNoE2QT+BNcE/QSBBYIF2ASSAq8BzQPSA88D0APOA9EDmgWZBZgFtgRYwAStBIAC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akEIaigCADYCACAGIAYpAjw3A9ADDAELIAZBgICAgHg2AtADCwJAAkAgJ6ciF0EBcUUNACAGQeADakEYaiAIQRhqKQMANwMAIAZB4ANqQRBqIAhBEGopAwA3AwAgBkHgA2pBCGogCEEIaikDADcDACAGIAgpAwA3A+ADDAELIAZBADYC4AMLQQAhBwJAAkAgBigCcEGAgICAeEciGA0AQdeDwABBBxDLAiEBIABBAjYCACAAIAE2AgRBACEBQQAhH0EAIQRBACEeQQAhHUEAIRxBACEbQQAhGkEAIRlBACEADAELIAZBgARqQQhqIAZB8ABqQQhqKAIANgIAIAYgBikCcDcDgAQCQAJAIAYoAnwiIEGBgICAeEYNACAGQZAEakEIaiAGQfwAakEIaigCADYCACAGIAYpAnw3A5AEDAELIAZBgICAgHg2ApAECwJAAkAgKKciGUEBcUUNACAGQaAEakEYaiAJQRhqKQMANwMAIAZBoARqQRBqIAlBEGopAwA3AwAgBkGgBGpBCGogCUEIaikDADcDACAGIAkpAwA3A6AEDAELIAZBADYCoAQLAkACQCAppyIaQQFxRQ0AIAZBwARqQRhqIApBGGopAwA3AwAgBkHABGpBEGogCkEQaikDADcDACAGQcAEakEIaiAKQQhqKQMANwMAIAYgCikDADcDwAQMAQsgBkEANgLABAsCQAJAIAYoAtwBQYGAgIB4RyIbRQ0AIAZB4ARqQQhqIAZB3AFqQQhqKAIANgIAIAYgBikC3AE3A+AEDAELIAZBgICAgHg2AuAECwJAAkAgKqciHEEBcUUNACAGQfAEakEYaiALQRhqKQMANwMAIAZB8ARqQRBqIAtBEGopAwA3AwAgBkHwBGpBCGogC0EIaikDADcDACAGIAspAwA3A/AEDAELIAZBADYC8AQLAkACQCAGKAKUAkGAgICAeEciHQ0AQbKEwABBHBDLAiEBIABBAjYCACAAIAE2AgRBACEHQQAhAUEAIR9BACEEQQAhHgwBCyAGQZAFakEIaiAGQZQCakEIaigCADYCACAGIAYpApQCNwOQBQJAAkAgBigCoAJBgICAgHhHIh4NAEHOhMAAQR8QywIhASAAQQI2AgAgACABNgIEQQAhB0EAIQFBACEfQQAhBAwBCyAGQaAFakEIaiAGQaACakEIaigCADYCACAGIAYpAqACNwOgBSAGIB9BACAEQQFxGyIHNgKsBQJAAkAgBigCtAJBgYCAgHhHIh8NACAGQYCAgIB4NgKwBQwBCyAGQbAFakEIaiAGQbQCakEIaigCADYCACAGIAYpArQCNwOwBQsCQAJAICunIgFBAXFFDQAgBkGYA2pBGGogDEEYaikDADcDACAGQZgDakEQaiAMQRBqKQMANwMAIAZBmANqQQhqIAxBCGopAwA3AwAgBiAMKQMANwOYAwwBCyAGQQA2ApgDCwJAAkAgBigC7AIiIUGBgICAeEYNACAGQcAFakEIaiAGQewCakEIaigCADYCACAGIAYpAuwCNwPABQwBCyAGQYCAgIB4NgLABQsCQCAFDQBBhIXAAEEMEMsCIQcgAEECNgIAIAAgBzYCBCAGQcAFahD5AwJAIAYoApgDRQ0AIAZBmANqIAZBGEEIEOACCyAhQYGAgIB4RyEHIAZBsAVqEPkDIAZBrAVqEPoBIAZBoAVqEKUDIAZBoAVqEOYEDAELIAAgBikDwAM3AowCIAAgBikD0AM3ApgCIAAgBikD4AM3A0ggACANKQMANwMwIABBOGogDUEIaikDADcDACAAQcAAaiANQRBqKQMANwMAIABBlAJqIAZBwANqQQhqKAIANgIAIABBoAJqIAZB0ANqQQhqKAIANgIAIABB0ABqIAZB4ANqQQhqKQMANwMAIABB2ABqIAZB4ANqQRBqKQMANwMAIABB4ABqIAZB4ANqQRhqKQMANwMAIAYoAvwCIQEgAEHwAWogBkHwAGpBCGooAgA2AgAgACAGKQJwNwLoASAAQawCaiAGQZAEakEIaigCADYCACAAIAYpA5AENwKkAiAAQYABaiAGQaAEakEYaikDADcDACAAQfgAaiAGQaAEakEQaikDADcDACAAQfAAaiAGQaAEakEIaikDADcDACAAIAYpA6AENwNoIABBoAFqIAZBwARqQRhqKQMANwMAIABBmAFqIAZBwARqQRBqKQMANwMAIABBkAFqIAZBwARqQQhqKQMANwMAIAAgBikDwAQ3A4gBIABBuAJqIAZB4ARqQQhqKAIANgIAIAAgBikD4AQ3ArACIABBwAFqIAZB8ARqQRhqKQMANwMAIABBuAFqIAZB8ARqQRBqKQMANwMAIABBsAFqIAZB8ARqQQhqKQMANwMAIAAgBikD8AQ3A6gBIABB/AFqIAZBlAJqQQhqKAIANgIAIAAgBikClAI3AvQBIABBiAJqIAZBoAJqQQhqKAIANgIAIAAgBikCoAI3AoACIABBxAJqIAZBsAVqQQhqKAIANgIAIAAgBikDsAU3ArwCIABB4AFqIAZBmANqQRhqKQMANwMAIABB2AFqIAZBmANqQRBqKQMANwMAIABB0AFqIAZBmANqQQhqKQMANwMAIAAgBikDmAM3A8gBIABB0AJqIAZBwAVqQQhqKAIANgIAIAAgBikDwAU3AsgCIABBAiASIBJB/wFxQQNGGzoA2QIgAEECIA4gDkH/AXFBA0YbOgDYAiAAIAc2AtQCIAAgATYCLCAAIAU2AiggACAiNgIkIABBACAUIBRBAkYbNgIgIAAgIzYCHCAAQQAgEyATQQJGGzYCGCAAICQ2AhQgAEEAIBEgEUECRhs2AhAgACAlNgIMIABBACAQIBBBAkYbNgIIIAAgJjYCBCAAQQAgDyAPQQJGGzYCAAwjCyAGQZAFahCZAyAGQZAFahDvBAsCQCAGKALwBEUNACAGQfAEaiAGQRhBCBDiAgsgBkHgBGoQ+QMCQCAGKALABEUNACAGQcAEaiAGQRhBCBDgAgsCQCAGKAKgBEUNACAGQaAEaiAGQShBCBDhAgsgIEGBgICAeEchACAGQZAEahD5AyAGQYAEahCsBSAGQYAEahDnBAsCQCAGKALgA0UNACAGQeADaiAGQRhBCBDgAgsgBkHQA2oQ+QMgBkHAA2oQ+QMMHAsgBigCHCEFIAZBADYCHAJAIAVFDQAgBkGYA2ogBigCIBA5IAYoApwDIR4CQCAGKAKYAyIFDQAgAEECNgIAIAAgHjYCBEEAIRVBACEWQQAhF0EAIRhBACEAQQAhGUEAIRpBACEbQQAhHEEAIR1BACEeQQAhBEEAIR9BACEBQQAhBwwgCyANIAEpAwA3AwAgDUEQaiABQRBqKQMANwMAIA1BCGogAUEIaikDADcDACAGIB42AvwCIAYgBTYC+AIMFwtBuJLAAEExEO0EAAsgBigCHCEeIAZBADYCHAJAIB5FDQAgBkGYA2ogBigCIBCQAiAGKAKcAyEeAkAgBigCmAMiHUGBgICAeEcNACAAQQI2AgAgACAeNgIEDBoLIAYoAqADIRwCQCAGKALsAkGBgICAeEYNACAGQewCahD5AwsgBiAcNgL0AiAGIB42AvACIAYgHTYC7AIMFgtBuJLAAEExEO0EAAsgBigCHCEeIAZBADYCHAJAIB5FDQAgBkGYA2ogBigCIBDTAQJAIAYoApgDRQ0AIAYoApwDIQEgAEECNgIAIAAgATYCBAwZCyAMIAEpAwA3AwAgDEEYaiABQRhqKQMANwMAIAxBEGogAUEQaikDADcDACAMQQhqIAFBCGopAwA3AwBCASErIAZCATcDwAIMFQtBuJLAAEExEO0EAAsgBigCHCEeIAZBADYCHAJAIB5FDQAgBkGYA2ogBigCIBCQAiAGKAKcAyEeAkAgBigCmAMiHUGBgICAeEcNACAAQQI2AgAgACAeNgIEDBgLIAYoAqADIRwCQCAGKAK0AkGBgICAeEYNACAGQbQCahD5AwsgBiAcNgK8AiAGIB42ArgCIAYgHTYCtAIMFAtBuJLAAEExEO0EAAsgBigCHCEEIAZBADYCHAJAIARFDQAgBkEIaiAGKAIgEKoCQQEhBCAGKAIMIR8CQCAGKAIIQQFxRQ0AIABBAjYCACAAIB82AgQMFwsgBiAfNgKwAiAGQQE2AqwCDBMLQbiSwABBMRDtBAALIAYoAhwhHiAGQQA2AhwCQCAeRQ0AIAZBmANqIAYoAiAQwAEgBigCnAMhHgJAIAYoApgDIh1BgICAgHhHDQAgAEECNgIAIAAgHjYCBAwWCyAGKAKgAyEcAkAgBigCoAJBgICAgHhGDQAgBkGgAmoQpQMgBkGgAmoQ5gQLIAYgHDYCqAIgBiAeNgKkAiAGIB02AqACDBILQbiSwABBMRDtBAALIAYoAhwhHiAGQQA2AhwCQCAeRQ0AIAZBmANqIAYoAiAQwQEgBigCnAMhHgJAIAYoApgDIh1BgICAgHhHDQAgAEECNgIAIAAgHjYCBAwVCyAGKAKgAyEcAkAgBigClAJBgICAgHhGDQAgBkGUAmoQmQMgBkGUAmoQ7wQLIAYgHDYCnAIgBiAeNgKYAiAGIB02ApQCDBELQbiSwABBMRDtBAALIAYoAhwhHiAGQQA2AhwCQCAeRQ0AIAZBmANqIAYoAiAQ1QECQCAGKAKYA0UNACAGKAKcAyEBIABBAjYCACAAIAE2AgQMFAsgCyABKQMANwMAIAtBGGogAUEYaikDADcDACALQRBqIAFBEGopAwA3AwAgC0EIaiABQQhqKQMANwMAQgEhKiAGQgE3A+gBDBALQbiSwABBMRDtBAALIAYoAhwhHiAGQQA2AhwCQCAeRQ0AIAZBmANqIAYoAiAQkAIgBigCnAMhHgJAIAYoApgDIh1BgYCAgHhHDQAgAEECNgIAIAAgHjYCBAwTCyAGKAKgAyEcAkAgBigC3AFBgYCAgHhGDQAgBkHcAWoQ+QMLIAYgHDYC5AEgBiAeNgLgASAGIB02AtwBDA8LQbiSwABBMRDtBAALIAYoAhwhHiAGQQA2AhwCQCAeRQ0AIAZBmANqIAYoAiAQtwIgBigCnAMhIiAGKAKYAyIUQQJHDQ4gAEECNgIAIAAgIjYCBAwRC0G4ksAAQTEQ7QQACyAGKAIcIR4gBkEANgIcAkAgHkUNACAGQZgDaiAGKAIgELcCIAYoApwDISMgBigCmAMiE0ECRw0NIABBAjYCACAAICM2AgQMEAtBuJLAAEExEO0EAAsgBigCHCEeIAZBADYCHAJAIB5FDQAgBkGYA2ogBigCIBDTAQJAIAYoApgDRQ0AIAYoApwDIQEgAEECNgIAIAAgATYCBAwQCyAKIAEpAwA3AwAgCkEYaiABQRhqKQMANwMAIApBEGogAUEQaikDADcDACAKQQhqIAFBCGopAwA3AwBCASEpIAZCATcDsAEMDAtBuJLAAEExEO0EAAsgBigCHCEeIAZBADYCHAJAIB5FDQAgBkGYA2ogBigCIBDUAQJAIAYoApgDRQ0AIAYoApwDIQEgAEECNgIAIAAgATYCBAwPCyAJIAEpAwA3AwAgCUEYaiABQRhqKQMANwMAIAlBEGogAUEQaikDADcDACAJQQhqIAFBCGopAwA3AwBCASEoIAZCATcDiAEMCwtBuJLAAEExEO0EAAsgBigCHCEeIAZBADYCHAJAIB5FDQAgBkGYA2ogBigCIBCfAgJAIAYtAJgDRQ0AIAYoApwDIQEgAEECNgIAIAAgATYCBAwOCyAGLQCZAyESDAoLQbiSwABBMRDtBAALIAYoAhwhHiAGQQA2AhwCQCAeRQ0AIAZBmANqIAYoAiAQnwICQCAGLQCYA0UNACAGKAKcAyEBIABBAjYCACAAIAE2AgQMDQsgBi0AmQMhDgwJC0G4ksAAQTEQ7QQACyAGKAIcIR4gBkEANgIcAkAgHkUNACAGQZgDaiAGKAIgEJACIAYoApwDIR4CQCAGKAKYAyIdQYGAgIB4Rw0AIABBAjYCACAAIB42AgQMDAsgBigCoAMhHAJAIAYoAnxBgYCAgHhGDQAgBkH8AGoQ+QMLIAYgHDYChAEgBiAeNgKAASAGIB02AnwMCAtBuJLAAEExEO0EAAsgBigCHCEeIAZBADYCHAJAIB5FDQAgBkGYA2ogBigCIBDXASAGKAKcAyEeAkAgBigCmAMiHUGAgICAeEcNACAAQQI2AgAgACAeNgIEDAsLIAYoAqADIRwgBkHwAGoQ+QMgBiAcNgJ4IAYgHjYCdCAGIB02AnAMBwtBuJLAAEExEO0EAAsgBigCHCEeIAZBADYCHAJAIB5FDQAgBkGYA2ogBigCIBC3AiAGKAKcAyEkIAYoApgDIhFBAkcNBiAAQQI2AgAgACAkNgIEDAkLQbiSwABBMRDtBAALIAYoAhwhHiAGQQA2AhwCQCAeRQ0AIAZBmANqIAYoAiAQ0wECQCAGKAKYA0UNACAGKAKcAyEBIABBAjYCACAAIAE2AgQMCQsgCCABKQMANwMAIAhBGGogAUEYaikDADcDACAIQRBqIAFBEGopAwA3AwAgCEEIaiABQQhqKQMANwMAQgEhJyAGQgE3A0gMBQtBuJLAAEExEO0EAAsgBigCHCEeIAZBADYCHAJAIB5FDQAgBkGYA2ogBigCIBCQAiAGKAKcAyEeAkAgBigCmAMiHUGBgICAeEcNACAAQQI2AgAgACAeNgIEDAgLIAYoAqADIRwCQCAGKAI8QYGAgIB4Rg0AIAZBPGoQ+QMLIAYgHDYCRCAGIB42AkAgBiAdNgI8DAQLQbiSwABBMRDtBAALIAYoAhwhHiAGQQA2AhwCQCAeRQ0AIAZBmANqIAYoAiAQtwIgBigCnAMhJSAGKAKYAyIQQQJHDQMgAEECNgIAIAAgJTYCBAwGC0G4ksAAQTEQ7QQACyAGKAIcIR4gBkEANgIcAkAgHkUNACAGQZgDaiAGKAIgELcCIAYoApwDISYgBigCmAMiD0ECRw0CIABBAjYCACAAICY2AgQMBQtBuJLAAEExEO0EAAsgBigCHCEeIAZBADYCHCAeRQ0GIAZBmANqIAYoAiAQkAIgBigCnAMhHgJAIAYoApgDIh1BgYCAgHhHDQAgAEECNgIAIAAgHjYCBAwECyAGKAKgAyEcAkAgBigCMEGBgICAeEYNACAGQTBqEPkDCyAGIBw2AjggBiAeNgI0IAYgHTYCMAsgBkGYA2ogBkEcahCOASAGLQCYA0UNAAsLIAYoApwDIQEgAEECNgIAIAAgATYCBAwBC0EAIQcLQQAhAUEAIR9BACEEQQAhHkEAIR1BACEcQQAhG0EAIRpBACEZQQAhAEEAIRhBACEXQQAhFkEAIRULIAVFDQIMAQtBuJLAAEExEO0EAAsgBkH4AmogBkEoQQgQ4QILAkAgBw0AIAYoAuwCQYGAgIB4Rg0AIAZB7AJqEPkDCwJAIAEgBigCwAJBf3NyQQFxDQAgBigCyAJFDQAgDCAGQRhBCBDgAgsCQCAfIAYoArQCQYGAgIB4RnINACAGQbQCahD5AwsCQCAGKAKsAkF/cyAEckEBcQ0AIAZBsAJqEPoBCwJAIB4gBigCoAJBgICAgHhGcg0AIAZBoAJqEKUDIAZBoAJqEOYECwJAIB0gBigClAJBgICAgHhGcg0AIAZBlAJqEJkDIAZBlAJqEO8ECwJAIBwgBigC6AFBf3NyQQFxDQAgBigC8AFFDQAgCyAGQRhBCBDiAgsCQCAbIAYoAtwBQYGAgIB4RnINACAGQdwBahD5AwsCQCAaIAYoArABQX9zckEBcQ0AIAYoArgBRQ0AIAogBkEYQQgQ4AILAkAgGSAGKAKIAUF/c3JBAXENACAGKAKQAUUNACAJIAZBKEEIEOECCwJAIAAgBigCfEGBgICAeEZyDQAgBkH8AGoQ+QMLAkAgGCAGKAJwQYCAgIB4RnINACAGQfAAahCsBSAGQfAAahDnBAsCQCAXIAYoAkhBf3NyQQFxDQAgBigCUEUNACAIIAZBGEEIEOACCwJAIBYgBigCPEGBgICAeEZyDQAgBkE8ahD5AwsgFSAGKAIwQYGAgIB4RnINACAGQTBqEPkDCyAGQRxqEJ8DCyAGQdAFaiQAC7kvAh1/BH4jAEGgCmsiBCQAAkACQAJAAkAgASkDACIhQgBRDQACQCABKQMIIiJCAFENAAJAIAEpAxAiI0IAUQ0AAkAgISAjfCIkICFUDQACQCAhICJUDQACQCADQRBNDQAgASwAGiEFIAEuARghASAEICE+AgAgBEEBQQIgIUKAgICAEFQiBhs2AqABIARBACAhQiCIpyAGGzYCBAJAQZgBRSIGDQAgBEEIakEAQZgB/AsACyAEICI+AqQBIARBAUECICJCgICAgBBUIgcbNgLEAiAEQQAgIkIgiKcgBxs2AqgBAkAgBg0AIARBpAFqQQhqQQBBmAH8CwALIAQgIz4CyAIgBEEBQQIgI0KAgICAEFQiBxs2AugDIARBACAjQiCIpyAHGzYCzAICQCAGDQAgBEHIAmpBCGpBAEGYAfwLAAsCQEGcAUUNACAEQfADakEAQZwB/AsACyAEQQE2AuwDIARBATYCjAUgAawgJEJ/fHl9QsKawegEfkKAoc2gtAJ8QiCIpyIGwSEIAkACQCABQQBIDQAgBCABEEEaIARBpAFqIAEQQRogBEHIAmogARBBGgwBCyAEQewDakEAIAFrwRBBGgsCQAJAIAhBf0oNACAEQQAgCGtB//8DcSIBED4aIARBpAFqIAEQPhogBEHIAmogARA+GgwBCyAEQewDaiAGQf//AXEQPhoLAkBBpAFFDQAgBEH8CGogBEGkAfwKAAALAkACQAJAAkACQCAEKALoAyIJIAQoApwKIgEgCSABSxsiCkEoSw0AAkAgCg0AQQAhCgwECyAKQQFxIQsgCkEBRw0BQQAhDEEAIQ0MAgsgCkEoQZT8wAAQzwQACyAKQT5xIQ5BACEMIARB/AhqIQEgBEHIAmohBkEAIQ0DQCABIAEoAgAiDyAGKAIAaiIHIAxBAXFqIhA2AgAgAUEEaiIMIAwoAgAiESAGQQRqKAIAaiIMIAcgD0kgECAHSXJqIgc2AgAgDCARSSAHIAxJciEMIAZBCGohBiABQQhqIQEgDiANQQJqIg1HDQALCwJAIAtFDQAgBEH8CGogDUECdCIBaiIGIAYoAgAiBiAEQcgCaiABaigCAGoiASAMaiIHNgIAIAEgBkkgByABSXIhDAsgDEEBcUUNACAKQShGDQEgBEH8CGogCkECdGpBATYCACAKQQFqIQoLIAQgCjYCnAoCQCAKIAQoAowFIgEgCiABSxsiAUEpTw0AIAFBAnQhAQJAAkADQCABRQ0BIAFBfGoiASAEQewDamooAgAiBiABIARB/AhqaigCACIHRg0ACyAGIAdLIAYgB0lrIQEMAQtBf0EAIAEbIQELAkACQAJAAkACQAJAAkAgASAFSA0AIAQoAqABIg1BKU8NBgJAAkAgDQ0AQQAhDQwBCyANQX9qQf////8DcSIBQQFqIgdBA3EhBgJAAkAgAUEDTw0AIAQhAUIAISIMAQsgB0H8////B3EhByAEIQFCACEiA0AgASABNQIAQgp+ICJ8IiE+AgAgAUEEaiIMIAw1AgBCCn4gIUIgiHwiIT4CACABQQhqIgwgDDUCAEIKfiAhQiCIfCIhPgIAIAFBDGoiDCAMNQIAQgp+ICFCIIh8IiE+AgAgIUIgiCEiIAFBEGohASAHQXxqIgcNAAsLAkAgBkUNAANAIAEgATUCAEIKfiAifCIhPgIAIAFBBGohASAhQiCIISIgBkF/aiIGDQALCyAhQoCAgIAQVA0AIA1BKEYNBiAEIA1BAnRqICKnNgIAIA1BAWohDQsgBCANNgKgASAEKALEAiINQSlPDQRBACEPQQAhAQJAIA1FDQAgDUF/akH/////A3EiAUEBaiIHQQNxIQYCQAJAIAFBA08NACAEQaQBaiEBQgAhIQwBCyAHQfz///8HcSEHIARBpAFqIQFCACEhA0AgASABNQIAQgp+ICF8IiE+AgAgAUEEaiIMIAw1AgBCCn4gIUIgiHwiIT4CACABQQhqIgwgDDUCAEIKfiAhQiCIfCIhPgIAIAFBDGoiDCAMNQIAQgp+ICFCIIh8IiI+AgAgIkIgiCEhIAFBEGohASAHQXxqIgcNAAsLAkAgBkUNAANAIAEgATUCAEIKfiAhfCIiPgIAIAFBBGohASAiQiCIISEgBkF/aiIGDQALCwJAICJCgICAgBBaDQAgDSEBDAELIA1BKEYNBCAEQaQBaiANQQJ0aiAhpzYCACANQQFqIQELIAQgATYCxAICQCAJRQ0AIAlBf2pB/////wNxIgFBAWoiB0EDcSEGAkACQCABQQNPDQAgBEHIAmohAUIAISEMAQsgB0H8////B3EhByAEQcgCaiEBQgAhIQNAIAEgATUCAEIKfiAhfCIhPgIAIAFBBGoiDCAMNQIAQgp+ICFCIIh8IiE+AgAgAUEIaiIMIAw1AgBCCn4gIUIgiHwiIT4CACABQQxqIgwgDDUCAEIKfiAhQiCIfCIiPgIAICJCIIghISABQRBqIQEgB0F8aiIHDQALCwJAIAZFDQADQCABIAE1AgBCCn4gIXwiIj4CACABQQRqIQEgIkIgiCEhIAZBf2oiBg0ACwsCQCAiQoCAgIAQWg0AIAQgCTYC6AMMAwsgCUEoRg0DIARByAJqIAlBAnRqICGnNgIAIAlBAWohDwsgBCAPNgLoAwwBCyAIQQFqIQgLAkBBpAFFIgENACAEQZAFaiAEQewDakGkAfwKAAALIARBkAVqQQEQQSESAkAgAQ0AIARBtAZqIARB7ANqQaQB/AoAAAsgBEG0BmpBAhBBIRMCQCABDQAgBEHYB2ogBEHsA2pBpAH8CgAACwJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQCAEQdgHakEDEEEiFCgCoAEiFSAEKAKgASINIBUgDUsbIgtBKEsNACAEQZAFakF8aiEJIARBtAZqQXxqIQogBEHYB2pBfGohDiASKAKgASEWIBMoAqABIRcgBCgCjAUhGEEAIRkDQCAZIRogC0ECdCEBAkACQAJAAkADQCABRQ0BIA4gAWohBiABQXxqIgEgBGooAgAiByAGKAIAIgZGDQALIAcgBkkNAQwCCyABRQ0BC0EAIRsgDSELDAELAkAgC0UNAEEBIQwgC0EBcSEbQQAhDQJAIAtBAUYNACALQT5xIRxBACENQQEhDCAEIQEgBEHYB2ohBgNAIAEgASgCACIPIAYoAgBBf3NqIgcgDEEBcWoiEDYCACABQQRqIgwgDCgCACIRIAZBBGooAgBBf3NqIgwgByAPSSAQIAdJcmoiBzYCACAMIBFJIAcgDElyIQwgBkEIaiEGIAFBCGohASAcIA1BAmoiDUcNAAsLAkAgG0UNACAEIA1BAnQiAWoiBiAGKAIAIgYgFCABaigCAEF/c2oiASAMaiIHNgIAIAEgBkkgByABSXIhDAsgDEEBcUUNBQsgBCALNgKgAUEIIRsLIBcgCyAXIAtLGyIcQSlPDQQgHEECdCEBAkACQAJAA0AgAUUNASAKIAFqIQYgAUF8aiIBIARqKAIAIgcgBigCACIGRg0ACyAHIAZPDQEgCyEcDAILIAFFDQAgCyEcDAELAkAgHEUNAEEBIQwgHEEBcSEdQQAhDQJAIBxBAUYNACAcQT5xIQtBACENQQEhDCAEIQEgBEG0BmohBgNAIAEgASgCACIPIAYoAgBBf3NqIgcgDEEBcWoiEDYCACABQQRqIgwgDCgCACIRIAZBBGooAgBBf3NqIgwgByAPSSAQIAdJcmoiBzYCACAMIBFJIAcgDElyIQwgBkEIaiEGIAFBCGohASALIA1BAmoiDUcNAAsLAkAgHUUNACAEIA1BAnQiAWoiBiAGKAIAIgYgEyABaigCAEF/c2oiASAMaiIHNgIAIAEgBkkgByABSXIhDAsgDEEBcUUNBwsgBCAcNgKgASAbQQRyIRsLIBYgHCAWIBxLGyILQSlPDQYgC0ECdCEBAkACQAJAA0AgAUUNASAJIAFqIQYgAUF8aiIBIARqKAIAIgcgBigCACIGRg0ACyAHIAZPDQEgHCELDAILIAFFDQAgHCELDAELAkAgC0UNAEEBIQwgC0EBcSEdQQAhDQJAIAtBAUYNACALQT5xIRxBACENQQEhDCAEIQEgBEGQBWohBgNAIAEgASgCACIPIAYoAgBBf3NqIgcgDEEBcWoiEDYCACABQQRqIgwgDCgCACIRIAZBBGooAgBBf3NqIgwgByAPSSAQIAdJcmoiBzYCACAMIBFJIAcgDElyIQwgBkEIaiEGIAFBCGohASAcIA1BAmoiDUcNAAsLAkAgHUUNACAEIA1BAnQiAWoiBiAGKAIAIgYgEiABaigCAEF/c2oiASAMaiIHNgIAIAEgBkkgByABSXIhDAsgDEEBcUUNCQsgBCALNgKgASAbQQJqIRsLIBggCyAYIAtLGyIcQSlPDQggHEECdCEBAkACQAJAA0AgAUUNASABQXxqIgEgBGooAgAiBiABIARB7ANqaigCACIHRg0ACyAGIAdPDQEgCyEcDAILIAFFDQAgCyEcDAELAkAgHEUNAEEBIQwgHEEBcSEdQQAhDQJAIBxBAUYNACAcQT5xIQtBACENQQEhDCAEIQEgBEHsA2ohBgNAIAEgASgCACIPIAYoAgBBf3NqIgcgDEEBcWoiEDYCACABQQRqIgwgDCgCACIRIAZBBGooAgBBf3NqIgwgByAPSSAQIAdJcmoiBzYCACAMIBFJIAcgDElyIQwgBkEIaiEGIAFBCGohASALIA1BAmoiDUcNAAsLAkAgHUUNACAEIA1BAnQiAWoiBiAGKAIAIgYgBEHsA2ogAWooAgBBf3NqIgEgDGoiBzYCACABIAZJIAcgAUlyIQwLIAxBAXFFDQsLIAQgHDYCoAEgG0EBaiEbCyAaIANGDQ4gAiAaaiAbQTBqOgAAIAQoAsQCIh4gHCAeIBxLGyIBQSlPDQogGkEBaiEZIAFBAnQhAQJAAkADQCABRQ0BIAFBfGoiASAEaigCACIGIAEgBEGkAWpqKAIAIgdGDQALIAYgB0sgBiAHSWshHwwBC0F/QQAgARshHwsCQEGkAUUNACAEQfwIaiAEQaQB/AoAAAsgBCgC6AMiHSAEKAKcCiIBIB0gAUsbIhtBKEsNCwJAAkAgGw0AQQAhGwwBCyAbQQFxISBBACEMQQAhDQJAIBtBAUYNACAbQT5xIQtBACEMIARB/AhqIQEgBEHIAmohBkEAIQ0DQCABIAEoAgAiDyAGKAIAaiIHIAxBAXFqIhA2AgAgAUEEaiIMIAwoAgAiESAGQQRqKAIAaiIMIAcgD0kgECAHSXJqIgc2AgAgDCARSSAHIAxJciEMIAZBCGohBiABQQhqIQEgCyANQQJqIg1HDQALCwJAICBFDQAgBEH8CGogDUECdCIBaiIGIAYoAgAiBiAEQcgCaiABaigCAGoiASAMaiIHNgIAIAEgBkkgByABSXIhDAsgDEEBcUUNACAbQShGDQ0gBEH8CGogG0ECdGpBATYCACAbQQFqIRsLIAQgGzYCnAogGyAYIBsgGEsbIgFBKU8NDSABQQJ0IQECQAJAA0AgAUUNASABQXxqIgEgBEHsA2pqKAIAIgYgASAEQfwIamooAgAiB0YNAAsgBiAHSyAGIAdJayEBDAELQX9BACABGyEBCwJAAkAgHyAFSCIGDQAgASAFTg0BCyABIAVODSMgBg0DDCILQQAhD0EAIQ0CQCAcRQ0AIBxBf2pB/////wNxIgFBAWoiB0EDcSEGAkACQCABQQNPDQAgBCEBQgAhIQwBCyAHQfz///8HcSEHIAQhAUIAISEDQCABIAE1AgBCCn4gIXwiIT4CACABQQRqIgwgDDUCAEIKfiAhQiCIfCIhPgIAIAFBCGoiDCAMNQIAQgp+ICFCIIh8IiE+AgAgAUEMaiIMIAw1AgBCCn4gIUIgiHwiIj4CACAiQiCIISEgAUEQaiEBIAdBfGoiBw0ACwsCQCAGRQ0AA0AgASABNQIAQgp+ICF8IiI+AgAgAUEEaiEBICJCIIghISAGQX9qIgYNAAsLAkAgIkKAgICAEFoNACAcIQ0MAQsgHEEoRg0QIAQgHEECdGogIac2AgAgHEEBaiENCyAEIA02AqABAkAgHkUNACAeQX9qQf////8DcSIBQQFqIgdBA3EhBgJAAkAgAUEDTw0AIARBpAFqIQFCACEiDAELIAdB/P///wdxIQcgBEGkAWohAUIAISIDQCABIAE1AgBCCn4gInwiIT4CACABQQRqIgwgDDUCAEIKfiAhQiCIfCIhPgIAIAFBCGoiDCAMNQIAQgp+ICFCIIh8IiE+AgAgAUEMaiIMIAw1AgBCCn4gIUIgiHwiIT4CACAhQiCIISIgAUEQaiEBIAdBfGoiBw0ACwsCQCAGRQ0AA0AgASABNQIAQgp+ICJ8IiE+AgAgAUEEaiEBICFCIIghIiAGQX9qIgYNAAsLAkAgIUKAgICAEFoNACAeIQ8MAQsgHkEoRg0RIARBpAFqIB5BAnRqICKnNgIAIB5BAWohDwsgBCAPNgLEAgJAAkAgHQ0AQQAhHQwBCyAdQX9qQf////8DcSIBQQFqIgdBA3EhBgJAAkAgAUEDTw0AIARByAJqIQFCACEhDAELIAdB/P///wdxIQcgBEHIAmohAUIAISEDQCABIAE1AgBCCn4gIXwiIT4CACABQQRqIgwgDDUCAEIKfiAhQiCIfCIhPgIAIAFBCGoiDCAMNQIAQgp+ICFCIIh8IiE+AgAgAUEMaiIMIAw1AgBCCn4gIUIgiHwiIj4CACAiQiCIISEgAUEQaiEBIAdBfGoiBw0ACwsCQCAGRQ0AA0AgASABNQIAQgp+ICF8IiI+AgAgAUEEaiEBICJCIIghISAGQX9qIgYNAAsLICJCgICAgBBUDQAgHUEoRg0SIARByAJqIB1BAnRqICGnNgIAIB1BAWohHQsgBCAdNgLoAyAVIA0gFSANSxsiC0EoTQ0ACwsgC0EoQZT8wAAQzwQACyAEQQEQQRogBCgCjAUiASAEKAKgASIGIAEgBksbIgFBKU8NDyABQQJ0IQEgBEF8aiEMIARB7ANqQXxqIQ0DQCABRQ0eIA0gAWohBiAMIAFqIQcgAUF8aiEBIAcoAgAiByAGKAIAIgZGDQALIAcgBk8NHgwfC0Gk/MAAQRpBlPzAABD/AgALIBxBKEGU/MAAEM8EAAtBpPzAAEEaQZT8wAAQ/wIACyALQShBlPzAABDPBAALQaT8wABBGkGU/MAAEP8CAAsgHEEoQZT8wAAQzwQAC0Gk/MAAQRpBlPzAABD/AgALIAFBKEGU/MAAEM8EAAsgG0EoQZT8wAAQzwQAC0EoQShBlPzAABCuAgALIAFBKEGU/MAAEM8EAAsgAyADQdzVwAAQrgIAC0EoQShBlPzAABCuAgALQShBKEGU/MAAEK4CAAtBKEEoQZT8wAAQrgIACyABQShBlPzAABDPBAALQShBKEGU/MAAEK4CAAtBKEEoQZT8wAAQrgIACyANQShBlPzAABDPBAALQShBKEGU/MAAEK4CAAsgDUEoQZT8wAAQzwQACyABQShBlPzAABDPBAALQShBKEGU/MAAEK4CAAtBnNXAAEEtQczVwAAQ/wIAC0GM1sAAQTdBxNbAABD/AgALQdTWwABBNkGM18AAEP8CAAtB8NTAAEEcQYzVwAAQ/wIAC0HA1MAAQR1B4NTAABD/AgALQZPUwABBHEGw1MAAEP8CAAsgAQ0BCyACIBlqIQ0gGiEBQX8hBgJAA0AgAUF/Rg0BIAZBAWohBiACIAFqIQcgAUF/aiIMIQEgBy0AAEE5Rg0ACyACIAxqIgdBAWoiASABLQAAQQFqOgAAIAxBAmogGksNASAGRQ0BIAdBAmpBMCAG/AsADAELIAJBMToAAAJAIBpFDQAgGkUNACACQQFqQTAgGvwLAAsCQCAZIANPDQAgDUEwOgAAIAhBAWohCCAaQQJqIRkMAQsgGSADQezVwAAQrgIACwJAIBkgA0sNACAAIAg7AQggACAZNgIEIAAgAjYCACAEQaAKaiQADwsgGSADQfzVwAAQzwQAC904AhF/AX4jAEGQA2siBiQAIAYgATYCFAJAAkAgARCMBUEBRg0AIAZBFGogBkHgAWpBqILAABBhIQUgAEGAgICAeDYCnAEgACAFNgIAIAFBhAFJDQEgARCPAwwBCyAGQRhqIAEgBCAFELkDIAZBgYCAgHg2AixBACEEIAZBADYCOCAGQYCAgIB4NgJYIAZBgYCAgHg2AmQgBkGBgICAeDYCcCAGQYCAgIB4NgJ8IAZBgICAgHg2AogBIAZBgYCAgHg2ApQBIAZBgICAgHg2AqABIAZBgYCAgHg2AqwBIAZBgYCAgHg2ArgBIAZBgYCAgHg2AsQBIAYoAjwhBSAGQeABaiAGQRhqEI8BAkACQAJAAkACQCAGLQDgAUUNAEEAIQEMAQsgBkE4akEIaiEHIAZB4AFqQQhqIQhBAiEJQQIhCkECIQtBAiEMQQIhDUECIQ5BACEPQQAhAQJAA0ACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQCAGLQDhAQ4VAQIDBAUGBwgJCgsMDQ4PEBESEwAUAQsgBkEIaiAGQRhqEIIDDCkLIAYoAixBgYCAgHhGDSUgBiAFNgI8IAYgATYCOEGQhcAAQQUQzAIhASAAQYCAgIB4NgKcASAAIAE2AgAMJgsgAUUNIyAGIAU2AjwgBiABNgI4QZWFwABBBhDMAiEBIABBgICAgHg2ApwBIAAgATYCAAwlCyAGKAJYQYCAgIB4Rg0hIAYgBTYCPCAGIAE2AjhBm4XAAEEIEMwCIQEgAEGAgICAeDYCnAEgACABNgIADCQLIAYoAmRBgYCAgHhGDR8gBiAFNgI8IAYgATYCOEHHg8AAQQsQzAIhASAAQYCAgIB4NgKcASAAIAE2AgAMIwsgCUH/AXFBAkYNHSAGIAU2AjwgBiABNgI4QaOFwABBChDMAiEBIABBgICAgHg2ApwBIAAgATYCAAwiCyAGKAJwQYGAgIB4Rg0bIAYgBTYCPCAGIAE2AjhBrYXAAEEJEMwCIQEgAEGAgICAeDYCnAEgACABNgIADCELIAYoAnxBgICAgHhGDRkgBiAFNgI8IAYgATYCOEG2hcAAQQgQzAIhASAAQYCAgIB4NgKcASAAIAE2AgAMIAsgCkH/AXFBAkYNFyAGIAU2AjwgBiABNgI4Qb6FwABBDhDMAiEBIABBgICAgHg2ApwBIAAgATYCAAwfCyALQf8BcUECRg0VIAYgBTYCPCAGIAE2AjhBzIXAAEEMEMwCIQEgAEGAgICAeDYCnAEgACABNgIADB4LIAxB/wFxQQJGDRMgBiAFNgI8IAYgATYCOEHYhcAAQQoQzAIhASAAQYCAgIB4NgKcASAAIAE2AgAMHQsgDUH/AXFBAkYNESAGIAU2AjwgBiABNgI4QeKFwABBDBDMAiEBIABBgICAgHg2ApwBIAAgATYCAAwcCyAOQf8BcUECRg0PIAYgBTYCPCAGIAE2AjhB7oXAAEEJEMwCIQEgAEGAgICAeDYCnAEgACABNgIADBsLIAYoAogBQYCAgIB4Rg0NIAYgBTYCPCAGIAE2AjhBiYTAAEEEEMwCIQEgAEGAgICAeDYCnAEgACABNgIADBoLIAYoApQBQYGAgIB4Rg0LIAYgBTYCPCAGIAE2AjhB94XAAEEMEMwCIQEgAEGAgICAeDYCnAEgACABNgIADBkLIAYoAqABQYCAgIB4Rg0JIAYgBTYCPCAGIAE2AjhBg4bAAEEGEMwCIQEgAEGAgICAeDYCnAEgACABNgIADBgLIAYoAqwBQYGAgIB4Rg0HIAYgBTYCPCAGIAE2AjhBiYbAAEENEMwCIQEgAEGAgICAeDYCnAEgACABNgIADBcLIAYoArgBQYGAgIB4Rg0FIAYgBTYCPCAGIAE2AjhBlobAAEEOEMwCIQEgAEGAgICAeDYCnAEgACABNgIADBYLIA9FDQMgBiAFNgI8IAYgATYCOEGkhsAAQQkQzAIhASAAQYCAgIB4NgKcASAAIAE2AgAMFQsgBigCxAFBgYCAgHhGDQEgBiAFNgI8IAYgATYCOEGthsAAQRoQzAIhASAAQYCAgIB4NgKcASAAIAE2AgAMFAsgBiAFNgI8IAYgATYCOAJAAkAgBigCLEGBgICAeEciEA0AIAZBgICAgHg2AtABDAELIAZB0AFqQQhqIAZBLGpBCGooAgA2AgAgBiAGKQIsNwPQAQsCQAJAIAENAEGVhcAAQQYQywIhBSAAQYCAgIB4NgKcASAAIAU2AgBBACEEQQAhD0EAIQVBACEHQQAhCEEAIRFBACEAQQAhEkEAIRMMAQsgBkHgAWpBGGogBkE4akEYaikDADcDACAGQeABakEQaiAGQThqQRBqKQMANwMAIAZB4AFqQQhqIAZBOGpBCGopAwA3AwAgBiAGKQM4NwPgAQJAAkAgBigCWEGAgICAeEciEw0AQZuFwABBCBDLAiEFIABBgICAgHg2ApwBIAAgBTYCAEEAIQRBACEPQQAhBUEAIQdBACEIQQAhEUEAIQBBACESDAELIAZBgAJqQQhqIAZB2ABqQQhqKAIANgIAIAYgBikCWDcDgAICQAJAIAYoAmRBgYCAgHhHIhINACAGQYCAgIB4NgKQAgwBCyAGQZACakEIaiAGQeQAakEIaigCADYCACAGIAYpAmQ3A5ACCwJAAkAgCUH/AXFBAkcNAEGjhcAAQQoQywIhBSAAQYCAgIB4NgKcASAAIAU2AgBBACEEQQAhD0EAIQVBACEHQQAhCEEAIRFBACEADAELAkACQCAGKAJwIhRBgYCAgHhGDQAgBkGgAmpBCGogBkHwAGpBCGooAgA2AgAgBiAGKQJwNwOgAgwBCyAGQYCAgIB4NgKgAgsCQAJAIAYoAnxBgICAgHhHIhENAEG2hcAAQQgQywIhBSAAQYCAgIB4NgKcASAAIAU2AgBBACEEQQAhD0EAIQVBACEHQQAhCAwBCyAGQbACakEIaiAGQfwAakEIaigCADYCACAGIAYpAnw3A7ACAkACQAJAIApB/wFxQQJHDQBBvoXAAEEOEMsCIQUgAEGAgICAeDYCnAEgACAFNgIADAELAkAgC0H/AXFBAkYNAAJAIAxB/wFxQQJHDQBB2IXAAEEKEMsCIQUgAEGAgICAeDYCnAEgACAFNgIADAILAkAgDUH/AXFBAkcNAEHihcAAQQwQywIhBSAAQYCAgIB4NgKcASAAIAU2AgAMAgsCQCAOQf8BcUECRw0AQe6FwABBCRDLAiEFIABBgICAgHg2ApwBIAAgBTYCAAwCCwJAIAYoAogBQYCAgIB4RyIIDQBBiYTAAEEEEMsCIQUgAEGAgICAeDYCnAEgACAFNgIAQQAhBEEAIQ9BACEFQQAhBwwDCyAGQcACakEIaiAGQYgBakEIaigCADYCACAGIAYpAogBNwPAAgJAAkAgBigClAFBgYCAgHhHIgdFDQAgBkHQAmpBCGogBkGUAWpBCGooAgA2AgAgBiAGKQKUATcD0AIMAQsgBkGAgICAeDYC0AILAkACQCAGKAKgAUGAgICAeEciBQ0AQYOGwABBBhDLAiEEIABBgICAgHg2ApwBIAAgBDYCAEEAIQRBACEPDAELIAZB4AJqQQhqIAZBoAFqQQhqKAIANgIAIAYgBikCoAE3A+ACAkACQCAGKAKsASIEQYGAgIB4Rg0AIAZB8AJqQQhqIAZBrAFqQQhqKAIANgIAIAYgBikCrAE3A/ACDAELIAZBgICAgHg2AvACCwJAAkAgBigCuAEiFUGBgICAeEYNACAGQYADakEIaiAGQbgBakEIaigCADYCACAGIAYpArgBNwOAAwwBCyAGQYCAgIB4NgKAAwsCQCAPQQFxDQAgBEGBgICAeEchDyAVQYGAgIB4RyEEQaSGwABBCRDLAiEJIABBgICAgHg2ApwBIAAgCTYCACAGQYADahD5AyAGQfACahD5AyAGQeACahCsBSAGQeACahDnBAwBCyAAIAYpA9ABNwIkIAAgBikDODcDACAAIAYpAlg3AnggAEEsaiAGQdABakEIaigCADYCACAAQQhqIAZBOGpBCGopAwA3AwAgAEEQaiAGQThqQRBqKQMANwMAIABBGGogBkE4akEYaikDADcDACAAQYABaiAGQdgAakEIaigCADYCACAGKALEASEBIAYpAsgBIRcgAEE4aiAGQZACakEIaigCADYCACAAIAYpA5ACNwIwIAAgBikDoAI3AjwgAEHEAGogBkGgAmpBCGooAgA2AgAgACAGKQJ8NwKEASAAQYwBaiAGQfwAakEIaigCADYCACAAIAYpAogBNwKQASAAQZgBaiAGQYgBakEIaigCADYCACAAQdAAaiAGQdACakEIaigCADYCACAAIAYpA9ACNwJIIAAgDkEBcToArQEgACANQQFxOgCsASAAIAxBAXE6AKsBIAAgC0EBcToAqgEgACAKQQFxOgCpASAAIAlBAXE6AKgBIAAgFzcDcCAAQYCAgIB4IAEgAUGBgICAeEYbNgJsIAAgFjYCICAAQaQBaiAGQaABakEIaigCADYCACAAIAYpAqABNwKcASAAQdwAaiAGQfACakEIaigCADYCACAAIAYpA/ACNwJUIABB6ABqIAZBgANqQQhqKAIANgIAIAAgBikDgAM3AmAMIwsgBkHQAmoQ+QMgBkHAAmoQrAUgBkHAAmoQ5wQMAgtBzIXAAEEMEMsCIQUgAEGAgICAeDYCnAEgACAFNgIAC0EAIQRBACEPQQAhBUEAIQdBACEICyAGQbACahCsBSAGQbACahDnBAsgFEGBgICAeEchACAGQaACahD5AwsgBkGQAmoQ+QMgBkGAAmoQrAUgBkGAAmoQ5wQLIAZB4AFqIAZBKEEIEOECCyABQQBHIQEgBkHQAWoQ+QMMGgsgBigCGCERIAZBADYCGAJAIBFFDQAgBkHgAWogBigCHBCQAiAGKALkASERAkAgBigC4AEiEkGBgICAeEcNACAGIAU2AjwgBiABNgI4IABBgICAgHg2ApwBIAAgETYCAAwUCyAGKALoASETAkAgBigCxAFBgYCAgHhGDQAgBkHEAWoQ+QMLIAYgEzYCzAEgBiARNgLIASAGIBI2AsQBDBULIAYgBTYCPCAGIAE2AjhBuJLAAEExEO0EAAsgBigCGCERIAZBADYCGAJAIBFFDQAgBiAGKAIcEN4CQQEhDyAGKAIEIRYgBigCAEEBcUUNFCAAQYCAgIB4NgKcASAAIBY2AgAgBiAFNgI8IAYgATYCOAwSCyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYoAhghESAGQQA2AhgCQCARRQ0AIAZB4AFqIAYoAhwQkAIgBigC5AEhEQJAIAYoAuABIhJBgYCAgHhHDQAgBiAFNgI8IAYgATYCOCAAQYCAgIB4NgKcASAAIBE2AgAMEgsgBigC6AEhEwJAIAYoArgBQYGAgIB4Rg0AIAZBuAFqEPkDCyAGIBM2AsABIAYgETYCvAEgBiASNgK4AQwTCyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYoAhghESAGQQA2AhgCQCARRQ0AIAZB4AFqIAYoAhwQkAIgBigC5AEhEQJAIAYoAuABIhJBgYCAgHhHDQAgBiAFNgI8IAYgATYCOCAAQYCAgIB4NgKcASAAIBE2AgAMEQsgBigC6AEhEwJAIAYoAqwBQYGAgIB4Rg0AIAZBrAFqEPkDCyAGIBM2ArQBIAYgETYCsAEgBiASNgKsAQwSCyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYoAhghESAGQQA2AhgCQCARRQ0AIAZB4AFqIAYoAhwQ1wEgBigC5AEhEQJAIAYoAuABIhJBgICAgHhHDQAgBiAFNgI8IAYgATYCOCAAQYCAgIB4NgKcASAAIBE2AgAMEAsgBigC6AEhEyAGQaABahD5AyAGIBM2AqgBIAYgETYCpAEgBiASNgKgAQwRCyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYoAhghESAGQQA2AhgCQCARRQ0AIAZB4AFqIAYoAhwQkAIgBigC5AEhEQJAIAYoAuABIhJBgYCAgHhHDQAgBiAFNgI8IAYgATYCOCAAQYCAgIB4NgKcASAAIBE2AgAMDwsgBigC6AEhEwJAIAYoApQBQYGAgIB4Rg0AIAZBlAFqEPkDCyAGIBM2ApwBIAYgETYCmAEgBiASNgKUAQwQCyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYoAhghESAGQQA2AhgCQCARRQ0AIAZB4AFqIAYoAhwQ1wEgBigC5AEhEQJAIAYoAuABIhJBgICAgHhHDQAgBiAFNgI8IAYgATYCOCAAQYCAgIB4NgKcASAAIBE2AgAMDgsgBigC6AEhEyAGQYgBahD5AyAGIBM2ApABIAYgETYCjAEgBiASNgKIAQwPCyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYoAhghESAGQQA2AhgCQCARRQ0AIAZB4AFqIAYoAhwQngICQCAGLQDgAUUNACAGIAU2AjwgBiABNgI4IAYoAuQBIQEgAEGAgICAeDYCnAEgACABNgIADA0LIAYtAOEBIQ4MDgsgBiAFNgI8IAYgATYCOEG4ksAAQTEQ7QQACyAGKAIYIREgBkEANgIYAkAgEUUNACAGQeABaiAGKAIcEJ4CAkAgBi0A4AFFDQAgBiAFNgI8IAYgATYCOCAGKALkASEBIABBgICAgHg2ApwBIAAgATYCAAwMCyAGLQDhASENDA0LIAYgBTYCPCAGIAE2AjhBuJLAAEExEO0EAAsgBigCGCERIAZBADYCGAJAIBFFDQAgBkHgAWogBigCHBCeAgJAIAYtAOABRQ0AIAYgBTYCPCAGIAE2AjggBigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgAMCwsgBi0A4QEhDAwMCyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYoAhghESAGQQA2AhgCQCARRQ0AIAZB4AFqIAYoAhwQngICQCAGLQDgAUUNACAGIAU2AjwgBiABNgI4IAYoAuQBIQEgAEGAgICAeDYCnAEgACABNgIADAoLIAYtAOEBIQsMCwsgBiAFNgI8IAYgATYCOEG4ksAAQTEQ7QQACyAGKAIYIREgBkEANgIYAkAgEUUNACAGQeABaiAGKAIcEJ4CAkAgBi0A4AFFDQAgBiAFNgI8IAYgATYCOCAGKALkASEBIABBgICAgHg2ApwBIAAgATYCAAwJCyAGLQDhASEKDAoLIAYgBTYCPCAGIAE2AjhBuJLAAEExEO0EAAsgBigCGCERIAZBADYCGAJAIBFFDQAgBkHgAWogBigCHBDXASAGKALkASERAkAgBigC4AEiEkGAgICAeEcNACAGIAU2AjwgBiABNgI4IABBgICAgHg2ApwBIAAgETYCAAwICyAGKALoASETIAZB/ABqEPkDIAYgEzYChAEgBiARNgKAASAGIBI2AnwMCQsgBiAFNgI8IAYgATYCOEG4ksAAQTEQ7QQACyAGKAIYIREgBkEANgIYAkAgEUUNACAGQeABaiAGKAIcEJACIAYoAuQBIRECQCAGKALgASISQYGAgIB4Rw0AIAYgBTYCPCAGIAE2AjggAEGAgICAeDYCnAEgACARNgIADAcLIAYoAugBIRMCQCAGKAJwQYGAgIB4Rg0AIAZB8ABqEPkDCyAGIBM2AnggBiARNgJ0IAYgEjYCcAwICyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYoAhghESAGQQA2AhgCQCARRQ0AIAZB4AFqIAYoAhwQngICQCAGLQDgAUUNACAGIAU2AjwgBiABNgI4IAYoAuQBIQEgAEGAgICAeDYCnAEgACABNgIADAYLIAYtAOEBIQkMBwsgBiAFNgI8IAYgATYCOEG4ksAAQTEQ7QQACyAGKAIYIREgBkEANgIYAkAgEUUNACAGQeABaiAGKAIcEJACIAYoAuQBIRECQCAGKALgASISQYGAgIB4Rw0AIAYgBTYCPCAGIAE2AjggAEGAgICAeDYCnAEgACARNgIADAULIAYoAugBIRMCQCAGKAJkQYGAgIB4Rg0AIAZB5ABqEPkDCyAGIBM2AmwgBiARNgJoIAYgEjYCZAwGCyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYoAhghESAGQQA2AhgCQCARRQ0AIAZB4AFqIAYoAhwQ1wEgBigC5AEhEQJAIAYoAuABIhJBgICAgHhHDQAgBiAFNgI8IAYgATYCOCAAQYCAgIB4NgKcASAAIBE2AgAMBAsgBigC6AEhEyAGQdgAahD5AyAGIBM2AmAgBiARNgJcIAYgEjYCWAwFCyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYoAhghASAGQQA2AhgCQCABRQ0AIAZB4AFqIAYoAhwQOSAGKALkASERAkAgBigC4AEiAQ0AIAYgBTYCPEEAIQQgBkEANgI4IABBgICAgHg2ApwBIAAgETYCAAwJCyAHIAgpAwA3AwAgB0EQaiAIQRBqKQMANwMAIAdBCGogCEEIaikDADcDACARIQUMBAsgBiAFNgI8IAZBADYCOEG4ksAAQTEQ7QQACyAGKAIYIREgBkEANgIYIBFFDQQgBkHgAWogBigCHBCQAiAGKALkASERIAYoAuABIhJBgYCAgHhHDQEgBiAFNgI8IAYgATYCOCAAQYCAgIB4NgKcASAAIBE2AgALQQAhBAwFCyAGKALoASETAkAgBigCLEGBgICAeEYNACAGQSxqEPkDCyAGIBM2AjQgBiARNgIwIAYgEjYCLAsgBkHgAWogBkEYahCPASAGLQDgAQ0CDAALCyAGIAU2AjwgBiABNgI4QbiSwABBMRDtBAALIAYgBTYCPCAGIAE2AjggBigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgALQQAhD0EAIQVBACEHQQAhCEEAIRFBACEAQQAhEkEAIRNBACEBQQAhEAsCQCAGKALEAUGBgICAeEYNACAGQcQBahD5AwsCQCAGKAK4AUGBgICAeEYgBHJBAXENACAGQbgBahD5AwsCQCAGKAKsAUGBgICAeEYgD3JBAXENACAGQawBahD5AwsCQCAFIAYoAqABQYCAgIB4RnINACAGQaABahCsBSAGQaABahDnBAsCQCAHIAYoApQBQYGAgIB4RnINACAGQZQBahD5AwsCQCAIIAYoAogBQYCAgIB4RnINACAGQYgBahCsBSAGQYgBahDnBAsCQCARIAYoAnxBgICAgHhGcg0AIAZB/ABqEKwFIAZB/ABqEOcECwJAIAAgBigCcEGBgICAeEZyDQAgBkHwAGoQ+QMLAkAgEiAGKAJkQYGAgIB4RnINACAGQeQAahD5AwsCQCATIAYoAlhBgICAgHhGcg0AIAZB2ABqEKwFIAZB2ABqEOcECwJAIAEgBigCOEVyDQAgBkE4aiAGQShBCBDhAgsgECAGKAIsQYGAgIB4RnINACAGQSxqEPkDCyAGQRhqEJ8DCyAGQZADaiQAC6koAht/A34jAEHABmsiBSQAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAIAEpAwAiIEIAUQ0AIAEpAwgiIUIAUQ0BIAEpAxAiIkIAUQ0CICAgInwgIFQNAyAgICFUDQQgAS4BGCEBIAUgID4CDCAFQQFBAiAgQoCAgIAQVCIGGzYCrAEgBUEAICBCIIinIAYbNgIQAkBBmAFFDQAgBUEUakEAQZgB/AsACwJAQZwBRQ0AIAVBtAFqQQBBnAH8CwALIAVBATYCsAEgBUEBNgLQAiABrCAgQn98eX1CwprB6AR+QoChzaC0AnxCIIinIgbBIQcCQAJAIAFBAEgNACAFQQxqIAEQQRoMAQsgBUGwAWpBACABa8EQQRoLAkACQCAHQX9KDQAgBUEMakEAIAdrQf//A3EQPhoMAQsgBUGwAWogBkH//wFxED4aCwJAQaQBRQ0AIAVBnAVqIAVBsAFqQaQB/AoAAAsgAyEIAkAgA0EKSQ0AIAVBnAVqQXhqIQkgAyEIA0AgBSgCvAYiAUEpTw0HAkAgAUUNACABQf////8DaiEKIAFBAnQhBgJAAkAgAUEBRw0AIAVBnAVqIAZqIQFCACEgDAELIAkgBmohASAKQf////8DcUEBakH+////B3EhBkIAISADQCABQQRqIgsgIEIghiALNQIAhCIgQoCU69wDgCIhPgIAIAEgICAhQoCU69wDfn1CIIYgATUCAIQiIEKAlOvcA4AiIT4CACAgICFCgJTr3AN+fSEgIAFBeGohASAGQX5qIgYNAAsgAUEIaiEBICBCIIYhIAsgCkEBcQ0AIAFBfGoiASAgIAE1AgCEQoCU69wDgD4CAAsgCEF3aiIIQQlLDQALCyAIQQJ0QdjlwABqKAIAQQF0IgZFDQYgBSgCvAYiAUEpTw0HAkACQCABDQBBACEBDAELIAFB/////wNqIQggAUECdCELIAatISACQAJAIAFBAUcNACAFQZwFaiALaiEBQgAhIQwBCyALIAVBnAVqakF4aiEBIAhB/////wNxQQFqQf7///8HcSEGQgAhIQNAIAFBBGoiCyAhQiCGIAs1AgCEIiEgIIAiIj4CACABICEgIiAgfn1CIIYgATUCAIQiISAggCIiPgIAICEgIiAgfn0hISABQXhqIQEgBkF+aiIGDQALIAFBCGohASAhQiCGISELAkAgCEEBcQ0AIAFBfGoiASAhIAE1AgCEICCAPgIACyAFKAK8BiEBCwJAAkACQAJAIAUoAqwBIgwgASAMIAFLGyINQShLDQACQCANDQBBACENDAQLIA1BAXEhDiANQQFHDQFBACEIQQAhCgwCCyANQShBlPzAABDPBAALIA1BPnEhD0EAIQggBUGcBWohASAFQQxqIQZBACEKA0AgASABKAIAIgkgBigCAGoiCyAIQQFxaiIQNgIAIAFBBGoiCCAIKAIAIhEgBkEEaigCAGoiCCALIAlJIBAgC0lyaiILNgIAIAggEUkgCyAISXIhCCAGQQhqIQYgAUEIaiEBIA8gCkECaiIKRw0ACwsCQCAORQ0AIAVBnAVqIApBAnQiAWoiBiAGKAIAIgYgBUEMaiABaigCAGoiASAIaiILNgIAIAEgBkkgCyABSXIhCAsgCEEBcUUNACANQShGDQkgBUGcBWogDUECdGpBATYCACANQQFqIQ0LIAUgDTYCvAYgBSgC0AIiCSANIAkgDUsbIgFBKU8NCSABQQJ0IQECQAJAA0AgAUUNASABQXxqIgEgBUGcBWpqKAIAIgYgASAFQbABamooAgAiC0YNAAsgBiALTw0BDAwLIAENCwsgB0EBaiEHDAsLQZPUwABBHEGc18AAEP8CAAtBwNTAAEEdQazXwAAQ/wIAC0Hw1MAAQRxBvNfAABD/AgALQdTWwABBNkGs2MAAEP8CAAtBjNbAAEE3QZzYwAAQ/wIACyABQShBlPzAABDPBAALQdv8wABBG0GU/MAAEP8CAAsgAUEoQZT8wAAQzwQAC0EoQShBlPzAABCuAgALIAFBKEGU/MAAEM8EAAsCQCAMDQBBACEMIAVBADYCrAEMAQsgDEF/akH/////A3EiAUEBaiILQQNxIQYCQAJAIAFBA08NACAFQQxqIQFCACEgDAELIAtB/P///wdxIQsgBUEMaiEBQgAhIANAIAEgATUCAEIKfiAgfCIgPgIAIAFBBGoiCCAINQIAQgp+ICBCIIh8IiA+AgAgAUEIaiIIIAg1AgBCCn4gIEIgiHwiID4CACABQQxqIgggCDUCAEIKfiAgQiCIfCIhPgIAICFCIIghICABQRBqIQEgC0F8aiILDQALCwJAIAZFDQADQCABIAE1AgBCCn4gIHwiIT4CACABQQRqIQEgIUIgiCEgIAZBf2oiBg0ACwsCQCAhQoCAgIAQVA0AIAxBKEYNAiAFQQxqIAxBAnRqICCnNgIAIAxBAWohDAsgBSAMNgKsAQtBACEIQQEhECAHwSIBIATBIgZIIhINDSAHIARrwSADIAEgBmsgA0kbIhNFDQ0CQEGkAUUiAQ0AIAVB1AJqIAVBsAFqQaQB/AoAAAtBASEUIAVB1AJqQQEQQSEVAkAgAQ0AIAVB+ANqIAVBsAFqQaQB/AoAAAsgBUH4A2pBAhBBIRYCQCABDQAgBUGcBWogBUGwAWpBpAH8CgAACyAFQbABakF8aiEPIAVB1AJqQXxqIREgBUH4A2pBfGohECAFQZwFakF8aiEKIAVBnAVqQQMQQSEXIBUoAqABIRggFigCoAEhGSAXKAKgASEaQQAhGyAFKAKsASEMIAUoAtACIQkCQANAIAxBKU8NAyAMQQJ0IQtBACEBAkACQAJAA0AgCyABRg0BIAVBDGogAWohBiABQQRqIQEgBigCAEUNAAsgGiAMIBogDEsbIhxBKU8NByAcQQJ0IQECQANAIAFFDQEgCiABaiEGIAFBfGoiASAFQQxqaigCACILIAYoAgAiBkYNAAsgCyAGTw0CQQAhHQwDCyABRQ0BQQAhHQwCCyATIANLDQcCQCATIBtGDQAgEyAbayIBRQ0AIAIgG2pBMCAB/AsACyAAIAc7AQggACATNgIEDBILQQEhCCAcQQFxIR1BACEMAkAgHEEBRg0AIBxBPnEhHkEAIQxBASEIIAVBDGohASAFQZwFaiEGA0AgASABKAIAIg0gBigCAEF/c2oiCyAIQQFxaiIENgIAIAFBBGoiCCAIKAIAIg4gBkEEaigCAEF/c2oiCCALIA1JIAQgC0lyaiILNgIAIAggDkkgCyAISXIhCCAGQQhqIQYgAUEIaiEBIB4gDEECaiIMRw0ACwsCQCAdRQ0AIAVBDGogDEECdCIBaiIGIAYoAgAiBiAXIAFqKAIAQX9zaiIBIAhqIgs2AgAgASAGSSALIAFJciEICyAIQQFxRQ0HIAUgHDYCrAFBCCEdIBwhDAsgGSAMIBkgDEsbIh5BKU8NByAeQQJ0IQECQAJAAkADQCABRQ0BIBAgAWohBiABQXxqIgEgBUEMamooAgAiCyAGKAIAIgZGDQALIAsgBk8NASAMIR4MAgsgAUUNACAMIR4MAQsCQCAeRQ0AQQEhCCAeQQFxIR9BACEMAkAgHkEBRg0AIB5BPnEhHEEAIQxBASEIIAVBDGohASAFQfgDaiEGA0AgASABKAIAIg0gBigCAEF/c2oiCyAIQQFxaiIENgIAIAFBBGoiCCAIKAIAIg4gBkEEaigCAEF/c2oiCCALIA1JIAQgC0lyaiILNgIAIAggDkkgCyAISXIhCCAGQQhqIQYgAUEIaiEBIBwgDEECaiIMRw0ACwsCQCAfRQ0AIAVBDGogDEECdCIBaiIGIAYoAgAiBiAWIAFqKAIAQX9zaiIBIAhqIgs2AgAgASAGSSALIAFJciEICyAIQQFxRQ0KCyAFIB42AqwBIB1BBHIhHQsgGCAeIBggHksbIhxBKU8NCSAcQQJ0IQECQAJAAkADQCABRQ0BIBEgAWohBiABQXxqIgEgBUEMamooAgAiCyAGKAIAIgZGDQALIAsgBk8NASAeIRwMAgsgAUUNACAeIRwMAQsCQCAcRQ0AQQEhCCAcQQFxIR9BACEMAkAgHEEBRg0AIBxBPnEhHkEAIQxBASEIIAVBDGohASAFQdQCaiEGA0AgASABKAIAIg0gBigCAEF/c2oiCyAIQQFxaiIENgIAIAFBBGoiCCAIKAIAIg4gBkEEaigCAEF/c2oiCCALIA1JIAQgC0lyaiILNgIAIAggDkkgCyAISXIhCCAGQQhqIQYgAUEIaiEBIB4gDEECaiIMRw0ACwsCQCAfRQ0AIAVBDGogDEECdCIBaiIGIAYoAgAiBiAVIAFqKAIAQX9zaiIBIAhqIgs2AgAgASAGSSALIAFJciEICyAIQQFxRQ0MCyAFIBw2AqwBIB1BAmohHQsgCSAcIAkgHEsbIgxBKU8NCyAMQQJ0IQECQAJAAkADQCABRQ0BIA8gAWohBiABQXxqIgEgBUEMamooAgAiCyAGKAIAIgZGDQALIAsgBk8NASAcIQwMAgsgAUUNACAcIQwMAQsCQCAMRQ0AQQEhCCAMQQFxIR9BACENAkAgDEEBRg0AIAxBPnEhHEEAIQ1BASEIIAVBDGohASAFQbABaiEGA0AgASABKAIAIgQgBigCAEF/c2oiCyAIQQFxaiIONgIAIAFBBGoiCCAIKAIAIh4gBkEEaigCAEF/c2oiCCALIARJIA4gC0lyaiILNgIAIAggHkkgCyAISXIhCCAGQQhqIQYgAUEIaiEBIBwgDUECaiINRw0ACwsCQCAfRQ0AIAVBDGogDUECdCIBaiIGIAYoAgAiBiAFQbABaiABaigCAEF/c2oiASAIaiILNgIAIAEgBkkgCyABSXIhCAsgCEEBcUUNDgsgBSAMNgKsASAdQQFqIR0LIBsgA08NASACIBtqIB1BMGo6AAAgDEEpTw0NAkACQCAMDQBBACEMDAELIAxBf2pB/////wNxIgFBAWoiC0EDcSEGAkACQCABQQNPDQAgBUEMaiEBQgAhIQwBCyALQfz///8HcSELIAVBDGohAUIAISEDQCABIAE1AgBCCn4gIXwiID4CACABQQRqIgggCDUCAEIKfiAgQiCIfCIgPgIAIAFBCGoiCCAINQIAQgp+ICBCIIh8IiA+AgAgAUEMaiIIIAg1AgBCCn4gIEIgiHwiID4CACAgQiCIISEgAUEQaiEBIAtBfGoiCw0ACwsCQCAGRQ0AA0AgASABNQIAQgp+ICF8IiA+AgAgAUEEaiEBICBCIIghISAGQX9qIgYNAAsLICBCgICAgBBUDQAgDEEoRg0PIAVBDGogDEECdGogIac2AgAgDEEBaiEMCyAFIAw2AqwBIBtBAWohGyAUIBQgE0kiAWohFCABDQALQQAhECATIQgMDgsgGyADQfzXwAAQrgIAC0EoQShBlPzAABCuAgALIAxBKEGU/MAAEM8EAAsgHEEoQZT8wAAQzwQACyATIANBjNjAABDPBAALQaT8wABBGkGU/MAAEP8CAAsgHkEoQZT8wAAQzwQAC0Gk/MAAQRpBlPzAABD/AgALIBxBKEGU/MAAEM8EAAtBpPzAAEEaQZT8wAAQ/wIACyAMQShBlPzAABDPBAALQaT8wABBGkGU/MAAEP8CAAsgDEEoQZT8wAAQzwQAC0EoQShBlPzAABCuAgALAkACQAJAAkACQAJAAkAgCUEpTw0AAkACQCAJDQBBACEJDAELIAlBf2pB/////wNxIgFBAWoiC0EDcSEGAkACQCABQQNPDQAgBUGwAWohAUIAISAMAQsgC0H8////B3EhCyAFQbABaiEBQgAhIANAIAEgATUCAEIFfiAgfCIgPgIAIAFBBGoiCiAKNQIAQgV+ICBCIIh8IiA+AgAgAUEIaiIKIAo1AgBCBX4gIEIgiHwiID4CACABQQxqIgogCjUCAEIFfiAgQiCIfCIhPgIAICFCIIghICABQRBqIQEgC0F8aiILDQALCwJAIAZFDQADQCABIAE1AgBCBX4gIHwiIT4CACABQQRqIQEgIUIgiCEgIAZBf2oiBg0ACwsgIUKAgICAEFQNACAJQShGDQIgBUGwAWogCUECdGogIKc2AgAgCUEBaiEJCyAFIAk2AtACIAkgDCAJIAxLGyIBQSlPDQIgAUECdCEBIAVBDGpBfGohCiAFQbABakF8aiEJAkACQANAIAFFDQEgCSABaiEGIAogAWohCyABQXxqIQEgCygCACILIAYoAgAiBkYNAAsgCyAGSyALIAZJayEBDAELQX9BACABGyEBCwJAIAFB/wFxDgIABAULQQAhASAQDQUCQCAIQX9qIgEgA08NACACIAFqLQAAQQFxDQQMBQsgASADQczXwAAQrgIACyAJQShBlPzAABDPBAALQShBKEGU/MAAEK4CAAsgAUEoQZT8wAAQzwQACwJAIAggA0sNACACIAhqIQpBACEBIAIhBgJAA0AgCCABRg0BIAFBAWohASAGQX9qIgYgCGoiCy0AAEE5Rg0ACyALIAstAABBAWo6AAAgCCABa0EBaiAITw0CIAFBf2oiAUUNAiALQQFqQTAgAfwLAAwCCwJAAkAgEEUNAEExIQEMAQsgAkExOgAAAkAgCEEBRw0AQTAhAQwBC0EwIQEgCEF/aiIGRQ0AIAJBAWpBMCAG/AsACyAHQQFqIQcgEg0BIAggA08NASAKIAE6AAAgCEEBaiEIDAELIAggA0Hc18AAEM8EAAsgCCADSw0BIAghAQsgACAHOwEIIAAgATYCBAwBCyAIIANB7NfAABDPBAALIAAgAjYCACAFQcAGaiQAC+sjAgl/AX4jAEEQayIBJAACQAJAAkACQAJAAkACQAJAIABB9QFJDQACQCAAQcz/e00NAEEAIQAMCAsgAEELaiICQXhxIQNBACgCqINBIgRFDQRBHyEFAkAgAEH0//8HSw0AIANBBiACQQh2ZyIAa3ZBAXEgAEEBdGtBPmohBQtBACADayECAkAgBUECdEGMgMEAaigCACIGDQBBACEAQQAhBwwCC0EAIQAgA0EAQRkgBUEBdmsgBUEfRht0IQhBACEHA0ACQCAGIgYoAgRBeHEiCSADSQ0AIAkgA2siCSACTw0AIAkhAiAGIQcgCQ0AQQAhAiAGIQcgBiEADAQLIAYoAhQiCSAAIAkgBiAIQR12QQRxaigCECIGRxsgACAJGyEAIAhBAXQhCCAGRQ0CDAALCwJAQQAoAqSDQSIGQRAgAEELakH4A3EgAEELSRsiA0EDdiICdiIAQQNxRQ0AAkACQCAAQX9zQQFxIAJqIghBA3QiA0GcgcEAaiIAIANBpIHBAGooAgAiAigCCCIHRg0AIAcgADYCDCAAIAc2AggMAQtBACAGQX4gCHdxNgKkg0ELIAJBCGohACACIANBA3I2AgQgAiADaiIDIAMoAgRBAXI2AgQMBwsgA0EAKAKsg0FNDQMCQAJAAkAgAA0AQQAoAqiDQSIARQ0GIABoQQJ0QYyAwQBqKAIAIgcoAgRBeHEgA2shAiAHIQYDQAJAIAcoAhAiAA0AIAcoAhQiAA0AIAYoAhghBQJAAkACQCAGKAIMIgAgBkcNACAGQRRBECAGKAIUIgAbaigCACIHDQFBACEADAILIAYoAggiByAANgIMIAAgBzYCCAwBCyAGQRRqIAZBEGogABshCANAIAghCSAHIgBBFGogAEEQaiAAKAIUIgcbIQggAEEUQRAgBxtqKAIAIgcNAAsgCUEANgIACyAFRQ0EAkACQCAGIAYoAhxBAnRBjIDBAGoiBygCAEYNAAJAIAUoAhAgBkYNACAFIAA2AhQgAA0CDAcLIAUgADYCECAADQEMBgsgByAANgIAIABFDQQLIAAgBTYCGAJAIAYoAhAiB0UNACAAIAc2AhAgByAANgIYCyAGKAIUIgdFDQQgACAHNgIUIAcgADYCGAwECyAAKAIEQXhxIANrIgcgAiAHIAJJIgcbIQIgACAGIAcbIQYgACEHDAALCwJAAkAgACACdEECIAJ0IgBBACAAa3JxaCIJQQN0IgJBnIHBAGoiByACQaSBwQBqKAIAIgAoAggiCEYNACAIIAc2AgwgByAINgIIDAELQQAgBkF+IAl3cTYCpINBCyAAIANBA3I2AgQgACADaiIIIAIgA2siB0EBcjYCBCAAIAJqIAc2AgACQEEAKAKsg0EiBkUNACAGQXhxQZyBwQBqIQJBACgCtINBIQMCQAJAQQAoAqSDQSIJQQEgBkEDdnQiBnENAEEAIAkgBnI2AqSDQSACIQYMAQsgAigCCCEGCyACIAM2AgggBiADNgIMIAMgAjYCDCADIAY2AggLIABBCGohAEEAIAg2ArSDQUEAIAc2AqyDQQwIC0EAQQAoAqiDQUF+IAYoAhx3cTYCqINBCwJAAkACQCACQRBJDQAgBiADQQNyNgIEIAYgA2oiAyACQQFyNgIEIAMgAmogAjYCAEEAKAKsg0EiCEUNASAIQXhxQZyBwQBqIQdBACgCtINBIQACQAJAQQAoAqSDQSIJQQEgCEEDdnQiCHENAEEAIAkgCHI2AqSDQSAHIQgMAQsgBygCCCEICyAHIAA2AgggCCAANgIMIAAgBzYCDCAAIAg2AggMAQsgBiACIANqIgBBA3I2AgQgBiAAaiIAIAAoAgRBAXI2AgQMAQtBACADNgK0g0FBACACNgKsg0ELIAZBCGohAAwGCwJAIAAgB3INAEEAIQdBAiAFdCIAQQAgAGtyIARxIgBFDQMgAGhBAnRBjIDBAGooAgAhAAsgAEUNAQsDQCAAIAcgACgCBEF4cSIGIANrIgkgAkkiBRshBCAGIANJIQggCSACIAUbIQkCQCAAKAIQIgYNACAAKAIUIQYLIAcgBCAIGyEHIAIgCSAIGyECIAYhACAGDQALCyAHRQ0AAkBBACgCrINBIgAgA0kNACACIAAgA2tPDQELIAcoAhghBQJAAkACQCAHKAIMIgAgB0cNACAHQRRBECAHKAIUIgAbaigCACIGDQFBACEADAILIAcoAggiBiAANgIMIAAgBjYCCAwBCyAHQRRqIAdBEGogABshCANAIAghCSAGIgBBFGogAEEQaiAAKAIUIgYbIQggAEEUQRAgBhtqKAIAIgYNAAsgCUEANgIACyAFRQ0CAkACQCAHIAcoAhxBAnRBjIDBAGoiBigCAEYNAAJAIAUoAhAgB0YNACAFIAA2AhQgAA0CDAULIAUgADYCECAADQEMBAsgBiAANgIAIABFDQILIAAgBTYCGAJAIAcoAhAiBkUNACAAIAY2AhAgBiAANgIYCyAHKAIUIgZFDQIgACAGNgIUIAYgADYCGAwCCwJAAkACQAJAAkACQEEAKAKsg0EiACADTw0AAkBBACgCsINBIgAgA0sNACABQQRqQdCDwQAgA0GvgARqQYCAfHEQhgMCQCABKAIEIgYNAEEAIQAMCgsgASgCDCEFQQBBACgCvINBIAEoAggiCWoiADYCvINBQQAgAEEAKALAg0EiAiAAIAJLGzYCwINBAkACQAJAQQAoAriDQSICRQ0AQYyBwQAhAANAIAYgACgCACIHIAAoAgQiCGpGDQIgACgCCCIADQAMAwsLAkACQEEAKALIg0EiAEUNACAGIABPDQELQQAgBjYCyINBC0EAQf8fNgLMg0FBACAFNgKYgUFBACAJNgKQgUFBACAGNgKMgUFBAEGcgcEANgKogUFBAEGkgcEANgKwgUFBAEGcgcEANgKkgUFBAEGsgcEANgK4gUFBAEGkgcEANgKsgUFBAEG0gcEANgLAgUFBAEGsgcEANgK0gUFBAEG8gcEANgLIgUFBAEG0gcEANgK8gUFBAEHEgcEANgLQgUFBAEG8gcEANgLEgUFBAEHMgcEANgLYgUFBAEHEgcEANgLMgUFBAEHUgcEANgLggUFBAEHMgcEANgLUgUFBAEHcgcEANgLogUFBAEHUgcEANgLcgUFBAEHcgcEANgLkgUFBAEHkgcEANgLwgUFBAEHkgcEANgLsgUFBAEHsgcEANgL4gUFBAEHsgcEANgL0gUFBAEH0gcEANgKAgkFBAEH0gcEANgL8gUFBAEH8gcEANgKIgkFBAEH8gcEANgKEgkFBAEGEgsEANgKQgkFBAEGEgsEANgKMgkFBAEGMgsEANgKYgkFBAEGMgsEANgKUgkFBAEGUgsEANgKggkFBAEGUgsEANgKcgkFBAEGcgsEANgKogkFBAEGkgsEANgKwgkFBAEGcgsEANgKkgkFBAEGsgsEANgK4gkFBAEGkgsEANgKsgkFBAEG0gsEANgLAgkFBAEGsgsEANgK0gkFBAEG8gsEANgLIgkFBAEG0gsEANgK8gkFBAEHEgsEANgLQgkFBAEG8gsEANgLEgkFBAEHMgsEANgLYgkFBAEHEgsEANgLMgkFBAEHUgsEANgLggkFBAEHMgsEANgLUgkFBAEHcgsEANgLogkFBAEHUgsEANgLcgkFBAEHkgsEANgLwgkFBAEHcgsEANgLkgkFBAEHsgsEANgL4gkFBAEHkgsEANgLsgkFBAEH0gsEANgKAg0FBAEHsgsEANgL0gkFBAEH8gsEANgKIg0FBAEH0gsEANgL8gkFBAEGEg8EANgKQg0FBAEH8gsEANgKEg0FBAEGMg8EANgKYg0FBAEGEg8EANgKMg0FBAEGUg8EANgKgg0FBAEGMg8EANgKUg0FBACAGQQ9qQXhxIgBBeGoiAjYCuINBQQBBlIPBADYCnINBQQAgBiAAayAJQVhqIgBqQQhqIgc2ArCDQSACIAdBAXI2AgQgBiAAakEoNgIEQQBBgICAATYCxINBDAgLIAIgBk8NACAHIAJLDQAgACgCDCIHQQFxDQAgB0EBdiAFRg0DC0EAQQAoAsiDQSIAIAYgACAGSRs2AsiDQSAGIAlqIQdBjIHBACEAAkACQAJAA0AgACgCACIIIAdGDQEgACgCCCIADQAMAgsLIAAoAgwiB0EBcQ0AIAdBAXYgBUYNAQtBjIHBACEAAkADQAJAIAAoAgAiByACSw0AIAIgByAAKAIEaiIHSQ0CCyAAKAIIIQAMAAsLQQAgBkEPakF4cSIAQXhqIgg2AriDQUEAIAYgAGsgCUFYaiIAakEIaiIENgKwg0EgCCAEQQFyNgIEIAYgAGpBKDYCBEEAQYCAgAE2AsSDQSACIAdBYGpBeHFBeGoiACAAIAJBEGpJGyIIQRs2AgRBACkCjIFBIQogCEEQakEAKQKUgUE3AgAgCCAKNwIIQQAgBTYCmIFBQQAgCTYCkIFBQQAgBjYCjIFBQQAgCEEIajYClIFBIAhBHGohAANAIABBBzYCACAAQQRqIgAgB0kNAAsgCCACRg0HIAggCCgCBEF+cTYCBCACIAggAmsiAEEBcjYCBCAIIAA2AgACQCAAQYACSQ0AIAIgABCGAQwICyAAQfgBcUGcgcEAaiEHAkACQEEAKAKkg0EiBkEBIABBA3Z0IgBxDQBBACAGIAByNgKkg0EgByEADAELIAcoAgghAAsgByACNgIIIAAgAjYCDCACIAc2AgwgAiAANgIIDAcLIAAgBjYCACAAIAAoAgQgCWo2AgQgBkEPakF4cUF4aiIHIANBA3I2AgQgCEEPakF4cUF4aiICIAcgA2oiAGshAyACQQAoAriDQUYNAyACQQAoArSDQUYNBAJAIAIoAgQiBkEDcUEBRw0AIAIgBkF4cSIGEHwgBiADaiEDIAIgBmoiAigCBCEGCyACIAZBfnE2AgQgACADQQFyNgIEIAAgA2ogAzYCAAJAIANBgAJJDQAgACADEIYBDAYLIANB+AFxQZyBwQBqIQICQAJAQQAoAqSDQSIGQQEgA0EDdnQiA3ENAEEAIAYgA3I2AqSDQSACIQMMAQsgAigCCCEDCyACIAA2AgggAyAANgIMIAAgAjYCDCAAIAM2AggMBQtBACAAIANrIgI2ArCDQUEAQQAoAriDQSIAIANqIgc2AriDQSAHIAJBAXI2AgQgACADQQNyNgIEIABBCGohAAwIC0EAKAK0g0EhAgJAAkAgACADayIHQQ9LDQBBAEEANgK0g0FBAEEANgKsg0EgAiAAQQNyNgIEIAIgAGoiACAAKAIEQQFyNgIEDAELQQAgBzYCrINBQQAgAiADaiIGNgK0g0EgBiAHQQFyNgIEIAIgAGogBzYCACACIANBA3I2AgQLIAJBCGohAAwHCyAAIAggCWo2AgRBAEEAKAK4g0EiAEEPakF4cSICQXhqIgc2AriDQUEAIAAgAmtBACgCsINBIAlqIgJqQQhqIgY2ArCDQSAHIAZBAXI2AgQgACACakEoNgIEQQBBgICAATYCxINBDAMLQQAgADYCuINBQQBBACgCsINBIANqIgM2ArCDQSAAIANBAXI2AgQMAQtBACAANgK0g0FBAEEAKAKsg0EgA2oiAzYCrINBIAAgA0EBcjYCBCAAIANqIAM2AgALIAdBCGohAAwDC0EAIQBBACgCsINBIgIgA00NAkEAIAIgA2siAjYCsINBQQBBACgCuINBIgAgA2oiBzYCuINBIAcgAkEBcjYCBCAAIANBA3I2AgQgAEEIaiEADAILQQBBACgCqINBQX4gBygCHHdxNgKog0ELAkACQCACQRBJDQAgByADQQNyNgIEIAcgA2oiACACQQFyNgIEIAAgAmogAjYCAAJAIAJBgAJJDQAgACACEIYBDAILIAJB+AFxQZyBwQBqIQMCQAJAQQAoAqSDQSIGQQEgAkEDdnQiAnENAEEAIAYgAnI2AqSDQSADIQIMAQsgAygCCCECCyADIAA2AgggAiAANgIMIAAgAzYCDCAAIAI2AggMAQsgByACIANqIgBBA3I2AgQgByAAaiIAIAAoAgRBAXI2AgQLIAdBCGohAAsgAUEQaiQAIAALpxECCH8afiMAQTBrIgQkAAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAIAEpAwAiDEIAUQ0AIAEpAwgiDUIAUQ0BIAEpAxAiDkIAUQ0CIAwgDnwiDiAMVA0DIAwgDVQNBCADQRBNDQUgDkKAgICAgICAgCBaDQYgBCABLwEYIgE7AQggBCAMIA19Ig83AwAgASABQWBqIAEgDkKAgICAEFQiBRsiBkFwaiAGIA5CIIYgDiAFGyIOQoCAgICAgMAAVCIFGyIGQXhqIAYgDkIQhiAOIAUbIg5CgICAgICAgIABVCIFGyIGQXxqIAYgDkIIhiAOIAUbIg5CgICAgICAgIAQVCIFGyIGQX5qIAYgDkIEhiAOIAUbIg5CgICAgICAgIDAAFQiBRsgDkIChiAOIAUbIhBCf1UiB2siBWvBIgZBf0wNByAEQn8gBq0iEYgiDiAPgzcDECAPIA5WDQggBCABOwEIIAQgDDcDACAEIA4gDIM3AxAgDCAOVg0JQaB/IAVrwUHQAGxBsKcFakHOEG0iAUHRAE8NCiABQQR0IgFBwNjAAGopAwAiDkL/////D4MiDSAMIBFCP4MiEYYiDEIgiCISfiITQiCIIhQgDkIgiCIVIBJ+IhZ8IBUgDEL/////D4MiDH4iDkIgiCIXfCEYIBNC/////w+DIA0gDH5CIIh8IA5C/////w+DfCIZQoCAgIAIfEIgiCEMQgFBACAFIAFByNjAAGovAQBqa0E/ca0iDoYiE0J/fCEaIA0gDyARhiIPQiCIIhF+IhtC/////w+DIA0gD0L/////D4MiD35CIIh8IBUgD34iD0L/////D4N8IhxCgICAgAh8QiCIIR0gFSARfiEeIA9CIIghHyAbQiCIISAgAUHK2MAAai8BACEBAkAgFSAQIAethiIPQiCIIiF+IiIgDSAhfiIQQiCIIiN8IBUgD0L/////D4MiD34iEUIgiCIkfCAQQv////8PgyANIA9+QiCIfCARQv////8Pg3wiJUKAgICACHxCIIh8QgF8IhsgDoinIgZBkM4ASQ0AIAZBwIQ9SQ0MAkAgBkGAwtcvSQ0AQQhBCSAGQYCU69wDSSIFGyEIQYDC1y9BgJTr3AMgBRshBQwOC0EGQQcgBkGAreIESSIFGyEIQcCEPUGAreIEIAUbIQUMDQsCQCAGQeQASQ0AQQJBAyAGQegHSSIFGyEIQeQAQegHIAUbIQUMDQtBCkEBIAZBCUsiCBshBQwMC0GT1MAAQRxBkOPAABD/AgALQcDUwABBHUGg48AAEP8CAAtB8NTAAEEcQbDjwAAQ/wIAC0HU1sAAQTZB0OTAABD/AgALQYzWwABBN0HA5MAAEP8CAAtBnNXAAEEtQcDjwAAQ/wIAC0HQ48AAQS1BgOTAABD/AgALQefRwABBHUGo0sAAEP8CAAsgBEEANgIYQQAgBEEQaiAEIARBGGpBuNLAABCXAwALIARBADYCGEEAIARBEGogBCAEQRhqQbjSwAAQlwMACyABQdEAQYDjwAAQrgIAC0EEQQUgBkGgjQZJIgUbIQhBkM4AQaCNBiAFGyEFCyAYIAx8IRggGyAagyEMIAggAWtBAWohCSAbIB4gIHwgH3wgHXx9Ih1CAXwiESAagyEPQQAhAQJAAkACQAJAAkACQAJAAkADQCAGIAVuIQcgAyABRg0CIAIgAWoiCiAHQTBqIgs6AAACQAJAIBEgBiAHIAVsayIGrSAOhiINIAx8IhBWDQAgCCABRw0BIAFBAWohAUIBIQ0DQCANIRAgDyERIAEgA08NBiACIAFqIAxCCn4iDCAOiKdBMGoiBToAACABQQFqIQEgEEIKfiENIBFCCn4iDyAMIBqDIgxYDQALIA0gGyAYfX4iDiANfCESIA8gDH0gE1QiBg0HIA4gDX0iGiAMVg0DDAcLIBEgEH0iESAFrSAOhiIOVCEFIBsgGH0iD0IBfCEYIBAgD0J/fCITWg0FIBEgDlQNBSAjICR8ICVCgICAgAh8QiCIIhF8ICJ8IQ9CAiAgIB98IBxCgICAgAh8QiCIfCAefCAMIA58IhAgDXx8fSEbQgAgFCAXfCAZQoCAgIAIfEIgiHwiGiAWfCANIAx8fH0hHiAaIBB8IBUgEiAhfX58ICN9ICR9IBF9IREDQAJAIA0gEHwiGiATVA0AIB4gD3wgDSARfFoNACANIAx8IRBBACEFDAcLIAogC0F/aiILOgAAIAwgDnwhDCAbIA98IRUCQCAaIBNaDQAgESAOfCERIBAgDnwhECAPIA59IQ8gFSAOWg0BCwsgFSAOVCEFIA0gDHwhEAwFCyABQQFqIQEgBUEKSSEHIAVBCm4hBSAHRQ0AC0GQ5MAAEJ0DAAsgAiABakF/aiEHIBMgFCAXfCAZQoCAgIAIfEIgiHwgFnxCCn4gIyAkfCAlQoCAgIAIfEIgiHwgInxCCn59IBB+fCEbIBFCCn4gEyAMfH0hFSAaIAx9IR5CACEOA0ACQCAMIBN8Ig0gGlQNACAeIA58IBsgDHxaDQBBACEGDAULIAcgBUF/aiIFOgAAIBUgDnwiESATVCEGIA0gGloNBSAOIBN9IQ4gDSEMIBEgE1QNBQwACwsgAyADQaDkwAAQrgIACyABIANBsOTAABCuAgALAkAgGCAQWA0AIAUNAAJAIBAgDnwiDCAYVA0AIBggEH0gDCAYfVQNAQsgAEEANgIADAQLAkACQCAQQgJUDQAgECAdQn18WA0BCyAAQQA2AgAMBAsgACAJOwEIIAAgAUEBajYCBAwCCyAMIQ0LAkAgEiANWA0AIAYNAAJAIA0gE3wiDCASVA0AIBIgDX0gDCASfVQNAQsgAEEANgIADAILAkACQCAQQhR+IA1WDQAgDSAQQlh+IA98WA0BCyAAQQA2AgAMAgsgACAJOwEIIAAgATYCBAsgACACNgIACyAEQTBqJAALxg4CB38DfiMAQZACayICJAAgAiABNgIUIAJB0AFqIAJBFGoQpAEgAigC0AEhAQJAAkACQAJAAkACQAJAIAItANQBIgNBfmoOAgIAAQsgAEEANgIAIAAgATYCBCACKAIUIgFBgwFLDQMMBAsgAiADOgAkIAIgATYCICACQQA2AhgCQAJAQQAoAtiDQUEBRw0AQQApA+iDQSEJQQApA+CDQSEKDAELIAJB0AFqENgCQQBCATcD2INBQQAgAikD2AEiCTcD6INBIAIpA9ABIQoLIAIgCjcDuAFBACAKQgF8NwPgg0EgAiAJNwPAASACQgA3A7ABIAJBADYCrAEgAkGYg8AANgKoASACQShqQQRqIQQgAkHcAWohASACQYgBakEEaiEFAkACQANAIAJB8ABqIAJBGGoQvQECQAJAAkACQCACKAJwIgNBgICAgHhqDgIFAAELIAIoAnQhAQwBCyACIAIpAnQiCjcCgAEgAiADNgJ8IAIoAhghBiACQQA2AhggBkUNCSACQfgBaiACKAIcEEkgAi0A+AFBBkcNASACKAL8ASEBIAJB/ABqEKwFIAJB/ABqEOcECyAAQQA2AgAgACABNgIEIAJBqAFqIAJBKEEIEOECDAMLIAUgAikD+AE3AgAgBUEQaiACQfgBakEQaikDADcCACAFQQhqIAJB+AFqQQhqKQMANwIAIAJBKGpBCGoiBiACQYgBakEIaikCADcDACACQShqQRBqIgcgAkGIAWpBEGopAgA3AwAgAkEoakEYaiIIIAJBiAFqQRhqKAIANgIAIAIgAikCiAE3AyggAUEYaiAIKAIANgIAIAFBEGogBykDADcCACABQQhqIAYpAwA3AgAgASACKQMoNwIAIAIgCqciBzYC1AEgAiADNgLQASACQdABakEIaiAKQiCIpyIGNgIAIAJByABqQQhqIAY2AgAgAiACKQPQATcDSCACQdgAakEQaiAEQRBqKQIANwMAIAJB2ABqQQhqIARBCGopAgA3AwAgAiAEKQIANwNYIAJBiAFqIAJBqAFqIAJByABqIAJB2ABqEFUgAi0AiAFBBkYNACACQYgBahCBAwwACwsgAiAGNgLYASACIAc2AtQBIAJBgICAgHg2AtABIAJB0AFqEMsDIABBGGogAkGoAWpBGGopAwA3AwAgAEEQaiACQagBakEQaikDADcDACAAQQhqIAJBqAFqQQhqKQMANwMAIAAgAikDqAE3AwALIAJBGGoQngMMAQsgAkEIaiACQRRqEJoDAkAgAigCCEEBcUUNACACIAIoAgw2AnwgAkHgAGogAkH8AGoQ7QMgAkEANgJsIAJBADYCWEEAIQECQCACKAJgRQ0AQQAgAigCaCIBIAIoAmRrIgMgAyABSxshAQsCQAJAQQAoAtiDQUEBRw0AQQApA+iDQSEJQQApA+CDQSEKDAELIAJB0AFqENgCQQBCATcD2INBQQAgAikD2AEiCTcD6INBIAIpA9ABIQoLQQAhA0EAIApCAXw3A+CDQQJAAkAgAQ0AQZiDwAAhAUIAIQsMAQsCQAJAIAFBB0sNAEEEQQggAUEESRshAQwBC0F/IAFB5swBIAFB5swBSRtBA3RBB25Bf2pndkEBaiEBCyACQdABaiACQShqQShBCCABQQEQrAEgAigC1AEhAwJAIAIoAtABIgENACACNQLYASELQQAhAQwBCyACKQLYASELIANBCWoiBEUNACABQf8BIAT8CwALIAIgCTcDoAEgAiAKNwOYASACIAs3A5ABIAIgAzYCjAEgAiABNgKIASACQdABaiACQdgAahCeAQJAAkACQAJAIAIoAtABQYGAgIB4Rg0AIAJB0AFqQRBqIQEDQAJAQShFDQAgAkGoAWogAkHQAWpBKPwKAAALIAIoAqgBQYCAgIB4Rg0CIAJBGGpBCGogAkHQAWpBCGooAgA2AgAgAkH4AWpBCGogAUEIaikDADcDACACQfgBakEQaiABQRBqKQMANwMAIAIgAikD0AE3AxggAiABKQMANwP4ASACQShqIAJBiAFqIAJBGGogAkH4AWoQVQJAIAItAChBBkYNACACQShqEIEDCyACQdABaiACQdgAahCeASACKALQAUGBgICAeEcNAAsLIAAgAigC1AE2AgQgAEEANgIAIAJBiAFqIAJBKEEIEOECIAIoAlhFDQIgAigCXCIBQYMBSw0BDAILIAJBqAFqEMsDIABBGGogAkGIAWpBGGopAwA3AwAgAEEQaiACQYgBakEQaikDADcDACAAQQhqIAJBiAFqQQhqKQMANwMAIAAgAikDiAE3AwAgAigCWEUNASACKAJcIgFBhAFJDQELIAEQjwMLIAIoAnwiAUGEAUkNASABEI8DDAELIAJBFGogAkEoakG4gcAAEGEhASAAQQA2AgAgACABNgIECyACKAIUIgFBhAFJDQELIAEQjwMLIAJBkAJqJAAPC0G4ksAAQTEQ7QQAC5YMAhF/A34jAEHAAGsiBSQA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ASACKAIQQYCAgIB4Rw0BIAIoAhQhASACQbQBahCsBSACQbQBahDnBAsgAEEANgIAIAAgATYCBCACQcgAaiACQRhBCBDgAgwDCyACQShqQQhqIAJBEGpBCGooAgAiBTYCACACQYABakEIaiAHQiCIpyIDNgIAIAJBOGpBCGogAzYCACACQZgBakEIaiAFNgIAIAIgAikCECIGNwMoIARBCGogBTYCACAEIAY3AgAgAiABNgKAASACIAenIgU2AoQBIAIgBjcDmAEgAiACKQKAATcDOCACQRBqIAJByABqIAJBOGogAkGYAWoQWyACQRBqEPkDDAALCyACIAM2AogBIAIgBTYChAEgAkGAgICAeDYCgAEgAkGAAWoQswMgAEEYaiACQcgAakEYaikDADcDACAAQRBqIAJByABqQRBqKQMANwMAIABBCGogAkHIAGpBCGopAwA3AwAgACACKQNINwMACyACQegAahCeAwwBCyACIAJBDGoQmgMCQCACKAIAQQFxRQ0AIAIgAigCBDYCOCACQRhqIAJBOGoQ7QMgAkEANgIkIAJBADYCEEEAIQECQCACKAIYRQ0AQQAgAigCICIBIAIoAhxrIgMgAyABSxshAQsCQAJAQQAoAtiDQUEBRw0AQQApA+iDQSEGQQApA+CDQSEHDAELIAJBgAFqENgCQQBCATcD2INBQQAgAikDiAEiBjcD6INBIAIpA4ABIQcLQQAhA0EAIAdCAXw3A+CDQQJAAkAgAQ0AQZiDwAAhAUIAIQgMAQsCQAJAIAFBB0sNAEEEQQggAUEESRshAQwBC0F/IAFBqtUCIAFBqtUCSRtBA3RBB25Bf2pndkEBaiEBCyACQYABaiACQbQBakEYQQggAUEBEKwBIAIoAoQBIQMCQCACKAKAASIBDQAgAjUCiAEhCEEAIQEMAQsgAikCiAEhCCADQQlqIgRFDQAgAUH/ASAE/AsACyACIAY3A2AgAiAHNwNYIAIgCDcDUCACIAM2AkwgAiABNgJIIAJBgAFqIAJBEGoQnAECQAJAAkACQCACKAKAAUGBgICAeEYNACACQYwBaiEBA0AgAkHoAGpBEGogAkGAAWpBEGopAgA3AwAgAkHoAGpBCGogAkGAAWpBCGoiAykCADcDACACIAIpAoABIgc3A2ggB6dBgICAgHhGDQIgAkGYAWpBCGogAygC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oAEgAA8LIANBKEGU/MAAEM8EAAtBKEEoQZT8wAAQrgIACyAFQShBlPzAABDPBAALQShBKEGU/MAAEK4CAAsCQAJAAkAgAUEIcUUNACAAKAKgASIDQSlPDQECQAJAIAMNAEEAIQMMAQsgA0F/akH/////A3EiAkEBaiIEQQNxIQYCQAJAIAJBA08NAEIAIQcgACECDAELIARB/P///wdxIQRCACEHIAAhAgNAIAIgAjUCAELh6xd+IAd8Igc+AgAgAkEEaiIFIAU1AgBC4esXfiAHQiCIfCIHPgIAIAJBCGoiBSAFNQIAQuHrF34gB0IgiHwiBz4CACACQQxqIgUgBTUCAELh6xd+IAdCIIh8Igg+AgAgCEIgiCEHIAJBEGohAiAEQXxqIgQNAAsLAkAgBkUNAANAIAIgAjUCAELh6xd+IAd8Igg+AgAgAkEEaiECIAhCIIghByAGQX9qIgYNAAsLIAhCgICAgBBUDQAgA0EoRg0DIAAgA0ECdGogB6c2AgAgA0EBaiEDCyAAIAM2AqABCwJAIAFBEHFFDQAgAEHI0sAAQQIQTxoLAkAgAUEgcUUNACAAQdDSwABBAxBPGgsCQCABQcAAcUUNACAAQdzSwABBBRBPGgsCQCABQYABcUUNACAAQfDSwABBChBPGgsCQCABQYACcUUNACAAQZjTwABBExBPGgsgACABEEEaIAAPCyADQShBlPzAABDPBAALQShBKEGU/MAAEK4CAAuxCgECfyMAQdABayIDJAAgA0HIAWogAkGficAAQQ9BFxCxBCADKALMASECAkACQCADKALIASIERQ0AIAMgAjYCxAEgAyAENgLAASADQbgBaiADQcABakGuicAAQQYgAUGMAmoQggICQAJAIAMoArgBQQFxRQ0AIAMoArwBIQIMAQsgA0GwAWogA0HAAWpBtInAAEEFIAEQhgICQCADKAKwAUEBcUUNACADKAK0ASECDAELIANBqAFqIANBwAFqQbmJwABBFyABQQhqEIYCAkAgAygCqAFBAXFFDQAgAygCrAEhAgwBCyADQaABaiADQcABakHQicAAQQUgAUGYAmoQggICQCADKAKgAUEBcUUNACADKAKkASECDAELIANBmAFqIANBwAFqQdWJwABBCyABQcgAahCJAgJAIAMoApgBQQFxRQ0AIAMoApwBIQIMAQsgA0GQAWogA0HAAWpB4InAAEEFIAFBEGoQhgICQCADKAKQAUEBcUUNACADKAKUASECDAELIANBiAFqIANBwAFqQeWJwABBByABQegBahC9AgJAIAMoAogBQQFxRQ0AIAMoAowBIQIMAQsgA0GAAWogA0HAAWpB7InAAEEJIAFBpAJqEIICAkAgAygCgAFBAXFFDQAgAygChAEhAgwBCyADQfgAaiADQcABakH1icAAQQsgAUHYAmoQgwICQCADKAJ4QQFxRQ0AIAMoAnwhAgwBCyADQfAAaiADQcABakGAisAAQQogAUHZAmoQgwICQCADKAJwQQFxRQ0AIAMoAnQhAgwBCyADQegAaiADQcABakGKisAAQQ0gAUHoAGoQiAICQCADKAJoQQFxRQ0AIAMoAmwhAgwBCyADQeAAaiADQcABakGXisAAQQQgAUGIAWoQiQICQCADKAJgQQFxRQ0AIAMoAmQhAgwBCyADQdgAaiADQcABakGbisAAQQogAUEYahCGAgJAIAMoAlhBAXFFDQAgAygCXCECDAELIANB0ABqIANBwAFqQaWKwABBBSABQSBqEIYCAkAgAygCUEEBcUUNACADKAJUIQIMAQsgA0HIAGogA0HAAWpBqorAAEELIAFBsAJqEIICAkAgAygCSEEBcUUNACADKAJMIQIMAQsgA0HAAGogA0HAAWpBtYrAAEELIAFBqAFqEIoCAkAgAygCQEEBcUUNACADKAJEIQIMAQsgA0E4aiADQcABakHAisAAQRwgAUH0AWoQygICQCADKAI4QQFxRQ0AIAMoAjwhAgwBCyADQTBqIANBwAFqQdyKwABBHyABQYACahDJAgJAIAMoAjBBAXFFDQAgAygCNCECDAELIANBKGogA0HAAWpB+4rAAEEEIAFB1AJqEIcCAkAgAygCKEEBcUUNACADKAIsIQIMAQsgA0EgaiADQcABakH/isAAQQQgAUG8AmoQggICQCADKAIgQQFxRQ0AIAMoAiQhAgwBCyADQRhqIANBwAFqQYOLwABBCCABQcgBahCJAgJAIAMoAhhBAXFFDQAgAygCHCECDAELIANBEGogA0HAAWpBi4vAAEEHIAFByAJqEIICAkAgAygCEEEBcUUNACADKAIUIQIMAQsgA0EIaiADQcABakGSi8AAQQwgAUEoahDIAgJAIAMoAghBAXFFDQAg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QQNxIQkCQAJAIAFBBE8NAEEAIQNBACECDAELIAFBfHEhCEEAIQNBACECA0AgAyAAIAJqIgEsAABBv39KaiABQQFqLAAAQb9/SmogAUECaiwAAEG/f0pqIAFBA2osAABBv39KaiEDIAggAkEEaiICRw0ACwsgCUUNACAAIAJqIQEDQCADIAEsAABBv39KaiEDIAFBAWohASAJQX9qIgkNAAsLIAML/AcCCH8BfiMAQZACayICJAAgAkEIaiABEL4BAkACQAJAAkAgAigCCCIDQYCAgIB4ag4CAQACCyAAIAIoAgw2AgQgAEEGOgAADAILIABBADYCDCAAQQA2AgQgAEEFOgAADAELIAIpAgwhCiACQQA2AhwgAkEANgIUIAIgCjcC9AEgAiADNgLwASACQSBqIAEQtAMCQAJAIAItACBBBkYNACACQThqQRBqIAJBIGpBEGopAwA3AwAgAkE4akEIaiACQSBqQQhqKQMANwMAIAIgAikDIDcDOCACQdAAaiACQRRqIAJB8AFqIAJBOGoQqgECQCACLQBQQQZGDQAgAkHQAGoQgwMLIAJB+ABqQQRqIQQgAkHcAGohAyACQfABakEEaiEFAkADQCACQcABaiABEL0BAkACQAJAAkAgAigCwAEiBkGAgICAeGoOAgUAAQsgAigCxAEhAwwBCyACIAIpAsQBIgo3AtABIAIgBjYCzAEgAkHYAWogARC0AyACLQDYAUEGRw0BIAIoAtwBIQMgAkHMAWoQrAUgAkHMAWoQ5wQLIABBBjoAACAAIAM2AgQMBAsgBSACKQPYATcCACAFQRBqIAJB2AFqQRBqKQMANwIAIAVBCGogAkHYAWpBCGopAwA3AgAgAkH4AGpBCGoiByACQfABakEIaikCADcDACACQfgAakEQaiIIIAJB8AFqQRBqKQIANwMAIAJB+ABqQRhqIgkgAkHwAWpBGGooAgA2AgAgAiACKQLwATcDeCADQRhqIAkoAgA2AgAgA0EQaiAIKQMANwIAIANBCGogBykDADcCACADIAIpA3g3AgAgAiAKpyIINgJUIAIgBjYCUCACQdAAakEIaiAKQiCIpyIHNgIAIAJBmAFqQQhqIAc2AgAgAiACKQNQNwOYASACQagBakEQaiAEQRBqKQIANwMAIAJBqAFqQQhqIARBCGopAgA3AwAgAiAEKQIANwOoASACQfABaiACQRRqIAJBmAFqIAJBqAFqEKoBIAItAPABQQZGDQAgAkHwAWoQgwMMAAsLIAIgBzYCWCACIAg2AlQgAkGAgICAeDYCUCACQdAAahDTAyACQdsAaiACQRRqQQhqKAIANgAAIABBBToAACACIAIpAhQ3AFMgACACKQBQNwABIABBCGogAkHXAGopAAA3AAAMAgsgACACKQMgNwMAIABBEGogAkEgakEQaikDADcDACAAQQhqIAJBIGpBCGopAwA3AwAgAkHwAWoQrAUgAkHwAWoQ5wQLIAJBFGoQmwILAkAgASgCCCIAQYQBSQ0AIAAQjwMLAkAgASgCAEUNACABKAIEIgFBhAFJDQAgARCPAwsgAkGQAmokAAuHCAEJfyMAQaABayIDJAACQAJAAkACQAJAIAINACADQQhqEOIDIAMoAggiBEUNASADKAIMDQQgA0EANgI4IAMgBDYCNEEAIQUCQCABLwGSA0UNACABQYwCaiEGIANByABqQQhqIQcgA0HIAGpBBHIhCCABIQlBACEFA0AgA0E8aiAGEKUCAkACQAJAAkACQAJAAkAgCS0AAA4GAAECAwQFAAsgA0EAOgBIDAULIANByABqQRBqIAlBEGopAwA3AwAgByAJQQhqKQMANwMAIAMgCSkDADcDSAwECyAHQQhqIAlBEGopAwA3AwAgByAJQQhqKQMANwMAIANBAjoASAwDCyAIIAlBBGoQpQIgA0EDOgBIDAILIAggCUEEakGQmsAAEGggA0EEOgBIDAELIAggCUEEahCNAyADQQU6AEgLIAZBDGohBiAJQRhqIQkgA0GUAWogA0E0aiADQTxqIANByABqEPwBIAVBAWoiBSABLwGSA0kNAAsLIAAgBTYCCCAAQQA2AgQgACAENgIADAMLIANBADYCUCADIAI2AkwgAyABNgJIIANBKGogA0HIAGoQugMgA0GUAWogAygCKCADKAIsEEggAygClAEiCUUNASADQSBqIAkgAygCmAEQlgMgAyADKAIkIgk2ApgBIAMgAygCICIGNgKUASADIAk2AkAgAyAGNgI8AkAgAS8BkgNFDQAgAUGMAmohBiADQfAAakEIaiEKIANB8ABqQQRyIQsgASEJQQAhBQNAIANB5ABqIAYQpQICQAJAAkACQAJAAkACQCAJLQAADgYAAQIDBAUACyAD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ABEgAEEIaiAELwEAOwEADAkLIABBgAQ7AQogAEIANwECIABB3OgBOwEADAgLIABBgAQ7AQogAEIANwECIABB3OQBOwEADAcLIABBgAQ7AQogAEIANwECIABB3NwBOwEADAYLIABBgAQ7AQogAEIANwECIABB3LgBOwEADAULIABBgAQ7AQogAEIANwECIABB3OAAOwEADAQLIAJBgAJxRQ0BIABBgAQ7AQogAEIANwECIABB3M4AOwEADAMLIAJB////B3FBgIAETw0BCwJAIAEQxAENACADQQA6ABYgA0EAOwEUIAMgAUEUdkG+6MAAai0AADoAFyADIAFBBHZBD3FBvujAAGotAAA6ABsgAyABQQh2QQ9xQb7owABqLQAAOgAaIAMgAUEMdkEPcUG+6MAAai0AADoAGSADIAFBEHZBD3FBvujAAGotAAA6ABggA0EUaiABQQFyZ0ECdiICaiIEQfsAOgAAIARBf2pB9QA6AAAgA0EUaiACQX5qIgJqQdwAOgAAIANBFGpBCGoiBCABQQ9xQb7owABqLQAAOgAAIABBCjoACyAAIAI6AAogACADKQIUNwIAIANB/QA6AB0gAEEIaiAELwEAOwEADAILIAAgATYCBCAAQYABOgAADAELIABBgAQ7AQogAEIANwECIABB3MQAOwEACyADQSBqJAAL9AUCCH8BfgJAAkAgAQ0AIAVBAWohBiAAKAIIIQdBLSEIDAELQStBgIDEACAAKAIIIgdBgICAAXEiARshCCABQRV2IAVqIQYLAkACQCAHQYCAgARxDQBBACECDAELAkACQCADQRBJDQAgAiADEEYhAQwBCwJAIAMNAEEAIQEMAQsgA0EDcSEJAkACQCADQQRPDQBBACEBQQAhCgwBCyADQQxxIQtBACEBQQAhCgNAIAEgAiAKaiIMLAAAQb9/SmogDEEBaiwAAEG/f0pqIAxBAmosAABBv39KaiAMQQNqLAAAQb9/SmohASALIApBBGoiCkcNAAsLIAlFDQAgAiAKaiEMA0AgASAMLAAAQb9/SmohASAMQQFqIQwgCUF/aiIJDQALCyABIAZqIQYLAkACQCAGIAAvAQwiC08NAAJAAkACQCAHQYCAgAhxDQAgCyAGayENQQAhAUEAIQsCQAJAAkAgB0EddkEDcQ4EAgABAAILIA0hCwwBCyANQf7/A3FB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BSACQdAAahDnBAsgAkEUahCbAgsgASgCAEUNAiABKAIEIgFBgwFLDQEMAgsgASgCAEUNASABKAIEIgFBgwFNDQELIAEQjwMLIAJB4AFqJAALswUCC38EfiMAQTBrIgQkACABQRBqIgUgAhBnIQ8CQCABKAIIDQAgBCABQQEgBUEBEFALIAEoAgQiBiAPp3EhBSAPQhmIIhBC/wCDQoGChIiQoMCAAX4hESACKAIIIQcgAigCBCEIIAEoAgAhCUEAIQpBACELAkACQANAAkAgCSAFaikAACISIBGFIg9Cf4UgD0L//fv379+//358g0KAgYKEiJCgwIB/gyIPUA0AA0AgCCAHIAEoAgBBACAPeqdBA3YgBWogBnFrIgxBKGxqIg1BXGooAgAgDUFgaigCABDfAw0DIA9Cf3wgD4MiD1BFDQALCyASQoCBgoSIkKDAgH+DIQ9BASENAkAgC0EBRg0AIA9CAFIhDSAPeqdBA3YgBWogBnEhDgsCQCAPIBJCAYaDQgBSDQAgBSAKQQhqIgpqIAZxIQUgDSELDAELCwJAIAkgDmosAABBAEgNACAJKQMAQoCBgoSIkKDAgH+DeqdBA3YhDgsgASgCACIFIA5qIgktAAAhBiACQQhqKAIAIQcgAikCACEPIAkgEKdB/wBxIgI6AAAgBEEIakEIaiAHNgIAIARBIGogA0EIaikDADcDACAEQShqIANBEGopAwA3AwAgASABKAIMQQFqNgIMIAUgASgCBCAOQXhqcWpBCGogAjoAACABIAEoAgggBkEBcWs2AgggBCAPNwMIIAQgAykDADcDGAJAQShFDQAgBUEAIA5rQShsakFYaiAEQQhqQSj8CgAACyAAQQY6AAAMAQsgACABKAIAIAxBKGxqQWhqIgEpAwA3AwAgASADKQMANwMAIABBCGogAUEIaiIFKQMANwMAIABBEGogAUEQaiIBKQMANwMAIAUgA0EIaikDADcDACABIANBEGopAwA3AwAgAhCsBSACEOcECyAEQTBqJAAL7AUCBH8FfiMAQYABayIEJAAgAb0hCAJAAkAgAZlEAAAAAAAA8H9iDQBBAyEFDAELAkAgCEKAgICAgICA+P8AgyIJQoCAgICAgID4/wBSDQBBAiEFDAELIAhC/////////weDIgpCgICAgICAgAiEIAhCAYZC/v///////w+D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AkACQAJAIAAoAgAiBEGBARDWBA0AQQAhBUEBQQIgBBCKBSIGQQFGG0EAIAYbIgZBAkYNAUEAIQRBACEHDAILIANBBzoAcCADQfAAaiABIAIQugIhAAwCCyADQRhqIAQQ+AQgA0HIAGogAygCGCADKwMgEMgDAkAgAygCSA0AIANBEGogBBD5BAJAAkAgAygCECIERQ0AIANBCGogBCADKAIUEPsCIANB2ABqIAMoAgggAygCDBCDBCADKAJYQYCAgIB4Rg0AIANBKGpBCGogA0HYAGpBCGooAgAiCDYCACADIAMpAlg3AyhBBSEHQQAhBEEBIQUgAygCLCEADAELIANB5ABqIAAQ7QECQAJAIAMoAmQiBEGAgICAeEYiBQ0AIANBOGpBCGohCCADQThqQQRqIQkgA0E4akEIaiADQeQAakEIaigCADYCACADIAMpAmQ3AzhBBiEHDAELIANBKGpBCGohCCADQShqQQRqIQkgA0EBNgJ0IANB6J3AADYCcCADQgE3AnwgA0EaNgKMASADIAA2AogBIAMgA0GIAWo2AnggA0EoaiADQfAAahB1QREhBwsgBEGAgICAeEchBCAIKAIAIQggCSgCACEACyAIrSEKDAELQQMhB0EAIQUgAykDUCEKQQAhBAsgAyAKNwN4IAMgADYCdCADIAY6AHEgAyAHOgBwIANB8ABqIAEgAhC6AiEAAkAgBEUNACADQThqEK0FIANBOGoQ7gQLIAVFDQAgA0EoahCtBSADQShqEO4ECyADQZABaiQAIAAL2QQBCH8jAEEQayIDJAAgAyABNgIEIAMgADYCACADQqCAgIAONwIIAkACQAJAAkACQCACKAIQIgRFDQAgAigCFCIBDQEMAgsgAigCDCIARQ0BIAIoAggiASAAQQN0aiEFIABBf2pB/////wFxQQFqIQYgAigCACEAA0ACQCAAQQRqKAIAIgdFDQAgAygCACAAKAIAIAcgAygCBCgCDBENAEUNAEEBIQEMBQsCQCABKAIAIAMgAUEEaigCABEJAEUNAEEBIQEMBQsgAEEIaiEAIAFBCGoiASAFRg0DDAALCyABQRhsIQggAUF/akH/////AXFBAWohBiACKAIIIQkgAigCACEAQQAhBwNAAkAgAEEEaigCACIBRQ0AIAMoAgAgACgCACABIAMoAgQoAgwRDQBFDQBB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ajYCKCACIAJBOGo2AiAgAiACQSBqNgIQIAEoAgAgASgCBCACQQhqEGIhACACQThqQQFBARCcAiACQeAAaiQAIAAL7wECBH8DfiMAQTBrIgIkAAJAAkAgAQ0AQQAhAUHYncAAIQMMAQsgASgCACEDIAFBADYCACABQQhqQdidwAAgA0EBcSIEGyEDIAEoAgRBACAEGyEBCyACQQhqQQhqIANBCGopAgAiBjcDACACIAMpAgAiBzcDCCACQRhqQRBqIABBEGoiAykCADcDACACQRhqQQhqIgUgAEEIaiIEKQIANwMAIAApAgAhCCAAIAE2AgQgAEEBNgIAIAQgBzcCACADIAY3AgAgAiAINwMYIABBBGohAAJAIAinRQ0AIAUgAEEMQQgQ4wILIAJBMGokACAAC/IBAQN/IwBBwABrIgIkACACQTBqIAEQuwICQAJAIAIoAjAiASgCjAJBgICAgHhHDQAgAkEwahCSA0EAIQFBACEDDAELIAJBJGogAUGMAmoQpQIgAigCJCEEIAJBMGoQkgNBACEBQQAhAyAEQYCAgIB4Rg0AIAJBMGpBCGogAkEkakEIaigCADYCACACIAIpAiQ3AzAgAkEYaiACQTBqQbiHwAAQqwIgAkEQaiACKAIYIAIoAhwQtAQgAigCFCEBIAIoAhAhAwsgAkEIaiADIAEQswQgAigCDCEBIAAgAigCCDYCACAAIAE2AgQgAkHAAGokAAvwAQEDfyMAQcAAayICJAAgAkEwaiABELsCAkACQCACKAIwIgEoAiRBgICAgHhHDQAgAkEwahCRA0EAIQFBACEDDAELIAJBJGogAUEkahClAiACKAIkIQQgAkEwahCRA0EAIQFBACEDIARBgICAgHhGDQAgAkEwakEIaiACQSRqQQhqKAIANgIAIAIgAikCJDcDMCACQRhqIAJBMGpBuIfAABCrAiACQRBqIAIoAhggAigCHBC0BCACKAIUIQEgAigCECEDCyACQQhqIAMgARCzBCACKAIMIQEgACACKAIINgIAIAAgATYCBCACQcAAaiQAC/ABAQN/IwBBwABrIgIkACACQTBqIAEQuwICQAJAIAIoAjAiASgCMEGAgICAeEcNACACQTBqEJEDQQAhAUEAIQMMAQsgAkEkaiABQTBqEKUCIAIoAiQhBCACQTBqEJEDQQAhAUEAIQMgBEGA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AgAMAQsgAkEcaiACQS9qQciBwAAQYSEBIABBgICAgHg2AgAgACABNgIECwJAIAIoAhwiAUGEAUkNACABEI8DCyACQTBqJAALwwEBBH8jAEEgayICJAAgAkEIaiABELICAkACQCACKAIIQQFHDQAgAigCECEDIAIoAgwhBAJAIAEoAgBFDQAgASgCBCIFQYQBSQ0AIAUQjwMLIAEgAzYCBCABQQE2AgAgAkEUaiAEENgBAkAgAigCFEGAgICAeEcNACAAIAIoAhg2AgQgAEGBgICAeDYCAAwCCyAAIAIpAhQ3AgAgAEEIaiACQRRqQQhqKAIANgIADAELIABBgICAgHg2AgALIAJBIGokAAvFAQEBfyMAQSBrIgQkAAJAAkACQAJAAkACQCAALQAADgQBAAUCAQsgAUUNAgsgAEECOgAAIAIoAgAiAi0AACEBIAJBADoAACABRQ0CQQFBvJTAABDlASAAQQM6AAALIARBIGokAA8LIARBADYCGCAEQQE2AgwgBEGAlcAANgIIIARCBDcCECAEQQhqIAMQqAMAC0GclsAAENIEAAsgBEEANgIYIARBATYCDCAEQcCVwAA2AgggBEIENwIQIARBCGogAxCoAwALqgEBAn8jAEEgayICJAACQAJAIAEoAgBFDQAgAiABEPUCIAIoAgBBAXFFDQAgAigCBCEDIAEgASgCDEEBajYCDCACQQhqIAMQSQJAIAItAAhBBkcNACAAIAIoAgw2AgQgAEEHOgAADAILIAAgAikDCDcDACAAQRBqIAJBCGpBEGopAwA3AwAgAEEIaiACQQhqQQhqKQMANwMADAELIABBBjoAAAsgAkEgaiQAC7MBAQJ/IwBB0AVrIgIkACACQQhqIAFBn4nAAEEPQaCLwABBFxAzIAIoAgwhAQJAAkAgAigCCCIDQQJHDQBBASEDDAELAkBB2AJFDQAgAkH4AmogAkEIakEIakHYAvwKAAALIAIgATYC9AIgAiADNgLwAkEAIQMgAkEANgLoAiACQegCahD9AkEIaiEBCyAAIAM2AgggACABQQAgAxs2AgQgAEEAIAEgAxs2AgAgAkHQBWokAAuzAQEBfyMAQSBrIgMkACADQQhqIAAQzgIgAygCCCEAAkACQCABRQ0AIAMgASACEPsCIANBFGogAygCACADKAIEEIMEDAELIANBgICAgHg2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IgAgACgCAEF/aiIANgIAAkAgAA0AIANBCGpBCGoQkwMLEKsFIANBIGokAAuiAQEBfyMAQSBrIgMkACADQQhqIAAQzgIgAygCCCEAIAMgASACEPsCIANBFGogAygCACADKAIEEIMEIABBhAFqIgIQrAUgAhDnBCAAQYwBaiADQRRqQQhqKAIANgIAIAAgAykCFDcChAEgAygCDEEANgIAIAMoAhAiACAAKAIAQX9qIgA2AgACQCAADQAgA0EIakEIahCTAwsQqwUgA0EgaiQAC6IBAQF/IwBBIGsiAyQAIANBCGogABDOAiADKAIIIQAgAyABIAIQ+wIgA0EUaiADKAIAIAMoAgQQgwQgAEGQAWoiAhCsBSACEOcEIABBmAFqIANBFGpBCGooAgA2AgAgACADKQIUNwKQASADKAIMQQA2AgAgAygCECIAIAAoAgBBf2oiADYCAAJAIAANACADQQhqQQhqEJMDCxCrBSADQSBqJAALogEBAX8jAEEgayIDJAAgA0EIaiAAEM4CIAMoAgghACADIAEgAhD7AiADQRRqIAMoAgAgAygCBBCDBCAAQZwBaiICEKwFIAIQ5wQgAEGkAWogA0EUakEIaigCADYCACAAIAMpAhQ3ApwBIAMoAgxBADYCACADKAIQIgAgACgCAEF/aiIANgIAAkAgAA0AIANBCGpBCGoQkwMLEKsFIANBIGokAAufAQEEfyMAQRBrIgIkAEEBIQMCQCABKAIAIgRBJyABKAIEIgUoAhAiAREJAA0AIAJBBGogACgCAEGBAhBRAkACQCACLQAEQYABRw0AIAQgAigCCCABEQkARQ0BQQEhAwwCCyAEIAJBBGogAi0ADiIDaiACLQAPIANrIAUoAgwRDQBFDQBBASEDDAELIARBJyABEQkAIQMLIAJBEGokACADC6IBAgJ/AX4jAEEwayICJAAgAkEIaiABEKsDQQEhAwJAAkACQCACKAIIQQFHDQAgAikDECIEQn9VDQELIAEgAkEvakGQgMAAEGEhAQwBCwJAIARCgICAgBBUDQBBASEDIAJBAToAGCACIAQ3AyAgAkEYaiACQS9qQZCAwAAQuQIhAQwBCyAEpyEBQQAhAwsgACABNgIEIAAgAzYCACACQTBqJAALpwEBAn8jAEEgayICJAACQAJAIAEoAgBFDQAgAkEIaiABEPUCIAIoAghBAXFFDQAgAigCDCEDIAEgASgCDEEBajYCDCACQRRqIAMQ1wECQCACKAIUQYCAgIB4Rw0AIAAgAigCGDYCBCAAQYGAgIB4NgIADAILIAAgAikCFDcCACAAQQhqIAJBFGpBCGooAgA2AgAMAQsgAEGAgICAeDYCAAsgAkEgaiQAC60BAQJ/IwBBMGsiAyQAIAMgAjcDCAJAAkAgAS0AAg0AAkAgAkL/////////D3xC/////////x9UDQAgA0ECNgIUIANBnJ7AADYCECADQgE3AhwgA0EbNgIsIAMgA0EoajYCGCADIANBCGo2AiggA0EQahC7ASEBQQEhBAwCC0EAIQQgArkQmgQhAQwBC0EAIQQgAhCYBCEBCyAAIAE2AgQgACAENgIAIANBMGokAAuVAQEFfyMAQRBrIgMkAAJAAkACQCACQQdLDQAgAg0BQQAhBAwCCyADQQhqQS4gASACEJsBIAMoAghBAUYhBAwBCyACQX9qIQUgASEGA0AgBSEHIAYtAABBLkYiBA0BIAZBAWohBiAHQX9qIQUgBw0ACwsgACAEIAAtAARyOgAEIAAoAgAgASACEJcEIQYgA0EQaiQAIAYLogEBAX8CQCAAKAIAIgBFDQAgAEEkahD5AyAAIABBKEEIEOECIABB+ABqIgEQrAUgARDnBCAAQTBqEPkDIABBPGoQ+QMgAEGEAWoiARCsBSABEOcEIABBkAFqIgEQrAUgARDnBCAAQcgAahD5AyAAQZwBaiIBEKwFIAEQ5wQgAEHUAGoQ+QMgAEHgAGoQ+QMgAEHsAGoQ+QMgAEGwAUEIEMUECwusAQECfyMAQSBrIgIkACACQQhqIAEQhQMCQAJAAkAgAigCCCIBQQJGDQAgAigCDCEDIAFBAXFFDQEgAEGBgICAeDYCACAAIAM2AgQMAgsgAEGAgICAeDYCAAwBCyACQRRqIAMQ1wECQCACKAIUIgFBgICAgHhGDQAgACACKQIYNwIEIAAgATYCAAwBCyACKAIYIQEgAEGBgICAeDYCACAAIAE2AgQLIAJBIGokAAuiAQEDfwJAIAEoAgAiBC8BkgMiBUELSQ0AQeylwABBIEHopsAAEP8CAAsgBCAFQQFqOwGSAyAEIAVBDGxqIgYgAikCADcCjAIgBkGUAmogAkEIaigCADYCACAAIAU2AgggACAENgIAIAAgASgCBDYCBCAEIAVBGGxqIgQgAykDADcDACAEQQhqIANBCGopAwA3AwAgBEEQaiADQRBqKQMANwMAC7QBAQN/IwBBEGsiASQAIAAoAgAiAigCDCEDAkACQAJAAkAgAigCBA4CAAECCyADDQFBASECQQAhAwwCCyADDQAgAigCACICKAIEIQMgAigCACECDAELIAFBgICAgHg2AgAgASAANgIMIAFBxMXAACAAKAIEIAAoAggiAC0ACCAALQAJEL8BAAsgASADNgIEIAEgAjYCACABQajFwAAgACgCBCAAKAIIIgAtAAggAC0ACRC/AQALowEBAn8jAEEwayIDJAAgAyACNwMIAkACQCABLQACDQACQCACQoCAgICAgIAQVA0AIANBAjYCFCADQZyewAA2AhAgA0IBNwIcIANBHDYCLCADIANBKGo2AhggAyADQQhqNgIoIANBEGoQuwEhAUEBIQQMAgtBACEEIAK6EJoEIQEMAQtBACEEIAIQmQQhAQsgACABNgIEIAAgBDYCACADQTBqJAALnAEBA38jAEEQayIDJAACQAJAIAEtAABFDQBBAiECDAELIAIoAgAQhwQhBCADQQhqELYDQQEhAgJAIAMoAghBAXFFDQAgAygCDCEFIAFBAToAAAwBCwJAAkAgBBCQBQ0AQQAhAiAEEIgEIQUMAQsgAUEBOgAAQQIhAgsgBEGEAUkNACAEEI8DCyAAIAU2AgQgACACNgIAIANBEGokAAucAQEBfyMAQcAAayICJAAgAkIANwM4IAJBOGogACgCABD8BCACIAIoAjwiADYCNCACIAIoAjg2AjAgAiAANgIsIAJBwQA2AiggAkECNgIQIAJBrLfAADYCDCACQgE3AhggAiACQSxqNgIkIAIgAkEkajYCFCABKAIAIAEoAgQgAkEMahBiIQEgAkEsakEBQQEQnQIgAkHAAGokACABC5sBAgJ/AX4jAEEwayICJAAgAkEIaiABEKsDQQEhAwJAAkAgAigCCEEBRw0AAkAgAikDECIEQoCAgIAIfEKAgICAEFQNACACQQI6ABggAiAENwMgIAJBGGogAkEvakGAgMAAELkCIQEMAgsgBKchAUEAIQMMAQsgASACQS9qQYCAwAAQYSEBCyAAIAE2AgQgACADNgIAIAJBMGokAAuZAQEDfyMAQRBrIgUkACABKAIAIQYC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aiABELsCIAJBJGogAigCGEHoAWoQpQIgAkEYahCSAyACQRBqIAJBJGpBuIfAABCrAiACQQhqIAIoAhAgAigCFBC0BCACIAIoAgggAigCDBCzBCACKAIEIQEgACACKAIANgIAIAAgATYCBCACQTBqJAALggEBAX8jAEEwayICJAAgAkEYaiABELsCIAJBJGogAigCGEH4AGoQpQIgAkEYahCRAyACQRBqIAJBJGpBuIfAABCrAiACQQhqIAIoAhAgAigCFBC0BCACIAIoAgggAigCDBCzBCACKAIEIQEgACACKAIANgIAIAAgATYCBCACQTBqJAALggEBAX8jAEEwayICJAAgAkEYaiABELsCIAJBJGogAigCGEGEAWoQpQIgAkEYahCRAyACQRBqIAJBJGpBuIfAABCrAiACQQhqIAIoAhAgAigCFBC0BCACIAIoAgggAigCDBCzBCACKAIEIQEgACACKAIANgIAIAAgATYCBCACQTBqJAALggEBAX8jAEEwayICJAAgAkEYaiABELsCIAJBJGogAigCGEGQAWoQpQIgAkEYahCRAyACQRBqIAJBJGpBuIfAABCrAiACQQhqIAIoAhAgAigCFBC0BCACIAIoAgggAigCDBCzBCACKAIEIQEgACACKAIANgIAIAAgATYCBCACQTBqJAALggEBAX8jAEEwayICJAAgAkEYaiABELsCIAJBJGogAigCGEGcAWoQpQIgAkEYahCRAyACQRBqIAJBJGpBuIfAABCrAiACQQhqIAIoAhAgAigCFBC0BCACIAIoAgggAigCDBCzBCACKAIEIQEgACACKAIANgIAIAAgATYCBCACQTBqJAALfgECfyMAQSBrIgMkACADQRRqIAEQzgIgA0EIaiADKAIUIAIQygEgAygCDCECIAMoAgghASADKAIYQQA2AgAgAygCHCIEIAQoAgBBf2oiBDYCAAJAIAQNACADQRxqEJMDCyAAIAE2AgQgACACQQAgAUEBcRs2AgAgA0EgaiQAC4QBAQJ/IwBBMGsiASQAAkACQCAAKAIAIgINAEEAIQBBACECDAELIAEgAjYCJCABQQA2AiAgASACNgIUIAFBADYCECABIAAoAgQiAjYCKCABIAI2AhggACgCCCECQQEhAAsgASACNgIsIAEgADYCHCABIAA2AgwgAUEMahC0AiABQTBq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EGsiAiQAIAJBATsBDCACIAE2AgggAiAANgIEIAJBBGoQpwMACywBAX8jAEEQayICJAAgAiAAIAEQ6QMgAigCACACKAIEIAIoAgggAkEQaiQACzIBAX8gASgCACECIAFBADYCAAJAIAINAEHpksAAQSxB+JPAABDNAgALIAAgASgCBBBJCy0BAX5CACECAkAgARDLBEUNACAAIAEoAgAQjQX8BjcDCEIBIQILIAAgAjcDAAs6AQF+QgMhAgJAIAG9Qv///////////wCDQv/////////3/wBVDQAgACABOQMIQgIhAgsgACACNwMACy0AIAEoAgAgAiADIAEoAgQoAgwRDQAhAyAAQQA6AAUgACADOgAEIAAgATYCAAsqAQF/IwBBEGsiASQAIAEgABCpBCABKAIAIAEoAgQgASgCCCABQRBqJAALKgEBfyMAQRBrIgEkACABIAAQoQIgASgCACABKAIEIAEoAgggAUEQaiQACyoBAX8jAEEQayIBJAAgASAAEKoEIAEoAgAgASgCBCABKAIIIAFBEGokAAsqAQF/IwBBEGsiASQAIAEgABCiAiABKAIAIAEoAgQgASgCCCABQRBqJAALKgEBfyMAQRBrIgEkACABIAAQowIgASgCACABKAIEIAEoAgggAUEQaiQACywAAkAgACgCAEGAgICAeEYNACAAEKwFIAAQ5wQgAEEMaiIAEKwFIAAQ5wQLCy0BAX8gASgCACECIAFBADYCAAJAIAINAEG4ksAAQTEQ7QQACyAAIAEoAgQQSQssACAAEKwFIAAQ5wQCQCAAKAIMQYCAgIB4Rg0AIABBDGoiABCsBSAAEOcECwsrAQF+QQApA/D/QCEBQQBCADcD8P9AIAAgAUIgiD4CBCAAIAGnQQFGNgIACyIBAX8gACgCACIAIABBH3UiAnMgAmsgAEF/c0EfdiABEHoLJwEBfyMAQRBrIgIkACACIAAgARCcBCACKAIAIAIoAgQgAkEQaiQACyQAIAAgAjYCCCAAIAE2AhAgAEEANgIAIAAgAiADQQN0ajYCDAsmACAAIAEoAgRBf2o2AgQgACABKAIAIAEoAghBAnRqKAKYAzYCAAslAQF/IwBBEGsiASQAIAEgABCVAiABKAIAIAEoAgQgAUEQaiQACyUBAX8jAEEQayIBJAAgASAAELEBIAEoAgAgASgCBCABQRBqJAALJQEBfyMAQRBrIgEkACABIAAQsgEgASgCACABKAIEIAFBEGokAAslAQF/IwBBEGsiASQAIAEgABCWAiABKAIAIAEoAgQgAUEQaiQACyUBAX8jAEEQayIBJAAgASAAELMBIAEoAgAgASgCBCABQRBqJAALJQEBfyMAQRBrIgEkACABIAAQtAEgASgCACABKAIEIAFBEGokAAslAQF/IwBBEGsiASQAIAEgABCXAiABKAIAIAEoAgQgAUEQaiQACyUBAX8jAEEQayIBJAAgASAAEJgCIAEoAgAgASgCBCABQRBqJAALJQEBfyMAQRBrIgEkACABIAAQtQEgASgCACABKAIEIAFBEGokAAslAQF/IwBBEGsiASQAIAEgABCZAiABKAIAIAEoAgQgAUEQaiQACyUBAX8jAEEQayIBJAAgASAAELYBIAEoAgAgASgCBCABQRBqJAALJQEBfyMAQRBrIgEkACABIAAQtwEgASgCACABKAIEIAFBEGokAAslAQF/IwBBEGsiASQAIAEgABC4ASABKAIAIAEoAgQgAUEQaiQACyYBAX4CQAJAIAENAEIAIQMMAQsgACACOQMIQgEhAwsgACADNwMACyYAAkAgAA0AQZihwABBMhDtBAALIAAgAiADIAQgBSABKAIQERIACx8BAn4gACkDACICIAJCP4ciA4UgA30gAkJ/VSABEHkLJQACQCAAKAIAQYCAgIB4Rg0AIAAQrAUgABDnBCAAQRBqEIEDCwslAAJAIAAoAgBBgICAgHhGDQAgABCsBSAAEOcEIABBDGoQ+QMLCx0AIAAoAgAoAgAgASgCAEEAIAJrQShsakFYahBnCx0AIAAoAgAoAgAgASgCAEEAIAJrQRhsakFoahBnCx0AIAAoAgAoAgAgASgCAEEAIAJrQRhsakFoahBnCx0AIAAoAgAoAgAgASgCAEEAIAJrQRhsakFoahBnCx0AIAAoAgAoAgAgASgCAEEAIAJrQRhsakFoahBnCx0AIAAoAgAoAgAgASgCAEEAIAJrQShsakFYahBnCyUAAkAgACgCAEGAgICAeEYNACAAEKwFIAAQ5wQgAEEQahCDAwsLJAACQCAADQBBmKHAAEEyEO0EAAsgACACIAMgBCABKAIQEQ8ACyQAAkAgAA0AQZihwABBMhDtBAALIAAgAiADIAQgASgCEBEP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ENsCQQALEwACQCABRQ0AIAAgASACEMUECwsSAEEAIACtQiCGQgGENwPw/0ALFgAgACABEKoFGkGAgICAeCABEPEDAAsZACABKAIAQdjowABBDiABKAIEKAIMEQ0ACxYAIAAoAgAgASACIAAoAgQoAgwRDQALFwIBfwFvIAAQBSECEJwFIgEgAiYBIAELFwIBfwFvIAAQCSECEJwFIgEgAiYBIAELFwIBfwFvIAAQDSECEJwFIgEgAiYBIAELEwAgACUBIAEgAiUBIAIQjwMQHwsWAQF/IAAgARCcBSIDIAImASADEJoCCxMAIAAgASgCBDYCBCAAQQA2AgALEwAgACACEJoENgIEIABBADYCAAsUACAAKAIAIAEoAgAgAigCABDlAwsUACAAKAIAIAEgACgCBCgCDBEJAAsUACAAKAIAIAEgACgCBCgCEBEJAAsPACAAIAEgAiADIAQQQAALFQIBfwFvEBMhARCcBSIAIAEmASAACxUCAX8BbxAVIQEQnAUiACABJgEgAAsVAgF/AW8QGiEBEJwFIgAgASYBIAALFQIBfwFvEB0hARCcBSIAIAEmASAACxUCAX8BbxArIQEQnAUiACABJgEgAAsVAgF/AW8QMSEBEJwFIgAgASYBIAALFAEBfyAAEJwFIgIgASYBIAIQ3AELFAEBfyAAEJwFIgIgASYBIAIQyQELEAEBfyAAIAEQ8gMhAiACDwsQAQF/IAAgARDqAiECIAIPCxEAIAAoAgAgACgCBCABEM4ECxAAIAAgAjYCBCAAIAE2AgALEAAgACgCAEGBARDWBEEARwsQACAAIAI2AgQgAEEANgIACxEAIAAQogU2AgQgACABNgIACxIAIAAoAgAgASgCABD1AxCkBQsQACAAIAI2AgQgACABNgIACxAAIAAgAjYCBCAAIAE2AgALEAAgACgCBCAAKAIIIAEQRQsRACAAKAIAIAAoAgQgARDOBAsRACAAKAIEIAAoAgggARD3BAsQACAAIAI2AgQgACABNgIACyIAIABC7bqtts2F1PXjADcDCCAAQviCmb2V7sbFuX83AwALIQAgAEKF47Hbz661/OwANwMIIABC79fH8LmD1ZUmNwMACxEAIAAoAgAgACgCBCABEPcECxEAIAAoAgQgACgCCCABEPcECxMAIABBmMXAADYCBCAAIAE2AgALEQAgASAAKAIAIAAoAgQQlwQLEAAgACgCACAAKAIEIAEQRQsRACAAKAIAIAAoAgQgARD3BAsQACABIAAoAgAgACgCBBBcCxAAIAEoAgAgASgCBCAAEGILEQEBfxCcBSIBIAAlASYBIAELDgAgACUBIAElASACECgLDAAgACABIAIQxgIPCw4AIAAoAgAgASACEPYDCw4AIAAoAgAgASACEJsECw0AIAAoAgAQkgVBAEcLDQAgACgCABCTBUEARwsNACAAKAIAEJQFQQBHCw0AIAAoAgAQlQVBAEcLDQAgACgCABCXBUEARwsNACAAKAIAEJEFQQFGCw4AIAAgAiABKAIMEQkACwwAIAAgASACELACAAsMACAAIAEgAhCxAgALDQAgACgCAEEBIAEQegsPAEGA6cAAQSsgABD/AgALDQAgACkDAEEBIAEQeQsMACAAJQEgASUBEAELDAAgACUBIAElARAGCwwAIAAlASABJQEQCwsOACABQdiMwABBFhCXBAsOACABQciPwABBERCXBAsOACABQciXwABBBRCXBAsOACABQbyWwABBChCXBAsKACAAIAEQ/gMPCwwAIAAoAgAgARCABAsNACAAQcyiwAAgARBiCwwAIAAQrQUgABDuBAsOACABQYikwABBBRCXBAsLACAAQQFBARCcAgsOACABQZSrwABBBRCXBAsNACAAQYiywAAgARBiCwsAIABBAUEBEJwCCw4AIAFBoLLAAEEUEJcECw4AIAFBtLLAAEEMEJcECwsAIABBCEEYEJwCCwsAIABBAUEBEJwCCw0AIABBoLPAACABEGILDgAgAUG4s8AAQQkQlwQLDgAgAUHBs8AAQQgQlwQLDgAgAUHOtcAAQQMQlwQLDgAgAUHRtcAAQQMQlwQLCQAgACABEDAACwsAIABBAUEBEJ0CCwsAIABBBEEMEJ0CCw0AQau4wABBGxDtBAALDgBBxrjAAEHPABDtBAALDQAgAEHAusAAIAEQYgsMACAAIAEpAgA3AwALDQAgAEGsycAAIAEQYgsOACABQaTJwABBBRCXBAsKACABIAAQ2wQACwoAIAIgACABEFwLCgAgACABJQEQBAsKACAAIAElARAHCwoAIAAgASUBECwLCgAgACABJQEQLgsKACAAIAElARAvCwkAIAAgARDkBAsJACAAIAEQ5QQLCQAgACABEOwECwkAIAAgARDrBAsJACAAIAEQ6QQLCQAgACABEOoECwkAIAAgARCFBAsKACAAKAIAEIkECwoAIAAoAgAQiwQLCQAgAEEANgIACwgAIAAQ/QEACwoAQQAoAtCDQUULCAAgACUBEAALCAAgACUBEAILCAAgACUBEAMLCAAgACUBEAgLCAAgACUBEAwLCAAgACUBEBILCAAgACUBEBQLCAAgACUBEBgLCAAgACUBEB4LCAAgACUBECALCAAgACUBECELCAAgACUBECILCAAgACUBECQLCAAgACUBECkLCAAgACUBECoLBwAgARCXAQsHACABEJcBCwcAIAEQlwELBwAgABC7AQsKAEGw/8AAENABCwYAEKkFAAsHABAyEPQCCwUAEKsFCwUAEKMECwUAEKQECwUAEKYECwUAEKUECwQAIAALBAAgAAsEACAACwQAIAALBQAQqAQLAwAACwMAAAsCAAsCAAsCAAsL3n8CAEGAgMAAC7B/AAAAAAAAAAABAAAAAQAAAAAAAAAAAAAAAQAAAAIAAAAvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZV9qc29uLTEuMC4xNDIvc3JjL3ZhbHVlL2RlLnJzACAAEABnAAAAcgAAABkAAAAAAAAAAAAAAAEAAAADAAAAAAAAAAAAAAABAAAABAAAAAAAAAAAAAAAAQAAAAQAAAAAAAAAAAAAAAEAAAAFAAAAAAAAAAAAAAABAAAABgAAAAAAAAAAAAAAAQAAAAcAAAAAAAAAAAAAAAEAAAAIAAAAAAAAAAAAAAABAAAABAAAAAAAAAAAAAAAAQAAAAkAAAAAAAAAAAAAAAEAAAAKAAAAAAAAAAAAAAABAAAAAwAAAENvdWxkbid0IGRlc2VyaWFsaXplIGk2NCBvciB1NjQgZnJvbSBhIEJpZ0ludCBvdXRzaWRlIGk2NDo6TUlOLi51NjQ6Ok1BWCBib3VuZHMA//////////9hdXRob3JjYXJkc2NhcmRzX3hfbm9kZXNfeF93aWRnZXRzY29sb3JkZXNjcmlwdGlvbmVkZ2VzZ3JhcGhpZGljb25jbGFzc2lzX2VkaXRhYmxlaXNyZXNvdXJjZWpzb25sZGNvbnRleHRuYW1lbm9kZWdyb3Vwc25vZGVzb250b2xvZ3lfaWRwdWJsaWNhdGlvbnJlbGF0YWJsZV9yZXNvdXJjZV9tb2RlbF9pZHNyZXNvdXJjZV8yX3Jlc291cmNlX2NvbnN0cmFpbnRzcm9vdHNsdWdzdWJ0aXRsZXZlcnNpb25leHRyYV9maWVsZHNhbGlhc2NvbmZpZ2RhdGF0eXBlZXhwb3J0YWJsZWZpZWxkbmFtZWdyYXBoX2lkaGFzY3VzdG9tYWxpYXNpc19jb2xsZWN0b3Jpc3JlcXVpcmVkaXNzZWFyY2hhYmxlaXN0b3Bub2Rlbm9kZWdyb3VwX2lkbm9kZWlkb250b2xvZ3ljbGFzc3BhcmVudHByb3BlcnR5c29ydG9yZGVyc291cmNlYnJhbmNocHVibGljYXRpb25faWQvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi93YXNtLWJpbmRnZW4tMC4yLjEwMC9zcmMvY29udmVydC9zbGljZXMucnMAAEcDEABvAAAAJAEAAA4AAAALAAAABAAAAAQAAAAMAAAAY2FsbGVkIGBSZXN1bHQ6OnVud3JhcCgpYCBvbiBhbiBgRXJyYCB2YWx1ZXNyYy9ncmFwaC5ycwADBBAADAAAAGcBAAAYAAAAAwQQAAwAAABtAQAAGAAAAAMEEAAMAAAAcwEAABgAAAADBBAADAAAAIIBAAANAAAAAwQQAAwAAACAAQAADQAAAGF0dGVtcHRlZCB0byB0YWtlIG93bmVyc2hpcCBvZiBSdXN0IHZhbHVlIHdoaWxlIGl0IHdhcyBib3Jyb3dlZFN0YXRpY0dyYXBoTWV0YWF1dGhvcmNhcmRzY2FyZHNfeF9ub2Rlc194X3dpZGdldHNjb2xvcmRlc2NyaXB0aW9uZWRnZXNncmFwaGlkaWNvbmNsYXNzaXNfZWRpdGFibGVpc3Jlc291cmNlanNvbmxkY29udGV4dG5hbWVub2RlZ3JvdXBzbm9kZXNvbnRvbG9neV9pZHB1YmxpY2F0aW9ucmVsYXRhYmxlX3Jlc291cmNlX21vZGVsX2lkc3Jlc291cmNlXzJfcmVzb3VyY2VfY29uc3RyYWludHNyb290c2x1Z3N1YnRpdGxldmVyc2lvbmV4dHJhX2ZpZWxkcwAArgQQAAYAAAC0BBAABQAAALkEEAAXAAAA0AQQAAUAAADVBBAACwAAAOAEEAAFAAAA5QQQAAcAAADsBBAACQAAAPUEEAALAAAAAAUQAAoAAAAKBRAADQAAABcFEAAEAAAAGwUQAAoAAAAlBRAABQAAACoFEAALAAAANQUQAAsAAABABRAAHAAAAFwFEAAfAAAAewUQAAQAAAB/BRAABAAAAIMFEAAIAAAAiwUQAAcAAACSBRAADAAAAHN0cnVjdCBTdGF0aWNHcmFwaE1ldGFTdGF0aWNOb2RlYWxpYXNjb25maWdkYXRhdHlwZWV4cG9ydGFibGVmaWVsZG5hbWVncmFwaF9pZGhhc2N1c3RvbWFsaWFzaXNfY29sbGVjdG9yaXNyZXF1aXJlZGlzc2VhcmNoYWJsZWlzdG9wbm9kZW5vZGVncm91cF9pZG5vZGVpZG9udG9sb2d5Y2xhc3NwYXJlbnRwcm9wZXJ0eXNvcnRvcmRlcnNvdXJjZWJyYW5jaHB1YmxpY2F0aW9uX2lkAHgGEAAFAAAAfQYQAAYAAACDBhAACAAAANUEEAALAAAAiwYQAAoAAACVBhAACQAAAJ4GEAAIAAAApgYQAA4AAAC0BhAADAAAAMAGEAAKAAAAygYQAAwAAADWBhAACQAAABcFEAAEAAAA3wYQAAwAAADrBhAABgAAAPEGEAANAAAA/gYQAA4AAAAMBxAACQAAABUHEAAaAAAAc3RydWN0IFN0YXRpY05vZGUvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZV9qc29uLTEuMC4xNDIvc3JjL3ZhbHVlL21vZC5ycwAAANkHEABoAAAAcwAAAAoAAAAAAAAA//////////9YCBAAAAAAAAAAAAAAAAAAAAAAAAQAAAAEAAAADQAAAA4AAAAOAAAAAAAAAAQAAAAEAAAADwAAABAAAAAQAAAAAAAAAAQAAAAEAAAAEQAAABIAAAASAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2YvY29uc29sZV9lcnJvcl9wYW5pY19ob29rLTAuMS43L3NyYy9saWIucnMAALgIEABuAAAAlQAAAA4AAABjYWxsZWQgYE9wdGlvbjo6dW53cmFwX3Rocm93KClgIG9uIGEgYE5vbmVgIHZhbHVlTWFwQWNjZXNzOjpuZXh0X3ZhbHVlIGNhbGxlZCBiZWZvcmUgbmV4dF9rZXkvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZS0xLjAuMjE5L3NyYy9kZS92YWx1ZS5ycwCVCRAAYgAAAGYFAAAbAAAATWFwIGtleSBpcyBub3QgYSBzdHJpbmcgYW5kIGNhbm5vdCBiZSBhbiBvYmplY3Qga2V5AAAAAAAAAAAAAQAAABMAAAAUAAAAFQAAAE9uY2UgaW5zdGFuY2UgaGFzIHByZXZpb3VzbHkgYmVlbiBwb2lzb25lZAAAVAoQACoAAABvbmUtdGltZSBpbml0aWFsaXphdGlvbiBtYXkgbm90IGJlIHBlcmZvcm1lZCByZWN1cnNpdmVseYgKEAA4AAAAL3J1c3RjLzA3N2NlZGMyYWZhOGFjMGI3MjdiN2E2Y2JlMDEyOTQwYmEyMjhkZWIvbGlicmFyeS9zdGQvc3JjL3N5bmMvcG9pc29uL29uY2UucnMAyAoQAFMAAACbAAAAMgAAAAAAAAAAAAAAAQAAAAcAAABhIHNlcXVlbmNlL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvc2VyZGUtMS4wLjIxOS9zcmMvZGUvaW1wbHMucnNGCxAAYgAAAJUEAAAiAAAARgsQAGIAAACYBAAAHAAAAGEgbWFwaW52YWxpZCB2YWx1ZTogLCBleHBlY3RlZCAAzQsQAA8AAADcCxAACwAAAG1pc3NpbmcgZmllbGQgYGD4CxAADwAAAAcMEAABAAAAZHVwbGljYXRlIGZpZWxkIGAAAAAYDBAAEQAAAAcMEAABAAAAL3J1c3RjLzA3N2NlZGMyYWZhOGFjMGI3MjdiN2E2Y2JlMDEyOTQwYmEyMjhkZWIvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAADwMEABKAAAAvgEAAB0AAAA8DBAASgAAAKgBAAAfAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvc2VyZGVfanNvbi0xLjAuMTQyL3NyYy92YWx1ZS9tb2QucnOoDBAAaAAAAHMAAAAKAAAAL3J1c3RjLzA3N2NlZGMyYWZhOGFjMGI3MjdiN2E2Y2JlMDEyOTQwYmEyMjhkZWIvbGlicmFyeS9hbGxvYy9zcmMvY29sbGVjdGlvbnMvYnRyZWUvbWFwLnJzAAAgDRAAWgAAAOMAAAA7AAAAaW50ZXJuYWwgZXJyb3I6IGVudGVyZWQgdW5yZWFjaGFibGUgY29kZSANEABaAAAA5gAAACwAAAAgDRAAWgAAAPoAAAA/AAAAIA0QAFoAAAAfAQAALgAAAC9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvc3RkL3NyYy90aHJlYWQvbG9jYWwucnMA5A0QAE8AAAAVAQAAGQAAAGludmFsaWQgdHlwZTogLCBleHBlY3RlZCAAAABEDhAADgAAAFIOEAALAAAAL3J1c3RjLzA3N2NlZGMyYWZhOGFjMGI3MjdiN2E2Y2JlMDEyOTQwYmEyMjhkZWIvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAAHAOEABKAAAAvgEAAB0AAAAAAAAA///////////QDhAAAAAAAAAAAAAAAAAAAQAAAAAAAAAgY2FuJ3QgYmUgcmVwcmVzZW50ZWQgYXMgYSBKYXZhU2NyaXB0IG51bWJlcgEAAAAAAAAA8A4QACwAAAAdAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvc2VyZGUtd2FzbS1iaW5kZ2VuLTAuNi41L3NyYy9saWIucnMwDxAAaAAAADUAAAAOAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2YvanMtc3lzLTAuMy43Ny9zcmMvbGliLnJzAAAAqA8QAF0AAAD7GAAAAQAAAC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3dhc20tYmluZGdlbi0wLjIuMTAwL3NyYy9jb252ZXJ0L3NsaWNlcy5ycwAYEBAAbwAAACQBAAAOAAAAY2xvc3VyZSBpbnZva2VkIHJlY3Vyc2l2ZWx5IG9yIGFmdGVyIGJlaW5nIGRyb3BwZWQvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi93YXNtLWJpbmRnZW4tMC4yLjEwMC9zcmMvY29udmVydC9zbGljZXMucnMAAADKEBAAbwAAACQBAAAOAAAAKgAAAAwAAAAEAAAAKwAAACwAAAAtAAAAAAAAAAAAAAABAAAALgAAAGEgRGlzcGxheSBpbXBsZW1lbnRhdGlvbiByZXR1cm5lZCBhbiBlcnJvciB1bmV4cGVjdGVkbHkvcnVzdGMvMDc3Y2VkYzJhZmE4YWMwYjcyN2I3YTZjYmUwMTI5NDBiYTIyOGRlYi9saWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMAAKsREABLAAAA7goAAA4AAABFcnJvci9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL3Jhd192ZWMvbW9kLnJzAAAADRIQAFAAAAAuAgAAEQAAAAoKU3RhY2s6CgoKCi9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL21hcC9lbnRyeS5yc3wSEABgAAAAoQEAAC4AAABhc3NlcnRpb24gZmFpbGVkOiBpZHggPCBDQVBBQ0lUWS9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25vZGUucnMADBMQAFsAAACVAgAACQAAAGFzc2VydGlvbiBmYWlsZWQ6IGVkZ2UuaGVpZ2h0ID09IHNlbGYuaGVpZ2h0IC0gMQwTEABbAAAArQIAAAkAAAAMExAAWwAAALECAAAJAAAAYXNzZXJ0aW9uIGZhaWxlZDogc3JjLmxlbigpID09IGRzdC5sZW4oKQwTEABbAAAASgcAAAUAAAAMExAAWwAAAMcEAAAjAAAADBMQAFsAAAAKBQAAJAAAAGFzc2VydGlvbiBmYWlsZWQ6IGVkZ2UuaGVpZ2h0ID09IHNlbGYubm9kZS5oZWlnaHQgLSAxAAAADBMQAFsAAAD6AwAACQAAAC9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25hdmlnYXRlLnJzAGgUEABfAAAAWAIAADAAAAAxAAAADAAAAAQAAAAyAAAAMwAAADQAAAAAAAAAAAAAAAEAAAA1AAAAYSBEaXNwbGF5IGltcGxlbWVudGF0aW9uIHJldHVybmVkIGFuIGVycm9yIHVuZXhwZWN0ZWRseS9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL3N0cmluZy5ycwAANxUQAEsAAADuCgAADgAAAEVycm9yAAAANxUQAEsAAABYBAAAEgAAAEVPRiB3aGlsZSBwYXJzaW5nIGEgbGlzdEVPRiB3aGlsZSBwYXJzaW5nIGFuIG9iamVjdEVPRiB3aGlsZSBwYXJzaW5nIGEgc3RyaW5nRU9GIHdoaWxlIHBhcnNpbmcgYSB2YWx1ZWV4cGVjdGVkIGA6YGV4cGVjdGVkIGAsYCBvciBgXWBleHBlY3RlZCBgLGAgb3IgYH1gZXhwZWN0ZWQgaWRlbnRleHBlY3RlZCB2YWx1ZWV4cGVjdGVkIGAiYGludmFsaWQgZXNjYXBlaW52YWxpZCBudW1iZXJudW1iZXIgb3V0IG9mIHJhbmdlaW52YWxpZCB1bmljb2RlIGNvZGUgcG9pbnRjb250cm9sIGNoYXJhY3RlciAoXHUwMDAwLVx1MDAxRikgZm91bmQgd2hpbGUgcGFyc2luZyBhIHN0cmluZ2tleSBtdXN0IGJlIGEgc3RyaW5naW52YWxpZCB2YWx1ZTogZXhwZWN0ZWQga2V5IHRvIGJlIGEgbnVtYmVyIGluIHF1b3Rlc2Zsb2F0IGtleSBtdXN0IGJlIGZpbml0ZSAoZ290IE5hTiBvciArLy1pbmYpbG9uZSBsZWFkaW5nIHN1cnJvZ2F0ZSBpbiBoZXggZXNjYXBldHJhaWxpbmcgY29tbWF0cmFpbGluZyBjaGFyYWN0ZXJzdW5leHBlY3RlZCBlbmQgb2YgaGV4IGVzY2FwZXJlY3Vyc2lvbiBsaW1pdCBleGNlZWRlZEVycm9yKCwgbGluZTogLCBjb2x1bW46ICkAAADgFxAABgAAAOYXEAAIAAAA7hcQAAoAAAD4FxAAAQAAAC9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25hdmlnYXRlLnJzABwYEABfAAAAxgAAACcAAAAcGBAAXwAAABYCAAAvAAAAHBgQAF8AAAChAAAAJAAAAC9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL3N0cmluZy5ycwCsGBAASwAAAFgEAAASAAAANgAAAAwAAAAEAAAANwAAADgAAAA0AAAAYW55IHZhbGlkIEpTT04gdmFsdWVhIHN0cmluZyBrZXkvcnVzdGMvMDc3Y2VkYzJhZmE4YWMwYjcyN2I3YTZjYmUwMTI5NDBiYTIyOGRlYi9saWJyYXJ5L2FsbG9jL3NyYy9yYXdfdmVjL21vZC5yc0AZEABQAAAALgIAABEAAAAAAAAACAAAAAQAAAA+AAAAPwAAAEAAAABhIGJvb2xlYW5hIHN0cmluZ2J5dGUgYXJyYXlib29sZWFuIGBgAAAA0xkQAAkAAADcGRAAAQAAAGludGVnZXIgYAAAAPAZEAAJAAAA3BkQAAEAAABmbG9hdGluZyBwb2ludCBgDBoQABAAAADcGRAAAQAAAGNoYXJhY3RlciBgACwaEAALAAAA3BkQAAEAAABzdHJpbmcgAEgaEAAHAAAAdW5pdCB2YWx1ZU9wdGlvbiB2YWx1ZW5ld3R5cGUgc3RydWN0c2VxdWVuY2VtYXBlbnVtdW5pdCB2YXJpYW50bmV3dHlwZSB2YXJpYW50dHVwbGUgdmFyaWFudHN0cnVjdCB2YXJpYW50AAAAAQAAAAAAAAAuMGkzMnUzMkxhenkgaW5zdGFuY2UgaGFzIHByZXZpb3VzbHkgYmVlbiBwb2lzb25lZAAA1BoQACoAAAAvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9vbmNlX2NlbGwtMS4yMS4zL3NyYy9saWIucnMIGxAAYAAAAAgDAAAZAAAAcmVlbnRyYW50IGluaXQAAHgbEAAOAAAACBsQAGAAAAB6AgAADQAAAEpzVmFsdWUoKQAAAKAbEAAIAAAAqBsQAAEAAAAvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi93YXNtLWJpbmRnZW4tMC4yLjEwMC9zcmMvY29udmVydC9zbGljZXMucnNudWxsIHBvaW50ZXIgcGFzc2VkIHRvIHJ1c3RyZWN1cnNpdmUgdXNlIG9mIGFuIG9iamVjdCBkZXRlY3RlZCB3aGljaCB3b3VsZCBsZWFkIHRvIHVuc2FmZSBhbGlhc2luZyBpbiBydXN0AAAAvBsQAG8AAADoAAAAAQAAAGxpYnJhcnkvc3RkL3NyYy9wYW5pY2tpbmcucnMvcnVzdGMvMDc3Y2VkYzJhZmE4YWMwYjcyN2I3YTZjYmUwMTI5NDBiYTIyOGRlYi9saWJyYXJ5L2FsbG9jL3NyYy9yYXdfdmVjL21vZC5yc8QcEABQAAAALgIAABEAAAA6AAAAAQAAAAAAAAAkHRAAAQAAACQdEAABAAAASAAAAAwAAAAEAAAASQAAAEoAAABLAAAAL3J1c3RjLzA3N2NlZGMyYWZhOGFjMGI3MjdiN2E2Y2JlMDEyOTQwYmEyMjhkZWIvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAAFgdEABKAAAAvgEAAB0AAAAvcnVzdC9kZXBzL2RsbWFsbG9jLTAuMi44L3NyYy9kbG1hbGxvYy5yc2Fzc2VydGlvbiBmYWlsZWQ6IHBzaXplID49IHNpemUgKyBtaW5fb3ZlcmhlYWQAtB0QACkAAACsBAAACQAAAGFzc2VydGlvbiBmYWlsZWQ6IHBzaXplIDw9IHNpemUgKyBtYXhfb3ZlcmhlYWQAALQdEAApAAAAsgQAAA0AAABBY2Nlc3NFcnJvcmNhbm5vdCBhY2Nlc3MgYSBUaHJlYWQgTG9jYWwgU3RvcmFnZSB2YWx1ZSBkdXJpbmcgb3IgYWZ0ZXIgZGVzdHJ1Y3Rpb246IABnHhAASAAAAAEAAAAAAAAAZW50aXR5IG5vdCBmb3VuZHBlcm1pc3Npb24gZGVuaWVkY29ubmVjdGlvbiByZWZ1c2VkY29ubmVjdGlvbiByZXNldGhvc3QgdW5yZWFjaGFibGVuZXR3b3JrIHVucmVhY2hhYmxlY29ubmVjdGlvbiBhYm9ydGVkbm90IGNvbm5lY3RlZGFkZHJlc3MgaW4gdXNlYWRkcmVzcyBub3QgYXZhaWxhYmxlbmV0d29yayBkb3duYnJva2VuIHBpcGVlbnRpdHkgYWxyZWFkeSBleGlzdHNvcGVyYXRpb24gd291bGQgYmxvY2tub3QgYSBkaXJlY3RvcnlpcyBhIGRpcmVjdG9yeWRpcmVjdG9yeSBub3QgZW1wdHlyZWFkLW9ubHkgZmlsZXN5c3RlbSBvciBzdG9yYWdlIG1lZGl1bWZpbGVzeXN0ZW0gbG9vcCBvciBpbmRpcmVjdGlvbiBsaW1pdCAoZS5nLiBzeW1saW5rIGxvb3Apc3RhbGUgbmV0d29yayBmaWxlIGhhbmRsZWludmFsaWQgaW5wdXQgcGFyYW1ldGVyaW52YWxpZCBkYXRhdGltZWQgb3V0d3JpdGUgemVyb25vIHN0b3JhZ2Ugc3BhY2VzZWVrIG9uIHVuc2Vla2FibGUgZmlsZXF1b3RhIGV4Y2VlZGVkZmlsZSB0b28gbGFyZ2VyZXNvdXJjZSBidXN5ZXhlY3V0YWJsZSBmaWxlIGJ1c3lkZWFkbG9ja2Nyb3NzLWRldmljZSBsaW5rIG9yIHJlbmFtZXRvbyBtYW55IGxpbmtzaW52YWxpZCBmaWxlbmFtZWFyZ3VtZW50IGxpc3QgdG9vIGxvbmdvcGVyYXRpb24gaW50ZXJydXB0ZWR1bnN1cHBvcnRlZHVuZXhwZWN0ZWQgZW5kIG9mIGZpbGVvdXQgb2YgbWVtb3J5aW4gcHJvZ3Jlc3NvdGhlciBlcnJvcnVuY2F0ZWdvcml6ZWQgZXJyb3IgKG9zIGVycm9yICkAAAABAAAAAAAAAK0hEAALAAAAuCEQAAEAAABwYW5pY2tlZCBhdCA6Cm1lbW9yeSBhbGxvY2F0aW9uIG9mICBieXRlcyBmYWlsZWTiIRAAFQAAAPchEAANAAAAbGlicmFyeS9zdGQvc3JjL2FsbG9jLnJzFCIQABgAAABkAQAACQAAAGNhbm5vdCBtb2RpZnkgdGhlIHBhbmljIGhvb2sgZnJvbSBhIHBhbmlja2luZyB0aHJlYWQ8IhAANAAAAKgcEAAcAAAAkAAAAAkAAABIAAAADAAAAAQAAABMAAAAAAAAAAgAAAAEAAAATQAAAAAAAAAIAAAABAAAAE4AAABPAAAAUAAAAFEAAABSAAAAEAAAAAQAAABTAAAAVAAAAFUAAABWAAAAb3BlcmF0aW9uIHN1Y2Nlc3NmdWwQAAAAEQAAABIAAAAQAAAAEAAAABMAAAASAAAADQAAAA4AAAAVAAAADAAAAAsAAAAVAAAAFQAAAA8AAAAOAAAAEwAAACYAAAA4AAAAGQAAABcAAAAMAAAACQAAAAoAAAAQAAAAFwAAAA4AAAAOAAAADQAAABQAAAAIAAAAGwAAAA4AAAAQAAAAFgAAABUAAAALAAAAFgAAAA0AAAALAAAACwAAABMAAADAHhAA0B4QAOEeEADzHhAAAx8QABMfEAAmHxAAOB8QAEUfEABTHxAAaB8QAHQfEAB/HxAAlB8QAKkfEAC4HxAAxh8QANkfEAD/HxAANyAQAFAgEABnIBAAcyAQAHwgEACGIBAAliAQAK0gEAC7IBAAySAQANYgEADqIBAA8iAQAA0hEAAbIRAAKyEQAEEhEABWIRAAYSEQAHchEACEIRAAjyEQAJohEABIYXNoIHRhYmxlIGNhcGFjaXR5IG92ZXJmbG93RCQQABwAAAAvcnVzdC9kZXBzL2hhc2hicm93bi0wLjE1LjIvc3JjL3Jhdy9tb2QucnMAAGgkEAAqAAAAIwAAACgAAABFcnJvcgAAAFcAAAAMAAAABAAAAFgAAABZAAAAWgAAAGNhcGFjaXR5IG92ZXJmbG93AAAAxCQQABEAAABsaWJyYXJ5L2FsbG9jL3NyYy9yYXdfdmVjL21vZC5yc+AkEAAgAAAALgIAABEAAABsaWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMAECUQABsAAADoAQAAFwAAAAAAAAAAAAAAAQAAAFsAAABhIGZvcm1hdHRpbmcgdHJhaXQgaW1wbGVtZW50YXRpb24gcmV0dXJuZWQgYW4gZXJyb3Igd2hlbiB0aGUgdW5kZXJseWluZyBzdHJlYW0gZGlkIG5vdGxpYnJhcnkvYWxsb2Mvc3JjL2ZtdC5ycwAAoiUQABgAAACKAgAADgAAAGxpYnJhcnkvYWxsb2Mvc3JjL3NsaWNlLnJzAADMJRAAGgAAAL4BAAAdAAAAAHAABwAtAQEBAgECAQFICzAVEAFlBwIGAgIBBCMBHhtbCzoJCQEYBAEJAQMBBSsDOwkqGAEgNwEBAQQIBAEDBwoCHQE6AQEBAgQIAQkBCgIaAQICOQEEAgQCAgMDAR4CAwELAjkBBAUBAgQBFAIWBgEBOgEBAgEECAEHAwoCHgE7AQEBDAEJASgBAwE3AQEDBQMBBAcCCwIdAToBAgIBAQMDAQQHAgsCHAI5AgEBAgQIAQkBCgIdAUgBBAECAwEBCAFRAQIHDAhiAQIJCwdJAhsBAQEBATcOAQUBAgULASQJAWYEAQYBAgICGQIEAxAEDQECAgYBDwEAAwAEHAMdAh4CQAIBBwgBAgsJAS0DAQF1AiIBdgMEAgkBBgPbAgIBOgEBBwEBAQECCAYKAgEwHzEEMAoEAyYJDAIgBAIGOAEBAgMBAQU4CAICmAMBDQEHBAEGAQMCxkAAAcMhAAONAWAgAAZpAgAEAQogAlACAAEDAQQBGQIFAZcCGhINASYIGQsBASwDMAECBAICAgEkAUMGAgICAgwBCAEvATMBAQMCAgUCAQEqAggB7gECAQQBAAEAEBAQAAIAAeIBlQUAAwECBQQoAwQBpQIABEEFAAJPBEYLMQR7ATYPKQECAgoDMQQCAgcBPQMkBQEIPgEMAjQJAQEIBAIBXwMCBAYBAgGdAQMIFQI5AgEBAQEMAQkBDgcDBUMBAgYBAQIBAQMEAwEBDgJVCAIDAQEXAVEBAgYBAQIBAQIBAusBAgQGAgECGwJVCAIBAQJqAQEBAghlAQEBAgQBBQAJAQL1AQoEBAGQBAICBAEgCigGAgQIAQkGAgMuDQECAAcBBgEBUhYCBwECAQJ6BgMBAQIBBwEBSAIDAQEBAAILAjQFBQMXAQABBg8ADAMDAAU7BwABPwRRAQsCAAIALgIXAAUDBggIAgceBJQDADcE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ACYAAABQMRAALgAAAMwCAAAmAAAAbGlicmFyeS9jb3JlL3NyYy9udW0vZmx0MmRlYy9tb2QucnMAYDMQACMAAAC7AAAABQAAAGFzc2VydGlvbiBmYWlsZWQ6IGJ1ZlswXSA+IGInMCcAYDMQACMAAAC8AAAABQAAAGFzc2VydGlvbiBmYWlsZWQ6IHBhcnRzLmxlbigpID49IDQAAGAzEAAjAAAAvQAAAAUAAAAuMC4tK05hTmluZjBhc3NlcnRpb24gZmFpbGVkOiBidWYubGVuKCkgPj0gbWF4bGVuAAAAYDMQACMAAAB+AgAADQAAAC4uMDEyMzQ1Njc4OWFiY2RlZgAAAQAAAAAAAABCb3Jyb3dNdXRFcnJvcmFscmVhZHkgYm9ycm93ZWQ6IGY0EAASAAAAY2FsbGVkIGBPcHRpb246OnVud3JhcCgpYCBvbiBhIGBOb25lYCB2YWx1ZWluZGV4IG91dCBvZiBib3VuZHM6IHRoZSBsZW4gaXMgIGJ1dCB0aGUgaW5kZXggaXMgAAAAqzQQACAAAADLNBAAEgAAAAAAAAAEAAAABAAAAGIAAAA9PSE9bWF0Y2hlc2Fzc2VydGlvbiBgbGVmdCAgcmlnaHRgIGZhaWxlZAogIGxlZnQ6IAogcmlnaHQ6IAALNRAAEAAAABs1EAAXAAAAMjUQAAkAAAAgcmlnaHRgIGZhaWxlZDogCiAgbGVmdDogAAAACzUQABAAAABUNRAAEAAAAGQ1EAAJAAAAMjUQAAkAAAA6IAAAAQAAAAAAAACQNRAAAgAAAH0gfTB4MDAwMTAyMDMwNDA1MDYwNzA4MDkxMDExMTIxMzE0MTUxNjE3MTgxOTIwMjEyMjIzMjQyNTI2MjcyODI5MzAzMTMyMzMzNDM1MzYzNzM4Mzk0MDQxNDI0MzQ0NDU0NjQ3NDg0OTUwNTE1MjUzNTQ1NTU2NTc1ODU5NjA2MTYyNjM2NDY1NjY2NzY4Njk3MDcxNzI3Mzc0NzU3Njc3Nzg3OTgwODE4MjgzODQ4NTg2ODc4ODg5OTA5MTkyOTM5NDk1OTY5Nzk4OTkwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwbGlicmFyeS9jb3JlL3NyYy9mbXQvbW9kLnJzZmFs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BIHaJgcMBQWAphCB9QcBICoGTASAjQSAvgMbAw8NbGlicmFyeS9jb3JlL3NyYy9udW0vYmlnbnVtLnJzAPU9EAAeAAAAqwEAAAEAAABhc3NlcnRpb24gZmFpbGVkOiBub2JvcnJvd2Fzc2VydGlvbiBmYWlsZWQ6IGRpZ2l0cyA8IDQwYXNzZXJ0aW9uIGZhaWxlZDogb3RoZXIgPiAwYXR0ZW1wdCB0byBkaXZpZGUgYnkgemVybwB2PhAAGQAAACBvdXQgb2YgcmFuZ2UgZm9yIHNsaWNlIG9mIGxlbmd0aCByYW5nZSBlbmQgaW5kZXggAAC6PhAAEAAAAJg+EAAiAAAAc2xpY2UgaW5kZXggc3RhcnRzIGF0ICBidXQgZW5kcyBhdCAA3D4QABYAAADyPhAADQAAAAADAACDBCAAkQVgAF0ToAASFyAfDCBgH+8sICsqMKArb6ZgLAKo4Cwe++AtAP4gNp7/YDb9AeE2AQohNyQN4TerDmE5LxjhOTAc4UrzHuFOQDShUh5h4VPwamFUT2/hVJ28YVUAz2FWZdGhVgDaIVcA4KFYruIhWuzk4VvQ6GFcIADuXPABf10ANRAAAjUQAAQ1EAACAAAAAgAAAAcAAAAAQbD/wAALHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEIAAAAArooDBG5hbWUACgl3YXNtLndhc20B8okDrAUAOHdhc21fYmluZGdlbjo6X193YmluZGdlbl9pc191bmRlZmluZWQ6OmgxZmRjNjllMjIwZDEyNWUzAS53YXNtX2JpbmRnZW46Ol9fd2JpbmRnZW5faW46Omg4OGM3N2RhNzdjMjVlZmYwAjd3YXNtX2JpbmRnZW46Ol9fd2JpbmRnZW5fYm9vbGVhbl9nZXQ6Omg1ZjAxMjMzYTZjNWJiYzFiAzV3YXNtX2JpbmRnZW46Ol9fd2JpbmRnZW5faXNfYmlnaW50OjpoMzcwZDliOGYzYzE0YThkZAQ2d2FzbV9iaW5kZ2VuOjpfX3diaW5kZ2VuX251bWJlcl9nZXQ6OmhmZTA0MTZiZTgyZjg2ZTg4BTt3YXNtX2JpbmRnZW46Ol9fd2JpbmRnZW5fYmlnaW50X2Zyb21faTY0OjpoZjUxZGFiYzM5MjI2ZmQzMgY0d2FzbV9iaW5kZ2VuOjpfX3diaW5kZ2VuX2pzdmFsX2VxOjpoYTI4ZjM4ODRmMjI5YWI1Ygc2d2FzbV9iaW5kZ2VuOjpfX3diaW5kZ2VuX3N0cmluZ19nZXQ6Omg0ZjhmYTI5YjY4NmE0ODkzCDV3YXNtX2JpbmRnZW46Ol9fd2JpbmRnZW5faXNfb2JqZWN0OjpoOTU4OTg4MWI2ZTYyYjIwNAk7d2FzbV9iaW5kZ2VuOjpfX3diaW5kZ2VuX2JpZ2ludF9mcm9tX3U2NDo6aGRjN2Y4OTNhMTkzNGJlMTkKNXdhc21fYmluZGdlbjo6X193YmluZGdlbl9lcnJvcl9uZXc6Omg3MjEzZGI0MjRkNTkzZmIzCzp3YXNtX2JpbmRnZW46Ol9fd2JpbmRnZW5fanN2YWxfbG9vc2VfZXE6OmhmOTRmNjFkOTcwNjMzMzM3DDV3YXNtX2JpbmRnZW46Ol9fd2JpbmRnZW5fYXNfbnVtYmVyOjpoNWUwNjQxMzI2ZTM2ZDRmZg02d2FzbV9iaW5kZ2VuOjpfX3diaW5kZ2VuX251bWJlcl9uZXc6Omg3ZWQ2NmU3M2Q2Y2ZjY2Q3DjZ3YXNtX2JpbmRnZW46Ol9fd2JpbmRnZW5fc3RyaW5nX25ldzo6aDZmMmIzYTljZGEyNjliMTAPaHNlcmRlX3dhc21fYmluZGdlbjo6T2JqZWN0RXh0OjpnZXRfd2l0aF9yZWZfa2V5OjpfX3diZ19nZXR3aXRocmVma2V5XzFkYzM2MWJkMTAwNTNiZmU6Omg2MGNlOTAyMTI3ZDVhM2M2EFFzZXJkZV93YXNtX2JpbmRnZW46Ok9iamVjdEV4dDo6c2V0OjpfX3diZ19zZXRfM2YxZDBiOTg0ZWQyNzJlZDo6aDI5YTY0Mjg5YmEwYTQyZjARQWpzX3N5czo6QXJyYXk6OmdldDo6X193YmdfZ2V0X2I5YjkzMDQ3ZmUzY2Y0NWI6Omg0N2NlODk3OTQzYWYzOTkyEkdqc19zeXM6OkFycmF5OjpsZW5ndGg6Ol9fd2JnX2xlbmd0aF9lMmQyYTQ5MTMyYzFiMjU2OjpoMmQ1ZjNjN2U4NjBhZTEzMxNBanNfc3lzOjpBcnJheTo6bmV3OjpfX3diZ19uZXdfNzhmZWIxMDhiNjQ3MjcxMzo6aDhlZmExMTU5NzRkMmU5NTIUN3dhc21fYmluZGdlbjo6X193YmluZGdlbl9pc19mdW5jdGlvbjo6aGZkNTViNWExZDIxNWY2N2IVP2pzX3N5czo6TWFwOjpuZXc6Ol9fd2JnX25ld181ZTBiZTczNTIxYmM4YzE3OjpoNTlkMWQ2ZDc5MDk3NDFiNRZqanNfc3lzOjpJdGVyYXRvcjo6bG9va3NfbGlrZV9pdGVyYXRvcjo6TWF5YmVJdGVyYXRvcjo6bmV4dDo6X193YmdfbmV4dF8yNWZlYWRmYzA5MTNmZWE5OjpoYjI5NGI2MzczZDAxZjA1MxdGanNfc3lzOjpJdGVyYXRvcjo6bmV4dDo6X193YmdfbmV4dF82NTc0ZTFhOGE2MmQxMDU1OjpoN2NmZDIyZDAzOTJjNmEwORhKanNfc3lzOjpJdGVyYXRvck5leHQ6OmRvbmU6Ol9fd2JnX2RvbmVfNzY5ZTVlZGU0YjMxYzY3Yjo6aDQ1ZTAzYThmNzQzMDJlMTAZTGpzX3N5czo6SXRlcmF0b3JOZXh0Ojp2YWx1ZTo6X193YmdfdmFsdWVfY2QxZmZhN2IxYWI3OTRmMTo6aGMwYjI4MmU2MjRiNTU3NTcaTGpzX3N5czo6U3ltYm9sOjppdGVyYXRvcjo6X193YmdfaXRlcmF0b3JfOWEyNGM4OGRmODYwZGM2NTo6aGQ0NGJlZDkwN2ViY2UxZWEbQ2pzX3N5czo6UmVmbGVjdDo6Z2V0OjpfX3diZ19nZXRfNjdiMmJhNjJmYzMwZGUxMjo6aDlhOTljZTM3MDI4NWEyNjEcR2pzX3N5czo6RnVuY3Rpb246OmNhbGwwOjpfX3diZ19jYWxsXzY3MmE0ZDIxNjM0ZDRhMjQ6OmgxNDMzZGU5ZDk3NDE1NTM4HUJqc19zeXM6Ok9iamVjdDo6bmV3OjpfX3diZ19uZXdfNDA1ZTIyZjM5MDU3NmNlMjo6aDM0MDNjYTk4ZmQ4YTQyMmMeNXdhc21fYmluZGdlbjo6X193YmluZGdlbl9pc19zdHJpbmc6OmgzMmU1YmUzYzQ4YmVhNjE4H0Fqc19zeXM6OkFycmF5OjpzZXQ6Ol9fd2JnX3NldF8zNzgzNzAyM2YzZDc0MGU4OjpoZjk5ODlmZDM2ZTc1YWY3YSBKanNfc3lzOjpBcnJheTo6aXNfYXJyYXk6Ol9fd2JnX2lzQXJyYXlfYTFlYWI3ZTBkMDY3MzkxYjo6aGRkZDdjZTQ3Y2MxYjVkMmEhkgFqc19zeXM6Ol86OjxpbXBsIHdhc21fYmluZGdlbjo6Y2FzdDo6SnNDYXN0IGZvciBqc19zeXM6OkFycmF5QnVmZmVyPjo6aW5zdGFuY2VvZjo6X193YmdfaW5zdGFuY2VvZl9BcnJheUJ1ZmZlcl9lMTQ1ODU0MzJlMzczN2ZjOjpoNTFlOWM4YmU3ZjhmYmJlYiKCAWpzX3N5czo6Xzo6PGltcGwgd2FzbV9iaW5kZ2VuOjpjYXN0OjpKc0Nhc3QgZm9yIGpzX3N5czo6TWFwPjo6aW5zdGFuY2VvZjo6X193YmdfaW5zdGFuY2VvZl9NYXBfZjM0NjljZTIyNDRkMjQzMDo6aDNjOTEzOGQ2YjZjZDRiNDQjP2pzX3N5czo6TWFwOjpzZXQ6Ol9fd2JnX3NldF84ZmM2YmY4YTViMTA3MWQxOjpoOWEzMjE2YzU2NWQ5NGRiYyRYanNfc3lzOjpOdW1iZXI6OmlzX3NhZmVfaW50ZWdlcjo6X193YmdfaXNTYWZlSW50ZWdlcl8zNDNlMmJlZWVlY2UxYmIwOjpoMTA2NzE4ZTM2MWIwZjU1OCVKanNfc3lzOjpPYmplY3Q6OmVudHJpZXM6Ol9fd2JnX2VudHJpZXNfMzI2NWQ0MTU4YjMzZTVkYzo6aGYyZTVhMjFiZTQ4NWI5OWUmVWpzX3N5czo6V2ViQXNzZW1ibHk6Ok1lbW9yeTo6YnVmZmVyOjpfX3diZ19idWZmZXJfNjA5Y2MzZWVlNTFlZDE1ODo6aDEwNDU2YzBjYzljODBlOTAnRmpzX3N5czo6VWludDhBcnJheTo6bmV3OjpfX3diZ19uZXdfYTEyMDAyYTdmOTFjNzViZTo6aDA4MmUwZDY2ODlmYmZiNzUoRmpzX3N5czo6VWludDhBcnJheTo6c2V0OjpfX3diZ19zZXRfNjU1OTViZGQ4NjhiMzAwOTo6aDVhZjcwODAwZjY4YTJlNDkpTGpzX3N5czo6VWludDhBcnJheTo6bGVuZ3RoOjpfX3diZ19sZW5ndGhfYTQ0NjE5M2RjMjJjMTJmODo6aDFhMjNlZDBmY2JjY2MyOTQqkAFqc19zeXM6Ol86OjxpbXBsIHdhc21fYmluZGdlbjo6Y2FzdDo6SnNDYXN0IGZvciBqc19zeXM6OlVpbnQ4QXJyYXk+OjppbnN0YW5jZW9mOjpfX3diZ19pbnN0YW5jZW9mX1VpbnQ4QXJyYXlfMTcxNTZiY2YxMTgwODZhOTo6aDZhNmNiMWYyZjg5OGVjNzUrU2NvbnNvbGVfZXJyb3JfcGFuaWNfaG9vazo6RXJyb3I6Om5ldzo6X193YmdfbmV3XzhhNmYyMzhhNmVjZTg2ZWE6OmgxMTVkZWQ4MzI0ZTcyYjE4LFdjb25zb2xlX2Vycm9yX3BhbmljX2hvb2s6OkVycm9yOjpzdGFjazo6X193Ymdfc3RhY2tfMGVkNzVkNjg1NzViMGYzYzo6aDE2NTg5NDk0N2Y0ZmFjOWUtUGNvbnNvbGVfZXJyb3JfcGFuaWNfaG9vazo6ZXJyb3I6Ol9fd2JnX2Vycm9yXzc1MzRiOGU5YTM2ZjFhYjQ6Omg0YjVkZTNmMWM1NTg3MzlkLj13YXNtX2JpbmRnZW46Ol9fd2JpbmRnZW5fYmlnaW50X2dldF9hc19pNjQ6OmhiNGJjOGI2OTZjMjUxNGVjLzh3YXNtX2JpbmRnZW46Ol9fd2JpbmRnZW5fZGVidWdfc3RyaW5nOjpoOTllNDFhMjVmZjlhYjgwNDAxd2FzbV9iaW5kZ2VuOjpfX3diaW5kZ2VuX3Rocm93OjpoMzM3ZTk5N2I3YzhkNDdkZjEyd2FzbV9iaW5kZ2VuOjpfX3diaW5kZ2VuX21lbW9yeTo6aDZmNjdhNTM2Nzc2MGFiOWUzaDxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXIgYXMgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZXI+OjpkZXNlcmlhbGl6ZV9zdHJ1Y3Q6OmhhNTkzMjcxY2U5NWE0MDZjNEhjb3JlOjpudW06OmZsdDJkZWM6OnN0cmF0ZWd5OjpkcmFnb246OmZvcm1hdF9zaG9ydGVzdDo6aDhiM2UzYWViMzNlM2Q1Yjg1aDxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXIgYXMgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZXI+OjpkZXNlcmlhbGl6ZV9zdHJ1Y3Q6OmhmMTE0OTkyODI3ODU5MzZmNkVjb3JlOjpudW06OmZsdDJkZWM6OnN0cmF0ZWd5OjpkcmFnb246OmZvcm1hdF9leGFjdDo6aGRkZmIzOTBhNjk0NGY3ZjY3OmRsbWFsbG9jOjpkbG1hbGxvYzo6RGxtYWxsb2M8QT46Om1hbGxvYzo6aDkzMmQ1NGU1ZGE1NTVlM2U4S2NvcmU6Om51bTo6Zmx0MmRlYzo6c3RyYXRlZ3k6OmdyaXN1Ojpmb3JtYXRfc2hvcnRlc3Rfb3B0OjpoN2ViNDA0YmEyNDEwMzgzZjllPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6OkRlc2VyaWFsaXplciBhcyBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplcj46OmRlc2VyaWFsaXplX21hcDo6aGEwN2NhNzRkNzI1NjhkZjU6QGhhc2hicm93bjo6cmF3OjpSYXdUYWJsZTxULEE+OjpyZXNlcnZlX3JlaGFzaDo6aDVhNmZiOWVkYmNhYTBlNDA7ZTxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXIgYXMgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZXI+OjpkZXNlcmlhbGl6ZV9tYXA6Omg1NTFhMzJhOGU3OWI4N2ZmPGU8c2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6RGVzZXJpYWxpemVyIGFzIHNlcmRlOjpkZTo6RGVzZXJpYWxpemVyPjo6ZGVzZXJpYWxpemVfbWFwOjpoZGFkNjhjMWU2ZGRiYTY4Zj0zd2FzbTo6Z3JhcGg6OlN0YXRpY05vZGU6OmNvbXBhcmU6OmgyYjUzYWQzYzY0YmExMjBkPkJjb3JlOjpudW06OmZsdDJkZWM6OnN0cmF0ZWd5OjpkcmFnb246Om11bF9wb3cxMDo6aDIzZWJhZDJiY2Q1NzE4MDM/a3dhc206OmdyYXBoOjpfOjo8aW1wbCBzZXJkZTo6c2VyOjpTZXJpYWxpemUgZm9yIHdhc206OmdyYXBoOjpTdGF0aWNHcmFwaE1ldGE+OjpzZXJpYWxpemU6OmhkYTlmYmEwNGRhZTc3ODZjQDFjb3JlOjpzdHI6OnNsaWNlX2Vycm9yX2ZhaWxfcnQ6OmhhYjFkMTM3ZWExZDM2MWU1QThjb3JlOjpudW06OmJpZ251bTo6QmlnMzJ4NDA6Om11bF9wb3cyOjpoZWE1YjQzZWM1NjkxOTQ5ZkJIY29yZTo6bnVtOjpmbHQyZGVjOjpzdHJhdGVneTo6Z3Jpc3U6OmZvcm1hdF9leGFjdF9vcHQ6OmgyYjAxYTk2ZmMxODJlMzEwQ2Z3YXNtOjpncmFwaDo6Xzo6PGltcGwgc2VyZGU6OnNlcjo6U2VyaWFsaXplIGZvciB3YXNtOjpncmFwaDo6U3RhdGljTm9kZT46OnNlcmlhbGl6ZTo6aGUwZjJhMGNmZTMwZGYzODRE+gFhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpIYW5kbGU8YWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6Tm9kZVJlZjxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6Ok11dCxLLFYsYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpMZWFmPixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkVkZ2U+OjppbnNlcnQ6OmhiNTc1NjFjMDE4NmVlYWZjRTE8c3RyIGFzIGNvcmU6OmZtdDo6RGVidWc+OjpmbXQ6OmhkZDllMzU0MDdjODNkMzAwRjNjb3JlOjpzdHI6OmNvdW50Ojpkb19jb3VudF9jaGFyczo6aDg1MmExNWNmYzBlZGM0N2VHogE8c2VyZGVfanNvbjo6dmFsdWU6OmRlOjo8aW1wbCBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplIGZvciBzZXJkZV9qc29uOjp2YWx1ZTo6VmFsdWU+OjpkZXNlcmlhbGl6ZTo6VmFsdWVWaXNpdG9yIGFzIHNlcmRlOjpkZTo6VmlzaXRvcj46OnZpc2l0X21hcDo6aDBiMzQwNTY5YmVhZDFjM2FIcDxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjptYXA6OkJUcmVlTWFwPEssVixBPiBhcyBjb3JlOjpjbG9uZTo6Q2xvbmU+OjpjbG9uZTo6Y2xvbmVfc3VidHJlZTo6aGIxYzJhZWI4OTkxYjgyMmVJZTxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXIgYXMgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZXI+OjpkZXNlcmlhbGl6ZV9hbnk6Omg3MDE4ODUwZmMxYWI3ODJlSihfX3J1c3RjWzZkYzAyMmNiZDE0YWU1NGRdOjpfX3JkbF9yZWFsbG9jS0Jjb3JlOjpmbXQ6OmZsb2F0OjpmbG9hdF90b19kZWNpbWFsX2NvbW1vbl9leGFjdDo6aGMwYjUxOGI3N2I2ODI4MjhMQGhhc2hicm93bjo6cmF3OjpSYXdUYWJsZTxULEE+OjpyZXNlcnZlX3JlaGFzaDo6aDMwNmRkOGQxMTI1YzJkODFNQGhhc2hicm93bjo6cmF3OjpSYXdUYWJsZTxULEE+OjpyZXNlcnZlX3JlaGFzaDo6aGQwYmMxMDNjYTRiZjhiYWJOMHdhc206OmdyYXBoOjpTdGF0aWNOb2RlOjpjb3B5OjpoY2I0MDVlZWY5N2Q3ZTAxY086Y29yZTo6bnVtOjpiaWdudW06OkJpZzMyeDQwOjptdWxfZGlnaXRzOjpoMjBiYTJhZWRkYjUwNDk2NlBAaGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlPFQsQT46OnJlc2VydmVfcmVoYXNoOjpoMDdmZTYwNzMwNWJlZDFjY1FFY29yZTo6Y2hhcjo6bWV0aG9kczo6PGltcGwgY2hhcj46OmVzY2FwZV9kZWJ1Z19leHQ6OmgyZTljZTUwMDk2NzUwZjJkUjVjb3JlOjpmbXQ6OkZvcm1hdHRlcjo6cGFkX2ludGVncmFsOjpoNzI1ODM4NTgxNjQ5NjczN1M4ZGxtYWxsb2M6OmRsbWFsbG9jOjpEbG1hbGxvYzxBPjo6ZnJlZTo6aDJlNDAzYmMyNWM4YzQ5NWVUogE8c2VyZGVfanNvbjo6dmFsdWU6OmRlOjo8aW1wbCBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplIGZvciBzZXJkZV9qc29uOjp2YWx1ZTo6VmFsdWU+OjpkZXNlcmlhbGl6ZTo6VmFsdWVWaXNpdG9yIGFzIHNlcmRlOjpkZTo6VmlzaXRvcj46OnZpc2l0X21hcDo6aDFmMzkxM2NkYTZmZWMwNTdVO2hhc2hicm93bjo6bWFwOjpIYXNoTWFwPEssVixTLEE+OjppbnNlcnQ6OmhlYWY4OTIwYjZmYTcxYjE2VkVjb3JlOjpmbXQ6OmZsb2F0OjpmbG9hdF90b19kZWNpbWFsX2NvbW1vbl9zaG9ydGVzdDo6aDQ1YmI4ZTM0OWQ5NDRhYzlXTDxjb3JlOjpoYXNoOjpzaXA6Okhhc2hlcjxTPiBhcyBjb3JlOjpoYXNoOjpIYXNoZXI+Ojp3cml0ZTo6aDgxMDA4YTBhY2U1ZmU3OTlYRTxzZXJkZTo6ZGU6OlVuZXhwZWN0ZWQgYXMgY29yZTo6Zm10OjpEaXNwbGF5Pjo6Zm10OjpoMjdiMWU0M2MxOTNhZmQ2ZllBaGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlSW5uZXI6OnJlaGFzaF9pbl9wbGFjZTo6aDkwYzNhYjA5YmY2YTFhYzZaO2hhc2hicm93bjo6bWFwOjpIYXNoTWFwPEssVixTLEE+OjppbnNlcnQ6OmgzNzM4YTU0YTZiZDEwNDYxWztoYXNoYnJvd246Om1hcDo6SGFzaE1hcDxLLFYsUyxBPjo6aW5zZXJ0OjpoNWU1ODQ2YjU4ZDdhMTliYlwsY29yZTo6Zm10OjpGb3JtYXR0ZXI6OnBhZDo6aDkxNjlkM2Q3OGNlOTA1YWFdXDxzdGQ6OmNvbGxlY3Rpb25zOjpoYXNoOjptYXA6Okhhc2hNYXA8SyxWLFM+IGFzIGNvcmU6OmNtcDo6UGFydGlhbEVxPjo6ZXE6OmgzYWZiOTU2NTMyYjBlODBlXko8Y29yZTo6b3BzOjpyYW5nZTo6UmFuZ2U8SWR4PiBhcyBjb3JlOjpmbXQ6OkRlYnVnPjo6Zm10OjpoMGNjOGE4NGJiMTI5MjM4OF8vc2VyZGVfd2FzbV9iaW5kZ2VuOjp0b192YWx1ZTo6aDZjMGM2Yjg1ZDYwN2E3MTRgPGNvcmU6OmZtdDo6Rm9ybWF0dGVyOjpwYWRfZm9ybWF0dGVkX3BhcnRzOjpoNjdlZTI0NWJhOGMzYTBlOGFGc2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6RGVzZXJpYWxpemVyOjppbnZhbGlkX3R5cGVfOjpoMjg4NDQ4N2UxN2MxNzI3ZmIjY29yZTo6Zm10Ojp3cml0ZTo6aDkzMGQxOTU5MzA0ODBiMjNjPmNvcmU6OmZtdDo6Rm9ybWF0dGVyOjp3cml0ZV9mb3JtYXR0ZWRfcGFydHM6OmhkM2MyMTRhNDNkNjdhYWM0ZG9zZXJkZV9qc29uOjp2YWx1ZTo6c2VyOjo8aW1wbCBzZXJkZTo6c2VyOjpTZXJpYWxpemUgZm9yIHNlcmRlX2pzb246OnZhbHVlOjpWYWx1ZT46OnNlcmlhbGl6ZTo6aDUwMTUzNjIxOTg0ZTNkNGFloQE8d2FzbTo6Z3JhcGg6Ol86OjxpbXBsIHNlcmRlOjpkZTo6RGVzZXJpYWxpemUgZm9yIHdhc206OmdyYXBoOjpTdGF0aWNHcmFwaE1ldGE+OjpkZXNlcmlhbGl6ZTo6X19GaWVsZFZpc2l0b3IgYXMgc2VyZGU6OmRlOjpWaXNpdG9yPjo6dmlzaXRfc3RyOjpoNTgyZjg5YWFmNTJjNTE4ZWZBaGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlPFQsQT46OmNsb25lX2Zyb21faW1wbDo6aGNiZjhlNjcwM2VkN2I0MzBnNGNvcmU6Omhhc2g6OkJ1aWxkSGFzaGVyOjpoYXNoX29uZTo6aGUyOTg2Mzk3MGY1MmU2YWRoRjxhbGxvYzo6dmVjOjpWZWM8VCxBPiBhcyBjb3JlOjpjbG9uZTo6Q2xvbmU+OjpjbG9uZTo6aDAwZTgwNjk3ZmEwYzFmN2FphAJhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpIYW5kbGU8YWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6Tm9kZVJlZjxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6Ok11dCxLLFYsYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpMZWFmPixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkVkZ2U+OjppbnNlcnRfcmVjdXJzaW5nOjpoNDhiZjAyOTdmZDY3MjJhOWplPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6OkRlc2VyaWFsaXplciBhcyBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplcj46OmRlc2VyaWFsaXplX3NlcTo6aDYxMjk1M2VlMDkxODQ2ZThrYDxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjptYXA6OkJUcmVlTWFwPEssVixBPiBhcyBjb3JlOjpjbXA6OlBhcnRpYWxFcT46OmVxOjpoZDYzYjhhMDg1NTI0OTNlOWxBZGxtYWxsb2M6OmRsbWFsbG9jOjpEbG1hbGxvYzxBPjo6ZGlzcG9zZV9jaHVuazo6aDQzZmQ1OWJjNTJiZTgwMDltnAE8d2FzbTo6Z3JhcGg6Ol86OjxpbXBsIHNlcmRlOjpkZTo6RGVzZXJpYWxpemUgZm9yIHdhc206OmdyYXBoOjpTdGF0aWNOb2RlPjo6ZGVzZXJpYWxpemU6Ol9fRmllbGRWaXNpdG9yIGFzIHNlcmRlOjpkZTo6VmlzaXRvcj46OnZpc2l0X3N0cjo6aDU4MmUyZDNhZjI4Y2RmNTVuOndhc206OmdyYXBoOjpTdGF0aWNOb2RlOjpjb21wYXJlX3N0YXRpYzo6aGYzODAyOTcyY2JmOTMxZTlvNnNlcmRlOjpzZXI6OlNlcmlhbGl6ZXI6OmNvbGxlY3RfbWFwOjpoMWMwZTM0NGI4ZjE1ZmZjN3A2c2VyZGU6OnNlcjo6U2VyaWFsaXplcjo6Y29sbGVjdF9tYXA6OmgzNjg4MjU1ZjY3MjNjNzg3cTZzZXJkZTo6c2VyOjpTZXJpYWxpemVyOjpjb2xsZWN0X21hcDo6aGY2ZmY0Mzg0OGZmMWYzMjRyWGNvcmU6Om51bTo6Zmx0MmRlYzo6c3RyYXRlZ3k6OmdyaXN1Ojpmb3JtYXRfZXhhY3Rfb3B0Ojpwb3NzaWJseV9yb3VuZDo6aDliODM3ZjVlZGVjZDM5NmRzmAE8c2VyZGU6OmRlOjppbXBsczo6PGltcGwgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZSBmb3IgYWxsb2M6OnZlYzo6VmVjPFQ+Pjo6ZGVzZXJpYWxpemU6OlZlY1Zpc2l0b3I8VD4gYXMgc2VyZGU6OmRlOjpWaXNpdG9yPjo6dmlzaXRfc2VxOjpoNzc2MjBiYzJhN2JkNTNkNXSYATxzZXJkZTo6ZGU6OmltcGxzOjo8aW1wbCBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplIGZvciBhbGxvYzo6dmVjOjpWZWM8VD4+OjpkZXNlcmlhbGl6ZTo6VmVjVmlzaXRvcjxUPiBhcyBzZXJkZTo6ZGU6OlZpc2l0b3I+Ojp2aXNpdF9zZXE6Omg5MTdiOGNiNDA4ZWNhYTlhdTNhbGxvYzo6Zm10Ojpmb3JtYXQ6OmZvcm1hdF9pbm5lcjo6aDQ3OWVlNjQzZjViNTE0NGF2ggJhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpIYW5kbGU8YWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6Tm9kZVJlZjxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6Ok11dCxLLFYsYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpJbnRlcm5hbD4sYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpFZGdlPjo6aW5zZXJ0X2ZpdDo6aDdjYmE2YjBjNjQ4ZmY2OWZ3PGRsbWFsbG9jOjpkbG1hbGxvYzo6RGxtYWxsb2M8QT46Om1lbWFsaWduOjpoMTJlZGYzZjM0OGUxMjUwZnhFPHN0ZDo6aW86OmVycm9yOjpFcnJvciBhcyBjb3JlOjpmbXQ6OkRpc3BsYXk+OjpmbXQ6OmgxNzBmZjZkNmNhODc0ZmNieThjb3JlOjpmbXQ6Om51bTo6aW1wOjo8aW1wbCB1NjQ+OjpfZm10OjpoZTEzNmMxZGUzMzBhMmQyZno4Y29yZTo6Zm10OjpudW06OmltcDo6PGltcGwgdTMyPjo6X2ZtdDo6aDhmYWM4MTlhNDg5ZWI2NzR7mAE8c2VyZGU6OmRlOjppbXBsczo6PGltcGwgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZSBmb3IgYWxsb2M6OnZlYzo6VmVjPFQ+Pjo6ZGVzZXJpYWxpemU6OlZlY1Zpc2l0b3I8VD4gYXMgc2VyZGU6OmRlOjpWaXNpdG9yPjo6dmlzaXRfc2VxOjpoYWVmMGU4MjBmZWY2MmQ5OXxAZGxtYWxsb2M6OmRsbWFsbG9jOjpEbG1hbGxvYzxBPjo6dW5saW5rX2NodW5rOjpoOTU0ZDc2NGM3Mzc3ZjlmMX2YATxzZXJkZTo6ZGU6OmltcGxzOjo8aW1wbCBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplIGZvciBhbGxvYzo6dmVjOjpWZWM8VD4+OjpkZXNlcmlhbGl6ZTo6VmVjVmlzaXRvcjxUPiBhcyBzZXJkZTo6ZGU6OlZpc2l0b3I+Ojp2aXNpdF9zZXE6Omg3NDQ4M2ViZDk1NGQ3MTM0fkxjb3JlOjp1bmljb2RlOjp1bmljb2RlX2RhdGE6OmdyYXBoZW1lX2V4dGVuZDo6bG9va3VwX3Nsb3c6Omg4MDhiZTA5MzYwMDg2Njkwf2g8c2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6Ok1hcFNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplTWFwPjo6c2VyaWFsaXplX3ZhbHVlOjpoMWU2MGU3MmYzNTk3ODA5ZoABN2NvcmU6OnBhbmlja2luZzo6YXNzZXJ0X2ZhaWxlZF9pbm5lcjo6aDRkMjlhN2UwNjkyMTZhYjmBAThjb3JlOjpudW06OmZsdDJkZWM6OmRpZ2l0c190b19kZWNfc3RyOjpoZTY1ZjUzMTNiOTI5Y2MzN4IB/gFhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpIYW5kbGU8YWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6Tm9kZVJlZjxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6Ok11dCxLLFYsYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpJbnRlcm5hbD4sYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpFZGdlPjo6aW5zZXJ0OjpoNTRlZjNiZGNiNjIwMjdkNYMBMmNvcmU6OnVuaWNvZGU6OnByaW50YWJsZTo6Y2hlY2s6OmgzNDZhNTUwZWQ0YzZlZWJjhAHcAWFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6OkhhbmRsZTxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpOb2RlUmVmPGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Om1hcmtlcjo6TXV0LEssVixOb2RlVHlwZT4sYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpLVj46OnNwbGl0X2xlYWZfZGF0YTo6aDQ3ODgwNTQ5NjVhNzJjMWOFARpfX3diZ19zdGF0aWNncmFwaG1ldGFfZnJlZYYBRmRsbWFsbG9jOjpkbG1hbGxvYzo6RGxtYWxsb2M8QT46Omluc2VydF9sYXJnZV9jaHVuazo6aDgwYTM1YmE2ZjU3MzUzYjeHAWg8c2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6Ok1hcFNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplTWFwPjo6c2VyaWFsaXplX3ZhbHVlOjpoMWY5ODQzMzMyMTZmNjhhNogBTDxzZXJkZV9qc29uOjplcnJvcjo6RXJyb3JDb2RlIGFzIGNvcmU6OmZtdDo6RGlzcGxheT46OmZtdDo6aDY2MTE1ODNhNTRlNDZlNWKJAV9oYXNoYnJvd246OnJ1c3RjX2VudHJ5Ojo8aW1wbCBoYXNoYnJvd246Om1hcDo6SGFzaE1hcDxLLFYsUyxBPj46OnJ1c3RjX2VudHJ5OjpoMjYzZDUxZDcyZGFmMTQ4YooBSTxzdGQ6OnBhbmljOjpQYW5pY0hvb2tJbmZvIGFzIGNvcmU6OmZtdDo6RGlzcGxheT46OmZtdDo6aDZhMGZlMTA3M2RiODY3MmWLAWg8c2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6Ok1hcFNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplTWFwPjo6c2VyaWFsaXplX3ZhbHVlOjpoYTcyZDJlMmQ3ZTYwZDhjNowBaDxzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6TWFwU2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVNYXA+OjpzZXJpYWxpemVfdmFsdWU6OmgzMTVlZGNjNzMyYmY0ODJhjQE2c2VyZGU6OnNlcjo6U2VyaWFsaXplcjo6Y29sbGVjdF9zZXE6OmhjM2Y3MjAwMDk0NmU5MDk0jgFgPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6Ok9iamVjdEFjY2VzcyBhcyBzZXJkZTo6ZGU6Ok1hcEFjY2Vzcz46Om5leHRfa2V5X3NlZWQ6OmgxNmE5MTkyZGYwYTljMjcyjwFgPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6Ok9iamVjdEFjY2VzcyBhcyBzZXJkZTo6ZGU6Ok1hcEFjY2Vzcz46Om5leHRfa2V5X3NlZWQ6OmhhMTM4Y2IxZmJmYTJkMjljkAFKPGFsbG9jOjpzdHJpbmc6OlN0cmluZyBhcyBjb3JlOjpmbXQ6OldyaXRlPjo6d3JpdGVfY2hhcjo6aDQ5ZTAwZDlkZDJjZThhOTKRAUo8YWxsb2M6OnN0cmluZzo6U3RyaW5nIGFzIGNvcmU6OmZtdDo6V3JpdGU+Ojp3cml0ZV9jaGFyOjpoNDllMDBkOWRkMmNlOGE5MpIBSjxhbGxvYzo6c3RyaW5nOjpTdHJpbmcgYXMgY29yZTo6Zm10OjpXcml0ZT46OndyaXRlX2NoYXI6Omg0OWUwMGQ5ZGQyY2U4YTkykwFBYWxsb2M6OnJhd192ZWM6OlJhd1ZlY0lubmVyPEE+Ojpncm93X2Ftb3J0aXplZDo6aDMwOTIxZjRlNmFhNzE3YzOUAUFhbGxvYzo6cmF3X3ZlYzo6UmF3VmVjSW5uZXI8QT46Omdyb3dfYW1vcnRpemVkOjpoYWZjNDIyOGRiZDU4NzgyNJUBSjxhbGxvYzo6c3RyaW5nOjpTdHJpbmcgYXMgY29yZTo6Zm10OjpXcml0ZT46OndyaXRlX2NoYXI6Omg0OWUwMGQ5ZGQyY2U4YTkylgFaYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bWFwOjplbnRyeTo6VmFjYW50RW50cnk8SyxWLEE+OjppbnNlcnRfZW50cnk6OmgyMDVhNzA3OTUwZTE2YzMwlwExY29uc29sZV9lcnJvcl9wYW5pY19ob29rOjpob29rOjpoOWZkYjE1ZGYxZGJkYWE3OJgBSjxhbGxvYzo6c3RyaW5nOjpTdHJpbmcgYXMgY29yZTo6Zm10OjpXcml0ZT46OndyaXRlX2NoYXI6Omg0OWUwMGQ5ZGQyY2U4YTkymQEwYWxsb2M6OnJjOjpSYzxULEE+Ojpkcm9wX3Nsb3c6Omg4OTU1ZTU5YTVkNzEwY2RkmgFPc2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6RGVzZXJpYWxpemVyOjpkZXNlcmlhbGl6ZV9mcm9tX2FycmF5OjpoMzlmYzFlNGFhNTRlNTJlMJsBNmNvcmU6OnNsaWNlOjptZW1jaHI6Om1lbWNocl9hbGlnbmVkOjpoOTU5MGE1ZTNjZjQ1Y2ExOZwBZDxzZXJkZTo6ZGU6OnZhbHVlOjpNYXBEZXNlcmlhbGl6ZXI8SSxFPiBhcyBzZXJkZTo6ZGU6Ok1hcEFjY2Vzcz46Om5leHRfZW50cnlfc2VlZDo6aDIzNzhiMjkwMDAyMmUwOTmdAWQ8c2VyZGU6OmRlOjp2YWx1ZTo6TWFwRGVzZXJpYWxpemVyPEksRT4gYXMgc2VyZGU6OmRlOjpNYXBBY2Nlc3M+OjpuZXh0X2VudHJ5X3NlZWQ6OmhjY2EwNzM3YTgxN2Y4NzM1ngFkPHNlcmRlOjpkZTo6dmFsdWU6Ok1hcERlc2VyaWFsaXplcjxJLEU+IGFzIHNlcmRlOjpkZTo6TWFwQWNjZXNzPjo6bmV4dF9lbnRyeV9zZWVkOjpoMDllNjE4MGE0ZDEzN2ZlZZ8BNDxjaGFyIGFzIGNvcmU6OmZtdDo6RGlzcGxheT46OmZtdDo6aGY5ZDUyNzZmNTk4ZTE1ZmKgAXQ8c3RkOjpwYW5pY2tpbmc6OmJlZ2luX3BhbmljX2hhbmRsZXI6OkZvcm1hdFN0cmluZ1BheWxvYWQgYXMgY29yZTo6cGFuaWM6OlBhbmljUGF5bG9hZD46OnRha2VfYm94OjpoNzMzZjczNDFkNDljYzIzOKEBMDwmVCBhcyBjb3JlOjpmbXQ6OkRlYnVnPjo6Zm10OjpoNTc5ZWM5M2I5ZjY3OWFkNKIBQm9uY2VfY2VsbDo6dW5zeW5jOjpPbmNlQ2VsbDxUPjo6Z2V0X29yX3RyeV9pbml0OjpoMTg1OWE4Yzc5YzRjN2MwZKMB+wFhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpIYW5kbGU8YWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6Tm9kZVJlZjxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6Ok11dCxLLFYsYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpJbnRlcm5hbD4sYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpLVj46OnNwbGl0OjpoZjc3MjVlNTc5ZTdmYmViMaQBI2pzX3N5czo6dHJ5X2l0ZXI6Omg5YjQ1NzQwYjY0MDQ4Y2RipQFAaGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlPFQsQT46Omluc2VydF9ub19ncm93OjpoNWM4NjEwYjZkODhiNmRiZqYBUWFsbG9jOjpyYXdfdmVjOjpSYXdWZWNJbm5lcjxBPjo6cmVzZXJ2ZTo6ZG9fcmVzZXJ2ZV9hbmRfaGFuZGxlOjpoYjBlZjlmOThiNDAzMTZiMKcBZTxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXIgYXMgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZXI+OjpkZXNlcmlhbGl6ZV9tYXA6Omg2ZWNlNDBkMzJhOGU5NTRlqAE2c2VyZGU6OnNlcjo6U2VyaWFsaXplcjo6Y29sbGVjdF9zZXE6OmgyMmE4MTMzNzQyNGFmMjFhqQFuPGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om1hcDo6SXRlcjxLLFY+IGFzIGNvcmU6Oml0ZXI6OnRyYWl0czo6aXRlcmF0b3I6Okl0ZXJhdG9yPjo6bmV4dDo6aDllZGQzMWRkZmUzMmU0ZmaqAUphbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjptYXA6OkJUcmVlTWFwPEssVixBPjo6aW5zZXJ0OjpoMjc5NzNlNzk3ODZiZmUzM6sBPWFsbG9jOjpyYXdfdmVjOjpSYXdWZWNJbm5lcjxBPjo6Z3Jvd19leGFjdDo6aDhiMTk0ZjZlMTIzNDc5YTCsAUNoYXNoYnJvd246OnJhdzo6UmF3VGFibGVJbm5lcjo6bmV3X3VuaW5pdGlhbGl6ZWQ6Omg1YmY4OTMyMDY0YTgxNDJhrQFGPFtBXSBhcyBjb3JlOjpzbGljZTo6Y21wOjpTbGljZVBhcnRpYWxFcTxCPj46OmVxdWFsOjpoMGUzYjVmNjcyMTBmMWY3OK4BN3dhc21fYmluZGdlbjo6ZXh0ZXJucmVmOjpTbGFiOjphbGxvYzo6aGU5MWUwM2JjN2YyNjUzODOvAUY8c2VyZGVfanNvbjo6ZXJyb3I6OkVycm9yIGFzIGNvcmU6OmZtdDo6RGVidWc+OjpmbXQ6Omg3Njg0YjFlMTY3ZGY2ZTJhsAFRc3RkOjpzeXM6OnRocmVhZF9sb2NhbDo6bm9fdGhyZWFkczo6TGF6eVN0b3JhZ2U8VD46OmluaXRpYWxpemU6OmhkNDI2NTM5ZjYxYmViZmVisQEZc3RhdGljZ3JhcGhtZXRhX2dldEF1dGhvcrIBFHN0YXRpY25vZGVfZ2V0X2FsaWFzswEac3RhdGljbm9kZV9nZXRfZGVzY3JpcHRpb260ARhzdGF0aWNub2RlX2dldF9maWVsZG5hbWW1ARtzdGF0aWNub2RlX2dldF9ub2RlZ3JvdXBfaWS2ARxzdGF0aWNub2RlX2dldF9vbnRvbG9neWNsYXNztwEdc3RhdGljbm9kZV9nZXRfcGFyZW50cHJvcGVydHm4ASlzdGF0aWNub2RlX2dldF9zb3VyY2VicmFuY2hwdWJsaWNhdGlvbl9pZLkBQ2FsbG9jOjpyYXdfdmVjOjpSYXdWZWNJbm5lcjxBPjo6c2hyaW5rX3VuY2hlY2tlZDo6aDYyZjI4YWYxM2JiNTg3YTm6AUNhbGxvYzo6cmF3X3ZlYzo6UmF3VmVjSW5uZXI8QT46OnNocmlua191bmNoZWNrZWQ6OmhiNDFlZmYwMjk4MjFiODgyuwE4c2VyZGVfd2FzbV9iaW5kZ2VuOjplcnJvcjo6RXJyb3I6Om5ldzo6aDA0MGE4NTRhNzhiMWQzNjm8AbMCYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bmF2aWdhdGU6OjxpbXBsIGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6OkhhbmRsZTxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpOb2RlUmVmPGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Om1hcmtlcjo6RHlpbmcsSyxWLGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Om1hcmtlcjo6TGVhZj4sYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpFZGdlPj46OmRlYWxsb2NhdGluZ19uZXh0OjpoYzIwOGFjYjc2N2VkNDc2Yb0BXTxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpNYXBBY2Nlc3MgYXMgc2VyZGU6OmRlOjpNYXBBY2Nlc3M+OjpuZXh0X2tleV9zZWVkOjpoMzZmZDU1NTc2Zjk2YjQzOb4BXTxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpNYXBBY2Nlc3MgYXMgc2VyZGU6OmRlOjpNYXBBY2Nlc3M+OjpuZXh0X2tleV9zZWVkOjpoZmU1NjdiNjQ3NTFhNTZlM78BN3N0ZDo6cGFuaWNraW5nOjpydXN0X3BhbmljX3dpdGhfaG9vazo6aDZlZDdjYzE2ODFkOWRhNmbAAWU8c2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6RGVzZXJpYWxpemVyIGFzIHNlcmRlOjpkZTo6RGVzZXJpYWxpemVyPjo6ZGVzZXJpYWxpemVfc2VxOjpoYjAzMGJkMjE0NDg5MDBlZsEBZTxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXIgYXMgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZXI+OjpkZXNlcmlhbGl6ZV9zZXE6OmhiZDM4MjM4MmNlOTJkZjNmwgFJYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bWFwOjpCVHJlZU1hcDxLLFYsQT46OmVudHJ5OjpoZDgyODk0MmQ1YjhjY2UzN8MBUWFsbG9jOjpyYXdfdmVjOjpSYXdWZWNJbm5lcjxBPjo6cmVzZXJ2ZTo6ZG9fcmVzZXJ2ZV9hbmRfaGFuZGxlOjpoODg3NDUyNTg3N2M2OTE2N8QBOWNvcmU6OnVuaWNvZGU6OnByaW50YWJsZTo6aXNfcHJpbnRhYmxlOjpoY2E2NjVlOGM5M2Y5NWRiNcUBTmFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om1hcDo6SW50b0l0ZXI8SyxWLEE+OjpkeWluZ19uZXh0OjpoYzY4N2NjMWEzOGJjYzhlNMYBSzxzZXJkZTo6ZGU6OldpdGhEZWNpbWFsUG9pbnQgYXMgY29yZTo6Zm10OjpEaXNwbGF5Pjo6Zm10OjpoNmUzOTE3OGI2Y2EwYWFmNccBigFhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpzZWFyY2g6OjxpbXBsIGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Ok5vZGVSZWY8Qm9ycm93VHlwZSxLLFYsVHlwZT4+OjpmaW5kX2tleV9pbmRleDo6aDMwNWY5MGFlZjFjYWE3NzXIAWE8c2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6U2VxQWNjZXNzIGFzIHNlcmRlOjpkZTo6U2VxQWNjZXNzPjo6bmV4dF9lbGVtZW50X3NlZWQ6Omg4ODY2NjY1OGExZDJmNjg1yQEOc3RhdGljbm9kZV9uZXfKATZ3YXNtOjpncmFwaDo6U3RhdGljTm9kZTo6c2V0X2NvbmZpZzo6aGQ4Yjc2Njc0NWI4MjVlMTTLAT53YXNtX2JpbmRnZW46Ol9fcnQ6OkxhenlDZWxsPFQsRj46OnRyeV93aXRoOjpoNjNkZDNjYjY4NTYyODhlY8wBugFhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpzZWFyY2g6OjxpbXBsIGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Ok5vZGVSZWY8Qm9ycm93VHlwZSxLLFYsYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpMZWFmT3JJbnRlcm5hbD4+OjpzZWFyY2hfdHJlZTo6aDIzNmI2ZTU5Y2ExZGI0MjXNAW88c3RkOjpwYW5pY2tpbmc6OmJlZ2luX3BhbmljX2hhbmRsZXI6OkZvcm1hdFN0cmluZ1BheWxvYWQgYXMgY29yZTo6cGFuaWM6OlBhbmljUGF5bG9hZD46OmdldDo6aDM5NDIxMjgwZWE5NDYxMTPOAThzdGQ6OnRocmVhZDo6bG9jYWw6OkxvY2FsS2V5PFQ+Ojp3aXRoOjpoYTI4Mjg5ZDEwNmNlZmU5NM8BpQFhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpOb2RlUmVmPGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Om1hcmtlcjo6TXV0LEssVixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkludGVybmFsPjo6cHVzaDo6aDdmYmQ2MTI3NjVhYmI3M2XQAT53YXNtX2JpbmRnZW46Ol9fcnQ6OkxhenlDZWxsPFQsRj46OnRyeV93aXRoOjpoNGYxZjZkMTI1MjgxOTRjNNEBQmFsbG9jOjpyYXdfdmVjOjpSYXdWZWNJbm5lcjxBPjo6dHJ5X2FsbG9jYXRlX2luOjpoODlmZDVjZjc2ODU3ZDlkMtIBQmFsbG9jOjpyYXdfdmVjOjpSYXdWZWNJbm5lcjxBPjo6dHJ5X2FsbG9jYXRlX2luOjpoMDVhZDc2YjM5MGNhNDMwNdMBaDxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXIgYXMgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZXI+OjpkZXNlcmlhbGl6ZV9vcHRpb246Omg5MDM1ZWMyODk3M2U2Zjhk1AFoPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6OkRlc2VyaWFsaXplciBhcyBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplcj46OmRlc2VyaWFsaXplX29wdGlvbjo6aDJmMTdiZDA4OTExZmI3N2bVAWg8c2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6RGVzZXJpYWxpemVyIGFzIHNlcmRlOjpkZTo6RGVzZXJpYWxpemVyPjo6ZGVzZXJpYWxpemVfb3B0aW9uOjpoODAzNTFmZjczNDA3NGI1ZNYBP2hhc2hicm93bjo6cmF3OjpSYXdUYWJsZUlubmVyOjpkcm9wX2VsZW1lbnRzOjpoNjQyYzY5YTY1ZTRlMmIzM9cBaDxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXIgYXMgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZXI+OjpkZXNlcmlhbGl6ZV9zdHJpbmc6Omg4ZTE5OTlmZjYzMzY2MGNm2AFoPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6OkRlc2VyaWFsaXplciBhcyBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplcj46OmRlc2VyaWFsaXplX3N0cmluZzo6aGQ0NzhmNTVjY2U3YWQ0ZWXZAWI8c2VyZGU6OmRlOjp2YWx1ZTo6TWFwRGVzZXJpYWxpemVyPEksRT4gYXMgc2VyZGU6OmRlOjpNYXBBY2Nlc3M+OjpuZXh0X2tleV9zZWVkOjpoY2JmNmY3MGM1NTRlYjY4YdoBP3N0ZDo6c3lzOjpzeW5jOjpvbmNlOjpub190aHJlYWRzOjpPbmNlOjpjYWxsOjpoZDY1Zjc3ZDlhMzgxY2ExMNsBZjxzZXJkZTo6ZGU6OnZhbHVlOjpTZXFEZXNlcmlhbGl6ZXI8SSxFPiBhcyBzZXJkZTo6ZGU6OlNlcUFjY2Vzcz46Om5leHRfZWxlbWVudF9zZWVkOjpoNWJkNjY5ZjllOGVhMDY2MNwBE3N0YXRpY2dyYXBobWV0YV9uZXfdARlzdGF0aWNncmFwaG1ldGFfc2V0QXV0aG9y3gEUc3RhdGljbm9kZV9zZXRfYWxpYXPfARpzdGF0aWNub2RlX3NldF9kZXNjcmlwdGlvbuABGHN0YXRpY25vZGVfc2V0X2ZpZWxkbmFtZeEBG3N0YXRpY25vZGVfc2V0X25vZGVncm91cF9pZOIBHHN0YXRpY25vZGVfc2V0X29udG9sb2d5Y2xhc3PjAR1zdGF0aWNub2RlX3NldF9wYXJlbnRwcm9wZXJ0eeQBKXN0YXRpY25vZGVfc2V0X3NvdXJjZWJyYW5jaHB1YmxpY2F0aW9uX2lk5QErc3RkOjpwYW5pY2tpbmc6OnNldF9ob29rOjpoOWFkODE5Y2Y4YWVhM2FhNuYBLmFsbG9jOjpyYXdfdmVjOjpmaW5pc2hfZ3Jvdzo6aDUwZGFmNmQ4Y2JhYTU3YTnnAS5hbGxvYzo6cmF3X3ZlYzo6ZmluaXNoX2dyb3c6OmhhYzgzYmZjMDg5MTAwZTZl6AE/aGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlSW5uZXI6OmRyb3BfZWxlbWVudHM6OmgwZjRkY2QxZjM4NGI2MThi6QE/aGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlSW5uZXI6OmRyb3BfZWxlbWVudHM6Omg3YzJhNTQzN2E4MGZjM2Vm6gE/aGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlSW5uZXI6OmRyb3BfZWxlbWVudHM6Omg5YWRlZjM2YjU4NDY2MDA06wEuYWxsb2M6OnJhd192ZWM6OmZpbmlzaF9ncm93OjpoZmQ0ODgwOGRkN2QyOTEzNuwBLmFsbG9jOjpyYXdfdmVjOjpmaW5pc2hfZ3Jvdzo6aGMwZWE2ZDg3YmZkYjcyZWLtAUFzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXI6OmFzX2J5dGVzOjpoNGZiOGU5MzYzMjEyOWQ0Y+4BaTx3YXNtOjpncmFwaDo6U3RhdGljR3JhcGhNZXRhIGFzIHdhc21fYmluZGdlbjo6Y29udmVydDo6dHJhaXRzOjpGcm9tV2FzbUFiaT46OmZyb21fYWJpOjpoYjM0MDk1NTM0MGUzYmViYe8BG3N0YXRpY2dyYXBobWV0YV9zZXRfZ3JhcGhpZPABZDx3YXNtOjpncmFwaDo6U3RhdGljTm9kZSBhcyB3YXNtX2JpbmRnZW46OmNvbnZlcnQ6OnRyYWl0czo6RnJvbVdhc21BYmk+Ojpmcm9tX2FiaTo6aGUxMzBiMGE1ZGYxN2FkZDHxARdzdGF0aWNub2RlX3NldF9kYXRhdHlwZfIBF3N0YXRpY25vZGVfc2V0X2dyYXBoX2lk8wETc3RhdGljbm9kZV9zZXRfbmFtZfQBFXN0YXRpY25vZGVfc2V0X25vZGVpZPUBMjxjaGFyIGFzIGNvcmU6OmZtdDo6RGVidWc+OjpmbXQ6OmgwYjY2ZjA0YmY3Y2Q3MDQx9gFcc2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6RGVzZXJpYWxpemVyOjpkZXNlcmlhbGl6ZV9mcm9tX2pzX251bWJlcl91bnNpZ25lZDo6aGQyZWFkZDk1Y2VmN2UwYTb3AWY8c2VyZGU6OmRlOjp2YWx1ZTo6U2VxRGVzZXJpYWxpemVyPEksRT4gYXMgc2VyZGU6OmRlOjpTZXFBY2Nlc3M+OjpuZXh0X2VsZW1lbnRfc2VlZDo6aGQ1MDI0YWZjYjVjNGQ4YTL4AWI8JnNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZXI+OjpzZXJpYWxpemVfaTY0OjpoM2IzOTdjZDQ5NjQwNzM4YvkBgQE8PHNlcmRlOjpkZTo6V2l0aERlY2ltYWxQb2ludCBhcyBjb3JlOjpmbXQ6OkRpc3BsYXk+OjpmbXQ6Okxvb2tGb3JEZWNpbWFsUG9pbnQgYXMgY29yZTo6Zm10OjpXcml0ZT46OndyaXRlX3N0cjo6aDE5MjdjZDIwNTdiZmE0NDn6AW1jb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8Y29yZTo6b3B0aW9uOjpPcHRpb248YWxsb2M6OmJveGVkOjpCb3g8d2FzbTo6Z3JhcGg6OlN0YXRpY05vZGU+Pj46Omg5N2VlMzJhMmM5ODIyN2Fh+wFhPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6OlNlcUFjY2VzcyBhcyBzZXJkZTo6ZGU6OlNlcUFjY2Vzcz46Om5leHRfZWxlbWVudF9zZWVkOjpoMmVmN2M5N2VlZGZmY2NkM/wBrQFhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpOb2RlUmVmPGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Om1hcmtlcjo6TXV0LEssVixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkxlYWY+OjpwdXNoX3dpdGhfaGFuZGxlOjpoZTcyYTYxOWQyZGQ0YmJiYf0BQ3N0ZDo6cGFuaWNraW5nOjpiZWdpbl9wYW5pY19oYW5kbGVyOjp7e2Nsb3N1cmV9fTo6aDI0ZWRiYmNiM2Y5N2Y3NGL+AWI8JnNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZXI+OjpzZXJpYWxpemVfdTY0OjpoZGE5ZmI5NmY1MWQxYWVjYv8BKmpzX3N5czo6SXRlclN0YXRlOjpuZXh0OjpoYWMyNGEwM2IyNmRhNTg4OIACQzx3YXNtX2JpbmRnZW46OkpzVmFsdWUgYXMgY29yZTo6Zm10OjpEZWJ1Zz46OmZtdDo6aDFhNjFjOGVhMzdkMmEyNWGBAlpzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXI6OmRlc2VyaWFsaXplX2Zyb21fanNfbnVtYmVyX3NpZ25lZDo6aGY4MGM2YmU5MDU2MTFhYTKCAm48c2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6Ok9iamVjdFNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplU3RydWN0Pjo6c2VyaWFsaXplX2ZpZWxkOjpoNWRkYzZiYWYzZTVjNjIwNIMCbjxzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6T2JqZWN0U2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVTdHJ1Y3Q+OjpzZXJpYWxpemVfZmllbGQ6OmhmMThmNDM0YTRhNDQ3M2YxhAIxY29yZTo6cHRyOjpzd2FwX25vbm92ZXJsYXBwaW5nOjpoNmI4OGE4YTViZGM4MmUwY4UCOHNlcmRlX3dhc21fYmluZGdlbjo6ZXJyb3I6OkVycm9yOjpuZXc6OmgyODA3OWRlYjE5MWExZTM2hgJuPHNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpPYmplY3RTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZVN0cnVjdD46OnNlcmlhbGl6ZV9maWVsZDo6aDc0NDc0ZWYzYzMwNWFlMDCHAm48c2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6Ok9iamVjdFNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplU3RydWN0Pjo6c2VyaWFsaXplX2ZpZWxkOjpoZjhmYzRhOGJkOTkxNzA0NogCbjxzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6T2JqZWN0U2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVTdHJ1Y3Q+OjpzZXJpYWxpemVfZmllbGQ6OmgzNTRhOGZlZTZiNGZiOTUziQJuPHNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpPYmplY3RTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZVN0cnVjdD46OnNlcmlhbGl6ZV9maWVsZDo6aDQ1MGY4MmJkMzc5NTMyNjeKAm48c2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6Ok9iamVjdFNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplU3RydWN0Pjo6c2VyaWFsaXplX2ZpZWxkOjpoZjViNDdmMjJmNDYyNmQyMYsCUzxzZXJkZV9qc29uOjpudW1iZXI6Ok51bWJlciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemU+OjpzZXJpYWxpemU6OmhjOWYwYWY1YWM1Mjg4NjdijAJEY29yZTo6cHRyOjpkcm9wX2luX3BsYWNlPHdhc206OmdyYXBoOjpTdGF0aWNOb2RlPjo6aDJlNWMwMDJlY2I0ZmQwNWGNAkRjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8d2FzbTo6Z3JhcGg6OlN0YXRpY05vZGU+OjpoMmU1YzAwMmVjYjRmZDA1YY4CvQJhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpuYXZpZ2F0ZTo6PGltcGwgYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6SGFuZGxlPGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Ok5vZGVSZWY8YWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpEeWluZyxLLFYsYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpMZWFmPixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkVkZ2U+Pjo6ZGVhbGxvY2F0aW5nX25leHRfdW5jaGVja2VkOjpoZDM2Nzg1ZjMyZDRmNTUyOY8CaDxzdGQ6OnBhbmlja2luZzo6YmVnaW5fcGFuaWNfaGFuZGxlcjo6Rm9ybWF0U3RyaW5nUGF5bG9hZCBhcyBjb3JlOjpmbXQ6OkRpc3BsYXk+OjpmbXQ6Omg3ZGU1YWQ3MDFmZDRkMjQ1kAJoPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6OkRlc2VyaWFsaXplciBhcyBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplcj46OmRlc2VyaWFsaXplX29wdGlvbjo6aGYxZjI5M2Y2ODgwZWIxMmWRAmhzZXJkZTo6ZGU6OmltcGxzOjo8aW1wbCBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplIGZvciBhbGxvYzo6Ym94ZWQ6OkJveDxUPj46OmRlc2VyaWFsaXplOjpoNGEyZDhkOGE5YTI2MmM5MJICRWNvcmU6OnB0cjo6ZHJvcF9pbl9wbGFjZTxzZXJkZV9qc29uOjplcnJvcjo6RXJyb3I+OjpoNjVmYmI0OGJkZWQwMmQ0N5MCZjxzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6TWFwU2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVNYXA+OjpzZXJpYWxpemVfa2V5OjpoN2IwOTdhM2FiZjBhODQyYZQCLmNvcmU6OnJlc3VsdDo6dW53cmFwX2ZhaWxlZDo6aDg2NjI1MzQzM2QxNTlkMTeVAhtzdGF0aWNncmFwaG1ldGFfZ2V0X2dyYXBoaWSWAhdzdGF0aWNub2RlX2dldF9kYXRhdHlwZZcCF3N0YXRpY25vZGVfZ2V0X2dyYXBoX2lkmAITc3RhdGljbm9kZV9nZXRfbmFtZZkCFXN0YXRpY25vZGVfZ2V0X25vZGVpZJoCFHN0YXRpY25vZGVfc2V0Q29uZmlnmwJjPGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om1hcDo6QlRyZWVNYXA8SyxWLEE+IGFzIGNvcmU6Om9wczo6ZHJvcDo6RHJvcD46OmRyb3A6Omg5NmZjNWIxMGE0NGI3OTJhnAI9YWxsb2M6OnJhd192ZWM6OlJhd1ZlY0lubmVyPEE+OjpkZWFsbG9jYXRlOjpoN2QwNmJhZTc3Mjg2YzVjMJ0CPWFsbG9jOjpyYXdfdmVjOjpSYXdWZWNJbm5lcjxBPjo6ZGVhbGxvY2F0ZTo6aDg1M2EwYTNiNjk5YmFlZTmeAmY8c2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6RGVzZXJpYWxpemVyIGFzIHNlcmRlOjpkZTo6RGVzZXJpYWxpemVyPjo6ZGVzZXJpYWxpemVfYm9vbDo6aDU4ODA1NDcxY2E5Y2ZhYmafAmg8c2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6RGVzZXJpYWxpemVyIGFzIHNlcmRlOjpkZTo6RGVzZXJpYWxpemVyPjo6ZGVzZXJpYWxpemVfb3B0aW9uOjpoOGU1MTcyYjk3ZDc3YmJiYqACZjxzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6TWFwU2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVNYXA+OjpzZXJpYWxpemVfa2V5OjpoMjhmY2EwZTNhODk5YzM5YqECFnN0YXRpY2dyYXBobWV0YV90b0pTT06iAhFzdGF0aWNub2RlX3RvSlNPTqMCFHN0YXRpY25vZGVfZ2V0Q29uZmlnpAJsPHNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpBcnJheVNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplU2VxPjo6c2VyaWFsaXplX2VsZW1lbnQ6OmhmNWJiODU1OTM1MDczMmM0pQJHPGFsbG9jOjpzdHJpbmc6OlN0cmluZyBhcyBjb3JlOjpjbG9uZTo6Q2xvbmU+OjpjbG9uZTo6aGI0NzRkMjJmYTg5NmRiZDGmAjphbGxvYzo6dmVjOjpWZWM8VCxBPjo6dHJ5X3Jlc2VydmVfZXhhY3Q6OmhmOGM5ZTc2MzAxODJhYWZmpwJsPHNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpBcnJheVNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplU2VxPjo6c2VyaWFsaXplX2VsZW1lbnQ6Omg2ZDhlNjgxOWMyYmE3MzJlqAItanNfc3lzOjpVaW50OEFycmF5Ojp0b192ZWM6OmgzNzZjM2U0N2Y2MGE4OGZiqQI/d2FzbV9iaW5kZ2VuOjpjb252ZXJ0OjpjbG9zdXJlczo6aW52b2tlM19tdXQ6Omg1ODM0M2ZiYjQ3YTY3N2RkqgJoPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6OkRlc2VyaWFsaXplciBhcyBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplcj46OmRlc2VyaWFsaXplX29wdGlvbjo6aDkzMDNmNzE5MDgwMDU4MjSrAjlhbGxvYzo6dmVjOjpWZWM8VCxBPjo6aW50b19ib3hlZF9zbGljZTo6aDBkZTM3MTZhMzAxNjY3MTWsAjlhbGxvYzo6dmVjOjpWZWM8VCxBPjo6aW50b19ib3hlZF9zbGljZTo6aGNkZmM2OTY4MmJkMmVmYjWtAjlhbGxvYzo6dmVjOjpWZWM8VCxBPjo6aW50b19ib3hlZF9zbGljZTo6aGU4YjRiZmI2ZjViZmUxNjmuAjZjb3JlOjpwYW5pY2tpbmc6OnBhbmljX2JvdW5kc19jaGVjazo6aDEwMGRmMGI1NzFlYjZjYzWvAjtjb3JlOjpmbXQ6OmJ1aWxkZXJzOjpEZWJ1Z1N0cnVjdDo6ZmluaXNoOjpoNTI0NzdjN2Q1OTcxYmM2ZbACUmNvcmU6OnNsaWNlOjppbmRleDo6c2xpY2VfZW5kX2luZGV4X2xlbl9mYWlsOjpkb19wYW5pYzo6cnVudGltZTo6aDUzMTIwNWI4YWRiM2EwNTaxAlBjb3JlOjpzbGljZTo6aW5kZXg6OnNsaWNlX2luZGV4X29yZGVyX2ZhaWw6OmRvX3BhbmljOjpydW50aW1lOjpoOGU1ZDljYTg0OTUzNTUyMbICRHNlcmRlOjpkZTo6dmFsdWU6Ok1hcERlc2VyaWFsaXplcjxJLEU+OjpuZXh0X3BhaXI6OmhjYTFiMmY3ZWYxYTQwOTIyswJFY29yZTo6cHRyOjpkcm9wX2luX3BsYWNlPHNlcmRlX2pzb246OnZhbHVlOjpWYWx1ZT46OmhjNjgwYTQ4MDdiMDg5NDNhtAJjPGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om1hcDo6SW50b0l0ZXI8SyxWLEE+IGFzIGNvcmU6Om9wczo6ZHJvcDo6RHJvcD46OmRyb3A6Omg2Mjk4ZDI5ZDlhNGMwZmQxtQJFY29yZTo6cHRyOjpkcm9wX2luX3BsYWNlPHNlcmRlX2pzb246OnZhbHVlOjpWYWx1ZT46OmhjNjgwYTQ4MDdiMDg5NDNhtgI3c3RkOjphbGxvYzo6ZGVmYXVsdF9hbGxvY19lcnJvcl9ob29rOjpoNzdmM2E4ZTg2MjU4YzE5Y7cCaDxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXIgYXMgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZXI+OjpkZXNlcmlhbGl6ZV9vcHRpb246OmhkZjNjNmMyMGI5YTdkYzAzuAJPPGhhc2hicm93bjo6cmF3OjpSYXdUYWJsZTxULEE+IGFzIGNvcmU6OmNsb25lOjpDbG9uZT46OmNsb25lOjpoYWJiZjFkYjZkMWEwZGRmNLkCMnNlcmRlOjpkZTo6RXJyb3I6OmludmFsaWRfdmFsdWU6OmhmOGJkNWE4MDc2MDMyZWFjugIxc2VyZGU6OmRlOjpFcnJvcjo6aW52YWxpZF90eXBlOjpoZDEzNWZlNzYwY2YwMWM2ZLsCazx3YXNtOjpncmFwaDo6U3RhdGljTm9kZSBhcyB3YXNtX2JpbmRnZW46OmNvbnZlcnQ6OnRyYWl0czo6UmVmRnJvbVdhc21BYmk+OjpyZWZfZnJvbV9hYmk6Omg5NTAzNDg4ZGUxMTVkOTc2vAIdc3RhdGljZ3JhcGhtZXRhX3NldElzUmVzb3VyY2W9Am48c2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6Ok9iamVjdFNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplU3RydWN0Pjo6c2VyaWFsaXplX2ZpZWxkOjpoZWQ2OGU2YjRiYTFiY2Y2NL4CQ2hhc2hicm93bjo6cmF3OjpSYXdUYWJsZTxULEE+OjpuZXdfdW5pbml0aWFsaXplZDo6aGM5YTVkMDFjYmJhNjI5ZDO/AmFhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpuYXZpZ2F0ZTo6TGF6eUxlYWZSYW5nZTxCb3Jyb3dUeXBlLEssVj46OmluaXRfZnJvbnQ6OmgzZDFkZjRiNjM4NGU2ZmM5wAI8c2VyZGU6OnNlcjo6U2VyaWFsaXplTWFwOjpzZXJpYWxpemVfZW50cnk6OmhmNmEzOWViNTI0YzRlZjI5wQI8c2VyZGU6OnNlcjo6U2VyaWFsaXplTWFwOjpzZXJpYWxpemVfZW50cnk6Omg0MmM2MDQ3ZWFmZjc4NGM3wgI8c2VyZGU6OnNlcjo6U2VyaWFsaXplTWFwOjpzZXJpYWxpemVfZW50cnk6OmgyNDFkY2NhOGZhNmQ1NWY4wwI8c2VyZGU6OnNlcjo6U2VyaWFsaXplTWFwOjpzZXJpYWxpemVfZW50cnk6Omg5MDE2ZmQ3NDFmYjkxNTY2xAJuPHNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpPYmplY3RTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZVN0cnVjdD46OnNlcmlhbGl6ZV9maWVsZDo6aDAyMzI4ZWNlNTE2NTIyMzDFAm48c2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6Ok9iamVjdFNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplU3RydWN0Pjo6c2VyaWFsaXplX2ZpZWxkOjpoMTMxMzU1NzA5ZDNjNGJkMcYCKF9fcnVzdGNbNmRjMDIyY2JkMTRhZTU0ZF06Ol9fcmRsX2RlYWxsb2PHAhVfX3diZ19zdGF0aWNub2RlX2ZyZWXIAm48c2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6Ok9iamVjdFNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplU3RydWN0Pjo6c2VyaWFsaXplX2ZpZWxkOjpoMDQ5OThhMDQ2MjkxMWNmOMkCbjxzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6T2JqZWN0U2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVTdHJ1Y3Q+OjpzZXJpYWxpemVfZmllbGQ6Omg4NGM5MDAzNDg1ODY1YmE5ygJuPHNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpPYmplY3RTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZVN0cnVjdD46OnNlcmlhbGl6ZV9maWVsZDo6aGU2MjZhNTEyOTJmMGIxMznLAjJzZXJkZTo6ZGU6OkVycm9yOjptaXNzaW5nX2ZpZWxkOjpoMDUxMDFiZTI2ZWRiMjRmZswCNHNlcmRlOjpkZTo6RXJyb3I6OmR1cGxpY2F0ZV9maWVsZDo6aGE1ZTEyNmEyZWI1MTE1MDfNAi5jb3JlOjpvcHRpb246OmV4cGVjdF9mYWlsZWQ6Omg0NjMyNjA0NzE3OWFhMjc1zgJyPHdhc206OmdyYXBoOjpTdGF0aWNOb2RlIGFzIHdhc21fYmluZGdlbjo6Y29udmVydDo6dHJhaXRzOjpSZWZNdXRGcm9tV2FzbUFiaT46OnJlZl9tdXRfZnJvbV9hYmk6Omg4YWNjMjZkY2Y3M2VlNTAyzwIZc3RhdGljbm9kZV9zZXRfZXhwb3J0YWJsZdACHXN0YXRpY25vZGVfc2V0X2hhc2N1c3RvbWFsaWFz0QIbc3RhdGljbm9kZV9zZXRfaXNfY29sbGVjdG9y0gIZc3RhdGljbm9kZV9zZXRfaXNyZXF1aXJlZNMCG3N0YXRpY25vZGVfc2V0X2lzc2VhcmNoYWJsZdQCGHN0YXRpY25vZGVfc2V0X2lzdG9wbm9kZdUCN3NlcmRlX3dhc21fYmluZGdlbjo6ZGU6OmNvbnZlcnRfcGFpcjo6aGVmOGM5OTJhNWJhYWMxNDTWAhhzdGF0aWNub2RlX3NldF9zb3J0b3JkZXLXAnw8YWxsb2M6OnZlYzo6VmVjPFQsQT4gYXMgYWxsb2M6OnZlYzo6c3BlY19leHRlbmQ6OlNwZWNFeHRlbmQ8JlQsY29yZTo6c2xpY2U6Oml0ZXI6Okl0ZXI8VD4+Pjo6c3BlY19leHRlbmQ6OmgxZGFjZmQwZWU3YTViNWE12AJFc3RkOjpzeXM6OnJhbmRvbTo6dW5zdXBwb3J0ZWQ6Omhhc2htYXBfcmFuZG9tX2tleXM6OmgxODkyNjk1ZTRiMTNhOTQ12QIPc3RhdGljbm9kZV9jb3B52gISc3RhdGljbm9kZV9jb21wYXJl2wJ8PGFsbG9jOjp2ZWM6OlZlYzxULEE+IGFzIGFsbG9jOjp2ZWM6OnNwZWNfZXh0ZW5kOjpTcGVjRXh0ZW5kPCZULGNvcmU6OnNsaWNlOjppdGVyOjpJdGVyPFQ+Pj46OnNwZWNfZXh0ZW5kOjpoMThlYjZlYzBiNmQ4ZjQxZdwCvgFjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8aGFzaGJyb3duOjpzY29wZWd1YXJkOjpTY29wZUd1YXJkPGhhc2hicm93bjo6cmF3OjpSYXdUYWJsZUlubmVyLGhhc2hicm93bjo6cmF3OjpSYXdUYWJsZUlubmVyOjpwcmVwYXJlX3Jlc2l6ZTxhbGxvYzo6YWxsb2M6Okdsb2JhbD46Ont7Y2xvc3VyZX19Pj46OmgyNzBiMmI1NmFjYThmNmMx3QI1c2VyZGU6OmRlOjpWaXNpdG9yOjp2aXNpdF9ieXRlX2J1Zjo6aGYzMzc3Y2FiNjVmNmMyZWbeAmU8c2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6RGVzZXJpYWxpemVyIGFzIHNlcmRlOjpkZTo6RGVzZXJpYWxpemVyPjo6ZGVzZXJpYWxpemVfaTMyOjpoZWUxODAxN2EzYTFhZjQ2NN8CZTxzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXIgYXMgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZXI+OjpkZXNlcmlhbGl6ZV91MzI6OmhiZDRmODQ4NTUyYTJkOWUx4AJCaGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlSW5uZXI6OmRyb3BfaW5uZXJfdGFibGU6Omg2ZTZjNDgwNmM2NzkxNmE24QJCaGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlSW5uZXI6OmRyb3BfaW5uZXJfdGFibGU6OmhjMDAyMWYzNmJjMWFjMjc04gJCaGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlSW5uZXI6OmRyb3BfaW5uZXJfdGFibGU6Omg1YzZhMjk4ODU5MTk2ZWE24wJCaGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlSW5uZXI6OmRyb3BfaW5uZXJfdGFibGU6Omg2ZWEyMTE5OGIzMjhiYTg25AK+AWFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Ok5vZGVSZWY8YWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpEeWluZyxLLFYsYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6bWFya2VyOjpMZWFmT3JJbnRlcm5hbD46OmRlYWxsb2NhdGVfYW5kX2FzY2VuZDo6aGQ1Nzg0NWIwNzE2MGY4MzDlAkk8YWxsb2M6OnN0cmluZzo6U3RyaW5nIGFzIGNvcmU6OmZtdDo6V3JpdGU+Ojp3cml0ZV9zdHI6Omg0NTI1NGQyOWQ1MjA5YjMw5gI5c3RkOjp0aHJlYWQ6OmxvY2FsOjpwYW5pY19hY2Nlc3NfZXJyb3I6Omg3ZmZhMTYwNTYwZjU0MmMx5wI1Y29yZTo6Y2VsbDo6cGFuaWNfYWxyZWFkeV9ib3Jyb3dlZDo6aDZjMTYzNGJiOWM1ZmE5YmboAgZtZW1jbXDpAkY8c2VyZGVfanNvbjo6bnVtYmVyOjpOIGFzIGNvcmU6OmNtcDo6UGFydGlhbEVxPjo6ZXE6OmhlOGRlNTU3ODU4OTIwOGE36gItX19ydXN0Y1s2ZGMwMjJjYmQxNGFlNTRkXTo6X19yZGxfYWxsb2NfemVyb2Vk6wJBaGFzaGJyb3duOjpyYXc6OkZhbGxpYmlsaXR5OjpjYXBhY2l0eV9vdmVyZmxvdzo6aGFlMWRkNjhlNDE3ZWFiMmLsAkk8YWxsb2M6OnN0cmluZzo6U3RyaW5nIGFzIGNvcmU6OmZtdDo6V3JpdGU+Ojp3cml0ZV9zdHI6Omg0NTI1NGQyOWQ1MjA5YjMw7QI2anNfc3lzOjpVaW50OEFycmF5OjpyYXdfY29weV90b19wdHI6OmhlZWE2MWQwNmFjMjQ1Njkz7gKyAmFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5hdmlnYXRlOjo8aW1wbCBhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpIYW5kbGU8YWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6Tm9kZVJlZjxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkR5aW5nLEssVixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkxlYWY+LGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Om1hcmtlcjo6RWRnZT4+OjpkZWFsbG9jYXRpbmdfZW5kOjpoODcxYjI4NGQ0ODdjNDgxZe8COGFsbG9jOjpyYXdfdmVjOjpSYXdWZWM8VCxBPjo6Z3Jvd19vbmU6OmhjMzc3YzdjYjczZTcwZWJj8AI5d2FzbV9iaW5kZ2VuOjpleHRlcm5yZWY6OlNsYWI6OmRlYWxsb2M6OmhmZmVlMzNkMGM5MDNkZmNm8QI4YWxsb2M6OnJhd192ZWM6OlJhd1ZlYzxULEE+Ojpncm93X29uZTo6aGRiNTE1M2E2ZDI1OGEwOTPyAh1zdGF0aWNncmFwaG1ldGFfZ2V0SXNSZXNvdXJjZfMCXWNvcmU6OnB0cjo6ZHJvcF9pbl9wbGFjZTwoYWxsb2M6OnN0cmluZzo6U3RyaW5nLHNlcmRlX2pzb246OnZhbHVlOjpWYWx1ZSk+OjpoNmIwZjkzZjA2NDE2MzgyNPQCBG1haW71AlY8anNfc3lzOjpBcnJheUl0ZXIgYXMgY29yZTo6aXRlcjo6dHJhaXRzOjppdGVyYXRvcjo6SXRlcmF0b3I+OjpuZXh0OjpoYTBiOTRkNTQ0NTM0NzEzOPYCUWFsbG9jOjpyYXdfdmVjOjpSYXdWZWNJbm5lcjxBPjo6cmVzZXJ2ZTo6ZG9fcmVzZXJ2ZV9hbmRfaGFuZGxlOjpoNTMwOWRmM2FhNTUxMThjMvcCUWFsbG9jOjpyYXdfdmVjOjpSYXdWZWNJbm5lcjxBPjo6cmVzZXJ2ZTo6ZG9fcmVzZXJ2ZV9hbmRfaGFuZGxlOjpoODA2MTFiMzczNmY4NDdiNPgCOHN0ZDo6cGFuaWNraW5nOjpwYW5pY19jb3VudDo6aW5jcmVhc2U6OmhhOWFjN2Q4NzcyZWIyOTkz+QJxPHN0ZDo6cGFuaWNraW5nOjpiZWdpbl9wYW5pY19oYW5kbGVyOjpTdGF0aWNTdHJQYXlsb2FkIGFzIGNvcmU6OnBhbmljOjpQYW5pY1BheWxvYWQ+Ojp0YWtlX2JveDo6aDRkYzQxN2ZmYjFlNzdhZmP6AjJ3YXNtX2JpbmRnZW46OmJpZ2ludF9nZXRfYXNfaTY0OjpoYmE5MTlhMmViM2Q4MzcxOPsCiAF3YXNtX2JpbmRnZW46OmNvbnZlcnQ6OnNsaWNlczo6PGltcGwgd2FzbV9iaW5kZ2VuOjpjb252ZXJ0Ojp0cmFpdHM6OkZyb21XYXNtQWJpIGZvciBhbGxvYzo6Ym94ZWQ6OkJveDxbVF0+Pjo6ZnJvbV9hYmk6OmhmMzFkMTQ2YjliZjJjZjhm/AJLY29yZTo6Zm10OjpmbG9hdDo6PGltcGwgY29yZTo6Zm10OjpEaXNwbGF5IGZvciBmNjQ+OjpmbXQ6Omg0MDA3OWY5N2EwYzBmZDA0/QIoYWxsb2M6OnJjOjpSYzxUPjo6bmV3OjpoNWQ0YWJkNjk5NGMwODk2MP4CKGFsbG9jOjpyYzo6UmM8VD46Om5ldzo6aDg1MDFlY2U4OWU1MjMzOWH/Ailjb3JlOjpwYW5pY2tpbmc6OnBhbmljOjpoMTg4ZmJjOTVlN2M3NGFiZYADEV9fd2JpbmRnZW5fbWFsbG9jgQNFY29yZTo6cHRyOjpkcm9wX2luX3BsYWNlPHNlcmRlX2pzb246OnZhbHVlOjpWYWx1ZT46Omg1ODA2NDdlMzFjMDExZGMzggNiPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6Ok9iamVjdEFjY2VzcyBhcyBzZXJkZTo6ZGU6Ok1hcEFjY2Vzcz46Om5leHRfdmFsdWVfc2VlZDo6aGJiOTYyNDk3YzYyNTE3NzCDA0Vjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8c2VyZGVfanNvbjo6dmFsdWU6OlZhbHVlPjo6aDU4MDY0N2UzMWMwMTFkYzOEA1tjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8Y29yZTo6b3B0aW9uOjpPcHRpb248c2VyZGVfanNvbjo6dmFsdWU6OlZhbHVlPj46Omg4ZGQ5N2JlNjdiMDQ2YzJhhQNVPGpzX3N5czo6SW50b0l0ZXIgYXMgY29yZTo6aXRlcjo6dHJhaXRzOjppdGVyYXRvcjo6SXRlcmF0b3I+OjpuZXh0OjpoZjBkMTY0YjYzNTkyNDg2OIYDSDxkbG1hbGxvYzo6c3lzOjpTeXN0ZW0gYXMgZGxtYWxsb2M6OkFsbG9jYXRvcj46OmFsbG9jOjpoYWE1NTI2Nzg5NGFjMmU4ZIcDGXN0YXRpY25vZGVfZ2V0X2V4cG9ydGFibGWIAx1zdGF0aWNub2RlX2dldF9oYXNjdXN0b21hbGlhc4kDG3N0YXRpY25vZGVfZ2V0X2lzX2NvbGxlY3RvcooDGXN0YXRpY25vZGVfZ2V0X2lzcmVxdWlyZWSLAxtzdGF0aWNub2RlX2dldF9pc3NlYXJjaGFibGWMAxhzdGF0aWNub2RlX2dldF9pc3RvcG5vZGWNA2E8YWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bWFwOjpCVHJlZU1hcDxLLFYsQT4gYXMgY29yZTo6Y2xvbmU6OkNsb25lPjo6Y2xvbmU6OmgzNGUwMGQzNDJkODA4NGQ4jgNiPCZzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6U2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVyPjo6c2VyaWFsaXplX21hcDo6aGFlMWYyMGY4YWFhZGYxMTGPAxlfX2V4dGVybnJlZl90YWJsZV9kZWFsbG9jkANDY29yZTo6Zm10OjpGb3JtYXR0ZXI6OnBhZF9pbnRlZ3JhbDo6d3JpdGVfcHJlZml4OjpoM2ViMjNlNTU0ZDk4MWNiNpEDX2NvcmU6OnB0cjo6ZHJvcF9pbl9wbGFjZTx3YXNtX2JpbmRnZW46Ol9fcnQ6OlJjUmVmPHdhc206OmdyYXBoOjpTdGF0aWNOb2RlPj46OmhkNDJjYmM4ZGZmZDc1MTAzkgNkY29yZTo6cHRyOjpkcm9wX2luX3BsYWNlPHdhc21fYmluZGdlbjo6X19ydDo6UmNSZWY8d2FzbTo6Z3JhcGg6OlN0YXRpY0dyYXBoTWV0YT4+OjpoZGUzMDM4MjJmMzIzYmM5OJMDMGFsbG9jOjpyYzo6UmM8VCxBPjo6ZHJvcF9zbG93OjpoMTM2OTgyYjcxNDJmM2JlN5QDU2NvcmU6OnB0cjo6c3dhcF9ub25vdmVybGFwcGluZ19ieXRlczo6c3dhcF9ub25vdmVybGFwcGluZ19jaHVua3M6OmgyM2E0MmYwNGZlZThjM2M5lQM4anNfc3lzOjpJdGVyYXRvcjo6bG9va3NfbGlrZV9pdGVyYXRvcjo6aDhmOTNlM2RmMDBjOTc5NzCWA68BYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6Tm9kZVJlZjxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6Ok93bmVkLEssVixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkludGVybmFsPjo6bmV3X2ludGVybmFsOjpoZTJkZDI4NzAzOTEzM2M2YpcDMWNvcmU6OnBhbmlja2luZzo6YXNzZXJ0X2ZhaWxlZDo6aGI2ZDgzYzY0ZjQ0NTk2MjGYAxhzdGF0aWNub2RlX2dldF9zb3J0b3JkZXKZA0g8YWxsb2M6OnZlYzo6VmVjPFQsQT4gYXMgY29yZTo6b3BzOjpkcm9wOjpEcm9wPjo6ZHJvcDo6aGFjYjRiOWNhMDA5MWQ3YTOaA0pzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXI6OmFzX29iamVjdF9lbnRyaWVzOjpoNWM5MTI2ZDg3M2IyNWFhY5sDEl9fd2JpbmRnZW5fcmVhbGxvY5wDNGFsbG9jOjpyYXdfdmVjOjpjYXBhY2l0eV9vdmVyZmxvdzo6aGZhZWVhYWNhNmZhMDhhZmKdA0hjb3JlOjpwYW5pY2tpbmc6OnBhbmljX2NvbnN0OjpwYW5pY19jb25zdF9kaXZfYnlfemVybzo6aGIxZjUyYWFmMzVlMDVjMGaeA05jb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8c2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6TWFwQWNjZXNzPjo6aGU4ZjZmYjQwOGZmYjcyYjafA1Fjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8c2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6T2JqZWN0QWNjZXNzPjo6aDgwMmE5MmU2YzRlMjk3NjigA1Njb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8c2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6Ok1hcFNlcmlhbGl6ZXI+OjpoY2ZhMmIyZTk5ZDMzM2QzMKEDXDxzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6TWFwU2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVNYXA+OjplbmQ6OmgzM2FlNmY0NTJjOWJkN2VjogM3c2VyZGVfd2FzbV9iaW5kZ2VuOjpzdGF0aWNfc3RyX3RvX2pzOjpoYzcyYzlhN2ViZWViNzc4MaMDTTxzdGQ6OnRocmVhZDo6bG9jYWw6OkFjY2Vzc0Vycm9yIGFzIGNvcmU6OmZtdDo6RGVidWc+OjpmbXQ6OmgyNmYwYzQzNzk4NjU0NzEwpAP3AWFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6OkhhbmRsZTxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpOb2RlUmVmPGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Om1hcmtlcjo6TXV0LEssVixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkxlYWY+LGFsbG9jOjpjb2xsZWN0aW9uczo6YnRyZWU6Om5vZGU6Om1hcmtlcjo6S1Y+OjpzcGxpdDo6aGQ5OWVhNjA2NTE2NzdiMDelA0g8YWxsb2M6OnZlYzo6VmVjPFQsQT4gYXMgY29yZTo6b3BzOjpkcm9wOjpEcm9wPjo6ZHJvcDo6aDJjN2M0ODg5MjI3YjgxNmSmAxZfX2V4dGVybnJlZl9kcm9wX3NsaWNlpwMsX19ydXN0Y1s2ZGMwMjJjYmQxNGFlNTRkXTo6cnVzdF9iZWdpbl91bndpbmSoAy1jb3JlOjpwYW5pY2tpbmc6OnBhbmljX2ZtdDo6aGYxMjQxNmViMDFhNDY1ZWGpAzFzdGF0aWNub2RlX2NvbXBhcmUgZXh0ZXJucmVmIHNoaW0gbXVsdGl2YWx1ZSBzaGltqgNkPHNlcmRlOjpkZTo6dmFsdWU6Ok1hcERlc2VyaWFsaXplcjxJLEU+IGFzIHNlcmRlOjpkZTo6TWFwQWNjZXNzPjo6bmV4dF92YWx1ZV9zZWVkOjpoNTA0NGRlYTFhYzQxOTgxM6sDSHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6OkRlc2VyaWFsaXplcjo6YXNfc2FmZV9pbnRlZ2VyOjpoMTczYmMwYzI2N2YzZjEyY6wDN3NlcmRlX2pzb246Om51bWJlcjo6TnVtYmVyOjpmcm9tX2Y2NDo6aGJkMjM3OTBkZjMxZTlmNGKtAzVjb3JlOjpmbXQ6OkZvcm1hdHRlcjo6ZGVidWdfc3RydWN0OjpoMjcwYmQzMWE0ZTFhM2Q2Nq4DMnN0YXRpY2dyYXBobWV0YV9uZXcgZXh0ZXJucmVmIHNoaW0gbXVsdGl2YWx1ZSBzaGltrwMmc3RhdGljZ3JhcGhtZXRhX3RvSlNPTiBtdWx0aXZhbHVlIHNoaW2wAy1zdGF0aWNub2RlX25ldyBleHRlcm5yZWYgc2hpbSBtdWx0aXZhbHVlIHNoaW2xAyFzdGF0aWNub2RlX3RvSlNPTiBtdWx0aXZhbHVlIHNoaW2yAyRzdGF0aWNub2RlX2dldENvbmZpZyBtdWx0aXZhbHVlIHNoaW2zA3Bjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8Y29yZTo6b3B0aW9uOjpPcHRpb248KGFsbG9jOjpzdHJpbmc6OlN0cmluZyxhbGxvYzo6c3RyaW5nOjpTdHJpbmcpPj46Omg5OTQ3M2I0N2QzODk1MjE1tANfPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6Ok1hcEFjY2VzcyBhcyBzZXJkZTo6ZGU6Ok1hcEFjY2Vzcz46Om5leHRfdmFsdWVfc2VlZDo6aDg2M2FhOGJiMTE3YWZhZTC1A3Bjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8KGFsbG9jOjpzdHJpbmc6OlN0cmluZyxjb3JlOjpvcHRpb246Ok9wdGlvbjxhbGxvYzo6c3RyaW5nOjpTdHJpbmc+KT46OmgwODUyMGI5ZTRmM2Y0ZmQ5tgM6d2FzbV9iaW5kZ2VuOjpfX3J0Ojp0YWtlX2xhc3RfZXhjZXB0aW9uOjpoZjY4YTg2NzJiZjlkZTAzYrcDTmNvcmU6OmZtdDo6bnVtOjppbXA6OjxpbXBsIGNvcmU6OmZtdDo6RGlzcGxheSBmb3IgaTMyPjo6Zm10OjpoZGQyYmNhNzU3YmZiZWJlOLgDM3N0YXRpY25vZGVfc2V0Q29uZmlnIGV4dGVybnJlZiBzaGltIG11bHRpdmFsdWUgc2hpbbkDPHNlcmRlX3dhc21fYmluZGdlbjo6ZGU6Ok9iamVjdEFjY2Vzczo6bmV3OjpoNzRmZWIyYzk2NGU5YWFmZboD3QFhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjpIYW5kbGU8YWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6Tm9kZVJlZjxCb3Jyb3dUeXBlLEssVixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkludGVybmFsPixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkVkZ2U+OjpkZXNjZW5kOjpoMTgxNmM2NGNlYWJiZjM4ZbsDK3N0YXRpY2dyYXBobWV0YV9nZXRfZ3JhcGhpZCBtdWx0aXZhbHVlIHNoaW28AylzdGF0aWNncmFwaG1ldGFfZ2V0QXV0aG9yIG11bHRpdmFsdWUgc2hpbb0DJHN0YXRpY25vZGVfZ2V0X2FsaWFzIG11bHRpdmFsdWUgc2hpbb4DJ3N0YXRpY25vZGVfZ2V0X2RhdGF0eXBlIG11bHRpdmFsdWUgc2hpbb8DKnN0YXRpY25vZGVfZ2V0X2Rlc2NyaXB0aW9uIG11bHRpdmFsdWUgc2hpbcADKHN0YXRpY25vZGVfZ2V0X2ZpZWxkbmFtZSBtdWx0aXZhbHVlIHNoaW3BAydzdGF0aWNub2RlX2dldF9ncmFwaF9pZCBtdWx0aXZhbHVlIHNoaW3CAyNzdGF0aWNub2RlX2dldF9uYW1lIG11bHRpdmFsdWUgc2hpbcMDK3N0YXRpY25vZGVfZ2V0X25vZGVncm91cF9pZCBtdWx0aXZhbHVlIHNoaW3EAyVzdGF0aWNub2RlX2dldF9ub2RlaWQgbXVsdGl2YWx1ZSBzaGltxQMsc3RhdGljbm9kZV9nZXRfb250b2xvZ3ljbGFzcyBtdWx0aXZhbHVlIHNoaW3GAy1zdGF0aWNub2RlX2dldF9wYXJlbnRwcm9wZXJ0eSBtdWx0aXZhbHVlIHNoaW3HAzlzdGF0aWNub2RlX2dldF9zb3VyY2VicmFuY2hwdWJsaWNhdGlvbl9pZCBtdWx0aXZhbHVlIHNoaW3IA0J3YXNtX2JpbmRnZW46OmNvbnZlcnQ6OnRyYWl0czo6V2FzbVJldDxUPjo6am9pbjo6aGUxOWM3NTZiNWE3ZDFjMzPJAz93YXNtX2JpbmRnZW46OmNvbnZlcnQ6OmNsb3N1cmVzOjppbnZva2U0X211dDo6aDkxNDliNjljNTNmM2RjZTnKA05jb3JlOjpmbXQ6Om51bTo6aW1wOjo8aW1wbCBjb3JlOjpmbXQ6OkRpc3BsYXkgZm9yIGk2ND46OmZtdDo6aDFiYmJjMDhmMzEzMGJlYmXLA3Njb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8Y29yZTo6b3B0aW9uOjpPcHRpb248KGFsbG9jOjpzdHJpbmc6OlN0cmluZyxzZXJkZV9qc29uOjp2YWx1ZTo6VmFsdWUpPj46OmhlZjNiZWY2MDdmYzFkYjBlzAOGAWNvcmU6OnB0cjo6ZHJvcF9pbl9wbGFjZTxjb3JlOjpvcHRpb246Ok9wdGlvbjwoYWxsb2M6OnN0cmluZzo6U3RyaW5nLGNvcmU6Om9wdGlvbjo6T3B0aW9uPGFsbG9jOjpzdHJpbmc6OlN0cmluZz4pPj46OmgyM2M0NzczMjVjYTEyZTQxzQNIY29yZTo6b3BzOjpmdW5jdGlvbjo6Rm5PbmNlOjpjYWxsX29uY2V7e3Z0YWJsZS5zaGltfX06Omg2MTc5YTk5OTc0NGJkMDc1zgNIY29yZTo6b3BzOjpmdW5jdGlvbjo6Rm5PbmNlOjpjYWxsX29uY2V7e3Z0YWJsZS5zaGltfX06Omg4MWIyMTUzZWJlODQ1ZTYyzwNIY29yZTo6b3BzOjpmdW5jdGlvbjo6Rm5PbmNlOjpjYWxsX29uY2V7e3Z0YWJsZS5zaGltfX06OmhhZTVhNTcyYTcxMDdjMWM10ANNaGFzaGJyb3duOjpyYXc6OlJhd1RhYmxlPFQsQT46OnJlc2VydmVfcmVoYXNoOjp7e2Nsb3N1cmV9fTo6aDE0OWU0ODA3YjdjZjZkMjnRA01oYXNoYnJvd246OnJhdzo6UmF3VGFibGU8VCxBPjo6cmVzZXJ2ZV9yZWhhc2g6Ont7Y2xvc3VyZX19OjpoNmZlMmE0MmI0NDlhZTFiM9IDTWhhc2hicm93bjo6cmF3OjpSYXdUYWJsZTxULEE+OjpyZXNlcnZlX3JlaGFzaDo6e3tjbG9zdXJlfX06Omg5MTg4OTVlZjU5YTIyNmE20wNzY29yZTo6cHRyOjpkcm9wX2luX3BsYWNlPGNvcmU6Om9wdGlvbjo6T3B0aW9uPChhbGxvYzo6c3RyaW5nOjpTdHJpbmcsc2VyZGVfanNvbjo6dmFsdWU6OlZhbHVlKT4+OjpoZWYzYmVmNjA3ZmMxZGIwZdQDP3dhc21fYmluZGdlbjo6Y29udmVydDo6Y2xvc3VyZXM6Omludm9rZTNfbXV0OjpoMTI3MDk3ODE0NWY4NmExNNUDP3dhc21fYmluZGdlbjo6Y29udmVydDo6Y2xvc3VyZXM6Omludm9rZTNfbXV0OjpoMTdmZWIyODAxMjkyZTAyZNYDP3dhc21fYmluZGdlbjo6Y29udmVydDo6Y2xvc3VyZXM6Omludm9rZTNfbXV0OjpoMWY3MWVjM2NmNTRhYzM4M9cDP3dhc21fYmluZGdlbjo6Y29udmVydDo6Y2xvc3VyZXM6Omludm9rZTNfbXV0OjpoMzZmNmE5Y2VjY2I2OTZhNdgDP3dhc21fYmluZGdlbjo6Y29udmVydDo6Y2xvc3VyZXM6Omludm9rZTNfbXV0OjpoNDczZTcxZTZiMjE4Y2Q2NNkDP3dhc21fYmluZGdlbjo6Y29udmVydDo6Y2xvc3VyZXM6Omludm9rZTNfbXV0OjpoNTExNjlmMTE5ZmUyOTBjZtoDP3dhc21fYmluZGdlbjo6Y29udmVydDo6Y2xvc3VyZXM6Omludm9rZTNfbXV0OjpoNWE3ZWI0NmRkMDYzYWY3M9sDP3dhc21fYmluZGdlbjo6Y29udmVydDo6Y2xvc3VyZXM6Omludm9rZTNfbXV0OjpoOTYxODIxZDg1ZTQzYWYyN9wDOGFsbG9jOjpib3hlZDo6Qm94PFQsQT46Om5ld191bmluaXRfaW46Omg3NDA2OTYxNzUwNzczNDQx3QM4YWxsb2M6OmJveGVkOjpCb3g8VCxBPjo6bmV3X3VuaW5pdF9pbjo6aGY3MjQ0ZjFiMTVjNzQzZjPeA2Vjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8c3RkOjpwYW5pY2tpbmc6OmJlZ2luX3BhbmljX2hhbmRsZXI6OkZvcm1hdFN0cmluZ1BheWxvYWQ+OjpoNWJiZTVlODUzOWZiZjM0M98DRjxbQV0gYXMgY29yZTo6c2xpY2U6OmNtcDo6U2xpY2VQYXJ0aWFsRXE8Qj4+OjplcXVhbDo6aDExZWE4NzcwMjBlNWI5MzbgAzloYXNoYnJvd246OnJhdzo6RmFsbGliaWxpdHk6OmFsbG9jX2Vycjo6aGIwMzdlZmUyYTdkY2IxMjThAz93YXNtX2JpbmRnZW46OmNvbnZlcnQ6OmNsb3N1cmVzOjppbnZva2UyX211dDo6aDE3Yzg3ZDRkNjJjMjIxNWbiA6cBYWxsb2M6OmNvbGxlY3Rpb25zOjpidHJlZTo6bm9kZTo6Tm9kZVJlZjxhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6Ok93bmVkLEssVixhbGxvYzo6Y29sbGVjdGlvbnM6OmJ0cmVlOjpub2RlOjptYXJrZXI6OkxlYWY+OjpuZXdfbGVhZjo6aGVlOTFhZWI5ZGUzODcyODnjA2I8JmFsbG9jOjp2ZWM6OlZlYzxULEE+IGFzIGNvcmU6Oml0ZXI6OnRyYWl0czo6Y29sbGVjdDo6SW50b0l0ZXJhdG9yPjo6aW50b19pdGVyOjpoODUyMWUxNDk1NTVjNzY2Y+QDNDxib29sIGFzIGNvcmU6OmZtdDo6RGlzcGxheT46OmZtdDo6aGQ5Y2E2MGViZDA2ZDdmNTLlAylfX3diZ19zZXRfOGZjNmJmOGE1YjEwNzFkMSBleHRlcm5yZWYgc2hpbeYDP3dhc21fYmluZGdlbjo6Y29udmVydDo6Y2xvc3VyZXM6Omludm9rZTFfbXV0OjpoMjAxZmVmYzE2NDExY2UwZecDMHNlcmRlOjpzZXI6Oml0ZXJhdG9yX2xlbl9oaW50OjpoYTI5YWM5MTA0YTQ3MDUzNegDggE8PHNlcmRlOjpkZTo6V2l0aERlY2ltYWxQb2ludCBhcyBjb3JlOjpmbXQ6OkRpc3BsYXk+OjpmbXQ6Okxvb2tGb3JEZWNpbWFsUG9pbnQgYXMgY29yZTo6Zm10OjpXcml0ZT46OndyaXRlX2NoYXI6Omg2Nzc1MzYxMDI4MzliODY16QMhc3RhdGljbm9kZV9jb21wYXJlIGV4dGVybnJlZiBzaGlt6gNaY29yZTo6cHRyOjpkcm9wX2luX3BsYWNlPChhbGxvYzo6c3RyaW5nOjpTdHJpbmcsYWxsb2M6OnN0cmluZzo6U3RyaW5nKT46OmhmOWZhNDA4NGFjYzEwMjZm6wM5Y29yZTo6b3BzOjpmdW5jdGlvbjo6Rm5PbmNlOjpjYWxsX29uY2U6OmhlNDc4YTQyZTJhOTQ2Y2Jm7ANiPCZzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6U2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVyPjo6c2VyaWFsaXplX3NlcTo6aGQ1MDcwYWY5ZTNmMTIzNTPtAyZqc19zeXM6OkFycmF5OjppdGVyOjpoZDkwMDE2ZGZlMDUxMTc1Ye4DYDxzZXJkZV9qc29uOjp2YWx1ZTo6c2VyOjpTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZXI+OjpzZXJpYWxpemVfaTY0OjpoZGU5ZjA5MzM5MGU1ZmRiMe8DdWNvcmU6OnB0cjo6ZHJvcF9pbl9wbGFjZTxjb3JlOjpyZXN1bHQ6OlJlc3VsdDwoKSxjb3JlOjpjZWxsOjpDZWxsPHdhc21fYmluZGdlbjo6ZXh0ZXJucmVmOjpTbGFiPj4+OjpoOTczYzM5ODFiODA1OTQ2MvADQmNvcmU6OnB0cjo6ZHJvcF9pbl9wbGFjZTxhbGxvYzo6c3RyaW5nOjpTdHJpbmc+OjpoOTliYzQ1YTEzMzJkZTcxNfEDVmNvcmU6OnB0cjo6ZHJvcF9pbl9wbGFjZTxjb3JlOjpvcHRpb246Ok9wdGlvbjxhbGxvYzo6dmVjOjpWZWM8dTg+Pj46OmgyYzc1MGY5ZWY5NTE4MzY58gMmX19ydXN0Y1s2ZGMwMjJjYmQxNGFlNTRkXTo6X19yZGxfYWxsb2PzA0Jjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8YWxsb2M6OnN0cmluZzo6U3RyaW5nPjo6aDMwNDJlMWUzZThkYjJmMzP0Ay9hbGxvYzo6cmF3X3ZlYzo6aGFuZGxlX2Vycm9yOjpoYjE5OWJmZDkzNWQ5OTViNfUDM19fd2JnX2dldHdpdGhyZWZrZXlfMWRjMzYxYmQxMDA1M2JmZSBleHRlcm5yZWYgc2hpbfYDKV9fd2JnX3NldF8zZjFkMGI5ODRlZDI3MmVkIGV4dGVybnJlZiBzaGlt9wMpX193YmdfZ2V0XzY3YjJiYTYyZmMzMGRlMTIgZXh0ZXJucmVmIHNoaW34AypfX3diZ19jYWxsXzY3MmE0ZDIxNjM0ZDRhMjQgZXh0ZXJucmVmIHNoaW35A1hjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8Y29yZTo6b3B0aW9uOjpPcHRpb248YWxsb2M6OnN0cmluZzo6U3RyaW5nPj46OmhhY2RjNmM2YTQxNDY3NDQ5+gNYY29yZTo6cHRyOjpkcm9wX2luX3BsYWNlPGNvcmU6Om9wdGlvbjo6T3B0aW9uPGFsbG9jOjpzdHJpbmc6OlN0cmluZz4+OjpoYWNkYzZjNmE0MTQ2NzQ0OfsDWGNvcmU6OnB0cjo6ZHJvcF9pbl9wbGFjZTxjb3JlOjpvcHRpb246Ok9wdGlvbjxhbGxvYzo6c3RyaW5nOjpTdHJpbmc+Pjo6aGFjZGM2YzZhNDE0Njc0NDn8A1hjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8Y29yZTo6b3B0aW9uOjpPcHRpb248YWxsb2M6OnN0cmluZzo6U3RyaW5nPj46OmhhY2RjNmM2YTQxNDY3NDQ5/QNjPCZzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6U2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVyPjo6c2VyaWFsaXplX25vbmU6OmhkZmU1ZjkxYzhkNGFkNWVh/gMjX19ydXN0Y1s2ZGMwMjJjYmQxNGFlNTRkXTo6X19yZ19vb23/A0Njb3JlOjphbGxvYzo6bGF5b3V0OjpMYXlvdXQ6OmlzX3NpemVfYWxpZ25fdmFsaWQ6OmhlMTU3MDU2MDI5YTc4OTE1gAQpX193YmdfZ2V0X2I5YjkzMDQ3ZmUzY2Y0NWIgZXh0ZXJucmVmIHNoaW2BBGM8JnNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZXI+OjpzZXJpYWxpemVfYm9vbDo6aGY0NDk4MzNjMTAxNzE0YWWCBDljb3JlOjpvcHM6OmZ1bmN0aW9uOjpGbk9uY2U6OmNhbGxfb25jZTo6aGRiMjgzMmFhM2Q4YWNmMWGDBGA8YWxsb2M6OnZlYzo6VmVjPFQsQT4gYXMgY29yZTo6Y29udmVydDo6RnJvbTxhbGxvYzo6Ym94ZWQ6OkJveDxbVF0sQT4+Pjo6ZnJvbTo6aGViZWIwYjMwMzI1MjBhZGaEBCNfX3diaW5kZ2VuX2Vycm9yX25ldyBleHRlcm5yZWYgc2hpbYUEJF9fd2JpbmRnZW5fc3RyaW5nX25ldyBleHRlcm5yZWYgc2hpbYYEKl9fd2JnX25leHRfMjVmZWFkZmMwOTEzZmVhOSBleHRlcm5yZWYgc2hpbYcEKl9fd2JnX25leHRfNjU3NGUxYThhNjJkMTA1NSBleHRlcm5yZWYgc2hpbYgEK19fd2JnX3ZhbHVlX2NkMWZmYTdiMWFiNzk0ZjEgZXh0ZXJucmVmIHNoaW2JBC1fX3diZ19lbnRyaWVzXzMyNjVkNDE1OGIzM2U1ZGMgZXh0ZXJucmVmIHNoaW2KBCxfX3diZ19idWZmZXJfNjA5Y2MzZWVlNTFlZDE1OCBleHRlcm5yZWYgc2hpbYsEKV9fd2JnX25ld19hMTIwMDJhN2Y5MWM3NWJlIGV4dGVybnJlZiBzaGltjAQpX19ydXN0Y1s2ZGMwMjJjYmQxNGFlNTRkXTo6X19ydXN0X3JlYWxsb2ONBGI8JnNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZXI+OjpzZXJpYWxpemVfaTMyOjpoMWE3YmQ3YWI4M2QzODIwMY4EYjwmc2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6OlNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplcj46OnNlcmlhbGl6ZV91MzI6Omg1M2U0OWM1NGE0MzM0ZWY2jwRiPCZzZXJkZV93YXNtX2JpbmRnZW46OnNlcjo6U2VyaWFsaXplciBhcyBzZXJkZTo6c2VyOjpTZXJpYWxpemVyPjo6c2VyaWFsaXplX3N0cjo6aDhiMzg2YTE1YWM0YzUzZDGQBEk8YWxsb2M6OnN0cmluZzo6U3RyaW5nIGFzIGNvcmU6OmZtdDo6V3JpdGU+Ojp3cml0ZV9zdHI6Omg0NTI1NGQyOWQ1MjA5YjMwkQRJPGFsbG9jOjpzdHJpbmc6OlN0cmluZyBhcyBjb3JlOjpmbXQ6OldyaXRlPjo6d3JpdGVfc3RyOjpoNDUyNTRkMjlkNTIwOWIzMJIESTxhbGxvYzo6c3RyaW5nOjpTdHJpbmcgYXMgY29yZTo6Zm10OjpXcml0ZT46OndyaXRlX3N0cjo6aDQ1MjU0ZDI5ZDUyMDliMzCTBA9fX3diaW5kZ2VuX2ZyZWWUBBRfX3diaW5kZ2VuX2V4bl9zdG9yZZUEJV9fcnVzdGNbNmRjMDIyY2JkMTRhZTU0ZF06OnJ1c3RfcGFuaWOWBEg8Y29yZTo6Y2VsbDo6Qm9ycm93TXV0RXJyb3IgYXMgY29yZTo6Zm10OjpEZWJ1Zz46OmZtdDo6aDE4MGQwZjRjMWMxMjgwMDeXBDJjb3JlOjpmbXQ6OkZvcm1hdHRlcjo6d3JpdGVfc3RyOjpoNDZkMWFmMDg1OTVmYTRlNpgEKV9fd2JpbmRnZW5fYmlnaW50X2Zyb21faTY0IGV4dGVybnJlZiBzaGltmQQpX193YmluZGdlbl9iaWdpbnRfZnJvbV91NjQgZXh0ZXJucmVmIHNoaW2aBCRfX3diaW5kZ2VuX251bWJlcl9uZXcgZXh0ZXJucmVmIHNoaW2bBClfX3diZ19zZXRfMzc4MzcwMjNmM2Q3NDBlOCBleHRlcm5yZWYgc2hpbZwEI3N0YXRpY25vZGVfc2V0Q29uZmlnIGV4dGVybnJlZiBzaGltnQRePHNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpBcnJheVNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplU2VxPjo6ZW5kOjpoMGYyNmNjMDgxZTU3YWJkMp4EYjwmc2VyZGVfd2FzbV9iaW5kZ2VuOjpzZXI6OlNlcmlhbGl6ZXIgYXMgc2VyZGU6OnNlcjo6U2VyaWFsaXplcj46OnNlcmlhbGl6ZV9mNjQ6Omg2NmNlNzcxYTM0Y2U2NDc3nwQjanNfc3lzOjpNYXA6OnNldDo6aDdmZTUzNjBhMWIzMmZjYzCgBDA8JlQgYXMgY29yZTo6Zm10OjpEZWJ1Zz46OmZtdDo6aDJjZWIzNjQ1YmVmNGE2MjKhBEk8Y29yZTo6Zm10OjpGb3JtYXR0ZXIgYXMgY29yZTo6Zm10OjpXcml0ZT46OndyaXRlX2NoYXI6OmhjYWNiZGE2ZTg3NmJmNzYxogQuY29yZTo6c3RyOjpzbGljZV9lcnJvcl9mYWlsOjpoYTc0ZmJlNjM2ZWJmMmFhZKMEKV9fd2JnX25ld183OGZlYjEwOGI2NDcyNzEzIGV4dGVybnJlZiBzaGltpAQpX193YmdfbmV3XzVlMGJlNzM1MjFiYzhjMTcgZXh0ZXJucmVmIHNoaW2lBC5fX3diZ19pdGVyYXRvcl85YTI0Yzg4ZGY4NjBkYzY1IGV4dGVybnJlZiBzaGltpgQpX193YmdfbmV3XzQwNWUyMmYzOTA1NzZjZTIgZXh0ZXJucmVmIHNoaW2nBClfX3diZ19uZXdfOGE2ZjIzOGE2ZWNlODZlYSBleHRlcm5yZWYgc2hpbagEIF9fd2JpbmRnZW5fbWVtb3J5IGV4dGVybnJlZiBzaGltqQQic3RhdGljZ3JhcGhtZXRhX25ldyBleHRlcm5yZWYgc2hpbaoEHXN0YXRpY25vZGVfbmV3IGV4dGVybnJlZiBzaGltqwQnX19ydXN0Y1s2ZGMwMjJjYmQxNGFlNTRkXTo6X19ydXN0X2FsbG9jrAQuX19ydXN0Y1s2ZGMwMjJjYmQxNGFlNTRkXTo6X19ydXN0X2FsbG9jX3plcm9lZK0EMjwmVCBhcyBjb3JlOjpmbXQ6OkRpc3BsYXk+OjpmbXQ6Omg0Njg5ZmQxMjYzNmU3MmJhrgQ7c2VyZGVfd2FzbV9iaW5kZ2VuOjpkZTo6c3RyX2Rlc2VyaWFsaXplcjo6aGFmMTBiZGExODEwMjU4NWGvBENzZXJkZV93YXNtX2JpbmRnZW46OmRlOjpEZXNlcmlhbGl6ZXI6OmlzX251bGxpc2g6Omg2ZDAxYmQwYjY5ZDIyMzM0sARiPHNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpPYmplY3RTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZVN0cnVjdD46OmVuZDo6aDA5NDQwNDUzMTYwNjQzMDexBGU8JnNlcmRlX3dhc21fYmluZGdlbjo6c2VyOjpTZXJpYWxpemVyIGFzIHNlcmRlOjpzZXI6OlNlcmlhbGl6ZXI+OjpzZXJpYWxpemVfc3RydWN0OjpoNjY5YWQyMDQ4YTc4MzY4N7IEQnNlcmRlX3dhc21fYmluZGdlbjo6T2JqZWN0RXh0OjpnZXRfd2l0aF9yZWZfa2V5OjpoMTg1NDIxMmEwMDdlYTYwZLMEQndhc21fYmluZGdlbjo6Y29udmVydDo6dHJhaXRzOjpXYXNtUmV0PFQ+Ojpqb2luOjpoMjExNDIyYWRkMDcxY2M4ObQEiAF3YXNtX2JpbmRnZW46OmNvbnZlcnQ6OnNsaWNlczo6PGltcGwgd2FzbV9iaW5kZ2VuOjpjb252ZXJ0Ojp0cmFpdHM6OkludG9XYXNtQWJpIGZvciBhbGxvYzo6Ym94ZWQ6OkJveDxbVF0+Pjo6aW50b19hYmk6OmhjNTM1NWI2Yjc4NGYxMWFltQRDPGFsbG9jOjpzdHJpbmc6OlN0cmluZyBhcyBjb3JlOjpmbXQ6OkRlYnVnPjo6Zm10OjpoMzIzODU2ZWE2NzA0ZmFkMbYEMjwmVCBhcyBjb3JlOjpmbXQ6OkRpc3BsYXk+OjpmbXQ6OmgzODAzMTRiOTJjNWNlMTAwtwRFPGFsbG9jOjpzdHJpbmc6OlN0cmluZyBhcyBjb3JlOjpmbXQ6OkRpc3BsYXk+OjpmbXQ6OmgzNDgzNGI1ZDQ2MWJjZmIyuASIAXdhc21fYmluZGdlbjo6Y29udmVydDo6c2xpY2VzOjo8aW1wbCB3YXNtX2JpbmRnZW46OmNvbnZlcnQ6OnRyYWl0czo6SW50b1dhc21BYmkgZm9yIGFsbG9jOjpib3hlZDo6Qm94PFtUXT4+OjppbnRvX2FiaTo6aDQ1YjY1Yzk4YjdjNWU4Mji5BDE8VCBhcyBjb3JlOjphbnk6OkFueT46OnR5cGVfaWQ6Omg2YTU2ZTk4NGQzYTM1MzhkugQxPFQgYXMgY29yZTo6YW55OjpBbnk+Ojp0eXBlX2lkOjpoYjEzMzcxYzc0OTczODY1NLsEMjwmVCBhcyBjb3JlOjpmbXQ6OkRpc3BsYXk+OjpmbXQ6OmgxYTVlMjk3NTIwOTJkYmY1vARFPGFsbG9jOjpzdHJpbmc6OlN0cmluZyBhcyBjb3JlOjpmbXQ6OkRpc3BsYXk+OjpmbXQ6OmgzNDgzNGI1ZDQ2MWJjZmIyvQRsPHN0ZDo6cGFuaWNraW5nOjpiZWdpbl9wYW5pY19oYW5kbGVyOjpTdGF0aWNTdHJQYXlsb2FkIGFzIGNvcmU6OnBhbmljOjpQYW5pY1BheWxvYWQ+OjpnZXQ6OmgzZGM5NTkxZjUwMjhlNzc0vgRlPHN0ZDo6cGFuaWNraW5nOjpiZWdpbl9wYW5pY19oYW5kbGVyOjpTdGF0aWNTdHJQYXlsb2FkIGFzIGNvcmU6OmZtdDo6RGlzcGxheT46OmZtdDo6aGJhYjI3MGZjODczZmVkNTi/BDA8JlQgYXMgY29yZTo6Zm10OjpEZWJ1Zz46OmZtdDo6aDQzYjZhZDAxY2M3MTMxYmHABDI8JlQgYXMgY29yZTo6Zm10OjpEaXNwbGF5Pjo6Zm10OjpoZmUwNzU0YjYzMzA5NWM4OMEEMjwmVCBhcyBjb3JlOjpmbXQ6OkRpc3BsYXk+OjpmbXQ6OmhlZTNkYTJmMTE5OTJjOTA1wgREPGNvcmU6OmZtdDo6QXJndW1lbnRzIGFzIGNvcmU6OmZtdDo6RGlzcGxheT46OmZtdDo6aDQ2MWQxZDA1ZjFkMzdjNGLDBBtfX3diaW5kZ2VuX29iamVjdF9jbG9uZV9yZWbEBClfX3diZ19zZXRfNjU1OTViZGQ4NjhiMzAwOSBleHRlcm5yZWYgc2hpbcUEKV9fcnVzdGNbNmRjMDIyY2JkMTRhZTU0ZF06Ol9fcnVzdF9kZWFsbG9jxgQ1c2VyZGVfd2FzbV9iaW5kZ2VuOjpPYmplY3RFeHQ6OnNldDo6aGE1ZTQ3NzRiOTFlMWQzNDfHBCVqc19zeXM6OkFycmF5OjpzZXQ6OmhjOWZiZWE3NjcyYjg2OWVlyAQqanNfc3lzOjpBcnJheTo6aXNfYXJyYXk6OmgxNGVkOWM0NDc3MDI2OGEwyQRjanNfc3lzOjpfOjo8aW1wbCB3YXNtX2JpbmRnZW46OmNhc3Q6OkpzQ2FzdCBmb3IganNfc3lzOjpBcnJheUJ1ZmZlcj46Omluc3RhbmNlb2Y6OmhiZDYxOTcwZDI3MDQ5YzI2ygRbanNfc3lzOjpfOjo8aW1wbCB3YXNtX2JpbmRnZW46OmNhc3Q6OkpzQ2FzdCBmb3IganNfc3lzOjpNYXA+OjppbnN0YW5jZW9mOjpoZDAxNzAzNzQyMzA5MTUxZMsEMmpzX3N5czo6TnVtYmVyOjppc19zYWZlX2ludGVnZXI6OmhkMjdhOWNjY2UzODJiMWJkzARianNfc3lzOjpfOjo8aW1wbCB3YXNtX2JpbmRnZW46OmNhc3Q6OkpzQ2FzdCBmb3IganNfc3lzOjpVaW50OEFycmF5Pjo6aW5zdGFuY2VvZjo6aGIxNGFhZTkwZGMyNDllNDjNBDd3YXNtX2JpbmRnZW46OmNhc3Q6OkpzQ2FzdDo6aGFzX3R5cGU6OmhiOWJlMDg2NjY0NzAwZWNlzgRHPGR5biBzZXJkZTo6ZGU6OkV4cGVjdGVkIGFzIGNvcmU6OmZtdDo6RGlzcGxheT46OmZtdDo6aDgzN2RlMzE3NzBkYzg1Y2PPBD9jb3JlOjpzbGljZTo6aW5kZXg6OnNsaWNlX2VuZF9pbmRleF9sZW5fZmFpbDo6aDFmYjVmMGZjYzI1Yjc5YTfQBD1jb3JlOjpzbGljZTo6aW5kZXg6OnNsaWNlX2luZGV4X29yZGVyX2ZhaWw6OmhjOWI3MTljM2UxMzNjNjA50QROY29yZTo6Zm10OjpudW06OmltcDo6PGltcGwgY29yZTo6Zm10OjpEaXNwbGF5IGZvciB1MzI+OjpmbXQ6Omg4OGQ1N2MzOTg3MTU2YmZk0gQuY29yZTo6b3B0aW9uOjp1bndyYXBfZmFpbGVkOjpoMjgxOWM0ZGEzZDMxNTg0NNMETmNvcmU6OmZtdDo6bnVtOjppbXA6OjxpbXBsIGNvcmU6OmZtdDo6RGlzcGxheSBmb3IgdTY0Pjo6Zm10OjpoNDM2ODU5NGY0OGU0MTFhNdQEHF9fd2JpbmRnZW5faW4gZXh0ZXJucmVmIHNoaW3VBCJfX3diaW5kZ2VuX2pzdmFsX2VxIGV4dGVybnJlZiBzaGlt1gQoX193YmluZGdlbl9qc3ZhbF9sb29zZV9lcSBleHRlcm5yZWYgc2hpbdcEnAE8d2FzbTo6Z3JhcGg6Ol86OjxpbXBsIHNlcmRlOjpkZTo6RGVzZXJpYWxpemUgZm9yIHdhc206OmdyYXBoOjpTdGF0aWNHcmFwaE1ldGE+OjpkZXNlcmlhbGl6ZTo6X19WaXNpdG9yIGFzIHNlcmRlOjpkZTo6VmlzaXRvcj46OmV4cGVjdGluZzo6aGE0ZTFkNTYyZjE3MDc5Y2HYBJcBPHdhc206OmdyYXBoOjpfOjo8aW1wbCBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplIGZvciB3YXNtOjpncmFwaDo6U3RhdGljTm9kZT46OmRlc2VyaWFsaXplOjpfX1Zpc2l0b3IgYXMgc2VyZGU6OmRlOjpWaXNpdG9yPjo6ZXhwZWN0aW5nOjpoNzQyMTgyZGIzMjZjNzY2MtkEtQE8c2VyZGU6OmRlOjppbXBsczo6PGltcGwgc2VyZGU6OmRlOjpEZXNlcmlhbGl6ZSBmb3Igc3RkOjpjb2xsZWN0aW9uczo6aGFzaDo6bWFwOjpIYXNoTWFwPEssVixTPj46OmRlc2VyaWFsaXplOjpNYXBWaXNpdG9yPEssVixTPiBhcyBzZXJkZTo6ZGU6OlZpc2l0b3I+OjpleHBlY3Rpbmc6OmgyYWJkMDA1N2FmMjJlOTRl2gQyPFQgYXMgc2VyZGU6OmRlOjpFeHBlY3RlZD46OmZtdDo6aDM3ZDdmMTA0ZmYyMjdhMGPbBDVfX3J1c3RjWzZkYzAyMmNiZDE0YWU1NGRdOjpfX3J1c3RfYWxsb2NfZXJyb3JfaGFuZGxlctwEJWpzX3N5czo6QXJyYXk6OmdldDo6aDllYjdjNjIxMTRhNmI3OTndBC5jb3JlOjpmbXQ6OldyaXRlOjp3cml0ZV9mbXQ6OmgyYzc0MmM0NDdhOGU3ZWQw3gRCY29yZTo6cHRyOjpkcm9wX2luX3BsYWNlPGFsbG9jOjpzdHJpbmc6OlN0cmluZz46Omg0ODVmMjFkZGRlYmMzMDU03wQ+PGNvcmU6OmZtdDo6RXJyb3IgYXMgY29yZTo6Zm10OjpEZWJ1Zz46OmZtdDo6aDQ5YTJkOWYxZmNhNjYwYTngBEJjb3JlOjpwdHI6OmRyb3BfaW5fcGxhY2U8YWxsb2M6OnN0cmluZzo6U3RyaW5nPjo6aDk4ZGUyZDdmNWQ4ZDAzYzjhBD48Y29yZTo6Zm10OjpFcnJvciBhcyBjb3JlOjpmbXQ6OkRlYnVnPjo6Zm10OjpoNDlhMmQ5ZjFmY2E2NjBhOeIELmNvcmU6OmZtdDo6V3JpdGU6OndyaXRlX2ZtdDo6aDBiYjlhYjY2ZjRhYmMzOTPjBNwBY29yZTo6cHRyOjpkcm9wX2luX3BsYWNlPGFsbG9jOjpib3hlZDo6Y29udmVydDo6PGltcGwgY29yZTo6Y29udmVydDo6RnJvbTxhbGxvYzo6c3RyaW5nOjpTdHJpbmc+IGZvciBhbGxvYzo6Ym94ZWQ6OkJveDxkeW4gY29yZTo6ZXJyb3I6OkVycm9yK2NvcmU6Om1hcmtlcjo6U2VuZCtjb3JlOjptYXJrZXI6OlN5bmM+Pjo6ZnJvbTo6U3RyaW5nRXJyb3I+OjpoYjRhNmNkY2U5MTI2ZmFlM+QEogE8c2VyZGVfanNvbjo6dmFsdWU6OmRlOjo8aW1wbCBzZXJkZTo6ZGU6OkRlc2VyaWFsaXplIGZvciBzZXJkZV9qc29uOjp2YWx1ZTo6VmFsdWU+OjpkZXNlcmlhbGl6ZTo6VmFsdWVWaXNpdG9yIGFzIHNlcmRlOjpkZTo6VmlzaXRvcj46OmV4cGVjdGluZzo6aDlhZmVkZDk0ZmViMzU4MWLlBFo8c2VyZGVfanNvbjo6dmFsdWU6OmRlOjpLZXlDbGFzc2lmaWVyIGFzIHNlcmRlOjpkZTo6VmlzaXRvcj46OmV4cGVjdGluZzo6aGU4MzYxOWUyZmUzMTIyOTLmBE88YWxsb2M6OnJhd192ZWM6OlJhd1ZlYzxULEE+IGFzIGNvcmU6Om9wczo6ZHJvcDo6RHJvcD46OmRyb3A6OmgyOTQ2YmVkZGUxOTU2MTdh5wRPPGFsbG9jOjpyYXdfdmVjOjpSYXdWZWM8VCxBPiBhcyBjb3JlOjpvcHM6OmRyb3A6OkRyb3A+Ojpkcm9wOjpoYjY2ODk2M2E4MWQ0ZGM5YegELmNvcmU6OmZtdDo6V3JpdGU6OndyaXRlX2ZtdDo6aDVmMDc1OTBhYjEzMDI4ZGPpBFM8c2VyZGU6OmRlOjppbXBsczo6Qm9vbFZpc2l0b3IgYXMgc2VyZGU6OmRlOjpWaXNpdG9yPjo6ZXhwZWN0aW5nOjpoYWJkNWNlY2QyMGQ2NGIwZuoEVTxzZXJkZTo6ZGU6OmltcGxzOjpTdHJpbmdWaXNpdG9yIGFzIHNlcmRlOjpkZTo6VmlzaXRvcj46OmV4cGVjdGluZzo6aGRkZjhkMmM5NmYyYmY4MDjrBIwBPHNlcmRlOjpkZTo6aW1wbHM6OjxpbXBsIHNlcmRlOjpkZTo6RGVzZXJpYWxpemUgZm9yIGkzMj46OmRlc2VyaWFsaXplOjpQcmltaXRpdmVWaXNpdG9yIGFzIHNlcmRlOjpkZTo6VmlzaXRvcj46OmV4cGVjdGluZzo6aDk0Njg3ZDcxYTM3YzlkMmTsBIwBPHNlcmRlOjpkZTo6aW1wbHM6OjxpbXBsIHNlcmRlOjpkZTo6RGVzZXJpYWxpemUgZm9yIHUzMj46OmRlc2VyaWFsaXplOjpQcmltaXRpdmVWaXNpdG9yIGFzIHNlcmRlOjpkZTo6VmlzaXRvcj46OmV4cGVjdGluZzo6aDY3NTQyY2Q5YjZhY2VmNGPtBCp3YXNtX2JpbmRnZW46OnRocm93X3N0cjo6aDc3NGVhNzZkZDIyODdkOGbuBE88YWxsb2M6OnJhd192ZWM6OlJhd1ZlYzxULEE+IGFzIGNvcmU6Om9wczo6ZHJvcDo6RHJvcD46OmRyb3A6OmgxZDQ4MmVmOTM2ZDE5OTIz7wRPPGFsbG9jOjpyYXdfdmVjOjpSYXdWZWM8VCxBPiBhcyBjb3JlOjpvcHM6OmRyb3A6OkRyb3A+Ojpkcm9wOjpoOTgwMThlN2ViMjg3OWQ4ZvAEMXdhc21fYmluZGdlbjo6X19ydDo6dGhyb3dfbnVsbDo6aDVmY2U3ZWI2MjkwN2UyOWPxBDJ3YXNtX2JpbmRnZW46Ol9fcnQ6OmJvcnJvd19mYWlsOjpoYjVmZjA4OTAyM2Y4NTNmMvIELmNvcmU6OmZtdDo6V3JpdGU6OndyaXRlX2ZtdDo6aGU2YTJhOWY5OWM2NWQzZTnzBG88c3RkOjpwYW5pY2tpbmc6OmJlZ2luX3BhbmljX2hhbmRsZXI6OlN0YXRpY1N0clBheWxvYWQgYXMgY29yZTo6cGFuaWM6OlBhbmljUGF5bG9hZD46OmFzX3N0cjo6aDc2YWEwMTk5M2RhMzE4Yzf0BC5jb3JlOjpmbXQ6OldyaXRlOjp3cml0ZV9mbXQ6Omg5MjM4YTFiMjcyZTg4MTc29QQ+PGNvcmU6OmZtdDo6RXJyb3IgYXMgY29yZTo6Zm10OjpEZWJ1Zz46OmZtdDo6aDQ5YTJkOWYxZmNhNjYwYTn2BDNhbGxvYzo6YWxsb2M6OmhhbmRsZV9hbGxvY19lcnJvcjo6aDFlZWQ1ZWFhNTM0ZDcyOTL3BDM8c3RyIGFzIGNvcmU6OmZtdDo6RGlzcGxheT46OmZtdDo6aDJkYjFmMDFkOGIwNjlkN2L4BCRfX3diaW5kZ2VuX251bWJlcl9nZXQgZXh0ZXJucmVmIHNoaW35BCRfX3diaW5kZ2VuX3N0cmluZ19nZXQgZXh0ZXJucmVmIHNoaW36BCtfX3diZ19zdGFja18wZWQ3NWQ2ODU3NWIwZjNjIGV4dGVybnJlZiBzaGlt+wQrX193YmluZGdlbl9iaWdpbnRfZ2V0X2FzX2k2NCBleHRlcm5yZWYgc2hpbfwEJl9fd2JpbmRnZW5fZGVidWdfc3RyaW5nIGV4dGVybnJlZiBzaGlt/QQyPFQgYXMgc2VyZGU6OmRlOjpFeHBlY3RlZD46OmZtdDo6aDgyZDg2ZjRlYjQ2MDM0NjX+BDI8VCBhcyBzZXJkZTo6ZGU6OkV4cGVjdGVkPjo6Zm10OjpoYzBhOWI5MmIwNTYyMWExZP8EMjxUIGFzIHNlcmRlOjpkZTo6RXhwZWN0ZWQ+OjpmbXQ6OmgyM2NlODIwMmNjOGU0YWE2gAUyPFQgYXMgc2VyZGU6OmRlOjpFeHBlY3RlZD46OmZtdDo6aDQyODYxZTkyNDE2NWIzMTWBBTI8VCBhcyBzZXJkZTo6ZGU6OkV4cGVjdGVkPjo6Zm10OjpoNzg0MjRiNDJhMjJkMWRjYoIFMjxUIGFzIHNlcmRlOjpkZTo6RXhwZWN0ZWQ+OjpmbXQ6OmhkYmE2NDZjNDUzNzU4ZjQxgwVNPGpzX3N5czo6SnNTdHJpbmcgYXMgY29yZTo6c3RyOjp0cmFpdHM6OkZyb21TdHI+Ojpmcm9tX3N0cjo6aGQ4NWRhODdkOThiYmQzMzGEBSpqc19zeXM6Ok9iamVjdDo6ZW50cmllczo6aDcxODI3ZTIwNDI1MjAzNWaFBSpqc19zeXM6OlVpbnQ4QXJyYXk6Om5ldzo6aDdlNzUxZGIzMjRkNzIzYTeGBTRjb3JlOjpwYW5pYzo6UGFuaWNQYXlsb2FkOjphc19zdHI6OmhhN2E5ZTY4MzhiNTk1ZGY3hwVCc3RkOjpzeXM6OmJhY2t0cmFjZTo6X19ydXN0X2VuZF9zaG9ydF9iYWNrdHJhY2U6Omg5ZmEyNzZkNzE0MDIwNGNhiAVBc3RkOjpwYW5pY2tpbmc6OnBhbmljX2NvdW50Ojppc196ZXJvX3Nsb3dfcGF0aDo6aDg3NDdiZmZlMjc1Y2Y5OWaJBSZfX3diaW5kZ2VuX2lzX3VuZGVmaW5lZCBleHRlcm5yZWYgc2hpbYoFJV9fd2JpbmRnZW5fYm9vbGVhbl9nZXQgZXh0ZXJucmVmIHNoaW2LBSNfX3diaW5kZ2VuX2lzX2JpZ2ludCBleHRlcm5yZWYgc2hpbYwFI19fd2JpbmRnZW5faXNfb2JqZWN0IGV4dGVybnJlZiBzaGltjQUjX193YmluZGdlbl9hc19udW1iZXIgZXh0ZXJucmVmIHNoaW2OBSxfX3diZ19sZW5ndGhfZTJkMmE0OTEzMmMxYjI1NiBleHRlcm5yZWYgc2hpbY8FJV9fd2JpbmRnZW5faXNfZnVuY3Rpb24gZXh0ZXJucmVmIHNoaW2QBSpfX3diZ19kb25lXzc2OWU1ZWRlNGIzMWM2N2IgZXh0ZXJucmVmIHNoaW2RBSNfX3diaW5kZ2VuX2lzX3N0cmluZyBleHRlcm5yZWYgc2hpbZIFLV9fd2JnX2lzQXJyYXlfYTFlYWI3ZTBkMDY3MzkxYiBleHRlcm5yZWYgc2hpbZMFPF9fd2JnX2luc3RhbmNlb2ZfQXJyYXlCdWZmZXJfZTE0NTg1NDMyZTM3MzdmYyBleHRlcm5yZWYgc2hpbZQFNF9fd2JnX2luc3RhbmNlb2ZfTWFwX2YzNDY5Y2UyMjQ0ZDI0MzAgZXh0ZXJucmVmIHNoaW2VBTNfX3diZ19pc1NhZmVJbnRlZ2VyXzM0M2UyYmVlZWVjZTFiYjAgZXh0ZXJucmVmIHNoaW2WBSxfX3diZ19sZW5ndGhfYTQ0NjE5M2RjMjJjMTJmOCBleHRlcm5yZWYgc2hpbZcFO19fd2JnX2luc3RhbmNlb2ZfVWludDhBcnJheV8xNzE1NmJjZjExODA4NmE5IGV4dGVybnJlZiBzaGltmAUwY29yZTo6b3BzOjpmdW5jdGlvbjo6Rm46OmNhbGw6OmhkMTYzMWQwMzVkMTZmY2NkmQU3Y29yZTo6b3BzOjpmdW5jdGlvbjo6Rm5NdXQ6OmNhbGxfbXV0OjpoZjI2ZWM0NjVjY2E3MGJjOZoFSGNvcmU6Om9wczo6ZnVuY3Rpb246OkZuT25jZTo6Y2FsbF9vbmNle3t2dGFibGUuc2hpbX19OjpoMzQ2ZDNiZWRkYjYzYTI3MZsFUTxzZXJkZV93YXNtX2JpbmRnZW46OmVycm9yOjpFcnJvciBhcyBzZXJkZTo6ZGU6OkVycm9yPjo6Y3VzdG9tOjpoZWU0YmU1NzU0NjBlNjFlNZwFF19fZXh0ZXJucmVmX3RhYmxlX2FsbG9jnQU1d2FzbV9iaW5kZ2VuOjpfX3J0OjptYWxsb2NfZmFpbHVyZTo6aGY2MmRmOGI2MzMwYWQxOGSfBQVncmVldKAFRTxqc19zeXM6OkFycmF5IGFzIGNvcmU6OmRlZmF1bHQ6OkRlZmF1bHQ+OjpkZWZhdWx0OjpoZDQ5MmM4MTA2YzQxN2YxYqEFQzxqc19zeXM6Ok1hcCBhcyBjb3JlOjpkZWZhdWx0OjpEZWZhdWx0Pjo6ZGVmYXVsdDo6aGYyMjcyYTg4MGQ5YTg2MzmiBUY8anNfc3lzOjpPYmplY3QgYXMgY29yZTo6ZGVmYXVsdDo6RGVmYXVsdD46OmRlZmF1bHQ6Omg1OTI4NzQ4Yzk2ODcyMmMwowUranNfc3lzOjpTeW1ib2w6Oml0ZXJhdG9yOjpoNDY1OGI4ZjEzYTYzMTQyMqQFQndhc21fYmluZGdlbjo6Y29udmVydDo6dHJhaXRzOjpXYXNtUmV0PFQ+Ojpqb2luOjpoOWMyMTdiOWQ3YjYxYjdlZKUFXjx3YXNtX2JpbmRnZW46OmNvbnZlcnQ6OnRyYWl0czo6V2FzbVJldDxUPiBhcyBjb3JlOjpjb252ZXJ0OjpGcm9tPFQ+Pjo6ZnJvbTo6aGRjYTkxZmVkZjA0ZmM5ZDmmBTx3YXNtX2JpbmRnZW46OmNhc3Q6OkpzQ2FzdDo6dW5jaGVja2VkX3JlZjo6aDMzZGVmMzhlMjUzYzE4MjOnBT13YXNtX2JpbmRnZW46OmNhc3Q6OkpzQ2FzdDo6dW5jaGVja2VkX2ludG86OmgyNGU1ZjgzNTJjOTEzOTNjqAUnd2FzbV9iaW5kZ2VuOjptZW1vcnk6Omg0NTE5MTNlYjBlNDM5ZTM5qQU+c3RkOjpzeXM6OnBhbDo6d2FzbTo6Y29tbW9uOjphYm9ydF9pbnRlcm5hbDo6aDYxODhkNmEwM2JlYzVkYTKqBS1fX3J1c3RjWzZkYzAyMmNiZDE0YWU1NGRdOjpfX3J1c3Rfc3RhcnRfcGFuaWOrBV48d2FzbV9iaW5kZ2VuOjpjb252ZXJ0Ojp0cmFpdHM6Oldhc21SZXQ8VD4gYXMgY29yZTo6Y29udmVydDo6RnJvbTxUPj46OmZyb206Omg0NTAyNWQ2NjhkZmE3NzM5rAVIPGFsbG9jOjp2ZWM6OlZlYzxULEE+IGFzIGNvcmU6Om9wczo6ZHJvcDo6RHJvcD46OmRyb3A6OmhlMjkyYzkzNTM4YmUwZDI3rQVIPGFsbG9jOjp2ZWM6OlZlYzxULEE+IGFzIGNvcmU6Om9wczo6ZHJvcDo6RHJvcD46OmRyb3A6OmgxNGZkNzE2NDNhY2FjYjEwBxIBAA9fX3N0YWNrX3BvaW50ZXIJEQIABy5yb2RhdGEBBS5kYXRhAHgJcHJvZHVjZXJzAghsYW5ndWFnZQEEUnVzdAAMcHJvY2Vzc2VkLWJ5AwVydXN0YyUxLjg4LjAtbmlnaHRseSAoMDc3Y2VkYzJhIDIwMjUtMDQtMTkpBndhbHJ1cwYwLjIzLjMMd2FzbS1iaW5kZ2VuBzAuMi4xMDAAlAEPdGFyZ2V0X2ZlYXR1cmVzCCsLYnVsay1tZW1vcnkrD2J1bGstbWVtb3J5LW9wdCsWY2FsbC1pbmRpcmVjdC1vdmVybG9uZysKbXVsdGl2YWx1ZSsPbXV0YWJsZS1nbG9iYWxzKxNub250cmFwcGluZy1mcHRvaW50Kw9yZWZlcmVuY2UtdHlwZXMrCHNpZ24tZXh0", import.meta.url);
    }
    const imports = __wbg_get_imports();
    if (typeof module_or_path === "string" || typeof Request === "function" && module_or_path instanceof Request || typeof URL === "function" && module_or_path instanceof URL) {
      module_or_path = fetch(module_or_path);
    }
    const { instance, module } = await __wbg_load(await module_or_path, imports);
    return __wbg_finalize_init(instance, module);
  }
  const wasmURL = "data:application/wasm;base64,AGFzbQEAAAABygIzYAJ/fwF/YAJ/fwBgA39/fwBgA39/fwF/YAF/AGAAAn9/YAF/AX9gBH9/f38AYAFvAX9gBX9/f39/AGABfwJ/f2AAA39/f2AAAGAAAX9gAAFvYAFvAW9gAn9vAGAEf39/fwF/YAJvbwF/YAJvbwFvYAN/f38BfmAGf39/f39/AGAFf39/f38Bf2ABfwN/f39gAX4Bb2ACf38Bb2AGf39/f39/AX9gAW8Df39/YAF+AX9gAW8BfGABfAFvYANvb28AYAJvfwFvYANvf28AYANvb28Bb2ADb29/AGACf38BfmAJf39/f39/fn5+AGADfn9/AX9gB39/f39/f38Bf2ACb28Df39/YAJ/bwJ/f2ADf398AGAFf399f38AYAR/fX9/AGAFf39+f38AYAR/fn9/AGAFf398f38AYAR/fH9/AGACf34AYAF8AX8Clw0zA3diZxdfX3diaW5kZ2VuX2lzX3VuZGVmaW5lZAAIA3diZw1fX3diaW5kZ2VuX2luABIDd2JnFl9fd2JpbmRnZW5fYm9vbGVhbl9nZXQACAN3YmcUX193YmluZGdlbl9pc19iaWdpbnQACAN3YmcVX193YmluZGdlbl9udW1iZXJfZ2V0ABADd2JnGl9fd2JpbmRnZW5fYmlnaW50X2Zyb21faTY0ABgDd2JnE19fd2JpbmRnZW5fanN2YWxfZXEAEgN3YmcVX193YmluZGdlbl9zdHJpbmdfZ2V0ABADd2JnFF9fd2JpbmRnZW5faXNfb2JqZWN0AAgDd2JnGl9fd2JpbmRnZW5fYmlnaW50X2Zyb21fdTY0ABgDd2JnFF9fd2JpbmRnZW5fZXJyb3JfbmV3ABkDd2JnGV9fd2JpbmRnZW5fanN2YWxfbG9vc2VfZXEAEgN3YmcUX193YmluZGdlbl9hc19udW1iZXIAHQN3YmcVX193YmluZGdlbl9udW1iZXJfbmV3AB4Dd2JnFV9fd2JpbmRnZW5fc3RyaW5nX25ldwAZA3diZyRfX3diZ19nZXR3aXRocmVma2V5XzFkYzM2MWJkMTAwNTNiZmUAEwN3YmcaX193Ymdfc2V0XzNmMWQwYjk4NGVkMjcyZWQAHwN3YmcaX193YmdfZ2V0X2I5YjkzMDQ3ZmUzY2Y0NWIAIAN3YmcdX193YmdfbGVuZ3RoX2UyZDJhNDkxMzJjMWIyNTYACAN3YmcaX193YmdfbmV3Xzc4ZmViMTA4YjY0NzI3MTMADgN3YmcWX193YmluZGdlbl9pc19mdW5jdGlvbgAIA3diZxpfX3diZ19uZXdfNWUwYmU3MzUyMWJjOGMxNwAOA3diZxtfX3diZ19uZXh0XzI1ZmVhZGZjMDkxM2ZlYTkADwN3YmcbX193YmdfbmV4dF82NTc0ZTFhOGE2MmQxMDU1AA8Dd2JnG19fd2JnX2RvbmVfNzY5ZTVlZGU0YjMxYzY3YgAIA3diZxxfX3diZ192YWx1ZV9jZDFmZmE3YjFhYjc5NGYxAA8Dd2JnH19fd2JnX2l0ZXJhdG9yXzlhMjRjODhkZjg2MGRjNjUADgN3YmcaX193YmdfZ2V0XzY3YjJiYTYyZmMzMGRlMTIAEwN3YmcbX193YmdfY2FsbF82NzJhNGQyMTYzNGQ0YTI0ABMDd2JnGl9fd2JnX25ld180MDVlMjJmMzkwNTc2Y2UyAA4Dd2JnFF9fd2JpbmRnZW5faXNfc3RyaW5nAAgDd2JnGl9fd2JnX3NldF8zNzgzNzAyM2YzZDc0MGU4ACEDd2JnHl9fd2JnX2lzQXJyYXlfYTFlYWI3ZTBkMDY3MzkxYgAIA3diZy1fX3diZ19pbnN0YW5jZW9mX0FycmF5QnVmZmVyX2UxNDU4NTQzMmUzNzM3ZmMACAN3YmclX193YmdfaW5zdGFuY2VvZl9NYXBfZjM0NjljZTIyNDRkMjQzMAAIA3diZxpfX3diZ19zZXRfOGZjNmJmOGE1YjEwNzFkMQAiA3diZyRfX3diZ19pc1NhZmVJbnRlZ2VyXzM0M2UyYmVlZWVjZTFiYjAACAN3YmceX193YmdfZW50cmllc18zMjY1ZDQxNThiMzNlNWRjAA8Dd2JnHV9fd2JnX2J1ZmZlcl82MDljYzNlZWU1MWVkMTU4AA8Dd2JnGl9fd2JnX25ld19hMTIwMDJhN2Y5MWM3NWJlAA8Dd2JnGl9fd2JnX3NldF82NTU5NWJkZDg2OGIzMDA5ACMDd2JnHV9fd2JnX2xlbmd0aF9hNDQ2MTkzZGMyMmMxMmY4AAgDd2JnLF9fd2JnX2luc3RhbmNlb2ZfVWludDhBcnJheV8xNzE1NmJjZjExODA4NmE5AAgDd2JnGl9fd2JnX25ld184YTZmMjM4YTZlY2U4NmVhAA4Dd2JnHF9fd2JnX3N0YWNrXzBlZDc1ZDY4NTc1YjBmM2MAEAN3YmccX193YmdfZXJyb3JfNzUzNGI4ZTlhMzZmMWFiNAABA3diZxxfX3diaW5kZ2VuX2JpZ2ludF9nZXRfYXNfaTY0ABADd2JnF19fd2JpbmRnZW5fZGVidWdfc3RyaW5nABADd2JnEF9fd2JpbmRnZW5fdGhyb3cAAQN3YmcRX193YmluZGdlbl9tZW1vcnkADgN3YmcfX193YmluZGdlbl9pbml0X2V4dGVybnJlZl90YWJsZQAMA+8C7QIBBgEBAAIDAAIBAgIaBAcCAAcHBwMAAAMDAwIkAQABAiUBBwAmAwEVJwIBAQIBAQAAFQAABAEBAQABAA0BAQIBBwcRAAkGAQkCAQACAQEJDQcBAQEBAgICAgICAgICBwECAgICAgABAwQBAAkJAAkJAgQAAQQCCQQCAQEBARUCAgEEAQEDAwEBCQYHCQEJAAABAQEBAQEBAQECBAYCBAQEBAIDAwAMAwEEBgwBAgIBAQIABgIABAEEAQYGBgYGBgECBBYEBAQCBgQBEQQEBAQBAAABBAEoARsXGxcXBAEEACkHAQoKCgoKCgoKCgoKCgoqGgAEBBQUCSsWLS8NDQQRBwQAAwAGBwExBAIEAQAAAgQCAAAGEQcDAwIEAAMcHDIBAwAJDQ0AAgYCAQAAAAEBAAEAAAAAAgICBgYGAgIABAAAAAYAAAAAAAAEAAQAAAQAAQQMDAABAAABAwEBAAAAAAAAAQYGBgYGAQ0MDAQJAnABY2NvAIABBQMBABEGCQF/AUGAgMAACwegDT8GbWVtb3J5AgAaX193Ymdfc3RhdGljZ3JhcGhtZXRhX2ZyZWUAXRNzdGF0aWNncmFwaG1ldGFfbmV3AIoCG3N0YXRpY2dyYXBobWV0YV9nZXRfZ3JhcGhpZACWAhtzdGF0aWNncmFwaG1ldGFfc2V0X2dyYXBoaWQAkwEWc3RhdGljZ3JhcGhtZXRhX3RvSlNPTgCLAhlzdGF0aWNncmFwaG1ldGFfZ2V0QXV0aG9yAJcCGXN0YXRpY2dyYXBobWV0YV9zZXRBdXRob3IAiAEdc3RhdGljZ3JhcGhtZXRhX2dldElzUmVzb3VyY2UA3AEdc3RhdGljZ3JhcGhtZXRhX3NldElzUmVzb3VyY2UAugEVX193Ymdfc3RhdGljbm9kZV9mcmVlAL8BDnN0YXRpY25vZGVfbmV3AIwCD3N0YXRpY25vZGVfY29weQDOARFzdGF0aWNub2RlX3RvSlNPTgCNAhRzdGF0aWNub2RlX2dldF9hbGlhcwCYAhRzdGF0aWNub2RlX3NldF9hbGlhcwCJARdzdGF0aWNub2RlX2dldF9kYXRhdHlwZQCZAhdzdGF0aWNub2RlX3NldF9kYXRhdHlwZQCUARpzdGF0aWNub2RlX2dldF9kZXNjcmlwdGlvbgCaAhpzdGF0aWNub2RlX3NldF9kZXNjcmlwdGlvbgCKARlzdGF0aWNub2RlX2dldF9leHBvcnRhYmxlAOwBGXN0YXRpY25vZGVfc2V0X2V4cG9ydGFibGUAxAEYc3RhdGljbm9kZV9nZXRfZmllbGRuYW1lAJsCGHN0YXRpY25vZGVfc2V0X2ZpZWxkbmFtZQCLARdzdGF0aWNub2RlX2dldF9ncmFwaF9pZACcAhdzdGF0aWNub2RlX3NldF9ncmFwaF9pZACVAR1zdGF0aWNub2RlX2dldF9oYXNjdXN0b21hbGlhcwDtAR1zdGF0aWNub2RlX3NldF9oYXNjdXN0b21hbGlhcwDFARtzdGF0aWNub2RlX2dldF9pc19jb2xsZWN0b3IA7gEbc3RhdGljbm9kZV9zZXRfaXNfY29sbGVjdG9yAMYBGXN0YXRpY25vZGVfZ2V0X2lzcmVxdWlyZWQA7wEZc3RhdGljbm9kZV9zZXRfaXNyZXF1aXJlZADHARtzdGF0aWNub2RlX2dldF9pc3NlYXJjaGFibGUA8AEbc3RhdGljbm9kZV9zZXRfaXNzZWFyY2hhYmxlAMgBGHN0YXRpY25vZGVfZ2V0X2lzdG9wbm9kZQDxARhzdGF0aWNub2RlX3NldF9pc3RvcG5vZGUAyQETc3RhdGljbm9kZV9nZXRfbmFtZQCdAhNzdGF0aWNub2RlX3NldF9uYW1lAJYBG3N0YXRpY25vZGVfZ2V0X25vZGVncm91cF9pZACeAhtzdGF0aWNub2RlX3NldF9ub2RlZ3JvdXBfaWQAjAEVc3RhdGljbm9kZV9nZXRfbm9kZWlkAJ8CFXN0YXRpY25vZGVfc2V0X25vZGVpZACXARxzdGF0aWNub2RlX2dldF9vbnRvbG9neWNsYXNzAKACHHN0YXRpY25vZGVfc2V0X29udG9sb2d5Y2xhc3MAjQEdc3RhdGljbm9kZV9nZXRfcGFyZW50cHJvcGVydHkAoQIdc3RhdGljbm9kZV9zZXRfcGFyZW50cHJvcGVydHkAjgEYc3RhdGljbm9kZV9nZXRfc29ydG9yZGVyAPoBGHN0YXRpY25vZGVfc2V0X3NvcnRvcmRlcgDLASlzdGF0aWNub2RlX2dldF9zb3VyY2VicmFuY2hwdWJsaWNhdGlvbl9pZACiAilzdGF0aWNub2RlX3NldF9zb3VyY2VicmFuY2hwdWJsaWNhdGlvbl9pZACPARRzdGF0aWNub2RlX2dldENvbmZpZwCOAhRzdGF0aWNub2RlX3NldENvbmZpZwCTAhJzdGF0aWNub2RlX2NvbXBhcmUAiAIEbWFpbgDdAQVncmVldACfAxRfX3diaW5kZ2VuX2V4bl9zdG9yZQDNAhdfX2V4dGVybnJlZl90YWJsZV9hbGxvYwCdAxNfX3diaW5kZ2VuX2V4cG9ydF8yAQEPX193YmluZGdlbl9mcmVlAMwCEV9fd2JpbmRnZW5fbWFsbG9jAOcBEl9fd2JpbmRnZW5fcmVhbGxvYwD9ARlfX2V4dGVybnJlZl90YWJsZV9kZWFsbG9jAPQBEF9fd2JpbmRnZW5fc3RhcnQAngMJvgEBAEEBC2KTA5ID+gL5ApED9wKQA5QDlQP4AqcBdqgCqAKpAqkCqQKpApwDnAOcA9UCQ+MC1QKdAaUC8wK4ArYCswKqAqoCrAKtAqsCqgKuArABrAKkAv0CygJi/AL+AvEC3wL/AssCY4EDgAP/AssCY7UCfWvmAuQBmgG3AoMD4ALDAoQCkgLgAuMCtQG8AtUBZYgD4gLhAuUC4QHkAokDsQKlAWyAAZYDvALZAWaKA4sDzgLVAucC6AJImAFtDAEXCrPLB+0CsTICE38BfiMAQZADayICJAAgAiABNgIUAkACQCABEJkDQQFHBEAgAkEUaiACQeABakGogsAAEEohBCAAQYCAgIB4NgKcASAAIAQ2AgAgAUGEAUkNASABEPQBDAELIAJBGGoiAyABQbCOwABBExCUAiACQYGAgIB4NgIsIAJBADYCOCACQYCAgIB4NgJYIAJBgYCAgHg2AmQgAkGBgICAeDYCcCACQYCAgIB4NgJ8IAJBgICAgHg2AogBIAJBgYCAgHg2ApQBIAJBgICAgHg2AqABIAJBgYCAgHg2AqwBIAJBgYCAgHg2ArgBIAJBgYCAgHg2AsQBIAIoAjwhBCACQeABaiADEGECQAJ/AkACQCACLQDgAQRAQQAhAQwBCyACQUBrIQggAkHoAWohCUECIQpBAiELQQIhDEECIQ1BAiEOQQIhD0EAIQEDQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQCACLQDhAUEBaw4UAgMEBQYHCAkKCwwNDg8QERITABQBCyACQQhqIAJBGGoQ6QEMJwsgAigCLEGBgICAeEYNJSACIAQ2AjwgAiABNgI4QZCFwABBBRDCASEBIABBgICAgHg2ApwBIAAgATYCAAwqCyABRQ0jIAIgBDYCPCACIAE2AjhBlYXAAEEGEMIBIQEgAEGAgICAeDYCnAEgACABNgIADCkLIAIoAlhBgICAgHhGDSEgAiAENgI8IAIgATYCOEGbhcAAQQgQwgEhASAAQYCAgIB4NgKcASAAIAE2AgAMKAsgAigCZEGBgICAeEYNHyACIAQ2AjwgAiABNgI4QceDwABBCxDCASEBIABBgICAgHg2ApwBIAAgATYCAAwnCyAKQQJGDR0gAiAENgI8IAIgATYCOEGjhcAAQQoQwgEhASAAQYCAgIB4NgKcASAAIAE2AgAMJgsgAigCcEGBgICAeEYNGyACIAQ2AjwgAiABNgI4Qa2FwABBCRDCASEBIABBgICAgHg2ApwBIAAgATYCAAwlCyACKAJ8QYCAgIB4Rg0ZIAIgBDYCPCACIAE2AjhBtoXAAEEIEMIBIQEgAEGAgICAeDYCnAEgACABNgIADCQLIAtBAkYNFyACIAQ2AjwgAiABNgI4Qb6FwABBDhDCASEBIABBgICAgHg2ApwBIAAgATYCAAwjCyAMQQJGDRUgAiAENgI8IAIgATYCOEHMhcAAQQwQwgEhASAAQYCAgIB4NgKcASAAIAE2AgAMIgsgDUECRg0TIAIgBDYCPCACIAE2AjhB2IXAAEEKEMIBIQEgAEGAgICAeDYCnAEgACABNgIADCELIA5BAkYNESACIAQ2AjwgAiABNgI4QeKFwABBDBDCASEBIABBgICAgHg2ApwBIAAgATYCAAwgCyAPQQJGDQ8gAiAENgI8IAIgATYCOEHuhcAAQQkQwgEhASAAQYCAgIB4NgKcASAAIAE2AgAMHwsgAigCiAFBgICAgHhGDQ0gAiAENgI8IAIgATYCOEGJhMAAQQQQwgEhASAAQYCAgIB4NgKcASAAIAE2AgAMHgsgAigClAFBgYCAgHhGDQsgAiAENgI8IAIgATYCOEH3hcAAQQwQwgEhASAAQYCAgIB4NgKcASAAIAE2AgAMHQsgAigCoAFBgICAgHhGDQkgAiAENgI8IAIgATYCOEGDhsAAQQYQwgEhASAAQYCAgIB4NgKcASAAIAE2AgAMHAsgAigCrAFBgYCAgHhGDQcgAiAENgI8IAIgATYCOEGJhsAAQQ0QwgEhASAAQYCAgIB4NgKcASAAIAE2AgAMGwsgAigCuAFBgYCAgHhGDQUgAiAENgI8IAIgATYCOEGWhsAAQQ4QwgEhASAAQYCAgIB4NgKcASAAIAE2AgAMGgsgB0UNAyACIAQ2AjwgAiABNgI4QaSGwABBCRDCASEBIABBgICAgHg2ApwBIAAgATYCAAwZCyACKALEAUGBgICAeEYNASACIAQ2AjwgAiABNgI4Qa2GwABBGhDCASEBIABBgICAgHg2ApwBIAAgATYCAAwYCyACIAQ2AjwgAiABNgI4AkAgAigCLEGBgICAeEciEkUEQCACQYCAgIB4NgLQAQwBCyACQdgBaiACQTRqKAIANgIAIAIgAikCLDcD0AELAkAgAUUEQEGVhcAAQQYQwQEhBCAAQYCAgIB4NgKcASAAIAQ2AgBBACEHQQAhBEEAIQNBACEIQQAhCUEAIQBBACEFQQAhBgwBCyACQfgBaiACQdAAaikDADcDACACQfABaiACQcgAaikDADcDACACQegBaiACQUBrKQMANwMAIAIgAikDODcD4AECQCACKAJYQYCAgIB4RyIGRQRAQZuFwABBCBDBASEEIABBgICAgHg2ApwBIAAgBDYCAEEAIQdBACEEQQAhA0EAIQhBACEJQQAhAEEAIQUMAQsgAkGIAmogAkHgAGooAgA2AgAgAiACKQJYNwOAAgJAIAIoAmRBgYCAgHhHIgVFBEAgAkGAgICAeDYCkAIMAQsgAkGYAmogAkHsAGooAgA2AgAgAiACKQJkNwOQAgsCfyAKQQJGBEBBo4XAAEEKEMEBIQQgAEGAgICAeDYCnAEgACAENgIAQQAhB0EAIQRBACEDQQAhCEEAIQlBAAwBCwJAIAIoAnAiE0GBgICAeEcEQCACQagCaiACQfgAaigCADYCACACIAIpAnA3A6ACDAELIAJBgICAgHg2AqACCwJAIAIoAnxBgICAgHhHIglFBEBBtoXAAEEIEMEBIQQgAEGAgICAeDYCnAEgACAENgIAQQAhB0EAIQRBACEDQQAhCAwBCyACQbgCaiACQYQBaigCADYCACACIAIpAnw3A7ACAkACfyALQQJGBEBBvoXAAEEOEMEBDAELIAxBAkcEQCANQQJGBEBB2IXAAEEKEMEBDAILIA5BAkYEQEHihcAAQQwQwQEMAgsgD0ECRgRAQe6FwABBCRDBAQwCCyACKAKIAUGAgICAeEciCEUEQEGJhMAAQQQQwQEhBCAAQYCAgIB4NgKcASAAIAQ2AgBBACEHQQAhBEEAIQMMAwsgAkHIAmogAkGQAWooAgA2AgAgAiACKQKIATcDwAICQCACKAKUAUGBgICAeEciAwRAIAJB2AJqIAJBnAFqKAIANgIAIAIgAikClAE3A9ACDAELIAJBgICAgHg2AtACCwJAIAIoAqABQYCAgIB4RyIERQRAQYOGwABBBhDBASEHIABBgICAgHg2ApwBIAAgBzYCAEEAIQcMAQsgAkHoAmogAkGoAWooAgA2AgAgAiACKQKgATcD4AICQCACKAKsASIRQYGAgIB4RwRAIAJB+AJqIAJBtAFqKAIANgIAIAIgAikCrAE3A/ACDAELIAJBgICAgHg2AvACCwJAIAIoArgBIhRBgYCAgHhHBEAgAkGIA2ogAkHAAWooAgA2AgAgAiACKQK4ATcDgAMMAQsgAkGAgICAeDYCgAMLIAdFBEAgEUGBgICAeEchByAUQYGAgIB4RyERQaSGwABBCRDBASEKIABBgICAgHg2ApwBIAAgCjYCACACQYADahC+AiACQfACahC+AiACQeACahD/AgwBCyAAIAIpA9ABNwIkIAAgAikDODcDACAAIAIpAlg3AnggAEEsaiACQdgBaigCADYCACAAQQhqIAJBQGspAwA3AwAgAEEQaiACQcgAaikDADcDACAAQRhqIAJB0ABqKQMANwMAIABBgAFqIAJB4ABqKAIANgIAIAIoAsQBIQEgAikCyAEhFSAAQThqIAJBmAJqKAIANgIAIAAgAikDkAI3AjAgACACKQOgAjcCPCAAQcQAaiACQagCaigCADYCACAAIAIpAnw3AoQBIABBjAFqIAJBhAFqKAIANgIAIAAgAikCiAE3ApABIABBmAFqIAJBkAFqKAIANgIAIABB0ABqIAJB2AJqKAIANgIAIAAgAikD0AI3AkggACAPQQFxOgCtASAAIA5BAXE6AKwBIAAgDUEBcToAqwEgACAMQQFxOgCqASAAIAtBAXE6AKkBIAAgCkEBcToAqAEgACAVNwNwIABBgICAgHggASABQYGAgIB4Rhs2AmwgACAQNgIgIABBpAFqIAJBqAFqKAIANgIAIAAgAikCoAE3ApwBIABB3ABqIAJB+AJqKAIANgIAIAAgAikD8AI3AlQgAEHoAGogAkGIA2ooAgA2AgAgACACKQOAAzcCYAwhCyACQdACahC+AiACQcACahD/AgwCC0HMhcAAQQwQwQELIQQgAEGAgICAeDYCnAEgACAENgIAQQAhB0EAIQRBACEDQQAhCAsgAkGwAmoQ/wILIAJBoAJqEL4CIBNBgYCAgHhHCyEAIAJBkAJqEL4CIAJBgAJqEP8CCyACQeABahDSAQsgAkHQAWoQvgIgAUEARwwYCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQpgEgAigC5AEhAyACKALgASIFQYGAgIB4RgRAIAIgBDYCPCACIAE2AjggAEGAgICAeDYCnAEgACADNgIADBgLIAIoAugBIQYgAigCxAFBgYCAgHhHBEAgAkHEAWoQvgILIAIgBjYCzAEgAiADNgLIASACIAU2AsQBDBMLDBoLIAIoAhggAkEANgIYBEAgAigCHCEFIwBBEGsiAyQAIAMgBTYCDCMAQTBrIgckACAHQQhqIANBDGoiEBCJAkEBIQYgAwJ/IAcoAghBAUYEQCAHKQMQIhVCgICAgAh8QoCAgIAQWgRAIAdBAjoAGCAHIBU3AyAgB0EYaiAHQS9qQYCAwAAQtwEMAgtBACEGIBWnDAELIBAgB0EvakGAgMAAEEoLNgIEIAMgBjYCACAHQTBqJAAgAygCBCEHIAMoAgAhBiAFQYQBTwRAIAUQ9AELIAIgBjYCACACIAc2AgQgA0EQaiQAQQEhByACKAIEIRAgAigCAEEBcUUNEiAAQYCAgIB4NgKcASAAIBA2AgAgAiAENgI8IAIgATYCOAwWCwwZCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQpgEgAigC5AEhAyACKALgASIFQYGAgIB4RgRAIAIgBDYCPCACIAE2AjggAEGAgICAeDYCnAEgACADNgIADBYLIAIoAugBIQYgAigCuAFBgYCAgHhHBEAgAkG4AWoQvgILIAIgBjYCwAEgAiADNgK8ASACIAU2ArgBDBELDBgLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBCmASACKALkASEDIAIoAuABIgVBgYCAgHhGBEAgAiAENgI8IAIgATYCOCAAQYCAgIB4NgKcASAAIAM2AgAMFQsgAigC6AEhBiACKAKsAUGBgICAeEcEQCACQawBahC+AgsgAiAGNgK0ASACIAM2ArABIAIgBTYCrAEMEAsMFwsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEIUBIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwUCyACKALoASEGIAJBoAFqEL4CIAIgBjYCqAEgAiADNgKkASACIAU2AqABDA8LDBYLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBCmASACKALkASEDIAIoAuABIgVBgYCAgHhGBEAgAiAENgI8IAIgATYCOCAAQYCAgIB4NgKcASAAIAM2AgAMEwsgAigC6AEhBiACKAKUAUGBgICAeEcEQCACQZQBahC+AgsgAiAGNgKcASACIAM2ApgBIAIgBTYClAEMDgsMFQsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEIUBIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwSCyACKALoASEGIAJBiAFqEL4CIAIgBjYCkAEgAiADNgKMASACIAU2AogBDA0LDBQLIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBCsASACLQDgAQRAIAIgBDYCPCACIAE2AjggAigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgAMEQsgAi0A4QEhDwwMCwwTCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQrAEgAi0A4AEEQCACIAQ2AjwgAiABNgI4IAIoAuQBIQEgAEGAgICAeDYCnAEgACABNgIADBALIAItAOEBIQ4MCwsMEgsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEKwBIAItAOABBEAgAiAENgI8IAIgATYCOCACKALkASEBIABBgICAgHg2ApwBIAAgATYCAAwPCyACLQDhASENDAoLDBELIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBCsASACLQDgAQRAIAIgBDYCPCACIAE2AjggAigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgAMDgsgAi0A4QEhDAwJCwwQCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQrAEgAi0A4AEEQCACIAQ2AjwgAiABNgI4IAIoAuQBIQEgAEGAgICAeDYCnAEgACABNgIADA0LIAItAOEBIQsMCAsMDwsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEIUBIAIoAuQBIQMgAigC4AEiBUGAgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwMCyACKALoASEGIAJB/ABqEL4CIAIgBjYChAEgAiADNgKAASACIAU2AnwMBwsMDgsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEKYBIAIoAuQBIQMgAigC4AEiBUGBgICAeEYEQCACIAQ2AjwgAiABNgI4IABBgICAgHg2ApwBIAAgAzYCAAwLCyACKALoASEGIAIoAnBBgYCAgHhHBEAgAkHwAGoQvgILIAIgBjYCeCACIAM2AnQgAiAFNgJwDAYLDA0LIAIoAhggAkEANgIYBEAgAkHgAWogAigCHBCsASACLQDgAQRAIAIgBDYCPCACIAE2AjggAigC5AEhASAAQYCAgIB4NgKcASAAIAE2AgAMCgsgAi0A4QEhCgwFCwwMCyACKAIYIAJBADYCGARAIAJB4AFqIAIoAhwQpgEgAigC5AEhAyACKALgASIFQYGAgIB4RgRAIAIgBDYCPCACIAE2AjggAEGAgICAeDYCnAEgACADNgIADAkLIAIoAugBIQYgAigCZEGBgICAeEcEQCACQeQAahC+AgsgAiAGNgJsIAIgAzYCaCACIAU2AmQMBAsMCwsgAigCGCACQQA2AhgEQCACQeABaiACKAIcEIUBIAIo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AEgMAgsgCSAEQQRqEE8gA0EEOgBIDAELIAkgBEEEahDyASADQQU6AEgLIAVBDGohBSAEQRhqIQQgA0GUAWohCiADQTxqIQ0gA0HIAGohCyADQTRqIg4oAgAiAi8BkgMiBkELTwRAQeylwABBIEHopsAAEOYBAAsgAiAGQQFqOwGSAyACIAZBDGxqIg8gDSkCADcCjAIgD0GUAmogDUEIaigCADYCACAKIAY2AgggCiACNgIAIAogDigCBDYCBCACIAZBGGxqIgIgCykDADcDACACQQhqIAtBCGopAwA3AwAgAkEQaiALQRBqKQMANwMAIAdBAWoiByABLwGSA0kNAAsLIAAgBzYCCCAAQQA2AgQgACAMNgIADAMLIANBADYCUCADIAI2AkwgAyABNgJIIANBKGogA0HIAGoQlQIgA0GUAWogAygCKCADKAIsEDsgAygClAEiBEUNASADKAKYASEHELACIgUgBDYCmAMgBUEAOwGSAyAFQQA2AogCIARBADsBkAMgBCAFNgKIAiADQSBqIgQgB0EBajYCBCAEIAU2AgAgAyADKAIkIgQ2ApgBIAMgAygCICIFNgKUASADIAQ2AkAgAyAFNgI8IAEvAZIDBEAgAUGMAmohBSADQfgAaiEGIANB8ABqQQRyIQggASEEQQAhBwNAIANB5ABqIAUQrgECQAJAAkACQAJAAkACQCAELQAAQQFrDgUBAgMEBQALIANBADoAcAwFCyADQYABaiAEQRBqKQMANwMAIAYgBEEIaikDADcDACADIAQpAwA3A3AMBAsgBkEIaiAEQRBqKQMANwMAIAYgBEEIaikDADcDACADQQI6AHAMAwsgCCAEQQRqEK4BIANBAzoAcAwCCyAIIARBBGoQTyADQQQ6AHAMAQsgCCAEQQRqEPIBIANBBToAcAsgAyAHQQFqIgc2AlAgAyACNgJMIAMgATYCSCADQRhqIANByABqEJUCIANBiAFqIAMoAhggAygCHBA7IAMoApABIQoCfyADKAKIASIJBEAgAygCjAEMAQsgA0EQahC0AiADKAIQIQkgAygCFAshCyADQTxqIANB5ABqIANB8ABqIAkgCxCBASADIAogAygCnAFqQQFqNgKcASAFQQxqIQUgBEEYaiEEIAcgAS8BkgNJDQALCyAAIAMpApQBNwIAIABBCGogA0GcAWooAgA2AgAMAgtB/JrAABDyAgALQcSbwAAQ8gIACyADQaABaiQADwtBjJvAAEEoQbSbwAAQ5gEAC48hAwp/AX4BfCMAQaABayICJAAgAiABNgIsAkACQAJAAkAgAkEsahDbAkUEQEEBQQIgAigCLBCYAyIBQQFGG0EAIAEbIgFBAkcEQCAAIAE6AAEgAEEBOgAADAMLAkACQAJAIAIoAiwlARADQQFHBEAgAkEYaiACKAIsEI4DIAJBMGogAigCGCACKwMgEKMCIAIoAjBFDQEgAisDOCENIAJBLGoQ7QINAyACQfgAaiIBIA29Qv///////////wCDQv/////////3/wBYBH4gASANOQMIQgIFQgMLNwMAIAJBADoAiAEgACACKQN4QgNSBH8gAkHvAGogAkGAAWopAwA3AAAgAiACKQN4NwBnIAJBiAFqEOgBQQIFQQALOgAAIAAgAikAYDcAASAAQQlqIAJB6ABqKQAANwAAIABBEGogAkHvAGopAAA3AAAMBgsgAiACKAIsIgE2AmAgAkGIAWogAkHgAGoQ4gEgAigCiAFBAUcNASABIAIpA5ABIgwQ0AIiARD1AiABQYQBTwRAIAEQ9AELIAIoAmAhAUUNASABQYQBTwRAIAEQ9AELIAAgDDcDECAAQQI6AAAgACAMQj+INwMIDAcLIAJBEGogAigCLBCPAwJAIAIoAhAiAUUNACACQQhqIAEgAigCFBDjASACQcQAaiACKAIIIAIoAgwQxAIgAigCREGAgICAeEYNACACQZMBaiACQcwAaigCADYAACAAQQM6AAAgAiACKQJENwCLASAAIAIpAIgBNwABIABBCGogAkGPAWopAAA3AAAMBQsgAkEsaiIBEOwCDQMgAkGIAWogARCSASACKAKIAUGAgICAeEcEQCACQdgAaiACQZABaigCADYCACACIAIpAogBNwNQIwBBIGsiASQAIAJB0ABqIgMpAgQhDCABQQY6AAggASAMNwIMIAFBCGogAUEfakGslsAAELgBIQQgAEEGOgAAIAAgBDYCBCADEP8CIAFBIGokAAwFCyACKAIsEJkDQQFGBEAQ1wIiASACKAIsEPQCQQFHBEAgAUGEAUkNByABEPQBDAcLIAFBhAFPBEAgARD0AQsgAkEsaigCACUBECINBgsgAkEsaiACQeAAakHogcAAEEohASAAQQY6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rYYgA4QhAyACQQJyIQILIAAgAiAJSQR+IAEgAiAKamoxAAAgAkEDdK2GIAOEBSADCzcDMAsgACAJNgI8C/gFAgF/AXwjAEEwayICJAACfwJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQCAALQAAQQFrDhEBAgMEBQYHCAkKCwwNDg8QEQALIAIgAC0AAToACCACQQI2AhQgAkHgs8AANgIQIAJCATcCHCACQTk2AiwgAiACQShqNgIYIAIgAkEIajYCKCABKAIAIAEoAgQgAkEQahBLDBELIAIgACkDCDcDCCACQQI2AhQgAkH8s8AANgIQIAJCATcCHCACQRw2AiwgAiACQShqNgIYIAIgAkEIajYCKCABKAIAIAEoAgQgAkEQahBLDBALIAIgACkDCDcDCCACQQI2AhQgAkH8s8AANgIQIAJCATcCHCACQRs2AiwgAiACQShqNgIYIAIgAkEIajYCKCABKAIAIAEoAgQgAkEQahBLDA8LIAArAwghAyACQQI2AhQgAkGctMAANgIQIAJCATcCHCACQTo2AgwgAiADOQMoIAIgAkEIajYCGCACIAJBKGo2AgggASgCACABKAIEIAJBEGoQSwwOCyACIAAoAgQ2AgggAkECNgIUIAJBuLTAADYCECACQgE3AhwgAkE7NgIsIAIgAkEoajYCGCACIAJBCGo2AiggASgCACABKAIEIAJBEGoQSwwNCyACIAApAgQ3AgggAkEBNgIUIAJB0LTAADYCECACQgE3AhwgAkE8NgIsIAIgAkEoajYCGCACIAJBCGo2AiggASgCACABKAIEIAJBEGoQSwwMCyABQcmzwABBChDPAgwLCyABQdi0wABBChDPAgwKCyABQeK0wABBDBDPAgwJCyABQe60wABBDhDPAgwICyABQfy0wABBCBDPAgwHCyABQYS1wABBAxDPAgwGCyABQYe1wABBBBDPAgwFCyABQYu1wABBDBDPAgwECyABQZe1wABBDxDPAgwDCyABQaa1wABBDRDPAgwCCyABQbO1wABBDhDPAgwBCyABIAAoAgQgACgCCBDPAgsgAkEwaiQAC+sFAgp/AX4gACgCACEFIAAoAgRBAWoiCEEDdiAIQQdxQQBHaiIJBEAgBSEEA0AgBCAEKQMAIg5Cf4VCB4hCgYKEiJCgwIABgyAOQv/+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IQUMAgsgA0EHOgBwIANB8ABqIAEgAhC4ASEADAILIANBGGogBBCOAyADQcgAaiADKAIYIAMrAyAQowIgAygCSEUEQCADQRBqIAQQjwMCfwJAIAMoAhAiBEUNACADQQhqIAQgAygCFBDjASADQdgAaiADKAIIIAMoAgwQxAIgAygCWEGAgICAeEYNACADQTBqIANB4ABqKAIAIgQ2AgAgAyADKQJYNwMoQQUhBUEBIQcgAygCLAwBCyADQeQAaiAAEJIBAn8gAygCZCIGQYCAgIB4RiIHRQRAIANBQGsiACEEIANBPGohCCAAIANB7ABqKAIANgIAIAMgAykCZDcDOEEGDAELIANBMGohBCADQSxqIQggA0EBNgJ0IANB6J3AADYCcCADQgE3AnwgA0EaNgKMASADIAA2AogBIAMgA0GIAWo2AnggA0EoaiADQfAAahBUQRELIQUgBkGAgICAeEchBiAEKAIAIQQgCCgCAAshACAErSEKDAELQQMhBSADKQNQIQoLIAMgCjcDeCADIAA2AnQgAyAJOgBxIAMgBToAcCADQfAAaiABIAIQuAEhACAGBEAgA0E4ahD/AgsgB0UNACADQShqEP8CCyADQZABaiQAIAALugQBCH8jAEEQayIDJAAgAyABNgIEIAMgADYCACADQqCAgIAONwIIAn8CQAJAAkAgAigCECIJBEAgAigCFCIADQEMAgsgAigCDCIARQ0BIAIoAggiASAAQQN0aiEEIABBAWtB/////wFxQQFqIQYgAigCACEAA0ACQCAAQQRqKAIAIgVFDQAgAygCACAAKAIAIAUgAygCBCgCDBEDAEUNAEEBDAULQQEgASgCACADIAFBBGooAgARAAANBBogAEEIaiEAIAQgAUEIaiIBRw0ACwwCCyAAQRhsIQogAEEBa0H/////AXFBAWohBiACKAIIIQQgAigCACEAA0ACQCAAQQRqKAIAIgFFDQAgAygCACAAKAIAIAEgAygCBCgCDBEDAEUNAEEBDAQLQQAhB0EAIQgCQAJAAkAgBSAJaiIBQQhqLwEAQQFrDgIBAgALIAFBCmovAQAhCAwBCyAEIAFBDGooAgBBA3RqLwEEIQgLAkACQAJAIAEvAQBBAWsOAgECAAsgAUECai8BACEHDAELIAQgAUEEaigCAEEDdGovAQQhBwsgAyAHOwEOIAMg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IARBEGpNDQAgACAEIAFBAXFyQQJyNgIEIAAgBGoiASACIARrIgRBA3I2AgQgACACaiICIAIoAgRBAXI2AgQgASAEEFELIABBCGohAwsgAwvqAgIGfwJ+IwBBIGsiBSQAQRQhAyAAIglC6AdaBEAgCSEKA0AgBUEMaiADaiIEQQNrIAogCkKQzgCAIglCkM4Afn2nIgZB//8DcUHkAG4iB0EBdCIIQarrwABqLQAAOgAAIARBBGsgCEGp68AAai0AADoAACAEQQFrIAYgB0HkAGxrQf//A3FBAXQiBkGq68AAai0AADoAACAEQQJrIAZBqevAAGotAAA6AAAgA0EEayEDIApC/6ziBFYgCSEKDQALCyAJQglWBEAgAyAFakELaiAJpyIEIARB//8DcUHkAG4iBEHkAGxrQf//A3FBAXQiBkGq68AAai0AADoAACADQQJrIgMgBUEMamogBkGp68AAai0AADoAACAErSEJCyAAUEUgCVBxRQRAIANBAWsiAyAFQQxqaiAJp0EBdEEecUGq68AAai0AADoAAAsgAiABQQFBACAFQQxqIANqQRQgA2sQPyAFQSBqJAAL5gIBCH8jAEEQayIGJABBCiEDIAAiBEHoB08EQCAEIQUDQCAGQQZqIANqIgdBA2sgBSAFQZDOAG4iBEGQzgBsayIIQf//A3FB5ABuIglBAXQiCkGq68AAai0AADoAACAHQQRrIApBqevAAGotAAA6AAAgB0EBayAIIAlB5ABsa0H//wNxQQF0IghBquvAAGotAAA6AAAgB0ECayAIQanrwABqLQAAOgAAIANBBGshAyAFQf+s4gRLIAQhBQ0ACwsCQCAEQQlNBEAgBCEFDAELIAMgBmpBBWogBCAEQf//A3FB5ABuIgVB5ABsa0H//wNxQQF0IgRBquvAAGotAAA6AAAgA0ECayIDIAZBBmpqIARBqevAAGotAAA6AAALQQAgACAFG0UEQCADQQFrIgMgBkEGamogBUEBdEEecUGq68AAai0AADoAAAsgAiABQQFBACAGQQZqIANqQQogA2sQPyAGQRBqJAALggMBBH8gACgCDCECAkACQAJAIAFBgAJPBEAgACgCGCEDAkACQCAAIAJGBEAgAEEUQRAgACgCFCICG2ooAgAiAQ0BQQAhAgwCCyAAKAIIIgEgAjYCDCACIAE2AggMAQsgAEEUaiAAQRBqIAIbIQQDQCAEIQUgASICQRRqIAJBEGogAigCFCIBGyEEIAJBFEEQIAEbaigCACIBDQALIAVBADYCAAsgA0UNAgJAIAAoAhxBAnRBjIDBAGoiASgCACAARwRAIAMoAhAgAEYNASADIAI2AhQgAg0DDAQLIAEgAjYCACACRQ0EDAILIAMgAjYCECACDQEMAgsgACgCCCIAIAJHBEAgACACNgIMIAIgADYCCA8LQaSDwQBBpIPBACgCAEF+IAFBA3Z3cTYCAA8LIAIgAzYCGCAAKAIQIgEEQCACIAE2AhAgASACNgIYCyAAKAIUIgBFDQAgAiAANgIUIAAgAjYCGA8LDwtBqIPBAEGog8EAKAIAQX4gACgCHHdxNgIAC/ICAQF/AkAgAgRAIAEtAABBME0NASAFQQI7AQACQAJAAkACQAJAIAPBIgZBAEoEQCAFIAE2AgQgAiADQf//A3EiA0sNASAFQQA7AQwgBSACNgIIIAUgAyACazYCECAEDQJBAiEBDAULIAUgAjYCICAFIAE2AhwgBUECOwEYIAVBADsBDCAFQQI2AgggBUH558AANgIEIAVBACAGayIDNgIQQQMhASACIARPDQQgBCACayICIANNDQQgAiAGaiEEDAMLIAVBAjsBGCAFQQE2AhQgBUH458AANgIQIAVBAjsBDCAFIAM2AgggBSACIANrIgI2AiAgBSABIANqNgIcIAIgBEkNAUEDIQEMAwsgBUEBNgIgIAVB+OfAADYCHCAFQQI7ARgMAQsgBCACayEECyAFIAQ2AiggBUEAOwEkQQQhAQsgACABNgIEIAAgBTYCAA8LQeDkwABBIUGE58AAEOYBAAtBlOfAAEEfQbTnwAAQ5gEAC8oCAQZ/IAEgAkEBdGohCSAAQYD+A3FBCHYhCiAAQf8BcSEMAkACQAJAAkADQCABQQJqIQsgByABLQABIgJqIQggCiABLQAAIgFHBEAgASAKSw0EIAghByALIgEgCUcNAQwECyAHIAhLDQEgBCAISQ0CIAMgB2ohAQNAIAJFBEAgCCEHIAsiASAJRw0CDAULIAJBAWshAiABLQAAIAFBAWohASAMRw0ACwtBACECDAMLIAcgCEG88MAAEPACAAsgCCAEQbzwwAAQ7wIACyAAQf//A3EhByAFIAZqIQNBASECA0AgBUEBaiEAAkAgBSwAACIBQQBOBEAgACEFDAELIAAgA0cEQCAFLQABIAFB/wBxQQh0ciEBIAVBAmohBQwBC0Gs8MAAEPICAAsgByABayIHQQBIDQEgAkEBcyECIAMgBUcNAAsLIAJBAXELygIBB38jAEEwayIDJAAgAiABKAIAIgUvAZIDIgcgASgCCCIGQX9zaiIBOwGSAyADQRBqIAVBjAJqIgggBkEMbGoiCUEIaigCADYCACADQSBqIAUgBkEYbGoiBEEIaikDADcDACADQShqIARBEGopAwA3AwAgAyAJKQIANwMIIAMgBCkDADcDGAJAIAFBDEkEQCAHIAZBAWoiBGsgAUcNASABQQxsIgcEQCACQYwCaiAIIARBDGxqIAf8CgAACyABQRhsIgEEQCACIAUgBEEYbGogAfwKAAALIAUgBjsBkgMgACADKQMINwIAIABBCGogA0EQaigCADYCACAAIAMpAxg3AxAgAEEYaiADQSBqKQMANwMAIABBIGogA0EoaikDADcDACADQTBqJAAPCyABQQtBgKjAABDvAgALQcinwABBKEHwp8AAEOYBAAu2AwECfyMAQeACayICJAACQAJAIAFFBEAjAEHwAmsiASQAAkACQCAABEAgAEEIayIDKAIAQQFHDQEgAUEIaiAAQegC/AoAACADQQA2AgACQCADQX9GDQAgAEEEayIAIAAoAgBBAWsiADYCACAADQAgA0HwAkEIEOkCCyACIAFBEGpB4AL8CgAAIAFB8AJqJAAMAgsQhgMAC0HgiMAAQT8QhAMACyACQYwCahC+AiACQZgCahC+AiACKAJIBEAgAkHIAGoQ0QELIAJB6AFqEP8CIAJBpAJqEL4CIAIoAmgEQCACQegAahDSAQsgAigCiAEEQCACQYgBahDRAQsgAkGwAmoQvgIgAigCqAEEQCACQagBahDTAQsgAkH0AWoiABD7ASAAEIUDIAJBgAJqIgAQhgIgABCCAyACKALUAiIABEAgABCkASAAQbABQQgQ6QILIAJBvAJqEL4CIAIoAsgBBEAgAkHIAWoQ0QELIAJByAJqEL4CIAJBKGoQ0gEMAQsgAEUNASACIABBCGsiADYCACAAIAAoAgBBAWsiADYCACAADQAgAhBnCyACQeACaiQADwsQhgMAC8QCAQR/IABCADcCECAAAn9BACABQYACSQ0AGkEfIAFB////B0sNABogAUEGIAFBCHZnIgNrdkEBcSADQQF0a0E+agsiAjYCHCACQQJ0QYyAwQBqIQRBASACdCIDQaiDwQAoAgBxRQRAIAQgADYCACAAIAQ2AhggACAANgIMIAAgADYCCEGog8EAQaiDwQAoAgAgA3I2AgAPCwJAAkAgASAEKAIAIgMoAgRBeHFGBEAgAyECDAELIAFBGSACQQF2a0EAIAJBH0cbdCEFA0AgAyAFQR12QQRxaiIEKAIQIgJFDQIgBUEBdCEFIAIhAyACKAIEQXhxIAFHDQALCyACKAIIIgEgADYCDCACIAA2AgggAEEANgIYIAAgAjYCDCAAIAE2AggPCyAEQRBqIAA2AgAgACADNgIYIAAgADYCDCAAIAA2AggLwQMBB38jAEHQAGsiAyQAIANBIGoiBCACKAIEIgU2AgAgBCAFIAIoAghBGGxqNgIEIAMgAykDIDcCKCADQRhqIgJBATYCACACIANBKGoiAigCBCACKAIAa0EYbjYCBCADQUBrIAEgAygCGCADKAIcELkCIAACfwJAAkAgAygCQEUEQCADKAJEIQEMAQsgA0E4aiADQcgAaigCADYCACADIAMpAkA3AzAgAygCKCECIAMoAiwhCQNAIAIgCUYNAiADIAI2AkAgAkEYaiECIANBEGohBSMAQRBrIgEkACADQTBqIgQoAgghByABQQhqIANBQGsoAgAgBCgCABBNQQEhBiABKAIMIQggASgCCEEBcUUEQCAEQQRqIAcgCBDrAiAEIAdBAWo2AghBACEGCyAFIAg2AgQgBSAGNgIAIAFBEGokACADKAIQQQFxRQ0ACyADKAIUIQEgAyACNgIoIAMoAjQiAkGEAUkNACACEPQBC0EBDAELIAMgAjYCKCADQcgAaiADQThqKAIANgIAIAMgAykDMDcDQCADQQhqIANBQGsQ0wIgAygCDCEBIAMoAggLNgIAIAAgATYCBCADQdAAaiQAC9sHAQR/IwBBIGsiBCQAAkACQCABKAIIIgIgASgCDEYNACABQRBqIQUCQANAAkAgASACQQhqNgIIIAQgAigCACACKAIEEIMCNgIUAkAgBSAEQRRqEN4CIgMQlwNBAUYEQCAEKAIUIAUoAgAQ9AJBAUcNAQsCQCABKAIARQ0AIAEoAgQiBUGEAUkNACAFEPQBCyABIAM2AgRBASEFIAFBATYCACAEQQhqIAIoAgAgAigCBBDaAiAEQRhqIQECQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQCAEKAIIIgIgBCgCDCIDQa6JwABBBhCyAkUEQCACIANBtInAAEEFELICDQEgAiADQbmJwABBFxCyAg0CIAIgA0HQicAAQQUQsgINAyACIANB1YnAAEELELICDQQgAiADQeCJwABBBRCyAg0FIAIgA0HlicAAQQcQsgINBiACIANB7InAAEEJELICDQcgAiADQfWJwABBCxCyAg0IIAIgA0GAisAAQQoQsgINCSACIANBiorAAEENELICDQogAiADQZeKwABBBBCyAg0LIAIgA0GbisAAQQoQsgINDCACIANBpYrAAEEFELICDQ0gAiADQaqKwABBCxCyAg0OIAIgA0G1isAAQQsQsgINDyACIANBwIrAAEEcELICDRAgAiADQdyKwABBHxCyAg0RIAIgA0H7isAAQQQQsgINEiACIANB/4rAAEEEELICDRMgAiADQYOLwABBCBCyAg0UIAIgA0GLi8AAQQcQsgINFSACIANBkovAAEEMELICRQRAIAFBFzoAAQwXCyABQRY6AAEMFgsgAUEAOgABDBULIAFBAToAAQwUCyABQQI6AAEMEwsgAUEDOgABDBILIAFBBDoAAQwRCyABQQU6AAEMEAsgAUEGOgABDA8LIAFBBzoAAQwOCyABQQg6AAEMDQsgAUEJOgABDAwLIAFBCjoAAQwLCyABQQs6AAEMCgsgAUEMOgABDAkLIAFBDToAAQwICyABQQ46AAEMBwsgAUEPOgABDAYLIAFBEDoAAQwFCyABQRE6AAEMBAsgAUESOgABDAMLIAFBEzoAAQwCCyABQRQ6AAEMAQsgAUEVOgABCyABQQA6AAAgBC0AGEUNASAAIAQoAhw2AgQMAwsgA0GEAU8EQCADEPQBCyAEKAIUIgJBhAFPBEAgAhD0AQsgASgCCCICIAEoAgxHDQEMAwsLIAAgBC0AGToAAUEAIQULIAAgBToAACAEKAIUIgBBhAFJDQEgABD0AQwBCyAAQYAwOwEACyAEQSBqJAAL6wYBBH8jAEEgayIEJAACQAJAIAEoAggiAiABKAIMRg0AIAFBEGohBQJAA0ACQCABIAJBCGo2AgggBCACKAIAIAIo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C0HcpcAAEPICAAsgECAFKAJINgIIIBAgBSkDQDcCAAsgBUGQAWokACANKAIMIQIgCSgCDCEKIAkoAgQhASAJKAIIDAELIA0oAgwhAhCvAiIBQQA2AogCIAJBADYCBCACIAE2AgAgAUEBOwGSAyABIAMpAwA3AwAgAUEIaiADQQhqKQMANwMAIAFBEGogA0EQaikDADcDACABIA0pAgA3AowCIAFBlAJqIA1BCGooAgA2AgBBAAshAyACIAIoAghBAWo2AgggESAKNgIIIBEgAzYCBCARIAE2AgAgESANKAIMNgIMIAlBMGokACAAQQY6AAALIAxB0ABqJAALxwECAn8BfiMAQRBrIgEkAAJAAkACQCACrSADrX4iBkIgiKcNACAGpyICQQdqIgQgAkkNACAEQXhxIgQgA0EIamoiAiAESSACQfj///8HS3INACACBH9B0f/AAC0AABogAkEIENkCBUEICyIFDQFBCCACEIwDAAsQ2AEgACABKQMANwIEIABBADYCAAwBCyAAQQA2AgwgACADQQFrIgI2AgQgACAEIAVqNgIAIAAgAiADQQN2QQdsIAJBCEkbNgIICyABQRBqJAAL8QEBBH8gASADRwRAQQAPCwJAIAEEQEEAIQMDQCAAIANqIgQtAAAiByACIANqIgUtAABHDQICQAJAAkACQAJAAkAgB0EBaw4FBAABAgMFCyAEQQhqIAVBCGoQ1wFFDQcMBAsgBEEIaigCACAEQQxqKAIAIAVBCGooAgAgBUEMaigCABCyAkUNBgwDCyAEQQhqKAIAIARBDGooAgAgBUEIaigCACAFQQxqKAIAEHVFDQUMAgsgBEEEaiAFQQRqEFBFDQQMAQsgBEEBai0AACAFQQFqLQAARw0DCyADQRhqIQMgAUEBayIBDQALC0EBIQYLIAYLiAkBBH8jAEHgAGsiBCQAIAAoAgAhACAEQQA2AkwgBEKAgICAEDcCRCAEQdipwAA2AlQgBEKggICADjcCWCAEIARBxABqNgJQAn8gBEHQAGohAgJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAAkACQAJAIAAoAgBBAWsOGAECAwQFBgcICQoLDA0ODxAREhMUFRYXGAALIAIgACgCBCAAKAIIEM8CDBgLAn8jAEFAaiIDJAACQAJAAkAC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fyMAQTBrIgIkAAJAIAApAwBC////////////AINCgICAgICAgPj/AFoEQCACQQE2AhQgAkHEtcAANgIQIAJCATcCHCACQT02AiwgAiAANgIoIAIgAkEoajYCGCABKAIAIAEoAgQgAkEQahBLIQMMAQsgAkEAOgAMIAIgATYCCEEBIQMgAkEBNgIUIAJBxLXAADYCECACQgE3AhwgAkE9NgIsIAIgADYCKCACIAJBKGo2AhggAkEIaiACQRBqEIMDDQAgAi0ADEUEQCABQcy1wABBAhDPAg0BC0EAIQMLIAJBMGokACADC7ABAQd/IAEoAgAiBS8BkgMiCUEMbCEBQX8hAyAFQYwCaiEEIAIoAgghBiACKAIEIQVBASEIAkADQCABRQRAIAkhAwwCCyAEKAIIIQcgBCgCBCECIANBAWohAyABQQxrIQEgBEEMaiEEIAUgAiAGIAcgBiAHSRsQ1gEiAiAGIAdrIAIbIgJBAEogAkEASGtB/wFxIgJBAUYNAAsgAg0AQQAhCAsgACADNgIEIAAgCDYCAAvFAQECfyMAQSBrIgIkACACIAEQ6wECQAJAIAIoAgAiA0ECRwRAIAIoAgQhASADQQFxRQ0BIABBBzoAACAAIAE2AgQMAgsgAEEGOgAADAELIAJBCGogARA8IAItAAgiAUEGRwRAIAAgAi8ACTsAASAAIAIpAxA3AwggAEEDaiACLQALOgAAIABBEGogAkEYaikDADcDACAAIAIoAgw2AgQgACABOgAADAELIAIoAgwhASAAQQc6AAAgACABNgIECyACQSBqJAALwQECA38BfiMAQTBrIgIkACABKAIAQYCAgIB4RgRAIAEoAgwhAyACQRRqIgRBADYCACACQoCAgIAQNwIMIAJBIGogAygCACIDQQhqKQIANwMAIAJBKGogA0EQaikCADcDACACIAMpAgA3AxggAkEMakHAusAAIAJBGGoQSxogAkEIaiAEKAIAIgM2AgAgAiACKQIMIgU3AwAgAUEIaiADNgIAIAEgBTcCAAsgAEGIxcAANgIEIAAgATYCACACQTBqJAALyQEBAn8CQCAAKAIEQQFrIARGBEAgACgCACIALwGSAyIEQQtPDQEgACAEQQFqIgU7AZIDIAAgBEEMbGoiBiABKQIANwKMAiAGQZQCaiABQQhqKAIANgIAIAAgBEEYbGoiASACKQMANwMAIAFBCGogAkEIaikDADcDACABQRBqIAJBEGopAwA3AwAgACAFQQJ0aiADNgKYAyADIAU7AZADIAMgADYCiAIPC0H4psAAQTBBqKfAABDmAQALQeylwABBIEG4p8AAEOYBAAvIBQISfwF+IwBBMGsiACQAIABBEGoiDhBuIgZBEGoiCCgCADYCACAAQQhqIg8gBkEIaiIJKQIANwMAIAhBADYCACAJQgA3AgAgACAGKQIANwMAIAZCgICAgMAANwIAIwBBEGsiCiQAAkACQCAAKAIMIgUgACgCCCIBRgRAIAUiASAAKAIAIgNGBEDQb0GAASABIAFBgAFNGyIB/A8BIgJBf0YNAgJAIAAoAhAiA0UEQCAAIAI2AhAMAQsgAyAFaiACRw0DCyAKQQhqIQsjAEEQayIHJAACf0GBgICAeCAAKAIAIAAoAggiA2sgAU8NABogB0EIaiEMIwBBIGsiAiQAAkAgASADaiINIANJDQAgDa1CAoYiEkIgiKcNACASpyIQQfz///8HSw0AAn8gACgCACIDRQRAIAJBGGohBEEADAELIAJBHGohBCACQQQ2AhggAiAAKAIENgIUIANBAnQLIREgBCARNgIAIAJBCGpBBCAQIAJBFGoQkQEgAigCCEEBRgRAIAIoAhAhAyACKAIMIQQMAQsgAigCDCEEIAAgDTYCACAAIAQ2AgRBgYCAgHghBAsgDCADNgIEIAwgBDYCACACQSBqJABBgYCAgHggBygCCCICQYGAgIB4Rg0AGiAHKAIMIQEgAgshAiALIAE2AgQgCyACNgIAIAdBEGokACAKKAIIQYGAgIB4Rw0CIAAoAgAhAyAAKAIIIQELIAEgA08NASAAKAIEIAFBAnRqIAVBAWo2AgAgACABQQFqIgE2AggLIAEgBUsNAQsACyAAIAAoAgQgBUECdGooAgA2AgwgACgCECAKQRBqJAAgBWogAEEoaiAIKAIANgIAIABBIGogCSkCADcDACAGKQIAIRIgBiAAKQMANwIAIAkgDykDADcCACAIIA4oAgA2AgAgACASNwMYIABBGGpBBEEEEKsBIABBMGokAAuYAQIBfgF/IAACfwJAIAIgA2pBAWtBACACa3GtIAGtfiIEQiCIUARAIASnIgNBgICAgHggAmtNDQELIABBADYCBEEBDAELIANFBEAgACACNgIIIABBADYCBEEADAELQdH/wAAtAAAaIAMgAhDZAiIFBEAgACAFNgIIIAAgATYCBEEADAELIAAgAzYCCCAAIAI2AgRBAQs2AgALpw0CBn8DfiMAQTBrIgUkACAFIAE2AgwCQCAFQQxqENsCRQRAIAVBEGohAyMAQcABayICJAAgAiABNgIMIAJByABqIAJBDGoQcCACKAJIIQECQAJAAkACQAJAAkACQAJAIAItAEwiBEECaw4CAgABCyADQQA2AgAgAyABNgIEIAIoAgwiAUGDAUsNAwwECyACIAQ6AHQgAiABNgJwIAJBADYCaCACAn5B2IPBACgCAEEBRgRAQeiDwQApAwAhCEHgg8EAKQMADAELIAJByABqEM0BQdiDwQBCATcDAEHog8EAIAIpA1AiCDcDACACKQNICyIJNwNYQeCDwQAgCUIBfDcDACACIAg3A2AgAkIANwNQIAJBADYCTCACQZiDwAA2AkggAkGMAWohAQJAA0ACQCACQagBaiACQegAahB5AkACQAJAAkAgAigCqAEiB0GAgICAeGsOAgQAAQsgAigCrAEhAQwBCyACIAIpAqwBIgg3ArgBIAIgBzYCtAEgAigCaCACQQA2AmhFDQkgAkEQaiACKAJsEIUBIAIoAhBBgICAgHhHDQEgAigCFCEBIAJBtAFqEP8CCyADQQA2AgAgAyABNgIEIAJByABqENEBDAMLIAJBMGogAkEYaigCACIGNgIAIAJBiAFqIAhCIIinIgQ2AgAgAkFAayAENgIAIAJBoAFqIAY2AgAgAiACKQIQIgk3AyggAUEIaiAGNgIAIAEgCTcCACACIAc2AoABIAIgCKciBjYChAEgAiAJNwOYASACIAIpAoABNwM4IAJBEGoiByACQcgAaiACQThqIAJBmAFqEEYgBxC+AgwBCwsgAiAENgKIASACIAY2AoQBIAJBgICAgHg2AoABIAJBgAFqEI8CIANBGGogAkHgAGopAwA3AwAgA0EQaiACQdgAaikDADcDACADQQhqIAJB0ABqKQMANwMAIAMgAikDSDcDAAsgAkHoAGoQ/wEMAQsgAiACQQxqEPwBIAIoAgBBAXEEQCACIAIoAgQ2AjggAkEYaiACQThqELoCIAJBADYCJCACQQA2AhBBACEBIAIoAhgEQCACKAIgIgEgAigCHGsiBEEAIAEgBE8bIQELQQAhBEHgg8EAAn5B2IPBACgCAEEBRgRAQeiDwQApAwAhCEHgg8EAKQMADAELIAJBgAFqEM0BQdiDwQBCATcDAEHog8EAIAIpA4gBIgg3AwAgAikDgAELIgpCAXw3AwACQCABRQRAQZiDwAAhAQwBCyACQYABaiACQbQBakEYIAFBB00Ef0EEQQggAUEESRsFQX9BqtUCIAEgAUGq1QJPG0EDdEEHbkEBa2d2QQFqCxB0IAIoAoQBIQQgAigCgAEiAUUEQCACNQKIASEJQQAhAQwBCyACKQKIASEJIARBCWoiBkUNACABQf8BIAb8CwALIAIgCDcDYCACIAo3A1ggAiAJNwNQIAIgBDYCTCACIAE2AkggAkGAAWogAkEQahBoAkACQAJAIAIoAoABQYGAgIB4RwRAIAJBjAFqIQEDQCACQfgAaiACQZABaikCADcDACACQfAAaiACQYgBaiIEKQIANwMAIAIgAikCgAEiCDcDaCAIp0GAgICAeEYNAiACQaABaiAEKAIANgIAIAJBsAFqIAFBCGooAgA2AgAgAiACKQKAATcDmAEgAiABKQIANwOoASACQbQBaiIEIAJByABqIAJBmAFqIAJBqAFqEEYgBBC+AiACQYABaiACQRBqEGggAigCgAFBgYCAgHhHDQALCyADIAIoAoQBNgIEIANBADYCACACQcgAahDRASACKAIQRQ0CIAIoAhQiAUGDAUsNAQwCCyACQegAahCPAiADQRhqIAJB4ABqKQMANwMAIANBEGogAkHYAGopAwA3AwAgA0EIaiACQdAAaikDADcDACADIAIpA0g3AwAgAigCEEUNASACKAIUIgFBhAFJDQELIAEQ9AELIAIoAjgiAUGEAUkNASABEPQBDAELIAJBDGogAkG0AWpBiILAABBKIQEgA0EANgIAIAMgATYCBAsgAigCDCIBQYQBSQ0BCyABEPQBCyACQcABaiQADAELQbiSwABBMRCEAwALIAACfyAFKAIQRQRAIAAgBSgCFDYCBEEBDAELIAAgBSkDEDcDCCAAQSBqIAVBKGopAwA3AwAgAEEYaiAFQSBqKQMANwMAIABBEGogBUEYaikDADcDAEEACzYCAAwBCyAAQQA2AgAgAEEANgIIIAFBhAFJDQAgARD0AQsgBUEwaiQAC7gBAQF/IwBBMGsiAiQAIAIgATYCHCACQRBqIAEQjwMCQAJAIAIoAhAiAUUNACACQQhqIAEgAigCFBDjASACQSBqIAIoAgggAigCDBDEAiACKAIgQYCAgIB4Rg0AIAAgAikCIDcCACAAQQhqIAJBKGooAgA2AgAMAQsgAkEcaiACQS9qQZiCwAAQSiEBIABBgICAgHg2AgAgACABNgIECyACKAIcIgBBhAFPBEAgABD0AQsgAkEwaiQAC7gBAQF/IwBBMGsiAiQAIAIgATYCHCACQRBqIAEQjwMCQAJAIAIoAhAiAUUNACACQQhqIAEgAigCFBDjASACQSBqIAIoAgggAigCDBDEAiACKAIgQYCAgIB4Rg0AIAAgAikCIDcCACAAQQhqIAJBKGooAgA2AgAMAQsgAkEcaiACQS9qQciBwAAQSiEBIABBgICAgHg2AgAgACABNgIECyACKAIcIgBBhAFPBEAgABD0AQsgAkEwaiQAC6IBAQJ/IwBBIGsiAiQAAkACQCABKAIARQ0AIAIgARDeASACKAIAQQFxRQ0AIAIoAgQhAyABIAEoAgxBAWo2AgwgAkEIaiADEDwgAi0ACEEGRgRAIAAgAigCDDYCBCAAQQc6AAAMAgsgACACKQMINwMAIABBEGogAkEYaikDADcDACAAQQhqIAJBEGopAwA3AwAMAQsgAEEGOgAACyACQSBqJAALpQEBAX8jAEEgayIDJAAgA0EIaiAAEMMBIAMoAgghAAJAIAEEQCADIAEgAhDjASADQRRqIAMoAgAgAygCBBDEAgwBCyADQYCAgIB4NgIUCyAAQYwCahC+AiAAQZQCaiADQRxqKAIANgIAIAAgAykCFDcCjAIgAygCDEEANgIAIAMoAhAiACAAKAIAQQFrIgA2AgAgAEUEQCADQRBqEGcLIANBIGokAAujAQEBfyMAQSBrIgMkACADQQhqIAAQwwEgAygCCCEAAkAgAQRAIAMgASACEOMBIANBFGogAygCACADKAIEEMQCDAELIANBgICAgHg2AhQLIABBJGoQvgIgAEEsaiADQRxqKAIANgIAIAAgAykCFDcCJCADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQ+AELIANBIGokAAujAQEBfyMAQSBrIgMkACADQQhqIAAQwwEgAygCCCEAAkAgAQRAIAMgASACEOMBIANBFGogAygCACADKAIEEMQCDAELIANBgICAgHg2AhQLIABBMGoQvgIgAEE4aiADQRxqKAIANgIAIAAgAykCFDcCMCADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQ+AELIANBIGokAAukAQEBfyMAQSBrIgMkACADQQhqIAAQwwEgAygCCCEAAkAgAQRAIAMgASACEOMBIANBFGogAygCACADKAIEEMQCDAELIANBgICAgHg2AhQLIABBPGoQvgIgAEHEAGogA0EcaigCADYCACAAIAMpAhQ3AjwgAygCDEEANgIAIAMoAhAiACAAKAIAQQFrIgA2AgAgAEUEQCADQRBqEPgBCyADQSBqJAALpQEBAX8jAEEgayIDJAAgA0EIaiAAEMMBIAMoAgghAAJAIAEEQCADIAEgAhDjASADQRRqIAMoAgAgAygCBBDEAgwBCyADQYCAgIB4NgIUCyAAQcgAahC+AiAAQdAAaiADQRxqKAIANgIAIAAgAykCFDcCSCADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQ+AELIANBIGokAAulAQEBfyMAQSBrIgMkACADQQhqIAAQwwEgAygCCCEAAkAgAQRAIAMgASACEOMBIANBFGogAygCACADKAIEEMQCDAELIANBgICAgHg2AhQLIABB1ABqEL4CIABB3ABqIANBHGooAgA2AgAgACADKQIUNwJUIAMoAgxBADYCACADKAIQIgAgACgCAEEBayIANgIAIABFBEAgA0EQahD4AQsgA0EgaiQAC6UBAQF/IwBBIGsiAyQAIANBCGogABDDASADKAIIIQACQCABBEAgAyABIAIQ4wEgA0EUaiADKAIAIAMoAgQQxAIMAQsgA0GAgICAeDYCFAsgAEHgAGoQvgIgAEHoAGogA0EcaigCADYCACAAIAMpAhQ3AmAgAygCDEEANgIAIAMoAhAiACAAKAIAQQFrIgA2AgAgAEUEQCADQRBqEPgBCyADQSBqJAALpQEBAX8jAEEgayIDJAAgA0EIaiAAEMMBIAMoAgghAAJAIAEEQCADIAEgAhDjASADQRRqIAMoAgAgAygCBBDEAgwBCyADQYCAgIB4NgIUCyAAQewAahC+AiAAQfQAaiADQRxqKAIANgIAIAAgAykCFDcCbCADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQ+AELIANBIGokAAuJAQEBfyABQQBOBEACfyACKAIEBEAgAigCCCIDBEAgAigCACADQQEgARDIAgwCCwtBASABRQ0AGkHR/8AALQAAGiABQQEQ2QILIgJFBEAgACABNgIIIABBATYCBCAAQQE2AgAPCyAAIAE2AgggACACNgIEIABBADYCAA8LIABBADYCBCAAQQE2AgALiAEBAX8gAAJ/An8CQCACQQBOBEAgAygCBARAIAMoAggiBARAIAMoAgAgBCABIAIQyAIMBAsLIAJFDQFB0f/AAC0AABogAiABENkCDAILIABBADYCBEEBDAILIAELIgNFBEAgACACNgIIIAAgATYCBEEBDAELIAAgAjYCCCAAIAM2AgRBAAs2AgALogEBAX8jAEEQayICJAACQCABKAIAJQEQKgRAIAJBBGogARCvASAAQQhqIAJBDGooAgA2AgAgACACKQIENwIADAELIAEoAgAlARAhBEAgAiABKAIAEMcCIgE2AgAgAkEEaiACEK8BIABBCGogAkEMaigCADYCACAAIAIpAgQ3AgAgAUGEAUkNASABEPQBDAELIABBgICAgHg2AgALIAJBEGokAAuQAQEBfyMAQSBrIgMkACADQQhqIAAQwwEgAygCCCEAIAMgASACEOMBIANBFGogAygCACADKAIEEMQCIABB6AFqEP8CIABB8AFqIANBHGooAgA2AgAgACADKQIUNwLoASADKAIMQQA2AgAgAygCECIAIAAoAgBBAWsiADYCACAARQRAIANBEGoQZwsgA0EgaiQAC5ABAQF/IwBBIGsiAyQAIANBCGogABDDASADKAIIIQAgAyABIAIQ4wEgA0EUaiADKAIAIAMoAgQQxAIgAEH4AGoQ/wIgAEGAAWogA0EcaigCADYCACAAIAMpAhQ3AnggAygCDEEANgIAIAMoAhAiACAAKAIAQQFrIgA2AgAgAEUEQCADQRBqEPgBCyADQSBqJAALkQEBAX8jAEEgayIDJAAgA0EIaiAAEMMBIAMoAgghACADIAEgAhDjASADQRRqIAMoAgAgAygCBBDEAiAAQYQBahD/AiAAQYwBaiADQRxqKAIANgIAIAAgAykCFDcChAEgAygCDEEANgIAIAMoAhAiACAAKAIAQQFrIgA2AgAgAEUEQCADQRBqEPgBCyADQSBqJAALkQEBAX8jAEEgayIDJAAgA0EIaiAAEMMBIAMoAgghACADIAEgAhDjASADQRRqIAMoAgAgAygCBBDEAiAAQZABahD/AiAAQZgBaiADQRxqKAIANgIAIAAgAykCFDcCkAEgAygCDEEANgIAIAMoAhAiACAAKAIAQQFrIgA2AgAgAEUEQCADQRBqEPgBCyADQSBqJAALkQEBAX8jAEEgayIDJAAgA0EIaiAAEMMBIAMoAgghACADIAEgAhDjASADQRRqIAMoAgAgAygCBBDEAiAAQZwBahD/AiAAQaQBaiADQRxqKAIANgIAIAAgAykCFDcCnAEgAygCDEEANgIAIAMoAhAiACAAKAIAQQFrIgA2AgAgAEUEQCADQRBqEPgBCyADQSBqJAALlAEBA38jAEEQayICJAACf0EBIAEoAgAiA0EnIAEoAgQiBCgCECIBEQAADQAaIAJBBGogACgCAEGBAhA+AkAgAi0ABEGAAUYEQCADIAIoAgggAREAAEUNAUEBDAILIAMgAi0ADiIAIAJBBGpqIAItAA8gAGsgBCgCDBEDAEUNAEEBDAELIANBJyABEQAACyACQRBqJAALogEBAn8jAEEgayICJAACQAJAIAEoAgBFDQAgAkEIaiABEN4BIAIoAghBAXFFDQAgAigCDCEDIAEgASgCDEEBajYCDCACQRRqIAMQhQEgAigCFEGAgICAeEYEQCAAIAIoAhg2AgQgAEGBgICAeDYCAAwCCyAAIAIpAhQ3AgAgAEEIaiACQRxqKAIANgIADAELIABBgICAgHg2AgALIAJBIGokAAuNAwEHfyMAQRBrIgckAAJAAkAgAkEHTQRAIAINAQwCCyAHQQhqIQYCQAJAAkACQCABQQNqQXxxIgMgAUYNACACIAMgAWsiAyACIANJGyIDRQ0AQQEhBQNAIAEgBGotAABBLkYNBCADIARBAWoiBEcNAAsgAyACQQhrIgVLDQIMAQsgAkEIayEFQQAhAwtBrty48QIhBANAQYCChAggASADaiIIKAIAQa7cuPECcyIJayAJckGAgoQIIAhBBGooAgBBrty48QJzIghrIAhycUGAgYKEeHFBgIGChHhHDQEgA0EIaiIDIAVNDQALCyACIANHBEBBLiEEQQEhBQNAIAEgA2otAABBLkYE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IANBCGogACABIAJBAUEBEGQgAygCCCIAQYGAgIB4RwRAIAAgAygCDEGQs8AAEL0CAAsgA0EQaiQAC08BAn9B0f/AAC0AABogASgCBCECIAEoAgAhA0EIQQQQ2QIiAUUEQEEEQQgQjAMACyABIAI2AgQgASADNgIAIABBmMXAADYCBCAAIAE2AgALOwEBfyMAQRBrIgIkACACIAEoAgAlARAuIAAgAigCAAR+IAAgAikDCDcDCEIBBUIACzcDACACQRBqJAALRQEBfyMAQSBrIgMkACADIAI2AhwgAyABNgIYIAMgAjYCFCADQQhqIANBFGpBmLnAABCxASAAIAMpAwg3AwAgA0EgaiQAC/11AyN/Gn4BfCABKAIIIgNBgICAAXEhAiAAKwMAIT8CQCADQYCAgIABcUUEQCABIAJBAEchAUEAIQAjAEGAAWsiBSQAID+9ISYCf0EDID+ZRAAAAAAAAPB/YQ0AGkECICZCgICAgICAgPj/AIMiJUKAgICAgICA+P8AUQ0AGiAmQv////////8HgyIpQoCAgICAgIAIhCAmQgGGQv7///////8PgyAmQjSIp0H/D3EiABsiJ0IBgyEoICVQBEBBBCApUA0BGiAAQbMIayEAQgEhJSAoUAwBC0KAgICAgICAICAnQgGGICdCgICAgICAgAhRIgIbISdCAkIBIAIbISVBy3dBzHcgAhsgAGohACAoUAshAiAFIAA7AXggBSAlNwNwIAVCATcDaCAFICc3A2AgBSACOgB6An8CQAJAAkAgAkECayICBEBBASEAQfvnwABB/OfAACAmQgBTIgMbQfvnwABBASADGyABGyEXICZCP4inIAFyIR5BAyACIAJBA08bQQJrDgIDAgELIAVBAzYCKCAFQf3nwAA2AiQgBUECOwEgQQEhF0EBIQAgBUEgagwDCyAFQQM2AiggBUGA6MAANgIkIAVBAjsBICAFQSBqDAILIAVBIGohBiAFQQ9qIQkjAEEwayIDJAACQAJAAn8CQAJAAkACQAJAAkACQAJAIAVB4ABqIgApAwAiJVBFBEAgACkDCCInUA0BIAApAxAiJlANAiAlICZ8IiYgJVQNAyAlICdUDQQgJkKAgICAgICAgCBaDQUgAyAALwEYIgA7AQggAyAlICd9Iic3AwAgACAAQSBrIAAgJkKAgICAEFQiARsiAkEQ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IAMoApgBIAIoApQBIAIoApgBELICIR8gAygCVEGAgICAeEYiCSACKAJUIhBBgICAgHhGcSEFIAkgEEGAgICAeEZyRQRAIAMoAlggAygCXCACKAJYIAIoAlwQsgIhBQsgAygCYEGAgICAeEYiECACKAJgQYCAgIB4RiIccSEJIBAgHHJFBEAgAygCZCADKAJoIAIoAmQgAigCaBCyAiEJCwJAAkACQCADKAIgIAIoAiBGBEAgAygCbEGAgICAeEYiECACKAJsQYCAgIB4RiIccSEgIAsgE3EgDXEgCHEgDiASRnEgCnEgGyAeRnEgECAccgR/ICAFIAMoAnAgAygCdCACKAJwIAIoAnQQsgILIA8gFkYgFCAVRnEgFyAYRnEgGSAaRnEgH3EgBXEgCXFxcUUNAQJAIAMoAowBIghFDQAgAigCjAEiBUUNACADKAKIASAIIAIoAogBIAUQsgJFDQMLIAMoAkhBgICAgHhGIAIoAkhBgICAgHhGciIJRQ0DDAcLIAMoAmxBgICAgHhGDQAgAigCbEGAgICAeEYNACADKAJwIAMoAnQgAigCcCACKAJ0ELICGgsgB0EBOwEADAYLIARCfRC7AiAELQAAQQZGDQIgByAEKQMANwMAIAdBEGogBEEQaikDADcDACAHQQhqIARBCGopAwA3AwAMBQsgAygCTCADKAJQIAIoAkwgAigCUBCyAg0DIARCfhC7AiAELQAAQQZGDQIgByAEKQMANwMAIAdBEGogBEEQaikDADcDACAHQQhqIARBCGopAwA3AwAMBAsgB0GBAjsBAAwDCyAEIAQoAgQ2AhxB2IfAAEErIARBHGpByIfAAEGQiMAAEKkBAAsgBCAEKAIENgIcQdiHwABBKyAEQRxqQciHwABBoIjAABCpAQALAkACQAJAAkACQCADKAKkASIKRQ0AIAIoAqQBIgVFDQAgAygCoAEgCiACKAKgASAFELICRQ0BC0EAIQUCf0EAIAhFDQAaQQAgAigCjAEiC0UNABogAygCiAEgCCACKAKIASALELICQQFzCyELIAkNAyADKAJQIggNAQwDCyAEQn8QuwIgBC0AAEEGRg0BIAcgBCkDADcDACAHQRBqIARBEGopAwA3AwAgB0EIaiAEQQhqKQMANwMADAMLIAIoAlAiCUUNASADKAJMIAggAigCTCAJELICQQFzIQUMAQsgBCAEKAIENgIcQdiHwABBKyAEQRxqQciHwABBsIjAABCpAQALQQAhCAJAIApFDQAgAigCpAEiCUUNACADKAKgASAKIAIoAqABIAkQsgJBAXMhCAsgBSALciAIcgRAIARCARC7AiAELQAAQQZGDQIgByAEKQMANwMAIAdBEGogBEEQaikDADcDACAHQQhqIARBCGopAwA3AwAMAQsgBEICELsCIAQtAABBBkYNAiAHIAQpAwA3AwAgB0EQaiAEQRBqKQMANwMAIAdBCGogBEEIaikDADcDAAsgBEEgaiQADAILIAQgBCgCBDYCHEHYh8AAQSsgBEEcakHIh8AAQcCIwAAQqQEACyAEIAQoAgQ2AhxB2IfAAEErIARBHGpByIfAAEHQiMAAEKkBAAsgBkEIaiEFQQAhBCMAQYABayICJAAgAkEANgI8QYABIQMCQAJAAkACQAJAAkAgBy0AAEEBaw4FAAECAwQFCyACIAJBPGogBy0AARDCAiACKAIEIQMgAigCACEEDAQLIAJBCGogB0EIaiACQTxqEKMBIAIoAgwhAyACKAIIIQQMAwsgAkEQaiACQTxqIAcoAgggBygCDBDJAiACKAIUIQMgAigCECEEDAILIAJBGGogAkE8aiAHQQRqEF8gAigCHCEDIAIoAhghBAwBCyACQdgAaiACQTxqIAcoAgwiAxDzAQJAAkAgAigCWEECRgRAIAIoAlwhAwwBCyACQdAAaiACQegAaigCADYCACACQcgAaiACQeAAaikCADcDACACIAIpAlg3A0AgBygCCCEEIAIgA0EAIAcoAgQiAxs2AnggAiAENgJ0IAIgAzYCcCACQQA2AmwgAiADQQBHIgc2AmggAiAENgJkIAIgAzYCYCACQQA2AlwgAiAHNgJYA0AgAkEwaiACQdgAahByIAIoAjAiA0UNAiACQShqIAJBQGsgAyACKAI0EL0BIAIoAihBAXFFDQALIAIoAiwhAyACKAJEIgRBhAFPBEAgBBD0AQsgAigCSEUNACACKAJMIgRBhAFJDQAgBBD0AQtBASEEDAELIAJB6ABqIAJB0ABqKAIANgIAIAJB4ABqIAJByABqKQMANwMAIAIgAikDQDcDWCACQSBqIAJB2ABqEIICIAIoAiQhAyACKAIgIQQLIAUgAzYCBCAFIAQ2AgAgAkGAAWokACAG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yANqIAFBOGooAgA2AgAgASABKQIwNwPAAwsCQCABKAI8QYGAgIB4RyIlBEAgAUHYA2ogAUHEAGooAgA2AgAgASABKQI8NwPQAwwBCyABQYCAgIB4NgLQAwsCQCAspyImBEAgAUH4A2ogDkEYaikDADcDACABQfADaiAOQRBqKQMANwMAIAFB6ANqIA5BCGopAwA3AwAgASAOKQMANwPgAwwBCyABQQA2AuADCwJ/IAEoAnBBgICAgHhHIidFBEBB14PAAEEHEMEBIQIgBEECNgIAIAQgAjYCBEEAIQVBACELQQAhCkEAIQNBACEGQQAhCUEAIQJBAAwBCyABQYgEaiABQfgAaigCADYCACABIAEpAnA3A4AEAkAgASgCfCIIQYGAgIB4RwRAIAFBmARqIAFBhAFqKAIANgIAIAEgASkCfDcDkAQMAQsgAUGAgICAeDYCkAQLAkAgLaciAgRAIAFBuARqIA9BGGopAwA3AwAgAUGwBGogD0EQaikDADcDACABQagEaiAPQQhqKQMANwMAIAEgDykDADcDoAQMAQsgAUEANgKgBAsCQCAupyIJBEAgAUHYBGogEEEYaikDADcDACABQdAEaiAQQRBqKQMANwMAIAFByARqIBBBCGopAwA3AwAgASAQKQMANwPABAwBCyABQQA2AsAECwJAIAEoAtwBQYGAgIB4RyIGBEAgAUHoBGogAUHkAWooAgA2AgAgASABKQLcATcD4AQMAQsgAUGAgICAeDYC4AQLAkAgK6ciKARAIAFBiAVqIBFBGGopAwA3AwAgAUGABWogEUEQaikDADcDACABQfgEaiARQQhqKQMANwMAIAEgESkDADcD8AQMAQsgAUEANgLwBAsCQCABKAKUAkGAgICAeEciKUUEQEGyhMAAQRwQwQEhAyAEQQI2AgAgBCADNgIEQQAhBUEAIQtBACEKQQAhAwwBCyABQZgFaiABQZwCaigCADYCACABIAEpApQCNwOQBQJAIAEoAqACQYCAgIB4RyIDRQRAQc6EwABBHxDBASEFIARBAjYCACAEIAU2AgRBACEFQQAhC0EAIQoMAQsgAUGoBWogAUGoAmooAgA2AgAgASABKQKgAjcDoAUgASALQQAgChsiBzYCrAUCQCABKAK0AkGBgICAeEciC0UEQCABQYCAgIB4NgKwBQwBCyABQbgFaiABQbwCaigCADYCACABIAEpArQCNwOwBQsCQCAvpyIFBEAgAUGwA2ogEkEYaikDADcDACABQagDaiASQRBqKQMANwMAIAFBoANqIBJBCGopAwA3AwAgASASKQMANwOYAwwBCyABQQA2ApgDCwJAIAEoAuwCIhdBgYCAgHhHBEAgAUHIBWogAUH0AmooAgA2AgAgASABKQLsAjcDwAUMAQsgAUGAgICAeDYCwAULIBRFBEBBhIXAAEEMEMEBIQcgBEECNgIAIAQgBzYCBCABQcAFahC+AiABKAKYAwRAIAFBmANqENEBCyAXQYGAgIB4RyEXIAFBsAVqEL4CIAFBrAVqEJsBIAFBoAVqIgQQhgIgBBCCAwwBCyAEIAEpA8ADNwKMAiAEIAEpA9ADNwKYAiAEIAEpA+ADNwNIIAQgFSkDADcDMCAEQThqIBVBCGopAwA3AwAgBEFAayAVQRBqKQMANwMAIARBlAJqIAFByANqKAIANgIAIARBoAJqIAFB2ANqKAIANgIAIARB0ABqIAFB6ANqKQMANwMAIARB2ABqIAFB8ANqKQMANwMAIARB4ABqIAFB+ANqKQMANwMAIAEoAvwCIQIgBEHwAWogAUH4AGooAgA2AgAgBCABKQJwNwLoASAEQawCaiABQZgEaigCADYCACAEIAEpA5AENwKkAiAEQYABaiABQbgEaikDADcDACAEQfgAaiABQbAEaikDADcDACAEQfAAaiABQagEaikDADcDACAEIAEpA6AENwNoIARBoAFqIAFB2ARqKQMANwMAIARBmAFqIAFB0ARqKQMANwMAIARBkAFqIAFByARqKQMANwMAIAQgASkDwAQ3A4gBIARBuAJqIAFB6ARqKAIANgIAIAQgASkD4AQ3ArACIARBwAFqIAFBiAVqKQMANwMAIARBuAFqIAFBgAVqKQMANwMAIARBsAFqIAFB+ARqKQMANwMAIAQgASkD8AQ3A6gBIARB/AFqIAFBnAJqKAIANgIAIAQgASkClAI3AvQBIARBiAJqIAFBqAJqKAIANgIAIAQgASkCoAI3AoACIARBxAJqIAFBuAVqKAIANgIAIAQgASkDsAU3ArwCIARB4AFqIAFBsANqKQMANwMAIARB2AFqIAFBqANqKQMANwMAIARB0AFqIAFBoANqKQMANwMAIAQgASkDmAM3A8gBIARB0AJqIAFByAVqKAIANgIAIAQgASkDwAU3AsgCIARBAiAcIBxBA0YbOgDZAiAEQQIgGCAYQQNGGzoA2AIgBCAHNgLUAiAEIAI2AiwgBCAUNgIoIAQgHzYCJCAEIB5BACAeQQJHGzYCICAEICA2AhwgBCAdQQAgHUECRxs2AhggBCAhNgIUIAQgG0EAIBtBAkcbNgIQIAQgIjYCDCAEIBpBACAaQQJHGzYCCCAEICM2AgQgBCAZQQAgGUECRxs2AgAMIQsgAUGQBWoiBBD7ASAEEIUDCyABKALwBARAIAFB8ARqENMBCyABQeAEahC+AiABKALABARAIAFBwARqENEBCyABKAKgBARAIAFBoARqENIBCyABQZAEahC+AiABQYAEahD/AiAIQYGAgIB4RwshBCABKALgAwRAIAFB4ANqENEBCyABQdADahC+AiABQcADahC+AgwbCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQNSABKAKcAyECIAEoApgDIhRFBEAgBEECNgIAIAQgAjYCBEEAIQRBACECQQAhCUEAIQZBACEDQQAhCkEAIQtBACEFDB4LIBUgCSkDADcDACAVQRBqIAlBEGopAwA3AwAgFUEIaiAJQQhqKQMANwMAIAEgAjYC/AIgASAUNgL4AgwXCwwfCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQpgEgASgCnAMhAiABKAKYAyIDQYGAgIB4RgRAIARBAjYCACAEIAI2AgQMGgsgASgCoAMhBSABKALsAkGBgICAeEcEQCABQewCahC+AgsgASAFNgL0AiABIAI2AvACIAEgAzYC7AIMFgsMHgsgASgCHCABQQA2AhwEQCABQZgDaiABKAIgEIQBIAEoApgDBEAgASgCnAMhAiAEQQI2AgAgBCACNgIEDBkLIBIgCSkDADcDACASQRhqIAlBGGopAwA3AwAgEkEQaiAJQRBqKQMANwMAIBJBCGogCUEIaikDADcDAEIBIS8gAUIBNwPAAgwVCwwdCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQpgEgASgCnAMhAiABKAKYAyIDQYGAgIB4RgRAIARBAjYCACAEIAI2AgQMGAsgASgCoAMhBSABKAK0AkGBgICAeEcEQCABQbQCahC+AgsgASAFNgK8AiABIAI2ArgCIAEgAzYCtAIMFAsMHAsgASgCHCABQQA2AhwEQCABKAIgIQJBACEKIwBBEGsiAyQAIAMgAjYCDCABQQhqIgsCfyADQQxqENsCRQRAIwBBsAFrIgUkACAFIAIQMwJAAkACfyAFKAKcAUGAgICAeEYEQCAFKAIAIQJBAQwBC0HR/8AALQAAGkGwAUEIENkCIgJFDQEgAiAFQbAB/AoAAEEACyEKIAMgAjYCBCADIAo2AgAgBUGwAWokAAwBC0EIQbABEIwDAAsgAygCACEKIAMoAgQMAQtBACACQYQBSQ0AGiACEPQBQQALNgIEIAsgCjYCACADQRBqJABBASEKIAEoAgwhCyABKAIIQQFxBEAgBEECNgIAIAQgCzYCBAwXCyABIAs2ArACIAFBATYCrAIMEwsMGwsgASgCHCABQQA2AhwEQCABQZgDaiEFIAEoAiAhAiMAQSBrIgYkACAGIAI2AgwCQCAGQQxqIgIQ7AIEQCAGQRBqIgggAhC6AiAGQQA2AhxBACEDIwBBQGoiAiQAIAgoAgAEQCAIKAIIIgMgCCgCBGsiB0EAIAMgB08bIQMLIAJBEGpBqtUCIAMgA0Gq1QJPG0EIQRgQgwEgAigCFCEDAkAgAigCEEEBRwRAIAJBADYCDCACIAIoAhg2AgggAiADNgIEIAJBKGogCBCHAQJAAkAgAi0AKEEHRwRAA0AgAkEgaiACQThqIgwpAwA3AwAgAkEYaiACQTBqIhYpAwA3AwAgAiACKQMoIio3AxAgKqdB/wFxQQZGDQIgAigCDCIDIAIoAgRGBEAgAkEEakG4l8AAENoBCyACKAIIIANBGGxqIgcgAikDKDcDACAHQQhqIBYpAwA3AwAgB0EQaiAMKQMANwMAIAIgA0EBajYCDCACQShqIAgQhwEgAi0AKEEHRw0ACwsgBSACKAIsNgIEIAVBgICAgHg2AgAgAkEEaiIDEIYCIAMQggMMAQsgAkEQahDqASAFIAIpAgQ3AgAgBUEIaiACQQxqKAIANgIACyACQUBrJAAMAQsgAyACKAIYQaiXwAAQvQIACwwBCyAGQRBqIAZBDGoQcCAGKAIQIQICQAJAAkAgBi0AFCIIQQJrDgIBAAILIAVBgICAgHg2AgAgBSACNgIEDAILIAZBDGogBkEQakG4gsAAEEohAiAFQYCAgIB4NgIAIAUgAjYCBAwBCyMAQdAAayIDJAAgAyAIQQFxOgAQIAMgAjYCDCADQSBqQQBBCEEYEIMBIAMoAiQhAgJAIAMoAiBBAUcEQCADQQA2AhwgAyADKAIoNgIYIAMgAjYCFCADQThqIANBDGoQfwJAAkACQCADLQA4QQdHBEADQCADQTBqIANByABqIgcpAwA3AwAgA0EoaiADQUBrIgwpAwA3AwAgAyADKQM4Iio3AyAgKqdB/wFxQQZGDQIgAygCHCICIAMoAhRGBEAgA0EUakG4l8AAENoBCyADKAIYIAJBGGxqIgggAykDODcDACAIQQhqIAwpAwA3AwAgCEEQaiAHKQMANwMAIAMgAkEBajYCHCADQThqIANBDGoQfyADLQA4QQdHDQALCyAFIAMoAjw2AgQgBUGAgICAeDYCACADQRRqIgIQhgIgAhCCAyADKAIMIgVBgwFLDQEMAgsgA0EgahDqASAFIAMpAhQ3AgAgBUEIaiADQRxqKAIANgIAIAMoAgwiBUGEAUkNAQsgBRD0AQsgA0HQAGokAAwBCwweCwsgBigCDCICQYMBSwRAIAIQ9AELIAZBIGokACABKAKcAyECIAEoApgDIgNBgICAgHhGBEAgBEECNgIAIAQgAjYCBAwWCyABKAKgAyEFIAEoAqACQYCAgIB4RwRAIAFBoAJqIgYQhgIgBhCCAwsgASAFNgKoAiABIAI2AqQCIAEgAzYCoAIMEgsMGgsgASgCHCABQQA2AhwEQCABQZgDaiEFIAEoAiAhAiMAQSBrIgYkACAGIAI2AgwCQCAGQQxqIgIQ7AIEQCAGQRBqIgggAhC6AiAGQQA2AhxBACEDIwBBMGsiAiQAIAgoAgAEQCAIKAIIIgMgCCgCBGsiB0EAIAMgB08bIQMLIAJBIGpB1aoFIAMgA0HVqgVPG0EEQQwQgwEgAigCJCEDAkAgAigCIEEBRwRAIAJBADYCECACIAIoAig2AgwgAiADNgIIIAJBFGogCBCZAQJAAkAgAigCFEGBgICAeEcEQANAIAJBKGogAkEcaiIHKAIANgIAIAIgAikCFCIqNwMgICqnQYCAgIB4Rg0CIAIoAhAiAyACKAIIRgRAIAJBCGoQ2wELIAIoAgwgA0EMbGoiDCACKQIUNwIAIAxBCGogBygCADYCACACIANBAWo2AhAgAkEUaiAIEJkBIAIoAhRBgYCAgHhHDQALCyAFIAIoAhg2AgQgBUGAgICAeDYCACACQQhqIgMQ+wEgAxCFAwwBCyACQSBqEL4CIAUgAikCCDcCACAFQQhqIAJBEGooAgA2AgALIAJBMGokAAwBCyADIAIoAihBqJfAABC9AgALDAELIAZBEGogBkEMahBwIAYoAhAhAgJAAkACQCAGLQAUIghBAmsOAgEAAgsgBUGAgICAeDYCACAFIAI2AgQMAgsgBkEMaiAGQRBqQZiBwAAQSiECIAVBgICAgHg2AgAgBSACNgIEDAELIwBBMGsiAyQAIAMgCEEBcToABCADIAI2AgAgA0EgakEAQQRBDBCDASADKAIkIQICQCADKAIgQQFHBEAgA0EANgIQIAMgAygCKDYCDCADIAI2AgggA0EUaiADEJwBAkACQAJAIAMoAhRBgYCAgHhHBEADQCADQShqIANBHGoiCCgCADYCACADIAMpAhQiKjcDICAqp0GAgICAeEYNAiADKAIQIgIgAygCCEYEQCADQQhqENsBCyADKAIMIAJBDGxqIgcgAykCFDcCACAHQQhqIAgoAgA2AgAgAyACQQFqNgIQIANBFGogAxCcASADKAIUQYGAgIB4Rw0ACwsgBSADKAIYNgIEIAVBgICAgHg2AgAgA0EIaiICEPsBIAIQhQMgAygCACIFQYMBSw0BDAILIANBIGoQvgIgBSADKQIINwIAIAVBCGogA0EQaigCADYCACADKAIAIgVBhAFJDQELIAUQ9AELIANBMGokAAwBCwwdCwsgBigCDCICQYMBSwRAIAIQ9AELIAZBIGokACABKAKcAyECIAEoApgDIgNBgICAgHhGBEAgBEECNgIAIAQgAjYCBAwVCyABKAKgAyEFIAEoApQCQYCAgIB4RwRAIAFBlAJqIgYQ+wEgBhCFAwsgASAFNgKcAiABIAI2ApgCIAEgAzYClAIMEQsMGQsgASgCHCABQQA2AhwEQCABQZgDaiEIIAEoAiAhAyMAQTBrIgYkACAGIAM2AgwCQCAGQQxqENsCRQRAIAZBEGohBUIAISpBACEMIwBBwAFrIgIkACACIAM2AgwgAkHIAGogAkEMahBwIAIoAkghAwJAAkACQAJAAkACQAJAAkAgAi0ATCIHQQJrDgICAAELIAVBADYCACAFIAM2AgQgAigCDCIDQYMBSw0DDAQLIAIgBzoAdCACIAM2AnAgAkEANgJoIAICfkHYg8EAKAIAQQFGBEBB6IPBACkDACErQeCDwQApAwAMAQsgAkHIAGoQzQFB2IPBAEIBNwMAQeiDwQAgAikDUCIrNwMAIAIpA0gLIio3A1hB4IPBACAqQgF8NwMAIAIgKzcDYCACQgA3A1AgAkEANgJMIAJBmIPAADYCSCACQYwBaiEDAkADQAJAIAJBqAFqIAJB6ABqEHkCQAJAAkACQCACKAKoASIWQYCAgIB4aw4CBAABCyACKAKsASEDDAELIAIgAikCrAEiKzcCuAEgAiAWNgK0ASACKAJoIAJBADYCaEUNCSACQRBqIAIoAmwQpgEgAigCEEGBgICAeEcNASACKAIUIQMgAkG0AWoQ/wILIAVBADYCACAFIAM2AgQgAkHIAGoQ0wEMAwsgAkEwaiACQRhqKAIAIgw2AgAgAkGIAWogK0IgiKciBzYCACACQUBrIAc2AgAgAkGgAWogDDYCACACIAIpAhAiKjcDKCADQQhqIAw2AgAgAyAqNwIAIAIgFjYCgAEgAiArpyIMNgKEASACICo3A5gBIAIgAikCgAE3AzggAkEQaiIWIAJByABqIAJBOGogAkGYAWoQRSACKAIQQYGAgIB4Rg0BIBYQvgIMAQsLIAIgBzYCiAEgAiAMNgKEASACQYCAgIB4NgKAASACQYABahCnAiAFQRhqIAJB4ABqKQMANwMAIAVBEGogAkHYAGopAwA3AwAgBUEIaiACQdAAaikDADcDACAFIAIpA0g3AwALIAJB6ABqEP8BDAELIAIgAkEMahD8ASACKAIAQQFxBEAgAiACKAIENgI4IAJBGGogAkE4ahC6AiACQQA2AiQgAkEANgIQQQAhAyACKAIYBEAgAigCICIDIAIoAhxrIgdBACADIAdPGyEDC0EAIQdB4IPBAAJ+QdiDwQAoAgBBAUYEQEHog8EAKQMAIStB4IPBACkDAAwBCyACQYABahDNAUHYg8EAQgE3AwBB6IPBACACKQOIASIrNwMAIAIpA4ABCyIwQgF8NwMAAkAgA0UEQEGYg8AAIQMMAQsgAkGAAWogAkG0AWpBGCADQQdNBH9BBEEIIANBBEkbBUF/QarVAiADIANBqtUCTxtBA3RBB25BAWtndkEBagsQdCACKAKEASEHIAIoAoABIgNFBEAgAjUCiAEhKkEAIQMMAQsgAikCiAEhKiAHQQlqIgxFDQAgA0H/ASAM/AsACyACICs3A2AgAiAwNwNYIAIgKjcDUCACIAc2AkwgAiADNgJIIAJBgAFqIAJBEGoQaQJAAkACQCACKAKAAUGBgICAeEcEQCACQYwBaiEDA0AgAkH4AGogAkGQAWopAgA3AwAgAkHwAGogAkGIAWoiBykCADcDACACIAIpAoABIis3A2ggK6dBgICAgHhGDQIgAkGgAWogBygCADYCACACQbABaiADQQhqKAIANgIAIAIgAikCgAE3A5gBIAIgAykCADcDqAEgAkG0AWoiByACQcgAaiACQZgBaiACQagBahBFIAIoArQBQYGAgIB4RwRAIAcQvgILIAJBgAFqIAJBEGoQaSACKAKAAUGBgICAeEcNAAsLIAUgAigChAE2AgQgBUEANgIAIAJByABqENMBIAIoAhBFDQIgAigCFCIDQYMBSw0BDAILIAJB6ABqEKcCIAVBGGogAkHgAGopAwA3AwAgBUEQaiACQdgAaikDADcDACAFQQhqIAJB0ABqKQMANwMAIAUgAikDSDcDACACKAIQRQ0BIAIoAhQiA0GEAUkNAQsgAxD0AQsgAigCOCIDQYQBSQ0BIAMQ9AEMAQsgAkEMaiACQbQBakGogcAAEEohAyAFQQA2AgAgBSADNgIECyACKAIMIgNBhAFJDQELIAMQ9AELIAJBwAFqJAAMAQtBuJLAAEExEIQDAAsgCAJ/IAYoAhBFBEAgCCAGKAIUNgIEQQEMAQsgCCAGKQMQNwMIIAhBIGogBkEoaikDADcDACAIQRhqIAZBIGopAwA3AwAgCEEQaiAGQRhqKQMANwMAQQALNgIADAELIAhBADYCACAIQQA2AgggA0GEAUkNACADEPQBCyAGQTBqJAAgASgCmAMEQCABKAKcAyECIARBAjYCACAEIAI2AgQMFAsgESAJKQMANwMAIBFBGGogCUEYaikDADcDACARQRBqIAlBEGopAwA3AwAgEUEIaiAJQQhqKQMANwMAQgEhKyABQgE3A+gBDBALDBgLIAEoAhwgAUEANgIcBEAgAUGYA2ogASgCIBCmASABKAKcAyECIAEoApgDIgNBgYCAgHhGBEAgBEECNgIAIAQgAjYCBAwTCyABKAKgAyEFIAEoAtwBQYGAgIB4RwRAIAFB3AFqEL4CCyABIAU2AuQBIAEgAjYC4AEgASADNgLcAQwPCwwXCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQtgEgASgCnAMhHyABKAKYAyIeQQJHDQ4gBEECNgIAIAQgHzYCBAwRCwwWCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQtgEgASgCnAMhICABKAKYAyIdQQJHDQ0gBEECNgIAIAQgIDYCBAwQCwwVCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQhAEgASgCmAMEQCABKAKcAyECIARBAjYCACAEIAI2AgQMEAsgECAJKQMANwMAIBBBGGogCUEYaikDADcDACAQQRBqIAlBEGopAwA3AwAgEEEIaiAJQQhqKQMANwMAQgEhLiABQgE3A7ABDAwLDBQLIAEoAhwgAUEANgIcBEAgAUGYA2ohAyABKAIgIQUjAEEwayICJAAgAiAFNgIMAkAgAkEMahDbAkUEQCACQRBqIAUQNSADAn8gAigCEEUEQCADIAIoAhQ2AgRBAQwBCyADIAIpAxA3AwggA0EgaiACQShqKQMANwMAIANBGGogAkEgaikDADcDACADQRBqIAJBGGopAwA3AwBBAAs2AgAMAQsgA0EANgIAIANBADYCCCAFQYQBSQ0AIAUQ9AELIAJBMGokACABKAKYAwRAIAEoApwDIQIgBEECNgIAIAQgAjYCBAwPCyAPIAkpAwA3AwAgD0EYaiAJQRhqKQMANwMAIA9BEGogCUEQaikDADcDACAPQQhqIAlBCGopAwA3AwBCASEtIAFCATcDiAEMCwsMEwsgASgCHCABQQA2AhwEQCABQZgDaiABKAIgEK0BIAEtAJgDBEAgASgCnAMhAiAEQQI2AgAgBCACNgIEDA4LIAEtAJkDIRwMCgsMEgsgASgCHCABQQA2AhwEQCABQZgDaiABKAIgEK0BIAEtAJgDBEAgASgCnAMhAiAEQQI2AgAgBCACNgIEDA0LIAEtAJkDIRgMCQsMEQsgASgCHCABQQA2AhwEQCABQZgDaiABKAIgEKYBIAEoApwDIQIgASgCmAMiA0GBgICAeEYEQCAEQQI2AgAgBCACNgIEDAwLIAEoAqADIQUgASgCfEGBgICAeEcEQCABQfwAahC+AgsgASAFNgKEASABIAI2AoABIAEgAzYCfAwICwwQCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQhQEgASgCnAMhAiABKAKYAyIDQYCAgIB4RgRAIARBAjYCACAEIAI2AgQMCwsgASgCoAMhBSABQfAAahC+AiABIAU2AnggASACNgJ0IAEgAzYCcAwHCwwPCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQtgEgASgCnAMhISABKAKYAyIbQQJHDQYgBEECNgIAIAQgITYCBAwJCwwOCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQhAEgASgCmAMEQCABKAKcAyECIARBAjYCACAEIAI2AgQMCQsgDiAJKQMANwMAIA5BGGogCUEYaikDADcDACAOQRBqIAlBEGopAwA3AwAgDkEIaiAJQQhqKQMANwMAQgEhLCABQgE3A0gMBQsMDQsgASgCHCABQQA2AhwEQCABQZgDaiABKAIgEKYBIAEoApwDIQIgASgCmAMiA0GBgICAeEYEQCAEQQI2AgAgBCACNgIEDAgLIAEoAqADIQUgASgCPEGBgICAeEcEQCABQTxqEL4CCyABIAU2AkQgASACNgJAIAEgAzYCPAwECwwMCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQtgEgASgCnAMhIiABKAKYAyIaQQJHDQMgBEECNgIAIAQgIjYCBAwGCwwLCyABKAIcIAFBADYCHARAIAFBmANqIAEoAiAQtgEgASgCnAMhIyABKAKYAyIZQQJHDQIgBEECNgIAIAQgIzYCBAwFCwwKCyABKAIcIAFBADYCHEUNCSABQZgDaiABKAIgEKYBIAEoApwDIQIgASgCmAMiA0GBgICAeEYEQCAEQQI2AgAgBCACNgIEDAQLIAEoAqADIQUgASgCMEGBgICAeEcEQCABQTBqEL4CCyABIAU2AjggASACNgI0IAEgAzYCMAsgAUGYA2ogAUEcahBgIAEtAJgDRQ0ACwsgASgCnAMhAiAEQQI2AgAgBCACNgIEC0EAIQVBACELQQAhCkEAIQNBACEGQQAhCUEAIQJBACEECyAURQ0BCyABQfgCahDSAQsCQCAXDQAgASgC7AJBgYCAgHhGDQAgAUHsAmoQvgILAkAgBSABKALAAkF/c3JBAXENACABKALIAkUNACASENEBCyALIAEoArQCQYGAgIB4RnJFBEAgAUG0AmoQvgILIAEoAqwCQX9zIApyQQFxRQRAIAFBsAJqEJsBCyADIAEoAqACQYCAgIB4RnJFBEAgAUGgAmoiAxCGAiADEIIDCyApIAEoApQCQYCAgIB4RnJFBEAgAUGUAmoiAxD7ASADEIUDCwJAICggASgC6AFBf3NyQQFxDQAgASgC8AFFDQAgERDTAQsgBiABKALcAUGBgICAeEZyRQRAIAFB3AFqEL4CCwJAIAkgASgCsAFBf3NyQQFxDQAgASgCuAFFDQAgEBDRAQsCQCACIAEoAogBQX9zckEBcQ0AIAEoApABRQ0AIA8Q0gELIAQgASgCfEGBgICAeEZyRQRAIAFB/ABqEL4CCyAnIAEoAnBBgICAgHhGckUEQCABQfAAahD/AgsCQCAmIAEoAkhBf3NyQQFxDQAgASgCUEUNACAOENEBCyAlIAEoAjxBgYCAgHhGckUEQCABQTxqEL4CCyAkIAEoAjBBgYCAgHhGcg0AIAFBMGoQvgILIAFBHGoQgAILIAFB0AVqJAAMAgtBuJLAAEExEIQDAAsgAiADKAIoQaiXwAAQvQIACyANKAIMIQIgEyANKAIIIgFBAkYEf0EBBSANQfgCaiANQRBqQdgC/AoAACANIAI2AvQCIA0gATYC8AIgDUEANgLoAgJ/QdH/wAAtAAAaQfACQQgQ2QIiAgRAIAJCgYCAgBA3AwAgAkEIaiANQegCakHoAvwKAAAgAgwBC0EIQfACEIwDAAtBCGohAkEACyIBNgIIIBMgAkEAIAEbNgIEIBNBACACIAEbNgIAIA1B0AVqJAAgEygCACATKAIEIBMoAgggE0EQaiQAC6sYAhV/AX4jAEEQayINJAAjAEEgayILJAAgC0EQaiISIAAQuQEgCygCECEEIAtBADYCHCMAQdABayIBJAAgAUHIAWogC0EcahDdAiABKALMASEAIAtBCGoiEwJ/AkAgASgCyAEiAkUNACABIAA2AsQBIAEgAjYCwAEgAUG4AWogAUHAAWpBronAAEEGIARBjAJqEJ4BAkAgASgCuAFBAXEEQCABKAK8ASEADAELIAFBsAFqIAFBwAFqQbSJwABBBSAEEKEBIAEoArABQQFxBEAgASgCtAEhAAwBCyABQagBaiABQcABakG5icAAQRcgBEEIahChASABKAKoAUEBcQRAIAEoAqwBIQAMAQsgAUGgAWogAUHAAWpB0InAAEEFIARBmAJqEJ4BIAEoAqABQQFxBEAgASgCpAEhAAwBCyABQZgBaiABQcABakHVicAAQQsgBEHIAGoQogEgASgCmAFBAXEEQCABKAKcASEADAELIAFBkAFqIAFBwAFqQeCJwABBBSAEQRBqEKEBIAEoApABQQFxBEAgASgClAEhAAwBCyABQYgBaiABQcABakHlicAAQQcgBEHoAWoQuwEgASgCiAFBAXEEQCABKAKMASEADAELIAFBgAFqIAFBwAFqQeyJwABBCSAEQaQCahCeASABKAKAAUEBcQRAIAEoAoQBIQAMAQsgAUH4AGogAUHAAWpB9YnAAEELIARB2AJqEJ8BIAEoAnhBAXEEQCABKAJ8IQAMAQsgAUHwAGogAUHAAWpBgIrAAEEKIARB2QJqEJ8BIAEoAnBBAXEEQCABKAJ0IQAMAQsjAEEQayICJAAgAUHAAWoiAygCACEAAn8gBEHoAGoiBSgCAARAIAJBCGogACAFEFIgAigCCCEAIAIoAgwMAQsgAiAAEL8CIAIoAgAhACACKAIECyEFQQEhBiAAQQFxRQRAIANBBGpBiorAAEENEIMCIAUQ6gJBACEGCyABQegAaiIAIAU2AgQgACAGNgIAIAJBEGokACABKAJoQQFxBEAgASgCbCEADAELIAFB4ABqIAFBwAFqQZeKwABBBCAEQYgBahCiASABKAJgQQFxBEAgASgCZCEADAELIAFB2ABqIAFBwAFqQZuKwABBCiAEQRhqEKEBIAEoAlhBAXEEQCABKAJcIQAMAQsgAUHQAGogAUHAAWpBpYrAAEEFIARBIGoQoQEgASgCUEEBcQRAIAEoAlQhAAwBCyABQcgAaiABQcABakGqisAAQQsgBEGwAmoQngEgASgCSEEBcQRAIAEoAkwhAAwBCyMAQRBrIgokACABQcABaiIUKAIAIQACfyAEQagBaiICKAIABEAjAEFAaiIDJAAgAigCACIOKQMAIRYgA0EoaiAAIAIoAgwiDxDzASAKQQhqIhUCfwJAAkAgAygCKEECRgRAIAMoAiwhBgwBCyADQSBqIANBOGooAgA2AgAgA0EYaiADQTBqKQIANwMAIAMgAykCKDcDECAOQQhqIQYgFkJ/hUKAgYKEiJCgwIB/gyEWA0AgD0UNAiAWUARAA0AgDkHAAWshDiAGKQMAIAZBCGohBkKAgYKEiJCgwIB/gyIWQoCBgoSIkKDAgH9RDQALIBZCgIGChIiQoMCAf4UhFgsgAyAOIBZ6p0EDdkFobGoiAEEYazYCPCADIABBDGs2AiggD0EBayEPIBZCAX0gFoMhFiADQQhqIRAgA0EoaiEIIwBBEGsiCSQAIAlBCGogA0EQaiIFIANBPGoQqAECfyAJKAIIIgBBAXEEQCAJKAIMDAELIwBBIGsiByQAIAUiACgCCCECIABBADYCCAJAAkAgAgRAIAcgACgCDCIMNgIUIAAoAhAhAgJ/IAgoAgAiCCgCAEGAgICAeEcEQCAHQQhqIAIgCCgCBCAIKAIIEMkCIAcoAgghESAHKAIMDAELIAcgAhC/AiAHKAIAIREgBygCBAshAkEBIQggEUEBcQRAIAxBhAFJDQIgDBD0AQwCCyAHIAI2AhggAEEEaiEIAkACQCAAKAIAQQFGBEAgByAMNgIcIAdBHGoQ7gINAUGIlMAAQTMQoAEhACAMQYQBTwRAIAwQ9AELIAJBhAFPBEAgAhD0AQtBASEIDAULIAggB0EUaiAHQRhqENQCIgBBhAFPBEAgABD0ASAHKAIYIQILIAJBhAFPBEAgAhD0AQtBACEIIAcoAhQiAEGEAUkNASAAEPQBDAELIAggDCACEOoCQQAhCAsMAgtBuJLAAEExEIQDAAsgAiEACyAJIAA2AgQgCSAINgIAIAdBIGokACAJKAIAIQAgCSgCBAshAiAQIAA2AgAgECACNgIEIAlBEGokACADKAIIQQFxRQ0ACyADKAIMIQYgBRCBAgtBAQwBCyADQThqIANBIGooAgA2AgAgA0EwaiADQRhqKQMANwMAIAMgAykDEDcDKCADIANBKGoQggIgAygCBCEGIAMoAgALNgIAIBUgBjYCBCADQUBrJAAgCigCCCEAIAooAgwMAQsgCiAAEL8CIAooAgAhACAKKAIECyECQQEhBiAAQQFxRQRAIBRBBGpBtYrAAEELEIMCIAIQ6gJBACEGCyABQUBrIgAgAjYCBCAAIAY2AgAgCkEQaiQAIAEoAkBBAXEEQCABKAJEIQAMAQsjAEEQayICJAAgAUHAAWoiDCgCACEFIwBBMGsiACQAIARB9AFqIgMoAgQhBiAAQSBqIAVBASADKAIIIgUQuQIgAkEIaiIOAn8CQAJAIAAoAiBFBEAgACgCJCEGDAELIABBGGoiCCAAQShqIhAoAgA2AgAgACAAKQIgNwMQIAVFDQEgBUEMbCEPA0ACQCAAIAY2AiAjAEEQayIFJAAgAEEQaiIDKAIIIQcgBUEIaiADKAIAIABBIGooAgAiCSgCBCAJKAIIEMkCQQEhCSAFKAIMIQogBSgCCEEBcUUEQCADQQRqIAcgChDrAiADIAdBAWo2AghBACEJCyAAQQhqIgMgCjYCBCADIAk2AgAgBUEQaiQAIAAoAghBAXENACAGQQxqIQYgD0EMayIPDQEMAwsLIAAoAgwhBiAAKAIUIgVBhAFJDQAgBRD0AQtBAQwBCyAQIAgoAgA2AgAgACAAKQMQNwMgIAAgAEEgahDTAiAAKAIEIQYgACgCAAs2AgAgDiAGNgIEIABBMGokAEEBIQAgAigCDCEFIAIoAghBAXFFBEAgDEEEakHAisAAQRwQgwIgBRDqAkEAIQALIAFBOGoiAyAFNgIEIAMgADYCACACQRBqJAAgASgCOEEBcQRAIAEoAjwhAAwBCyMAQRBrIgIkACACQQhqIAFBwAFqIgMoAgAgBEGAAmoQX0EBIQAgAigCDCEFIAIoAghBAXFFBEAgA0EEakHcisAAQR8QgwIgBRDqAkEAIQALIAFBMGoiAyAFNgIEIAMgADYCACACQRBqJAAgASgCMEEBcQRAIAEoAjQhAAwBCyMAQRBrIgIkACABQcABaiIDKAIAIQACfyAEQdQCaigCACIFBEAgAkEIaiAFIAAQOCACKAIIIQAgAigCDAwBCyACIAAQvwIgAigCACEAIAIoAgQLIQVBASEGIABBAXFFBEAgA0EEakH7isAAQQQQgwIgBRDqAkEAIQYLIAFBKGoiACAFNgIEIAAgBjYCACACQRBqJAAgASgCKEEBcQRAIAEoAiwhAAwBCyABQSBqIAFBwAFqQf+KwABBBCAEQbwCahCeASABKAIgQQFxBEAgASgCJCEADAELIAFBGGogAUHAAWpBg4vAAEEIIARByAFqEKIBIAEoAhhBAXEEQCABKAIcIQAMAQsgAUEQaiABQcABakGLi8AAQQcgBEHIAmoQngEgASgCEEEBcQRAIAEoAhQhAAwBCyABQQhqIAFBwAFqQZKLwABBDCAEQShqEMABIAEoAghBAXEEQCABKAIMIQAMAQsgASABKALAASABKALEARDcAiABKAIEIQAgASgCAAwCCyABKALEASIEQYQBSQ0AIAQQ9AELQQELNgIAIBMgADYCBCABQdABaiQAIAsoAgghBCALKAIMIQAgEhD3ASANIARBAXEiBDYCCCANIABBACAEGzYCBCANQQAgACAEGzYCACALQSBqJAAgDSgCACANKAIEIA0oAgggDUEQaiQAC90BAQR/IwBBEGsiAiQAEIIBIgMgACYBIwBB8AJrIgEkACABQQhqIAMQMyABKAIIIQMgAiABKAKkASIEQYCAgIB4RgR/QQEFIAFBxAFqIAFBCGpBBHJBmAH8CgAAIAFB6AJqIAFBsAFqKQMANwMAIAEgASkDqAE3A+ACIAEgBDYC3AIgASADNgLAASABQQA2ArgBIAFBuAFqEOUBQQhqIQNBAAsiBDYCCCACIANBACAEGzYCBCACQQAgAyAEGzYCACABQfACaiQAIAIoAgAgAigCBCACKAIIIAJBEGokAAuSAQEEfyMAQRBrIgIkACMAQSBrIgEkACABQRBqIgMgABC5ASABKAIQIQAgAUEANgIcIAFBCGogACABQRxqEDggASgCCCEEIAEoAgwhACADEPYBIAIgBEEBcSIDNgIIIAIgAEEAIAMbNgIEIAJBACAAIAMbNgIAIAFBIGokACACKAIAIAIoAgQgAigCCCACQRBqJAALkgEBBH8jAEEQayICJAAjAEEgayIBJAAgAUEQaiIDIAAQuQEgASgCECEAIAFBADYCHCABQQhqIAFBHGogABBSIAEoAgghBCABKAIMIQAgAxD2ASACIARBAXEiAzYCCCACIABBACADGzYCBCACQQAgACADGzYCACABQSBqJAAgAigCACACKAIEIAIoAgggAkEQaiQACx4AIAAoAgBBgICAgHhHBEAgABD/AiAAQQxqEP8CCwsoAQF/IAEoAgAgAUEANgIARQRAQbiSwABBMRCEAwALIAAgASgCBBA8Cy0BAX5B8P/AACkDACEBQfD/wABCADcDACAAIAFCIIg+AgQgACABp0EBRjYCAAsiAQF/IAAoAgAiACAAQR91IgJzIAJrIABBf3NBH3YgARBYC8MCAQh/IwBBEGsiBSQAEIIBIgYgASYBIwBBIGsiAyQAIANBFGogABDDASADQQhqIQQgAygCFCECIwBBQGoiACQAIABBCGogBhA1IAAoAgwhBiAAKAIIIgcEfyAAQThqIgggAEEgaikDADcDACAAQTBqIgkg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aG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZV9qc29uLTEuMC4xNDIvc3JjL3ZhbHVlL2RlLnJzACAAEABnAAAAcgAAABkAQaCBwAALBQEAAAADAEGwgcAACwUBAAAABABBwIHAAAsFAQAAAAQAQdCBwAALBQEAAAAFAEHggcAACwUBAAAABgBB8IHAAAsFAQAAAAcAQYCCwAALBQEAAAAIAEGQgsAACwUBAAAABABBoILAAAsFAQAAAAkAQbCCwAALBQEAAAAKAEHAgsAAC6MOAQAAAAMAAABDb3VsZG4ndCBkZXNlcmlhbGl6ZSBpNjQgb3IgdTY0IGZyb20gYSBCaWdJbnQgb3V0c2lkZSBpNjQ6Ok1JTi4udTY0OjpNQVggYm91bmRzAP//////////YXV0aG9yY2FyZHNjYXJkc194X25vZGVzX3hfd2lkZ2V0c2NvbG9yZGVzY3JpcHRpb25lZGdlc2dyYXBoaWRpY29uY2xhc3Npc19lZGl0YWJsZWlzcmVzb3VyY2Vqc29ubGRjb250ZXh0bmFtZW5vZGVncm91cHNub2Rlc29udG9sb2d5X2lkcHVibGljYXRpb25yZWxhdGFibGVfcmVzb3VyY2VfbW9kZWxfaWRzcmVzb3VyY2VfMl9yZXNvdXJjZV9jb25zdHJhaW50c3Jvb3RzbHVnc3VidGl0bGV2ZXJzaW9uZXh0cmFfZmllbGRzYWxpYXNjb25maWdkYXRhdHlwZWV4cG9ydGFibGVmaWVsZG5hbWVncmFwaF9pZGhhc2N1c3RvbWFsaWFzaXNfY29sbGVjdG9yaXNyZXF1aXJlZGlzc2VhcmNoYWJsZWlzdG9wbm9kZW5vZGVncm91cF9pZG5vZGVpZG9udG9sb2d5Y2xhc3NwYXJlbnRwcm9wZXJ0eXNvcnRvcmRlcnNvdXJjZWJyYW5jaHB1YmxpY2F0aW9uX2lkL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvd2FzbS1iaW5kZ2VuLTAuMi4xMDAvc3JjL2NvbnZlcnQvc2xpY2VzLnJzAABHAxAAbwAAACQBAAAOAAAACwAAAAQAAAAEAAAADAAAAGNhbGxlZCBgUmVzdWx0Ojp1bndyYXAoKWAgb24gYW4gYEVycmAgdmFsdWVzcmMvZ3JhcGgucnMAAwQQAAwAAABnAQAAGAAAAAMEEAAMAAAAbQEAABgAAAADBBAADAAAAHMBAAAYAAAAAwQQAAwAAACCAQAADQAAAAMEEAAMAAAAgAEAAA0AAABhdHRlbXB0ZWQgdG8gdGFrZSBvd25lcnNoaXAgb2YgUnVzdCB2YWx1ZSB3aGlsZSBpdCB3YXMgYm9ycm93ZWRTdGF0aWNHcmFwaE1ldGFhdXRob3JjYXJkc2NhcmRzX3hfbm9kZXNfeF93aWRnZXRzY29sb3JkZXNjcmlwdGlvbmVkZ2VzZ3JhcGhpZGljb25jbGFzc2lzX2VkaXRhYmxlaXNyZXNvdXJjZWpzb25sZGNvbnRleHRuYW1lbm9kZWdyb3Vwc25vZGVzb250b2xvZ3lfaWRwdWJsaWNhdGlvbnJlbGF0YWJsZV9yZXNvdXJjZV9tb2RlbF9pZHNyZXNvdXJjZV8yX3Jlc291cmNlX2NvbnN0cmFpbnRzcm9vdHNsdWdzdWJ0aXRsZXZlcnNpb25leHRyYV9maWVsZHMAAK4EEAAGAAAAtAQQAAUAAAC5BBAAFwAAANAEEAAFAAAA1QQQAAsAAADgBBAABQAAAOUEEAAHAAAA7AQQAAkAAAD1BBAACwAAAAAFEAAKAAAACgUQAA0AAAAXBRAABAAAABsFEAAKAAAAJQUQAAUAAAAqBRAACwAAADUFEAALAAAAQAUQABwAAABcBRAAHwAAAHsFEAAEAAAAfwUQAAQAAACDBRAACAAAAIsFEAAHAAAAkgUQAAwAAABzdHJ1Y3QgU3RhdGljR3JhcGhNZXRhU3RhdGljTm9kZWFsaWFzY29uZmlnZGF0YXR5cGVleHBvcnRhYmxlZmllbGRuYW1lZ3JhcGhfaWRoYXNjdXN0b21hbGlhc2lzX2NvbGxlY3RvcmlzcmVxdWlyZWRpc3NlYXJjaGFibGVpc3RvcG5vZGVub2RlZ3JvdXBfaWRub2RlaWRvbnRvbG9neWNsYXNzcGFyZW50cHJvcGVydHlzb3J0b3JkZXJzb3VyY2VicmFuY2hwdWJsaWNhdGlvbl9pZAB4BhAABQAAAH0GEAAGAAAAgwYQAAgAAADVBBAACwAAAIsGEAAKAAAAlQYQAAkAAACeBhAACAAAAKYGEAAOAAAAtAYQAAwAAADABhAACgAAAMoGEAAMAAAA1gYQAAkAAAAXBRAABAAAAN8GEAAMAAAA6wYQAAYAAADxBhAADQAAAP4GEAAOAAAADAcQAAkAAAAVBxAAGgAAAHN0cnVjdCBTdGF0aWNOb2RlL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvc2VyZGVfanNvbi0xLjAuMTQyL3NyYy92YWx1ZS9tb2QucnMAAADZBxAAaAAAAHMAAAAKAAAAAAAAAP//////////WAgQAEH0kMAAC8cDBAAAAAQAAAANAAAADgAAAA4AAAAAAAAABAAAAAQAAAAPAAAAEAAAABAAAAAAAAAABAAAAAQAAAARAAAAEgAAABIAAAAvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9jb25zb2xlX2Vycm9yX3BhbmljX2hvb2stMC4xLjcvc3JjL2xpYi5ycwAAuAgQAG4AAACVAAAADgAAAGNhbGxlZCBgT3B0aW9uOjp1bndyYXBfdGhyb3coKWAgb24gYSBgTm9uZWAgdmFsdWVNYXBBY2Nlc3M6Om5leHRfdmFsdWUgY2FsbGVkIGJlZm9yZSBuZXh0X2tleS9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3NlcmRlLTEuMC4yMTkvc3JjL2RlL3ZhbHVlLnJzAJUJEABiAAAAZgUAABsAAABNYXAga2V5IGlzIG5vdCBhIHN0cmluZyBhbmQgY2Fubm90IGJlIGFuIG9iamVjdCBrZXkAQcSUwAAL5QEBAAAAEwAAABQAAAAVAAAAT25jZSBpbnN0YW5jZSBoYXMgcHJldmlvdXNseSBiZWVuIHBvaXNvbmVkAABUChAAKgAAAG9uZS10aW1lIGluaXRpYWxpemF0aW9uIG1heSBub3QgYmUgcGVyZm9ybWVkIHJlY3Vyc2l2ZWx5iAoQADgAAAAvcnVzdGMvMDc3Y2VkYzJhZmE4YWMwYjcyN2I3YTZjYmUwMTI5NDBiYTIyOGRlYi9saWJyYXJ5L3N0ZC9zcmMvc3luYy9wb2lzb24vb25jZS5ycwDIChAAUwAAAJsAAAAyAEG0lsAAC6cHAQAAAAcAAABhIHNlcXVlbmNlL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvc2VyZGUtMS4wLjIxOS9zcmMvZGUvaW1wbHMucnNGCxAAYgAAAJUEAAAiAAAARgsQAGIAAACYBAAAHAAAAGEgbWFwaW52YWxpZCB2YWx1ZTogLCBleHBlY3RlZCAAzQsQAA8AAADcCxAACwAAAG1pc3NpbmcgZmllbGQgYGD4CxAADwAAAAcMEAABAAAAZHVwbGljYXRlIGZpZWxkIGAAAAAYDBAAEQAAAAcMEAABAAAAL3J1c3RjLzA3N2NlZGMyYWZhOGFjMGI3MjdiN2E2Y2JlMDEyOTQwYmEyMjhkZWIvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAADwMEABKAAAAvgEAAB0AAAA8DBAASgAAAKgBAAAfAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvc2VyZGVfanNvbi0xLjAuMTQyL3NyYy92YWx1ZS9tb2QucnOoDBAAaAAAAHMAAAAKAAAAL3J1c3RjLzA3N2NlZGMyYWZhOGFjMGI3MjdiN2E2Y2JlMDEyOTQwYmEyMjhkZWIvbGlicmFyeS9hbGxvYy9zcmMvY29sbGVjdGlvbnMvYnRyZWUvbWFwLnJzAAAgDRAAWgAAAOMAAAA7AAAAaW50ZXJuYWwgZXJyb3I6IGVudGVyZWQgdW5yZWFjaGFibGUgY29kZSANEABaAAAA5gAAACwAAAAgDRAAWgAAAPoAAAA/AAAAIA0QAFoAAAAfAQAALgAAAC9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvc3RkL3NyYy90aHJlYWQvbG9jYWwucnMA5A0QAE8AAAAVAQAAGQAAAGludmFsaWQgdHlwZTogLCBleHBlY3RlZCAAAABEDhAADgAAAFIOEAALAAAAL3J1c3RjLzA3N2NlZGMyYWZhOGFjMGI3MjdiN2E2Y2JlMDEyOTQwYmEyMjhkZWIvbGlicmFyeS9hbGxvYy9zcmMvc2xpY2UucnMAAHAOEABKAAAAvgEAAB0AAAAAAAAA///////////QDhAAQeidwAAL+QQBAAAAAAAAACBjYW4ndCBiZSByZXByZXNlbnRlZCBhcyBhIEphdmFTY3JpcHQgbnVtYmVyAQAAAAAAAADwDhAALAAAAB0AAAAvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9zZXJkZS13YXNtLWJpbmRnZW4tMC42LjUvc3JjL2xpYi5yczAPEABoAAAANQAAAA4AAAAvaG9tZS9waGlsdHdlaXIvLmNhcmdvL3JlZ2lzdHJ5L3NyYy9pbmRleC5jcmF0ZXMuaW8tMTk0OWNmOGM2YjViNTU3Zi9qcy1zeXMtMC4zLjc3L3NyYy9saWIucnMAAACoDxAAXQAAAPsYAAABAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvd2FzbS1iaW5kZ2VuLTAuMi4xMDAvc3JjL2NvbnZlcnQvc2xpY2VzLnJzABgQEABvAAAAJAEAAA4AAABjbG9zdXJlIGludm9rZWQgcmVjdXJzaXZlbHkgb3IgYWZ0ZXIgYmVpbmcgZHJvcHBlZC9ob21lL3BoaWx0d2Vpci8uY2FyZ28vcmVnaXN0cnkvc3JjL2luZGV4LmNyYXRlcy5pby0xOTQ5Y2Y4YzZiNWI1NTdmL3dhc20tYmluZGdlbi0wLjIuMTAwL3NyYy9jb252ZXJ0L3NsaWNlcy5ycwAAAMoQEABvAAAAJAEAAA4AAAAqAAAADAAAAAQAAAArAAAALAAAAC0AQeyiwAALgQcBAAAALgAAAGEgRGlzcGxheSBpbXBsZW1lbnRhdGlvbiByZXR1cm5lZCBhbiBlcnJvciB1bmV4cGVjdGVkbHkvcnVzdGMvMDc3Y2VkYzJhZmE4YWMwYjcyN2I3YTZjYmUwMTI5NDBiYTIyOGRlYi9saWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMAAKsREABLAAAA7goAAA4AAABFcnJvci9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL3Jhd192ZWMvbW9kLnJzAAAADRIQAFAAAAAuAgAAEQAAAAoKU3RhY2s6CgoKCi9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL21hcC9lbnRyeS5yc3wSEABgAAAAoQEAAC4AAABhc3NlcnRpb24gZmFpbGVkOiBpZHggPCBDQVBBQ0lUWS9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25vZGUucnMADBMQAFsAAACVAgAACQAAAGFzc2VydGlvbiBmYWlsZWQ6IGVkZ2UuaGVpZ2h0ID09IHNlbGYuaGVpZ2h0IC0gMQwTEABbAAAArQIAAAkAAAAMExAAWwAAALECAAAJAAAAYXNzZXJ0aW9uIGZhaWxlZDogc3JjLmxlbigpID09IGRzdC5sZW4oKQwTEABbAAAASgcAAAUAAAAMExAAWwAAAMcEAAAjAAAADBMQAFsAAAAKBQAAJAAAAGFzc2VydGlvbiBmYWlsZWQ6IGVkZ2UuaGVpZ2h0ID09IHNlbGYubm9kZS5oZWlnaHQgLSAxAAAADBMQAFsAAAD6AwAACQAAAC9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL2NvbGxlY3Rpb25zL2J0cmVlL25hdmlnYXRlLnJzAGgUEABfAAAAWAIAADAAAAAxAAAADAAAAAQAAAAyAAAAMwAAADQAQfipwAALwSABAAAANQAAAGEgRGlzcGxheSBpbXBsZW1lbnRhdGlvbiByZXR1cm5lZCBhbiBlcnJvciB1bmV4cGVjdGVkbHkvcnVzdGMvMDc3Y2VkYzJhZmE4YWMwYjcyN2I3YTZjYmUwMTI5NDBiYTIyOGRlYi9saWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMAADcVEABLAAAA7goAAA4AAABFcnJvcgAAADcVEABLAAAAWAQAABIAAABFT0Ygd2hpbGUgcGFyc2luZyBhIGxpc3RFT0Ygd2hpbGUgcGFyc2luZyBhbiBvYmplY3RFT0Ygd2hpbGUgcGFyc2luZyBhIHN0cmluZ0VPRiB3aGlsZSBwYXJzaW5nIGEgdmFsdWVleHBlY3RlZCBgOmBleHBlY3RlZCBgLGAgb3IgYF1gZXhwZWN0ZWQgYCxgIG9yIGB9YGV4cGVjdGVkIGlkZW50ZXhwZWN0ZWQgdmFsdWVleHBlY3RlZCBgImBpbnZhbGlkIGVzY2FwZWludmFsaWQgbnVtYmVybnVtYmVyIG91dCBvZiByYW5nZWludmFsaWQgdW5pY29kZSBjb2RlIHBvaW50Y29udHJvbCBjaGFyYWN0ZXIgKFx1MDAwMC1cdTAwMUYpIGZvdW5kIHdoaWxlIHBhcnNpbmcgYSBzdHJpbmdrZXkgbXVzdCBiZSBhIHN0cmluZ2ludmFsaWQgdmFsdWU6IGV4cGVjdGVkIGtleSB0byBiZSBhIG51bWJlciBpbiBxdW90ZXNmbG9hdCBrZXkgbXVzdCBiZSBmaW5pdGUgKGdvdCBOYU4gb3IgKy8taW5mKWxvbmUgbGVhZGluZyBzdXJyb2dhdGUgaW4gaGV4IGVzY2FwZXRyYWlsaW5nIGNvbW1hdHJhaWxpbmcgY2hhcmFjdGVyc3VuZXhwZWN0ZWQgZW5kIG9mIGhleCBlc2NhcGVyZWN1cnNpb24gbGltaXQgZXhjZWVkZWRFcnJvcigsIGxpbmU6ICwgY29sdW1uOiApAAAA4BcQAAYAAADmFxAACAAAAO4XEAAKAAAA+BcQAAEAAAAvcnVzdGMvMDc3Y2VkYzJhZmE4YWMwYjcyN2I3YTZjYmUwMTI5NDBiYTIyOGRlYi9saWJyYXJ5L2FsbG9jL3NyYy9jb2xsZWN0aW9ucy9idHJlZS9uYXZpZ2F0ZS5ycwAcGBAAXwAAAMYAAAAnAAAAHBgQAF8AAAAWAgAALwAAABwYEABfAAAAoQAAACQAAAAvcnVzdGMvMDc3Y2VkYzJhZmE4YWMwYjcyN2I3YTZjYmUwMTI5NDBiYTIyOGRlYi9saWJyYXJ5L2FsbG9jL3NyYy9zdHJpbmcucnMArBgQAEsAAABYBAAAEgAAADYAAAAMAAAABAAAADcAAAA4AAAANAAAAGFueSB2YWxpZCBKU09OIHZhbHVlYSBzdHJpbmcga2V5L3J1c3RjLzA3N2NlZGMyYWZhOGFjMGI3MjdiN2E2Y2JlMDEyOTQwYmEyMjhkZWIvbGlicmFyeS9hbGxvYy9zcmMvcmF3X3ZlYy9tb2QucnNAGRAAUAAAAC4CAAARAAAAAAAAAAgAAAAEAAAAPgAAAD8AAABAAAAAYSBib29sZWFuYSBzdHJpbmdieXRlIGFycmF5Ym9vbGVhbiBgYAAAANMZEAAJAAAA3BkQAAEAAABpbnRlZ2VyIGAAAADwGRAACQAAANwZEAABAAAAZmxvYXRpbmcgcG9pbnQgYAwaEAAQAAAA3BkQAAEAAABjaGFyYWN0ZXIgYAAsGhAACwAAANwZEAABAAAAc3RyaW5nIABIGhAABwAAAHVuaXQgdmFsdWVPcHRpb24gdmFsdWVuZXd0eXBlIHN0cnVjdHNlcXVlbmNlbWFwZW51bXVuaXQgdmFyaWFudG5ld3R5cGUgdmFyaWFudHR1cGxlIHZhcmlhbnRzdHJ1Y3QgdmFyaWFudAAAAAEAAAAAAAAALjBpMzJ1MzJMYXp5IGluc3RhbmNlIGhhcyBwcmV2aW91c2x5IGJlZW4gcG9pc29uZWQAANQaEAAqAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvb25jZV9jZWxsLTEuMjEuMy9zcmMvbGliLnJzCBsQAGAAAAAIAwAAGQAAAHJlZW50cmFudCBpbml0AAB4GxAADgAAAAgbEABgAAAAegIAAA0AAABKc1ZhbHVlKCkAAACgGxAACAAAAKgbEAABAAAAL2hvbWUvcGhpbHR3ZWlyLy5jYXJnby9yZWdpc3RyeS9zcmMvaW5kZXguY3JhdGVzLmlvLTE5NDljZjhjNmI1YjU1N2Yvd2FzbS1iaW5kZ2VuLTAuMi4xMDAvc3JjL2NvbnZlcnQvc2xpY2VzLnJzbnVsbCBwb2ludGVyIHBhc3NlZCB0byBydXN0cmVjdXJzaXZlIHVzZSBvZiBhbiBvYmplY3QgZGV0ZWN0ZWQgd2hpY2ggd291bGQgbGVhZCB0byB1bnNhZmUgYWxpYXNpbmcgaW4gcnVzdAAAALwbEABvAAAA6AAAAAEAAABsaWJyYXJ5L3N0ZC9zcmMvcGFuaWNraW5nLnJzL3J1c3RjLzA3N2NlZGMyYWZhOGFjMGI3MjdiN2E2Y2JlMDEyOTQwYmEyMjhkZWIvbGlicmFyeS9hbGxvYy9zcmMvcmF3X3ZlYy9tb2QucnPEHBAAUAAAAC4CAAARAAAAOgAAAAEAAAAAAAAAJB0QAAEAAAAkHRAAAQAAAEgAAAAMAAAABAAAAEkAAABKAAAASwAAAC9ydXN0Yy8wNzdjZWRjMmFmYThhYzBiNzI3YjdhNmNiZTAxMjk0MGJhMjI4ZGViL2xpYnJhcnkvYWxsb2Mvc3JjL3NsaWNlLnJzAABYHRAASgAAAL4BAAAdAAAAL3J1c3QvZGVwcy9kbG1hbGxvYy0wLjIuOC9zcmMvZGxtYWxsb2MucnNhc3NlcnRpb24gZmFpbGVkOiBwc2l6ZSA+PSBzaXplICsgbWluX292ZXJoZWFkALQdEAApAAAArAQAAAkAAABhc3NlcnRpb24gZmFpbGVkOiBwc2l6ZSA8PSBzaXplICsgbWF4X292ZXJoZWFkAAC0HRAAKQAAALIEAAANAAAAQWNjZXNzRXJyb3JjYW5ub3QgYWNjZXNzIGEgVGhyZWFkIExvY2FsIFN0b3JhZ2UgdmFsdWUgZHVyaW5nIG9yIGFmdGVyIGRlc3RydWN0aW9uOiAAZx4QAEgAAAABAAAAAAAAAGVudGl0eSBub3QgZm91bmRwZXJtaXNzaW9uIGRlbmllZGNvbm5lY3Rpb24gcmVmdXNlZGNvbm5lY3Rpb24gcmVzZXRob3N0IHVucmVhY2hhYmxlbmV0d29yayB1bnJlYWNoYWJsZWNvbm5lY3Rpb24gYWJvcnRlZG5vdCBjb25uZWN0ZWRhZGRyZXNzIGluIHVzZWFkZHJlc3Mgbm90IGF2YWlsYWJsZW5ldHdvcmsgZG93bmJyb2tlbiBwaXBlZW50aXR5IGFscmVhZHkgZXhpc3Rzb3BlcmF0aW9uIHdvdWxkIGJsb2Nrbm90IGEgZGlyZWN0b3J5aXMgYSBkaXJlY3RvcnlkaXJlY3Rvcnkgbm90IGVtcHR5cmVhZC1vbmx5IGZpbGVzeXN0ZW0gb3Igc3RvcmFnZSBtZWRpdW1maWxlc3lzdGVtIGxvb3Agb3IgaW5kaXJlY3Rpb24gbGltaXQgKGUuZy4gc3ltbGluayBsb29wKXN0YWxlIG5ldHdvcmsgZmlsZSBoYW5kbGVpbnZhbGlkIGlucHV0IHBhcmFtZXRlcmludmFsaWQgZGF0YXRpbWVkIG91dHdyaXRlIHplcm9ubyBzdG9yYWdlIHNwYWNlc2VlayBvbiB1bnNlZWthYmxlIGZpbGVxdW90YSBleGNlZWRlZGZpbGUgdG9vIGxhcmdlcmVzb3VyY2UgYnVzeWV4ZWN1dGFibGUgZmlsZSBidXN5ZGVhZGxvY2tjcm9zcy1kZXZpY2UgbGluayBvciByZW5hbWV0b28gbWFueSBsaW5rc2ludmFsaWQgZmlsZW5hbWVhcmd1bWVudCBsaXN0IHRvbyBsb25nb3BlcmF0aW9uIGludGVycnVwdGVkdW5zdXBwb3J0ZWR1bmV4cGVjdGVkIGVuZCBvZiBmaWxlb3V0IG9mIG1lbW9yeWluIHByb2dyZXNzb3RoZXIgZXJyb3J1bmNhdGVnb3JpemVkIGVycm9yIChvcyBlcnJvciApAAAAAQAAAAAAAACtIRAACwAAALghEAABAAAAcGFuaWNrZWQgYXQgOgptZW1vcnkgYWxsb2NhdGlvbiBvZiAgYnl0ZXMgZmFpbGVk4iEQABUAAAD3IRAADQAAAGxpYnJhcnkvc3RkL3NyYy9hbGxvYy5ycxQiEAAYAAAAZAEAAAkAAABjYW5ub3QgbW9kaWZ5IHRoZSBwYW5pYyBob29rIGZyb20gYSBwYW5pY2tpbmcgdGhyZWFkPCIQADQAAACoHBAAHAAAAJAAAAAJAAAASAAAAAwAAAAEAAAATAAAAAAAAAAIAAAABAAAAE0AAAAAAAAACAAAAAQAAABOAAAATwAAAFAAAABRAAAAUgAAABAAAAAEAAAAUwAAAFQAAABVAAAAVgAAAG9wZXJhdGlvbiBzdWNjZXNzZnVsEAAAABEAAAASAAAAEAAAABAAAAATAAAAEgAAAA0AAAAOAAAAFQAAAAwAAAALAAAAFQAAABUAAAAPAAAADgAAABMAAAAmAAAAOAAAABkAAAAXAAAADAAAAAkAAAAKAAAAEAAAABcAAAAOAAAADgAAAA0AAAAUAAAACAAAABsAAAAOAAAAEAAA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biBgbGVmdCAgcmlnaHRgIGZhaWxlZAogIGxlZnQ6IAogcmlnaHQ6IAALNRAAEAAAABs1EAAXAAAAMjUQAAkAAAAgcmlnaHRgIGZhaWxlZDogCiAgbGVmdDogAAAACzUQABAAAABUNRAAEAAAAGQ1EAAJAAAAMjUQAAkAAAA6IAAAAQAAAAAAAACQNRAAAgAAAH0gfTB4MDAwMTAyMDMwNDA1MDYwNzA4MDkxMDExMTIxMzE0MTUxNjE3MTgxOTIwMjEyMjIzMjQyNTI2MjcyODI5MzAzMTMyMzMzNDM1MzYzNzM4Mzk0MDQxNDI0MzQ0NDU0NjQ3NDg0OTUwNTE1MjUzNTQ1NTU2NTc1ODU5NjA2MTYyNjM2NDY1NjY2NzY4Njk3MDcxNzI3Mzc0NzU3Njc3Nzg3OTgwODE4MjgzODQ4NTg2ODc4ODg5OTA5MTkyOTM5NDk1OTY5Nzk4OTkwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwbGlicmFyeS9jb3JlL3NyYy9mbXQvbW9kLnJzZmFsc2V0cnVlAAAAsTYQABsAAAC8CgAAJgAAALE2EAAbAAAAxQoAABoAAABsaWJyYXJ5L2NvcmUvc3JjL3N0ci9tb2QucnNbLi4uXWJlZ2luIDw9IGVuZCAoIDw9ICkgd2hlbiBzbGljaW5nIGBgABg3EAAOAAAAJjcQAAQAAAAqNxAAEAAAADo3EAABAAAAYnl0ZSBpbmRleCAgaXMgbm90IGEgY2hhciBib3VuZGFyeTsgaXQgaXMgaW5zaWRlICAoYnl0ZXMgKSBvZiBgAFw3EAALAAAAZzcQACYAAACNNxAACAAAAJU3EAAGAAAAOjcQAAEAAAAgaXMgb3V0IG9mIGJvdW5kcyBvZiBgAABcNxAACwAAAMQ3EAAWAAAAOjcQAAEAAAD4NhAAGwAAAJwBAAAsAAAAbGlicmFyeS9jb3JlL3NyYy91bmljb2RlL3ByaW50YWJsZS5ycwAAAAQ4EAAlAAAAGgAAADYAAAAEOBAAJQAAAAoAAAArAAAAAAYBAQMBBAIFBwcCCAgJAgoFCwIOBBABEQISBRMcFAEVAhcCGQ0cBR0IHwEkAWoEawKvA7ECvALPAtEC1AzVCdYC1wLaAeAF4QLnBOgC7iDwBPgC+gT7AQwnOz5OT4+enp97i5OWorK6hrEGBwk2PT5W89DRBBQYNjdWV3+qrq+9NeASh4mOngQNDhESKTE0OkVGSUpOT2RlioyNj7bBw8TGy9ZctrcbHAcICgsUFzY5Oqip2NkJN5CRqAcKOz5maY+SEW9fv+7vWmL0/P9TVJqbLi8nKFWdoKGjpKeorbq8xAYLDBUdOj9FUaanzM2gBxkaIiU+P+fs7//FxgQgIyUmKDM4OkhKTFBTVVZYWlxeYGNlZmtzeH1/iqSqr7DA0K6vbm/d3pNeInsFAwQtA2YDAS8ugIIdAzEPHAQkCR4FKwVEBA4qgKoGJAQkBCgINAtOAzQMgTcJFgoIGDtFOQNjCAkwFgUhAxsFAUA4BEsFLwQKBwkHQCAnBAwJNgM6BRoHBAwHUEk3Mw0zBy4ICgYmAx0IAoDQUhADNywIKhYaJhwUFwlOBCQJRA0ZBwoGSAgnCXULQj4qBjsFCgZRBgEFEAMFC1kIAh1iHkgICoCmXiJFCwoGDRM6BgoGFBwsBBeAuTxkUwxICQpGRRtICFMNSQcKgLYiDgoGRgodA0dJNwMOCAoGOQcKgTYZBzsDHVUBDzINg5tmdQuAxIpMYw2EMBAWCo+bBYJHmrk6hsaCOQcqBFwGJgpGCigFE4GwOoDGW2VLBDkHEUAFCwIOl/gIhNYpCqLngTMPAR0GDgQIgYyJBGsFDQMJBxCPYID6BoG0TEcJdDyA9gpzCHAVRnoUDBQMVwkZgIeBRwOFQg8VhFAfBgaA1SsFPiEBcC0DGgQCgUAfEToFAYHQKoDWKwQBgeCA9ylMBAoEAoMRREw9gMI8BgEEVQUbNAKBDiwEZAxWCoCuOB0NLAQJBwIOBoCag9gEEQMNA3cEXwYMBAEPDAQ4CAoGKAgsBAI+gVQMHQMKBTgHHAYJB4D6hAYAAQMFBQYGAgcGCAcJEQocCxkMGg0QDgwPBBADEhITCRYBFwQYARkDGgcbARwCHxYgAysDLQsuATAEMQIyAacEqQKqBKsI+gL7Bf0C/gP/Ca14eYuNojBXWIuMkBzdDg9LTPv8Li8/XF1f4oSNjpGSqbG6u8XGycre5OX/AAQREikxNDc6Oz1JSl2EjpKpsbS6u8bKzs/k5QAEDQ4REikxNDo7RUZJSl5kZYSRm53Jzs8NESk6O0VJV1tcXl9kZY2RqbS6u8XJ3+Tl8A0RRUlkZYCEsry+v9XX8PGDhYukpr6/xcfP2ttImL3Nxs7PSU5PV1leX4mOj7G2t7/BxsfXERYXW1z29/7/gG1x3t8OH25vHB1ffX6ur027vBYXHh9GR05PWFpcXn5/tcXU1dzw8fVyc490dZYmLi+nr7e/x8/X35oAQJeYMI8fzs/S1M7/Tk9aWwcIDxAnL+7vbm83PT9CRZCRU2d1yMnQ0djZ5/7/ACBfIoLfBIJECBsEBhGBrA6AqwUfCIEcAxkIAQQvBDQEBwMBBwYHEQpQDxIHVQcDBBwKCQMIAwcDAgMDAwwEBQMLBgEOFQVOBxsHVwcCBhcMUARDAy0DAQQRBg8MOgQdJV8gbQRqJYDIBYKwAxoGgv0DWQcWCRgJFAwUDGoGCgYaBlkHKwVGCiwEDAQBAzELLAQaBgsDgKwGCgYvMYD0CDwDDwM+BTgIKwWC/xEYCC8RLQMhDyEPgIwEgpoWCxWIlAUvBTsHAg4YCYC+InQMgNYagRAFgOEJ8p4DNwmBXBSAuAiA3RU7AwoGOAhGCAwGdAseA1oEWQmAgxgcChYJTASAigarpAwXBDGhBIHaJgcMBQWAphCB9QcBICoGTASAjQSAvgMbAw8NbGlicmFyeS9jb3JlL3NyYy9udW0vYmlnbnVtLnJzAPU9EAAeAAAAqwEAAAEAAABhc3NlcnRpb24gZmFpbGVkOiBub2JvcnJvd2Fzc2VydGlvbiBmYWlsZWQ6IGRpZ2l0cyA8IDQwYXNzZXJ0aW9uIGZhaWxlZDogb3RoZXIgPiAwYXR0ZW1wdCB0byBkaXZpZGUgYnkgemVybwB2PhAAGQAAACBvdXQgb2YgcmFuZ2UgZm9yIHNsaWNlIG9mIGxlbmd0aCByYW5nZSBlbmQgaW5kZXggAAC6PhAAEAAAAJg+EAAiAAAAc2xpY2UgaW5kZXggc3RhcnRzIGF0ICBidXQgZW5kcyBhdCAA3D4QABYAAADyPhAADQAAAAADAACDBCAAkQVgAF0ToAASFyAfDCBgH+8sICsqMKArb6ZgLAKo4Cwe++AtAP4gNp7/YDb9AeE2AQohNyQN4TerDmE5LxjhOTAc4UrzHuFOQDShUh5h4VPwamFUT2/hVJ28YVUAz2FWZdGhVgDaIVcA4KFYruIhWuzk4VvQ6GFcIADuXPABf10ANRAAAjUQAAQ1EAACAAAAAgAAAAcAQcj/wAALAUIAeAlwcm9kdWNlcnMCCGxhbmd1YWdlAQRSdXN0AAxwcm9jZXNzZWQtYnkDBXJ1c3RjJTEuODguMC1uaWdodGx5ICgwNzdjZWRjMmEgMjAyNS0wNC0xOSkGd2FscnVzBjAuMjMuMwx3YXNtLWJpbmRnZW4HMC4yLjEwMABrD3RhcmdldF9mZWF0dXJlcwYrD211dGFibGUtZ2xvYmFscysTbm9udHJhcHBpbmctZnB0b2ludCsLYnVsay1tZW1vcnkrCHNpZ24tZXh0Kw9yZWZlcmVuY2UtdHlwZXMrCm11bHRpdmFsdWU=";
  run = async function() {
    console.warn("Does init need called for Rust except in unbundled browser?");
    await __wbg_init(wasmURL);
    greet();
  };
  AlizarinModel = ResourceInstanceViewModel;
  setCurrentLanguage = setCurrentLanguage$1;
  getCurrentLanguage = getCurrentLanguage$1;
})();
export {
  AlizarinModel,
  GraphManager,
  GraphMutator,
  RDM,
  ResourceModelWrapper,
  WKRM,
  __tla,
  client,
  getCurrentLanguage,
  graphManager,
  interfaces,
  nodeConfig,
  renderers,
  run,
  setCurrentLanguage,
  staticStore,
  staticTypes,
  utils,
  viewModels
};
