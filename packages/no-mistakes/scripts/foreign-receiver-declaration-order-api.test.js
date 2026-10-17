const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled async CJS and ESM ignore foreign callable declaration order",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/foreign-receiver-declaration-order/fixture",
    );
    for (const name of [
      "writer-first",
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
    ]) {
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
      assert.deepEqual(report.callSites, [], name);
    }
    for (const [name, expected] of [
      ["controls", ["primary.info", "replica.info", "unrelated.get", "replica.get", "primary.get"]],
      ["captured-root", ["replica.get"]],
      ["cold", ["primary.get"]],
    ]) {
      const options = {
        root,
        config: join(root, "effects.yml"),
        kind: "receiver",
        entry: `src/${name}.mts`,
      };
      const report = await cjs.effects(options);
      assert.deepEqual(await esm.effects(options), report, name);
      assert.deepEqual(
        report.callSites.map((site) => site.callee),
        expected,
        name,
      );
    }
  },
);
