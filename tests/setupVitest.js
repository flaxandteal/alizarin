import createFetchMock from "vitest-fetch-mock";
import { vi } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { setCurrentLanguage } from "../js/utils.ts";

const fetchMocker = createFetchMock(vi);
fetchMocker.enableMocks();

// Set default language to 'en' for consistent test behavior regardless of system locale
setCurrentLanguage('en');

// Mock fetch for WASM files in tests
global.fetch = vi.fn((url) => {
  const originalFetch = fetchMocker.getMockImplementation() || fetch;
  
  // Handle WASM file requests
  if (typeof url === 'string' && url.endsWith('.wasm')) {
    try {
      // Try to load the WASM file from the pkg directory
      const wasmPath = url.includes('pkg/') 
        ? join(process.cwd(), url)
        : join(process.cwd(), 'pkg', 'wasm_bg.wasm');
        
      const wasmBuffer = readFileSync(wasmPath);
      
      return Promise.resolve({
        ok: true,
        status: 200,
        headers: new Headers({
          'content-type': 'application/wasm',
        }),
        arrayBuffer: () => Promise.resolve(wasmBuffer.buffer),
        blob: () => Promise.resolve(new Blob([wasmBuffer], { type: 'application/wasm' })),
        json: () => Promise.reject(new Error('Not JSON')),
        text: () => Promise.reject(new Error('Not text')),
      });
    } catch (error) {
      console.error('Failed to load WASM file:', error);
      return Promise.reject(error);
    }
  }
  
  // For non-WASM requests, use the regular fetch mock
  return originalFetch(url);
});
