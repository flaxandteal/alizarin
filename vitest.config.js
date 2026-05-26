import { resolve } from "path";
import { readFileSync } from "fs";
import { defineConfig } from "vitest/config";

function wasmInlinePlugin() {
  return {
    name: 'wasm-inline',
    enforce: 'pre',
    load(id) {
      if (id.endsWith('.wasm')) {
        const bytes = readFileSync(id);
        const base64 = bytes.toString('base64');
        return `const base64 = "${base64}";\nconst bytes = Uint8Array.from(atob(base64), c => c.charCodeAt(0));\nexport default bytes;`;
      }
    }
  };
}

export default defineConfig({
  plugins: [wasmInlinePlugin()],
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
