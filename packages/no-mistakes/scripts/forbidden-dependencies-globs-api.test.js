const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

test(
  "compiled glob roots and origin exceptions share async CJS, ESM, and prepared checks",
  { skip: !process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/forbidden-dependencies-globs/fixture",
    );
    const options = { root, config: join(root, ".no-mistakes.yml") };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(
      report.rules.map(({ file, target }) => [file, target]),
      [
        ["backend/api/routes/get.mts", "shared/testing/helper.mts"],
        ["backend/services/service.mts", "shared/testing/helper.mts"],
        ["backend/workers/job.mts", "shared/testing/helper.mts"],
        ["ci/check-coverage.mts", "tests/other.mts"],
        ["ci/check-health.mts", "shared/testing/coverage.mts"],
        ["ci/check-health.mts", "shared/testing/helper.mts"],
        ["ci/check-health.mts", "tests/other.mts"],
        ["web/app/page.mts", "shared/testing/helper.mts"],
      ],
    );
    assert.ok(
      report.rules.every(({ rule, line }) => rule === "forbidden-dependencies" && line === 1),
    );
    assert.deepEqual(await esm.check(options), report);

    // The allowed helper remains traversable to the separate forbidden tests/other target.
    const traversal = { files: ["ci/check-coverage.mts"], relationships: ["import"] };
    const dependencies = await cjs.dependencies({ root, ...traversal });
    const request = {
      ...options,
      reports: [
        { type: "check", id: "boundaries" },
        { type: "dependencies", id: "imports", ...traversal },
      ],
    };
    const preparedPending = cjs.analyzeProject(request);
    assert.ok(preparedPending instanceof Promise);
    const prepared = await preparedPending;
    assert.deepEqual(prepared.reports, [
      { id: "boundaries", type: "check", result: report },
      { id: "imports", type: "dependencies", result: dependencies },
    ]);
    assert.deepEqual(await esm.analyzeProject(request), prepared);
  },
);
