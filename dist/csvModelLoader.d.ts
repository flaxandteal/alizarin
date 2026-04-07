/**
 * CSV Model Loader
 *
 * Thin TypeScript wrapper over the WASM `buildGraphFromModelCsvs` and
 * `validateModelCsvs` functions. Parses the 3-CSV format (graph.csv,
 * nodes.csv, collections.csv) and builds an Arches resource model graph
 * with SKOS collections.
 *
 * @module csvModelLoader
 */
export interface CsvModelDiagnostic {
    level: 'Error' | 'Warning';
    file: string;
    line: number | null;
    message: string;
}
export interface CsvModelBuildResult {
    graph: any;
    collections: any[];
}
/**
 * Build a graph and collections from the 3-CSV model format.
 *
 * @param graphCsv - Contents of graph.csv
 * @param nodesCsv - Contents of nodes.csv
 * @param rdmNamespace - RDM namespace string (UUID or URL) for deterministic ID generation
 * @param collectionsCsv - Contents of collections.csv (optional)
 * @returns The built graph and collections
 * @throws Error with diagnostics if validation or build fails
 */
export declare function buildGraphFromModelCsvs(graphCsv: string, nodesCsv: string, rdmNamespace: string, collectionsCsv?: string): CsvModelBuildResult;
/**
 * Validate 3-CSV model files without building.
 *
 * @param graphCsv - Contents of graph.csv
 * @param nodesCsv - Contents of nodes.csv
 * @param collectionsCsv - Contents of collections.csv (optional)
 * @returns Array of diagnostics (errors and warnings)
 */
export declare function validateModelCsvs(graphCsv: string, nodesCsv: string, collectionsCsv?: string): CsvModelDiagnostic[];
