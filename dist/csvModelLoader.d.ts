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
export interface BusinessDataResult {
    business_data: {
        resources: any[];
    };
}
/**
 * Build resource instances from a business data CSV.
 *
 * Columns are node aliases (not UUIDs). Concept values are labels
 * resolved against the collections. ResourceIDs generate deterministic UUIDs.
 *
 * @param csvData - Business data CSV with ResourceID as first column
 * @param graph - Built graph JSON (from buildGraphFromModelCsvs)
 * @param collections - Built collections array (from buildGraphFromModelCsvs)
 * @param defaultLanguage - Default language code (default "en")
 * @param strictConcepts - Error on unresolved concept labels (default true)
 * @returns Business data wrapper with resources array
 */
export declare function buildResourcesFromBusinessCsv(csvData: string, graph: any, collections: any[], defaultLanguage?: string, strictConcepts?: boolean): BusinessDataResult;
