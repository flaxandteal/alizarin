/**
 * Source parity: structural rules that keep one definition of alizarin's
 * output, checked over the sources so a new route can't drift unnoticed.
 * tests/dual-backend/outputParity.test.ts checks what the fixture's routes
 * return; this checks every route that exists.
 *
 * 1. WASM hands Rust values to JS only through `js_json::to_value` (via
 *    serde_json::Value: plain objects, sorted keys, `null` for None).
 * 2. NAPI likewise only through `json_arg::to_js` or a `serde_json::Value`
 *    return (never `env.to_js_value` on a struct, which writes field order).
 * 3. No binding builds a `toJSON` by hand: it is core's serialisation.
 * 4. No WASM getter returns `Option<…>`, which wasm-bindgen makes `undefined`
 *    where NAPI gives `null`.
 * 5. A `HashMap` field of a serialisable core struct is written in sorted key
 *    order (`crate::sorted_serde`), or skipped, so direct serialisation is
 *    byte-stable (the rule in sorted_serde.rs and notes/backend-support-matrix.md).
 * 6. An exported binding function that lists a map's keys sorts them through
 *    `sorted_serde::sorted_keys`, whatever the map: a hashed map lists them
 *    in an order that differs between processes, and the rule can't see
 *    types. String-level, over exported functions only (a `#[pymethods]`
 *    method with no attribute of its own isn't seen): a ratchet, not a proof.
 */
import { describe, expect, test } from "vitest";
import * as fs from "fs";
import * as path from "path";

const ROOT = path.join(__dirname, "..");

function rustFiles(dir: string): string[] {
  const abs = path.join(ROOT, dir);
  if (!fs.existsSync(abs)) return [];
  return fs.readdirSync(abs, { recursive: true, encoding: "utf8" })
    .filter((f) => f.endsWith(".rs"))
    .map((f) => path.join(dir, f));
}

function extDirs(kind: "wasm" | "napi"): string[] {
  return fs.readdirSync(path.join(ROOT, "ext"))
    .map((e) => `ext/${e}/${kind}/src`)
    .filter((d) => fs.existsSync(path.join(ROOT, d)));
}

const WASM = ["crates/alizarin-wasm/src", "crates/wasm-wrapper-derive/src", ...extDirs("wasm")].flatMap(rustFiles);
const NAPI = ["crates/alizarin-napi/src", ...extDirs("napi")].flatMap(rustFiles);
const CORE = rustFiles("crates/alizarin-core/src");
const PYTHON = ["crates/alizarin-python/src", ...fs.readdirSync(path.join(ROOT, "ext"))
  .map((e) => `ext/${e}/python/src`)
  .filter((d) => fs.existsSync(path.join(ROOT, d)))].flatMap(rustFiles);

function lines(file: string): string[] {
  return fs.readFileSync(path.join(ROOT, file), "utf8").split("\n");
}

/** `file:line: text` for each line matching `pattern`, outside comments. */
function grep(files: string[], pattern: RegExp): string[] {
  return files.flatMap((file) =>
    lines(file).flatMap((text, i) =>
      pattern.test(text) && !text.trim().startsWith("//") ? [`${file}:${i + 1}: ${text.trim()}`] : [],
    ),
  );
}

/** The body of each function carrying `attr` (from its `fn` to the closing brace at its indent). */
function functionsWith(files: string[], attr: RegExp): { where: string; body: string }[] {
  const found: { where: string; body: string }[] = [];
  for (const file of files) {
    const src = lines(file);
    src.forEach((text, i) => {
      if (!attr.test(text)) return;
      let start = i + 1;
      // The attribute's item: a function, or something else (a struct, an
      // impl, a module), which isn't one.
      while (start < src.length && !/\b(fn|struct|enum|impl|mod)\s/.test(src[start])) start++;
      if (start >= src.length || !/\bfn\s/.test(src[start])) return;
      const indent = src[start].match(/^\s*/)![0];
      let end = start + 1;
      while (end < src.length && src[end] !== `${indent}}`) end++;
      found.push({ where: `${file}:${start + 1}`, body: src.slice(start, end + 1).join("\n") });
    });
  }
  return found;
}

