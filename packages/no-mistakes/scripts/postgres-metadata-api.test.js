const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");

test(
  "compiled comment facts accept dollar, Unicode and escape string literals",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = { sql: fixture("comment-literals.sql") };
    const facts = await cjs.parsePostgresSql(source);
    assert.deepEqual(await esm.parsePostgresSql(source), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.deepEqual(
      facts.statements.slice(0, 7).map((statement) => statement.comment.comment),
      ["dollar text", "tagged text", "doc", "escaped\nline", "table dollar", "view", null],
    );
    assert.equal(facts.statements[7].block.statements[0].branches[0].statements[0].kind, "comment");
  },
);

test(
  "compiled CJS and ESM accept typed PostgreSQL metadata and newline strings",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("valid-metadata.sql");
    const pending = cjs.parsePostgresSql({ sql });
    assert.equal(typeof pending.then, "function");
    const facts = await pending;
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 12);
    assert.equal(facts.statements[0].kind, "comment");
    assert.deepEqual(facts.statements[0].comment.arguments, []);
    assert.equal(facts.statements[0].comment.name.parts[0].identity, "example_function");
    assert.equal(facts.statements[1].kind, "alterIndex");
    assert.equal(facts.statements[1].index.operation.kind, "attachPartition");
    assert.equal(facts.statements[1].index.operation.partition.sql, "example_child");
    assert.equal(facts.statements[2].comment.comment, "First. Second.");
    assert.equal(facts.statements[3].comment.arguments[0].name.identity, "Name");
    assert.equal(facts.statements[3].comment.arguments[1].mode, "VARIADIC");
    assert.equal(facts.statements[3].comment.comment, "雪's documentation");
    assert.equal(facts.statements[4].comment.comment, null);
    assert.equal(facts.statements[5].comment.arguments, null);
    assert.equal(facts.statements[6].index.ifExists, false);
    assert.equal(facts.statements[7].index.ifExists, true);
    assert.equal(facts.statements[7].index.operation.kind, "rename");
    assert.equal(facts.statements[9].comment.comment, "abc");
    for (const statement of facts.statements) {
      assert.equal(
        Buffer.from(sql)
          .subarray(statement.span.start.offset, statement.span.end.offset)
          .toString(),
        statement.sql,
      );
    }
  },
);

test(
  "compiled metadata parser keeps malformed controls diagnostic and recovers neighbors",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = { sql: fixture("invalid-metadata.sql") };
    const facts = await cjs.parsePostgresSql(source);
    assert.deepEqual(await esm.parsePostgresSql(source), facts);
    assert.equal(facts.diagnostics.length, 22);
    assert.match(facts.diagnostics[0].message, /column name or constraint/);
    assert.deepEqual(
      facts.statements.map((statement) => statement.kind),
      ["createIndex"],
    );
    assert.equal(facts.statements[0].ordinal, 22);
  },
);
