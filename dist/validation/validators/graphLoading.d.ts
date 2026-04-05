export interface ValidationResult {
    passed: number;
    failed: number;
    errors: Array<{
        file: string;
        error: string;
    }>;
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
export declare class GraphLoadingValidator {
    private ajv;
    private validateGraphModel;
    private validateBusinessData;
    private validateGraphsRegistry;
    private results;
    private basePath;
    constructor(basePath?: string);
    private reportError;
    private reportSuccess;
    private formatErrors;
    /**
     * Validate graph model files
     */
    validateGraphModels(): void;
    /**
     * Validate business data files
     */
    validateBusinessDataFiles(): void;
    /**
     * Validate graphs registry file
     */
    validateGraphsRegistryFile(): void;
    /**
     * Check Alizarin compatibility requirements
     */
    checkAlizarinCompatibility(): void;
    /**
     * Simulate graph loading process (as Alizarin would do it)
     */
    simulateGraphLoading(): void;
    /**
     * Run all validation checks
     */
    validate(): ValidationSummary;
    /**
     * Get validation results
     */
    getResults(): ValidationResults;
    /**
     * Print validation summary to console
     */
    printSummary(summary: ValidationSummary): void;
}
/**
 * Convenience function to run validation from command line or scripts
 */
export declare function validateGraphLoading(basePath?: string): ValidationSummary;
