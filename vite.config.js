import wasm from "vite-plugin-wasm";
import { resolve } from "path";
import { defineConfig } from "vite";
import dts from 'vite-plugin-dts';
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [
    // dts({ rollupTypes: true }), // Temporarily disabled due to type errors
    wasm(),
    topLevelAwait(),
  ],
  optimizeDeps: {
    exclude: ['./pkg', 'alizarin'],
  },
  worker: {
    format: 'es'
  },
  build: {
    minify: false,
    sourcemap: true,
    lib: {
      entry: {
        alizarin: resolve(__dirname, "js/main.ts"),
        'validation/index': resolve(__dirname, "js/validation/index.ts"),
      },
      name: "Alizarin",
      fileName: (format, entryName) => {
        if (entryName.includes('validation')) {
          return `${entryName}.js`;
        }
        return format === 'es' ? 'alizarin.js' : 'alizarin.umd.cjs';
      },
      formats: ['es'],
    },
    rollupOptions: {
      external: [
        'fs',
        'path',
        'url',
        'ajv',
        'ajv-formats',
      ],
    },
  },
});
