import Ajv, { type ErrorObject } from 'ajv';
import addFormats from 'ajv-formats';
import { readFileSync, readdirSync, existsSync } from 'fs';
import { join } from 'path';
import graphModelSchema from '../schemas/graphModel.json';
import businessDataSchema from '../schemas/businessData.json';
import graphsRegistrySchema from '../schemas/graphsRegistry.json';

export interface ValidationResult {
  passed: number;
  failed: number;
  errors: Array<{ file: string; error: string }>;
}

export interface ValidationResults {
  graphModels: ValidationResult;
  businessData: ValidationResult;
  graphsRegistry: ValidationResult;
  alizarinCompatibility: ValidationResult;
  graphLoadingTests: ValidationResult;
}

export interface ValidationSummary {
  results: ValidationResults;
  totalPassed: number;
  totalFailed: number;
  success: boolean;
}

export class GraphLoadingValidator {
  private ajv: Ajv;
  private validateGraphModel: any;
  private validateBusinessData: any;
  private validateGraphsRegistry: any;
  private results: ValidationResults;
  private basePath: string;

  constructor(basePath: string = '.') {
    this.basePath = basePath;

    // Initialize AJV with format support (strict mode off for cleaner output)
    this.ajv = new Ajv({
      allErrors: true,
      verbose: true,
      strict: false
    });
    addFormats(this.ajv);

    // Compile validators
    this.validateGraphModel = this.ajv.compile(graphModelSchema);
    this.validateBusinessData = this.ajv.compile(businessDataSchema);
    this.validateGraphsRegistry = this.ajv.compile(graphsRegistrySchema);

    // Initialize results
    this.results = {
      graphModels: { passed: 0, failed: 0, errors: [] },
      businessData: { passed: 0, failed: 0, errors: [] },
      graphsRegistry: { passed: 0, failed: 0, errors: [] },
      alizarinCompatibility: { passed: 0, failed: 0, errors: [] },
      graphLoadingTests: { passed: 0, failed: 0, errors: [] }
    };
  }

  private reportError(category: keyof ValidationResults, file: string, error: string): void {
    this.results[category].failed++;
    this.results[category].errors.push({ file, error });
  }

  private reportSuccess(category: keyof ValidationResults, _file: string): void {
    this.results[category].passed++;
  }

  private formatErrors(errors: ErrorObject[] | null | undefined): string {
    if (!errors) return 'Unknown error';
    return errors
      .map(e => `${e.instancePath || 'root'}: ${e.message}`)
      .join('; ');
  }

  /**
   * Validate graph model files
   */
  public validateGraphModels(): void {
    const modelFiles = readdirSync(this.basePath).filter(f =>
      f.startsWith('arches-') && f.endsWith('-model.json')
    );

    modelFiles.forEach(file => {
      try {
        const content = JSON.parse(readFileSync(join(this.basePath, file), 'utf8'));

        if (this.validateGraphModel(content)) {
          this.reportSuccess('graphModels', file);
        } else {
          const error = this.formatErrors(this.validateGraphModel.errors);
          this.reportError('graphModels', file, error);
        }
      } catch (error) {
        this.reportError('graphModels', file, `Parse error: ${(error as Error).message}`);
      }
    });
  }

  /**
   * Validate business data files
   */
  public validateBusinessDataFiles(): void {
    const businessDataFiles = readdirSync(this.basePath).filter(f =>
      f.startsWith('arches-business-data-') &&
      f.endsWith('.json') &&
      f !== 'arches-business-data-schema.json'
    );

    businessDataFiles.forEach(file => {
      try {
        const content = JSON.parse(readFileSync(join(this.basePath, file), 'utf8'));

        if (this.validateBusinessData(content)) {
          this.reportSuccess('businessData', file);
        } else {
          const error = this.formatErrors(this.validateBusinessData.errors);
          this.reportError('businessData', file, error);
        }
      } catch (error) {
        this.reportError('businessData', file, `Parse error: ${(error as Error).message}`);
      }
    });
  }

