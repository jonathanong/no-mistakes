const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled CJS and ESM preserve cross-file inline receiver timing",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/cross-file-inline-receiver/fixture",
    );
    for (const [name, expected] of [
      ["earlier", 1],
      ["sibling", 1],
      ["reexported", 1],
      ["defaults", 1],
      ["mutated-default", 0],
      ["later", 0],
      ["prior", 0],
      ["other-context", 0],
      ["conflicting", 0],
      ["unknown", 0],
      ["escaped", 0],
    ]) {
      const options = { root, kind: "cache", entry: `src/${name}.mts` };
      const pending = cjs.effects(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.effects(options), report);
      assert.equal(report.callSites.length, expected, name);
      assert.ok(report.callSites.every((site) => site.callee === "primary.get"));
    }
  },
);
