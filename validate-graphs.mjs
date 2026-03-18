#!/usr/bin/env node
/**
 * Alizarin Graph Validation CLI
 *
 * Command-line tool to validate Arches graph models and business data files.
 * Ensures files are compatible with Alizarin's loading requirements.
 *
 * Usage:
 *   node validate-graphs.mjs [path-to-data-directory]
 *
 * Examples:
 *   node validate-graphs.mjs
 *   node validate-graphs.mjs ./data
 *   node validate-graphs.mjs ../magic
 */

import { validateGraphLoading } from './dist/validation/index.js';
import { resolve } from 'path';

const basePath = process.argv[2] || '.';
const absolutePath = resolve(basePath);

console.log('🔍 ALIZARIN GRAPH VALIDATION');
console.log('============================\n');
console.log(`📁 Validating files in: ${absolutePath}\n`);

try {
  const result = validateGraphLoading(basePath);

  // Exit with appropriate status code
  process.exit(result.success ? 0 : 1);

} catch (error) {
  console.error('\n❌ VALIDATION ERROR');
  console.error('===================');
  console.error(`\nFailed to run validation: ${error.message}`);
  console.error('\nStack trace:');
  console.error(error.stack);
  process.exit(2);
}
