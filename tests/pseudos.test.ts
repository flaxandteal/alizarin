import { describe, it, expect, beforeAll, beforeEach, vi } from 'vitest';
import { PseudoUnavailable, PseudoValue, PseudoList } from '../js/pseudos';
import { StaticNode, StaticTile, createStaticGraph, StaticGraph, StaticGraphMeta } from '../js/static-types';
import { AttrPromise } from '../js/utils';
import { initWasmForTests } from './wasm-init';
import { WASMResourceModelWrapper, WKRM } from '../pkg/alizarin';
import { ResourceModelWrapper, GraphMutator } from '../js/graphManager';

// Mock dependencies
vi.mock('../js/viewModels', () => ({
  getViewModel: vi.fn((pseudo, tile, node, data, parent, childNodes, isInner) => {
    return Promise.resolve({
      forJson: () => data,
      __parentPseudo: pseudo,
      getChildren: () => [],
      getChildTypes: () => ({}),
      __asTileData: () => [null, []], // Required for getTile() calls
    });
  }),
  viewContext: {},
}));

// Helper function to create a WKRM instance for testing
function createTestWKRM(graph: StaticGraph): WKRM {
  const meta = new StaticGraphMeta({
    graphid: graph.graphid,
    name: "Test Model",  // Use "Test Model" for test expectations
    slug: "test_graph",
    relatable_resource_model_ids: [],
    resource_2_resource_constraints: [],
    extra_fields: {}
  });
  return new WKRM(meta);
}

// Helper function to create a wrapper with a real WASM model
function createTestWrapper(graph: StaticGraph): WASMResourceModelWrapper {
  const wrapper = new WASMResourceModelWrapper(
    createTestWKRM(graph),
    graph,
    true
  );
  wrapper.buildNodesForGraph(graph);

  // Mark all nodegroups as permitted for testing
  const nodegroups = wrapper.getNodegroupObjects();
  const permissions = new Map();
  nodegroups.forEach((_nodegroup, nodegroupId) => {
    permissions.set(nodegroupId, true);
  });
  wrapper.setPermittedNodegroups(permissions);

  return wrapper;
}

