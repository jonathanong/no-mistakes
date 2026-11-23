const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled shell compatibility checks share CLI, async, and prepared reports",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(__dirname, "../../../test-cases/rules/shell-compatibility/fixture");
    const config = join(root, "check.yml");
    const options = { root, config };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual([...new Set(report.rules.map(({ rule }) => rule))].sort(), [
      "bash-empty-array-nounset",
      "zsh-readonly-parameter",
    ]);
    assert.deepEqual(await esm.check(options), report);

    const request = { ...options, reports: [{ type: "check" }] };
    const prepared = await cjs.analyzeProject(request);
    assert.deepEqual(prepared.reports[0].result, report);
    assert.deepEqual(await esm.analyzeProject(request), prepared);

    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    const output = spawnSync(
      cli,
      ["check", "--root", root, "--config", config, "--format", "json"],
      {
        encoding: "utf8",
      },
    );
    assert.equal(output.error, undefined);
    assert.equal(output.status, 1, output.stderr);
    assert.deepEqual(JSON.parse(output.stdout), report);
  },
);
