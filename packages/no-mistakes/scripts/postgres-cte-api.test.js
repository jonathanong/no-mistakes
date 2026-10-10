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
    assert.equal(query.nestedStatements[3].returning[0].kind, "expression");
    const mergeEqualities = query.equalities.filter(
      (e) => e.scopeId === query.nestedStatements[3].queryScopeId,
    );
    assert.equal(mergeEqualities.length, 2);
    assert.ok(mergeEqualities.every((e) => !e.context.mandatory && e.clause === "other"));
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

test(
  "compiled CJS/ESM SELECT-source ON CONFLICT inside a CTE stays one complete query",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("query-cte-select-conflict.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 4);
    const query = facts.statements[0].query;
    assert.equal(query.complete, true);
    assert.equal(query.nestedStatements.length, 1);
    const child = query.nestedStatements[0];
    assert.equal(child.kind, "insert");
    assert.equal(child.complete, true);
    assert.equal(
      Buffer.from(sql).subarray(child.span.start.offset, child.span.end.offset).toString(),
      child.sql,
    );
    assert.equal(child.insert.source.kind, "select");
    assert.equal(child.insert.onConflict.action.kind, "doNothing");
    assert.ok(child.returning.length > 0);
    const topLevel = facts.statements[1].insert;
    assert.equal(facts.statements[1].kind, "insert");
    assert.equal(topLevel.complete, true);
    assert.equal(topLevel.onConflict.action.kind, "doNothing");
    const ordinary = facts.statements[2].query;
    assert.equal(ordinary.complete, true);
    assert.deepEqual(ordinary.nestedStatements, []);
    const values = facts.statements[3].query.nestedStatements[0];
    assert.equal(values.complete, true);
    assert.equal(values.insert.source.kind, "values");
    assert.equal(values.insert.onConflict.action.kind, "doNothing");
  },
);

test(
  "compiled CJS/ESM conflict actions retain canonical query expressions",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = { sql: fixture("query-cte-upsert.sql") };
    const facts = await cjs.parsePostgresSql(source);
    assert.deepEqual(await esm.parsePostgresSql(source), facts);
    assert.deepEqual(facts.diagnostics, []);
    const query = facts.statements[0].query;
    assert.equal(query.complete, true);
    const child = query.nestedStatements[0];
    assert.equal(child.complete, true);
    const columns = query.columns.filter(
      (c) => c.scopeId === child.queryScopeId && c.clause === "other",
    );
    assert.equal(columns.length, 3);
    assert.equal(columns.filter((c) => c.name.parts[0].identity === "excluded").length, 2);
    const target = columns.find((c) => c.name.parts[0].identity === "t");
    assert.equal(target.resolution, "resolved");
    assert.equal(query.relations[target.relationId].name.sql, "target");
    assert.ok(
      query.equalities.some(
        (e) => e.scopeId === child.queryScopeId && e.clause === "other" && !e.context.mandatory,
      ),
    );
    const incomplete = facts.statements[1].query;
    assert.equal(incomplete.complete, false);
    assert.equal(incomplete.ctes[0].referenced, true);
    assert.equal(incomplete.ctes[0].used, true);
  },
);

test(
  "compiled CJS/ESM write targets retain physical identity under CTE shadowing",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = { sql: fixture("query-cte-shadowed-targets.sql") };
    const facts = await cjs.parsePostgresSql(source);
    assert.deepEqual(await esm.parsePostgresSql(source), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 4);
    for (const { query } of facts.statements) {
      assert.equal(query.complete, true);
      assert.equal(query.ctes[0].referenced, false);
      assert.equal(query.ctes[0].used, false);
      const child = query.nestedStatements[0];
      const target = query.relations.find(
        (r) => r.scopeId === child.queryScopeId && r.name?.sql === "target",
      );
      assert.equal(target.kind, "table");
      assert.equal(target.cteId, null);
    }
  },
);

test(
  "compiled CJS/ESM INSERT source locks report explicit incompleteness",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = { sql: fixture("query-cte-source-locks.sql") };
    const facts = await cjs.parsePostgresSql(source);
    assert.deepEqual(await esm.parsePostgresSql(source), facts);
    assert.deepEqual(facts.diagnostics, []);
    for (const statement of facts.statements.slice(0, 2)) {
      assert.equal(statement.query.complete, false);
      const child = statement.query.nestedStatements[0];
      assert.equal(child.complete, false);
      assert.deepEqual(
        child.unsupported.map((item) => item.reason),
        ["INSERT source locking"],
      );
      assert.ok(child.unsupported[0].span.end.offset > child.unsupported[0].span.start.offset);
    }
    assert.equal(facts.statements[2].query.complete, true);
    assert.equal(facts.statements[3].query.complete, true);
  },
);

