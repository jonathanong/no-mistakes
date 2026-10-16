const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");
const withoutSpans = (value) => {
  if (Array.isArray(value)) return value.map(withoutSpans);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value)
        .filter(([key]) => key !== "span")
        .map(([key, value]) => [key, withoutSpans(value)]),
    );
  return value;
};

test(
  "compiled conditional WITH inserts preserve ordinary and wrapped source contracts",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("conditional-with-insert.sql");
    const result = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 4);
    const baseline = result.statements[0].insert;
    const block = result.statements[2].block;
    assert.equal(block.complete, false);
    const children = block.statements[0].branches[0].statements;
    assert.equal(children.length, 7);
    const bytes = Buffer.from(sql);
    for (let index = 0; index < 3; index++) {
      const child = index === 0 ? children[index] : children[index].wrapper.statements[0];
      assert.equal(child.kind, "insert");
      assert.deepEqual(withoutSpans(child.insert), withoutSpans(baseline));
      assert.equal(
        bytes.subarray(child.span.start.offset, child.span.end.offset).toString(),
        child.sql,
      );
      const span = child.insert.source.span;
      assert.equal(
        bytes.subarray(span.start.offset, span.end.offset).toString(),
        "SELECT id FROM src",
      );
    }
    for (const index of [3, 4]) {
      const wrapper = children[index].wrapper;
      assert.equal(wrapper.complete, false);
      assert.equal(wrapper.statements[0].kind, "insert");
      assert.equal(wrapper.statements[0].insert.source.kind, "values");
      assert.equal(wrapper.statements[0].insert.complete, false);
    }
    assert.deepEqual(withoutSpans(children[6].insert), withoutSpans(result.statements[1].insert));
    assert.equal(result.statements[3].sql, "SELECT 88;");
  },
);

test(
  "compiled conditional partial conflict targets remain explicit and keep neighbors",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("conditional-partial-conflict-controls.sql");
    const result = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 6);
    for (const index of [0, 2, 4]) {
      const block = result.statements[index].block;
      assert.equal(block.complete, false);
      assert.deepEqual(block.statements, []);
      assert.ok(block.diagnostics.length > 0);
      assert.equal(result.statements[index + 1].kind, "select");
    }
  },
);
