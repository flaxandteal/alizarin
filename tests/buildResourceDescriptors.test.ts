import { describe, it, expect, vi, beforeEach } from 'vitest';
import { buildResourceDescriptors, DESCRIPTOR_FUNCTION_ID } from '../js/utils';
import { StaticGraph, StaticNode, StaticResourceDescriptors, StaticFunctionsXGraphs } from '../js/static-types';

describe('buildResourceDescriptors', () => {
  let mockGraph: StaticGraph;
  let mockNodes: Map<string, StaticNode>;
  let mockWrapper: any;

  beforeEach(() => {
    // Reset mocks before each test
    mockNodes = new Map();
    mockWrapper = {
      retrievePseudo: vi.fn()
    };
  });

  describe('No descriptor configuration', () => {
    it('should return empty descriptors when graph has no functions_x_graphs', async () => {
      mockGraph = {
        functions_x_graphs: undefined
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result).toBeInstanceOf(StaticResourceDescriptors);
      expect(result.name).toBeUndefined();
      expect(result.description).toBeUndefined();
    });

    it('should return empty descriptors when descriptor function is not found', async () => {
      mockGraph = {
        functions_x_graphs: [
          {
            function_id: 'some-other-function-id',
            config: {}
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result).toBeInstanceOf(StaticResourceDescriptors);
      expect(result.name).toBeUndefined();
    });

    it('should return empty descriptors when config has no descriptor_types', async () => {
      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {}
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result).toBeInstanceOf(StaticResourceDescriptors);
      expect(result.name).toBeUndefined();
    });
  });

  describe('Basic string template replacement', () => {
    it('should replace single placeholder with value', async () => {
      const nodeId = 'node-1';
      const semanticNode: StaticNode = {
        nodeid: nodeId,
        nodegroup_id: nodeId,
        alias: 'semantic_node',
        name: 'Semantic Node',
        datatype: 'semantic'
      } as StaticNode;

      const childNode: StaticNode = {
        nodeid: 'child-1',
        nodegroup_id: nodeId,
        alias: 'title',
        name: 'Title',
        datatype: 'string'
      } as StaticNode;

      mockNodes.set(nodeId, semanticNode);
      mockNodes.set('child-1', childNode);

      // Mock value retrieval
      const mockSemanticValue = {
        __has: vi.fn().mockReturnValue(true),
        title: Promise.resolve('Test Resource')
      };

      mockWrapper.retrievePseudo = vi.fn().mockResolvedValue([{
        getValue: vi.fn().mockResolvedValue(mockSemanticValue)
      }]);

      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                name: {
                  nodegroup_id: nodeId,
                  string_template: '<Title>'
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result.name).toBe('Test Resource');
    });

    it('should replace multiple placeholders in template', async () => {
      const nodeId = 'node-1';
      const semanticNode: StaticNode = {
        nodeid: nodeId,
        nodegroup_id: nodeId,
        alias: 'semantic_node',
        name: 'Semantic Node',
        datatype: 'semantic'
      } as StaticNode;

      const titleNode: StaticNode = {
        nodeid: 'child-1',
        nodegroup_id: nodeId,
        alias: 'title',
        name: 'Title',
        datatype: 'string'
      } as StaticNode;

      const yearNode: StaticNode = {
        nodeid: 'child-2',
        nodegroup_id: nodeId,
        alias: 'year',
        name: 'Year',
        datatype: 'string'
      } as StaticNode;

      mockNodes.set(nodeId, semanticNode);
      mockNodes.set('child-1', titleNode);
      mockNodes.set('child-2', yearNode);

      const mockSemanticValue = {
        __has: vi.fn().mockReturnValue(true),
        title: Promise.resolve('Rome'),
        year: Promise.resolve('100 AD')
      };

      mockWrapper.retrievePseudo = vi.fn().mockResolvedValue([{
        getValue: vi.fn().mockResolvedValue(mockSemanticValue)
      }]);

      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                description: {
                  nodegroup_id: nodeId,
                  string_template: 'The <Title> from <Year>'
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result.description).toBe('The Rome from 100 AD');
    });
  });

  describe('Missing values handling', () => {
    it('should leave placeholder unchanged when value is missing', async () => {
      const nodeId = 'node-1';
      const semanticNode: StaticNode = {
        nodeid: nodeId,
        nodegroup_id: nodeId,
        alias: 'semantic_node',
        name: 'Semantic Node',
        datatype: 'semantic'
      } as StaticNode;

      const titleNode: StaticNode = {
        nodeid: 'child-1',
        nodegroup_id: nodeId,
        alias: 'title',
        name: 'Title',
        datatype: 'string'
      } as StaticNode;

      mockNodes.set(nodeId, semanticNode);
      mockNodes.set('child-1', titleNode);

      // Mock value retrieval with no semantic value found
      // and no individual value either (returns empty/undefined)
      mockWrapper.retrievePseudo = vi.fn()
        .mockResolvedValueOnce([{
          getValue: vi.fn().mockResolvedValue(null)
        }])
        .mockResolvedValueOnce([undefined]); // Individual retrieval returns nothing

      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                name: {
                  nodegroup_id: nodeId,
                  string_template: '<Title> Resource'
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      // Placeholder should remain when value is missing
      expect(result.name).toBe('<Title> Resource');
    });

    it('should replace available values and leave missing ones as placeholders', async () => {
      const nodeId = 'node-1';
      const semanticNode: StaticNode = {
        nodeid: nodeId,
        nodegroup_id: nodeId,
        alias: 'semantic_node',
        name: 'Semantic Node',
        datatype: 'semantic'
      } as StaticNode;

      const titleNode: StaticNode = {
        nodeid: 'child-1',
        nodegroup_id: nodeId,
        alias: 'title',
        name: 'Title',
        datatype: 'string'
      } as StaticNode;

      const yearNode: StaticNode = {
        nodeid: 'child-2',
        nodegroup_id: nodeId,
        alias: 'year',
        name: 'Year',
        datatype: 'string'
      } as StaticNode;

      mockNodes.set(nodeId, semanticNode);
      mockNodes.set('child-1', titleNode);
      mockNodes.set('child-2', yearNode);

      // Only title has value, year is missing
      const mockSemanticValue = {
        __has: (alias: string) => alias === 'title',
        title: Promise.resolve('Rome')
      };

      mockWrapper.retrievePseudo = vi.fn().mockResolvedValue([{
        getValue: vi.fn().mockResolvedValue(mockSemanticValue)
      }]);

      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                name: {
                  nodegroup_id: nodeId,
                  string_template: '<Title> from <Year>'
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result.name).toBe('Rome from <Year>');
    });
  });

  describe('Empty template handling', () => {
    it('should skip descriptor when string_template is empty', async () => {
      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                name: {
                  nodegroup_id: 'node-1',
                  string_template: ''
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result.name).toBeUndefined();
    });

    it('should skip descriptor when string_template is null/undefined', async () => {
      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                name: {
                  nodegroup_id: 'node-1',
                  string_template: null as any
                },
                description: {
                  nodegroup_id: 'node-1',
                  string_template: undefined as any
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result.name).toBeUndefined();
      expect(result.description).toBeUndefined();
    });
  });

  describe('Multiple descriptor types', () => {
    it('should process multiple descriptor types (name, description, map_popup)', async () => {
      const nodeId = 'node-1';
      const semanticNode: StaticNode = {
        nodeid: nodeId,
        nodegroup_id: nodeId,
        alias: 'semantic_node',
        name: 'Semantic Node',
        datatype: 'semantic'
      } as StaticNode;

      const titleNode: StaticNode = {
        nodeid: 'child-1',
        nodegroup_id: nodeId,
        alias: 'title',
        name: 'Title',
        datatype: 'string'
      } as StaticNode;

      mockNodes.set(nodeId, semanticNode);
      mockNodes.set('child-1', titleNode);

      const mockSemanticValue = {
        __has: vi.fn().mockReturnValue(true),
        title: Promise.resolve('Ancient Rome')
      };

      mockWrapper.retrievePseudo = vi.fn().mockResolvedValue([{
        getValue: vi.fn().mockResolvedValue(mockSemanticValue)
      }]);

      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                name: {
                  nodegroup_id: nodeId,
                  string_template: '<Title>'
                },
                description: {
                  nodegroup_id: nodeId,
                  string_template: 'Description of <Title>'
                },
                map_popup: {
                  nodegroup_id: nodeId,
                  string_template: 'Location: <Title>'
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result.name).toBe('Ancient Rome');
      expect(result.description).toBe('Description of Ancient Rome');
      expect(result.map_popup).toBe('Location: Ancient Rome');
    });
  });

  describe('PseudoList handling', () => {
    it('should handle PseudoList by taking first element', async () => {
      const nodeId = 'node-1';
      const semanticNode: StaticNode = {
        nodeid: nodeId,
        nodegroup_id: nodeId,
        alias: 'semantic_node',
        name: 'Semantic Node',
        datatype: 'semantic'
      } as StaticNode;

      const titleNode: StaticNode = {
        nodeid: 'child-1',
        nodegroup_id: nodeId,
        alias: 'title',
        name: 'Title',
        datatype: 'string'
      } as StaticNode;

      mockNodes.set(nodeId, semanticNode);
      mockNodes.set('child-1', titleNode);

      // Mock PseudoList with first element
      const mockSemanticValue = {
        __has: vi.fn().mockReturnValue(true),
        title: Promise.resolve('From PseudoList')
      };

      const mockPseudoList = [
        {
          getValue: vi.fn().mockResolvedValue(mockSemanticValue)
        }
      ];

      // Make it behave like PseudoList for instanceof check
      Object.setPrototypeOf(mockPseudoList, Array.prototype);
      (mockPseudoList as any).constructor = { name: 'PseudoList' };

      mockWrapper.retrievePseudo = vi.fn().mockResolvedValue([mockPseudoList]);

      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                name: {
                  nodegroup_id: nodeId,
                  string_template: '<Title>'
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      // Should handle PseudoList and extract value
      expect(result.name).toBe('From PseudoList');
    });
  });

  describe('Nested value handling (inner property)', () => {
    it('should handle nested values with inner property', async () => {
      const nodeId = 'node-1';
      const semanticNode: StaticNode = {
        nodeid: nodeId,
        nodegroup_id: nodeId,
        alias: 'semantic_node',
        name: 'Semantic Node',
        datatype: 'semantic'
      } as StaticNode;

      mockNodes.set(nodeId, semanticNode);

      const innerValue = {
        __has: vi.fn().mockReturnValue(false)
      };

      const outerValue = {
        inner: {
          getValue: vi.fn().mockResolvedValue(innerValue)
        },
        getValue: vi.fn().mockResolvedValue('Outer Value')
      };

      mockWrapper.retrievePseudo = vi.fn().mockResolvedValue([outerValue]);

      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                name: {
                  nodegroup_id: nodeId,
                  string_template: 'Name'
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(outerValue.getValue).toHaveBeenCalled();
      expect(outerValue.inner.getValue).toHaveBeenCalled();
    });
  });

  describe('Fallback to individual value retrieval', () => {
    it('should fall back to individual retrieval when semantic node is missing', async () => {
      const nodeId = 'node-1';
      const titleNode: StaticNode = {
        nodeid: 'child-1',
        nodegroup_id: nodeId,
        alias: 'title',
        name: 'Title',
        datatype: 'string'
      } as StaticNode;

      mockNodes.set('child-1', titleNode);

      // No semantic node in mockNodes, so valueList.retrieve will be called for individual nodes
      mockWrapper.retrievePseudo = vi.fn()
        .mockResolvedValueOnce([{
          getValue: vi.fn().mockResolvedValue('Individual Value')
        }]);

      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                name: {
                  nodegroup_id: nodeId,
                  string_template: '<Title>'
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result.name).toBe('Individual Value');
      expect(mockWrapper.retrievePseudo).toHaveBeenCalledTimes(1);
    });
  });

  describe('Template with no placeholders', () => {
    it('should return template as-is when no placeholders exist', async () => {
      mockGraph = {
        functions_x_graphs: [
          {
            function_id: DESCRIPTOR_FUNCTION_ID,
            config: {
              descriptor_types: {
                name: {
                  nodegroup_id: 'node-1',
                  string_template: 'Static Name'
                }
              }
            }
          } as StaticFunctionsXGraphs
        ]
      } as any;

      const result = await buildResourceDescriptors(mockGraph, mockNodes, mockWrapper);

      expect(result.name).toBe('Static Name');
    });
  });
});
