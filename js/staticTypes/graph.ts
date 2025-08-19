
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

class StaticNode {
  alias: string | null;
  config: { [key: string]: any };
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
  ontologyclass: string | null = null;
  parentproperty: string | null = null;
  sortorder: number;
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

export { StaticGraphMeta, StaticNode };
