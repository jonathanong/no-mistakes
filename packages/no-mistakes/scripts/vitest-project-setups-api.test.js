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
    for (const [config, expected] of [
      [
        ".no-mistakes.yml",
        [
          "other/uncovered.test.mts",
          "nested/nested.test.mts",
          "nested/sibling.test.mts",
          "ordered/order.test.mts",
          "ordinary/cut.test.mts",
          "ordinary/sibling.test.mts",
          "overlap/helper-cut.test.mts",
          "overlap/shared.test.mts",
          "web/excluded.test.mts",
          "web/genuine.test.mts",
        ],
      ],
      [
        ".no-mistakes-both.yml",
        [
          "nested/nested.test.mts",
          "nested/sibling.test.mts",
          "ordered/order.test.mts",
          "ordinary/cut.test.mts",
          "ordinary/sibling.test.mts",
          "overlap/helper-cut.test.mts",
          "web/excluded.test.mts",
          "web/genuine.test.mts",
        ],
      ],
      [".no-mistakes-selected.yml", ["overlap/helper-cut.test.mts", "web/genuine.test.mts"]],
      [".no-mistakes-playwright.yml", ["overlap/shared.test.mts"]],
      [".no-mistakes-runner-ignored.yml", ["web/covered.test.mts"]],
      [".no-mistakes-app-project.yml", ["web/genuine.test.mts"]],
      [".no-mistakes-ordered.yml", []],
      [".no-mistakes-reversed.yml", ["ordered/order.test.mts"]],
      [".no-mistakes-nested-cut.yml", ["nested/nested.test.mts"]],
      [".no-mistakes-nested-live.yml", []],
      [".no-mistakes-nested-sibling.yml", ["nested/sibling.test.mts"]],
      [".no-mistakes-ordinary.yml", ["ordinary/cut.test.mts", "ordinary/sibling.test.mts"]],
    ]) {
      const options = { root, config: join(root, config) };
      const report = await cjs.check(options);
      assert.deepEqual(await esm.check(options), report);
      assert.deepEqual(report.rules.map((finding) => finding.file).sort(), expected);
    }
    for (const [config, expected] of [
      [".no-mistakes-missing.yml", "missing from the analysis file inventory"],
      [".no-mistakes-invalid.yml", "invalid repository-relative path"],
      [".no-mistakes-ignored.yml", "missing from the analysis file inventory"],
    ]) {
      const options = { root, config: join(root, config) };
      const invalid = await cjs.check(options);
      assert.deepEqual(await esm.check(options), invalid);
      assert.deepEqual(invalid.rules, []);
      assert.ok(
        invalid.warnings.some((warning) => warning.includes(expected)),
        invalid.warnings,
      );
    }
  },
);
