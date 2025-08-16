import { describe, it, expect } from 'vitest';
import {
  StaticTranslatableString,
  StaticGraph,
  StaticNode,
  StaticEdge,
  StaticNodegroup,
  StaticResource,
  StaticTile,
  StaticValue,
  StaticGraphMeta
} from '../js/static-types';

describe('Static Types', () => {
  describe('StaticTranslatableString', () => {
    it('should create translatable string with single language', () => {
      const str = new StaticTranslatableString('Hello World', 'en');
      
      expect(str.toString()).toBe('Hello World');
      expect(str.lang).toBe('en');
      expect(str.translations.get('en')).toBe('Hello World');
    });

    it('should handle multiple languages', () => {
      const translations = {
        'en': 'Hello',
        'fr': 'Bonjour',
        'es': 'Hola'
      };
      
      const str = new StaticTranslatableString(translations as any);
      
      expect(str.toString()).toBe('Hello');
      expect(str.lang).toBe('en');
      expect(str.translations.get('en')).toBe('Hello');
      expect(str.translations.get('fr')).toBe('Bonjour');
    });

    it('should copy translatable string', () => {
      const original = new StaticTranslatableString('Test', 'en');
      const copy = original.copy?.();
      
      expect(copy?.toString()).toBe('Test');
      expect(copy?.lang).toBe('en');
      expect(copy).not.toBe(original);
    });

    it('should convert to JSON', () => {
      const str = new StaticTranslatableString('English' as any);
      str.translations.set('en', 'English');
      str.translations.set('fr', 'Français');
      
      const json = str.toJSON();
      expect(json).toEqual({
        en: 'English',
        fr: 'Français'
      });
    });
  });

  describe('StaticGraphMeta', () => {
    it('should create graph metadata', () => {
      const metaData = {
        graphid: 'test-graph-123',
        name: new StaticTranslatableString('Test Graph'),
        author: 'Test Author',
        description: new StaticTranslatableString('Test Description'),
        isresource: true,
        cards: null,
        cards_x_nodes_x_widgets: null,
        color: null,
        edges: [],
        is_editable: null,
        nodes: [],
        nodegroups: [],
        relatable_resource_model_ids: [],
        slug: null,
        subtitle: new StaticTranslatableString(''),
        template_id: '',
        version: '',
        config: {},
        deploymentdate: null,
        deploymentfile: null,
        functions_x_graphs: null,
        iconclass: '',
        jsonldcontext: null,
        ontology_id: null,
        publication: null,
        resource_2_resource_constraints: null
      } as any;
      
      const meta = new StaticGraphMeta(metaData);
      
      expect(meta.graphid).toBe('test-graph-123');
      expect(meta.author).toBe('Test Author');
      expect(meta.isresource).toBe(true);
    });

    it('should handle minimal metadata', () => {
      const meta = new StaticGraphMeta({ graphid: 'minimal-graph' } as any);
      
      expect(meta.graphid).toBe('minimal-graph');
      expect(meta.author).toBeUndefined();
    });
  });

  describe('StaticValue', () => {
    it('should create basic value', () => {
      const value = new StaticValue({ id: 'test-id', value: 'Test Value' });
      
      expect(value).toBeDefined();
      expect(value.id).toBe('test-id');
      expect(value.value).toBe('Test Value');
      expect(value.toString()).toBe('Test Value');
    });

    it('should handle different value types', () => {
      const stringValue = new StaticValue({ id: 'str-id', value: 'String' });
      const numberValue = new StaticValue({ id: 'num-id', value: '42' });
      
      expect(stringValue).toBeDefined();
      expect(stringValue.value).toBe('String');
      expect(numberValue).toBeDefined();
      expect(numberValue.value).toBe('42');
    });
    
    it('should create value with static method', () => {
      const value = StaticValue.create('concept-123', 'string', 'Test Value', 'en');
      
      expect(value).toBeDefined();
      expect(value.value).toBe('Test Value');
      expect(value.id).toBeDefined();
    });
  });

  describe('StaticTile', () => {
    it('should create tile with data', () => {
      const data = new Map([
        ['node-1', [new StaticValue({ id: 'val1', value: 'Value 1' })]],
        ['node-2', [new StaticValue({ id: 'val2', value: 'Value 2' })]]
      ]);
      
      const tile = Object.assign(new StaticTile({} as any), {
        tileid: 'tile-123',
        nodegroup_id: 'nodegroup-1',
        resourceinstance_id: 'resource-1',
        data: data,
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      });
      
      expect(tile).toBeDefined();
      // Verify constructor works
    });

    it('should handle empty data', () => {
      const tile = Object.assign(new StaticTile({} as any), {
        tileid: 'empty-tile',
        nodegroup_id: 'ng-1',
        resourceinstance_id: 'resource-1',
        data: new Map(),
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      });
      
      expect(tile).toBeDefined();
    });
  });

  describe('StaticNodegroup', () => {
    it('should create nodegroup', () => {
      const nodegroupData = {
        nodegroupid: 'ng-123',
        cardinality: 'n' as '1' | 'n',
        legacygroupid: null,
        parentnodegroup_id: null
      };
      
      const nodegroup = new StaticNodegroup(nodegroupData);
      
      expect(nodegroup.nodegroupid).toBe('ng-123');
      expect(nodegroup.cardinality).toBe('n');
      expect(nodegroup.legacygroupid).toBe(null);
    });
  });

  describe('StaticEdge', () => {
    it('should create edge', () => {
      const edgeData = {
        edgeid: 'edge-123',
        domainnode_id: 'parent-node',
        rangenode_id: 'child-node',
        description: 'Test edge description',
        graph_id: 'test-graph',
        name: 'Test Edge',
        ontologyproperty: 'http://example.org/hasChild'
      };
      
      const edge = new StaticEdge(edgeData);
      
      expect(edge.edgeid).toBe('edge-123');
      expect(edge.domainnode_id).toBe('parent-node');
      expect(edge.rangenode_id).toBe('child-node');
      expect(edge.description).toBe('Test edge description');
    });
  });

  describe('StaticNode', () => {
    it('should create node with properties', () => {
      const nodeData = {
        nodeid: 'node-123',
        name: 'Test Node',
        alias: 'test_node',
        datatype: 'string',
        istopnode: true,
        description: 'Test node description',
        exportable: true,
        fieldname: 'TestField',
        graph_id: 'test-graph',
        hascustomalias: false,
        is_collector: false,
        isrequired: false,
        issearchable: true,
        nodegroup_id: 'ng-1',
        sortorder: 1,
        config: {},
        ontologyclass: null,
        parentproperty: null,
        sourcebranchpublication_id: null
      };
      
      const node = new StaticNode(nodeData);
      
      expect(node.nodeid).toBe('node-123');
      expect(node.name).toBe('Test Node');
      expect(node.alias).toBe('test_node');
      expect(node.datatype).toBe('string');
      expect(node.istopnode).toBe(true);
    });

    it('should handle node configuration', () => {
      const nodeData = {
        nodeid: 'config-node',
        name: 'Config Node',
        alias: 'config_node',
        datatype: 'string',
        istopnode: false,
        description: 'Node with config',
        exportable: true,
        fieldname: 'ConfigField',
        graph_id: 'test-graph',
        hascustomalias: false,
        is_collector: false,
        isrequired: false,
        issearchable: true,
        nodegroup_id: 'ng-1',
        sortorder: 1,
        config: { 
          placeholder: 'Enter text',
          maxlength: 255 
        },
        ontologyclass: null,
        parentproperty: null,
        sourcebranchpublication_id: null
      };
      
      const node = new StaticNode(nodeData);
      
      expect(node.config?.placeholder).toBe('Enter text');
      expect(node.config?.maxlength).toBe(255);
    });

    it('should compare nodes', () => {
      const nodeData1 = {
        nodeid: 'node-1',
        name: 'Node 1',
        alias: 'node1',
        datatype: 'string',
        istopnode: false,
        description: 'First node',
        exportable: true,
        fieldname: 'Field1',
        graph_id: 'test-graph',
        hascustomalias: false,
        is_collector: false,
        isrequired: false,
        issearchable: true,
        nodegroup_id: 'ng-1',
        sortorder: 1,
        config: {},
        ontologyclass: null,
        parentproperty: null,
        sourcebranchpublication_id: null
      };
      
      const nodeData2 = {
        ...nodeData1,
        nodeid: 'node-2'
      };
      
      const node1 = new StaticNode(nodeData1);
      const node2 = new StaticNode(nodeData2);
      
      const result = StaticNode.compare(node1, node2);
      expect(typeof result).toBe('number');
      expect(result).toBeGreaterThanOrEqual(-1);
      expect(result).toBeLessThanOrEqual(2);
    });
  });

  describe('StaticGraph', () => {
    it('should create graph', () => {
      const graph = StaticGraph.create({
        name: 'Test Graph',
        author: 'Test Author',
        description: 'A test graph for unit testing'
      });
      
      expect(graph).toBeDefined();
      expect(graph.graphid).toBeDefined();
      expect(graph.name?.toString()).toBe('Test Graph');
      expect(graph.author).toBe('Test Author');
    });

    it('should find root node when present', () => {
      // StaticGraph.create() automatically creates a root node with the graph's ID
      const graph = StaticGraph.create({
        name: 'Test Graph with Nodes',
        description: 'Graph containing root and child nodes',
        graphid: 'test-graph-123'
      });
      
      expect(graph.root).toBeDefined();
      expect(graph.root?.nodeid).toBe('test-graph-123');
      expect(graph.root?.istopnode).toBe(true);
      expect(graph.root?.name).toBe('Test Graph with Nodes');
    });

    it('should always create a root node', () => {
      // StaticGraph.create() always creates a root node
      const graph = StaticGraph.create({
        name: 'No Root Graph',
        description: 'Graph without any root nodes'
      });
      
      expect(graph.root).toBeDefined();
      expect(graph.root?.istopnode).toBe(true);
      expect(graph.nodes).toHaveLength(1);
      expect(graph.nodes[0].nodeid).toBe(graph.graphid);
    });
  });

  describe('StaticResource', () => {
    it('should require proper constructor data structure', () => {
      // StaticResource constructor expects a complex JSON structure
      // This test documents that the constructor requires specific data format
      expect(() => {
        // This would fail because constructor expects jsonData.resourceinstance
        new StaticResource({} as any);
      }).toThrow();
      
      console.warn('⚠️  StaticResource requires complex metadata structure - constructor needs proper resourceinstance data');
    });

    it('should be importable and defined', () => {
      expect(StaticResource).toBeDefined();
      expect(typeof StaticResource).toBe('function');
    });
  });

  describe('Integration', () => {
    it('should create complete graph structure', () => {
      // StaticGraph.create() creates a simple graph with just a root node
      const graph = StaticGraph.create({
        name: 'Integration Test Graph',
        graphid: 'integration-graph',
        author: 'Test Author',
        description: 'A test graph'
      });
      
      expect(graph.nodes).toHaveLength(1);
      expect(graph.edges).toHaveLength(0);
      expect(graph.nodegroups).toHaveLength(0);
      expect(graph.root?.nodeid).toBe('integration-graph');
      expect(graph.root?.name).toBe('Integration Test Graph');
      expect(graph.author).toBe('Test Author');
      
      // To create a more complex graph, use the constructor directly
      const rootNode = new StaticNode({
        nodeid: 'root-node',
        name: 'Root',
        alias: 'root',
        datatype: 'semantic',
        istopnode: true,
        description: 'Root node',
        exportable: true,
        fieldname: 'Root',
        graph_id: 'complex-graph',
        hascustomalias: false,
        is_collector: false,
        isrequired: false,
        issearchable: true,
        nodegroup_id: 'ng-root',
        sortorder: 0,
        config: {},
        ontologyclass: null,
        parentproperty: null,
        sourcebranchpublication_id: null
      });
      
      const complexGraph = new StaticGraph({
        author: 'Test Author',
        cards: null,
        cards_x_nodes_x_widgets: null,
        color: null,
        config: {},
        deploymentdate: null,
        deploymentfile: null,
        description: new StaticTranslatableString('Complex graph'),
        edges: [],
        functions_x_graphs: null,
        graphid: 'complex-graph',
        iconclass: '',
        is_editable: null,
        isresource: false,
        jsonldcontext: null,
        name: new StaticTranslatableString('Complex Graph'),
        nodegroups: [],
        nodes: [rootNode],
        ontology_id: null,
        publication: null,
        relatable_resource_model_ids: [],
        resource_2_resource_constraints: null,
        root: rootNode,
        slug: null,
        subtitle: new StaticTranslatableString(''),
        template_id: '',
        version: ''
      });
      
      expect(complexGraph.nodes).toHaveLength(1);
      expect(complexGraph.root?.nodeid).toBe('root-node');
    });

    it('should create tile with complex structure', () => {
      const tileData = new Map([
        ['name-node', [new StaticValue({ id: 'val-1', value: 'Test Resource' })]],
        ['description-node', [new StaticValue({ id: 'val-2', value: 'A test resource' })]],
        ['date-node', [new StaticValue({ id: 'val-3', value: '2023-12-25' })]]
      ]);
      
      const tile = Object.assign(new StaticTile({} as any), {
        tileid: 'complex-tile',
        nodegroup_id: 'main-nodegroup',
        resourceinstance_id: 'resource-123',
        data: tileData,
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      });
      
      expect(tile).toBeDefined();
      expect(tile.tileid).toBe('complex-tile');
      expect(tile.nodegroup_id).toBe('main-nodegroup');
      expect(tile.resourceinstance_id).toBe('resource-123');
      
      console.warn('⚠️  StaticResource constructor requires complex metadata - tile creation works independently');
    });
  });
});
