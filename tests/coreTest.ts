import { test } from "vitest";
import { StaticGraph } from '../src/static-types';

async function basicGraph({ test }, use) {
  const graph = StaticGraph.create({
    author: "Author",
    description: "Graph description",
    name: "Graph",
  });
  await use(graph);
}

const coreTest = test.extend<{
  basicGraph: StaticGraph
}>({
  basicGraph: basicGraph
});

export { coreTest };
