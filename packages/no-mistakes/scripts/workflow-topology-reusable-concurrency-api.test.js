const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const root = join(
  __dirname,
  "../../../test-cases/rules/workflow-topology-policy/fixture/reusable-concurrency",
);
const config = (name) => join(root, "../../configs", `reusable-concurrency${name}.yml`);

test(
  "compiled reusable workflow concurrency policy shares async check findings",
  { skip: !process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = { root, config: config("") };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(
      report.rules.map(({ rule, file, line }) => [rule, file, line]),
      [
        ["workflow-topology-policy", ".github/workflows/mapping.yml", 6],
        ["workflow-topology-policy", ".github/workflows/scalar.yml", 3],
        ["workflow-topology-policy", ".github/workflows/sequence.yml", 3],
      ],
    );
    assert.ok(report.rules.every(({ message }) => message.includes("workflow-level concurrency")));
    assert.deepEqual(await esm.check(options), report);
    for (const name of ["-false", "-disabled"]) {
      assert.deepEqual((await cjs.check({ root, config: config(name) })).rules, []);
    }
    const custom = await cjs.check({ root, config: config("-custom") });
    assert.deepEqual(
      custom.rules.map(({ message }) => message),
      Array(3).fill("Declare concurrency on the caller or on individual jobs."),
    );
    const request = { ...options, reports: [{ type: "check", id: "reusable-locks" }] };
    const preparedPending = cjs.analyzeProject(request);
    assert.ok(preparedPending instanceof Promise);
    const prepared = await preparedPending;
    assert.deepEqual(prepared.reports, [{ id: "reusable-locks", type: "check", result: report }]);
    assert.deepEqual(await esm.analyzeProject(request), prepared);
  },
);
