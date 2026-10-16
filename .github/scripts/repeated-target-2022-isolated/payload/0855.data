const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");

function assertSpan(source, fact) {
  if (!fact?.span) return;
  assert.equal(
    Buffer.from(source).subarray(fact.span.start.offset, fact.span.end.offset).toString(),
    fact.sql,
  );
}

test(
  "compiled CJS/ESM expression spans cover rendered SQL bytes",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const declarations = readFileSync(join(__dirname, "../postgres-source-types.d.ts"), "utf8");
    const procedural = readFileSync(join(__dirname, "../postgres-procedural-types.d.ts"), "utf8");
    assert.match(
      declarations,
      /interface PostgresSqlExpression \{[\s\S]*span: PostgresSqlSpan \| null;/,
    );
    assert.match(procedural, /using: PostgresSqlExpression\[\];/);
    const sql = fixture("expression-span-bytes.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    const using = facts.statements[0].block.statements[0].execute.using[0];
    assert.equal(using.sql, "now()");
    assertSpan(sql, using);
    const coalesce = facts.statements[1].insert.source.rows[0][0];
    assert.equal(coalesce.sql, "coalesce(now(), now())");
    assertSpan(sql, coalesce);
    assert.ok(coalesce.children.some((child) => child.sql === "now()" && child.span));
    for (const child of coalesce.children) assertSpan(sql, child);
    const snow = facts.statements[2].insert.source.rows[0][0];
    assert.equal(snow.sql, "'雪'");
    assertSpan(sql, snow);
    const qualified = facts.statements[3].insert.source.rows[0][0];
    assert.equal(qualified.sql, "schema.col");
    assertSpan(sql, qualified);
    const selected = facts.statements[4].insert.columnSources.columns[0].sources[0].expression;
    assert.equal(selected.sql, "coalesce(now(), 1)");
    assertSpan(sql, selected);
    const rows = facts.statements[5].insert.source.rows;
    assert.equal(rows.length, 2);
    assert.equal(rows[0][0].sql, "1");
    assert.equal(rows[1][0].sql, "coalesce(now(), '雪')");
    for (const row of rows) for (const expression of row) assertSpan(sql, expression);
  },
);
