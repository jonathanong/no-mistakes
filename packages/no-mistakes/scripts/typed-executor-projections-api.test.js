const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(
  __dirname,
  "../../../test-cases/rules/query-reached-per-item/typed-executor-projections-fixture",
);
for (const [entry, lines] of [
  ["entry.mts", [4, 7, 15, 20]],
  ["controls.mts", []],
  ["imported-utility.mts", []],
  ["unselected-indexed.mts", []],
  ["nullish-branches.mts", [7, 11, 15, 19]],
]) {
  test(`compiled async typed executor projections: ${entry}`, { skip: !compiled }, async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = { root, kind: "postgres", entry };
    const pending = cjs.effects(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.effects(options), report);
    assert.deepEqual(report.callSites.map((site) => site.line), lines);
    if (entry === "nullish-branches.mts") {
      assert.deepEqual(
        report.callSites.map((site) => site.callee),
        ["TransactionQuery", "write", "write", "TransactionQuery"],
      );
    }
  });
}
