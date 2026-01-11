import { describe, it, expect, beforeAll } from 'vitest';
import {
  coerceValue,
  coerceNumber,
  coerceString,
  coerceDate,
  coerceConceptValue,
  coerceDomainValue,
  coerceDomainValueList,
  coerceResourceInstance,
  coerceResourceInstanceList,
  WasmCoercionResult
} from '../pkg/alizarin';
import { initWasmForTests } from './wasm-init';
import * as fs from 'fs';
import * as path from 'path';

// Load Person model for realistic node configs
const personModelPath = path.join(__dirname, 'data/models/Person.json');
const personModel = JSON.parse(fs.readFileSync(personModelPath, 'utf-8'));
const personGraph = personModel.graph[0];

// Helper to find a node by alias
function findNodeByAlias(alias: string) {
  return personGraph.nodes.find((n: any) => n.alias === alias);
}

// Helper to convert Map to plain object (WASM returns Maps for JSON objects)
function mapToObject(value: any): any {
  if (value instanceof Map) {
    const obj: Record<string, any> = {};
    for (const [k, v] of value) {
      obj[k] = mapToObject(v);
    }
    return obj;
  }
  if (Array.isArray(value)) {
    return value.map(mapToObject);
  }
  return value;
}

// Helper to assert coercion result
function assertCoercionResult(
  result: WasmCoercionResult,
  expectedTileData: any,
  expectedDisplayValue?: any
) {
  expect(result.isError).toBe(false);
  expect(result.isNull).toBe(false);
  expect(mapToObject(result.tileData)).toEqual(expectedTileData);
  if (expectedDisplayValue !== undefined) {
    expect(mapToObject(result.displayValue)).toEqual(expectedDisplayValue);
  }
}

