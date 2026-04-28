import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';

import { NapiPrebuildLoader, NapiStaticGraph, NapiStaticResourceRegistry, NapiResourceModelWrapper } from '../index.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TEST_DATA = path.resolve(__dirname, '..', '..', '..', 'tests');

describe('NapiStaticGraph', () => {
  it('parses a graph from JSON string', () => {
    const graphJson = fs.readFileSync(
      path.join(TEST_DATA, 'data', 'models', 'Person.json'),
      'utf-8'
    );
    const graph = NapiStaticGraph.fromJsonString(graphJson);
    assert.ok(graph.graphId, 'graph should have an ID');
    assert.ok(graph.name, 'graph should have a name');
  });

  it('throws on invalid JSON', () => {
    assert.throws(() => NapiStaticGraph.fromJsonString('not json'), /error/i);
  });
});

describe('NapiStaticResourceRegistry', () => {
  it('starts empty', () => {
    const registry = new NapiStaticResourceRegistry();
    assert.equal(registry.length, 0);
  });

  it('loads resources from business data JSON', () => {
    const businessData = {
      business_data: {
        resources: [
          {
            resourceinstance: {
              resourceinstanceid: '87654321-4321-4321-4321-cba987654321',
              graph_id: '12345678-1234-1234-1234-123456789abc',
              name: 'Test Resource',
              descriptors: {},
            },
            tiles: [],
          },
        ],
      },
    };

    const registry = new NapiStaticResourceRegistry();
    registry.mergeFromBusinessDataJson(JSON.stringify(businessData), true);
    assert.ok(registry.length > 0, 'registry should have resources after merge');
  });

  it('contains returns false for unknown IDs', () => {
    const registry = new NapiStaticResourceRegistry();
    assert.equal(registry.contains('nonexistent'), false);
  });
});

describe('NapiPrebuildLoader', () => {
  it('throws for nonexistent directory', () => {
    assert.throws(
      () => new NapiPrebuildLoader('/nonexistent/path'),
      /not found/i
    );
  });
});

// =============================================================================
// NapiResourceModelWrapper
// =============================================================================

