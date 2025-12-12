import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  resolve: {
    alias: {
      'alizarin': resolve(__dirname, '../../../../dist/alizarin.js')
    }
  },
  server: {
    fs: {
      // Allow serving files from the parent alizarin package
      allow: ['../../../..']
    }
  }
});
