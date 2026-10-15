const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled async CJS and ESM agree on enclosing initialized captures",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-receiver-evaluation-order/fixture",
    );
    const options = { root, kind: "cache", entry: "src/enclosing-tdz.mts" };
    const pending = cjs.effects(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.effects(options), report);

    const sites = new Map(report.callSites.map((site) => [site.caller, site.callee]));
    for (const caller of ["neverRun", "adjacentRun", "nestedRun", "unreachableRun"]) {
      assert.equal(sites.has(caller), false, caller);
    }
    for (const caller of [
      "initializedRun",
      "conditionalRun",
      "branchRun",
      "hoistedRun",
      "staticInvoke",
    ]) {
      assert.equal(sites.get(caller), "primary.get", caller);
    }
  },
);