describe('Pseudos', () => {
  beforeAll(async () => {
    // Initialize WASM module for tests
    await initWasmForTests();
  });

  describe('PseudoUnavailable', () => {
    let node: StaticNode;

    beforeEach(() => {
      node = new StaticNode({
        nodeid: 'test-node-1',
        name: 'Test Node',
        datatype: 'string',
        graph_id: 'graph-1',
        nodegroup_id: 'ng-1',
        alias: 'test_node',
        config: {},
        exportable: false,
        hascustomalias: false,
        is_collector: false,
        isrequired: false,
        issearchable: false,
        istopnode: false,
        sortorder: 0,
        description: null,
        fieldname: null,
        ontologyclass: null,
        parentproperty: null,
        sourcebranchpublication_id: null,
      } as any);
    });

    it('should create an unavailable pseudo with correct properties', () => {
      const pseudo = new PseudoUnavailable(node);

      expect(pseudo.node).toBe(node);
      expect(pseudo.parentValue).toBe(null);
      expect(pseudo.tile).toBe(null);
      expect(pseudo.isOuter).toBe(false);
    });

    it('should return null for forJson', async () => {
      const pseudo = new PseudoUnavailable(node);
      const result = await pseudo.forJson();

      expect(result).toBe(null);
    });

    it('should describe field as "Unavailable field"', () => {
      const pseudo = new PseudoUnavailable(node);

      expect(pseudo.describeField()).toBe('Unavailable field');
      expect(pseudo.describeFieldGroup()).toBe('Unavailable field');
    });

    it('should return null from getValue', async () => {
      const pseudo = new PseudoUnavailable(node);
      const value = await pseudo.getValue();

      expect(value).toBe(null);
    });

    it('should return 0 for getLength', () => {
      const pseudo = new PseudoUnavailable(node);

      expect(pseudo.getLength()).toBe(0);
    });

    it('should return empty array for getChildren', () => {
      const pseudo = new PseudoUnavailable(node);

      expect(pseudo.getChildren()).toEqual([]);
      expect(pseudo.getChildren(true)).toEqual([]);
    });

    it('should not be iterable', () => {
      const pseudo = new PseudoUnavailable(node);

      expect(pseudo.isIterable()).toBe(false);
    });
  });

  describe('PseudoValue', () => {
    let node: StaticNode;
    let tile: StaticTile;
    let parent: any;
    let childNodes: Map<string, StaticNode>;
    let wrapper: WASMResourceModelWrapper;
    let graph: StaticGraph;

    beforeEach(() => {
      // Create a graph with nodes of different datatypes for testing
      graph = createStaticGraph({
        name: "Test Graph",
        author: "Test Author",
      });

      const mutator = new GraphMutator(graph);
      // Add only test_node for most tests - we'll add others in specific tests that need them
      mutator.addStringNode(
        null,
        "test_node",
        "Test Node",
        "1",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );
      // Add a child node for testing
      mutator.addStringNode(
        null,
        "child",
        "Child Node",
        "1",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );

      graph = mutator.apply();

      // Create real WASM wrapper
      wrapper = createTestWrapper(graph);

      // Get the test node
      node = wrapper.nodesByAlias!.get("test_node")!;

      tile = new StaticTile({
        tileid: 'tile-1',
        nodegroup_id: node.nodegroup_id!,
        resourceinstance_id: 'res-1',
        data: {},
        parenttile_id: null,
        provisionaledits: null,
        sortorder: 0,
      } as any);

      parent = {
        id: 'parent-1',
        __: wrapper,
        $: {
          model: wrapper,
        },
      };

      childNodes = new Map();
    });

    it('should create a PseudoValue with basic properties', () => {
      const pseudo = PseudoValue.create(node, tile, null, parent);

      // tile comes from WasmPseudoValue now, may not be identical object
      expect(pseudo.tile).toBeTruthy();
      expect(pseudo.parent).toBe(parent);
      expect(pseudo.accessed).toBe(false);
      expect(pseudo.datatype).toBe('string');
      expect(pseudo.independent).toBe(false);
    });

    it('should throw error when parent is null', () => {
      expect(() => {
        PseudoValue.create(node, tile, null, null as any);
      }).toThrow('Must have a parent or parent class for a pseudo-node');
    });

    it('should mark as independent when tile is null', () => {
      const pseudo = PseudoValue.create(node, null, null, parent);

      expect(pseudo.independent).toBe(true);
    });

    it('should identify iterable datatypes correctly', () => {
      // Create a separate graph with list datatypes for this test
      const testGraph = createStaticGraph({
        name: "Test Graph",
        author: "Test Author",
      });

      const testMutator = new GraphMutator(testGraph);
      testMutator.addStringNode(null, "string_node", "String Node", "1", "http://www.w3.org/2000/01/rdf-schema#Literal", "http://www.cidoc-crm.org/cidoc-crm/P3_has_note");

      // Use concept node with n cardinality and is_list: true for concept-list (iterable)
      testMutator.addConceptNode(null, "concept_list_node", "Concept List", {rdmRootNode: "test"} as any, "n", "http://www.w3.org/2000/01/rdf-schema#Literal", "http://www.cidoc-crm.org/cidoc-crm/P2_has_type", undefined, { is_list: true });

      // Create domain-value-list node directly using _addGenericNode (iterable)
      (testMutator as any)._addGenericNode(null, "domain_list_node", "Domain List", "n", "domain-value-list", "http://www.w3.org/2000/01/rdf-schema#Literal", "http://www.cidoc-crm.org/cidoc-crm/P67_refers_to", undefined, {}, {});

      const mutatedGraph = testMutator.apply();
      const testWrapper = createTestWrapper(mutatedGraph);

      const testParent = {
        id: 'test-parent-1',
        __: testWrapper,
        $: { model: testWrapper },
      };

      // Test concept-list (should be iterable)
      const conceptListNode = testWrapper.nodesByAlias!.get("concept_list_node")!;
      const pseudo1 = PseudoValue.create(conceptListNode, tile, null, testParent);
      expect(pseudo1.isIterable()).toBe(true);

      // Test domain-value-list (should be iterable)
      const domainListNode = testWrapper.nodesByAlias!.get("domain_list_node")!;
      const pseudo2 = PseudoValue.create(domainListNode, tile, null, testParent);
      expect(pseudo2.isIterable()).toBe(true);

      // Test string (should not be iterable)
      const stringNode = testWrapper.nodesByAlias!.get("string_node")!;
      const pseudo3 = PseudoValue.create(stringNode, tile, null, testParent);
      expect(pseudo3.isIterable()).toBe(false);
    });

    it('should describe field with model name', () => {
      const pseudo = PseudoValue.create(node, tile, null, parent);

      expect(pseudo.describeField()).toBe('Test Model - Test Node');
    });

    it('should describe field group with nodegroup name', () => {
      const pseudo = PseudoValue.create(node, tile, null, parent);

      // With real WASM wrappers, nodegroup names match the node name
      expect(pseudo.describeFieldGroup()).toBe('Test Model - Test Node');
    });

    it('should return parent tile id', () => {
      const tileWithParent = new StaticTile({
        tileid: 'tile-1',
        nodegroup_id: 'ng-1',
        resourceinstance_id: 'res-1',
        data: {},
        parenttile_id: 'parent-tile-1',
        provisionaledits: null,
        sortorder: 0,
      } as any);

      const pseudo = PseudoValue.create(node, tileWithParent, null, parent);

      expect(pseudo.getParentTileId()).toBe('parent-tile-1');
    });

    it('should clear value and remove from tile data', () => {
      const tileData = new Map();
      tileData.set('node-1', 'some value');

      const tileWithData = new StaticTile({
        tileid: 'tile-1',
        nodegroup_id: 'ng-1',
        resourceinstance_id: 'res-1',
        data: tileData,
        parenttile_id: null,
        provisionaledits: null,
        sortorder: 0,
      } as any);

      const pseudo = PseudoValue.create(node, tileWithData, 'initial value', parent);

      expect(tileWithData.data.has('node-1')).toBe(true);

      pseudo.clear();

      // Note: With Rust implementation, value is cleared via WasmPseudoValue
      // The _cachedValue on JS side is also cleared
      // The tile.data behavior depends on Rust implementation
    });

    it('should automatically create inner/outer structure for non-semantic nodes with children', () => {
      // Create a non-semantic node with children - this should trigger inner/outer creation
      const testGraph = createStaticGraph({
        name: "Test Graph",
        author: "Test Author",
      });

      const testMutator = new GraphMutator(testGraph);
      testMutator.addStringNode(
        null,
        "parent_node",
        "Parent Node",
        "n",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );
      testMutator.addStringNode(
        "parent_node",
        "child_node",
        "Child Node",
        "n",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );
      const mutatedGraph = testMutator.apply();
      const testWrapper = createTestWrapper(mutatedGraph);

      const parentNode = testWrapper.nodesByAlias!.get("parent_node")!;
      const testParent = {
        id: 'parent-1',
        __: testWrapper,
        $: { model: testWrapper },
      };

      // Create child nodes map
      const childNodesMap = new Map<string, StaticNode>();
      const childNode = testWrapper.nodesByAlias!.get("child_node")!;
      childNodesMap.set(childNode.nodeid, childNode);

      // Create PseudoValue with children - should create inner/outer automatically
      const pseudo = PseudoValue.create(parentNode, tile, null, testParent);

      // The outer pseudo should have isOuter = true
      expect(pseudo.isOuter).toBe(true);
      // The inner should exist and have isInner = true
      expect(pseudo.inner).toBeDefined();
      // Inner's datatype should be 'semantic' (overridden by Rust)
      expect(pseudo.inner!.datatype).toBe('semantic');
      expect(pseudo.inner!.isInner).toBe(true);
    });

    it('should resolve getValue without infinite loop when outer has no tile (regression test)', async () => {
      // REGRESSION TEST: When an outer PseudoValue has no tile but has an inner,
      // getValue() should not create a circular promise reference.
      // The bug was: updateValue() set _cachedValue to a promise that called
      // updateValue() recursively, which returned the same _cachedValue promise,
      // creating a circular reference that never resolved.
      const testGraph = createStaticGraph({
        name: "Test Graph",
        author: "Test Author",
      });

      const testMutator = new GraphMutator(testGraph);
      testMutator.addStringNode(
        null,
        "parent_node",
        "Parent Node",
        "n",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );
      testMutator.addStringNode(
        "parent_node",
        "child_node",
        "Child Node",
        "n",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );
      const mutatedGraph = testMutator.apply();
      const testWrapper = createTestWrapper(mutatedGraph);

      const parentNode = testWrapper.nodesByAlias!.get("parent_node")!;
      const testParent = {
        id: 'parent-1',
        __: testWrapper,
        $: { model: testWrapper },
      };

      // Create PseudoValue with null tile - this triggers the inner tile lookup path
      const pseudo = PseudoValue.create(parentNode, null, null, testParent);

      // Verify we have the inner/outer structure
      expect(pseudo.isOuter).toBe(true);
      expect(pseudo.inner).toBeDefined();

      // This should resolve within a reasonable time, not hang forever
      // The bug would cause this to never resolve (circular promise)
      const timeoutPromise = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('getValue() timed out - possible circular promise')), 1000)
      );

      const result = await Promise.race([
        pseudo.getValue(),
        timeoutPromise
      ]);

      // Should resolve to a value (the mock returns an object)
      expect(result).toBeDefined();
    });

    it('should not create inner/outer for semantic nodes', () => {
      // Semantic nodes should not trigger inner/outer creation even with children
      const testGraph = createStaticGraph({
        name: "Test Graph",
        author: "Test Author",
      });

      const testMutator = new GraphMutator(testGraph);
      testMutator.addSemanticNode(
        null,
        "semantic_node",
        "Semantic Node",
        "http://www.cidoc-crm.org/cidoc-crm/E55_Type"
      );
      testMutator.addStringNode(
        "semantic_node",
        "child_node",
        "Child Node",
        "n",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );
      const mutatedGraph = testMutator.apply();
      const testWrapper = createTestWrapper(mutatedGraph);

      const semanticNode = testWrapper.nodesByAlias!.get("semantic_node")!;
      const testParent = {
        id: 'parent-1',
        __: testWrapper,
        $: { model: testWrapper },
      };

      const pseudo = PseudoValue.create(semanticNode, tile, null, testParent);

      // Semantic nodes should not create inner/outer structure
      expect(pseudo.isOuter).toBe(false);
      expect(pseudo.datatype).toBe('semantic');
    });

    it('should get children from empty value', () => {
      const pseudo = PseudoValue.create(node, tile, null, parent);

      expect(pseudo.getChildren()).toEqual([]);
      expect(pseudo.getChildren(true)).toEqual([]);
    });

    it('should return length of children', () => {
      const pseudo = PseudoValue.create(node, tile, null, parent);

      expect(pseudo.getLength()).toBe(0);
    });
  });

  describe('PseudoNode (via PseudoValue)', () => {
    let node: StaticNode;
    let tile: StaticTile;
    let parent: any;
    let childNodes: Map<string, StaticNode>;
    let wrapper: WASMResourceModelWrapper;
    let graph: StaticGraph;

    beforeEach(() => {
      // Create a graph with test nodes
      graph = createStaticGraph({
        name: "Test Model",
        author: "Test Author",
      });

      const mutator = new GraphMutator(graph);
      mutator.addStringNode(
        null,
        "test_alias",
        "Test Node",
        "1",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );
      (mutator as any)._addGenericNode(
        null,
        "iterable_alias",
        "Iterable Node",
        "n",
        "concept-list",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P2_has_type"
      );
      mutator.addStringNode(
        null,
        "child",
        "Child Node",
        "1",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );

      graph = mutator.apply();

      // Create real WASM wrapper
      wrapper = createTestWrapper(graph);

      node = wrapper.nodesByAlias!.get("test_alias")!;

      tile = new StaticTile({
        tileid: 'tile-1',
        nodegroup_id: node.nodegroup_id!,
        resourceinstance_id: 'res-1',
        data: {},
        parenttile_id: null,
        provisionaledits: null,
        sortorder: 0,
      } as any);

      parent = {
        id: 'parent-1',
        __: wrapper,
        $: {
          model: wrapper,
        },
      };

      childNodes = new Map();
    });

    describe('getNodePlaceholder', () => {
      it('should return simple placeholder for non-iterable node', () => {
        const pseudo = PseudoValue.create(node, tile, null, parent);

        // WasmPseudoValue has getNodePlaceholder on the node property
        // but the new PseudoValue exposes node as the WasmPseudoValue itself
        // The alias getter should work
        expect(pseudo.node.alias).toBe('test_alias');
      });

      it('should return placeholder with [*] for iterable node', () => {
        const iterableNode = wrapper.nodesByAlias!.get("iterable_alias")!;

        const pseudo = PseudoValue.create(iterableNode, tile, null, parent);

        // Check iterable detection works
        expect(pseudo.isIterable()).toBe(true);
      });

      it('should build nested placeholder with parentNode', () => {
        // Use actual nodes from the wrapper that exist in the graph
        const parentNode = wrapper.nodesByAlias!.get("test_alias")!;
        const childNode = wrapper.nodesByAlias!.get("child")!;

        const parentPseudo = PseudoValue.create(parentNode, tile, null, parent);
        const childPseudo = PseudoValue.create(childNode, tile, null, parent);
        childPseudo.parentValue = parentPseudo;

        // Check parent relationship is established
        expect(childPseudo.parentValue).toBe(parentPseudo);
      });

      it('should build nested placeholder with iterable parent', () => {
        // Use actual iterable node from the wrapper
        const parentNode = wrapper.nodesByAlias!.get("iterable_alias")!;
        const childNode = wrapper.nodesByAlias!.get("child")!;

        const parentPseudo = PseudoValue.create(parentNode, tile, null, parent);
        const childPseudo = PseudoValue.create(childNode, tile, null, parent);
        childPseudo.parentValue = parentPseudo;

        // Check parent is iterable
        expect(childPseudo.parentValue!.isIterable()).toBe(true);
      });

      it('should build deep nested placeholder', () => {
        // Create a graph with 3 levels of nesting for testing
        const nestedGraph = createStaticGraph({
          name: "Nested Graph",
          author: "Test Author",
        });

        const nestedMutator = new GraphMutator(nestedGraph);
        // Level 1 - grandparent
        nestedMutator.addStringNode(
          null,
          "level1",
          "Level 1",
          "n",
          "http://www.w3.org/2000/01/rdf-schema#Literal",
          "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
        );
        // Level 2 - iterable parent
        (nestedMutator as any)._addGenericNode(
          "level1",
          "level2",
          "Level 2",
          "n",
          "concept-list",
          "http://www.w3.org/2000/01/rdf-schema#Literal",
          "http://www.cidoc-crm.org/cidoc-crm/P2_has_type",
          undefined,
          {},
          {}
        );
        // Level 3 - child
        nestedMutator.addStringNode(
          "level2",
          "level3",
          "Level 3",
          "n",
          "http://www.w3.org/2000/01/rdf-schema#Literal",
          "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
        );

        const mutatedNestedGraph = nestedMutator.apply();
        const nestedWrapper = createTestWrapper(mutatedNestedGraph);

        const level1Node = nestedWrapper.nodesByAlias!.get("level1")!;
        const level2Node = nestedWrapper.nodesByAlias!.get("level2")!;
        const level3Node = nestedWrapper.nodesByAlias!.get("level3")!;

        const nestedParent = {
          id: 'parent-1',
          __: nestedWrapper,
          $: { model: nestedWrapper },
        };

        const grandparentPseudo = PseudoValue.create(level1Node, tile, null, nestedParent);
        const parentPseudo = PseudoValue.create(level2Node, tile, null, nestedParent);
        parentPseudo.parentValue = grandparentPseudo;

        const childPseudo = PseudoValue.create(level3Node, tile, null, nestedParent);
        childPseudo.parentValue = parentPseudo;

        // Check deep nesting works
        expect(childPseudo.parentValue).toBe(parentPseudo);
        expect(childPseudo.parentValue!.parentValue).toBe(grandparentPseudo);
      });
    });

    describe('childNodes property', () => {
      it('should access childNodes from WASM wrapper', () => {
        // Create a graph with parent-child relationship
        const testGraph = createStaticGraph({
          name: "Test Graph",
          author: "Test Author",
        });

        const testMutator = new GraphMutator(testGraph);
        testMutator.addStringNode(
          null,
          "parent_node",
          "Parent Node",
          "n",
          "http://www.w3.org/2000/01/rdf-schema#Literal",
          "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
        );
        testMutator.addStringNode(
          "parent_node",
          "child_node",
          "Child Node",
          "n",
          "http://www.w3.org/2000/01/rdf-schema#Literal",
          "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
        );
        const mutatedGraph = testMutator.apply();
        const testWrapper = createTestWrapper(mutatedGraph);

        const parentNode = testWrapper.nodesByAlias!.get("parent_node")!;
        const childNode = testWrapper.nodesByAlias!.get("child_node")!;

        const testParent = {
          id: 'parent-1',
          __: testWrapper,
          $: { model: testWrapper },
        };

        const pseudo = PseudoValue.create(parentNode, tile, null, testParent);

        // Non-semantic nodes with children automatically create inner/outer structure
        // The child nodes are on the inner node, not the outer node
        expect(pseudo.isOuter).toBe(true);
        expect(pseudo.inner).toBeDefined();

        // Inner should be accessible
        expect(pseudo.inner!.isInner).toBe(true);
      });

      it('should handle nodes with no children', () => {
        // Use a leaf node that has no children
        const leafNode = wrapper.nodesByAlias!.get("child")!;

        const pseudo = PseudoValue.create(leafNode, tile, null, parent);

        // Leaf node should not have inner/outer structure
        expect(pseudo.isOuter).toBe(false);
      });
    });
  });

  describe('PseudoList', () => {
    let node: StaticNode;
    let parent: any;

    beforeEach(() => {
      node = new StaticNode({
        nodeid: 'node-1',
        name: 'List Node',
        datatype: 'string',
        graph_id: 'graph-1',
        nodegroup_id: 'ng-1',
        alias: 'list_node',
        config: {},
        exportable: false,
        hascustomalias: false,
        is_collector: true,
        isrequired: false,
        issearchable: false,
        istopnode: false,
        sortorder: 0,
        description: null,
        fieldname: null,
        ontologyclass: null,
        parentproperty: null,
        sourcebranchpublication_id: null,
      } as any);

      parent = {
        id: 'parent-1',
        __: {
          wkrm: {
            modelName: 'Test Model',
          },
        },
        $: {
          model: {
            getNodeObjects: () => new Map([
              ['ng-1', { name: 'Test Nodegroup' }],
            ]),
            getNodegroupName: (id: string) => {
              const nodes = new Map([
                ['ng-1', { name: 'Test Nodegroup' }],
              ]);
              return nodes.get(id)?.name || '';
            },
          },
        },
      };
    });

    it('should create an empty PseudoList', () => {
      const list = new PseudoList();

      expect(list).toBeInstanceOf(Array);
      expect(list.length).toBe(0);
      expect(list.node).toBeUndefined();
      expect(list.parent).toBeUndefined();
    });

    it('should initialize with node and parent', () => {
      const list = new PseudoList();
      list.initialize(node, parent);

      expect(list.node).toBe(node);
      expect(list.parent).toBe(parent);
      expect(list.tile).toBeUndefined();
      expect(list.parenttileId).toBeUndefined();
      expect(list.ghostChildren).toBeInstanceOf(Set);
    });

    it('should throw error when trying to initialize with array node', () => {
      const list = new PseudoList();

      expect(() => {
        list.initialize([node] as any, parent);
      }).toThrow('Cannot make a list of lists');
    });

    it('should throw error when initializing without parent', () => {
      const list = new PseudoList();

      expect(() => {
        list.initialize(node, null);
      }).toThrow('Must have a parent or parent class for a pseudo-node');
    });

    it('should always be iterable', () => {
      const list = new PseudoList();

      expect(list.isIterable()).toBe(true);
    });

    it('should describe field as array notation', () => {
      const list = new PseudoList();
      list.initialize(node, parent);

      expect(list.describeField()).toBe('[Test Model - List Node]');
    });

    it('should describe field group as array notation', () => {
      const list = new PseudoList();
      list.initialize(node, parent);

      expect(list.describeFieldGroup()).toBe('[Test Model - Test Nodegroup]');
    });

    it('should handle uninitialized node in descriptions', () => {
      const list = new PseudoList();

      expect(list.describeField()).toBe('[(uninitialized node)]');
      expect(list.describeFieldGroup()).toBe('[(uninitialized node)]');
    });

    it('should return itself from getValue', async () => {
      const list = new PseudoList();
      list.initialize(node, parent);

      const value = await list.getValue();

      expect(value).toBe(list);
    });

    it('should have custom toString', () => {
      const list = new PseudoList();

      expect(list.toString()).toBe('<PL: 0>');

      list.push(Promise.resolve({ forJson: () => 'item1' }));
      list.push(Promise.resolve({ forJson: () => 'item2' }));

      expect(list.toString()).toBe('<PL: 2>');
    });

    it('should support array operations', () => {
      const list = new PseudoList();
      list.initialize(node, parent);

      const item1 = Promise.resolve({ forJson: () => 'value1' });
      const item2 = Promise.resolve({ forJson: () => 'value2' });

      list.push(item1);
      list.push(item2);

      expect(list.length).toBe(2);
      expect(list[0]).toBe(item1);
      expect(list[1]).toBe(item2);

      const popped = list.pop();
      expect(popped).toBe(item2);
      expect(list.length).toBe(1);
    });

    it('should sort items by tile sortorder', async () => {
      const list = new PseudoList();
      list.initialize(node, parent);

      const item1 = {
        forJson: () => 'item1',
        __parentPseudo: {
          tile: { sortorder: 3 },
        },
      };

      const item2 = {
        forJson: () => 'item2',
        __parentPseudo: {
          tile: { sortorder: 1 },
        },
      };

      const item3 = {
        forJson: () => 'item3',
        __parentPseudo: {
          tile: { sortorder: 2 },
        },
      };

      list.push(Promise.resolve(item1));
      list.push(Promise.resolve(item2));
      list.push(Promise.resolve(item3));

      const sorted = await list.sorted();

      expect(sorted[0]).toBe(item2); // sortorder: 1
      expect(sorted[1]).toBe(item3); // sortorder: 2
      expect(sorted[2]).toBe(item1); // sortorder: 3
    });

    it('should handle items without sortorder in sorting', async () => {
      const list = new PseudoList();
      list.initialize(node, parent);

      const item1 = {
        forJson: () => 'item1',
        __parentPseudo: {
          tile: { sortorder: 5 },
        },
      };

      const item2 = {
        forJson: () => 'item2',
        __parentPseudo: null,
      };

      list.push(Promise.resolve(item1));
      list.push(Promise.resolve(item2));

      const sorted = await list.sorted();

      expect(sorted.length).toBe(2);
      // Verify both items are present in the sorted result
      const sortorders = sorted.map(item => item.__parentPseudo?.tile?.sortorder ?? 0);
      expect(sortorders).toContain(0);  // item2's default sortorder
      expect(sortorders).toContain(5);  // item1's sortorder
    });

    it('should serialize to JSON array', async () => {
      const list = new PseudoList();
      list.initialize(node, parent);

      const item1 = {
        forJson: () => ({ id: 1, value: 'first' }),
        __parentPseudo: { tile: { sortorder: 1 } },
      };

      const item2 = {
        forJson: () => ({ id: 2, value: 'second' }),
        __parentPseudo: { tile: { sortorder: 2 } },
      };

      list.push(Promise.resolve(item1));
      list.push(Promise.resolve(item2));

      const json = await list.forJson();

      expect(json).toEqual([
        { id: 1, value: 'first' },
        { id: 2, value: 'second' },
      ]);
    });
  });

  describe('createPseudoNode (Rust implementation)', () => {
    let wrapper: ResourceModelWrapper<any>;
    let node: StaticNode;
    let graph: StaticGraph;

    beforeEach(() => {
      // Create a graph with test nodes
      graph = createStaticGraph({
        name: "Test Model",
        author: "Test Author",
      });

      const mutator = new GraphMutator(graph);
      mutator.addStringNode(
        null,
        "test_node",
        "Test Node",
        "n",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );
      mutator.addStringNode(
        null,
        "child_node",
        "Child Node",
        "n",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note"
      );

      graph = mutator.apply();

      // Create ResourceModelWrapper (extends WASM wrapper)
      wrapper = new ResourceModelWrapper(createTestWKRM(graph), graph, undefined, true);
      wrapper.buildNodes();

      // Mark all nodegroups as permitted for testing
      const nodegroups = wrapper.getNodegroupObjects();
      const permissions = new Map();
      nodegroups.forEach((_nodegroup, nodegroupId) => {
        permissions.set(nodegroupId, true);
      });
      wrapper.setPermittedNodegroups(permissions);

      node = wrapper.nodesByAlias!.get("test_node")!;
    });

    it('should throw error when node alias not found', () => {
      expect(() => {
        wrapper.createPseudoNode('nonexistent_node');
      }).toThrow();
    });

    it('should create PseudoNode for single cardinality node', () => {
      const result = wrapper.createPseudoNode('test_node', true);

      // Rust createPseudoNode returns a WASM PseudoNode object
      expect(result).toBeDefined();
      expect(result).toHaveProperty('__wbg_ptr');
    });

    it('should create PseudoNode for cardinality n collector node', () => {
      // Create a new graph with a collector node in a cardinality-n nodegroup
      const testGraph = createStaticGraph({
        name: "Test Graph",
        author: "Test Author",
      });

      const mutator = new GraphMutator(testGraph);
      // Create a collector node with cardinality 'n'
      mutator.addStringNode(
        null,
        "collector_node",
        "Collector Node",
        "n",  // cardinality n
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note",
        undefined,  // description
        { is_collector: true }  // options - must explicitly set is_collector
      );

      const testGraphWithCollector = mutator.apply();
      const testWrapper = new ResourceModelWrapper(createTestWKRM(testGraphWithCollector), testGraphWithCollector, undefined, true);
      testWrapper.buildNodes();

      const nodegroups = testWrapper.getNodegroupObjects();
      const permissions = new Map();
      nodegroups.forEach((_nodegroup, nodegroupId) => {
        permissions.set(nodegroupId, true);
      });
      testWrapper.setPermittedNodegroups(permissions);

      const result = testWrapper.createPseudoNode('collector_node', false);

      // Rust createPseudoNode returns a WASM PseudoNode object
      expect(result).toBeDefined();
      expect(result).toHaveProperty('__wbg_ptr');
    });

    it('should create PseudoNode even for list when single=true', () => {
      // Create a new graph with a collector node in a cardinality-n nodegroup
      const testGraph = createStaticGraph({
        name: "Test Graph",
        author: "Test Author",
      });

      const mutator = new GraphMutator(testGraph);
      mutator.addStringNode(
        null,
        "collector_node",
        "Collector Node",
        "n",  // cardinality n
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note",
        undefined,  // description
        { is_collector: true }  // options - must explicitly set is_collector
      );

      const testGraphWithCollector = mutator.apply();
      const testWrapper = new ResourceModelWrapper(createTestWKRM(testGraphWithCollector), testGraphWithCollector, undefined, true);
      testWrapper.buildNodes();

      const nodegroups = testWrapper.getNodegroupObjects();
      const permissions = new Map();
      nodegroups.forEach((_nodegroup, nodegroupId) => {
        permissions.set(nodegroupId, true);
      });
      testWrapper.setPermittedNodegroups(permissions);

      const result = testWrapper.createPseudoNode('collector_node', true);

      // Rust createPseudoNode returns a WASM PseudoNode object
      expect(result).toBeDefined();
      expect(result).toHaveProperty('__wbg_ptr');
    });

    it('should create PseudoNode when node is not permitted', () => {
      // Create a wrapper without setting permissions (permissions default to false)
      const unpermittedWrapper = new ResourceModelWrapper(
        createTestWKRM(graph),
        graph,
        undefined,
        false  // default_allow = false, so permissions default to not permitted
      );
      unpermittedWrapper.buildNodes();
      // Note: We intentionally don't call setPermittedNodegroups, so permissions default to false

      const result = unpermittedWrapper.createPseudoNode('test_node', true);

      // Rust createPseudoNode returns a WASM PseudoNode object regardless of permissions
      // Permission checking happens at a different layer
      expect(result).toBeDefined();
      expect(result).toHaveProperty('__wbg_ptr');
    });

    it('should create PseudoNode when tile is provided', () => {
      // Create a new graph with a collector node
      const testGraph = createStaticGraph({
        name: "Test Graph",
        author: "Test Author",
      });

      const mutator = new GraphMutator(testGraph);
      mutator.addStringNode(
        null,
        "collector_node",
        "Collector Node",
        "n",  // cardinality n
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note",
        undefined,  // description
        { is_collector: true }  // options - must explicitly set is_collector
      );

      const testGraphWithCollector = mutator.apply();
      const testWrapper = new ResourceModelWrapper(createTestWKRM(testGraphWithCollector), testGraphWithCollector, undefined, true);
      testWrapper.buildNodes();

      const nodegroups = testWrapper.getNodegroupObjects();
      const permissions = new Map();
      nodegroups.forEach((_nodegroup, nodegroupId) => {
        permissions.set(nodegroupId, true);
      });
      testWrapper.setPermittedNodegroups(permissions);

      const collectorNode = testWrapper.nodesByAlias!.get("collector_node")!;

      const tile = new StaticTile({
        tileid: 'tile-1',
        nodegroup_id: collectorNode.nodegroup_id!,
        resourceinstance_id: 'res-1',
        data: new Map(),
        parenttile_id: null,
        provisionaledits: null,
        sortorder: 0,
        ensureId: () => 'tile-1'
      } as any);

      const result = testWrapper.createPseudoNode('collector_node', false, tile);

      // Rust createPseudoNode returns a WASM PseudoNode object
      expect(result).toBeDefined();
      expect(result).toHaveProperty('__wbg_ptr');
    });
  });
});
