const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
// Keep the closed uncalled writers as positive controls for the initial provider.
const receiverCases = [
  ...[
    "writer-first",
    "closure-writer-first",
    "closure-rebind-first",
    "closure-live-capture",
    "closure-writers-16",
    "closure-writers-64",
    "rebind-first",
    "reverse-calls",
    "uncalled",
    "deferred",
    "escaped",
    "cycle",
    "root-cycle",
    "mixed-writes",
    "multiple-origins",
    "scale-16",
    "scale-256",
  ].map((name) => [name, ["uncalled", "deferred"].includes(name) ? ["primary.get"] : []]),
  ["controls", ["primary.info", "replica.info", "unrelated.get", "replica.get", "primary.get"]],
  ["captured-root", ["replica.get"]],
  ["closure-module-captured", ["replica.get"]],
  ["closure-same-owner", ["replica.get"]],
  ["closure-controls", ["primary.info", "replica.info", "unrelated.get"]],
  ["cold", ["primary.get"]],
];

for (const [name, expected] of receiverCases)
  test(
    `compiled async CJS and ESM ignore foreign callable declaration order: ${name}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const root = join(
        __dirname,
        "../../../test-cases/codebase-analysis/foreign-receiver-declaration-order/fixture",
      );
      const options = {
        root,
        config: join(root, "effects.yml"),
        kind: "receiver",
        entry: `src/${name}.mts`,
      };
      const pending = cjs.effects(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.effects(options), report, name);
      assert.deepEqual(
        report.callSites.map((site) => site.callee),
        expected,
        name,
      );
    },
  );
