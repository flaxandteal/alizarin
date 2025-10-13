# Alizarin Validation Module

Comprehensive validation utilities for Arches graph models and business data files, ensuring compatibility with Alizarin's loading requirements.

## Features

- ✅ **Graph Model Validation**: Validates Arches graph model structure against JSON Schema
- ✅ **Business Data Validation**: Ensures business data files have correct format and metadata
- ✅ **Registry Validation**: Validates the graphs.json registry file
- ✅ **Alizarin Compatibility**: Checks for required metadata and structure needed by Alizarin
- ✅ **Graph Loading Simulation**: Simulates the actual loading process to catch integration issues

## Installation

```bash
npm install alizarin
```

The validation module requires:
- `ajv` - JSON Schema validator
- `ajv-formats` - Additional format validators for ajv

These are automatically installed as dependencies.

## Usage

### Programmatic Usage

```typescript
import { validateGraphLoading, GraphLoadingValidator } from 'alizarin/validation';

// Quick validation with auto-printed summary
const result = validateGraphLoading('./data');
if (result.success) {
  console.log('All validation checks passed!');
}

// Advanced usage with custom handling
const validator = new GraphLoadingValidator('./data');
const summary = validator.validate();

// Access detailed results
console.log('Graph models:', summary.results.graphModels);
console.log('Business data:', summary.results.businessData);

// Run specific validation checks
validator.validateGraphModels();
validator.checkAlizarinCompatibility();
```

### Command Line Usage

Create a script to run validation from the command line:

```javascript
// validate.mjs
import { validateGraphLoading } from 'alizarin/validation';

const basePath = process.argv[2] || '.';
const result = validateGraphLoading(basePath);

process.exit(result.success ? 0 : 1);
```

Run it:
```bash
node validate.mjs ./my-data-directory
```

## API Reference

### `validateGraphLoading(basePath: string): ValidationSummary`

Convenience function that runs all validation checks and prints a summary.

**Parameters:**
- `basePath` - Base directory containing Arches files (default: `'.'`)

**Returns:** `ValidationSummary` object with:
- `results` - Detailed results for each validation category
- `totalPassed` - Total number of checks that passed
- `totalFailed` - Total number of checks that failed
- `success` - Boolean indicating if all checks passed

### `GraphLoadingValidator`

Class providing fine-grained control over validation.

#### Constructor

```typescript
new GraphLoadingValidator(basePath: string = '.')
```

#### Methods

- **`validate(): ValidationSummary`** - Run all validation checks
- **`validateGraphModels(): void`** - Validate graph model files only
- **`validateBusinessDataFiles(): void`** - Validate business data files only
- **`validateGraphsRegistryFile(): void`** - Validate graphs.json registry
- **`checkAlizarinCompatibility(): void`** - Check Alizarin-specific requirements
- **`simulateGraphLoading(): void`** - Simulate the graph loading process
- **`getResults(): ValidationResults`** - Get current validation results
- **`printSummary(summary: ValidationSummary): void`** - Print formatted summary

## Validation Checks

### 1. Graph Model Validation

Validates `arches-*-model.json` files against the graph model schema:

- Required fields (graphid, name, nodes, nodegroups, edges)
- UUID format validation
- Node structure (istopnode, datatype, nodegroup_id)
- Nodegroup structure and relationships
- Top node must have `nodegroup_id: null` (Alizarin requirement)
- Card definitions

### 2. Business Data Validation

Validates `arches-business-data-*.json` files:

- Required top-level structure (`business_data.resources`)
- Resource structure (resourceid, graph_id, tiles, resourceinstance)
- Tile structure (tileid, nodegroup_id, data)
- UUID format validation
- Data values format (strings, numbers, multilingual objects)

### 3. Graphs Registry Validation

Validates `graphs.json` registry file:

- Required models object
- Model reference structure (id, name, slug)
- Slug format validation
- Color format validation (hex colors)
- Icon class format validation

### 4. Alizarin Compatibility Checks

Ensures files meet Alizarin's specific requirements:

- **Graph ID References**: All business data graph_ids exist in registry
- **Resource Instance Metadata**: Required fields (legacyid, name, displayname)
- **Descriptors Object**: Required fields (name, description, map_popup, displayname)
- **Complete Metadata**: All resources have proper display information

