import { resolve } from "path";
import { defineConfig } from "vite";

export default defineConfig({
  build: {
    minify: false,
    lib: {
      entry: resolve(__dirname, "src/main.ts"),
      name: "Alizarin",
      fileName: "alizarin",
    },
  },
});
