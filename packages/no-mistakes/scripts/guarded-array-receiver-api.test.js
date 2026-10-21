const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled async guarded array effects retain both receiver candidates",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-receiver-escapes/fixture",
    );
    const options = {
      root,
      config: join(root, "effects.yml"),
      kind: "receiver",
      entry: "src/array-reassign-conditional.mts",
    };
    const pending = cjs.effects(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    // Keep all four old matches and recover the guarded replacement's original method.
    assert.deepEqual(
      report.callSites.map(({ line, callee }) => [line, callee]),
      [
        [12, "replica.get"],
        [13, "untouched.get"],
        [14, "primary.get"],
        [15, "replica.info"],
        [16, "unrelated.get"],
      ],
    );
    assert.deepEqual(await esm.effects(options), report);
    for (const api of [cjs, esm]) {
      const batch = api.analyzeProject({
        root,
        config: options.config,
        reports: [{ type: "effects", kind: "receiver", entry: options.entry }],
      });
      assert.ok(batch instanceof Promise);
      assert.deepEqual((await batch).reports[0].result, report);
    }
  },
);
