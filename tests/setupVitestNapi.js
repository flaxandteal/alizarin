/**
 * Vitest setup file for NAPI backend tests.
 *
 * Does NOT initialize WASM — tests the pure NAPI path as it runs in production
 * (e.g. starches-builder with ALIZARIN_BACKEND=napi). All type construction
 * must go through backend-aware factories.
 */
import { createRequire } from 'module';
import { setCurrentLanguage } from "../js/utils.ts";
import { setNapiModule } from "../js/backend.ts";

// Set default language to 'en' for consistent test behavior
setCurrentLanguage('en');

// Load the native NAPI module via require() (native addons need CJS loading)
const require_ = createRequire(import.meta.url);
const napi = require_('../crates/alizarin-napi');

// Register the NAPI module as the active backend (WITHOUT WASM init)
// This tests the real production scenario where WASM is never loaded.
setNapiModule(napi);
