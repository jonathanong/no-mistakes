const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

test(
  "compiled CommonJS and ESM checks retain eager Annex B for-in helpers",
  { skip: !process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions/conditional-requires",
    );
    const options = { root };
    const report = await cjs.check(options);
    assert.deepEqual(await esm.check(options), report);
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(
      report.rules.map(({ file, target }) => [file, target]),
      [
        ["tests/conditionals.test.cjs", "src/target.cjs"],
        ["tests/setup-skipped.test.cjs", "src/setup-target.cjs"],
      ],
    );
    const prepared = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
    assert.deepEqual(prepared.reports[0].result, report);
    assert.deepEqual(
      await esm.analyzeProject({ ...options, reports: [{ type: "check" }] }),
      prepared,
    );
  },
);
