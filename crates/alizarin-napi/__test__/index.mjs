import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';

import { NapiPrebuildLoader, NapiStaticGraph, NapiStaticResourceRegistry } from '../index.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TEST_DATA = path.resolve(__dirname, '..', '..', '..', 'tests');

describe('NapiStaticGraph', () => {
  it('parses a graph from JSON string', () => {
    const graphJson = fs.readFileSync(
      path.join(TEST_DATA, 'data', 'models', 'Person.json'),
      'utf-8'
    );
    const graph = NapiStaticGraph.fromJsonString(graphJson);
    assert.ok(graph.graphId, 'graph should have an ID');
    assert.ok(graph.name, 'graph should have a name');
  });

  it('throws on invalid JSON', () => {
    assert.throws(() => NapiStaticGraph.fromJsonString('not json'), /error/i);
  });
});

describe('NapiStaticResourceRegistry', () => {
  it('starts empty', () => {
    const registry = new NapiStaticResourceRegistry();
    assert.equal(registry.length, 0);
  });

  it('loads resources from business data JSON', () => {
    const businessData = {
      business_data: {
        resources: [
          {
            resourceinstance: {
              resourceinstanceid: '87654321-4321-4321-4321-cba987654321',
              graph_id: '12345678-1234-1234-1234-123456789abc',
              name: 'Test Resource',
              descriptors: {},
            },
            tiles: [],
          },
        ],
      },
    };

    const registry = new NapiStaticResourceRegistry();
    registry.mergeFromBusinessDataJson(JSON.stringify(businessData), true);
    assert.ok(registry.length > 0, 'registry should have resources after merge');
  });

  it('contains returns false for unknown IDs', () => {
    const registry = new NapiStaticResourceRegistry();
    assert.equal(registry.contains('nonexistent'), false);
  });
});

describe('NapiPrebuildLoader', () => {
  it('throws for nonexistent directory', () => {
    assert.throws(
      () => new NapiPrebuildLoader('/nonexistent/path'),
      /not found/i
    );
  });
});
