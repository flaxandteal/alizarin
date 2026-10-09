/**
 * CSV Model Loader
 *
 * Thin TypeScript wrapper over the WASM `buildGraphFromModelCsvs` and
 * `validateModelCsvs` functions. Parses the 3-CSV format (graph.csv,
 * nodes.csv, collections.csv) and builds an Arches resource model graph
 * with SKOS collections.
 *
 * The build functions route to native NAPI implementations when that backend
 * is active and exposes them; validateModelCsvs always uses WASM.
 *
 * @module csvModelLoader
 */

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore — generated WASM bindings
import { buildGraphFromModelCsvs as wasmBuild, validateModelCsvs as wasmValidate, buildResourcesFromBusinessCsv as wasmBuildBusinessData, composeResourceLayers as wasmComposeResourceLayers } from '../pkg/alizarin';
import { getBackend, getNapiModule, safeStringify } from './backend';

export interface CsvModelDiagnostic {
  level: 'Error' | 'Warning';
  file: string;
  line: number | null;
  message: string;
}

export interface CsvModelBuildResult {
  graph: any; // StaticGraph JSON
  collections: any[]; // SkosCollection JSON array
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
export function buildGraphFromModelCsvs(
  graphCsv: string,
  nodesCsv: string,
  rdmNamespace: string,
  collectionsCsv?: string,
): CsvModelBuildResult {
  if (getBackend() === 'napi') {
    const napi = getNapiModule();
    if (napi?.buildGraphFromCsvs) {
      return napi.buildGraphFromCsvs(graphCsv, nodesCsv, collectionsCsv ?? null, rdmNamespace);
    }
  }
  return wasmBuild(graphCsv, nodesCsv, collectionsCsv ?? null, rdmNamespace);
}

/**
 * Validate 3-CSV model files without building.
 *
 * @param graphCsv - Contents of graph.csv
 * @param nodesCsv - Contents of nodes.csv
 * @param collectionsCsv - Contents of collections.csv (optional)
 * @returns Array of diagnostics (errors and warnings)
 */
export function validateModelCsvs(
  graphCsv: string,
  nodesCsv: string,
  collectionsCsv?: string,
): CsvModelDiagnostic[] {
  return wasmValidate(graphCsv, nodesCsv, collectionsCsv ?? null);
}

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
 * @param uuidNamespace - Override UUID v5 namespace for tile IDs (for layer isolation)
 * @returns Business data wrapper with resources array
 */
export function buildResourcesFromBusinessCsv(
  csvData: string,
  graph: any,
  collections: any[],
  defaultLanguage?: string,
  strictConcepts?: boolean,
  uuidNamespace?: string,
): BusinessDataResult {
  if (getBackend() === 'napi') {
    const napi = getNapiModule();
    if (napi?.buildBusinessDataFromCsv) {
      return napi.buildBusinessDataFromCsv(
        csvData,
        safeStringify(graph),
        safeStringify(collections),
        defaultLanguage ?? 'en',
        strictConcepts ?? null,
        uuidNamespace ?? null,
      );
    }
  }
  return wasmBuildBusinessData(
    csvData,
    safeStringify(graph),
    safeStringify(collections),
    defaultLanguage ?? null,
    strictConcepts ?? null,
    uuidNamespace ?? null,
  );
}

/** The composed resource plus any merge/unify warnings. */
export interface ComposedResourceResult {
  resource: unknown;
  warnings: string[];
}

/**
 * Compose one resource across an ordered layer stack, entirely in memory (no
 * DuckDB) — the binding form of the substrate's `hydrate_layers`.
 *
 * @param resources - The same resource as it appears in each layer, TOPMOST-FIRST
 *   (highest-priority layer first). Tiles are merged (identical tiles deduped,
 *   topmost wins), then cardinality-1 nodegroups are unified PerNodegroup: the
 *   topmost layer overrides a single-valued group whole (no field-by-field
 *   inheritance), while multi-valued nodegroups accumulate across layers.
 * @param baseGraph - The base model `StaticGraph`.
 * @param overlayGraphs - Overlay `StaticGraph`s bottom-to-top (default `[]`);
 *   composition runs against the merged model via `LayeredGraph`.
 * @param strict - Make a cross-layer conflict on a single-valued group an error.
 * @returns `{ resource, warnings }`.
 */
export function composeResourceLayers(
  resources: unknown[],
  baseGraph: unknown,
  overlayGraphs: unknown[] = [],
  strict?: boolean,
): ComposedResourceResult {
  if (getBackend() === 'napi') {
    const napi = getNapiModule();
    if (napi?.composeResourceLayers) {
      return napi.composeResourceLayers(
        safeStringify(resources),
        safeStringify(baseGraph),
        safeStringify(overlayGraphs),
        strict ?? null,
      );
    }
  }
  return wasmComposeResourceLayers(
    safeStringify(resources),
    safeStringify(baseGraph),
    safeStringify(overlayGraphs),
    strict ?? null,
  ) as ComposedResourceResult;
}