describe("source parity", () => {
  test("1. WASM output goes through js_json::to_value", () => {
    expect(grep(WASM, /serde_wasm_bindgen::(to_value|Serializer)|json_compatible\(/)).toEqual([]);
  });

  test("2. NAPI output goes through json_arg::to_js", () => {
    const direct = grep(NAPI.filter((f) => !f.endsWith("json_arg.rs")), /\.to_js_value\(/);
    expect(direct).toEqual([]);
  });

  test("3. no binding builds a toJSON by hand", () => {
    const handBuilt = functionsWith([...WASM, ...NAPI], /^\s*#\[(wasm_bindgen|napi)\(.*js_name\s*=\s*"?toJSON/)
      .filter(({ body }) => /\.insert\(|Reflect::set|json!\(/.test(body))
      .map(({ where }) => where);
    expect(handBuilt).toEqual([]);
  });

  test("4. no exported WASM function returns Option (undefined where NAPI gives null)", () => {
    const optional: string[] = [];
    for (const file of WASM) {
      const src = lines(file);
      let exportedImpl = false;
      let attributes = "";
      src.forEach((text, i) => {
        if (/^\s*#\[/.test(text)) {
          attributes += text;
          return;
        }
        const exportedAttr = /#\[wasm_bindgen/.test(attributes) && !/wasm_bindgen\(skip/.test(attributes);
        attributes = "";
        if (/^impl\b/.test(text)) exportedImpl = exportedAttr;
        else if (text === "}") exportedImpl = false;
        const fn = text.match(/^(\s*)pub fn \w+/);
        if (!fn || !(exportedAttr || (exportedImpl && fn[1] === "    "))) return;
        let signature = text;
        for (let j = i + 1; !/\{\s*$|;\s*$/.test(signature) && j < src.length; j++) signature += src[j];
        if (/\)\s*->\s*(Result<\s*)?Option</.test(signature)) optional.push(`${file}:${i + 1}: ${text.trim()}`);
      });
    }
    expect(optional).toEqual([]);
  });

  test("4b. no WASM code hands JS an explicit undefined", () => {
    expect(grep(WASM, /JsValue::UNDEFINED/)).toEqual([]);
  });

  test("5. serialisable HashMap fields in core are written in sorted order", () => {
    const unsorted: string[] = [];
    for (const file of CORE) {
      const src = lines(file);
      let serialisable = false;
      let inStruct = false;
      src.forEach((text, i) => {
        if (/^#\[derive\(.*\bSerialize\b/.test(text)) serialisable = true;
        else if (/^pub struct \w+.*\{\s*$/.test(text)) inStruct = serialisable;
        else if (/^(pub )?(struct|enum|impl|fn|mod)\b/.test(text)) serialisable = false;
        if (text === "}") inStruct = serialisable = false;
        if (!inStruct || !/^\s+pub \w+: (Option<)?HashMap</.test(text)) return;
        // The field's attributes: the `#[...]` lines (possibly multi-line) above it.
        let j = i - 1;
        let attrs = "";
        while (j >= 0 && !/^\s+pub \w+:|\{\s*$/.test(src[j])) attrs = src[j--] + attrs;
        if (!/sorted_|skip/.test(attrs)) unsorted.push(`${file}:${i + 1}: ${text.trim()}`);
      });
    }
    expect(unsorted).toEqual([]);
  });

  test("6. exported binding functions list a map's keys through sorted_keys", () => {
    const exported = functionsWith(
      [...WASM, ...NAPI, ...PYTHON],
      /^\s*#\[(wasm_bindgen|napi|getter|pyfunction|pyo3)\b/,
    );
    const unsorted = exported.flatMap(({ where, body }) =>
      body
        .split("\n")
        .filter((text) => /\.keys\(\)/.test(text) && !/sorted_keys\(/.test(text))
        .filter((text) => !text.trim().startsWith("//"))
        .map((text) => `${where}: ${text.trim()}`),
    );
    expect(unsorted).toEqual([]);
  });
});
