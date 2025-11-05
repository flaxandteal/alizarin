import { describe, it, expect, beforeAll } from 'vitest';
import {
  StaticTranslatableString,
  StaticGraph,
  createStaticGraph,
  StaticNode,
  StaticEdge,
  StaticNodegroup,
  StaticResource,
  StaticTile,
  StaticValue,
  StaticGraphMeta,
  StaticConstraint,
  StaticCard,
  StaticCardsXNodesXWidgets,
  StaticFunctionsXGraphs,
  StaticPublication
} from '../js/static-types';
import { initWasmForTests } from './wasm-init';

describe('Static Types', () => {
  beforeAll(async () => {
    // Initialize WASM module for tests
    await initWasmForTests();
  });
  describe('StaticTranslatableString', () => {
    it('should create translatable string with single language', () => {
      const str = new StaticTranslatableString('Hello World', 'en');

      expect(str.toString()).toBe('Hello World');
      expect(str.lang).toBe('en');
      expect(str.translations.en).toBe('Hello World');
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
      expect(str.translations.en).toBe('Hello');
      expect(str.translations.fr).toBe('Bonjour');
    });

    it('should copy translatable string', () => {
      const original = new StaticTranslatableString('Test', 'en');
      const copy = original.copy?.();

      expect(copy?.toString()).toBe('Test');
      expect(copy?.lang).toBe('en');
      expect(copy).not.toBe(original);
    });

    it('should convert to JSON', () => {
      const str = new StaticTranslatableString({
        en: 'English',
        fr: 'Français'
      } as any);

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
        name: { en: 'Test Graph' },  // Plain object, not StaticTranslatableString
        author: 'Test Author',
        description: { en: 'Test Description' },  // Plain object
        isresource: true,
        cards: null,
        cards_x_nodes_x_widgets: null,
        color: null,
        edges: null,  // Counts, not arrays
        is_editable: null,
        nodes: null,  // Counts, not arrays
        nodegroups: null,  // Count, not array
        relatable_resource_model_ids: [],
        slug: null,
        subtitle: { en: '' },  // Plain object
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
        resource_2_resource_constraints: []
      } as any;
      const meta = new StaticGraphMeta(metaData);

      expect(meta.graphid).toBe('test-graph-123');
      expect(meta.getAuthor()).toBe('Test Author');
      expect(meta.getIsResource()).toBe(true);
    });

    it('should handle minimal metadata', () => {
      const meta = new StaticGraphMeta({ graphid: 'minimal-graph' } as any);

      expect(meta.graphid).toBe('minimal-graph');
      expect(meta.getAuthor()).toBeUndefined();
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
    it('should create tile with basic properties', () => {
      const tile = new StaticTile({
        tileid: 'tile-123',
        nodegroup_id: 'nodegroup-456',
        resourceinstance_id: 'resource-789',
        data: {},
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      } as any);

      expect(tile.tileid).toBe('tile-123');
      expect(tile.nodegroup_id).toBe('nodegroup-456');
      expect(tile.resourceinstance_id).toBe('resource-789');
      expect(tile.data).toBeInstanceOf(Map);
      expect(tile.parenttile_id).toBe(null);
      expect(tile.provisionaledits).toBe(null);
      expect(tile.sortorder).toBe(null);
    });

    it('should convert object data to Map', () => {
      const objectData = {
        'node-1': 'value1',
        'node-2': 'value2',
        'node-3': 42
      };

      const tile = new StaticTile({
        tileid: 'tile-map',
        nodegroup_id: 'ng-1',
        resourceinstance_id: 'res-1',
        data: objectData,
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      } as any);

      expect(tile.data).toBeInstanceOf(Map);
      expect(tile.data.get('node-1')).toBe('value1');
      expect(tile.data.get('node-2')).toBe('value2');
      expect(tile.data.get('node-3')).toBe(42);
    });

    it('should handle Map data directly', () => {
      const mapData = new Map([
        ['node-a', 'valueA'],
        ['node-b', 'valueB']
      ]);

      const tile = new StaticTile({
        tileid: 'tile-direct-map',
        nodegroup_id: 'ng-2',
        resourceinstance_id: 'res-2',
        data: mapData,
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      } as any);

      expect(tile.data).toBeInstanceOf(Map);
      expect(tile.data.get('node-a')).toBe('valueA');
      expect(tile.data.get('node-b')).toBe('valueB');
    });

    it('should handle various data types in Map values', () => {
      const tile = new StaticTile({
        tileid: 'tile-types',
        nodegroup_id: 'ng-3',
        resourceinstance_id: 'res-3',
        data: {
          'string-node': 'text value',
          'number-node': 123,
          'boolean-node': true,
          'null-node': null,
          'array-node': ['item1', 'item2', 'item3'],
          'object-node': { key: 'value', nested: { deep: 'data' } }
        },
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      } as any);

      expect(tile.data.get('string-node')).toBe('text value');
      expect(tile.data.get('number-node')).toBe(123);
      expect(tile.data.get('boolean-node')).toBe(true);
      expect(tile.data.get('null-node')).toBe(null);
      expect(tile.data.get('array-node')).toEqual(['item1', 'item2', 'item3']);
      expect(tile.data.get('object-node')).toEqual({ key: 'value', nested: { deep: 'data' } });
    });

    it('should create tile without tileid and generate one with ensureId', () => {
      const tile = new StaticTile({
        tileid: null,
        nodegroup_id: 'ng-4',
        resourceinstance_id: 'res-4',
        data: {},
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      } as any);

      expect(tile.tileid).toBe(null);

      const generatedId = tile.ensureId();

      expect(generatedId).toBeDefined();
      expect(typeof generatedId).toBe('string');
      expect(generatedId.length).toBeGreaterThan(0);
      expect(tile.tileid).toBe(generatedId);
    });

    it('should not change existing tileid when calling ensureId', () => {
      const originalId = 'existing-tile-id';
      const tile = new StaticTile({
        tileid: originalId,
        nodegroup_id: 'ng-5',
        resourceinstance_id: 'res-5',
        data: {},
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      } as any);

      const returnedId = tile.ensureId();

      expect(returnedId).toBe(originalId);
      expect(tile.tileid).toBe(originalId);
    });

    it('should create child tile with parenttile_id', () => {
      const parentTile = new StaticTile({
        tileid: 'parent-tile-123',
        nodegroup_id: 'ng-parent',
        resourceinstance_id: 'res-1',
        data: {},
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      } as any);

      const childTile = new StaticTile({
        tileid: 'child-tile-456',
        nodegroup_id: 'ng-child',
        resourceinstance_id: 'res-1',
        data: {},
        parenttile_id: parentTile.tileid,
        provisionaledits: null,
        sortorder: 1
      } as any);

      expect(childTile.parenttile_id).toBe('parent-tile-123');
      expect(childTile.sortorder).toBe(1);
    });

    it('should handle provisionaledits', () => {
      const provisionalEdits = [
        { action: 'create', data: { some: 'data' } },
        { action: 'update', data: { other: 'data' } }
      ];

      const tile = new StaticTile({
        tileid: 'tile-provisional',
        nodegroup_id: 'ng-6',
        resourceinstance_id: 'res-6',
        data: {},
        parenttile_id: null,
        provisionaledits: provisionalEdits,
        sortorder: null
      } as any);

      expect(tile.provisionaledits).toEqual(provisionalEdits);
      expect(tile.provisionaledits).toHaveLength(2);
    });

    it('should handle empty data map', () => {
      const tile = new StaticTile({
        tileid: 'tile-empty',
        nodegroup_id: 'ng-7',
        resourceinstance_id: 'res-7',
        data: {},
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      } as any);

      expect(tile.data).toBeInstanceOf(Map);
      expect(tile.data.size).toBe(0);
    });

    it('should handle sortorder values', () => {
      const tile1 = new StaticTile({
        tileid: 'tile-sort-1',
        nodegroup_id: 'ng-8',
        resourceinstance_id: 'res-8',
        data: {},
        parenttile_id: null,
        provisionaledits: null,
        sortorder: 0
      } as any);

      const tile2 = new StaticTile({
        tileid: 'tile-sort-2',
        nodegroup_id: 'ng-8',
        resourceinstance_id: 'res-8',
        data: {},
        parenttile_id: null,
        provisionaledits: null,
        sortorder: 5
      } as any);

      expect(tile1.sortorder).toBe(0);
      expect(tile2.sortorder).toBe(5);
    });

    it('should handle complex nested data structures', () => {
      const complexData = {
        'node-values': [
          new StaticValue({ id: 'val1', value: 'First Value' }),
          new StaticValue({ id: 'val2', value: 'Second Value' })
        ],
        'node-object': { 'nested-key': 'nested-value' },
        'node-mixed': {
          strings: ['a', 'b', 'c'],
          numbers: [1, 2, 3],
          nested: {
            deep: {
              value: 'very nested'
            }
          }
        }
      };

      const tile = new StaticTile({
        tileid: 'tile-complex',
        nodegroup_id: 'ng-9',
        resourceinstance_id: 'res-9',
        data: complexData,
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      } as any);

      expect(tile.data).toBeInstanceOf(Map);
      const values = tile.data.get('node-values') as any[];
      expect(values).toHaveLength(2);
      expect(values[0].value).toBe('First Value');

      const nestedObj = tile.data.get('node-object') as any;
      expect(nestedObj['nested-key']).toBe('nested-value');

      const mixed = tile.data.get('node-mixed') as any;
      expect(mixed.strings).toEqual(['a', 'b', 'c']);
      expect(mixed.nested.deep.value).toBe('very nested');
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
      const graph = createStaticGraph({
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
      // createStaticGraph() automatically creates a root node with the graph's ID
      const graph = createStaticGraph({
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
      // createStaticGraph() always creates a root node
      const graph = createStaticGraph({
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
    });

    it('should be importable and defined', () => {
      expect(StaticResource).toBeDefined();
      expect(typeof StaticResource).toBe('function');
    });
  });

  describe('Integration', () => {
    it('should create complete graph structure', () => {
      // createStaticGraph() creates a simple graph with just a root node
      const graph = createStaticGraph({
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
      const tileData = {
        'name-node': [new StaticValue({ id: 'val-1', value: 'Test Resource' })],
        'description-node': [new StaticValue({ id: 'val-2', value: 'A test resource' })],
        'date-node': [new StaticValue({ id: 'val-3', value: '2023-12-25' })]
      };

      const tile = new StaticTile({
        tileid: 'complex-tile',
        nodegroup_id: 'main-nodegroup',
        resourceinstance_id: 'resource-123',
        data: tileData,
        parenttile_id: null,
        provisionaledits: null,
        sortorder: null
      } as any);

      expect(tile).toBeDefined();
      expect(tile.tileid).toBe('complex-tile');
      expect(tile.nodegroup_id).toBe('main-nodegroup');
      expect(tile.resourceinstance_id).toBe('resource-123');
    });
  });

  describe('StaticNodegroup', () => {
    it('should create a nodegroup with basic properties', () => {
      const nodegroup = new StaticNodegroup({
        nodegroupid: 'ng-123',
        cardinality: 'n',
        legacygroupid: null,
        parentnodegroup_id: null
      });

      expect(nodegroup.nodegroupid).toBe('ng-123');
      expect(nodegroup.cardinality).toBe('n');
      expect(nodegroup.legacygroupid).toBe(null);
      expect(nodegroup.parentnodegroup_id).toBe(null);
    });

    it('should create a nodegroup with single cardinality', () => {
      const nodegroup = new StaticNodegroup({
        nodegroupid: 'ng-single',
        cardinality: '1',
        legacygroupid: null,
        parentnodegroup_id: null
      });

      expect(nodegroup.cardinality).toBe('1');
    });

    it('should create a nodegroup with parent reference', () => {
      

      const nodegroup = new StaticNodegroup({
        nodegroupid: 'ng-child',
        cardinality: 'n',
        legacygroupid: null,
        parentnodegroup_id: 'ng-parent'
      });

      expect(nodegroup.parentnodegroup_id).toBe('ng-parent');
    });

    it('should copy a nodegroup', () => {
      

      const original = new StaticNodegroup({
        nodegroupid: 'ng-copy-test',
        cardinality: 'n',
        legacygroupid: null,
        parentnodegroup_id: 'parent-ng'
      });

      const copy = original.copy?.();

      expect(copy).toBeDefined();
      expect(copy?.nodegroupid).toBe('ng-copy-test');
      expect(copy?.cardinality).toBe('n');
      expect(copy?.parentnodegroup_id).toBe('parent-ng');
      expect(copy).not.toBe(original);
    });
  });

  describe('StaticConstraint', () => {
    it('should create a constraint with basic properties', () => {
      

      const constraint = new StaticConstraint({
        card_id: 'card-123',
        constraintid: 'constraint-456',
        nodes: ['node-1', 'node-2', 'node-3'],
        uniquetoallinstances: true
      });

      expect(constraint.card_id).toBe('card-123');
      expect(constraint.constraintid).toBe('constraint-456');
      expect(constraint.nodes).toEqual(['node-1', 'node-2', 'node-3']);
      expect(constraint.uniquetoallinstances).toBe(true);
    });

    it('should create a constraint with empty nodes array', () => {
      

      const constraint = new StaticConstraint({
        card_id: 'card-empty',
        constraintid: 'constraint-empty',
        nodes: [],
        uniquetoallinstances: false
      });

      expect(constraint.nodes).toEqual([]);
      expect(constraint.uniquetoallinstances).toBe(false);
    });

    it('should create a constraint not unique to all instances', () => {
      

      const constraint = new StaticConstraint({
        card_id: 'card-789',
        constraintid: 'constraint-789',
        nodes: ['node-a'],
        uniquetoallinstances: false
      });

      expect(constraint.uniquetoallinstances).toBe(false);
    });
  });

  describe('StaticCard', () => {
    it('should create a card with all properties', () => {
      

      const card = new StaticCard({
        active: true,
        cardid: 'card-123',
        component_id: 'component-456',
        config: { option1: true, option2: 'value' },
        constraints: [],
        cssclass: 'custom-card-class',
        description: 'Test card description',
        graph_id: 'graph-789',
        helpenabled: true,
        helptext: { en: 'Help text' },
        helptitle: { en: 'Help title' },
        instructions: { en: 'Instructions' },
        is_editable: true,
        name: { en: 'Test Card' },
        nodegroup_id: 'ng-123',
        sortorder: 1,
        visible: true
      });

      expect(card.active).toBe(true);
      expect(card.cardid).toBe('card-123');
      expect(card.component_id).toBe('component-456');
      expect(card.config).toEqual({ option1: true, option2: 'value' });
      expect(card.constraints).toEqual([]);
      expect(card.cssclass).toBe('custom-card-class');
      expect(card.description).toBe('Test card description');
      expect(card.graph_id).toBe('graph-789');
      expect(card.helpenabled).toBe(true);
      expect(card.helptext.toString()).toBe('Help text');
      expect(card.helptitle.toString()).toBe('Help title');
      expect(card.instructions.toString()).toBe('Instructions');
      expect(card.is_editable).toBe(true);
      expect(card.name.toString()).toBe('Test Card');
      expect(card.nodegroup_id).toBe('ng-123');
      expect(card.sortorder).toBe(1);
      expect(card.visible).toBe(true);
    });

    it('should create a card with constraints', () => {
      

      const constraint = new StaticConstraint({
        card_id: 'card-with-constraints',
        constraintid: 'constraint-1',
        nodes: ['node-1', 'node-2'],
        uniquetoallinstances: true
      });

      const card = new StaticCard({
        active: true,
        cardid: 'card-with-constraints',
        component_id: 'comp-1',
        constraints: [constraint],
        cssclass: null,
        description: null,
        graph_id: 'graph-1',
        helpenabled: false,
        helptext: { en: '' },
        helptitle: { en: '' },
        instructions: { en: '' },
        is_editable: true,
        name: { en: 'Card with constraints' },
        nodegroup_id: 'ng-1',
        sortorder: 0,
        visible: true
      });

      expect(card.constraints).toHaveLength(1);
      expect(card.constraints[0].constraintid).toBe('constraint-1');
      expect(card.constraints[0].nodes).toEqual(['node-1', 'node-2']);
    });

    it('should create an inactive card', () => {
      

      const card = new StaticCard({
        active: false,
        cardid: 'inactive-card',
        component_id: 'comp-inactive',
        constraints: [],
        cssclass: null,
        description: null,
        graph_id: 'graph-1',
        helpenabled: false,
        helptext: { en: '' },
        helptitle: { en: '' },
        instructions: { en: '' },
        is_editable: false,
        name: { en: 'Inactive Card' },
        nodegroup_id: 'ng-1',
        sortorder: 99,
        visible: false
      });

      expect(card.active).toBe(false);
      expect(card.is_editable).toBe(false);
      expect(card.visible).toBe(false);
    });
  });

  describe('StaticCardsXNodesXWidgets', () => {
    it('should create a card-node-widget relationship', () => {
      

      const cnw = new StaticCardsXNodesXWidgets({
        card_id: 'card-123',
        config: { widget_option: 'value' },
        id: 'cnw-456',
        label: { en: 'Widget Label' },
        node_id: 'node-789',
        sortorder: 1,
        visible: true,
        widget_id: 'widget-abc'
      });

      expect(cnw.card_id).toBe('card-123');
      expect(cnw.config).toEqual({ widget_option: 'value' });
      expect(cnw.id).toBe('cnw-456');
      expect(cnw.label.toString()).toBe('Widget Label');
      expect(cnw.node_id).toBe('node-789');
      expect(cnw.sortorder).toBe(1);
      expect(cnw.visible).toBe(true);
      expect(cnw.widget_id).toBe('widget-abc');
    });

    it('should create a hidden widget', () => {
      

      const cnw = new StaticCardsXNodesXWidgets({
        card_id: 'card-1',
        config: {},
        id: 'cnw-hidden',
        label: { en: 'Hidden Widget' },
        node_id: 'node-1',
        sortorder: 0,
        visible: false,
        widget_id: 'widget-hidden'
      });

      expect(cnw.visible).toBe(false);
    });

    it('should create a widget with empty config', () => {
      

      const cnw = new StaticCardsXNodesXWidgets({
        card_id: 'card-minimal',
        config: {},
        id: 'cnw-minimal',
        label: { en: 'Minimal' },
        node_id: 'node-minimal',
        sortorder: 0,
        visible: true,
        widget_id: 'widget-minimal'
      });

      expect(cnw.config).toEqual({});
    });

    it('should handle multilingual labels', () => {
      

      const cnw = new StaticCardsXNodesXWidgets({
        card_id: 'card-multi',
        config: {},
        id: 'cnw-multi',
        label: { en: 'English Label', fr: 'Étiquette française', es: 'Etiqueta española' },
        node_id: 'node-multi',
        sortorder: 0,
        visible: true,
        widget_id: 'widget-multi'
      });

      expect(cnw.label.translations.en).toBe('English Label');
      expect(cnw.label.translations.fr).toBe('Étiquette française');
      expect(cnw.label.translations.es).toBe('Etiqueta española');
    });
  });

  describe('StaticFunctionsXGraphs', () => {
    it('should create a function-graph relationship with descriptor config', () => {
      

      const fxg = new StaticFunctionsXGraphs({
        config: {
          descriptor_types: [
            {
              nodegroup_id: 'ng-1',
              string_template: 'Template {{value}}'
            },
            {
              nodegroup_id: 'ng-2',
              string_template: 'Another {{field}}'
            }
          ]
        },
        function_id: 'func-123',
        graph_id: 'graph-456',
        id: 'fxg-789'
      });

      expect(fxg.function_id).toBe('func-123');
      expect(fxg.graph_id).toBe('graph-456');
      expect(fxg.id).toBe('fxg-789');
      expect(fxg.config.descriptor_types).toHaveLength(2);
      expect(fxg.config.descriptor_types[0].nodegroup_id).toBe('ng-1');
      expect(fxg.config.descriptor_types[0].string_template).toBe('Template {{value}}');
      expect(fxg.config.descriptor_types[1].nodegroup_id).toBe('ng-2');
      expect(fxg.config.descriptor_types[1].string_template).toBe('Another {{field}}');
    });

    it('should create a function with empty descriptor types', () => {
      

      const fxg = new StaticFunctionsXGraphs({
        config: {
          descriptor_types: []
        },
        function_id: 'func-empty',
        graph_id: 'graph-empty',
        id: 'fxg-empty'
      });

      expect(fxg.config.descriptor_types).toEqual([]);
    });

    it('should copy a function-graph relationship', () => {
      

      const original = new StaticFunctionsXGraphs({
        config: {
          descriptor_types: [
            {
              nodegroup_id: 'ng-copy',
              string_template: 'Copy test {{x}}'
            }
          ]
        },
        function_id: 'func-copy',
        graph_id: 'graph-copy',
        id: 'fxg-copy'
      });

      const copy = original.copy();

      expect(copy).toBeDefined();
      expect(copy.function_id).toBe('func-copy');
      expect(copy.graph_id).toBe('graph-copy');
      expect(copy.id).toBe('fxg-copy');
      expect(copy.config.descriptor_types).toHaveLength(1);
      expect(copy).not.toBe(original);
    });

    it('should handle complex descriptor templates', () => {
      

      const fxg = new StaticFunctionsXGraphs({
        config: {
          descriptor_types: [
            {
              nodegroup_id: 'ng-complex',
              string_template: '<span class="{{class}}">{{name}}: {{value}}</span>'
            }
          ]
        },
        function_id: 'func-complex',
        graph_id: 'graph-complex',
        id: 'fxg-complex'
      });

      expect(fxg.config.descriptor_types[0].string_template).toContain('{{class}}');
      expect(fxg.config.descriptor_types[0].string_template).toContain('{{name}}');
      expect(fxg.config.descriptor_types[0].string_template).toContain('{{value}}');
    });
  });

  describe('StaticPublication', () => {
    it('should create a publication with all fields', () => {
      

      const pub = new StaticPublication({
        graph_id: 'graph-123',
        notes: 'Publication notes here',
        publicationid: 'pub-456',
        published_time: '2024-01-15T10:30:00Z'
      });

      expect(pub.graph_id).toBe('graph-123');
      expect(pub.notes).toBe('Publication notes here');
      expect(pub.publicationid).toBe('pub-456');
      expect(pub.published_time).toBe('2024-01-15T10:30:00Z');
    });

    it('should create a publication with null notes', () => {
      

      const pub = new StaticPublication({
        graph_id: 'graph-no-notes',
        notes: null,
        publicationid: 'pub-no-notes',
        published_time: '2024-02-01T14:00:00Z'
      });

      expect(pub.notes).toBe(null);
    });

    it('should copy a publication', () => {
      

      const original = new StaticPublication({
        graph_id: 'graph-copy',
        notes: 'Copy test',
        publicationid: 'pub-copy',
        published_time: '2024-03-01T09:00:00Z'
      });

      const copy = original.copy?.();

      expect(copy).toBeDefined();
      expect(copy?.graph_id).toBe('graph-copy');
      expect(copy?.notes).toBe('Copy test');
      expect(copy?.publicationid).toBe('pub-copy');
      expect(copy?.published_time).toBe('2024-03-01T09:00:00Z');
      expect(copy).not.toBe(original);
    });

    it('should handle ISO timestamp formats', () => {
      

      const timestamps = [
        '2024-01-01T00:00:00Z',
        '2024-12-31T23:59:59.999Z',
        '2024-06-15T12:30:45.123456Z'
      ];

      timestamps.forEach((timestamp, index) => {
        const pub = new StaticPublication({
          graph_id: `graph-${index}`,
          notes: null,
          publicationid: `pub-${index}`,
          published_time: timestamp
        });

        expect(pub.published_time).toBe(timestamp);
      });
    });
  });
});
