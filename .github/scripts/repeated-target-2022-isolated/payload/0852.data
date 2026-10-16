const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");

test(
  "compiled async CJS/ESM INSERT-source SELECT spans retain trailing calls",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("query-insert-source-call-spans.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    const source = Buffer.from(sql);
    const slice = (span) => source.subarray(span.start.offset, span.end.offset).toString();
    const cases = [
      {
        sourceSql: "SELECT now()",
        insertSql: 'INSERT INTO "té" SELECT now() RETURNING id',
        rootSql: 'WITH c AS (INSERT INTO "té" SELECT now() RETURNING id) SELECT * FROM c',
      },
      {
        sourceSql: "SELECT lower('é')",
        insertSql: "INSERT INTO t SELECT lower('é') ON CONFLICT DO NOTHING RETURNING id",
        rootSql:
          "WITH c AS (INSERT INTO t SELECT lower('é') ON CONFLICT DO NOTHING RETURNING id) SELECT * FROM c",
      },
      {
        sourceSql: "SELECT t.returning, t.on, t.conflict, now()",
        insertSql:
          "INSERT INTO t SELECT t.returning, t.on, t.conflict, now() ON CONFLICT DO NOTHING RETURNING t.returning",
        rootSql:
          "WITH c AS (INSERT INTO t SELECT t.returning, t.on, t.conflict, now() ON CONFLICT DO NOTHING RETURNING t.returning) SELECT * FROM c",
      },
      {
        sourceSql: "SELECT now() UNION ALL SELECT now()",
        insertSql: "INSERT INTO t SELECT now() UNION ALL SELECT now() RETURNING id",
        rootSql:
          "WITH c AS (INSERT INTO t SELECT now() UNION ALL SELECT now() RETURNING id) SELECT * FROM c",
        branches: ["SELECT now()", "SELECT now()"],
      },
    ];
    assert.equal(facts.statements.length, cases.length);
    for (const [index, expected] of cases.entries()) {
      const query = facts.statements[index].query;
      assert.equal(query.complete, true);
      assert.deepEqual(query.unsupported, []);
      const child = query.nestedStatements[0];
      assert.equal(child.kind, "insert");
      assert.equal(child.complete, true);
      assert.equal(child.sql, expected.insertSql);
      assert.equal(slice(child.span), expected.insertSql);
      assert.equal(slice(query.scopes[child.queryScopeId].span), expected.insertSql);
      assert.equal(child.insert.source.kind, "select");
      assert.equal(slice(child.insert.source.span), expected.sourceSql);
      assert.equal(slice(query.scopes[child.insert.source.queryScopeId].span), expected.sourceSql);
      assert.equal(slice(query.scopes[0].span), expected.rootSql);
      if (expected.branches) {
        assert.deepEqual(
          query.scopes
            .filter((scope) => scope.clause === "setBranch")
            .map((scope) => slice(scope.span)),
          expected.branches,
        );
      }
    }
  },
);

test(
  "compiled async CJS/ESM read-only CTE scopes keep exact UTF-8 source slices",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("query-readonly-cte-spans.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    const source = Buffer.from(sql);
    const scopes = facts.statements.map((statement) => {
      assert.equal(statement.query.complete, true);
      assert.deepEqual(statement.query.unsupported, []);
      return statement.query.scopes.map((scope) => {
        assert.ok(scope.span);
        return source.subarray(scope.span.start.offset, scope.span.end.offset).toString();
      });
    });
    assert.equal(scopes.length, 5);
    for (const expected of ["SELECT now()", "WITH s AS (SELECT now()) SELECT count(*) FROM s"]) {
      assert.ok(scopes[0].includes(expected), `${expected}: ${scopes[0]}`);
    }
    for (const expected of [
      "SELECT 'é' AS word",
      "SELECT lower('é')",
      "WITH nested AS (SELECT lower('é')) SELECT * FROM nested",
      "WITH first AS (SELECT 'é' AS word), second AS (WITH nested AS (SELECT lower('é')) SELECT * FROM nested) SELECT count(*) FROM first, second",
    ]) {
      assert.ok(scopes[1].includes(expected), `${expected}: ${scopes[1]}`);
    }
    assert.ok(scopes[2].includes("SELECT now()"));
    assert.ok(scopes[3].includes("SELECT now()"));
    assert.ok(scopes[3].includes("SELECT (SELECT now()) AS value"));
    assert.ok(scopes[4].includes("(SELECT now()) ORDER BY 1"));
    assert.ok(scopes[4].includes("SELECT now()"));
  },
);

