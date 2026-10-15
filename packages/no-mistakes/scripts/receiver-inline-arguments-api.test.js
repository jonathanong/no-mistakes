const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled async CJS and ESM preserve repeated inline argument reads",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = {
      root: join(
        __dirname,
        "../../../test-cases/codebase-analysis/value-flow-receiver-evaluation-order/fixture",
      ),
      kind: "direct-inline",
      entry: "src/direct-inline-arguments.mts",
    };
    const pending = cjs.effects(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.effects(options), report);
    assert.deepEqual(report.callSites.map((site) => [site.caller, site.callee]).sort(), [
      ["takeCopied", "directSaved.get"],
      ["takeOnce", "directOnce.get"],
      ["takeSibling", "directSibling.set"],
    ]);
  },
);
