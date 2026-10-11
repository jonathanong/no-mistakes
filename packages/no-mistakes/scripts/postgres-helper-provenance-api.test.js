const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled helper SQL diagnostics retain literal locations through assignment and append",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-explicit-columns/fixture/helper-provenance",
    );
    const report = await cjs.check({ root });
    assert.deepEqual(await esm.check({ root }), report);
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(
      report.rules.map(({ rule, file, line }) => [rule, file, line]),
      [
        ["postgres-explicit-columns", "src/composed.mts", 7],
        ["postgres-explicit-columns", "src/composed.mts", 16],
        ["postgres-explicit-columns", "src/helper.mts", 12],
        ["postgres-explicit-columns", "src/imported-other.mts", 4],
        ["postgres-explicit-columns", "src/imported-source.mts", 6],
        ["postgres-explicit-columns", "src/imported-source.mts", 11],
        ["postgres-explicit-columns", "src/imported-source.mts", 20],
        ["postgres-explicit-columns", "src/single.mts", 7],
      ],
    );
    const prepared = await cjs.analyzeProject({ root, reports: [{ type: "check" }] });
    assert.deepEqual(prepared.reports[0].result, report);
  },
);
