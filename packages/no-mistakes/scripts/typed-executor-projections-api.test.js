const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(
  __dirname,
  "../../../test-cases/rules/query-reached-per-item/typed-executor-projections-fixture",
);
const cases = [
  ["entry.mts", [4, 7, 15, 20]],
  ["controls.mts", []],
  ["imported-utility.mts", []],
  ["unselected-indexed.mts", []],
  ["class-utility.mts", []],
  ["unsupported-shapes.mts", []],
  ["forward-utility.mts", []],
  ["forward-import-utility.mts", []],
  ["forward-type-shadow.mts", []],
  ["contextual-controls.mts", []],
  ["contextual-agreement.mts", [17]],
  ["nullish-receipts.mts", [6, 10, 14]],
  ["nullish-literals-and-types.mts", [5, 9, 14]],
  ["nullish-acquisitions.mts", [15, 26, 35]],
  ["nullish-branches.mts", [7, 11, 15, 19]],
  ["type-shapes.mts", [8, 12, 15, 18, 22]],
  ["recursive-indexed-types.mts", [5, 10]],
];
test(
  "compiled async typed executor projections share prepared reports",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const reports = cases.map(([entry], index) => ({
      type: "effects",
      id: `effects-${index}`,
      kind: "postgres",
      entry,
    }));
    reports.push({ type: "check", id: "check" });
    const pending = cjs.analyzeProject({ root, reports });
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.analyzeProject({ root, reports }), report);
    for (const [index, [entry, lines]] of cases.entries()) {
      const effects = report.reports.find((item) => item.id === `effects-${index}`).result;
      assert.deepEqual(
        effects.callSites.map((site) => site.line),
        lines,
        entry,
      );
      if (entry === "nullish-branches.mts") {
        assert.deepEqual(
          effects.callSites.map((site) => site.callee),
          ["TransactionQuery", "write", "write", "TransactionQuery"],
        );
      }
    }
    const check = report.reports.find((item) => item.id === "check").result;
    assert.deepEqual(check.warnings, []);
    assert.equal(
      check.rules.filter((finding) => finding.rule === "query-reached-per-item").length,
      cases.reduce((count, [, lines]) => count + lines.length, 0),
    );
  },
);
