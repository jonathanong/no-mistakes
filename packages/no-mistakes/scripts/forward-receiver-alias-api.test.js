const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

for (const file of ["before", "after"]) {
  test(
    `compiled async receiver aliases preserve acquisitions with ${file} writer declaration`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const root = join(
        __dirname,
        "../../../test-cases/codebase-analysis/value-flow-forward-receiver-alias/fixture",
      );
      const options = { root, kind: "receiver", entry: `src/${file}.mts` };
      const pending = cjs.effects(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.effects(options), report);
      // Live methods changed; the two previously acquired methods remain effects.
      assert.deepEqual(report.callSites.map((site) => site.callee).sort(), [
        "primary.get",
        "replica.get",
        "unrelated.get",
      ]);
      assert.ok(report.callSites.every((site) => site.caller === undefined));
      for (const api of [cjs, esm]) {
        const batch = await api.analyzeProject({
          root,
          reports: [{ type: "effects", kind: "receiver", entry: options.entry }],
        });
        assert.deepEqual(batch.reports[0].result, report);
      }
    },
  );
}

for (const [file, expected] of [
  ["only-before", ["replica.get"]],
  ["only-after", ["primary.get"]],
  ["shadow", ["primary.get"]],
  ["uncalled", []],
  ["uncalled-after", []],
  ["tdz", []],
  ["escaped", []],
  ["cyclic", []],
  ["grouped", []],
  ["uninitialized", ["primary.get"]],
]) {
  test(`compiled async forward receiver alias control: ${file}`, { skip: !compiled }, async () => {
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-forward-receiver-alias/fixture",
    );
    const options = { root, kind: "receiver", entry: `src/${file}.mts` };
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const report = await cjs.effects(options);
    assert.deepEqual(await esm.effects(options), report);
    assert.deepEqual(
      report.callSites.map((site) => site.callee),
      expected,
    );
  });
}
