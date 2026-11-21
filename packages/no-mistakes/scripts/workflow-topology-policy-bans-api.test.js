const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const eventsRoot = join(
  __dirname,
  "../../../test-cases/rules/workflow-topology-policy/fixture/forbidden-events",
);
const secretsRoot = join(
  __dirname,
  "../../../test-cases/rules/workflow-topology-policy/fixture/secrets-inherit",
);
const configs = join(eventsRoot, "../../configs");

test(
  "compiled forbidden events and secrets inherit bans share async checks",
  { skip: !process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const events = {
      root: eventsRoot,
      config: join(configs, "forbidden-events.yml"),
    };
    const pending = cjs.check(events);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(
      report.rules.map(({ rule, file, line }) => [rule, file, line]),
      [
        ["workflow-topology-policy", ".github/workflows/comment-gap.yml", 4],
        ["workflow-topology-policy", ".github/workflows/duplicate.yml", 2],
        ["workflow-topology-policy", ".github/workflows/flow-mapping.yml", 2],
        ["workflow-topology-policy", ".github/workflows/flow-sequence.yml", 4],
        ["workflow-topology-policy", ".github/workflows/mapping.yml", 3],
        ["workflow-topology-policy", ".github/workflows/null-mapping.yml", 3],
        ["workflow-topology-policy", ".github/workflows/quoted-event.yml", 3],
        ["workflow-topology-policy", ".github/workflows/quoted-on.yml", 3],
        ["workflow-topology-policy", ".github/workflows/scalar.yml", 2],
        ["workflow-topology-policy", ".github/workflows/sequence-block.yml", 4],
        ["workflow-topology-policy", ".github/workflows/sequence.yml", 2],
        ["workflow-topology-policy", ".github/workflows/stale-null.yml", 3],
        ["workflow-topology-policy", ".github/workflows/stale-sequence.yml", 3],
        ["workflow-topology-policy", ".github/workflows/tagged-on.yml", 1],
      ],
    );
    assert.ok(report.rules.every((finding) => finding.message.includes("pull_request_target")));
    assert.deepEqual(await esm.check(events), report);

    const custom = await cjs.check({
      root: eventsRoot,
      config: join(configs, "forbidden-events-custom.yml"),
    });
    assert.deepEqual(
      custom.rules.map(({ message }) => message),
      Array(14).fill("Use pull_request for these widget workflows."),
    );
    const empty = await cjs.check({
      root: eventsRoot,
      config: join(configs, "forbidden-events-empty.yml"),
    });
    assert.deepEqual(empty.rules, []);

    const secrets = {
      root: secretsRoot,
      config: join(configs, "secrets-inherit.yml"),
    };
    const secretReport = await cjs.check(secrets);
    assert.deepEqual(
      secretReport.rules.map(({ rule, file, line }) => [rule, file, line]),
      [
        ["workflow-topology-policy", ".github/workflows/widgets.yml", 6],
        ["workflow-topology-policy", ".github/workflows/widgets.yml", 18],
        ["workflow-topology-policy", ".github/workflows/widgets.yml", 21],
      ],
    );
    assert.ok(secretReport.rules.every((finding) => finding.message.includes("secrets: inherit")));
    assert.deepEqual(await esm.check(secrets), secretReport);
    const secretOff = await cjs.check({
      root: secretsRoot,
      config: join(configs, "secrets-inherit-false.yml"),
    });
    assert.deepEqual(secretOff.rules, []);

    const request = { ...events, reports: [{ type: "check", id: "forbidden-events" }] };
    const preparedPending = cjs.analyzeProject(request);
    assert.ok(preparedPending instanceof Promise);
    const prepared = await preparedPending;
    assert.deepEqual(prepared.reports, [{ id: "forbidden-events", type: "check", result: report }]);
    assert.deepEqual(await esm.analyzeProject(request), prepared);
  },
);
