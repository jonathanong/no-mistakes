const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled wrapper boundaries retain conflict identifiers and SELECT INTO neighbors",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = readFileSync(
      join(
        __dirname,
        "../../../fixtures/postgres-facts/source/wrapper-insert-identifier-boundaries.sql",
      ),
      "utf8",
    );
    const result = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 8);
    const atomic = result.statements[0].function.wrapper;
    assert.equal(atomic.complete, true);
    for (const child of [
      atomic.statements[0],
      ...result.statements.slice(4, 6).map((parent) => parent.wrapper.statements[0]),
    ]) {
      assert.equal(child.insert.complete, true);
      assert.equal(child.insert.onConflict.predicate.sql, "id > 0");
      assert.equal(child.insert.source.query.columns[0].name.parts[0].identity, "begin");
      assert.equal(sql.slice(child.span.start.offset, child.span.end.offset), child.sql);
    }
    assert.equal(result.statements[1].sql, "SELECT 111;");
    assert.equal(result.statements[3].sql, "SELECT 112;");
    assert.equal(result.statements[2].function.wrapper.complete, false);
    assert.match(result.statements[2].function.wrapper.diagnostics[0].message, /SELECT INTO/);
    for (const parent of result.statements.slice(6)) {
      assert.equal(parent.wrapper.complete, false);
      assert.equal(parent.wrapper.statements[0].sql, "SELECT 1 case INTO tmp");
    }
  },
);

test(
  "compiled metadata preserves qualified END targets and malformed body neighbors",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = readFileSync(
      join(__dirname, "../../../fixtures/postgres-facts/source/metadata-qualified-end.sql"),
      "utf8",
    );
    const result = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 7);
    for (const parent of result.statements.slice(0, 3)) {
      assert.equal(parent.comment.name.parts.at(-1).identity, "end");
      assert.equal(sql.slice(parent.span.start.offset, parent.span.end.offset), parent.sql);
    }
    const declaration = result.statements[3].function.wrapper;
    assert.equal(declaration.complete, true);
    assert.equal(declaration.statements.length, 2);
    for (const child of declaration.statements) {
      assert.equal(child.comment.name.parts.at(-1).identity, "end");
      assert.equal(sql.slice(child.span.start.offset, child.span.end.offset), child.sql);
    }
    assert.equal(result.statements[4].block.complete, true);
    assert.equal(result.statements[5].function.wrapper.complete, false);
    assert.equal(result.statements[6].sql, "SELECT 99;");
  },
);
