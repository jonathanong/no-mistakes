const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled script checks preserve input policy without hiding same-named expressions",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/github-actions-script-expressions/fixture/unreadable-input",
    );
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    for (const prefix of ["", "collision-"]) {
      for (const policy of ["report", "advise", "ignore"]) {
        const options = { root, config: join(root, `${prefix}${policy}.yml`) };
        const pending = cjs.check(options);
        assert.ok(pending instanceof Promise);
        const report = await pending;
        const findings = report.rules.filter(
          (row) => row.rule === "github-actions-script-expressions",
        );
        const advisories = report.advisories.filter(
          (row) => row.rule === "github-actions-script-expressions",
        );
        assert.equal(findings.length, prefix || policy === "report" ? 1 : 0);
        assert.equal(advisories.length, !prefix && policy === "advise" ? 1 : 0);
        for (const row of [...findings, ...advisories]) {
          assert.equal(row.target, "unanalyzable");
          if (prefix) {
            assert.equal(row.file, ".github/workflows/collision.yml");
            assert.ok(row.line > 1);
          } else {
            assert.equal(row.file, ".github/workflows/bad.yml");
            assert.equal(row.line, 2);
            assert.match(row.message, /read/i);
          }
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
        assert.equal(child.status, prefix || policy === "report" ? 1 : 0, child.stderr);
        assert.deepEqual(JSON.parse(child.stdout), report);
      }
    }
  },
);
