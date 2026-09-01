import { defineConfig } from 'vite';
import { resolve } from 'path';
import { readFileSync } from 'fs';

/**
 * Inline .wasm files as Uint8Array bytes so the built bundle is self-contained
 * (no separate .wasm to ship in the npm package). Mirrors ext/geo/js + filelist.
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
