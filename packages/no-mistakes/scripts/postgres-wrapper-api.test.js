const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");

test(
  "compiled CJS and ESM wrappers expose execution and declaration provenance",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("wrappers.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 15);
    assert.deepEqual(
      facts.statements.slice(0, 10).map((item) => item.wrapper.execution),
      [
        "nonExecuting",
        "executesForAnalysis",
        "nonExecuting",
        "executesForAnalysis",
        "executesForAnalysis",
        "nonExecuting",
        "executesForAnalysis",
        "nonExecuting",
        "executesForAnalysis",
        "nonExecuting",
      ],
    );
    const bytes = Buffer.from(sql);
    for (const statement of facts.statements) {
      assert.equal(
        bytes.subarray(statement.span.start.offset, statement.span.end.offset).toString(),
        statement.sql,
      );
      const wrapper = statement.wrapper || statement.function?.wrapper;
      if (!wrapper) continue;
      assert.deepEqual(wrapper.span, statement.span);
      for (const child of wrapper.statements) {
        assert.equal(
          bytes.subarray(child.span.start.offset, child.span.end.offset).toString(),
          child.sql,
        );
      }
    }
    assert.equal(facts.statements[10].wrapper.wrapperKind, "prepare");
    assert.equal(facts.statements[10].wrapper.execution, "nonExecuting");
    const declaration = facts.statements[11].function.wrapper;
    assert.equal(declaration.wrapperKind, "functionDeclaration");
    assert.equal(declaration.execution, "nonExecuting");
    assert.equal(declaration.complete, true);
    assert.deepEqual(
      declaration.statements.map((item) => item.ordinal),
      [0, 1],
    );
    assert.deepEqual(
      declaration.statements.map((item) => item.kind),
      ["insert", "select"],
    );
    assert.equal(facts.statements[13].function.wrapper.complete, false);
    assert.deepEqual(facts.statements[13].function.wrapper.statements, []);
    assert.equal(facts.statements.filter((item) => item.kind === "insert").length, 0);
    for (const name of [
      "wrapper-options.sql",
      "wrapper-functions.sql",
      "wrapper-recovery.sql",
      "wrapper-compatibility.sql",
      "wrapper-boundaries.sql",
    ]) {
      const result = await cjs.parsePostgresSql({ sql: fixture(name) });
      assert.deepEqual(await esm.parsePostgresSql({ sql: fixture(name) }), result);
      assert.deepEqual(result.diagnostics, []);
      assert.ok(result.statements.every((item) => item.kind !== "other"));
    }
  },
);
