const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const rule = "postgres-materialized-view-shape";
const saved = join(__dirname, "../../../test-cases/rules", rule);
const options = (name, fixture = "fail") => ({
  root: join(saved, "fixture", fixture),
  config: join(saved, "configs", name),
});

test(
  "compiled CJS and ESM check materialized view shape with shared application severity",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const pending = cjs.check(options("advisory.yml"));
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.check(options("advisory.yml")), report);
    for (const api of [cjs, esm]) {
      const batch = await api.analyzeProject({
        ...options("advisory.yml"),
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
    const blocking = await cjs.check(options("default.yml"));
    assert.deepEqual(blocking.advisories, []);
    assert.ok(blocking.rules.some((finding) => finding.rule === rule));
    const mixed = await cjs.check(options("mixed.yml", "mixed"));
    assert.deepEqual(mixed.advisories, []);
    for (const name of ["bad_advised", "bad_reported"]) {
      assert.ok(mixed.rules.some((finding) => finding.target === `materialized-view:${name}`));
    }
    assert.deepEqual(await esm.check(options("mixed.yml", "mixed")), mixed);
    assert.deepEqual(await cjs.check(options("off.yml")), await cjs.check(options("empty.yml")));
  },
);