test(
  "compiled CJS/ESM RETURNING CTE inserts keep direct column lineage",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("insert-returning.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    const direct = facts.statements[0].insert;
    const same = facts.statements[10].query.nestedStatements[0];
    assert.equal(same.complete, true);
    assert.equal(same.insert.source.kind, "values");
    assert.equal(same.returning[0].kind, "expression");
    assert.deepEqual(
      same.insert.columnSources.columns.map((column) => [
        column.column.parts[0].identity,
        column.sources.map((source) => source.expression.sql),
      ]),
      direct.columnSources.columns.map((column) => [
        column.column.parts[0].identity,
        column.sources.map((source) => source.expression.sql),
      ]),
    );
    const selected = facts.statements[11].query.nestedStatements[0].insert;
    assert.equal(selected.source.kind, "select");
    assert.equal(selected.columnSources.kind, "mapped");
    assert.equal(
      selected.columnSources.columns[0].sources[0].expression.sql,
      facts.statements[2].insert.columnSources.columns[0].sources[0].expression.sql,
    );
    const plain = facts.statements[12].query.nestedStatements[0].insert;
    assert.equal(plain.columnSources, undefined);
    const reparsed = await cjs.parsePostgresSql({ sql: same.sql });
    assert.equal(reparsed.statements[0].insert.complete, true);
    assert.equal(reparsed.statements[0].insert.columnSources.kind, "mapped");
    const executeSql = fixture("insert-returning-execute.sql");
    const executed = await cjs.parsePostgresSql({ sql: executeSql });
    const nested =
      executed.statements[0].block.statements[1].execute.statements[0].query.nestedStatements[0];
    assert.equal(nested.insert.columnSources.columns[0].sources[0].expression.sql, "2");
    assert.ok(nested.sql.includes("RETURNING id"));

    const withValuesSql = fixture("insert-with-values-column-sources.sql");
    const withValues = await cjs.parsePostgresSql({ sql: withValuesSql });
    assert.deepEqual(await esm.parsePostgresSql({ sql: withValuesSql }), withValues);
    assert.deepEqual(withValues.diagnostics, []);
    const directWith = withValues.statements[0].insert;
    const changed = withValues.statements[1].query;
    const changedInsert = changed.nestedStatements[0];
    assert.equal(directWith.complete, false);
    assert.equal(directWith.source.kind, "values");
    assert.equal(directWith.columnSources.kind, "mapped");
    assert.equal(changedInsert.insert.source.kind, "values");
    assert.equal(changedInsert.insert.columnSources.kind, "mapped");
    assert.deepEqual(
      changedInsert.unsupported.map((item) => item.reason),
      ["INSERT VALUES query modifiers"],
    );
    assert.deepEqual(
      changedInsert.insert.columnSources.columns.map((column) => [
        column.column.parts[0].identity,
        column.sources.map((source) => [source.kind, source.rowIndex, source.expression.sql]),
      ]),
      directWith.columnSources.columns.map((column) => [
        column.column.parts[0].identity,
        column.sources.map((source) => [source.kind, source.rowIndex, source.expression.sql]),
      ]),
    );
    assert.equal(changedInsert.insert.columnSources.columns[0].sources[0].expression.sql, "1");
    const changedSeed = changed.ctes.find((cte) => cte.name.identity === "seed");
    assert.equal(
      changed.scopes[changedSeed.ownerScopeId].parentScopeId,
      changedInsert.queryScopeId,
    );
    const plainWith = withValues.statements[2].query;
    assert.equal(plainWith.nestedStatements[0].insert.columnSources, undefined);
    assert.equal(plainWith.nestedStatements[0].insert.source.kind, "values");
    assert.equal(
      plainWith.scopes[plainWith.ctes.find((cte) => cte.name.identity === "seed").ownerScopeId]
        .parentScopeId,
      plainWith.nestedStatements[0].queryScopeId,
    );
    const selectedWith = withValues.statements[3].query;
    assert.equal(selectedWith.nestedStatements[0].complete, true);
    assert.equal(selectedWith.nestedStatements[0].insert.source.kind, "select");
    assert.equal(
      selectedWith.scopes[
        selectedWith.ctes.find((cte) => cte.name.identity === "seed").ownerScopeId
      ].parentScopeId,
      selectedWith.nestedStatements[0].queryScopeId,
    );
  },
);
