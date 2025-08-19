import { AlizarinModel, graphManager, staticStore, client, RDM, run } from '../dist/alizarin';
import htm from 'https://unpkg.com/htm@3.0.4/dist/htm.module.js?module';

const html = htm.bind(Vue.h);

await run();

async function initializeAlizarin() {
  const archesClient = new client.ArchesClientRemoteStatic('', {
    allGraphFile: (() => "docs/example/graphs.json"),
    graphToGraphFile: ((graph) => `docs/example/resource_models/${graph.id}.json`),
    graphIdToResourcesFiles: ((graph) => [`docs/example/business_data/_${graph}.json`]),
    collectionIdToFile: ((collectionId) => `docs/example/reference_data/collections/${collectionId}.json`)
  });
  graphManager.archesClient = archesClient;
  staticStore.archesClient = archesClient;
  RDM.archesClient = archesClient;

  await graphManager.initialize();
  return graphManager;
}

export { initializeAlizarin, html };
