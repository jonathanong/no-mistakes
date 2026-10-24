const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled checks enforce expression arbiter dependencies without changing source facts",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const fixture = join(__dirname, "../../../test-cases/rules/postgres-idempotent-insert/fixture");
    for (const scenario of [
      "expression-arbiters",
      "expression-arbiters-safe",
      "expression-arbiter-recovered",
    ]) {
      const root = join(fixture, scenario);
      const report = await cjs.check({ root });
      assert.deepEqual(await esm.check({ root }), report);
      assert.deepEqual(report.warnings, []);
      const prepared = await cjs.analyzeProject({ root, reports: [{ type: "check" }] });
      assert.deepEqual(prepared.reports[0].result, report);
      if (scenario === "expression-arbiters-safe") {
        assert.deepEqual(report.rules, []);
      } else if (scenario === "expression-arbiter-recovered") {
        assert.deepEqual(
          report.rules.map(({ file }) => file),
          ["sql/malformed-neighbor.sql", "sql/malformed-neighbor.sql"],
        );
        assert.match(report.rules[0].message, /expression arbiter depends on email/);
        // Recovery keeps the INSERT proof and reports its unsupported neighbor.
        assert.equal(report.rules[1].line, 4);
        assert.equal(report.rules[1].target, "SELECT * FROM");
        assert.match(report.rules[1].message, /could not be analyzed/);
      } else {
        assert.deepEqual(
          report.rules.map(({ file }) => file),
          [
            "sql/coupled_partial.sql",
            "sql/expression.sql",
            "sql/mixed.sql",
            "sql/parenthesized.sql",
            "sql/plain_column_control.sql",
            "sql/quoted.sql",
            "sql/unsupported.sql",
          ],
        );
        for (const finding of report.rules)
          assert.equal(finding.rule, "postgres-idempotent-insert");
      }
    }
    const root = join(fixture, "expression-arbiters");
    for (const name of ["expression.sql", "mixed.sql", "parenthesized.sql", "quoted.sql"]) {
      const sql = readFileSync(join(root, "sql", name), "utf8");
      const parsed = await cjs.parsePostgresSql({ sql });
      assert.deepEqual(await esm.parsePostgresSql({ sql }), parsed);
      assert.deepEqual(parsed.diagnostics, []);
      const insert = parsed.statements[0].insert;
      assert.equal(insert.complete, true);
      assert.equal(insert.onConflict.target.kind, "expressions");
      assert.ok(insert.onConflict.target.expressions.length > 0);
      for (const expression of insert.onConflict.target.expressions) {
        assert.ok(expression.span);
        assert.equal(
          Buffer.from(sql)
            .subarray(expression.span.start.offset, expression.span.end.offset)
            .toString(),
          expression.sql,
        );
      }
    }
  },
);

test(
  "compiled checks preserve recall policies across CJS, ESM and project reports",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const fixtures = join(__dirname, "../../../test-cases/rules");
    for (const [scenario, config, findings, advisories, subdirectory = ""] of [
      ["forbidden-calls/consumer-matrix", "configs/unknown-calls-default.yml", 1, 0],
      ["forbidden-calls/consumer-matrix", "configs/unknown-calls-finding.yml", 1, 0],
      ["forbidden-calls/consumer-matrix", "configs/unknown-calls-ignore.yml", 0, 0],
      ["postgres-ddl-unanalyzable", "report.yml", 22, 0],
      ["postgres-ddl-unanalyzable", "advise.yml", 0, 22],
      ["postgres-ddl-unanalyzable", "ignore.yml", 0, 0],
      ["postgres-ddl-unanalyzable", "narrow.yml", 2, 0],
      ["postgres-ddl-unanalyzable", "report.yml", 3, 0, "unsupported-valid"],
      ["postgres-ddl-unanalyzable", "positive.yml", 1, 0, "copy"],
      ["postgres-ddl-unanalyzable", "report.yml", 9, 0, "procedural"],
    ]) {
      const options = { root: join(fixtures, scenario, "fixture", subdirectory), config };
      const report = await cjs.check(options);
      assert.deepEqual(await esm.check(options), report);
      const prepared = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.deepEqual(prepared.reports[0].result, report);
      assert.deepEqual(report.warnings, []);
      assert.equal(report.rules.length, findings, config);
      assert.equal(report.advisories.length, advisories, config);
    }
  },
);
