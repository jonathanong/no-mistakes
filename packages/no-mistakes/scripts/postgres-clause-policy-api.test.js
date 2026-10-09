const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled CJS, ESM and prepared checks preserve scoped and legacy SQL policies",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-sql-shape-policy/fixture/clause-api",
    );
    const source = readFileSync(join(root, "sql/queries.sql"), "utf8");
    const expected = source.split("\n").flatMap((line, index) => {
      const kind = /-- finding: (scoped|legacy)/.exec(line)?.[1];
      return kind ? [{ line: index + 1, kind }] : [];
    });
    const pending = cjs.check({ root });
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(await esm.check({ root }), report);
    assert.deepEqual(await cjs.check({ root }), report);
    assert.deepEqual(
      report.rules.map((finding) => [finding.rule, finding.file, finding.line]),
      expected.map(({ line }) => ["postgres-sql-shape-policy", "sql/queries.sql", line]),
    );
    for (const { line, kind } of expected) {
      const finding = report.rules.find((entry) => entry.line === line);
      assert.equal(finding.target, "banned-function-call");
      if (kind === "scoped") {
        assert.match(finding.message, /uuidv7\(\) is banned in/);
        assert.ok(finding.message.endsWith("; compute a stable bound once per statement"));
      } else {
        assert.equal(
          finding.message,
          `sql/queries.sql:${line}: function call "pg_sleep" is banned by this SQL shape policy; remove the call or replace it with an allowed operation`,
        );
      }
    }
    const batch = await cjs.analyzeProject({
      root,
      reports: [{ type: "check", id: "clauses" }],
    });
    assert.deepEqual(batch.reports[0].result, report);
    const included = await cjs.check({ root, includeSuppressed: true });
    assert.deepEqual(included.rules, report.rules);
    assert.equal(included.suppressed.length, 3);
  },
);
