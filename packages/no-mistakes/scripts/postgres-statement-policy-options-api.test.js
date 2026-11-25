const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;

test(
  "compiled CJS, ESM and prepared statement policies preserve explicit empty lists",
  { skip: !process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const suite = join(__dirname, "../../../test-cases/rules/postgres-sql-statement-policy");
    const root = join(suite, "fixture/issue-3311");
    const defaults = [
      "CREATE TABLE",
      "ALTER TABLE",
      "CREATE INDEX",
      "CREATE VIEW",
      "TRUNCATE",
      "DROP INDEX",
      "DROP VIEW",
    ];
    for (const [name, statements, settingCount] of [
      ["omitted", defaults, 4],
      ["empty-with-settings", [], 4],
      ["empty", [], 0],
      ["explicit", ["CREATE VIEW"], 4],
    ]) {
      const config = join(suite, `configs/issue-3311-${name}.yml`);
      const options = { root, config };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(report.warnings, [], name);
      assert.deepEqual(report.advisories, [], name);
      const expected = [
        ...statements.flatMap((target) => [target, target]),
        ...Array(settingCount).fill("setting:statement_timeout"),
      ].sort();
      assert.deepEqual(report.rules.map(({ target }) => target).sort(), expected, name);
      assert.ok(report.rules.every(({ rule }) => rule === "postgres-sql-statement-policy"));
      for (const file of ["sql/policy.sql", "src/policy.test.ts"]) {
        const findings = report.rules.filter((finding) => finding.file === file);
        assert.equal(findings.length, statements.length + settingCount / 2, `${name}: ${file}`);
        assert.equal(
          findings.filter(({ target }) => target === "setting:statement_timeout").length,
          settingCount / 2,
          `${name}: ${file} settings`,
        );
      }
      assert.deepEqual(await esm.check(options), report, name);
      const request = { ...options, reports: [{ type: "check", id: name }] };
      const prepared = cjs.analyzeProject(request);
      assert.ok(prepared instanceof Promise);
      const batch = await prepared;
      assert.deepEqual(batch.reports, [{ type: "check", id: name, result: report }]);
      assert.deepEqual(await esm.analyzeProject(request), batch, name);
    }
  },
);

test(
  "compiled async statement clause policies match CLI reports",
  { skip: !process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const suite = join(__dirname, "../../../test-cases/rules/postgres-sql-statement-policy");
    const root = join(suite, "fixture/issue-3856");
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(
        dirname(process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH),
        process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes",
      );
    function checkCli(config) {
      const child = spawnSync(
        cli,
        ["--lock-timeout", "0", "check", "--root", root, "--config", config, "--format", "json"],
        { encoding: "utf8" },
      );
      assert.equal(child.error, undefined);
      return child;
    }
    for (const [name, exitCode, findingCount] of [
      ["omitted", 0, 0],
      ["clauses", 1, 12],
    ]) {
      const config = join(suite, `configs/issue-3856-${name}.yml`);
      const child = checkCli(config);
      assert.equal(child.status, exitCode, child.stderr);
      const expected = JSON.parse(child.stdout);
      assert.equal(expected.rules.length, findingCount, name);
      for (const api of [cjs, esm]) {
        const options = { root, config };
        const pending = api.check(options);
        assert.ok(pending instanceof Promise);
        assert.deepEqual(await pending, expected, name);
        const prepared = api.analyzeProject({ ...options, reports: [{ type: "check" }] });
        assert.ok(prepared instanceof Promise);
        assert.deepEqual((await prepared).reports[0].result, expected, name);
      }
    }

    const config = join(suite, "configs/issue-3856-unknown.yml");
    const child = checkCli(config);
    assert.equal(child.status, 2, child.stderr);
    assert.equal(child.stdout, "");
    const message = child.stderr.trim().replace(/^error: /, "");
    assert.match(
      message,
      /issue-3856-unknown\.yml:4: postgres-sql-statement-policy: unknown bannedClauses entry/,
    );
    const options = { root, config };
    for (const api of [cjs, esm]) {
      await assert.rejects(api.check(options), { message });
      await assert.rejects(api.analyzeProject({ ...options, reports: [{ type: "check" }] }), {
        message,
      });
    }
  },
);
