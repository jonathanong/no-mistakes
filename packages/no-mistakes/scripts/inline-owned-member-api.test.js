const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled CJS and ESM preserve single inline method acquisitions",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/inline-member-precision/fixture",
    );
    const cases = [
      ["inline-snapshot-owned", 1],
      ["default-snapshot-owned", 1],
      ["default-object-snapshot-owned", 1],
      ["default-wrapped-snapshot-owned", 1],
      ["default-restored-snapshot-owned", 1],
      ["default-deferred-arrow", 0],
      ["default-deferred-function", 0],
      ["default-deferred-method", 0],
      ["default-deferred-field", 0],
      ["default-deferred-accessor", 0],
      ["default-deferred-nested", 0],
      ["default-snapshot-write-first", 0],
      ["default-live-receiver", 0],
      ["default-snapshot-async", 0],
      ["default-snapshot-captured", 1],
      ["default-receiver-captured", 0],
      ["alias-inline-snapshot-owned", 1],
      ["inline-snapshot-owned-write-first", 0],
      ["inline-snapshot-repeated-for", 0],
      ["inline-snapshot-repeated-header", 0],
      ["inline-snapshot-repeated-while", 0],
      ["inline-snapshot-repeated-do", 0],
      ["inline-snapshot-repeated-function", 0],
      ["inline-snapshot-repeated-field", 0],
      ["inline-snapshot-repeated-nested-field", 0],
      ["inline-snapshot-repeated-accessor", 0],
    ];
    const reports = cases.map(([name]) => ({
      type: "effects",
      kind: "cache",
      entry: `src/${name}.mts`,
    }));
    const pending = cjs.analyzeProject({ root, reports });
    assert.ok(pending instanceof Promise);
    const result = await pending;
    assert.deepEqual(await esm.analyzeProject({ root, reports }), result);
    for (const [index, [name, expected]] of cases.entries()) {
      const sites = result.reports[index].result.callSites;
      assert.equal(sites.length, expected, name);
      assert.ok(
        sites.every((site) => site.callee === "primary.get"),
        name,
      );
    }
  },
);
