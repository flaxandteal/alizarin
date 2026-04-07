#!/usr/bin/env node
/**
 * CLI tool to validate 3-CSV model files (graph.csv, nodes.csv, collections.csv)
 * using alizarin's WASM-based CSV model loader.
 *
 * Usage:
 *   node scripts/validate-model-csvs.mjs <dir>
 *   node scripts/validate-model-csvs.mjs <graph.csv> <nodes.csv> [collections.csv]
 *
 * When given a directory, looks for graph.csv, nodes.csv, and collections.csv inside it.
 *
 * Exit codes:
 *   0 - valid (no errors, may have warnings)
 *   1 - validation errors found
 *   2 - usage/file error
 */

import { readFileSync, existsSync, statSync } from 'fs';
import { join, resolve } from 'path';
import { fileURLToPath } from 'url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const pkgDir = resolve(__dirname, '..', 'pkg');

// Import WASM module
const { initSync, validateModelCsvs, buildGraphFromModelCsvs } = await import(join(pkgDir, 'alizarin.js'));

// Initialize WASM synchronously for CLI use
const wasmPath = join(pkgDir, 'alizarin_bg.wasm');
if (!existsSync(wasmPath)) {
  console.error('Error: WASM not built. Run `npm run build:wasm` first.');
  process.exit(2);
}
initSync({ module: readFileSync(wasmPath) });

// Parse arguments
const args = process.argv.slice(2);
let graphPath, nodesPath, collectionsPath, rdmNamespace;

// Check for --build flag
const buildIdx = args.indexOf('--build');
let doBuild = false;
if (buildIdx !== -1) {
  doBuild = true;
  args.splice(buildIdx, 1);
}

// Check for --namespace flag
const nsIdx = args.indexOf('--namespace');
if (nsIdx !== -1 && nsIdx + 1 < args.length) {
  rdmNamespace = args[nsIdx + 1];
  args.splice(nsIdx, 2);
}

if (args.length === 1 && existsSync(args[0]) && statSync(args[0]).isDirectory()) {
  const dir = args[0];
  graphPath = join(dir, 'graph.csv');
  nodesPath = join(dir, 'nodes.csv');
  collectionsPath = join(dir, 'collections.csv');
} else if (args.length >= 2) {
  graphPath = args[0];
  nodesPath = args[1];
  collectionsPath = args[2] || null;
} else {
  console.error('Usage: validate-model-csvs.mjs <dir>');
  console.error('       validate-model-csvs.mjs <graph.csv> <nodes.csv> [collections.csv]');
  console.error('');
  console.error('Options:');
  console.error('  --build              Also attempt to build the graph (requires --namespace)');
  console.error('  --namespace <str>    RDM namespace (UUID or URL) for deterministic ID generation');
  process.exit(2);
}

// Read files
function readCsv(path, required) {
  if (!path) return null;
  if (!existsSync(path)) {
    if (required) {
      console.error(`Error: required file not found: ${path}`);
      process.exit(2);
    }
    return null;
  }
  return readFileSync(path, 'utf-8');
}

const graphCsv = readCsv(graphPath, true);
const nodesCsv = readCsv(nodesPath, true);
const collectionsCsv = readCsv(collectionsPath, false);

// Validate
let diagnostics;
try {
  diagnostics = validateModelCsvs(graphCsv, nodesCsv, collectionsCsv || null);
} catch (e) {
  console.error(`Validation failed: ${e}`);
  process.exit(1);
}

// Report diagnostics
const errors = diagnostics.filter(d => d.level === 'Error');
const warnings = diagnostics.filter(d => d.level === 'Warning');

for (const d of diagnostics) {
  const loc = d.line != null ? `:${d.line}` : '';
  const prefix = d.level === 'Error' ? '\x1b[31mERROR\x1b[0m' : '\x1b[33mWARN\x1b[0m';
  console.log(`${prefix} [${d.file}${loc}] ${d.message}`);
}

if (errors.length === 0 && warnings.length === 0) {
  console.log('\x1b[32m✓ No issues found\x1b[0m');
}

if (errors.length > 0) {
  console.log(`\n\x1b[31m${errors.length} error(s), ${warnings.length} warning(s)\x1b[0m`);
} else if (warnings.length > 0) {
  console.log(`\n\x1b[33m${warnings.length} warning(s)\x1b[0m`);
}

// Optionally build
if (doBuild) {
  if (!rdmNamespace) {
    console.error('\n--build requires --namespace <rdm-namespace>');
    process.exit(2);
  }
  console.log('\nBuilding graph...');
  try {
    const result = buildGraphFromModelCsvs(graphCsv, nodesCsv, collectionsCsv || null, rdmNamespace);
    const graph = result.graph;
    const collections = result.collections;
    const graphName = typeof graph.name === 'object' ? (graph.name?.en || JSON.stringify(graph.name)) : graph.name;
    console.log(`\x1b[32m✓ Graph built: "${graphName}" with ${Object.keys(graph.nodes || {}).length} nodes\x1b[0m`);
    if (collections && collections.length > 0) {
      console.log(`\x1b[32m✓ ${collections.length} collection(s) created\x1b[0m`);
    }
    // Output JSON to stdout if piped
    if (!process.stdout.isTTY) {
      console.log(JSON.stringify(result, null, 2));
    }
  } catch (e) {
    console.error(`\x1b[31mBuild failed: ${e}\x1b[0m`);
    process.exit(1);
  }
}

process.exit(errors.length > 0 ? 1 : 0);
