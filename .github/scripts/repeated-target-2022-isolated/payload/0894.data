const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled atomic bodies retain bare END aliases and real CASE closers",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = readFileSync(
      join(__dirname, "../../../fixtures/postgres-facts/source/wrapper-bare-end-alias.sql"),
      "utf8",
    );
    const result = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 12);
    const bytes = Buffer.from(sql);
    for (const [index, following] of [
      [0, 2],
      [2, 3],
      [4, 4],
      [6, 0],
      [8, 5],
      [10, -1],
    ]) {
      const parent = result.statements[index];
      const fn = parent.function;
      assert.equal(parent.sql.endsWith("END;"), true);
      assert.equal(fn.bodySql.startsWith("BEGIN ATOMIC"), true);
      assert.equal(fn.bodySql.endsWith("END"), true);
      assert.equal(
        bytes.subarray(fn.wrapper.span.start.offset, fn.wrapper.span.end.offset).toString(),
        parent.sql,
      );
      assert.deepEqual(fn.wrapper.span, parent.span);
      assert.equal(fn.bodySql, parent.sql.slice(parent.sql.indexOf("BEGIN ATOMIC"), -1));
      if (following > 0) {
        assert.equal(fn.wrapper.statements.at(-1).sql, `SELECT ${following};`);
      } else if (following === 0) {
        assert.equal(fn.wrapper.complete, true);
        assert.equal(fn.wrapper.statements.length, 6);
      } else {
        assert.equal(fn.wrapper.complete, false);
        assert.equal(fn.wrapper.diagnostics.length > 0, true);
      }
      for (const child of fn.wrapper.statements) {
        assert.equal(
          bytes.subarray(child.span.start.offset, child.span.end.offset).toString(),
          child.sql,
        );
      }
    }
    const first = result.statements[0].function;
    assert.equal(first.wrapper.complete, false);
    assert.equal(first.wrapper.diagnostics.length > 0, true);
    assert.equal(first.bodySql.includes("end FROM source;"), true);
    for (const [index, query] of [
      [1, 101],
      [3, 102],
      [5, 103],
      [7, 104],
      [9, 105],
      [11, 106],
    ]) {
      assert.equal(result.statements[index].sql, `SELECT ${query};`);
    }
    for (const parent of result.statements) {
      assert.equal(
        bytes.subarray(parent.span.start.offset, parent.span.end.offset).toString(),
        parent.sql,
      );
    }
  },
);
