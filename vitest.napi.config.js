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
    setupFiles: ["./tests/setupVitestNapi.js"],
    testTimeout: 30000,
    include: ["tests/dual-backend/**/*.test.ts"],
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
