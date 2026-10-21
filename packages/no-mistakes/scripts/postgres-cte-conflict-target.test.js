const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");

test(
  "compiled column-target WHERE inside a CTE stays one complete query",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("query-cte-conflict-target.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 5);
    const child = facts.statements[0].query.nestedStatements[0];
    assert.equal(child.complete, true);
    assert.equal(child.insert.onConflict.predicate.sql, "id > 0");
    assert.equal(child.insert.onConflict.action.kind, "doNothing");
    assert.equal(facts.statements[1].sql, "SELECT 42 AS neighbor;");
    const updated = facts.statements[2].query;
    const names = updated.ctes.map((cte) => [cte.name.identity, cte.referenced]);
    assert.deepEqual(names, [
      ["written", true],
      ["seen", true],
    ]);
    const conflict = updated.nestedStatements[0].insert.onConflict;
    assert.equal(conflict.predicate.sql, "id > 0");
    assert.equal(conflict.action.kind, "doUpdate");
    assert.equal(conflict.action.predicate.sql, "id < 9");
    const plain = facts.statements[3].query.nestedStatements[0];
    assert.equal(plain.insert.onConflict.action.kind, "doNothing");
    assert.equal(plain.insert.onConflict.predicate.sql, "id > 0");
    assert.deepEqual(plain.returning, []);
    const joined = facts.statements[4].query.nestedStatements[0];
    assert.equal(joined.complete, true);
    assert.equal(joined.insert.onConflict.predicate.sql, "id > 0");
    assert.match(joined.sql, /JOIN right_table ON/);
  },
);
