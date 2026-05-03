/**
 * Vitest setup file for NAPI backend tests that also need WASM for graph construction.
 *
 * WASM is initialized because createStaticGraph, GraphMutator, etc. use WASM
 * constructors. After WASM init, we switch the backend to NAPI so that
 * ResourceModelWrapper delegates to NapiResourceModelWrapper.
 *
 * Use setupVitestNapi.js (no WASM) to test the pure production NAPI path.
 */
import { createRequire } from 'module';
import { setCurrentLanguage } from "../js/utils.ts";
import { setNapiModule } from "../js/backend.ts";
import { initWasm } from "../js/_wasm.ts";

// Set default language to 'en' for consistent test behavior
setCurrentLanguage('en');

// Initialize WASM — needed for graph construction (createStaticGraph, GraphMutator)
await initWasm();

// Load the native NAPI module via require() (native addons need CJS loading)
const require_ = createRequire(import.meta.url);
const napi = require_('../crates/alizarin-napi');

// Register the NAPI module as the active backend (after WASM init)
// setNapiModule also sets backend to 'napi'
setNapiModule(napi);
