// Alizarin geo extension — registers a validate-only handler for the
// `geojson-feature-collection` datatype.
//
// WASM backend: validation runs in the geo WASM module (validateGeojson).
// NAPI backend: wired via the optional @alizarin/geo-napi native peer, if present.
import { registerExtensionHandler, wasmReady } from "alizarin";
import { initSync, validateGeojson } from "../../wasm/pkg/alizarin_geo_wasm";
import wasmBytes from "../../wasm/pkg/alizarin_geo_wasm_bg.wasm";

// Register the geojson validator for the WASM backend once the core WASM is ready.
wasmReady.then(() => {
  initSync(wasmBytes);
  registerExtensionHandler("geojson-feature-collection", {
    validate: (value: unknown, _config: unknown) => validateGeojson(value),
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