test(
  "compiled async CJS/ESM set branches keep exact operand source slices",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("query-set-branch-spans.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    const source = Buffer.from(sql);
    const branches = facts.statements.map((statement) => {
      assert.equal(statement.query.complete, true);
      return statement.query.scopes
        .filter((scope) => scope.clause === "setBranch")
        .map((scope) => source.subarray(scope.span.start.offset, scope.span.end.offset).toString());
    });
    assert.equal(branches.length, 17);
    assert.deepEqual(branches[0], ["SELECT 'é'", "SELECT now()"]);
    assert.deepEqual(branches[1], ["(SELECT now())", "SELECT now()", "SELECT now()"]);
    assert.deepEqual(branches[2], ["SELECT now()", "(SELECT now())", "SELECT now()"]);
    assert.deepEqual(branches[3], ["SELECT 1", "SELECT 2 AS order"]);
    assert.deepEqual(branches[4], ["SELECT 1", "SELECT t.limit FROM metrics AS t"]);
    assert.deepEqual(branches[5], ["SELECT now()", "SELECT now()"]);
    assert.deepEqual(branches[6], [
      "SELECT now() UNION ALL SELECT now()",
      "SELECT now()",
      "SELECT now()",
      "SELECT now()",
    ]);
    assert.deepEqual(branches[7], ["SELECT now()", "SELECT now()"]);
    assert.deepEqual(branches[8], ["SELECT now()", "SELECT now()"]);
    assert.deepEqual(branches[9], ["SELECT now()", "SELECT now()"]);
    assert.deepEqual(branches[10], ["SELECT 1", "SELECT t.fetch FROM metrics AS t"]);
    assert.ok(branches[11].includes("((SELECT now()))"));
    assert.ok(branches[12].includes("((SELECT now()))"));
    assert.deepEqual(branches[13], ["SELECT now()", "SELECT now()"]);
    assert.deepEqual(branches[14], ["SELECT now()", "SELECT now()"]);
    assert.deepEqual(branches[15], ["VALUES (1)", "VALUES (2)"]);
    assert.deepEqual(branches[16], ["SELECT now()", "SELECT now()"]);

    const chainSql = fixture("query-set-branch-chain.sql");
    const chain = await cjs.parsePostgresSql({ sql: chainSql });
    assert.deepEqual(chain.diagnostics, []);
    const chainSource = Buffer.from(chainSql);
    const chainBranches = chain.statements[0].query.scopes
      .filter((scope) => scope.clause === "setBranch")
      .map((scope) =>
        chainSource.subarray(scope.span.start.offset, scope.span.end.offset).toString(),
      );
    assert.equal(chainBranches.length, 46);
    assert.equal(chainBranches.filter((slice) => slice === "SELECT now()").length, 24);
  },
);

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

function nestedChildren(statement) {
  if (statement.kind === "select") return statement.query.nestedStatements;
  if (statement.kind === "doBlock") {
    return statement.block.statements.flatMap(nestedChildren);
  }
  return [];
}

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

test(
  "compiled nested data-modifying CTE sql matches its span and reparses",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const cases = [
      {
        sql: "WITH a AS (INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING) SELECT 1;",
        expected: ["INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING"],
      },
      {
        sql: "WITH a AS (INSERT INTO t(id) SELECT 1 WHERE NOT EXISTS (SELECT 1)) SELECT 1;",
        expected: ["INSERT INTO t(id) SELECT 1 WHERE NOT EXISTS (SELECT 1)"],
      },
      {
        sql: "WITH a AS (INSERT INTO t(id) VALUES (1) /* kept */ ON CONFLICT (id) DO NOTHING RETURNING id /* tail */) SELECT 1;",
        expected: [
          "INSERT INTO t(id) VALUES (1) /* kept */ ON CONFLICT (id) DO NOTHING RETURNING id",
        ],
      },
      {
        sql: "DO $$ BEGIN WITH a AS (INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING) SELECT 1; END $$;",
        expected: ["INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING"],
      },
      {
        sql: "WITH a AS ((INSERT INTO t VALUES (1) ON CONFLICT DO NOTHING)) SELECT 1;",
        expected: ["INSERT INTO t VALUES (1) ON CONFLICT DO NOTHING"],
      },
      {
        sql: "WITH a AS ((INSERT INTO t(id) SELECT 1 WHERE NOT EXISTS (SELECT 1))) SELECT 1;",
        expected: ["INSERT INTO t(id) SELECT 1 WHERE NOT EXISTS (SELECT 1)"],
      },
      {
        sql: "WITH a AS ((INSERT INTO t(id) VALUES (1) /* kept */ ON CONFLICT (id) DO NOTHING RETURNING id /* tail */)) SELECT 1;",
        expected: [
          "INSERT INTO t(id) VALUES (1) /* kept */ ON CONFLICT (id) DO NOTHING RETURNING id",
        ],
      },
    ];
    for (const { sql, expected } of cases) {
      const facts = await cjs.parsePostgresSql({ sql });
      assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
      assert.deepEqual(facts.diagnostics, []);
      const children = facts.statements.flatMap(nestedChildren);
      assert.deepEqual(
        children.map((child) => child.sql),
        expected,
      );
      let previous = 0;
      for (const child of children) {
        assert.equal(child.complete, true);
        assert.equal(
          Buffer.from(sql).subarray(child.span.start.offset, child.span.end.offset).toString(),
          child.sql,
        );
        assert.ok(child.span.start.offset >= previous);
        assert.ok(child.span.end.offset > child.span.start.offset);
        previous = child.span.end.offset;
        const again = await cjs.parsePostgresSql({ sql: child.sql });
        assert.deepEqual(again.diagnostics, []);
        assert.equal(again.statements[0].sql, child.sql);
      }
    }
  },
);

test(
  "compiled CJS/ESM RETURNING CTE column sources keep direct call trivia",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("insert-returning-cte-spans.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    const slice = (span) =>
      Buffer.from(sql).subarray(span.start.offset, span.end.offset).toString();
    const direct = facts.statements[0].insert.columnSources;
    const nested = facts.statements[1].query.nestedStatements[0].insert.columnSources;
    const plain = facts.statements[2].query.nestedStatements[0].insert;
    for (const sources of [direct, nested]) {
      assert.equal(sources.kind, "mapped");
      assert.equal(sources.complete, true);
      const expression = sources.columns[0].sources[0].expression;
      assert.equal(expression.sql, "now()");
      assert.equal(slice(expression.span), "now /*keep*/ ()");
    }
    assert.equal(plain.columnSources, undefined);
  },
);
