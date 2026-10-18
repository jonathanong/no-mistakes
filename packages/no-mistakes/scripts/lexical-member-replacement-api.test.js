const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled async effects invalidate one-time lexical writes before a loop",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-lexical-member-replacement/fixture",
    );
    for (const [entry, expected] of [
      [
        "entry",
        [
          [3, "primary.get"],
          [8, "primary.get"],
          [9, "primary.info"],
          [10, "untouched.get"],
        ],
      ],
      ["after", [[2, "primary.get"]]],
      ["minimal", []],
    ]) {
      const options = { root, kind: "cache", entry: `src/${entry}.mts` };
      const pending = cjs.effects(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.effects(options), report);
      assert.deepEqual(
        report.callSites.map(({ line, callee }) => [line, callee]).sort((a, b) => a[0] - b[0]),
        expected,
      );
    }
  },
);

test(
  "compiled checks report replaced lexical members as unknown",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-lexical-member-replacement/fixture",
    );
    const options = { root, config: join(root, "rules.yml") };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.check(options), report);
    assert.deepEqual(
      report.rules.map(({ line, target }) => [line, target]),
      [
        [7, "unknown call"],
        [13, "unknown call"],
        [15, "unknown call"],
      ],
    );
  },
);
