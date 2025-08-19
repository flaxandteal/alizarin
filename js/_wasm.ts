import init, { greet } from "../pkg/wasm";
import wasmURL from "../pkg/wasm_bg.wasm?url"

export async function run() {
  console.warn("Does init need called for Rust except in unbundled browser?");
  await init(wasmURL);
  greet();
}
