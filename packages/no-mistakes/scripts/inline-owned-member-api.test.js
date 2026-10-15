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
    cases.push(
      ["loop-setup-annex.cjs", 0],
      ["loop-setup-async.mts", 0],
      ["loop-setup-for-declaration.mts", 1],
      ["loop-setup-for-expression.mts", 1],
      ["loop-setup-for-update.mts", 0],
      ["loop-setup-function.mts", 0],
      ["loop-setup-in-body.mts", 0],
      ["loop-setup-in-default.mts", 0],
      ["loop-setup-in-rhs.mts", 1],
      ["loop-setup-in-target.mts", 0],
      ["loop-setup-nested-for.mts", 0],
      ["loop-setup-nested-in.mts", 0],
      ["loop-setup-nested-of.mts", 0],
      ["loop-setup-of-body.mts", 0],
      ["loop-setup-of-default.mts", 0],
      ["loop-setup-of-rhs.mts", 1],
      ["loop-setup-of-target.mts", 0],
    );
    const reports = cases.map(([name]) => ({
      type: "effects",
      kind: "cache",
      entry: `src/${name.includes(".") ? name : `${name}.mts`}`,
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