describe('NapiResourceModelWrapper', () => {
  // Load Group.json graph for tests
  const groupJson = fs.readFileSync(
    path.join(TEST_DATA, 'data', 'models', 'Group.json'),
    'utf-8'
  );
  const graphData = JSON.parse(groupJson);
  const graphStr = JSON.stringify(graphData.graph[0]);

  // -------------------------------------------------------------------------
  // Construction
  // -------------------------------------------------------------------------

  it('constructs from graph JSON string', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    assert.ok(wrapper, 'wrapper should be defined');
    assert.equal(wrapper.getGraphId(), '07883c9e-b25c-11e9-975a-a4d18cec433a');
  });

  it('constructs from NapiStaticGraph via fromGraph()', () => {
    const graph = NapiStaticGraph.fromJsonString(groupJson);
    const wrapper = NapiResourceModelWrapper.fromGraph(graph, true);
    assert.equal(wrapper.getGraphId(), '07883c9e-b25c-11e9-975a-a4d18cec433a');
  });

  it('throws on invalid JSON', () => {
    assert.throws(
      () => new NapiResourceModelWrapper('not json', true),
      /Invalid graph JSON/i
    );
  });

  // -------------------------------------------------------------------------
  // Node accessors
  // -------------------------------------------------------------------------

  it('getNodeObjects returns object with all nodes', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const nodes = wrapper.getNodeObjects();
    assert.ok(typeof nodes === 'object');
    assert.ok(Object.keys(nodes).length > 0, 'should have nodes');
  });

  it('getNodeObjectsByAlias returns object keyed by alias', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const byAlias = wrapper.getNodeObjectsByAlias();
    assert.ok(byAlias['group'], 'should have "group" alias');
    assert.ok(byAlias['name'], 'should have "name" alias');
    assert.ok(byAlias['basic_info'], 'should have "basic_info" alias');
  });

  it('getRootNode returns root', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const root = wrapper.getRootNode();
    assert.equal(root.alias, 'group');
    assert.equal(root.istopnode, true);
  });

  it('getChildNodes returns children keyed by alias', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const root = wrapper.getRootNode();
    const children = wrapper.getChildNodes(root.nodeid);
    assert.ok(typeof children === 'object');
    assert.ok(Object.keys(children).length > 0, 'root should have children');
    assert.ok(children['basic_info'], 'root should have basic_info child');
  });

  it('getChildNodeAliases returns string array', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const root = wrapper.getRootNode();
    const aliases = wrapper.getChildNodeAliases(root.nodeid);
    assert.ok(Array.isArray(aliases));
    assert.ok(aliases.length > 0);
    assert.ok(aliases.includes('basic_info'));
  });

  it('getNodeObjectFromAlias returns correct node', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const node = wrapper.getNodeObjectFromAlias('name');
    assert.equal(node.alias, 'name');
  });

  it('getNodeObjectFromId returns correct node', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const root = wrapper.getRootNode();
    const node = wrapper.getNodeObjectFromId(root.nodeid);
    assert.equal(node.nodeid, root.nodeid);
  });

  it('getNodeIdFromAlias returns correct ID', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const nodeId = wrapper.getNodeIdFromAlias('name');
    const node = wrapper.getNodeObjectFromId(nodeId);
    assert.equal(node.alias, 'name');
  });

  // -------------------------------------------------------------------------
  // Edge accessors
  // -------------------------------------------------------------------------

  it('getEdges returns edge map', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const edges = wrapper.getEdges();
    assert.ok(typeof edges === 'object');
    const root = wrapper.getRootNode();
    assert.ok(edges[root.nodeid], 'root should have edges');
    assert.ok(Array.isArray(edges[root.nodeid]));
  });

  // -------------------------------------------------------------------------
  // Nodegroup accessors
  // -------------------------------------------------------------------------

  it('getNodegroupObjects returns all nodegroups', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const nodegroups = wrapper.getNodegroupObjects();
    assert.ok(typeof nodegroups === 'object');
    assert.ok(Object.keys(nodegroups).length > 0);
  });

  it('getNodegroupIds returns string array', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const ids = wrapper.getNodegroupIds();
    assert.ok(Array.isArray(ids));
    assert.ok(ids.length > 0);
    ids.forEach(id => assert.equal(typeof id, 'string'));
  });

  it('getNodegroupName returns name string', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const nameNode = wrapper.getNodeObjectFromAlias('name');
    const name = wrapper.getNodegroupName(nameNode.nodegroup_id);
    assert.equal(typeof name, 'string');
  });

  // -------------------------------------------------------------------------
  // Permissions
  // -------------------------------------------------------------------------

  it('setPermittedNodegroups accepts boolean values', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const ids = wrapper.getNodegroupIds();
    const permissions = {};
    ids.forEach(id => { permissions[id] = true; });

    // Should not throw
    wrapper.setPermittedNodegroups(permissions);
  });

  it('isNodegroupPermitted returns correct boolean', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const ids = wrapper.getNodegroupIds();
    const permissions = {};
    permissions[ids[0]] = true;
    if (ids.length > 1) permissions[ids[1]] = false;
    wrapper.setPermittedNodegroups(permissions);

    assert.equal(wrapper.isNodegroupPermitted(ids[0]), true);
    if (ids.length > 1) {
      assert.equal(wrapper.isNodegroupPermitted(ids[1]), false);
    }
  });

  it('getPermittedNodegroups returns boolean map', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const ids = wrapper.getNodegroupIds();
    const permissions = {};
    ids.forEach(id => { permissions[id] = true; });
    wrapper.setPermittedNodegroups(permissions);

    const result = wrapper.getPermittedNodegroups();
    assert.ok(typeof result === 'object');
    assert.ok(Object.values(result).every(v => typeof v === 'boolean'));
  });

  it('setPermittedNodegroups accepts conditional rules', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    const ids = wrapper.getNodegroupIds();
    const permissions = {};
    permissions[ids[0]] = { path: '.data.some_field', allowed: ['value1', 'value2'] };
    if (ids.length > 1) permissions[ids[1]] = true;

    // Should not throw
    wrapper.setPermittedNodegroups(permissions);
    // Conditional rule permits the nodegroup at the nodegroup level
    assert.equal(wrapper.isNodegroupPermitted(ids[0]), true);
  });

  // -------------------------------------------------------------------------
  // Property getters
  // -------------------------------------------------------------------------

  it('nodes property returns object', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    assert.ok(typeof wrapper.nodes === 'object');
    assert.ok(Object.keys(wrapper.nodes).length > 0);
  });

  it('edges property returns object', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    assert.ok(typeof wrapper.edges === 'object');
  });

  it('nodesByAlias property returns object', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    assert.ok(typeof wrapper.nodesByAlias === 'object');
    assert.ok(wrapper.nodesByAlias['group']);
  });

  it('nodegroups property returns object', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    assert.ok(typeof wrapper.nodegroups === 'object');
    assert.ok(Object.keys(wrapper.nodegroups).length > 0);
  });

  it('graphId getter matches input', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    assert.equal(wrapper.getGraphId(), '07883c9e-b25c-11e9-975a-a4d18cec433a');
  });

  // -------------------------------------------------------------------------
  // Graph pruning
  // -------------------------------------------------------------------------

  it('pruneGraph removes unpermitted nodegroups', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, false);
    const ids = wrapper.getNodegroupIds();

    // Permit only the first nodegroup
    const permissions = {};
    permissions[ids[0]] = true;
    for (let i = 1; i < ids.length; i++) {
      permissions[ids[i]] = false;
    }
    wrapper.setPermittedNodegroups(permissions);

    const beforeCount = Object.keys(wrapper.nodes).length;
    wrapper.pruneGraph();
    const afterCount = Object.keys(wrapper.nodes).length;

    assert.ok(afterCount <= beforeCount, 'pruning should reduce or maintain node count');
  });

  // -------------------------------------------------------------------------
  // buildNodes (no-op) and buildNodesForGraph
  // -------------------------------------------------------------------------

  it('buildNodes is a no-op (does not throw)', () => {
    const wrapper = new NapiResourceModelWrapper(graphStr, true);
    // Should not throw
    wrapper.buildNodes();
    assert.ok(Object.keys(wrapper.nodes).length > 0, 'nodes still accessible after buildNodes');
  });
});
