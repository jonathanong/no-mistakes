const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled PostgreSQL locations match across worker counts and prepared reports",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const relative of [
      "test-cases/rules/postgres-fk-index/fixture/statement-spans",
      "fixtures/rules/postgres-migration-statement-lines",
    ]) {
      const root = join(__dirname, "../../..", relative);
      const report = await cjs.check({ root });
      assert.ok(report.rules.length > 0);
      for (const jobs of [1, 4]) {
        const parallel = await cjs.check({ root, jobs });
        // The native process pool is fixed; its resize warning is not a finding.
        delete parallel.jobsPoolWarning;
        assert.deepEqual(parallel, report);
      }
      assert.deepEqual(await esm.check({ root }), report);
      const prepared = await cjs.analyzeProject({ root, reports: [{ type: "check" }] });
      assert.deepEqual(prepared.reports[0].result, report);
      if (relative.includes("postgres-fk-index")) {
        const repeated = report.rules.filter(({ file }) => file === "repeated.sql");
        assert.deepEqual(
          repeated.map(({ target, line }) => [target, line]),
          [
            ["Attribute.personId", 1],
            ["Session.personId", 2],
          ],
        );
      }
    }
  },
);

test(
  "compiled schema rules accept release-blocker DDL with explicit report policy",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(__dirname, "../../../fixtures/postgres-facts/source/ddl-compat");
    for (const config of [".no-mistakes.yml", "guarded-do.yml", "guarded-function.yml"]) {
      const report = await cjs.check({ root, config });
      assert.deepEqual(await esm.check({ root, config }), report);
      const prepared = await cjs.analyzeProject({ root, config, reports: [{ type: "check" }] });
      assert.deepEqual(prepared.reports[0].result, report);
      assert.deepEqual(report.rules, []);
      assert.deepEqual(report.advisories, []);
    }
    const options = { root, config: "plain-query-policy.yml" };
    const report = await cjs.check(options);
    assert.deepEqual(await esm.check(options), report);
    const prepared = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
    assert.deepEqual(prepared.reports[0].result, report);
    assert.deepEqual(
      report.rules.map(({ line }) => line),
      [7, 8],
    );
    assert.ok(
      report.rules.every(
        ({ rule, target }) =>
          rule === "postgres-sql-shape-policy" && target === "banned-function-call",
      ),
    );
  },
);

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

// Each scenario independently checks its CJS, ESM, and prepared-report contract.
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
  ["postgres-ddl-unanalyzable", "report.yml", 69, 0, "procedural"],
  ["postgres-ddl-unanalyzable", "default.yml", 121, 77, "unsupported-policy"],
  ["postgres-ddl-unanalyzable", "report.yml", 198, 0, "unsupported-policy"],
  ["postgres-ddl-unanalyzable", "advise.yml", 0, 198, "unsupported-policy"],
  ["postgres-ddl-unanalyzable", "ignore.yml", 0, 0, "unsupported-policy"],
  ["postgres-ddl-unanalyzable", "override.yml", 198, 0, "unsupported-policy"],
]) {
  test(
    `compiled recall policy: ${scenario}/${subdirectory || "root"}/${config}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const fixtures = join(__dirname, "../../../test-cases/rules");
      const options = { root: join(fixtures, scenario, "fixture", subdirectory), config };
      const report = await cjs.check(options);
      assert.deepEqual(await esm.check(options), report);
      const prepared = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.deepEqual(prepared.reports[0].result, report);
      assert.deepEqual(report.warnings, []);
      assert.equal(report.rules.length, findings, config);
      assert.equal(report.advisories.length, advisories, config);
    },
  );
}

for (const policy of ["default", "report", "ignore", "malformed"]) {
  test(
    `compiled PostgreSQL 17 DDL survey: ${policy} policy across eleven schema rules`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const root = join(
        __dirname,
        "../../../test-cases/rules/postgres-ddl-unanalyzable/fixture/docs-survey",
      );
      const malformedCount = readFileSync(join(root, "malformed/001.sql"), "utf8")
        .trim()
        .split("\n").length;
      const options = { root, config: `${policy}.yml` };
      const report = await cjs.check(options);
      assert.deepEqual(await esm.check(options), report);
      const prepared = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.deepEqual(prepared.reports[0].result, report);
      assert.deepEqual(report.warnings, []);
      const unanalyzable = (items) =>
        items.filter(({ message }) => message.includes("could not be analyzed"));
      const findings = unanalyzable(report.rules);
      const advisories = unanalyzable(report.advisories);
      if (policy === "malformed") {
        assert.equal(findings.length, malformedCount * 11);
        assert.deepEqual(advisories, []);
      } else if (policy === "ignore") {
        assert.deepEqual(findings, []);
        assert.deepEqual(advisories, []);
      } else {
        const selected = policy === "report" ? findings : advisories;
        assert.deepEqual(policy === "report" ? advisories : findings, []);
        // Unsupported ALTER DOMAIN must retain one diagnostic per schema rule.
        const rules = selected
          .filter(({ file, line }) => file === "migrations/001.sql" && line === 5)
          .map(({ rule }) => rule);
        assert.equal(new Set(rules).size, 11);
      }
    },
  );
}
