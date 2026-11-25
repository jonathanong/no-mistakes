const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const rule = "terraform-iam-no-undocumented-wildcards";

test(
  "Terraform IAM wildcard findings and suppression agree across async Node and CLI",
  {
    skip: !addon?.endsWith(".node"),
  },
  async () => {
    const { check, analyzeProject } = require("../index.js");
    const root = join(__dirname, "../../../test-cases/rules", rule, "fixture");
    const pending = check({ root, includeSuppressed: true });
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(
      report.rules.map(({ rule, file, line, target }) => [rule, file, line, target ?? null]),
      [
        [rule, "policies/accepted.tf", 12, "unanalyzable"],
        [rule, "policies/accepted.tf", 18, "unanalyzable"],
        [rule, "policies/accepted.tf", 22, "unanalyzable"],
        [rule, "policies/accepted.tf", 32, "unanalyzable"],
        [rule, "policies/accepted.tf", 62, "unanalyzable"],
        [rule, "policies/accepted.tf", 75, "unanalyzable"],
        [rule, "policies/accepted.tf", 79, "unanalyzable"],
        [rule, "policies/example.tf.json", 1, "unanalyzable"],
        [rule, "policies/receipt.tf", 4, null],
        [rule, "policies/receipt.tf", 10, null],
        [rule, "policies/receipt.tf", 19, null],
        [rule, "policies/receipt.tf", 24, null],
        [rule, "policies/receipt.tf", 29, null],
      ],
    );
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
