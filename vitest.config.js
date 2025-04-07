import { resolve } from 'path'

export default {
  resolve: {
    alias: [{ find: "@", replacement: resolve(__dirname, "./src") }]
  },
  test: {
    globals: true,
    setupFiles: [
      "./tests/setupVitest.js"
    ]
  }
}
