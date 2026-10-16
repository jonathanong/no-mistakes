const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled namespace type barrels preserve effects and aggregate parity",
  { skip: !compiled },
  async () => {
    const root = join(
      __dirname,
      "../../../test-cases/rules/query-reached-per-item/namespace-types-fixture",
    );
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = { root, kind: "postgres", entry: "src/entry.mts" };
    const pending = cjs.effects(options);
    assert.equal(typeof pending.then, "function");
    const effects = await pending;
    assert.equal(effects.callSites.length, 3);
    assert.deepEqual(await esm.effects(options), effects);
    const check = await cjs.check({ root });
    assert.deepEqual(
      check.rules.map(({ file, line }) => [file, line]),
      [
        ["src/entry.mts", 8],
        ["src/entry.mts", 11],
        ["src/entry.mts", 14],
      ],
    );
    const reports = [
      { type: "effects", kind: "postgres", entry: "src/entry.mts" },
      { type: "check" },
    ];
    const aggregate = await cjs.analyzeProject({ root, reports });
    assert.deepEqual(
      aggregate.reports.map(({ result }) => result),
      [effects, check],
    );
    assert.deepEqual(await esm.analyzeProject({ root, reports }), aggregate);
  },
);

test(
  "compiled typed executors respect namespace and class type shadows",
  { skip: !compiled },
  async () => {
    const root = join(
      __dirname,
      "../../../test-cases/rules/query-reached-per-item/namespace-shadow-fixture",
    );
    const expected = require(join(root, "expected.json"));
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const reports = [];
    const results = [];
    for (const [entry, lines] of Object.entries(expected)) {
      const options = { root, kind: "postgres", entry };
      const pending = cjs.effects(options);
      assert.equal(typeof pending.then, "function");
      const result = await pending;
      assert.deepEqual(
        result.callSites.map(({ line }) => line),
        lines,
      );
      assert.ok(result.callSites.every(({ file }) => file === entry));
      assert.deepEqual(await esm.effects(options), result);
      reports.push({ type: "effects", kind: "postgres", entry });
      results.push(result);
    }
    const check = await cjs.check({ root });
    const wanted = Object.entries(expected).flatMap(([file, lines]) =>
      lines.map((line) => [file, line]),
    );
    assert.deepEqual(
      check.rules.map(({ file, line }) => [file, line]),
      wanted,
    );
    assert.deepEqual(await esm.check({ root }), check);
    reports.push({ type: "check" });
    results.push(check);
    const aggregate = await cjs.analyzeProject({ root, reports });
    assert.deepEqual(
      aggregate.reports.map(({ result }) => result),
      results,
    );
    assert.deepEqual(await esm.analyzeProject({ root, reports }), aggregate);
  },
);
