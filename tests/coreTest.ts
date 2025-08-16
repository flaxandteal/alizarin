import { test } from "vitest";
import { StaticGraph, StaticCollection, StaticConcept } from '../js/static-types';

async function basicGraph(_context: any, use: any) {
  const graph = StaticGraph.create({
    author: "Author",
    description: "Graph description",
    name: "Graph",
  });
  await use(graph);
}

async function basicCollection(_context: any, use: any) {
  const graph = StaticCollection.fromConceptScheme({
    conceptScheme: StaticConcept.fromValue(
      null,
      "My Concept",
      ["Concept A", "Concept B", "Concept C"]
    )
  });
  await use(graph);
}

const coreTest = test.extend<{
  basicGraph: StaticGraph,
  basicCollection: StaticCollection
}>({
  basicGraph: basicGraph as any,
  basicCollection: basicCollection as any
});

export { coreTest };