describe('Type Coercion Integration Tests', () => {
  beforeAll(async () => {
    await initWasmForTests();
  });

  describe('Person Model Node Lookups', () => {
    it('should find expected nodes in Person model', () => {
      const forenameNode = findNodeByAlias('forename');
      expect(forenameNode).toBeDefined();
      expect(forenameNode.datatype).toBe('string');

      const numberNode = findNodeByAlias('primary_reference_number');
      expect(numberNode).toBeDefined();
      expect(numberNode.datatype).toBe('number');

      const dateNode = findNodeByAlias('creation_date');
      expect(dateNode).toBeDefined();
      expect(dateNode.datatype).toBe('date');

      const conceptNode = findNodeByAlias('forename_name_type');
      expect(conceptNode).toBeDefined();
      expect(conceptNode.datatype).toBe('concept');

      const resourceNode = findNodeByAlias('favourite_activity');
      expect(resourceNode).toBeDefined();
      expect(resourceNode.datatype).toBe('resource-instance');

      const resourceListNode = findNodeByAlias('associated_activities');
      expect(resourceListNode).toBeDefined();
      expect(resourceListNode.datatype).toBe('resource-instance-list');
    });
  });

  describe('String coercion for Person forename', () => {
    const forenameNode = () => findNodeByAlias('forename');

    it('should coerce a plain string to localized format', () => {
      const result = coerceValue('string', 'John', null);

      // Both tile data and display value are the localized object
      assertCoercionResult(result, { en: 'John' }, { en: 'John' });
    });

    it('should coerce a localized string object as-is', () => {
      const result = coerceValue('string', { en: 'John', fr: 'Jean' }, null);

      assertCoercionResult(result, { en: 'John', fr: 'Jean' });
    });

    it('should coerce number input to localized string', () => {
      const result = coerceValue('string', 42, null);

      // String coercion now auto-converts numbers to localized strings
      expect(result.isError).toBe(false);
      assertCoercionResult(result, { en: '42' });
    });

    it('should handle null input', () => {
      const result = coerceValue('string', null, null);

      expect(result.isNull).toBe(true);
      expect(result.isError).toBe(false);
    });
  });

  describe('Number coercion for Person primary_reference_number', () => {
    it('should coerce an integer', () => {
      const result = coerceValue('number', 12345, null);

      assertCoercionResult(result, 12345, 12345);
    });

    it('should coerce a float', () => {
      const result = coerceValue('number', 123.45, null);

      assertCoercionResult(result, 123.45, 123.45);
    });

    it('should coerce a numeric string', () => {
      const result = coerceValue('number', '12345', null);

      assertCoercionResult(result, 12345, 12345);
    });

    it('should coerce a numeric string with decimals', () => {
      const result = coerceValue('number', '123.45', null);

      assertCoercionResult(result, 123.45, 123.45);
    });

    it('should error on non-numeric string', () => {
      const result = coerceValue('number', 'not a number', null);

      expect(result.isError).toBe(true);
      expect(result.error).toContain('Cannot parse');
    });

    it('should handle null input', () => {
      const result = coerceValue('number', null, null);

      expect(result.isNull).toBe(true);
      expect(result.isError).toBe(false);
    });
  });

  describe('Date coercion for Person creation_date', () => {
    const dateNode = () => findNodeByAlias('creation_date');

    it('should coerce an ISO date string', () => {
      const result = coerceValue('date', '2024-01-15', dateNode()?.config);

      assertCoercionResult(result, '2024-01-15', '2024-01-15');
    });

    it('should coerce a datetime string (preserves full value)', () => {
      const result = coerceValue('date', '2024-01-15T10:30:00Z', dateNode()?.config);

      // Date coercion preserves the full datetime string
      assertCoercionResult(result, '2024-01-15T10:30:00Z', '2024-01-15T10:30:00Z');
    });

    it('should handle various date formats', () => {
      // These should all normalize to ISO format
      const testCases = [
        '2024-01-15',
        '15/01/2024',  // DD/MM/YYYY
        '01/15/2024',  // MM/DD/YYYY
      ];

      for (const dateStr of testCases) {
        const result = coerceValue('date', dateStr, dateNode()?.config);
        // Either it coerces successfully or returns an error for ambiguous formats
        expect(result.isNull).toBe(false);
      }
    });

    it('should handle null input', () => {
      const result = coerceValue('date', null, dateNode()?.config);

      expect(result.isNull).toBe(true);
      expect(result.isError).toBe(false);
    });
  });

  describe('Concept coercion for Person forename_name_type', () => {
    const conceptNode = () => findNodeByAlias('forename_name_type');

    it('should coerce a valid UUID', () => {
      const validUuid = '550e8400-e29b-41d4-a716-446655440000';
      const result = coerceValue('concept', validUuid, conceptNode()?.config);

      assertCoercionResult(result, validUuid, validUuid);
    });

    it('should error on invalid UUID', () => {
      const result = coerceValue('concept', 'not-a-uuid', conceptNode()?.config);

      expect(result.isError).toBe(true);
      expect(result.error).toContain('must be a UUID');
    });

    it('should extract UUID from object with id field', () => {
      const validUuid = '550e8400-e29b-41d4-a716-446655440000';
      const input = { id: validUuid };
      const result = coerceValue('concept', input, conceptNode()?.config);

      // Tile data is the UUID, display value is the original object
      expect(result.isError).toBe(false);
      expect(mapToObject(result.tileData)).toEqual(validUuid);
      expect(mapToObject(result.displayValue)).toEqual(input);
    });

    it('should handle null input', () => {
      const result = coerceValue('concept', null, conceptNode()?.config);

      expect(result.isNull).toBe(true);
      expect(result.isError).toBe(false);
    });
  });

  describe('Resource Instance coercion for Person favourite_activity', () => {
    const resourceNode = () => findNodeByAlias('favourite_activity');

    it('should coerce a plain UUID to resource-instance format', () => {
      const validUuid = '550e8400-e29b-41d4-a716-446655440000';
      const result = coerceValue('resource-instance', validUuid, resourceNode()?.config);

      // Resource instance format: [{resourceId: uuid}]
      assertCoercionResult(result, [{ resourceId: validUuid }]);
    });

    it('should accept resource-instance format as-is', () => {
      const validUuid = '550e8400-e29b-41d4-a716-446655440000';
      const input = [{ resourceId: validUuid }];
      const result = coerceValue('resource-instance', input, resourceNode()?.config);

      assertCoercionResult(result, input);
    });

    it('should coerce resourceId object', () => {
      const validUuid = '550e8400-e29b-41d4-a716-446655440000';
      const result = coerceValue('resource-instance', { resourceId: validUuid }, resourceNode()?.config);

      assertCoercionResult(result, [{ resourceId: validUuid }]);
    });

    it('should error on invalid UUID', () => {
      const result = coerceValue('resource-instance', 'not-a-uuid', resourceNode()?.config);

      expect(result.isError).toBe(true);
    });

    it('should handle null input', () => {
      const result = coerceValue('resource-instance', null, resourceNode()?.config);

      expect(result.isNull).toBe(true);
      expect(result.isError).toBe(false);
    });
  });

  describe('Resource Instance List coercion for Person associated_activities', () => {
    const resourceListNode = () => findNodeByAlias('associated_activities');

    it('should coerce an array of UUIDs', () => {
      const uuid1 = '550e8400-e29b-41d4-a716-446655440000';
      const uuid2 = '660e8400-e29b-41d4-a716-446655440001';
      const result = coerceValue('resource-instance-list', [uuid1, uuid2], resourceListNode()?.config);

      assertCoercionResult(result, [
        { resourceId: uuid1 },
        { resourceId: uuid2 }
      ]);
    });

    it('should coerce an array of resourceId objects', () => {
      const uuid1 = '550e8400-e29b-41d4-a716-446655440000';
      const uuid2 = '660e8400-e29b-41d4-a716-446655440001';
      const input = [{ resourceId: uuid1 }, { resourceId: uuid2 }];
      const result = coerceValue('resource-instance-list', input, resourceListNode()?.config);

      assertCoercionResult(result, input);
    });

    it('should error on single UUID (requires array)', () => {
      const validUuid = '550e8400-e29b-41d4-a716-446655440000';
      const result = coerceValue('resource-instance-list', validUuid, resourceListNode()?.config);

      // Resource instance list requires an array
      expect(result.isError).toBe(true);
    });

    it('should handle empty array', () => {
      const result = coerceValue('resource-instance-list', [], resourceListNode()?.config);

      assertCoercionResult(result, []);
    });

    it('should error on invalid UUID in array', () => {
      const result = coerceValue('resource-instance-list', ['not-a-uuid'], resourceListNode()?.config);

      expect(result.isError).toBe(true);
    });

    it('should handle null input', () => {
      const result = coerceValue('resource-instance-list', null, resourceListNode()?.config);

      expect(result.isNull).toBe(true);
      expect(result.isError).toBe(false);
    });
  });

  describe('Domain Value coercion', () => {
    const validUuid = '550e8400-e29b-41d4-a716-446655440000';
    const validUuid2 = '660e8400-e29b-41d4-a716-446655440001';

    // Config with domain options
    const domainConfig = {
      options: [
        { id: validUuid, text: { en: 'Option 1' }, selected: false },
        { id: validUuid2, text: { en: 'Option 2' }, selected: false }
      ]
    };

    describe('domain-value coercion', () => {
      it('should coerce a valid UUID without config', () => {
        const result = coerceValue('domain-value', validUuid, null);

        expect(result.isError).toBe(false);
        expect(mapToObject(result.tileData)).toEqual(validUuid);
      });

      it('should coerce a valid UUID with config and resolve display value', () => {
        const result = coerceValue('domain-value', validUuid, domainConfig);

        expect(result.isError).toBe(false);
        expect(mapToObject(result.tileData)).toEqual(validUuid);
        // Display value should be the resolved domain option
        const displayVal = mapToObject(result.displayValue);
        expect(displayVal.id).toEqual(validUuid);
        expect(displayVal.text).toEqual({ en: 'Option 1' });
      });

      it('should error on UUID not found in config', () => {
        const unknownUuid = '770e8400-e29b-41d4-a716-446655440002';
        const result = coerceValue('domain-value', unknownUuid, domainConfig);

        expect(result.isError).toBe(true);
        expect(result.error).toContain('not found');
      });

      it('should coerce domain value object by extracting id', () => {
        const input = { id: validUuid, text: { en: 'Already resolved' } };
        const result = coerceValue('domain-value', input, null);

        expect(result.isError).toBe(false);
        expect(mapToObject(result.tileData)).toEqual(validUuid);
      });

      it('should handle null input', () => {
        const result = coerceValue('domain-value', null, domainConfig);

        expect(result.isNull).toBe(true);
        expect(result.isError).toBe(false);
      });

      it('should handle empty string as null', () => {
        const result = coerceValue('domain-value', '', domainConfig);

        expect(result.isNull).toBe(true);
        expect(result.isError).toBe(false);
      });

      it('should error on invalid UUID format', () => {
        const result = coerceValue('domain-value', 'not-a-uuid', null);

        expect(result.isError).toBe(true);
        expect(result.error).toContain('UUID');
      });

      it('should error on number input', () => {
        const result = coerceValue('domain-value', 12345, null);

        expect(result.isError).toBe(true);
      });
    });

    describe('domain-value-list coercion', () => {
      it('should coerce an array of UUIDs', () => {
        const result = coerceValue('domain-value-list', [validUuid, validUuid2], null);

        expect(result.isError).toBe(false);
        expect(mapToObject(result.tileData)).toEqual([validUuid, validUuid2]);
      });

      it('should coerce an array of UUIDs with config', () => {
        const result = coerceValue('domain-value-list', [validUuid, validUuid2], domainConfig);

        expect(result.isError).toBe(false);
        expect(mapToObject(result.tileData)).toEqual([validUuid, validUuid2]);
        // Display values should be resolved
        const displayVals = mapToObject(result.displayValue);
        expect(displayVals).toHaveLength(2);
        expect(displayVals[0].id).toEqual(validUuid);
        expect(displayVals[1].id).toEqual(validUuid2);
      });

      it('should handle empty array', () => {
        const result = coerceValue('domain-value-list', [], null);

        expect(result.isError).toBe(false);
        expect(mapToObject(result.tileData)).toEqual([]);
      });

      it('should handle null input', () => {
        const result = coerceValue('domain-value-list', null, domainConfig);

        expect(result.isNull).toBe(true);
        expect(result.isError).toBe(false);
      });

      it('should error on invalid UUID in array', () => {
        const result = coerceValue('domain-value-list', [validUuid, 'not-a-uuid'], null);

        expect(result.isError).toBe(true);
      });

      it('should error on single UUID (requires array)', () => {
        const result = coerceValue('domain-value-list', validUuid, null);

        expect(result.isError).toBe(true);
      });

      it('should coerce array of domain value objects', () => {
        const input = [
          { id: validUuid, text: { en: 'Option 1' } },
          { id: validUuid2, text: { en: 'Option 2' } }
        ];
        const result = coerceValue('domain-value-list', input, null);

        expect(result.isError).toBe(false);
        expect(mapToObject(result.tileData)).toEqual([validUuid, validUuid2]);
      });
    });

    describe('direct coercion functions', () => {
      it('should use coerceDomainValue directly', () => {
        const result = coerceDomainValue(validUuid, null);

        expect(result.isError).toBe(false);
        expect(mapToObject(result.tileData)).toEqual(validUuid);
      });

      it('should use coerceDomainValue with config', () => {
        const result = coerceDomainValue(validUuid, domainConfig);

        expect(result.isError).toBe(false);
        const displayVal = mapToObject(result.displayValue);
        expect(displayVal.text).toEqual({ en: 'Option 1' });
      });

      it('should use coerceDomainValueList directly', () => {
        const result = coerceDomainValueList([validUuid, validUuid2], null);

        expect(result.isError).toBe(false);
        expect(mapToObject(result.tileData)).toEqual([validUuid, validUuid2]);
      });

      it('should use coerceDomainValueList with config', () => {
        const result = coerceDomainValueList([validUuid], domainConfig);

        expect(result.isError).toBe(false);
        const displayVals = mapToObject(result.displayValue);
        expect(displayVals[0].text).toEqual({ en: 'Option 1' });
      });
    });
  });

  describe('Using direct coercion functions', () => {
    it('should use coerceNumber directly', () => {
      const result = coerceNumber(42);

      assertCoercionResult(result, 42, 42);
    });

    it('should use coerceString directly', () => {
      const result = coerceString('Hello', 'en');

      assertCoercionResult(result, { en: 'Hello' }, { en: 'Hello' });
    });

    it('should use coerceDate directly', () => {
      const result = coerceDate('2024-01-15');

      assertCoercionResult(result, '2024-01-15', '2024-01-15');
    });

    it('should use coerceConceptValue directly', () => {
      const validUuid = '550e8400-e29b-41d4-a716-446655440000';
      const result = coerceConceptValue(validUuid, null);

      assertCoercionResult(result, validUuid, validUuid);
    });

    it('should use coerceResourceInstance directly', () => {
      const validUuid = '550e8400-e29b-41d4-a716-446655440000';
      const result = coerceResourceInstance(validUuid, null);

      assertCoercionResult(result, [{ resourceId: validUuid }]);
    });

    it('should use coerceResourceInstanceList directly', () => {
      const uuid1 = '550e8400-e29b-41d4-a716-446655440000';
      const uuid2 = '660e8400-e29b-41d4-a716-446655440001';
      const result = coerceResourceInstanceList([uuid1, uuid2], null);

      assertCoercionResult(result, [
        { resourceId: uuid1 },
        { resourceId: uuid2 }
      ]);
    });
  });

  describe('Building tile data from plain JSON', () => {
    it('should build tile data for a Person with forename', () => {
      // Given: User provides plain JSON
      const plainInput = {
        forename: 'John',
        primary_reference_number: 12345,
        creation_date: '2024-01-15'
      };

      // When: We coerce each field using the appropriate datatype
      const forenameResult = coerceValue('string', plainInput.forename, null);
      const numberResult = coerceValue('number', plainInput.primary_reference_number, null);
      const dateResult = coerceValue('date', plainInput.creation_date, null);

      // Then: We get properly formatted tile data
      expect(mapToObject(forenameResult.tileData)).toEqual({ en: 'John' });
      expect(mapToObject(numberResult.tileData)).toEqual(12345);
      expect(mapToObject(dateResult.tileData)).toEqual('2024-01-15');
    });

    it('should build tile data for a Person with related activities', () => {
      const activityId1 = '550e8400-e29b-41d4-a716-446655440000';
      const activityId2 = '660e8400-e29b-41d4-a716-446655440001';

      // User provides plain UUIDs for related resources
      const plainInput = {
        favourite_activity: activityId1,
        associated_activities: [activityId1, activityId2]
      };

      const favouriteResult = coerceValue('resource-instance', plainInput.favourite_activity, null);
      const associatedResult = coerceValue('resource-instance-list', plainInput.associated_activities, null);

      // Tile data is in proper Arches format
      expect(mapToObject(favouriteResult.tileData)).toEqual([{ resourceId: activityId1 }]);
      expect(mapToObject(associatedResult.tileData)).toEqual([
        { resourceId: activityId1 },
        { resourceId: activityId2 }
      ]);
    });

    it('should handle mixed coerceable values in a tile', () => {
      const conceptId = '550e8400-e29b-41d4-a716-446655440000';

      // Simulate building a complete tile for a Person's forename group
      const tileInput = {
        forename: 'Jane',                    // Will be coerced to localized string
        forename_name_type: conceptId        // Will be validated as UUID
      };

      const forenameNode = findNodeByAlias('forename');
      const nameTypeNode = findNodeByAlias('forename_name_type');

      const forenameResult = coerceValue(forenameNode.datatype, tileInput.forename, forenameNode.config);
      const nameTypeResult = coerceValue(nameTypeNode.datatype, tileInput.forename_name_type, nameTypeNode.config);

      // Build the tile data object
      const tileData: Record<string, any> = {};
      tileData[forenameNode.nodeid] = mapToObject(forenameResult.tileData);
      tileData[nameTypeNode.nodeid] = mapToObject(nameTypeResult.tileData);

      expect(tileData[forenameNode.nodeid]).toEqual({ en: 'Jane' });
      expect(tileData[nameTypeNode.nodeid]).toEqual(conceptId);
    });
  });

  describe('Error handling', () => {
    it('should pass through value for unknown datatype', () => {
      const result = coerceValue('unknown-datatype', 'value', null);

      // Unknown datatypes pass through - this is intentional
      // to avoid breaking on new datatypes
      expect(result.isError).toBe(false);
      expect(mapToObject(result.tileData)).toEqual('value');
    });

    it('should provide error details for type mismatches', () => {
      // Try to coerce an object to number
      const result = coerceValue('number', { key: 'value' }, null);

      expect(result.isError).toBe(true);
      expect(result.error).toBeDefined();
    });
  });
});
