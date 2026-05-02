import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';

import { NapiStaticGraph, NapiStaticResourceRegistry, NapiResourceModelWrapper, NapiResourceInstanceWrapper } from '../index.js';

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

// =============================================================================
// NapiPseudoValue property names
// Regression: NAPI once exposed tileDataJson instead of tileData,
// and was missing the .node getter.
// =============================================================================

describe('NapiPseudoValue property names', () => {
  // Register the graph and load a resource with tiles
  const groupJson = fs.readFileSync(
    path.join(TEST_DATA, 'data', 'models', 'Group.json'),
    'utf-8'
  );
  const graphData = JSON.parse(groupJson);
  const graphStr = JSON.stringify(graphData.graph[0]);

  const resourceJson = fs.readFileSync(
    path.join(TEST_DATA, 'definitions', 'resources', '_07883c9e-b25c-11e9-975a-a4d18cec433a.json'),
    'utf-8'
  );
  const resourceData = JSON.parse(resourceJson);
  const resource = resourceData.business_data.resources[0];

  // Create model wrapper (registers graph)
  const _modelWrapper = new NapiResourceModelWrapper(graphStr, true);

  // Create instance wrapper and load tiles
  const instanceWrapper = new NapiResourceInstanceWrapper(resource.resourceinstance.graph_id);
  instanceWrapper.loadTilesFromResource(resource);

  // Get a PseudoList and PseudoValue
  const pseudoList = instanceWrapper.getValuesAtPath('basic_info.name');
  const pseudoValue = pseudoList.getValue(0);

  it('exposes tileData (not tileDataJson)', () => {
    assert.ok(pseudoValue, 'pseudoValue should exist');
    // Must be .tileData, not .tileDataJson
    assert.notEqual(pseudoValue.tileData, undefined, 'tileData should be defined');
  });

  it('exposes tileId', () => {
    const tileId = pseudoValue.tileId;
    if (tileId != null) {
      assert.equal(typeof tileId, 'string');
    }
  });

  it('exposes nodeId', () => {
    assert.equal(typeof pseudoValue.nodeId, 'string');
  });

  it('exposes nodeAlias', () => {
    if (pseudoValue.nodeAlias != null) {
      assert.equal(typeof pseudoValue.nodeAlias, 'string');
    }
  });

  it('exposes datatype', () => {
    assert.equal(typeof pseudoValue.datatype, 'string');
  });

  it('exposes node as object with datatype', () => {
    const node = pseudoValue.node;
    assert.ok(node, 'node should be defined');
    assert.equal(typeof node, 'object');
    assert.ok(node.datatype, 'node.datatype should be defined');
    assert.equal(typeof node.datatype, 'string');
  });

  it('exposes nodegroupId', () => {
    if (pseudoValue.nodegroupId != null) {
      assert.equal(typeof pseudoValue.nodegroupId, 'string');
    }
  });

  it('exposes isCollector (boolean)', () => {
    assert.equal(typeof pseudoValue.isCollector, 'boolean');
  });

  it('exposes sortorder', () => {
    // sortorder can be number or null — just verify it doesn't throw
    const _sortorder = pseudoValue.sortorder;
    assert.ok(true);
  });

  it('exposes valueLoaded (boolean)', () => {
    assert.equal(typeof pseudoValue.valueLoaded, 'boolean');
  });
});

// =============================================================================
// NapiResourceInstanceWrapper methods
// Regression: NAPI was missing setTileDataForNode, tilesLoaded, setLazy.
// =============================================================================

