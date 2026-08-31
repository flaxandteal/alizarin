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
      name: 'AlizarinClm',
      fileName: 'clm',
      formats: ['es']
    },
    rollupOptions: {
      // Externalize alizarin (provided by the consumer) and the optional napi
      // peers (@alizarin/napi, @alizarin/clm-napi) — dynamically imported at
      // runtime only when the napi backend is in use, never bundled.
      external: ['alizarin', '@alizarin/napi', '@alizarin/clm-napi'],
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
