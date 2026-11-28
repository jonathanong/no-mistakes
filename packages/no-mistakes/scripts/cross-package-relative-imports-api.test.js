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
    const config = join(root, "allow.yml");
    const pending = check({ root, config, includeSuppressed: true });
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.equal(report.rules.length, 3);
    assert.ok(report.rules.every((finding) => finding.rule === rule));
    assert.equal(report.advisories.length, 2);
    assert.ok(report.advisories.every((finding) => finding.message.startsWith("Unanalyzable")));
    const batch = await analyzeProject({
      root,
      config,
      reports: [{ type: "check", includeSuppressed: true }],
    });
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
        root,
        "--config",
        config,
        "--include-suppressed",
        "--format",
        "json",
      ],
      { encoding: "utf8" },
    );
    assert.equal(child.error, undefined);
    assert.equal(child.status, 1, child.stderr);
    assert.deepEqual(JSON.parse(child.stdout), report);

    const helperConfig = join(root, "helper-vitest-default.yml");
    const helperReport = await check({ root, config: helperConfig });
    assert.deepEqual(
      helperReport.advisories.map(({ line }) => line),
      [5, 6, 8, 9],
    );
    const helperBatch = await analyzeProject({
      root,
      config: helperConfig,
      reports: [{ type: "check" }],
    });
    assert.deepEqual(helperBatch.reports[0].result, helperReport);

    const invalidConfig = join(root, "allow-unknown-target.yml");
    await assert.rejects(check({ root, config: invalidConfig }), /allow\[0\]\.target/);
    await assert.rejects(
      analyzeProject({ root, config: invalidConfig, reports: [{ type: "check" }] }),
      /allow\[0\]\.target/,
    );

    const adviseConfig = join(root, "computed-default.yml");
    const adviseReport = await check({ root, config: adviseConfig });
    assert.deepEqual(adviseReport.rules, []);
    assert.equal(adviseReport.advisories.length, 1);
    assert.equal(adviseReport.advisories[0].file, "packages/app/computed/locale.mts");
    const adviseBatch = await analyzeProject({
      root,
      config: adviseConfig,
      reports: [{ type: "check" }],
    });
    assert.deepEqual(adviseBatch.reports[0].result, adviseReport);

    const reportConfig = join(root, "computed-rule-report.yml");
    const strictReport = await check({ root, config: reportConfig });
    assert.equal(strictReport.rules.length, 1);
    assert.equal(strictReport.rules[0].file, "packages/app/computed/locale.mts");
    assert.deepEqual(strictReport.advisories, []);
  },
);
