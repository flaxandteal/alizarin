import { defineConfig } from 'vite';
import { resolve } from 'path';
import { readFileSync } from 'fs';

/**
 * Inline .wasm files as Uint8Array bytes so the built bundle is self-contained.
 * This avoids needing to ship the .wasm binary separately in the npm package.
 */
function wasmInlinePlugin() {
  return {
    name: 'wasm-inline',
    enforce: 'pre' as const,
    load(id: string) {
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
  resolve: {
    alias: {
      'alizarin': resolve(__dirname, '../../../dist/alizarin.js')
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
      allow: ['../../..']
    }
  }
});
