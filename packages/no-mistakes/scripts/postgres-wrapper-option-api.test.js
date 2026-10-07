const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled wrapper options preserve PostgreSQL boolean and identifier grammar",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = readFileSync(
      join(__dirname, "../../../fixtures/postgres-facts/source/wrapper-boolean-identity.sql"),
      "utf8",
    );
    const result = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.deepEqual(
      result.statements.slice(0, 13).map((item) => item.wrapper.execution),
      [
        "executesForAnalysis",
        "unknown",
        "unknown",
        "nonExecuting",
        "unknown",
        "unknown",
        "executesForAnalysis",
        "nonExecuting",
        "executesForAnalysis",
        "unknown",
        "executesForAnalysis",
        "executesForAnalysis",
        "unknown",
      ],
    );
    assert.equal(result.statements[15].query.columns[0].name.parts[0].value, "explain");
    assert.equal(result.statements[16].query.columns[0].name.parts[0].value, "explain");
    assert.equal(result.statements[13].block.complete, true);
    assert.equal(result.statements[14].function.wrapper.complete, true);
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
