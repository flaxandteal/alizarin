import { resolve } from "path";
import { defineConfig } from "vitest/config";

export default defineConfig({
  define: {
    __ALIZARIN_VERSION__: JSON.stringify("bench"),
  },
  resolve: {
    alias: [
      { find: "@", replacement: resolve(__dirname, "../js") },
      { find: "alizarin", replacement: resolve(__dirname, "../js/main.ts") },
    ],
  },
  test: {
    globals: true,
    setupFiles: [resolve(__dirname, "../tests/setupVitest.js")],
    benchmark: {
      include: ["benchmarks/**/*.bench.ts"],
      outputJson: "benchmarks/results-wasm.json",
    },
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
