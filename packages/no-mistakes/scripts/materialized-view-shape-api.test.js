const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const rule = "postgres-materialized-view-shape";
const saved = join(__dirname, "../../../test-cases/rules", rule);
const options = (name) => ({
  root: join(saved, "fixture/fail"),
  config: join(saved, "configs", name),
});

test(
  "compiled CJS and ESM check materialized view shape asynchronously",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const pending = cjs.check(options("default.yml"));
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.check(options("default.yml")), report);
    for (const api of [cjs, esm]) {
      const batch = await api.analyzeProject({
        ...options("default.yml"),
        reports: [{ type: "check" }],
      });
      assert.deepEqual(batch.reports[0].result, report);
    }
    assert.deepEqual(report.rules, []);
    assert.ok(
      report.advisories.some((finding) => finding.target === "materialized-view:bad_unbounded"),
    );
    assert.ok(
      !report.advisories.some((finding) => finding.target === "materialized-view:good_range"),
    );
    const strict = await cjs.check(options("report.yml"));
    assert.deepEqual(strict.advisories, []);
    assert.ok(strict.rules.some((finding) => finding.rule === rule));
    assert.deepEqual(await cjs.check(options("off.yml")), await cjs.check(options("empty.yml")));
  },
);
