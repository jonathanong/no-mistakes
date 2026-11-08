const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled column CHECK rule shares async CJS, ESM and prepared reports",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-column-check-constraints/fixture/fail",
    );
    const pending = cjs.check({ root });
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(report.rules, []);
    assert.ok(
      report.advisories.some((finding) => finding.target === "column:metrics.missing_count"),
    );
    assert.ok(!report.advisories.some((finding) => finding.target === "column:metrics.ge_count"));
    assert.deepEqual(await esm.check({ root }), report);
    const batch = await cjs.analyzeProject({
      root,
      reports: [{ type: "check", id: "column-checks" }],
    });
    assert.deepEqual(batch.reports[0].result, report);
    const config = join(root, "../../configs/report.yml");
    const reported = await cjs.check({ root, config });
    assert.deepEqual(reported.advisories, []);
    assert.deepEqual(reported.rules, report.advisories);
  },
);
