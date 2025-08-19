import { resolve } from "path";
import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: {
    alias: [{ find: "@", replacement: resolve(__dirname, "./js") }],
  },
  test: {
    globals: true,
    setupFiles: ["./tests/setupVitest.js"],
    pool: "threads",
    poolOptions: {
      threads: {
        execArgv: ["--experimental-wasm-modules"],
      },
    },
    environment: "node",
    environmentOptions: {
      // Enable experimental WASM support
      node: {
        execArgv: ["--experimental-wasm-modules"],
      },
    },
  },
  server: {
    fs: {
      allow: [".."],
    },
  },
  assetsInclude: ["**/*.wasm"],
});