### 5. Graph Loading Simulation

Simulates Alizarin's actual loading process:

- Model files exist for all registered graphs
- Graph IDs match between registry and model files
- Business data references valid graphs
- Files can be parsed without errors

## File Naming Conventions

The validator expects files to follow Arches naming conventions:

- **Graph Models**: `arches-{slug}-model.json`
- **Business Data**: `arches-business-data-{name}.json`
- **Registry**: `graphs.json`

## Validation Results

Each validation category returns results with:

```typescript
interface ValidationResult {
  passed: number;      // Number of checks that passed
  failed: number;      // Number of checks that failed
  errors: Array<{      // Detailed error information
    file: string;      // File that failed validation
    error: string;     // Error message
  }>;
}
```

## Schemas

The module includes three JSON Schema files:

- **`graphModel.json`** - Schema for Arches graph model files
- **`businessData.json`** - Schema for Arches business data files
- **`graphsRegistry.json`** - Schema for graphs.json registry

Access schemas programmatically:

```typescript
import { schemas } from 'alizarin/validation';

console.log(schemas.graphModel);
console.log(schemas.businessData);
console.log(schemas.graphsRegistry);
```

## Examples

### Validate Before Loading

```typescript
import { validateGraphLoading } from 'alizarin/validation';
import { AlizarinClient } from 'alizarin';

// Validate first
const validation = validateGraphLoading('./data');
if (!validation.success) {
  console.error('Validation failed, aborting load');
  process.exit(1);
}

// Load with confidence
const client = new AlizarinClient();
await client.loadGraphs('./data');
```

### Custom Error Handling

```typescript
import { GraphLoadingValidator } from 'alizarin/validation';

const validator = new GraphLoadingValidator('./data');
const summary = validator.validate();

// Check specific validation categories
if (summary.results.alizarinCompatibility.failed > 0) {
  console.error('Alizarin compatibility issues detected:');
  summary.results.alizarinCompatibility.errors.forEach(err => {
    console.error(`  ${err.file}: ${err.error}`);
  });
}

// Generate custom reports
const graphModelErrors = summary.results.graphModels.errors;
if (graphModelErrors.length > 0) {
  // Send to error tracking service, generate report, etc.
}
```

### Integration Testing

```typescript
import { GraphLoadingValidator } from 'alizarin/validation';
import { describe, it, expect } from 'vitest';

describe('Data Validation', () => {
  it('should validate all graph models', () => {
    const validator = new GraphLoadingValidator('./test-data');
    validator.validateGraphModels();
    const results = validator.getResults();
    expect(results.graphModels.failed).toBe(0);
  });

  it('should pass Alizarin compatibility checks', () => {
    const validator = new GraphLoadingValidator('./test-data');
    validator.checkAlizarinCompatibility();
    const results = validator.getResults();
    expect(results.alizarinCompatibility.failed).toBe(0);
  });
});
```

## Common Issues and Solutions

### Issue: "Top node must have nodegroup_id: null"

**Solution:** Ensure nodes with `istopnode: true` have `nodegroup_id` set to `null`:

```json
{
  "nodeid": "...",
  "istopnode": true,
  "nodegroup_id": null
}
```

### Issue: "Missing resourceinstance metadata"

**Solution:** Add required Alizarin metadata to business data resources:

```json
{
  "resourceid": "...",
  "resourceinstance": {
    "resourceinstanceid": "...",
    "legacyid": "unique-identifier",
    "name": "Display Name",
    "displayname": "Display Name",
    "descriptors": {
      "name": "Display Name",
      "description": "Resource description",
      "map_popup": "Popup text",
      "displayname": "Display Name"
    }
  }
}
```

### Issue: "Graph ID mismatch"

**Solution:** Ensure graph IDs match between `graphs.json` and model files:

```javascript
// In graphs.json
{
  "models": {
    "abc123...": { "slug": "my-model" }
  }
}

// In arches-my-model-model.json
{
  "graph": [{
    "graphid": "abc123..."  // Must match
  }]
}
```

## License

AGPL-3.0-or-later
