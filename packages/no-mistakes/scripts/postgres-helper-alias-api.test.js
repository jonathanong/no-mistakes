const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(
  __dirname,
  "../../../test-cases/rules/postgres-no-offset/fixture/helper-alias",
);

test(
  "compiled CJS, ESM, and prepared checks retain helper SQL aliases",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const report = await cjs.check({ root });
    assert.deepEqual(await esm.check({ root }), report);
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(
      report.rules.map(({ rule, file, line }) => [rule, file, line]),
      [
        ["postgres-no-offset", "src/query.mts", 6],
        ["postgres-no-offset", "src/query.mts", 13],
      ],
    );
    const prepared = await cjs.analyzeProject({ root, reports: [{ type: "check" }] });
    assert.deepEqual(prepared.reports[0].result, report);
  },
);
