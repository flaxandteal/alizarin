import { resolve } from "path";
import { defineConfig } from "vitest/config";

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
    setupFiles: ["./tests/setupVitest.js"],
    bail: 1, // Exit immediately on first test failure
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
