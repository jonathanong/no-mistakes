const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixtures = join(__dirname, "../../../test-cases");

test(
  "compiled changes diff shares directory and incomplete-root reports",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(fixtures, "changes-diff");
    const options = { baseRoot: join(root, "base"), headRoot: join(root, "head") };
    const pending = cjs.changesDiff(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.equal(report.schemaVersion, 1);
    assert.equal(report.complete, true);
    assert.deepEqual(report.fixtures, {
      changed: ["profile.avatar", "profile.description"],
      added: ["profile.new"],
      removed: ["profile.legacy"],
    });
    assert.deepEqual(report.catalogs.changed, ["label.welcome", "retained", "shared"]);
    assert.deepEqual(report.unclassified, [".notice", "ignored/data.txt"]);
    assert.deepEqual(await esm.changesDiff(options), report);
    const incomplete = await cjs.changesDiff({ ...options, baseRoot: join(root, "missing") });
    assert.equal(incomplete.complete, false);
    assert.equal(incomplete.reasons.length, 1);
  },
);

test(
  "compiled test planning consumes semantic reports and preserves safety fallback",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(fixtures, "tests-plan-changes");
    const options = { root, framework: "vitest", changes: "reports/matched.json" };
    const pending = cjs.testsPlan(options);
    assert.ok(pending instanceof Promise);
    const plan = await pending;
    assert.deepEqual(
      plan.selectedTests.map(({ testFile }) => testFile),
      ["tests/reader.test.ts"],
    );
    assert.equal(plan.fallbackTriggered, false);
    assert.deepEqual(await esm.testsPlan(options), plan);
    const additive = await cjs.testsPlan({ ...options, changedFiles: ["src/example.ts"] });
    assert.deepEqual(
      additive.selectedTests.map(({ testFile }) => testFile),
      ["tests/example.test.ts", "tests/reader.test.ts"],
    );
    const fallback = await cjs.testsPlan({ ...options, changes: "reports/incomplete.json" });
    assert.equal(fallback.fallbackTriggered, true);
    assert.equal(fallback.selectedTests.length, 3);
    assert.match(fallback.fallbackReason, /incomplete/);
  },
);

test(
  "directory metadata changes feed test planning without changing the body file",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const root = join(fixtures, "changes-diff/e2e");
    const report = await api.changesDiff({
      baseRoot: join(root, "base"),
      headRoot: join(root, "head"),
    });
    assert.deepEqual(report, require("../../../test-cases/changes-diff/e2e/changes.json"));
    const plan = await api.testsPlan({
      root: join(root, "head"),
      framework: "vitest",
      changes: "../changes.json",
    });
    assert.deepEqual(
      plan.selectedTests.map(({ testFile }) => testFile),
      ["tests/profile.test.ts"],
    );
    assert.equal(plan.fallbackTriggered, false);
  },
);
