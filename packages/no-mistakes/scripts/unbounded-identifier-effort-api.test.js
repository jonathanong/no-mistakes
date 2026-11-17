const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled unbounded identifier effort preserves async CLI and shared-report parity",
  {
    skip: !addon?.endsWith(".node"),
  },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/effort-low",
    );
    const cli = process.env.NO_MISTAKES_TEST_CLI_PATH || join(dirname(addon), "no-mistakes");
    for (const config of [
      ".no-mistakes.yml",
      ".no-mistakes-advise.yml",
      ".no-mistakes-ignore.yml",
    ]) {
      const options = { root, config: join(root, config) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const low = await pending;
      assert.ok(low.rules.length > 0, "proven env relation violations survive every policy");
      assert.deepEqual(await esm.check(options), low);
      const child = spawnSync(
        cli,
        ["check", "--root", root, "--config", options.config, "--format", "json"],
        { encoding: "utf8" },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, 1, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), low);
      const max = await cjs.check({ ...options, effort: "max" });
      const request = {
        ...options,
        reports: [
          { type: "check", id: "low", effort: "low" },
          { type: "check", id: "max", effort: "max" },
        ],
      };
      const batch = await cjs.analyzeProject(request);
      assert.deepEqual(batch.reports.find((report) => report.id === "low").result, low);
      assert.deepEqual(batch.reports.find((report) => report.id === "max").result, max);
      assert.deepEqual(await esm.analyzeProject(request), batch);
    }
  },
);

test(
  "compiled SQL argument aliases preserve bind pairing across async report APIs",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3525",
    );
    const options = { root, effort: "max" };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(
      report.rules.map((finding) => finding.line).sort((a, b) => a - b),
      [7, 20, 27, 51, 59, 77],
    );
    assert.deepEqual(await esm.check(options), report);
    const request = { ...options, reports: [{ type: "check" }] };
    const batch = await cjs.analyzeProject(request);
    assert.deepEqual(batch.reports[0].result, report);
    assert.deepEqual(await esm.analyzeProject(request), batch);
  },
);
