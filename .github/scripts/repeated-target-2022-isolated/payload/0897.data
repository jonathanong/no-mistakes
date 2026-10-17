const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled CJS/ESM wrapped WITH INSERT retains partial conflict facts and boundaries",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = readFileSync(
      join(__dirname, "../../../fixtures/postgres-facts/source/wrapper-with-partial-conflict.sql"),
      "utf8",
    );
    const pending = cjs.parsePostgresSql({ sql });
    assert.equal(typeof pending.then, "function");
    const result = await pending;
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 9);
    const baseline = result.statements[0].insert;
    const parents = [
      result.statements[1].wrapper,
      result.statements[2].wrapper,
      result.statements[3].function.wrapper,
    ];
    for (const wrapper of parents) {
      assert.equal(wrapper.complete, true);
      const child = wrapper.statements[0];
      assert.equal(child.kind, "insert");
      assert.ok(child.sql.startsWith("WITH src"));
      assert.equal(
        Buffer.from(sql).subarray(child.span.start.offset, child.span.end.offset).toString(),
        child.sql,
      );
      assert.equal(child.insert.complete, true);
      assert.deepEqual(child.insert.onConflict.target, baseline.onConflict.target);
      assert.deepEqual(child.insert.onConflict.action, baseline.onConflict.action);
      assert.equal(child.insert.onConflict.predicate.sql, "id > 0");
      const source = child.insert.source;
      assert.equal(source.kind, "select");
      assert.equal(source.query.ctes.length, baseline.source.query.ctes.length);
      assert.equal(source.query.ctes[0].name.identity, baseline.source.query.ctes[0].name.identity);
      assert.equal(source.query.ctes[0].referenced, true);
      assert.equal(source.query.ctes[0].used, true);
      assert.equal(
        Buffer.from(sql).subarray(source.span.start.offset, source.span.end.offset).toString(),
        "SELECT id FROM src",
      );
    }
    assert.equal(result.statements[4].wrapper.complete, false);
    assert.equal(result.statements[4].wrapper.statements[0].insert.complete, false);
    for (const index of [5, 7]) {
      assert.equal(result.statements[index].wrapper.complete, false);
      assert.ok(result.statements[index].wrapper.diagnostics.length);
    }
    assert.equal(result.statements[6].sql, "SELECT 71;");
    assert.equal(result.statements[8].sql, "SELECT 72;");
  },
);
