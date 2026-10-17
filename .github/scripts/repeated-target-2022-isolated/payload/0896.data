const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled EXPLAIN options validate all occurrences and use their final values",
  {
    skip: !compiled,
  },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = readFileSync(
      join(__dirname, "../../../fixtures/postgres-facts/source/wrapper-repeated-options.sql"),
      "utf8",
    );
    const result = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 28);
    const plan = "nonExecuting";
    const runs = "executesForAnalysis";
    const unknown = "unknown";
    const expected = [
      plan,
      runs,
      plan,
      runs,
      runs,
      unknown,
      plan,
      unknown,
      plan,
      unknown,
      plan,
      unknown,
      plan,
      runs,
      plan,
      plan,
      unknown,
      unknown,
      unknown,
      unknown,
      unknown,
      unknown,
      unknown,
      plan,
      plan,
    ];
    assert.deepEqual(
      result.statements.slice(0, 25).map((item) => item.wrapper.execution),
      expected,
    );
    for (const [index, execution] of expected.entries()) {
      const wrapper = result.statements[index].wrapper;
      assert.equal(wrapper.complete, execution !== unknown);
      assert.equal(wrapper.diagnostics.length === 0, execution !== unknown);
      assert.equal(wrapper.statements.length, 1);
      assert.equal(wrapper.statements[0].kind, "select");
    }
    const block = result.statements[25].block;
    assert.equal(block.complete, true);
    assert.equal(block.statements[0].branches[0].statements[0].wrapper.execution, plan);
    const atomic = result.statements[26].function.wrapper;
    assert.equal(atomic.complete, true);
    assert.equal(atomic.statements[0].wrapper.execution, runs);
    assert.equal(result.statements[27].sql, "SELECT 98;");
    const bytes = Buffer.from(sql);
    for (const parent of result.statements) {
      assert.equal(
        bytes.subarray(parent.span.start.offset, parent.span.end.offset).toString(),
        parent.sql,
      );
      for (const child of (parent.wrapper || parent.function?.wrapper)?.statements || []) {
        assert.equal(
          bytes.subarray(child.span.start.offset, child.span.end.offset).toString(),
          child.sql,
        );
      }
    }
  },
);
