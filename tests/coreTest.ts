import { test } from "vitest";
import { StaticGraph, StaticCollection, StaticConcept } from '../src/static-types';

async function basicGraph({ test }, use) {
  const graph = StaticGraph.create({
    author: "Author",
    description: "Graph description",
    name: "Graph",
  });
  await use(graph);
}

async function basicCollection({ test }, use) {
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
  basicGraph: basicGraph,
  basicCollection: basicCollection
});

export { coreTest };
