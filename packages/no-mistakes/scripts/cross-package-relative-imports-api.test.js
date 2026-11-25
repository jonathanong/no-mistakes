const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "Workspace-relative import findings agree across async Node checks and the CLI",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const { check, analyzeProject } = require("../index.js");
    const rule = "no-cross-package-relative-imports";
    const root = join(__dirname, "../../../test-cases/rules", rule, "fixture");
    const pending = check({ root, includeSuppressed: true });
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.equal(report.rules.length, 7);
    assert.ok(report.rules.every((finding) => finding.rule === rule));
    assert.equal(report.suppressed.length, 3);
    const batch = await analyzeProject({
      root,
      reports: [{ type: "check", includeSuppressed: true }],
    });
    assert.deepEqual(batch.reports[0].result, report);

    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    const child = spawnSync(
      cli,
      ["--lock-timeout", "0", "check", "--root", root, "--include-suppressed", "--format", "json"],
      { encoding: "utf8" },
    );
    assert.equal(child.error, undefined);
    assert.equal(child.status, 1, child.stderr);
    assert.deepEqual(JSON.parse(child.stdout), report);
  },
);
