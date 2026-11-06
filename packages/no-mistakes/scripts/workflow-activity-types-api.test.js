const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const root = join(
  __dirname,
  "../../../test-cases/rules/workflow-topology-policy/fixture/activity-types",
);

test(
  "compiled activity type policy shares async CJS, ESM, and prepared checks",
  { skip: !process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = { root, config: join(root, "../../configs/activity-types.yml") };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(
      report.rules.map(({ rule, file, line }) => [rule, file, line]),
      [
        ["workflow-topology-policy", ".github/workflows/events.yml", 3],
        ["workflow-topology-policy", ".github/workflows/events.yml", 5],
        ["workflow-topology-policy", ".github/workflows/events.yml", 7],
      ],
    );
    assert.ok(report.rules[0].message.includes("pull_request: ready_for_review"));
    assert.ok(report.rules[1].message.includes("pull_request_target: converted_to_draft"));
    assert.ok(report.rules[2].message.includes("issues: ready_for_review"));
    assert.deepEqual(await esm.check(options), report);

    const custom = await cjs.check({
      root,
      config: join(root, "../../configs/activity-types-custom.yml"),
    });
    assert.deepEqual(
      custom.rules.map(({ message }) => message),
      Array(3).fill("Remove draft transitions; these workflows do not read draft state."),
    );
    const filtered = await cjs.check({
      root,
      config: join(root, "../../configs/activity-types-filter.yml"),
    });
    assert.deepEqual(
      filtered.rules.map(({ rule, file, line }) => [rule, file, line]),
      [["workflow-topology-policy", ".github/workflows/events.yml", 3]],
    );

    const request = { ...options, reports: [{ type: "check", id: "activity-types" }] };
    const preparedPending = cjs.analyzeProject(request);
    assert.ok(preparedPending instanceof Promise);
    const prepared = await preparedPending;
    assert.deepEqual(prepared.reports, [{ id: "activity-types", type: "check", result: report }]);
    assert.deepEqual(await esm.analyzeProject(request), prepared);
  },
);
