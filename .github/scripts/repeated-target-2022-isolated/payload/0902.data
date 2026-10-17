const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled async CJS and ESM preserve receiver suspension boundaries",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/query-reached-per-item/provenance-fixture",
    );
    for (const [name, expected] of [
      ["global-after-capture-await", 0],
      ["global-await-operand-capture", 0],
      ["global-before-capture-await", 1],
      ["global-deferred-await", 1],
      ["global-await-saved", 1],
      ["global-after-capture-yield", 0],
      ["global-yield-operand-capture", 0],
      ["global-before-capture-yield", 1],
    ]) {
      const options = {
        root,
        kind: "valkey",
        entry: `src/nested-receiver-containers/${name}.mts`,
      };
      const pending = cjs.effects(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.effects(options), report, name);
      assert.equal(report.callSites.length, expected, name);
      assert.ok(
        report.callSites.every((site) => site.callee === "primary.get"),
        name,
      );
    }
  },
);
