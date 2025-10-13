/**
 * Alizarin Validation Module
 *
 * Provides validation utilities for Arches graph models and business data files.
 * Ensures compatibility with Alizarin's loading requirements.
 *
 * @module validation
 */

// Export validators
export {
  GraphLoadingValidator,
  validateGraphLoading,
  type ValidationResult,
  type ValidationResults,
  type ValidationSummary
} from './validators/index.js';

// Export schemas
import graphModelSchema from './schemas/graphModel.json';
import businessDataSchema from './schemas/businessData.json';
import graphsRegistrySchema from './schemas/graphsRegistry.json';

export const schemas = {
  graphModel: graphModelSchema,
  businessData: businessDataSchema,
  graphsRegistry: graphsRegistrySchema
};

/**
 * Quick validation function for common use cases
 *
 * @example
 * ```typescript
 * import { quickValidate } from 'alizarin/validation';
 *
 * const result = quickValidate('./data');
 * if (result.success) {
 *   console.log('All files are valid!');
 * }
 * ```
 */
export { validateGraphLoading as quickValidate } from './validators/index.js';
