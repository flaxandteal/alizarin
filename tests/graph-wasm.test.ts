import { describe, it, expect, beforeAll } from 'vitest';
import { StaticGraphMeta, StaticNode, StaticTranslatableString } from '../pkg/alizarin';
import { initWasmForTests } from './wasm-init';

describe('WASM Graph Module', () => {
  beforeAll(async () => {
    // Initialize WASM module with custom loader for tests
    await initWasmForTests();
  });

  describe('StaticGraphMeta', () => {
    it('should create a graph meta instance from JSON data', () => {
      const graphData = {
        graphid: 'test-graph-123',
        author: 'Test Author',
        isresource: true,
        name: { en: 'Test Graph' },
        description: { en: 'A test graph' },
        relatable_resource_model_ids: [],
        resource_2_resource_constraints: [],
        extra_fields: {}
      };

      const graphMeta = new StaticGraphMeta(graphData);
      
      expect(graphMeta).toBeDefined();
      expect(graphMeta.graphid).toBe('test-graph-123');
    });

    it('should get and set author', () => {
      const graphMeta = new StaticGraphMeta({ 
        graphid: 'test-123',
        relatable_resource_model_ids: [],
        resource_2_resource_constraints: [],
        extra_fields: {}
      });
      
      expect(graphMeta.getAuthor()).toBeNull();

      graphMeta.setAuthor('New Author');
      expect(graphMeta.getAuthor()).toBe('New Author');
      
      graphMeta.setAuthor(null);
      expect(graphMeta.getAuthor()).toBeNull(); // None is null, as on NAPI
    });

    it('should get and set isresource', () => {
      const graphMeta = new StaticGraphMeta({ 
        graphid: 'test-123',
        relatable_resource_model_ids: [],
        resource_2_resource_constraints: [],
        extra_fields: {}
      });
      
      expect(graphMeta.getIsResource()).toBeNull();
      
      graphMeta.setIsResource(true);
      expect(graphMeta.getIsResource()).toBe(true);
      
      graphMeta.setIsResource(false);
      expect(graphMeta.getIsResource()).toBe(false);
    });

    it('should serialize to JSON', () => {
      const graphData = {
        graphid: 'test-graph-123',
        author: 'Test Author',
        isresource: true,
        name: { en: 'Test Graph' },
        description: { en: 'A test graph' },
        relatable_resource_model_ids: [],
        resource_2_resource_constraints: []
      };

      const graphMeta = new StaticGraphMeta(graphData);
      const json = graphMeta.toJSON();

      expect(json).toBeDefined();
      expect(json.graphid).toBe('test-graph-123');
      expect(json.author).toBe('Test Author');
      expect(json.isresource).toBe(true);
    });
  });

  describe('StaticNode', () => {
    const createTestNode = (overrides = {}) => ({
      nodeid: 'node-123',
      name: 'Test Node',
      alias: 'test_node',
      datatype: 'string',
      istopnode: true,
      description: 'Test node description',
      exportable: true,
      fieldname: 'TestField',
      graph_id: 'test-graph-123',
      hascustomalias: false,
      is_collector: false,
      isrequired: false,
      issearchable: true,
      nodegroup_id: 'ng-1',
      sortorder: 1,
      config: {},
      ontologyclass: null,
      parentproperty: null,
      sourcebranchpublication_id: null,
      ...overrides
    });

    it('should create a node instance from JSON data', () => {
      const nodeData = createTestNode();
      const node = new StaticNode(nodeData);
      
      expect(node).toBeDefined();
      expect(node.nodeid).toBe('node-123');
      expect(node.name).toBe('Test Node');
      expect(node.datatype).toBe('string');
      expect(node.istopnode).toBe(true);
    });

    it('should get and set node properties', () => {
      const node = new StaticNode(createTestNode());
      
      // Test alias
      expect(node.alias).toBe('test_node');
      node.alias = 'new_alias';
      expect(node.alias).toBe('new_alias');
      
      // Test description
      expect(node.description.toString()).toBe('Test node description');
      node.description = new StaticTranslatableString('New description');
      expect(node.description.toString()).toBe('New description');
      node.description = undefined;
      expect(node.description).toBeNull(); // None is null, as on NAPI
      
      // Test boolean properties
      expect(node.exportable).toBe(true);
      node.exportable = false;
      expect(node.exportable).toBe(false);
      
      // Test sortorder
      expect(node.sortorder).toBe(1);
      node.sortorder = 5;
      expect(node.sortorder).toBe(5);
    });

    it('should copy a node', () => {
      const node = new StaticNode(createTestNode());
      const copy = node.copy();
      
      expect(copy).toBeDefined();
      expect(copy).not.toBe(node); // Different instances
      expect(copy.nodeid).toBe(node.nodeid);
      expect(copy.name).toBe(node.name);
      expect(copy.datatype).toBe(node.datatype);
    });

    it('should serialize to JSON', () => {
      const nodeData = createTestNode();
      const node = new StaticNode(nodeData);
      const json = node.toJSON();

      expect(json).toBeDefined();
      expect(json.nodeid).toBe('node-123');
      expect(json.name).toBe('Test Node');
      expect(json.alias).toBe('test_node');
      expect(json.datatype).toBe('string');
    });

    it('should get and set config', () => {
      const node = new StaticNode(createTestNode());
      
      // Get initial empty config (using property getter, not method)
      const initialConfig = node.config;
      expect(initialConfig).toEqual({}); // Rust HashMap becomes JS object
      
      // Set new config
      const newConfig = {
        placeholder: 'Enter text',
        maxlength: 255,
        options: ['a', 'b', 'c']
      };
      
      node.config = newConfig;

      // Get config via property access
      const retrievedConfig = node.config;
      expect(retrievedConfig).toEqual(newConfig);
    });

    describe('compare', () => {
      it('gives same for an object and itself', () => {
        const nodeData = createTestNode();
        const result = StaticNode.compare(nodeData, nodeData);
        expect(result).toBe('same');
      });

      it('gives same for identical nodes with the same ids', () => {
        const nodeData1 = createTestNode();
        const nodeData2 = { ...createTestNode() };

        const result = StaticNode.compare(nodeData1, nodeData2);
        expect(result).toBe('same');
      });

      it('gives nodeid_conflict for identical nodes except nodeid', () => {
        const nodeData1 = createTestNode();
        const nodeData2 = createTestNode({ nodeid: 'node-456' });

        const result = StaticNode.compare(nodeData1, nodeData2);
        expect(result).toBe('nodeid_conflict');
      });

      it('gives nodegroup_id_conflict for identical nodes except nodegroup_id', () => {
        const nodeData1 = createTestNode();
        const nodeData2 = createTestNode({ nodegroup_id: 'ng-2' });

        const result = StaticNode.compare(nodeData1, nodeData2);
        expect(result).toBe('nodegroup_id_conflict');
      });

      it('gives graph_id_conflict for identical nodes except graph_id', () => {
        const nodeData1 = createTestNode();
        const nodeData2 = createTestNode({ graph_id: 'test-graph-456' });

        const result = StaticNode.compare(nodeData1, nodeData2);
        expect(result).toBe('graph_id_conflict');
      });

      it('gives different for different nodes', () => {
        const nodeData1 = createTestNode();
        const nodeData2 = createTestNode({ 
          name: 'Different Node',
          datatype: 'number' 
        });
        
        const result = StaticNode.compare(nodeData1, nodeData2);
        expect(result).toBe('different');
      });

      it('gives same for identical nodes whose ids are all unset', () => {
        const nodeData1 = createTestNode({ 
          nodeid: '',
          nodegroup_id: null,
          graph_id: '' 
        });
        const nodeData2 = createTestNode({ 
          nodeid: '',
          nodegroup_id: null,
          graph_id: '' 
        });
        
        const result = StaticNode.compare(nodeData1, nodeData2);
        expect(result).toBe('same'); // unset on both sides: equal
      });
    });
  });
});
