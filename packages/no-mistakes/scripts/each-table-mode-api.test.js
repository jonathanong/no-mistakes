const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled each-table mode preserves async check, prepared analysis and CLI parity",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const saved = join(__dirname, "../../../test-cases/rules/test-fixture-size-cap");
    const root = join(saved, "fixture/each-table-mode");
    const options = { root, config: join(saved, "configs/all-elements.yml") };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(
      report.rules.map(({ line }) => line),
      [8, 13, 25],
    );
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(await esm.check(options), report);
    const request = { ...options, reports: [{ type: "check", id: "table-sizes" }] };
    const prepared = cjs.analyzeProject(request);
    assert.ok(prepared instanceof Promise);
    const analysis = await prepared;
    assert.deepEqual(analysis.reports, [{ type: "check", id: "table-sizes", result: report }]);
    assert.deepEqual(await esm.analyzeProject(request), analysis);
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    const child = spawnSync(
      cli,
      ["check", "--root", root, "--config", options.config, "--format", "json"],
      { encoding: "utf8", timeout: 30_000, env: process.env },
    );
    assert.equal(child.error, undefined);
    assert.equal(child.status, 1, child.stderr);
    assert.deepEqual(JSON.parse(child.stdout), report);
  },
);
