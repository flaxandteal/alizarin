import { resolve } from "path";

export default {
  resolve: {
    alias: [{ find: "@", replacement: resolve(__dirname, "./js") }],
  },
  test: {
    globals: true,
    setupFiles: ["./tests/setupVitest.js"],
  },
};
