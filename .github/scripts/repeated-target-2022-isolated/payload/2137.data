const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { join } = require("node:path");

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(__dirname, "../../../fixtures/napi/analyze-project-symbol-graph-only");

test(
  "compiled CJS and ESM keep visible graph-only symbol targets in mixed check batches",
  { skip: !compiled, timeout: 20_000 },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");

    for (const file of ["exported.mts", "empty.mts"]) {
      const symbolsOptions = { root, files: [file], include: "both", mode: "list" };
      const standaloneSymbols = await cjs.symbols(symbolsOptions);
      const standaloneCheck = await cjs.check({ root });
      const request = {
        root,
        reports: [
          { type: "symbols", files: [file], include: "both", mode: "list" },
          { type: "check" },
        ],
      };

      for (const api of [cjs, esm]) {
        const pending = api.analyzeProject(request);
        assert.ok(pending instanceof Promise);
        const batch = await pending;
        assert.deepEqual(
          batch.reports.map((report) => report.result),
          [standaloneSymbols, standaloneCheck],
        );
        assert.equal(
          Boolean(batch.reports[0].result.files[0].exports?.length),
          file === "exported.mts",
        );
      }
    }
  },
);
