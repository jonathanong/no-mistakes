const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

for (const [fixture, entry, expected] of [
  ["exact-fixture", "entry", []],
  ["fixture", "entry", ["other.get", "shared.get", "shared.set"]],
  ["fixture", "barrel-entry", ["other.get", "shared.get", "shared.set"]],
]) {
  test(
    `compiled async exported receiver aliases: ${fixture}/${entry}`,
    { skip: !compiled },
    async () => {
      const root = join(
        __dirname,
        "../../../test-cases/codebase-analysis/value-flow-exported-value-receiver",
        fixture,
      );
      const options = { root, kind: "receiver", entry: `src/${entry}.mts` };
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const pending = cjs.effects(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.effects(options), report);
      assert.deepEqual(report.callSites.map((site) => site.callee).sort(), expected);
      // The live read is line 6 in the exact reproduction and line 7 in controls.
      assert.ok(
        report.callSites.every((site) => site.line !== (fixture === "exact-fixture" ? 6 : 7)),
      );
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
