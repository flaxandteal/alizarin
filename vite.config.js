import wasm from "vite-plugin-wasm";
import { resolve } from "path";
import { defineConfig } from "vite";
import { readFileSync, writeFileSync, existsSync } from "fs";
import dts from 'vite-plugin-dts';
import topLevelAwait from "vite-plugin-top-level-await";
import pkg from './package.json' with { type: 'json' };

// Workaround for Vite 5 library mode inlining all assets as base64 (vitejs/vite#4454).
// vite-plugin-wasm generates `import url from "file.wasm?url"`, but Vite 5 lib mode
// resolves ?url imports as base64 data URIs. This plugin replaces the inlined WASM
// with a relative file path, and copies the actual WASM file to dist.
// Can be removed once upgraded to Vite 6+ (use ?no-inline instead).
function externalizeWasmPlugin() {
  return {
    name: 'externalize-wasm',
    enforce: 'post',
    generateBundle(_options, bundle) {
      for (const [fileName, chunk] of Object.entries(bundle)) {
        if (chunk.type === 'chunk' && chunk.code) {
          const before = chunk.code.length;
          chunk.code = chunk.code.replace(
            /["']data:application\/wasm;base64,[A-Za-z0-9+/=]+["']/g,
            '"alizarin_bg.wasm"'
          );
          if (chunk.code.length !== before) {
            const saved = ((before - chunk.code.length) / 1024 / 1024).toFixed(2);
            console.log(`[externalize-wasm] Replaced inlined WASM in ${fileName} (saved ${saved} MB)`);
          }
        }
      }
    },
    writeBundle(options) {
      const outDir = options.dir || 'dist';
      const wasmSrc = resolve(__dirname, 'pkg/alizarin_bg.wasm');
      const wasmDest = resolve(outDir, 'alizarin_bg.wasm');

      if (existsSync(wasmSrc)) {
        const wasmContent = readFileSync(wasmSrc);
        writeFileSync(wasmDest, wasmContent);
        console.log(`[externalize-wasm] Copied WASM to ${wasmDest} (${(wasmContent.length / 1024 / 1024).toFixed(2)} MB)`);
      }
    }
  };
}

export default defineConfig({
  define: {
    __ALIZARIN_VERSION__: JSON.stringify(pkg.version),
  },
  plugins: [
    // dts({ rollupTypes: true }), // Temporarily disabled due to type errors
    wasm(),
    topLevelAwait(),
    externalizeWasmPlugin(),
  ],
  resolve: {
    alias: {
      // Extensions (@alizarin/clm, @alizarin/filelist) import from 'alizarin'.
      // Alias it to the local source so the 'full' entry bundles everything
      // into a single module instance.
      'alizarin': resolve(__dirname, 'js/main.ts'),
    },
  },
  optimizeDeps: {
    exclude: ['./pkg', 'alizarin'],
  },
  worker: {
    format: 'es'
  },
  build: {
    minify: false,
    sourcemap: true,
    // Don't inline any assets - emit WASM as separate file
    assetsInlineLimit: 0,
    lib: {
      entry: {
        alizarin: resolve(__dirname, "js/main.ts"),
        'alizarin.full': resolve(__dirname, "js/full.ts"),
        'validation/index': resolve(__dirname, "js/validation/index.ts"),
      },
      name: "Alizarin",
      fileName: (format, entryName) => {
        if (entryName.includes('validation')) {
          return `${entryName}.js`;
        }
        if (entryName === 'alizarin.full') {
          return 'alizarin.full.js';
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
        '@alizarin/napi',
      ],
      output: {
        // Preserve WASM filename without hash for predictable imports
        assetFileNames: (assetInfo) => {
          if (assetInfo.name?.endsWith('.wasm')) {
            return 'alizarin_bg.wasm';
          }
          return 'assets/[name]-[hash][extname]';
        },
      },
    },
  },
});
