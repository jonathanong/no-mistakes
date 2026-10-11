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
  { skip: !compiled, timeout: 20_000 },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const [config, expected] of [
      [
        ".no-mistakes.yml",
        [
          "nested/nested.test.mts",
          "nested/sibling.test.mts",
          "ordered/order.test.mts",
          "ordinary/cut.test.mts",
          "ordinary/sibling.test.mts",
          "other/uncovered.test.mts",
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
      [".no-mistakes-runner-overlap.yml", ["web/covered.test.mts"]],
      [".no-mistakes-runner-reverse.yml", ["web/covered.test.mts"]],
      [".no-mistakes-runner-vitest-only.yml", ["web/covered.test.mts"]],
      [".no-mistakes-runner-vitest-config-overlap.yml", ["web/covered.test.mts"]],
      [".no-mistakes-runner-jest-config-overlap.yml", ["web/covered.test.mts"]],
      [".no-mistakes-runner-playwright-only.yml", ["web/covered.test.mts"]],
      [".no-mistakes-stale-app.yml", ["web/covered.test.mts"]],
      [".no-mistakes-cjs-helper.yml", []],
      [".no-mistakes-mixed-runners.yml", ["web/covered.test.mts"]],
      [".no-mistakes-disjoint-runners.yml", ["other/uncovered.test.mts"]],
      [".no-mistakes-inherit.yml", []],
      [".no-mistakes-independent.yml", ["web/covered.test.mts"]],
      [".no-mistakes-omitted.yml", ["web/covered.test.mts"]],
      [".no-mistakes-cleared.yml", ["web/covered.test.mts"]],
      [".no-mistakes-referenced.yml", ["web/covered.test.mts"]],
      [".no-mistakes-playwright-invalid-vitest.yml", ["web/covered.test.mts"]],
      [".no-mistakes-playwright-config.yml", ["web/covered.test.mts"]],
      [".no-mistakes-mixed-config-runners.yml", ["web/covered.test.mts"]],
      [".no-mistakes-partial-projects.yml", ["web/covered.test.mts"]],
      [".no-mistakes-partial-app.yml", ["web/covered.test.mts"]],
      [".no-mistakes-partial-selected.yml", []],
      [".no-mistakes-chained-cuts.yml", []],
      [".no-mistakes-lazy-requires.yml", ["lazy/ordinary.case.cjs", "lazy/setup.case.cjs"]],
      [".no-mistakes-playwright-custom.yml", ["custom/sample.case.mts"]],
      [".no-mistakes-repository-playwright.yml", ["web/covered.test.mts"]],
      [".no-mistakes-app-playwright.yml", ["web/covered.test.mts"]],
      [".no-mistakes-shared-runner-path.yml", ["web/covered.test.mts"]],
    ]) {
      const options = { root, config: join(root, config) };
      const report = await cjs.check(options);
      assert.deepEqual(await esm.check(options), report);
      assert.deepEqual(report.rules.map((finding) => finding.file).sort(), expected, config);
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

test(
  "compiled checks use implicit Vitest workspace and project catalogs",
  { skip: !compiled, timeout: 20_000 },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const kind of ["workspace", "projects"]) {
      const options = { root: join(root, "../test-no-unmocked-dynamic-imports-workspaces", kind) };
      const report = await cjs.check(options);
      assert.deepEqual(await esm.check(options), report);
      assert.deepEqual(
        report.rules.map((finding) => finding.file),
        ["custom/other.case.mts"],
      );
      assert.deepEqual(report.warnings, []);
    }
  },
);

test(
  "compiled checks resolve runner-local aliases outside declared workspaces",
  { skip: !compiled, timeout: 20_000 },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = { root: join(root, "../test-no-unmocked-dynamic-imports-runner-aliases") };
    const report = await cjs.check(options);
    assert.deepEqual(await esm.check(options), report);
    assert.deepEqual(report.rules, []);
    assert.deepEqual(report.warnings, []);
  },
);

test(
  "compiled test planning follows explicit project setup dependencies",
  { skip: !compiled, timeout: 20_000 },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const fixture = join(__dirname, "../../../fixtures/test-plan/explicit-policy-setups");
    for (const file of ["setup/web.ts", "setup/helper.ts"]) {
      const options = {
        root: fixture,
        framework: "vitest",
        changedFiles: [file],
        globalConfigFallback: false,
      };
      const plan = await cjs.testsPlan(options);
      assert.deepEqual(await esm.testsPlan(options), plan);
      assert.deepEqual(
        plan.selectedTests.map((test) => test.testFile),
        ["web/live.test.ts"],
      );
      assert.equal(plan.fallbackTriggered, false);
      const impactOptions = { root: fixture, entrypoints: [file] };
      const impact = await cjs.testsImpact(impactOptions);
      assert.deepEqual(await esm.testsImpact(impactOptions), impact);
      assert.deepEqual(
        impact.selectedTests.map((test) => test.testFile),
        ["web/live.test.ts"],
      );
    }
  },
);
