import { resolve } from "path";
import { configDefaults, defineConfig } from "vitest/config";

// Every TS test runs on NAPI too, unless listed here. Shrink the "to port" and
// "NAPI gap" lists as they're fixed; only "WASM-only by design" should remain.
const NAPI_EXCLUDED = [
  // WASM-only by design: they test the WASM bindings themselves.
  "tests/graph-wasm.test.ts",
  "tests/wasmErrors.test.ts",
  "tests/wasmResourceModelWrapper.test.ts",
  // To port: they build WASM objects (pkg/ types) directly, not via backend.ts.
  "tests/buildResourceDescriptors.test.ts",
  "tests/graphConfig.test.ts",
  "tests/graphs.test.ts",
  "tests/pseudos.test.ts",
  "tests/resources.test.ts",
  "tests/static-types.test.ts",
  "tests/type-coercion.test.ts",
  // NAPI gaps: no PseudoNode (createPseudoNode); the TS GraphMutator needs the
  // WASM StaticGraph's copy/push* API; three doc examples time out.
  "tests/graphManager.test.ts",
  "tests/docExamples.test.ts",
];

export default defineConfig({
  define: {
    __ALIZARIN_VERSION__: JSON.stringify("test"),
  },
  resolve: {
    alias: [
      { find: "@", replacement: resolve(__dirname, "./js") },
      { find: "alizarin", replacement: resolve(__dirname, "./js/main.ts") },
    ],
  },
  test: {
    globals: true,
    setupFiles: ["./tests/setupVitestNapi.js"],
    testTimeout: 30000,
    include: ["tests/**/*.test.ts"],
    exclude: [...configDefaults.exclude, ...NAPI_EXCLUDED],
    pool: "threads",
    poolOptions: {
      threads: {
        execArgv: ["--experimental-wasm-modules"],
      },
    },
    environment: "node",
  },
  server: {
    fs: {
      allow: [".."],
    },
  },
  assetsInclude: ["**/*.wasm"],
});
