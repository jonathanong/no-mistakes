const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

for (const [fixture, selected] of [
  ["superseded-2065-projections-fixture", 10],
  ["superseded-2065-import-order-fixture", 3],
]) {
  const root = join(__dirname, "../../../test-cases/rules/query-reached-per-item", fixture);
  const expected = Object.entries(
    JSON.parse(readFileSync(join(root, "expected-lines.json"), "utf8")),
  );
  const reports = [
    ...expected.map(([entry]) => ({ type: "effects", kind: "postgres", entry })),
    { type: "check" },
  ];
  for (const [index, [entry, lines]] of expected.entries()) {
    test(
      `compiled preserved provenance entry: ${fixture}/${entry}`,
      { skip: !compiled },
      async () => {
        const cjs = require("../index.js");
        const esm = await import("../index.mjs");
        const options = { root, kind: "postgres", entry };
        const pending = cjs.effects(options);
        assert.equal(typeof pending.then, "function");
        const result = await pending;
        assert.deepEqual(
          result.callSites.map(({ file, line }) => [file, line]),
          lines.map((line) => [entry, line]),
        );
        assert.deepEqual(await esm.effects(options), result);
        const prepared = await cjs.analyzeProject({ root, reports });
        assert.deepEqual(prepared.reports[index].result, result);
      },
    );
  }

  test(`compiled preserved provenance root parity: ${fixture}`, { skip: !compiled }, async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const wanted = expected.flatMap(([file, lines]) => lines.map((line) => [file, line]));
    assert.equal(wanted.length, selected);
    const check = await cjs.check({ root });
    assert.deepEqual(
      check.rules.map(({ file, line }) => [file, line]),
      wanted,
    );
    assert.deepEqual(await esm.check({ root }), check);
    const prepared = await cjs.analyzeProject({ root, reports });
    assert.equal(prepared.reports.length, reports.length);
    assert.deepEqual(prepared.reports[expected.length].result, check);
    assert.deepEqual(await esm.analyzeProject({ root, reports }), prepared);
  });
  test(
    `compiled preserved provenance mixed-demand union: ${fixture}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const reports = [];
      const results = [];
      for (const [entry] of expected) {
        reports.push({ type: "effects", kind: "postgres", entry });
        results.push(await cjs.effects({ root, kind: "postgres", entry }));
      }
      reports.push({ type: "check" });
      results.push(await cjs.check({ root }));
      // Union demand must preserve every standalone report, including negative modules.
      const prepared = await cjs.analyzeProject({ root, reports });
      assert.deepEqual(
        prepared.reports.map(({ result }) => result),
        results,
      );
      assert.deepEqual(await esm.analyzeProject({ root, reports }), prepared);
    },
  );
}

test(
  "compiled plain non-nullish control preserves the complete report",
  { skip: !compiled },
  async () => {
    const root = join(
      __dirname,
      "../../../test-cases/rules/query-reached-per-item/superseded-2065-plain-fixture",
    );
    const expected = JSON.parse(readFileSync(join(root, "expected.json"), "utf8"));
    const options = { root, kind: "database", entry: "src/defaults.mts" };
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    assert.deepEqual(await cjs.effects(options), expected);
    assert.deepEqual(await esm.effects(options), expected);
    const reports = [{ type: "effects", kind: "database", entry: options.entry }];
    const prepared = await cjs.analyzeProject({ root, reports });
    assert.deepEqual(prepared.reports[0].result, expected);
    assert.deepEqual(await esm.analyzeProject({ root, reports }), prepared);
  },
);
