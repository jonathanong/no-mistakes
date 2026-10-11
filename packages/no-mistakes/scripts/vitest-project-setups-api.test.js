const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

const root = join(
  __dirname,
  "../../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-project-setups",
);

test(
  "compiled CJS and ESM checks scope declared Vitest setup mocks per project",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const report = await cjs.check({ root });
    assert.deepEqual(await esm.check({ root }), report);
    assert.deepEqual(report.rules.map((finding) => finding.file).sort(), [
      "other/uncovered.test.mts",
      "web/excluded.test.mts",
      "web/genuine.test.mts",
    ]);
    for (const [config, expected] of [
      [".no-mistakes-missing.yml", "missing from the analysis file inventory"],
      [".no-mistakes-invalid.yml", "invalid repository-relative path"],
    ]) {
      await assert.rejects(cjs.check({ root, config: join(root, config) }), new RegExp(expected));
      await assert.rejects(esm.check({ root, config: join(root, config) }), new RegExp(expected));
    }
  },
);
