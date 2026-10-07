const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");

test(
  "compiled async CJS/ESM modifying CTE facts preserve source and scoped provenance",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("query-modifying-ctes.sql");
    const pending = cjs.parsePostgresSql({ sql });
    assert.equal(typeof pending.then, "function");
    const facts = await pending;
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    const query = facts.statements[0].query;
    assert.equal(query.complete, true);
    assert.deepEqual(
      query.nestedStatements.map((s) => s.kind),
      ["insert", "update", "delete", "merge"],
    );
    query.nestedStatements.forEach((s, ordinal) => {
      assert.equal(s.ordinal, ordinal);
      assert.equal(s.complete, true);
      assert.deepEqual(s.unsupported, []);
      assert.equal(
        Buffer.from(sql).subarray(s.span.start.offset, s.span.end.offset).toString(),
        s.sql,
      );
      const cte = query.ctes[s.cteId];
      assert.equal(cte.queryScopeId, s.queryScopeId);
      assert.equal(cte.ownerScopeId, s.parentScopeId);
    });
    assert.equal(query.nestedStatements[0].insert.table.parts[0].identity, "Schéma");
    assert.equal(query.nestedStatements[0].returning[0].alias.identity, "ID");
    assert.equal(query.nestedStatements[1].update.assignments.length, 1);
    assert.equal(query.nestedStatements[2].delete.usingRelationIds.length, 1);
    assert.deepEqual(
      query.nestedStatements[3].merge.clauses.map((c) => c.action.kind),
      ["update", "delete", "insert", "doNothing"],
    );
    const unreferenced = facts.statements[1].query;
    assert.equal(unreferenced.nestedStatements.length, 1);
    assert.equal(unreferenced.ctes[0].referenced, false);
    assert.equal(unreferenced.ctes[0].used, false);
    assert.deepEqual(unreferenced.nestedStatements[0].returning, []);
    const nested = facts.statements[2].query;
    assert.deepEqual(
      nested.nestedStatements.map((s) => nested.ctes[s.cteId].name.identity),
      ["inner_cte", "final_insert", "deeper"],
    );
    assert.ok(nested.nestedStatements[1].insert.source.queryScopeId < nested.scopes.length);
  },
);

test(
  "compiled modifying CTE unsupported children remain typed in both facades",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sources = ["query-modifying-ctes.sql", "query-modifying-ctes-unsupported.sql"].map(
      (name) => ({ sql: fixture(name), fileName: name }),
    );
    const facts = await cjs.parsePostgresSql(sources);
    assert.deepEqual(await esm.parsePostgresSql(sources), facts);
    assert.deepEqual(facts[1].diagnostics, []);
    const query = facts[1].statements[0].query;
    assert.equal(query.complete, false);
    assert.deepEqual(
      query.nestedStatements.map((s) => s.kind),
      ["insert", "merge"],
    );
    assert.ok(query.nestedStatements.every((s) => !s.complete && s.unsupported.length));
    assert.equal(query.nestedStatements[0].returning[0].kind, "expression");
  },
);

test(
  "compiled rejected CTE target predicate reports a diagnostic and preserves following query",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = { sql: fixture("query-cte-conflict-rejected.sql") };
    const facts = await cjs.parsePostgresSql(source);
    assert.deepEqual(await esm.parsePostgresSql(source), facts);
    assert.equal(facts.diagnostics.length, 1);
    assert.match(facts.diagnostics[0].message, /sql parser error/);
    assert.deepEqual(
      facts.statements.map((s) => s.kind),
      ["select"],
    );
    assert.equal(facts.statements[0].sql, "SELECT 42 AS neighbor;");
  },
);
