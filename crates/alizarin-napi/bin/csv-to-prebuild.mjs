#!/usr/bin/env node

// Convert a CSV-based model directory into PrebuildLoader-compatible JSON.
//
// Input layout (per model):
//   <dir>/graphs/resource_models/<model>/graph.csv
//   <dir>/graphs/resource_models/<model>/nodes.csv
//   <dir>/graphs/resource_models/<model>/collections.csv  (optional)
//   <dir>/business_data/<model>.csv                       (optional)
//
// Output layout (PrebuildLoader-compatible):
//   <output>/graphs/resource_models/<graph_id>.json
//   <output>/business_data/<graph_id>.json
//
// Usage:
//   node csv-to-prebuild.mjs <input_dir> [output_dir]
//
// If output_dir is omitted, writes alongside the input (in-place conversion).

import { readFileSync, writeFileSync, mkdirSync, readdirSync, existsSync, statSync } from 'fs';
import { join, basename, resolve } from 'path';

// Import from the parent package (works when run from the alizarin-napi directory
// or when installed as a dependency).
let napi;
try {
  napi = await import('../index.js');
} catch {
  // Fallback for when run via npx or as an installed package
  napi = await import('@alizarin/napi');
}

const { buildGraphFromCsvs, buildBusinessDataFromCsv } = napi;

const args = process.argv.slice(2);
if (args.length < 1) {
  console.error('Usage: csv-to-prebuild <input_dir> [output_dir]');
  console.error('');
  console.error('Converts CSV model definitions to PrebuildLoader-compatible JSON.');
  process.exit(1);
}

const inputDir = resolve(args[0]);
const outputDir = resolve(args[1] || inputDir);

const modelsDir = join(inputDir, 'graphs', 'resource_models');
const businessDataDir = join(inputDir, 'business_data');

if (!existsSync(modelsDir)) {
  console.error(`No graphs/resource_models/ directory found in ${inputDir}`);
  process.exit(1);
}

// Create output directories
const outGraphsDir = join(outputDir, 'graphs', 'resource_models');
const outBusinessDir = join(outputDir, 'business_data');
mkdirSync(outGraphsDir, { recursive: true });
mkdirSync(outBusinessDir, { recursive: true });

// Find all model directories (each has graph.csv + nodes.csv)
const modelDirs = readdirSync(modelsDir)
  .map(name => join(modelsDir, name))
  .filter(p => statSync(p).isDirectory())
  .filter(p => existsSync(join(p, 'graph.csv')) && existsSync(join(p, 'nodes.csv')));

if (modelDirs.length === 0) {
  console.error('No model directories found with graph.csv + nodes.csv');
  process.exit(1);
}

console.log(`Found ${modelDirs.length} model(s) in ${modelsDir}`);

const RDM_NAMESPACE = 'https://example.org/rdm/';

for (const modelDir of modelDirs) {
  const modelName = basename(modelDir);
  console.log(`  ${modelName}:`);

  // Read CSVs
  const graphCsv = readFileSync(join(modelDir, 'graph.csv'), 'utf-8');
  const nodesCsv = readFileSync(join(modelDir, 'nodes.csv'), 'utf-8');
  const collectionsPath = join(modelDir, 'collections.csv');
  const collectionsCsv = existsSync(collectionsPath)
    ? readFileSync(collectionsPath, 'utf-8')
    : undefined;

  // Build graph
  let result;
  try {
    result = buildGraphFromCsvs(graphCsv, nodesCsv, collectionsCsv, RDM_NAMESPACE);
  } catch (e) {
    console.error(`    Failed to build graph: ${e.message}`);
    continue;
  }

  const { graph, collections } = result;
  const graphId = graph.graphid;
  console.log(`    graph: ${graphId}`);

  // Write graph JSON
  const graphOutPath = join(outGraphsDir, `${graphId}.json`);
  writeFileSync(graphOutPath, JSON.stringify(graph, null, 2));
  console.log(`    wrote ${graphOutPath}`);

  // Look for matching business data CSV
  // Try: business_data/<modelName>.csv
  const bdCsvPath = join(businessDataDir, `${modelName}.csv`);
  if (existsSync(bdCsvPath)) {
    const csvData = readFileSync(bdCsvPath, 'utf-8');
    try {
      const businessData = buildBusinessDataFromCsv(
        csvData,
        JSON.stringify(graph),
        JSON.stringify(collections),
      );
      const bdOutPath = join(outBusinessDir, `${graphId}.json`);
      writeFileSync(bdOutPath, JSON.stringify(businessData, null, 2));
      const resourceCount = businessData?.business_data?.resources?.length ?? 0;
      console.log(`    wrote ${bdOutPath} (${resourceCount} resources)`);
    } catch (e) {
      console.error(`    Failed to build business data: ${e.message}`);
    }
  } else {
    console.log(`    no business data CSV found at ${bdCsvPath}`);
  }
}

console.log(`\nDone. Output: ${outputDir}`);
