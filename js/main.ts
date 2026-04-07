import * as client from "./client";
import * as interfaces from "./interfaces";
import { RDM, ResolveLabelsOptions, registerResolvableDatatype, unregisterResolvableDatatype } from "./rdm";
import { ResourceModelWrapper, WKRM, graphManager, staticStore, GraphManager, GraphMutator, getWasmTimings } from "./graphManager";
import * as staticTypes from "./static-types";
import { CollectionMutator } from "./collectionMutator";
import { buildGraphFromModelCsvs, validateModelCsvs, buildResourcesFromBusinessCsv } from "./csvModelLoader";
import type { CsvModelDiagnostic, CsvModelBuildResult, BusinessDataResult } from "./csvModelLoader";
import * as utils from "./utils";
import * as viewModels from "./viewModels";
import * as renderers from "./renderers";
import * as nodeConfig from "./nodeConfig";
import { initWasm, setWasmURL, ensureWasmRdmCache, parseSkosXml, parseSkosXmlToCollection, collectionToSkosXml, collectionsToSkosXml, registerExtensionHandler } from "./_wasm";
import { newWASMResourceInstanceWrapperForResource, WASMResourceModelWrapper } from "../pkg/alizarin";
import { resetTimingStats, getTimingStats, logTimingStats } from "./semantic";
import * as tracing from "./tracing";
import { IStringKeyedObject } from "./interfaces";

// Version injected at build time by Vite
declare const __ALIZARIN_VERSION__: string;
export const version: string = __ALIZARIN_VERSION__;

// Register alizarin JS timing getter for unified tracing
tracing.registerAlizarinTimingGetter(getTimingStats);
// Register detailed WASM timing getter
tracing.registerWasmTimingGetter(getWasmTimings);

// Initialize WASM module - deferred to allow consumers to call setWasmURL() first
// Consumers should await wasmReady before using WASM features
let _wasmReadyResolve: () => void;
const wasmReady = new Promise<void>(resolve => { _wasmReadyResolve = resolve; });

// Hook into initWasm so that wasmReady resolves on ANY successful call —
// not just the auto-init. This is critical for consumers like alizarin-loader
// that poison the auto-init URL and retry with a valid one: extensions
// (@alizarin/clm, @alizarin/filelist) do wasmReady.then(...) to register
// handlers, so wasmReady must resolve once WASM is actually available.
const _origInitWasm = initWasm;
const _wrappedInitWasm = async function() {
  await _origInitWasm();
  _wasmReadyResolve();
};
// Replace the export binding
Object.defineProperty(_wrappedInitWasm, 'name', { value: 'initWasm' });

// Auto-init after a microtask, giving consumers a chance to call setWasmURL() synchronously.
// Failure is silently swallowed — consumers that poison the URL will call initWasm() again.
Promise.resolve().then(() => _wrappedInitWasm().catch(() => {}));

const AlizarinModel = viewModels.ResourceInstanceViewModel;
const setCurrentLanguage = utils.setCurrentLanguage;
const getCurrentLanguage = utils.getCurrentLanguage;
const slugify = utils.slugify;
const getValueFromPath = utils.getValueFromPath;
const getValueFromPathSync = utils.getValueFromPathSync;
export type {
  IStringKeyedObject,
  ResolveLabelsOptions,
  CsvModelDiagnostic,
  CsvModelBuildResult,
  BusinessDataResult,
};
export {
  AlizarinModel,
  client,
  graphManager,
  GraphManager,
  staticTypes,
  utils,
  slugify,
  getValueFromPath,
  getValueFromPathSync,
  viewModels,
  staticStore,
  RDM,
  registerResolvableDatatype,
  unregisterResolvableDatatype,
  renderers,
  interfaces,
  WKRM,
  nodeConfig,
  ResourceModelWrapper,
  GraphMutator,
  setCurrentLanguage,
  getCurrentLanguage,
  _wrappedInitWasm as initWasm,
  setWasmURL,
  wasmReady,
  ensureWasmRdmCache,
  // Legacy timing (deprecated - use tracing instead)
  resetTimingStats,
  getTimingStats,
  logTimingStats,
  // New unified tracing
  tracing,
  // SKOS RDF/XML parsing and serialization
  parseSkosXml,
  parseSkosXmlToCollection,
  collectionToSkosXml,
  collectionsToSkosXml,
  // Collection mutator
  CollectionMutator,
  // CSV model loader
  buildGraphFromModelCsvs,
  validateModelCsvs,
  buildResourcesFromBusinessCsv,
  // Extension function
  registerExtensionHandler,
  // Low-level WASM wrappers for direct resource access
  newWASMResourceInstanceWrapperForResource,
  WASMResourceModelWrapper,
};
