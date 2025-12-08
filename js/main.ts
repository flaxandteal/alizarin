import * as client from "./client";
import * as interfaces from "./interfaces";
import { RDM } from "./rdm";
import { ResourceModelWrapper, WKRM, graphManager, staticStore, GraphManager, GraphMutator } from "./graphManager";
import * as staticTypes from "./static-types";
import * as utils from "./utils";
import * as viewModels from "./viewModels";
import * as renderers from "./renderers";
import * as nodeConfig from "./nodeConfig";
import { run, initWasm } from "./_wasm";
import { resetTimingStats, getTimingStats, logTimingStats } from "./semantic";
import * as tracing from "./tracing";

// Version injected at build time by Vite
declare const __ALIZARIN_VERSION__: string;
export const version: string = __ALIZARIN_VERSION__;

// Register alizarin JS timing getter for unified tracing
tracing.registerAlizarinTimingGetter(getTimingStats);

// Initialize WASM module at startup
// Note: This returns a promise that must be awaited before using WASM features
const wasmReady = initWasm();

const AlizarinModel = viewModels.ResourceInstanceViewModel;
const setCurrentLanguage = utils.setCurrentLanguage;
const getCurrentLanguage = utils.getCurrentLanguage;
const slugify = utils.slugify;
export {
  AlizarinModel,
  client,
  graphManager,
  GraphManager,
  staticTypes,
  utils,
  slugify,
  viewModels,
  staticStore,
  RDM,
  renderers,
  interfaces,
  WKRM,
  nodeConfig,
  ResourceModelWrapper,
  GraphMutator,
  setCurrentLanguage,
  getCurrentLanguage,
  run,
  initWasm,
  wasmReady,
  // Legacy timing (deprecated - use tracing instead)
  resetTimingStats,
  getTimingStats,
  logTimingStats,
  // New unified tracing
  tracing,
};
