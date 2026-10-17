const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test("compiled async APIs preserve hoisted custom type imports", { skip: !compiled }, async () => {
  const cjs = require("../index.js");
  const esm = await import("../index.mjs");
  const root = join(
    __dirname,
    "../../../test-cases/rules/query-reached-per-item/typed-import-order-fixture",
  );
  for (const [entry, line] of [
    ["early.mts", 5],
    ["late.mts", 4],
    ["mixed.mts", 4],
    ["late-shapes.mts", 4],
  ]) {
    const options = { root, kind: "postgres", entry };
    const pending = cjs.effects(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(
      report.callSites.map((site) => [site.file, site.line, site.callee]),
      [[entry, line, "NonNullable"]],
    );
    assert.deepEqual(await esm.effects(options), report);
    for (const api of [cjs, esm]) {
      const prepared = await api.analyzeProject({
        root,
        reports: [{ type: "effects", kind: "postgres", entry }],
      });
      assert.deepEqual(prepared.reports[0].result, report);
    }
  }
  const negative = await cjs.effects({ root, kind: "postgres", entry: "controls.mts" });
  assert.deepEqual(negative.callSites, []);
});
