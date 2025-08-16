import { resolve } from "path";
import { defineConfig } from "vite";
import dts from 'vite-plugin-dts';

export default defineConfig({
  plugins: [dts({ rollupTypes: true })],
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