  /**
   * Validate graphs registry file
   */
  public validateGraphsRegistryFile(): void {
    const registryPath = join(this.basePath, 'graphs.json');

    if (existsSync(registryPath)) {
      try {
        const content = JSON.parse(readFileSync(registryPath, 'utf8'));

        if (this.validateGraphsRegistry(content)) {
          this.reportSuccess('graphsRegistry', 'graphs.json');
        } else {
          const error = this.formatErrors(this.validateGraphsRegistry.errors);
          this.reportError('graphsRegistry', 'graphs.json', error);
        }
      } catch (error) {
        this.reportError('graphsRegistry', 'graphs.json', `Parse error: ${(error as Error).message}`);
      }
    } else {
      this.reportError('graphsRegistry', 'graphs.json', 'File does not exist');
    }
  }

  /**
   * Check Alizarin compatibility requirements
   */
  public checkAlizarinCompatibility(): void {
    const registryPath = join(this.basePath, 'graphs.json');

    if (!existsSync(registryPath)) {
      return;
    }

    const graphs = JSON.parse(readFileSync(registryPath, 'utf8'));
    const availableGraphIds = new Set(Object.keys(graphs.models || {}));

    const businessDataFiles = readdirSync(this.basePath).filter(f =>
      f.startsWith('arches-business-data-') &&
      f.endsWith('.json') &&
      f !== 'arches-business-data-schema.json'
    );

    businessDataFiles.forEach(file => {
      try {
        const content = JSON.parse(readFileSync(join(this.basePath, file), 'utf8'));
        const resources = content.business_data?.resources || [];

        resources.forEach((resource: any, index: number) => {
          const graphId = resource.graph_id;

          // Check graph ID exists
          if (!availableGraphIds.has(graphId)) {
            this.reportError('alizarinCompatibility', file,
              `Resource ${index}: graph_id ${graphId} not found in graphs.json`);
            return;
          }

          // Check resourceinstance structure for Alizarin
          const ri = resource.resourceinstance;
          if (!ri) {
            this.reportError('alizarinCompatibility', file,
              `Resource ${index}: Missing resourceinstance - required for Alizarin`);
            return;
          }

          // Check required Alizarin fields
          const requiredFields = ['resourceinstanceid', 'graph_id', 'legacyid', 'name', 'displayname', 'descriptors'];
          for (const field of requiredFields) {
            if (!ri[field]) {
              this.reportError('alizarinCompatibility', file,
                `Resource ${index}: Missing resourceinstance.${field} - required for Alizarin`);
              return;
            }
          }

          // Check descriptors structure
          const descriptors = ri.descriptors;
          const requiredDescriptors = ['name', 'description', 'map_popup', 'displayname'];
          for (const field of requiredDescriptors) {
            if (!descriptors[field]) {
              this.reportError('alizarinCompatibility', file,
                `Resource ${index}: Missing descriptors.${field} - required for Alizarin`);
              return;
            }
          }
        });

        if (resources.length > 0) {
          this.reportSuccess('alizarinCompatibility', file);
        }

      } catch (error) {
        this.reportError('alizarinCompatibility', file, `Parse error: ${(error as Error).message}`);
      }
    });
  }

