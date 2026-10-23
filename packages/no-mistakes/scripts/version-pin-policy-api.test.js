const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled version pin checks report unavailable sources across public APIs",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/version-pin-consistency/fixture/unavailable-source",
    );
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    for (const policy of ["report", "advise", "ignore"]) {
      const options = { root, config: join(root, `${policy}.yml`) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      const findings = report.rules.filter((row) => row.rule === "version-pin-consistency");
      const advisories = report.advisories.filter((row) => row.rule === "version-pin-consistency");
      assert.equal(findings.length, policy === "report" ? 1 : 0);
      assert.equal(advisories.length, policy === "advise" ? 1 : 0);
      for (const row of [...findings, ...advisories]) {
        assert.equal(row.file, "README.md");
        assert.equal(row.line, 1);
        assert.equal(row.target, "unanalyzable");
        assert.match(row.message, /versions\.toml/);
      }
      assert.deepEqual(await esm.check(options), report);
      const request = { ...options, reports: [{ type: "check" }] };
      const batch = await cjs.analyzeProject(request);
      assert.deepEqual(batch.reports[0].result, report);
      assert.deepEqual(await esm.analyzeProject(request), batch);
      const child = spawnSync(
        cli,
        ["check", "--root", root, "--config", options.config, "--format", "json"],
        { encoding: "utf8" },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, policy === "report" ? 1 : 0, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), report);
    }
  },
);
