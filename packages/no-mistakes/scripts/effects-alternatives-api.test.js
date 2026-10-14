const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled effects alternatives share the CLI physical occurrence and category projections",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/effects-alternatives/fixture",
    );
    for (const projection of ["all", "read", "write"]) {
      const options = { root, kind: "database", entry: "src/entry.mts" };
      if (projection !== "all") options.categories = [projection];
      const expected = JSON.parse(readFileSync(join(root, `expected-${projection}.json`), "utf8"));
      const pending = cjs.effects(options);
      assert.equal(typeof pending.then, "function");
      assert.deepEqual(await pending, expected);
      assert.deepEqual(await esm.effects(options), expected);
      const { root: _root, ...report } = options;
      const batch = await cjs.analyzeProject({ root, reports: [{ type: "effects", ...report }] });
      assert.deepEqual(batch.reports[0].result, expected);
    }
  },
);
