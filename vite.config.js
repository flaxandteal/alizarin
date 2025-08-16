import wasm from "vite-plugin-wasm";
import { resolve } from "path";
import { defineConfig } from "vite";
import dts from 'vite-plugin-dts';
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [
    dts({ rollupTypes: true }),
    wasm(),
    topLevelAwait()
  ],
  build: {
    minify: false,
    sourcemap: true,
    lib: {
      entry: resolve(__dirname, "js/main.ts"),
      name: "Alizarin",
      fileName: "alizarin",
    },
  },
});
