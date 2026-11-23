const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled single-enqueue patterns agree across CLI, CJS, ESM and prepared checks",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/no-single-enqueue-in-loop/fixtures/pattern-targets",
    );
    for (const [config, lines] of [
      [".no-mistakes.yml", [13, 14, 17]],
      ["mixed.yml", [13, 14, 16, 20]],
      ["unscoped.yml", [13, 14, 17, 19]],
    ]) {
      const options = { root, config: join(root, config) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(report.warnings, []);
      assert.deepEqual(
        report.rules.map((finding) => finding.line),
        lines,
      );
      assert.ok(report.rules.every((finding) => finding.rule === "no-single-enqueue-in-loop"));
      assert.deepEqual(await esm.check(options), report);
      for (const api of [cjs, esm]) {
        const batch = await api.analyzeProject({
          root,
          reports: [{ type: "check", config: options.config }],
        });
        assert.deepEqual(batch.reports[0].result, report);
      }
      const cli =
        process.env.NO_MISTAKES_TEST_CLI_PATH ||
        join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
      const child = spawnSync(
        cli,
        ["check", "--root", root, "--config", options.config, "--format", "json"],
        { encoding: "utf8" },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, 1, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), report);
    }
  },
);
