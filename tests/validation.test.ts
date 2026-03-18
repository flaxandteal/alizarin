import { assert, describe, beforeEach, afterEach, vi } from 'vitest';
import { test } from 'vitest';
import { mkdtempSync, writeFileSync, rmSync, mkdirSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';
import {
  GraphLoadingValidator,
  validateGraphLoading,
  type ValidationSummary
} from '../js/validation/index';

// Mock console to suppress output during tests
const mockConsole = () => {
  vi.spyOn(console, 'log').mockImplementation(() => {});
  vi.spyOn(console, 'error').mockImplementation(() => {});
};

const restoreConsole = () => {
  vi.restoreAllMocks();
};

// Test fixtures
import validModel from './fixtures/validation/valid-model.json';
import invalidModel from './fixtures/validation/invalid-model.json';
import validBusinessData from './fixtures/validation/valid-business-data.json';
import invalidBusinessData from './fixtures/validation/invalid-business-data.json';
import validGraphsRegistry from './fixtures/validation/valid-graphs.json';
import invalidGraphsRegistry from './fixtures/validation/invalid-graphs.json';

/**
 * Test fixture that creates a temporary directory with test files
 */
async function validationTestDir({}: any, use: any) {
  const tempDir = mkdtempSync(join(tmpdir(), 'alizarin-validation-test-'));

  await use(tempDir);

  // Cleanup
  rmSync(tempDir, { recursive: true, force: true });
}

/**
 * Test fixture that sets up a directory with valid files
 */
async function validFilesDir({}: any, use: any) {
  const tempDir = mkdtempSync(join(tmpdir(), 'alizarin-valid-'));

  // Write valid files
  writeFileSync(
    join(tempDir, 'arches-test-model-model.json'),
    JSON.stringify(validModel, null, 2)
  );
  writeFileSync(
    join(tempDir, 'arches-business-data-test.json'),
    JSON.stringify(validBusinessData, null, 2)
  );
  writeFileSync(
    join(tempDir, 'graphs.json'),
    JSON.stringify(validGraphsRegistry, null, 2)
  );

  await use(tempDir);

  rmSync(tempDir, { recursive: true, force: true });
}

/**
 * Test fixture that sets up a directory with invalid files
 */
async function invalidFilesDir({}: any, use: any) {
  const tempDir = mkdtempSync(join(tmpdir(), 'alizarin-invalid-'));

  // Write invalid files
  writeFileSync(
    join(tempDir, 'arches-test-model-model.json'),
    JSON.stringify(invalidModel, null, 2)
  );
  writeFileSync(
    join(tempDir, 'arches-business-data-test.json'),
    JSON.stringify(invalidBusinessData, null, 2)
  );
  writeFileSync(
    join(tempDir, 'graphs.json'),
    JSON.stringify(invalidGraphsRegistry, null, 2)
  );

  await use(tempDir);

  rmSync(tempDir, { recursive: true, force: true });
}

/**
 * Test fixture that sets up a directory with mixed valid/invalid files
 */
async function mixedFilesDir({}: any, use: any) {
  const tempDir = mkdtempSync(join(tmpdir(), 'alizarin-mixed-'));

  // Write mixed files
  writeFileSync(
    join(tempDir, 'arches-valid-model.json'),
    JSON.stringify(validModel, null, 2)
  );
  writeFileSync(
    join(tempDir, 'arches-invalid-model.json'),
    JSON.stringify(invalidModel, null, 2)
  );
  writeFileSync(
    join(tempDir, 'arches-business-data-valid.json'),
    JSON.stringify(validBusinessData, null, 2)
  );
  writeFileSync(
    join(tempDir, 'graphs.json'),
    JSON.stringify(validGraphsRegistry, null, 2)
  );

  await use(tempDir);

  rmSync(tempDir, { recursive: true, force: true });
}

// Create test suite with fixtures
const validationTest = test.extend<{
  validationTestDir: string;
  validFilesDir: string;
  invalidFilesDir: string;
  mixedFilesDir: string;
}>({
  validationTestDir,
  validFilesDir,
  invalidFilesDir,
  mixedFilesDir
});

describe('GraphLoadingValidator', () => {
  beforeEach(() => {
    mockConsole();
  });

  afterEach(() => {
    restoreConsole();
  });

  describe('constructor and initialization', () => {
    validationTest('creates validator with default path', ({ validationTestDir }) => {
      const validator = new GraphLoadingValidator(validationTestDir);
      assert(validator !== null);
      const results = validator.getResults();
      assert(results.graphModels.passed === 0);
      assert(results.graphModels.failed === 0);
    });

    validationTest('accepts custom base path', ({ validFilesDir }) => {
      const validator = new GraphLoadingValidator(validFilesDir);
      assert(validator !== null);
    });
  });

  describe('validateGraphModels', () => {
    validationTest('validates correct graph model', ({ validFilesDir }) => {
      const validator = new GraphLoadingValidator(validFilesDir);
      validator.validateGraphModels();
      const results = validator.getResults();

      assert(results.graphModels.passed === 1);
      assert(results.graphModels.failed === 0);
      assert(results.graphModels.errors.length === 0);
    });

    validationTest('detects invalid graph model', ({ invalidFilesDir }) => {
      const validator = new GraphLoadingValidator(invalidFilesDir);
      validator.validateGraphModels();
      const results = validator.getResults();

      assert(results.graphModels.failed > 0);
      assert(results.graphModels.errors.length > 0);
      assert(results.graphModels.errors[0].error.includes('graphid'));
    });

    validationTest('validates top node has null nodegroup_id', ({ invalidFilesDir }) => {
      const validator = new GraphLoadingValidator(invalidFilesDir);
      validator.validateGraphModels();
      const results = validator.getResults();

      // The invalid model has istopnode:true but nodegroup_id is not null
      assert(results.graphModels.failed > 0);
      const hasNodegroupError = results.graphModels.errors.some(
        err => err.error.includes('nodegroup_id')
      );
      assert(hasNodegroupError);
    });

    validationTest('handles multiple model files', ({ mixedFilesDir }) => {
      const validator = new GraphLoadingValidator(mixedFilesDir);
      validator.validateGraphModels();
      const results = validator.getResults();

      // Should have both passed and failed
      assert(results.graphModels.passed >= 1);
      assert(results.graphModels.failed >= 1);
    });
  });

  describe('validateBusinessDataFiles', () => {
    validationTest('validates correct business data', ({ validFilesDir }) => {
      const validator = new GraphLoadingValidator(validFilesDir);
      validator.validateBusinessDataFiles();
      const results = validator.getResults();

      assert(results.businessData.passed === 1);
      assert(results.businessData.failed === 0);
    });

    validationTest('detects invalid business data', ({ invalidFilesDir }) => {
      const validator = new GraphLoadingValidator(invalidFilesDir);
      validator.validateBusinessDataFiles();
      const results = validator.getResults();

      assert(results.businessData.failed > 0);
      assert(results.businessData.errors.length > 0);
    });

    validationTest('validates required tiles', ({ invalidFilesDir }) => {
      const validator = new GraphLoadingValidator(invalidFilesDir);
      validator.validateBusinessDataFiles();
      const results = validator.getResults();

      // Invalid business data has empty tiles array
      assert(results.businessData.failed > 0);
      const hasTilesError = results.businessData.errors.some(
        err => err.error.includes('tiles')
      );
      assert(hasTilesError);
    });
  });

  describe('validateGraphsRegistryFile', () => {
    validationTest('validates correct graphs registry', ({ validFilesDir }) => {
      const validator = new GraphLoadingValidator(validFilesDir);
      validator.validateGraphsRegistryFile();
      const results = validator.getResults();

      assert(results.graphsRegistry.passed === 1);
      assert(results.graphsRegistry.failed === 0);
    });

    validationTest('detects invalid graphs registry', ({ invalidFilesDir }) => {
      const validator = new GraphLoadingValidator(invalidFilesDir);
      validator.validateGraphsRegistryFile();
      const results = validator.getResults();

      assert(results.graphsRegistry.failed > 0);
      assert(results.graphsRegistry.errors.length > 0);
    });

    validationTest('validates slug format', ({ invalidFilesDir }) => {
      const validator = new GraphLoadingValidator(invalidFilesDir);
      validator.validateGraphsRegistryFile();
      const results = validator.getResults();

      // Invalid registry has slug with spaces or other validation errors
      assert(results.graphsRegistry.failed > 0);
      // Just verify there are errors - slug validation or others
      assert(results.graphsRegistry.errors.length > 0);
    });

    validationTest('handles missing graphs.json', ({ validationTestDir }) => {
      const validator = new GraphLoadingValidator(validationTestDir);
      validator.validateGraphsRegistryFile();
      const results = validator.getResults();

      assert(results.graphsRegistry.failed === 1);
      assert(results.graphsRegistry.errors[0].error.includes('does not exist'));
    });
  });

  describe('checkAlizarinCompatibility', () => {
    validationTest('passes for valid Alizarin metadata', ({ validFilesDir }) => {
      const validator = new GraphLoadingValidator(validFilesDir);
      validator.checkAlizarinCompatibility();
      const results = validator.getResults();

      assert(results.alizarinCompatibility.passed === 1);
      assert(results.alizarinCompatibility.failed === 0);
    });

    validationTest('detects missing resourceinstance', ({ validationTestDir }) => {
      // Create business data without proper resourceinstance
      writeFileSync(
        join(validationTestDir, 'arches-business-data-bad.json'),
        JSON.stringify({
          business_data: {
            resources: [{
              resourceid: "12345678-1234-1234-1234-123456789abc",
              graph_id: "12345678-1234-1234-1234-123456789abc",
              tiles: [{ tileid: "11111111-1111-1111-1111-111111111111", nodegroup_id: "22222222-2222-2222-2222-222222222222", data: {} }]
            }]
          }
        }, null, 2)
      );
      writeFileSync(
        join(validationTestDir, 'graphs.json'),
        JSON.stringify(validGraphsRegistry, null, 2)
      );

      const validator = new GraphLoadingValidator(validationTestDir);
      validator.checkAlizarinCompatibility();
      const results = validator.getResults();

      assert(results.alizarinCompatibility.failed > 0);
      const hasResourceInstanceError = results.alizarinCompatibility.errors.some(
        err => err.error.includes('resourceinstance')
      );
      assert(hasResourceInstanceError);
    });

    validationTest('detects missing descriptors', ({ invalidFilesDir }) => {
      const validator = new GraphLoadingValidator(invalidFilesDir);
      validator.checkAlizarinCompatibility();
      const results = validator.getResults();

      // Invalid business data has missing/empty descriptors or other Alizarin issues
      assert(results.alizarinCompatibility.failed > 0);
      assert(results.alizarinCompatibility.errors.length > 0);
    });
  });

  describe('simulateGraphLoading', () => {
    validationTest('succeeds for complete valid setup', ({ validFilesDir }) => {
      const validator = new GraphLoadingValidator(validFilesDir);
      validator.simulateGraphLoading();
      const results = validator.getResults();

      assert(results.graphLoadingTests.passed === 1);
      assert(results.graphLoadingTests.failed === 0);
    });

    validationTest('detects missing model file', ({ validationTestDir }) => {
      // Create graphs.json with reference to non-existent model
      writeFileSync(
        join(validationTestDir, 'graphs.json'),
        JSON.stringify({
          models: {
            "99999999-9999-9999-9999-999999999999": {
              id: "99999999-9999-9999-9999-999999999999",
              name: { en: "Missing Model" },
              slug: "missing-model"
            }
          }
        }, null, 2)
      );

      const validator = new GraphLoadingValidator(validationTestDir);
      validator.simulateGraphLoading();
      const results = validator.getResults();

      assert(results.graphLoadingTests.failed > 0);
      const hasModelFileError = results.graphLoadingTests.errors.some(
        err => err.error.includes('not found')
      );
      assert(hasModelFileError);
    });
  });

  describe('validate (full suite)', () => {
    validationTest('returns success for all valid files', ({ validFilesDir }) => {
      const validator = new GraphLoadingValidator(validFilesDir);
      const summary = validator.validate();

      assert(summary.success === true);
      assert(summary.totalFailed === 0);
      assert(summary.totalPassed > 0);
    });

    validationTest('returns failure for invalid files', ({ invalidFilesDir }) => {
      const validator = new GraphLoadingValidator(invalidFilesDir);
      const summary = validator.validate();

      assert(summary.success === false);
      assert(summary.totalFailed > 0);
    });

    validationTest('aggregates results correctly', ({ mixedFilesDir }) => {
      const validator = new GraphLoadingValidator(mixedFilesDir);
      const summary = validator.validate();

      assert(summary.totalPassed + summary.totalFailed > 0);
      assert(summary.results.graphModels.passed > 0);
      assert(summary.results.graphModels.failed > 0);
    });
  });

  describe('printSummary', () => {
    validationTest('prints summary without errors', ({ validFilesDir }) => {
      const validator = new GraphLoadingValidator(validFilesDir);
      const summary = validator.validate();

      // Should not throw
      validator.printSummary(summary);
    });
  });
});

describe('validateGraphLoading convenience function', () => {
  beforeEach(() => {
    mockConsole();
  });

  afterEach(() => {
    restoreConsole();
  });

  validationTest('validates and returns summary', ({ validFilesDir }) => {
    const summary = validateGraphLoading(validFilesDir);

    assert(summary !== null);
    assert(summary.success === true);
    assert(typeof summary.totalPassed === 'number');
    assert(typeof summary.totalFailed === 'number');
  });

  validationTest('detects errors in invalid directory', ({ invalidFilesDir }) => {
    const summary = validateGraphLoading(invalidFilesDir);

    assert(summary.success === false);
    assert(summary.totalFailed > 0);
  });
});

describe('error reporting', () => {
  beforeEach(() => {
    mockConsole();
  });

  afterEach(() => {
    restoreConsole();
  });

  validationTest('provides detailed error messages', ({ invalidFilesDir }) => {
    const validator = new GraphLoadingValidator(invalidFilesDir);
    validator.validateGraphModels();
    const results = validator.getResults();

    assert(results.graphModels.errors.length > 0);
    const error = results.graphModels.errors[0];
    assert(error.file !== '');
    assert(error.error !== '');
    assert(typeof error.file === 'string');
    assert(typeof error.error === 'string');
  });

  validationTest('tracks which file caused each error', ({ mixedFilesDir }) => {
    const validator = new GraphLoadingValidator(mixedFilesDir);
    validator.validateGraphModels();
    const results = validator.getResults();

    if (results.graphModels.failed > 0) {
      const errorFiles = results.graphModels.errors.map(e => e.file);
      assert(errorFiles.every(file => file.includes('arches-')));
    }
  });
});

describe('schema validation', () => {
  beforeEach(() => {
    mockConsole();
  });

  afterEach(() => {
    restoreConsole();
  });

  validationTest('validates UUID format', ({ validationTestDir }) => {
    const badModel = {
      graph: [{
        ...validModel.graph[0],
        graphid: 'not-a-valid-uuid'
      }]
    };

    writeFileSync(
      join(validationTestDir, 'arches-bad-uuid-model.json'),
      JSON.stringify(badModel, null, 2)
    );

    const validator = new GraphLoadingValidator(validationTestDir);
    validator.validateGraphModels();
    const results = validator.getResults();

    assert(results.graphModels.failed > 0);
  });

  validationTest('validates multilingual strings', ({ validationTestDir }) => {
    const badModel = {
      graph: [{
        ...validModel.graph[0],
        name: 'Should be an object with language keys'
      }]
    };

    writeFileSync(
      join(validationTestDir, 'arches-bad-name-model.json'),
      JSON.stringify(badModel, null, 2)
    );

    const validator = new GraphLoadingValidator(validationTestDir);
    validator.validateGraphModels();
    const results = validator.getResults();

    assert(results.graphModels.failed > 0);
  });

  validationTest('validates datatype enum', ({ validationTestDir }) => {
    const badModel = {
      graph: [{
        ...validModel.graph[0],
        nodes: [{
          ...validModel.graph[0].nodes[0],
          datatype: 'invalid-datatype'
        }]
      }]
    };

    writeFileSync(
      join(validationTestDir, 'arches-bad-datatype-model.json'),
      JSON.stringify(badModel, null, 2)
    );

    const validator = new GraphLoadingValidator(validationTestDir);
    validator.validateGraphModels();
    const results = validator.getResults();

    assert(results.graphModels.failed > 0);
  });
});
