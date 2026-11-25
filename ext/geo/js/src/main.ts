// Alizarin geo extension — registers a handler for the
// `geojson-feature-collection` datatype: validation, and folding values into a
// geometry summary ({count, bbox, geometry_types}) for alizarin's fold and
// Aggregate functions.
//
// WASM backend: both run in the geo WASM module. Each fold call crosses into JS
// once per batch of values, so folding geometry here suits occasional, build-time
// use rather than a per-view hot path.
// NAPI backend: wired via the optional @alizarin/geo-napi native peer, if present.
import { registerExtensionHandler, registerDeriveProvider, wasmReady, getBackend } from "alizarin";
import {
  initSync,
  utmDerive,
  validateGeojson,
  geoFoldIdentity,
  geoFold,
  geoFoldMerge,
  geoFoldRemove,
  geoFoldMayMatch,
} from "../../wasm/pkg/alizarin_geo_wasm";
import wasmBytes from "../../wasm/pkg/alizarin_geo_wasm_bg.wasm";

// Register the geojson handler for the WASM backend once the core WASM is ready.
// On the NAPI backend the native ext registers both (below): a JS derive
// provider is refused there.
wasmReady.then(() => {
  if (getBackend() === "napi") {
    return;
  }
  initSync(wasmBytes);
  registerExtensionHandler("geojson-feature-collection", {
    validate: (value: unknown, _config: unknown) => validateGeojson(value),
    foldIdentity: (_config: unknown) => geoFoldIdentity(),
    fold: (acc: unknown, values: unknown[], _config: unknown) => geoFold(acc, values),
    foldMerge: (a: unknown, b: unknown, _config: unknown) => geoFoldMerge(a, b),
    foldRemove: (acc: unknown, values: unknown[], _config: unknown) => geoFoldRemove(acc, values),
    foldMayMatch: (summary: unknown, predicate: unknown, _config: unknown) =>
      geoFoldMayMatch(summary, predicate),
  });
  // The UTM derive function, for graphs whose declarations name provider "utm".
  registerDeriveProvider("utm", {
    derive: (resourceId: string, tiles: unknown, config: unknown, lookup: (kind: number, key: string) => unknown) =>
      utmDerive(resourceId, tiles, config, lookup as any),
  });
});

// Also register for the NAPI backend, when it is in use and the native ext
// (@alizarin/geo-napi) is installed. Both are OPTIONAL peers: a WASM/browser
// consumer has neither, so the import fails and this is skipped. Importing the
// ext registers its handler and UTM provider with @alizarin/napi (its
// register.js).
(async () => {
  try {
    const extPkg = "@alizarin/geo-napi";
    await import(extPkg);
  } catch {
    // Not using the napi backend, or the native ext isn't installed — nothing to do.
  }
})();

export { validateGeojson };
