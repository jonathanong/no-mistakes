const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiled = addon?.endsWith(".node");

for (const [config, failures, advice] of [
  [".no-mistakes.yml", 0, 6],
  ["report.yml", 6, 0],
  ["ignore.yml", 0, 0],
]) {
  test(
    `unused disable directives preserve CLI and async CJS/ESM parity: ${config}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const root = join(__dirname, "../../../fixtures/check/unused-disable-directives");
      const options = { root, config: join(root, config) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const expected = await pending;
      assert.equal(expected.rules.length, failures);
      assert.equal(expected.advisories.length, advice);
      assert.equal(expected.suppressed, undefined);
      const diagnostics = [...expected.rules, ...expected.advisories].sort(
        (a, b) => a.line - b.line,
      );
      assert.ok(diagnostics.every((finding) => finding.file === "web/app/stale.ts"));
      if (diagnostics.length) {
        assert.deepEqual(
          diagnostics.map((finding) => finding.line),
          [1, 2, 4, 5, 7, 9],
        );
        assert.match(diagnostics[3].message, /disabled/);
        assert.match(diagnostics[4].message, /unknown/);
      }
      for (const api of [cjs, esm]) {
        assert.deepEqual(await api.check(options), expected);
        const batchPending = api.analyzeProject({
          ...options,
          reports: [{ type: "check" }, { type: "check", includeSuppressed: true }],
        });
        assert.ok(batchPending instanceof Promise);
        const batch = await batchPending;
        assert.deepEqual(batch.reports[0].result, expected);
        const accounted = batch.reports[1].result;
        assert.deepEqual(accounted.rules, expected.rules);
        assert.deepEqual(accounted.advisories, expected.advisories);
        assert.equal(accounted.suppressed.length, 3);
      }
      const cli =
        process.env.NO_MISTAKES_TEST_CLI_PATH ||
        join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
      const child = spawnSync(
        cli,
        ["check", "--root", root, "--config", options.config, "--format", "json"],
        { encoding: "utf8", env: process.env },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, failures ? 1 : 0, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), expected);
    },
  );
}
