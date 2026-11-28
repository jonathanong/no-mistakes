const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled polymorphic target rule shares async CJS, ESM and prepared reports",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-polymorphic-target/fixture/fail",
    );
    const pending = cjs.check({ root });
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(report.warnings, []);
    assert.equal(report.rules.length, 6);
    assert.ok(
      report.rules.some(
        (finding) =>
          finding.target === "table:comments" &&
          finding.message.includes("target_type and target_id"),
      ),
    );
    assert.ok(!report.rules.some((finding) => finding.target === "table:attachments"));
    assert.deepEqual(await esm.check({ root }), report);
    const batch = await cjs.analyzeProject({
      root,
      reports: [{ type: "check", id: "polymorphic-target" }],
    });
    assert.deepEqual(batch.reports[0].result, report);
    const customConfig = join(root, "../../configs/custom-suffix.yml");
    const customReport = await cjs.check({ root, config: customConfig });
    assert.equal(customReport.rules.length, 1);
    assert.equal(customReport.rules[0].target, "table:custom_links");
  },
);
