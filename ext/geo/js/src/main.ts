// Alizarin geo extension — registers a handler for the
// `geojson-feature-collection` datatype: validation, and folding values into a
// geometry summary ({count, bbox, geometry_types}) for alizarin's fold and
// Aggregate functions.
//
// WASM backend: both run in the geo WASM module. Each fold call crosses into JS
// once per batch of values, so folding geometry here suits occasional, build-time
// use rather than a per-view hot path.
// NAPI backend: wired via the optional @alizarin/geo-napi native peer, if present.
import { registerExtensionHandler, wasmReady } from "alizarin";
import {
  initSync,
  validateGeojson,
  geoFoldIdentity,
  geoFold,
  geoFoldMerge,
  geoFoldRemove,
  geoFoldMayMatch,
} from "../../wasm/pkg/alizarin_geo_wasm";
import wasmBytes from "../../wasm/pkg/alizarin_geo_wasm_bg.wasm";

// Register the geojson handler for the WASM backend once the core WASM is ready.
wasmReady.then(() => {
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
});

// Also register for the NAPI backend, when it is in use and the native ext
// (@alizarin/geo-napi) is installed. Both are OPTIONAL peers: a WASM/browser
// consumer has neither, so the imports fail and this is skipped.
(async () => {
  try {
    const napiPkg = "@alizarin/napi";
    const extPkg = "@alizarin/geo-napi";
    const napi = await import(napiPkg) as { registerExtensionHandler(ptr: bigint): unknown };
    const ext = await import(extPkg) as { geoHandlerPtr(): bigint };
    napi.registerExtensionHandler(ext.geoHandlerPtr());
  } catch {
    // Not using the napi backend, or the native ext isn't installed — nothing to do.
  }
})();

export { validateGeojson };
