const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");

test(
  "compiled CJS and ESM represent scalar ANY/ALL and fail closed for unsupported forms",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("query-scalar-quantified.sql");
    const pending = cjs.parsePostgresSql({ sql });
    assert.equal(typeof pending.then, "function");
    const facts = await pending;
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements[0].insert.complete, true);
    assert.equal(facts.statements[8].insert.complete, true);
    const queries = facts.statements.map(
      (statement) => statement.query || statement.insert.source.query,
    );
    const slice = (span) =>
      span && Buffer.from(sql).subarray(span.start.offset, span.end.offset).toString();
    for (const [index, [operator, quantifier]] of [
      ["=", "any"],
      ["<>", "all"],
      ["<", "any"],
      ["<=", "all"],
      [">", "any"],
      [">=", "all"],
      ["=", "any"],
    ].entries()) {
      const query = queries[index];
      assert.equal(query.complete, true);
      assert.deepEqual(query.unsupported, []);
      const comparison = query.quantifiedComparisons[0];
      assert.equal(comparison.operator, operator);
      assert.equal(comparison.quantifier, quantifier);
      assert.equal(comparison.complete, true);
      assert.equal(comparison.scopeId, 0);
      assert.equal(comparison.clause, "where");
      assert.equal(comparison.left.childrenComplete, true);
      assert.equal(comparison.right.childrenComplete, true);
      assert.ok(comparison.span);
      assert.equal(
        slice(comparison.span),
        [
          "x = ANY(t.aliases)",
          '"T"."x" <> ALL("T"."aliases")',
          "-1 < ANY(ARRAY[-1, 2])",
          "x <= ALL('{1,2}'::integer[])",
          "3 > ANY($1::integer[])",
          "$1 >= ALL(t.aliases)",
          "'x' = ANY(CAST($2 AS text[]))",
        ][index],
      );
    }
    const first = queries[0].quantifiedComparisons[0];
    assert.equal(slice(first.left.span), "x");
    assert.equal(slice(first.right.span), "t.aliases");
    assert.equal(first.left.root.kind, "columnReference");
    assert.deepEqual(
      queries[1].quantifiedComparisons[0].right.root.name.parts.map((part) => [
        part.identity,
        part.quoted,
      ]),
      [
        ["T", true],
        ["aliases", true],
      ],
    );
    const array = queries[2].quantifiedComparisons[0].right;
    assert.equal(array.root.kind, "array");
    assert.deepEqual(
      array.children.map((child) => [
        child.role,
        child.index,
        child.root.kind === "unary" ? child.children[0].root.value.value : child.root.value.value,
      ]),
      [
        ["arrayElement", 0, "1"],
        ["arrayElement", 1, "2"],
      ],
    );
    assert.equal(slice(array.children[0].children[0].span), "1");
    assert.equal(queries[3].quantifiedComparisons[0].right.root.kind, "cast");
    assert.equal(queries[4].quantifiedComparisons[0].right.children[0].root.placeholder, "$1");
    assert.equal(queries[5].quantifiedComparisons[0].left.root.placeholder, "$1");
    const contexts = queries[7].quantifiedComparisons;
    assert.equal(contexts[0].joinId, 0);
    assert.equal(contexts[1].context.underNot, true);
    assert.equal(contexts[2].context.underOr, true);
    assert.deepEqual(queries[7].equalities, []);
    assert.equal(queries[8].quantifiedComparisons.length, 0);
    assert.equal(queries[8].exists.length, 1);
    for (const name of [
      "query-scalar-quantified-unsupported.sql",
      "query-scalar-quantified-malformed.sql",
    ]) {
      const source = { sql: fixture(name) };
      const result = await cjs.parsePostgresSql(source);
      assert.deepEqual(await esm.parsePostgresSql(source), result);
      if (name.includes("malformed")) assert.ok(result.diagnostics.length);
      else
        for (const statement of result.statements) {
          assert.equal(statement.query.complete, false);
          assert.ok(statement.query.unsupported.length);
          assert.ok(
            statement.query.quantifiedComparisons.every((comparison) => !comparison.complete),
          );
        }
    }
  },
);
