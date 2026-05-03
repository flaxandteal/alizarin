#!/usr/bin/env node
/**
 * Run WASM and NAPI benchmarks, then print a comparison table.
 *
 * Usage: node benchmarks/compare.mjs
 *
 * Outputs a markdown table suitable for CI comments / terminal.
 */
import { execSync } from "child_process";
import { readFileSync, existsSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");

function run(label, configPath) {
  console.error(`Running ${label} benchmarks...`);
  try {
    execSync(`npx vitest bench --run --config ${configPath}`, {
      cwd: root,
      stdio: ["ignore", "pipe", "pipe"],
      env: {
        ...process.env,
        NODE_OPTIONS: "--experimental-wasm-modules",
      },
      timeout: 120_000,
    });
  } catch (e) {
    // vitest bench exits 0 on success but may print to stderr
    if (e.status && e.status !== 0) {
      console.error(`${label} bench failed (exit ${e.status})`);
      if (e.stderr) console.error(e.stderr.toString());
      process.exit(1);
    }
  }
}

// Run both
run("WASM", "benchmarks/vitest.bench.wasm.config.js");
run("NAPI", "benchmarks/vitest.bench.napi.config.js");

// Load results
const wasmPath = resolve(root, "benchmarks/results-wasm.json");
const napiPath = resolve(root, "benchmarks/results-napi.json");

if (!existsSync(wasmPath) || !existsSync(napiPath)) {
  console.error("Missing result files. Ensure benchmarks ran successfully.");
  process.exit(1);
}

const wasmResults = JSON.parse(readFileSync(wasmPath, "utf-8"));
const napiResults = JSON.parse(readFileSync(napiPath, "utf-8"));

// Extract benchmark entries from vitest bench JSON output
function extractBenches(results) {
  const benches = new Map();
  for (const file of results.files || []) {
    for (const group of file.groups || []) {
      for (const bench of group.benchmarks || []) {
        // Strip backend label from name for matching
        const name = bench.name.replace(/\s*\[(wasm|napi)\]\s*/i, "");
        benches.set(name, {
          hz: bench.hz,
          mean: bench.mean,
          p99: bench.p99,
        });
      }
    }
  }
  return benches;
}

const wasmBenches = extractBenches(wasmResults);
const napiBenches = extractBenches(napiResults);

// Print comparison table
console.log("\n## Benchmark Comparison: WASM vs NAPI\n");
console.log(
  "| Benchmark | WASM (ops/s) | NAPI (ops/s) | Speedup | Winner |"
);
console.log(
  "|-----------|-------------|-------------|---------|--------|"
);

const allNames = new Set([...wasmBenches.keys(), ...napiBenches.keys()]);

for (const name of allNames) {
  const wasm = wasmBenches.get(name);
  const napi = napiBenches.get(name);

  if (!wasm || !napi) {
    const hzW = wasm ? Math.round(wasm.hz).toLocaleString() : "—";
    const hzN = napi ? Math.round(napi.hz).toLocaleString() : "—";
    console.log(`| ${name} | ${hzW} | ${hzN} | — | — |`);
    continue;
  }

  const hzW = Math.round(wasm.hz).toLocaleString();
  const hzN = Math.round(napi.hz).toLocaleString();
  const ratio = napi.hz / wasm.hz;
  const speedup =
    ratio >= 1
      ? `${ratio.toFixed(2)}x faster`
      : `${(1 / ratio).toFixed(2)}x slower`;
  const winner = ratio >= 1.05 ? "NAPI" : ratio <= 0.95 ? "WASM" : "~tie";

  console.log(`| ${name} | ${hzW} | ${hzN} | ${speedup} | ${winner} |`);
}

console.log("");
