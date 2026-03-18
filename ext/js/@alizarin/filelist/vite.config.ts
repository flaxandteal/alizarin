import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  resolve: {
    alias: {
      'alizarin': resolve(__dirname, '../../../../dist/alizarin.js')
    }
  },
  build: {
    lib: {
      entry: resolve(__dirname, 'src/main.ts'),
      name: 'AlizarinFilelist',
      fileName: 'filelist',
      formats: ['es']
    },
    rollupOptions: {
      // Externalize alizarin - it will be provided by the consumer
      external: ['alizarin'],
      output: {
        globals: {
          alizarin: 'Alizarin'
        }
      }
    },
    copyPublicDir: false
  },
  server: {
    fs: {
      // Allow serving files from the parent alizarin package
      allow: ['../../../..']
    }
  }
});
