const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

const withoutSpans = (value) => {
  if (Array.isArray(value)) return value.map(withoutSpans);
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .filter(([key]) => key !== "span")
        .map(([key, item]) => [key, withoutSpans(item)]),
    );
  }
  return value;
};

test(
  "compiled CJS and ESM preserve COMMENT relation and alias facts across wrappers",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = readFileSync(
      join(__dirname, "../../../fixtures/postgres-facts/source/comment-join-alias-parity.sql"),
      "utf8",
    );
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 13);
    const bytes = Buffer.from(sql);
    for (let index = 0; index < 10; index += 2) {
      const baseline = facts.statements[index];
      const child = facts.statements[index + 1].wrapper.statements[0];
      assert.equal(baseline.kind, "select");
      assert.equal(child.kind, "select");
      assert.deepEqual(withoutSpans(child.query), withoutSpans(baseline.query));
      assert.equal(
        bytes.subarray(child.span.start.offset, child.span.end.offset).toString(),
        child.sql,
      );
      assert.match(child.sql, /comment ON/);
    }
    const baseline = withoutSpans(facts.statements[0].query);
    const branch = facts.statements[10].block.statements[0].branches[0].statements[0];
    const atomic = facts.statements[11].function.wrapper.statements[0];
    assert.deepEqual(withoutSpans(branch.query), baseline);
    assert.deepEqual(withoutSpans(atomic.query), baseline);
    assert.equal(facts.statements[12].sql, "SELECT 99;");
  },
);
