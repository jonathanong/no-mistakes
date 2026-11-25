const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "package export prefix findings agree across async Node APIs and CLI",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const { check, analyzeProject } = require("../index.js");
    const base = join(__dirname, "../../../test-cases/rules/package-json-export-target-prefix");
    const options = { root: join(base, "fixture"), config: join(base, "configs/advisory.yml") };
    const pending = check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(
      report.rules.map(({ target }) => target),
      ["/exports", "/exports/./import"],
    );
    assert.deepEqual(
      report.advisories.map(({ target }) => target),
      ["unanalyzable"],
    );
    const batch = await analyzeProject({ ...options, reports: [{ type: "check" }] });
    assert.deepEqual(batch.reports[0].result, report);
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    const child = spawnSync(
      cli,
      [
        "--lock-timeout",
        "0",
        "check",
        "--root",
        options.root,
        "--config",
        options.config,
        "--format",
        "json",
      ],
      { encoding: "utf8" },
    );
    assert.equal(child.error, undefined);
    assert.equal(child.status, 1, child.stderr);
    assert.deepEqual(JSON.parse(child.stdout), report);
  },
);