describe('NapiResourceInstanceWrapper methods', () => {
  const groupJson = fs.readFileSync(
    path.join(TEST_DATA, 'data', 'models', 'Group.json'),
    'utf-8'
  );
  const graphData = JSON.parse(groupJson);
  const graphStr = JSON.stringify(graphData.graph[0]);
  const graphId = graphData.graph[0].graphid;

  const resourceJson = fs.readFileSync(
    path.join(TEST_DATA, 'definitions', 'resources', '_07883c9e-b25c-11e9-975a-a4d18cec433a.json'),
    'utf-8'
  );
  const resourceData = JSON.parse(resourceJson);
  const resource = resourceData.business_data.resources[0];

  // Ensure graph is registered
  const _modelWrapper = new NapiResourceModelWrapper(graphStr, true);

  it('has setTileDataForNode method', () => {
    const wrapper = new NapiResourceInstanceWrapper(graphId);
    wrapper.loadTilesFromResource(resource);
    assert.equal(typeof wrapper.setTileDataForNode, 'function');
  });

  it('setTileDataForNode mutates tile data', () => {
    const wrapper = new NapiResourceInstanceWrapper(graphId);
    wrapper.loadTilesFromResource(resource);

    const tileIds = wrapper.getAllTileIds();
    assert.ok(tileIds.length > 0, 'should have tiles');
    const tileId = tileIds[0];

    // Set data and verify it was stored
    const result = wrapper.setTileDataForNode(tileId, 'test-node-id', [{ name: 'test.jpg' }]);
    assert.equal(result, true);

    // Read it back
    const data = wrapper.getTileData(tileId, 'test-node-id');
    assert.deepEqual(data, [{ name: 'test.jpg' }]);
  });

  it('has tilesLoaded method', () => {
    const wrapper = new NapiResourceInstanceWrapper(graphId);
    assert.equal(typeof wrapper.tilesLoaded, 'function');
    assert.equal(wrapper.tilesLoaded(), false);

    wrapper.loadTilesFromResource(resource);
    assert.equal(wrapper.tilesLoaded(), true);
  });

  it('has toJson method', () => {
    const wrapper = new NapiResourceInstanceWrapper(graphId);
    wrapper.loadTilesFromResource(resource);
    assert.equal(typeof wrapper.toJson, 'function');
    // toJson requires populate() first; just verify the method exists
    // and that calling it without populate gives a meaningful error
    assert.throws(() => wrapper.toJson(), /populate/i);
  });

  it('has setLazy method', () => {
    const wrapper = new NapiResourceInstanceWrapper(graphId);
    assert.equal(typeof wrapper.setLazy, 'function');
    // Should not throw
    wrapper.setLazy(true);
  });

  it('exportTilesJson returns valid JSON array of tiles', () => {
    const wrapper = new NapiResourceInstanceWrapper(graphId);
    wrapper.loadTilesFromResource(resource);

    const json = wrapper.exportTilesJson();
    assert.equal(typeof json, 'string');

    const tiles = JSON.parse(json);
    assert.ok(Array.isArray(tiles), 'parsed result should be an array');
    assert.ok(tiles.length > 0, 'should have tiles');

    // Each tile should have tileid and data
    for (const tile of tiles) {
      assert.ok(tile.tileid || tile.tileid === null, 'tile should have tileid field');
      assert.ok('data' in tile, 'tile should have data field');
    }
  });

  it('exportTilesJson reflects setTileDataForNode mutations', () => {
    const wrapper = new NapiResourceInstanceWrapper(graphId);
    wrapper.loadTilesFromResource(resource);

    const tileIds = wrapper.getAllTileIds();
    const tileId = tileIds[0];

    // Mutate a tile
    wrapper.setTileDataForNode(tileId, 'test-node-id', [{ name: 'mutated.jpg' }]);

    // Export and verify mutation is present
    const tiles = JSON.parse(wrapper.exportTilesJson());
    const mutatedTile = tiles.find(t => t.tileid === tileId);
    assert.ok(mutatedTile, 'mutated tile should be in export');
    assert.deepEqual(mutatedTile.data['test-node-id'], [{ name: 'mutated.jpg' }]);
  });

  it('exportTilesJson returns empty array when no tiles loaded', () => {
    const wrapper = new NapiResourceInstanceWrapper(graphId);
    const json = wrapper.exportTilesJson();
    const tiles = JSON.parse(json);
    assert.ok(Array.isArray(tiles), 'should be an array');
    assert.equal(tiles.length, 0, 'should be empty');
  });
});
