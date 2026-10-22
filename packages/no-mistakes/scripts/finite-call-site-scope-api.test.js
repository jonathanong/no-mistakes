const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(
  __dirname,
  "../../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped",
);

test(
  "compiled async checks retain finite calls outside ordinary scope",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = { root, config: ".no-mistakes.yml" };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const standalone = await pending;
    assert.deepEqual(await esm.check(options), standalone);
    assert.ok(
      standalone.rules.some(
        ({ rule, target }) => rule === "finite-set-consistency" && target === "missing",
      ),
    );
    assert.ok(
      standalone.rules.some(
        ({ rule, target }) => rule === "forbidden-dependencies" && target === "src/lazy.mts",
      ),
    );
    for (const api of [cjs, esm]) {
      const prepared = await api.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.deepEqual(prepared.reports[0].result, standalone);
    }
  },
);
