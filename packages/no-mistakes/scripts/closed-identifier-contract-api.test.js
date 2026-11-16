const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled closed identifier contracts preserve CLI and async report parity",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3501",
    );
    const options = { root, config: join(root, ".no-mistakes.yml") };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.equal(report.rules.length, 50);
    assert.ok(
      report.rules.every((finding) => finding.file.replaceAll("\\", "/").startsWith("negative/")),
    );
    assert.deepEqual(await esm.check(options), report);
    const request = { ...options, reports: [{ type: "check", id: "bounded" }] };
    const batch = await cjs.analyzeProject(request);
    assert.deepEqual(batch.reports[0].result, report);
    assert.deepEqual(await esm.analyzeProject(request), batch);
    const cli = process.env.NO_MISTAKES_TEST_CLI_PATH || join(dirname(addon), "no-mistakes");
    const child = spawnSync(
      cli,
      ["check", "--root", root, "--config", options.config, "--format", "json"],
      { encoding: "utf8" },
    );
    assert.equal(child.error, undefined);
    assert.equal(child.status, 1, child.stderr);
    assert.deepEqual(JSON.parse(child.stdout), report);
  },
);
