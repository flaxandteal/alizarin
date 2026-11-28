/**
 * The prebuild export written to a directory is the in-memory export's files,
 * as in Python (crates/alizarin-python/tests/test_prebuild_export.py): both
 * are core's. NAPI writes them itself (exportGraphsToDir, exportPrebuild);
 * WASM has no filesystem, so a WASM caller writes the in-memory export, which
 * is what the WASM run does (and so checks little beyond the export's paths).
 *
 * Runs under both:
 *   npx vitest run
 *   npx vitest run --config vitest.napi.config.js
 */
import { beforeAll, describe, expect, test } from "vitest";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import {
  buildPrebuildExport,
  createStaticGraphMeta,
  createWKRM,
  getBackend,
  getNapiModule,
  parseStaticGraph,
} from "../../js/backend";
import { ResourceModelWrapper } from "../../js/graphManager";

const BASE_URI = "http://localhost/";
const graphText = fs.readFileSync(path.join(__dirname, "../data/output-parity/graph.json"), "utf8");
let graph: any;

beforeAll(() => {
  graph = parseStaticGraph(graphText);
  const wkrm = createWKRM(
    createStaticGraphMeta({
      graphid: graph.graphid,
      name: "Parity",
      slug: "parity",
      relatable_resource_model_ids: [],
      resource_2_resource_constraints: [],
      extra_fields: {},
    }),
  );
  // Registers the graph, as the output parity test does.
  new ResourceModelWrapper(wkrm, graph, undefined, true);
});

/** Every file under `dir`, by path relative to it. */
function written(dir: string): Record<string, string> {
  const files: Record<string, string> = {};
  for (const entry of fs.readdirSync(dir, { recursive: true }) as string[]) {
    const full = path.join(dir, entry);
    if (fs.statSync(full).isFile()) files[entry.split(path.sep).join("/")] = fs.readFileSync(full, "utf8");
  }
  return files;
}

/** What a WASM caller does: write the in-memory export's graph files. */
function writeExport(graphIds: string[], dir: string) {
  for (const file of buildPrebuildExport(BASE_URI, graphIds).graphFiles) {
    fs.mkdirSync(path.dirname(path.join(dir, file.relativePath)), { recursive: true });
    fs.writeFileSync(path.join(dir, file.relativePath), file.content);
  }
}

describe(`prebuild export [${getBackend()} backend]`, () => {
  test("graphs written to a directory are the in-memory export's files", () => {
    const export_ = buildPrebuildExport(BASE_URI, [graph.graphid]);
    const expected = Object.fromEntries(export_.graphFiles.map((f: any) => [f.relativePath, f.content]));
    expect(Object.keys(expected)).toEqual(["graphs/resource_models/Parity.json"]);

    const napi: any = getBackend() === "napi" ? getNapiModule() : null;
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "alizarin-prebuild-"));
    try {
      const byId = path.join(tmp, "by_id");
      (napi ? napi.exportGraphsToDir : writeExport)([graph.graphid], byId);
      expect(written(byId)).toEqual(expected);

      const pkg = path.join(tmp, "package");
      if (napi) napi.exportPrebuild(pkg, BASE_URI, [graph.graphid]);
      else writeExport([graph.graphid], pkg);
      const graphs = Object.entries(written(pkg)).filter(([k]) => k.startsWith("graphs/"));
      expect(Object.fromEntries(graphs)).toEqual(expected);
    } finally {
      fs.rmSync(tmp, { recursive: true, force: true });
    }
  });
});