  /**
   * Simulate graph loading process (as Alizarin would do it)
   */
  public simulateGraphLoading(): void {
    const registryPath = join(this.basePath, 'graphs.json');

    if (!existsSync(registryPath)) {
      this.reportError('graphLoadingTests', 'graphs.json', 'Registry file does not exist');
      return;
    }

    try {
      const graphsRegistry = JSON.parse(readFileSync(registryPath, 'utf8'));

      for (const [graphId, model] of Object.entries(graphsRegistry.models) as [string, any][]) {
        const modelFile = `arches-${model.slug}-model.json`;
        const modelPath = join(this.basePath, modelFile);

        // Check model file exists
        if (!existsSync(modelPath)) {
          this.reportError('graphLoadingTests', graphId, `Model file ${modelFile} not found`);
          continue;
        }

        // Check model file loads
        try {
          const modelContent = JSON.parse(readFileSync(modelPath, 'utf8'));

          // Check graph ID matches
          if (modelContent.graph[0].graphid !== graphId) {
            this.reportError('graphLoadingTests', graphId,
              `Graph ID mismatch: registry has ${graphId}, model has ${modelContent.graph[0].graphid}`);
            continue;
          }

          // Check business data files exist for this graph
          const businessDataFiles = readdirSync(this.basePath).filter(f =>
            f.startsWith('arches-business-data-') && f.endsWith('.json')
          );

          // Verify business data can be loaded (optional - don't fail if missing)
          for (const file of businessDataFiles) {
            try {
              const content = JSON.parse(readFileSync(join(this.basePath, file), 'utf8'));
              const resources = content.business_data?.resources || [];

              if (resources.some((r: any) => r.graph_id === graphId)) {
                break;
              }
            } catch (error) {
              // Skip invalid files
              this.reportError('graphLoadingTests', graphId, `Business data file parse error: ${(error as Error).message}`);
            }
          }

          this.reportSuccess('graphLoadingTests', graphId);

        } catch (error) {
          this.reportError('graphLoadingTests', graphId, `Model file parse error: ${(error as Error).message}`);
        }
      }

    } catch (error) {
      this.reportError('graphLoadingTests', 'graphs.json', `Registry parse error: ${(error as Error).message}`);
    }
  }

  /**
   * Run all validation checks
   */
  public validate(): ValidationSummary {
    this.validateGraphModels();
    this.validateBusinessDataFiles();
    this.validateGraphsRegistryFile();
    this.checkAlizarinCompatibility();
    this.simulateGraphLoading();

    const totalPassed = Object.values(this.results)
      .reduce((sum, result) => sum + result.passed, 0);
    const totalFailed = Object.values(this.results)
      .reduce((sum, result) => sum + result.failed, 0);

    return {
      results: this.results,
      totalPassed,
      totalFailed,
      success: totalFailed === 0
    };
  }

  /**
   * Get validation results
   */
  public getResults(): ValidationResults {
    return this.results;
  }

  /**
   * Print validation summary to console
   */
  public printSummary(summary: ValidationSummary): void {
    console.log('\n📊 VALIDATION SUMMARY');
    console.log('====================\n');

    const categories = [
      { name: 'Graph Models', key: 'graphModels' as const },
      { name: 'Business Data', key: 'businessData' as const },
      { name: 'Graphs Registry', key: 'graphsRegistry' as const },
      { name: 'Alizarin Compatibility', key: 'alizarinCompatibility' as const },
      { name: 'Graph Loading Tests', key: 'graphLoadingTests' as const }
    ];

    categories.forEach(category => {
      const result = summary.results[category.key];
      const total = result.passed + result.failed;
      const percentage = total > 0 ? Math.round((result.passed / total) * 100) : 0;

      console.log(`${category.name}: ${result.passed}/${total} passed (${percentage}%)`);

      if (result.failed > 0) {
        console.log(`   ❌ ${result.failed} failures:`);
        result.errors.slice(0, 3).forEach(error => {
          console.log(`      • ${error.file}: ${error.error}`);
        });
        if (result.errors.length > 3) {
          console.log(`      • ... and ${result.errors.length - 3} more errors`);
        }
      }
    });

    console.log('');
    console.log(`🎯 OVERALL RESULT: ${summary.totalPassed}/${summary.totalPassed + summary.totalFailed} checks passed`);

    if (summary.success) {
      console.log('🎉 ALL VALIDATION CHECKS PASSED!');
      console.log('✅ Graph structure is valid');
      console.log('✅ Business data has proper Alizarin metadata');
      console.log('✅ All files can be loaded by Alizarin');
      console.log('✅ No compatibility issues detected');
    } else {
      console.log(`⚠️  ${summary.totalFailed} validation issues found`);
      console.log('💡 Review the errors above and fix the issues');
      console.log('💡 Re-run this validator after making corrections');
    }
  }
}

/**
 * Convenience function to run validation from command line or scripts
 */
export function validateGraphLoading(basePath: string = '.'): ValidationSummary {
  const validator = new GraphLoadingValidator(basePath);
  const summary = validator.validate();
  validator.printSummary(summary);
  return summary;
}
