const assert = require("node:assert/strict");
const { join } = require("node:path");
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
