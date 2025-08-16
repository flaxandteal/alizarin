import { assert, describe, beforeEach } from 'vitest';
import fetchMock from '@fetch-mock/vitest';
import { GraphMutator } from "../js/graphManager";
import { StaticNode } from "../js/static-types";
import { coreTest } from "./coreTest";

describe("testing graph", () => {
  beforeEach(() => {
    fetchMock.mockReset();
  });

  coreTest(
    "loads graph",
    async ({ basicGraph }) => {
      assert(basicGraph.name.toString() === "Graph");
    }
  );

  coreTest(
    "adds a string node",
    async ({ basicGraph }) => {
      const mutator = new GraphMutator(basicGraph);
      mutator.addStringNode(
        null,
        "test",
        "Test",
        "n",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note",
        "Describe",
        {
          exportable: true,
          fieldname: "TestDescription",
          hascustomalias: false,
          is_collector: false,
          isrequired: false,
          issearchable: true,
          istopnode: true,
          sortorder: 0
        }
      );
      const outputGraph = mutator.apply();
      assert(outputGraph.nodes.length === 2);
      assert(StaticNode.compare(
        outputGraph.nodes[0],
        outputGraph.root
      ) === 2);
      assert(StaticNode.compare(
        outputGraph.nodes[1],
        {
          alias: 'test',
          config: null,
          datatype: 'string',
          description: 'Describe',
          exportable: true,
          fieldname: 'TestDescription',
          hascustomalias: false,
          is_collector: false,
          isrequired: false,
          issearchable: true,
          istopnode: true,
          name: 'Test',
          parentproperty: 'http://www.cidoc-crm.org/cidoc-crm/P3_has_note',
          sortorder: 0,
          ontologyclass: 'http://www.w3.org/2000/01/rdf-schema#Literal',
          sourcebranchpublication_id: null
        }
      ) === 1);
      assert(outputGraph.edges.length === 1);
      assert(
        outputGraph.edges[0].domainnode_id === outputGraph.nodes[0].nodeid &&
        outputGraph.edges[0].rangenode_id === outputGraph.nodes[1].nodeid
      );
      assert(outputGraph.nodegroups.length === 1);
      assert(
        outputGraph.nodegroups[0].nodegroupid === outputGraph.nodes[1].nodegroup_id &&
        outputGraph.nodegroups[0].cardinality === 'n'
      );
    }
  );

  coreTest(
    "adds a concept node",
    async ({ basicGraph, basicCollection }) => {
      const mutator = new GraphMutator(basicGraph);
      mutator.addConceptNode(
        null,
        "test",
        "Test",
        basicCollection,
        "n",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
        "http://www.cidoc-crm.org/cidoc-crm/P3_has_note",
        "Describe",
        {
          exportable: true,
          fieldname: "TestDescription",
          hascustomalias: false,
          is_collector: false,
          isrequired: false,
          issearchable: true,
          istopnode: true,
          sortorder: 0
        }
      );
      const outputGraph = mutator.apply();
      assert(mutator.autocreateCard);
      assert(outputGraph.nodes.length === 2);
      assert(outputGraph.cards?.length === 1);
      assert(outputGraph.cards_x_nodes_x_widgets?.length === 1);
      assert(StaticNode.compare(
        outputGraph.nodes[0],
        outputGraph.root
      ) === 2);
      assert(StaticNode.compare(
        outputGraph.nodes[1],
        {
          alias: 'test',
          config: {
            rdmCollection: 'f63b1a5d-75af-565b-a78f-c710ee6ea2fd',
          },
          datatype: 'concept',
          description: 'Describe',
          exportable: true,
          fieldname: 'TestDescription',
          hascustomalias: false,
          is_collector: false,
          isrequired: false,
          issearchable: true,
          istopnode: true,
          name: 'Test',
          parentproperty: 'http://www.cidoc-crm.org/cidoc-crm/P3_has_note',
          sortorder: 0,
          ontologyclass: 'http://www.w3.org/2000/01/rdf-schema#Literal',
          sourcebranchpublication_id: null
        }
      ) === 1);
      assert(outputGraph.edges.length === 1);
      assert(
        outputGraph.edges[0].domainnode_id === outputGraph.nodes[0].nodeid &&
        outputGraph.edges[0].rangenode_id === outputGraph.nodes[1].nodeid
      );
      assert(outputGraph.nodegroups.length === 1);
      assert(
        outputGraph.nodegroups[0].nodegroupid === outputGraph.nodes[1].nodegroup_id &&
        outputGraph.nodegroups[0].cardinality === 'n'
      );
    }
  )
});
