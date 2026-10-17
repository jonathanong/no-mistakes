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
      "wrapper-option-identity.sql",
      "wrapper-boolean-identity.sql",
      "wrapper-metadata.sql",
      "wrapper-comment-eof.sql",
      "wrapper-routine-comment-eof.sql",
      "wrapper-explain-comment-eof.sql",
      "wrapper-prepare-comment-eof.sql",
      "wrapper-functions.sql",
      "wrapper-recovery.sql",
      "wrapper-compatibility.sql",
      "wrapper-boundaries.sql",
      "wrapper-review.sql",
      "wrapper-review-boundaries.sql",
      "wrapper-cte-parity.sql",
    ]) {
      const result = await cjs.parsePostgresSql({ sql: fixture(name) });
      assert.deepEqual(await esm.parsePostgresSql({ sql: fixture(name) }), result);
      assert.deepEqual(result.diagnostics, []);
      assert.ok(result.statements.every((item) => item.kind !== "other"));
      if (name.endsWith("comment-eof.sql")) {
        assert.equal(result.statements[0].sql, fixture(name).trimEnd());
        const child = result.statements[0].wrapper?.statements[0];
        if (child) assert.equal(child.sql.endsWith("at EOF'"), true);
      }
      if (name === "wrapper-metadata.sql") {
        assert.equal(result.statements[0].function.wrapper.complete, true);
        assert.deepEqual(
          result.statements[0].function.wrapper.statements
            .slice(0, 3)
            .map((child) => child.comment.comment),
          ["café", "first second", null],
        );
        assert.equal(result.statements[1].function.wrapper.complete, false);
        assert.equal(result.statements[8].sql, "SELECT 63;");
        assert.deepEqual(
          result.statements.slice(3, 6).map((item) => item.wrapper.execution),
          ["unknown", "nonExecuting", "unknown"],
        );
        const sourceBytes = Buffer.from(fixture(name));
        for (const parent of result.statements) {
          for (const child of (parent.wrapper || parent.function?.wrapper)?.statements || []) {
            assert.equal(
              sourceBytes.subarray(child.span.start.offset, child.span.end.offset).toString(),
              child.sql,
            );
          }
        }
      }
      if (name === "wrapper-option-identity.sql") {
        assert.deepEqual(
          result.statements.slice(0, 17).map((item) => item.wrapper.execution),
          [
            "executesForAnalysis",
            "nonExecuting",
            "executesForAnalysis",
            "nonExecuting",
            "nonExecuting",
            "nonExecuting",
            "nonExecuting",
            "unknown",
            "executesForAnalysis",
            "executesForAnalysis",
            "unknown",
            "nonExecuting",
            "unknown",
            "unknown",
            "unknown",
            "unknown",
            "unknown",
          ],
        );
        const children = result.statements[17].block.statements[0].branches[0].statements;
        assert.equal(children[0].wrapper.statements[0].columns[1].generated.storage, "VIRTUAL");
        assert.equal(
          children[1].wrapper.statements[0].operations[0].column.generated.storage,
          "VIRTUAL",
        );
      }
      if (name === "wrapper-review-boundaries.sql") {
        assert.equal(result.statements[0].function.wrapper.complete, true);
        assert.equal(result.statements[1].function.wrapper.complete, false);
        assert.equal(result.statements[2].function.bodySql, "AS 'SELECT 1'");
        assert.equal(result.statements[11].function.bodySql, "AS 'SELECT 1'");
        assert.deepEqual(
          result.statements.slice(5, 9).map((item) => item.wrapper.execution),
          ["nonExecuting", "nonExecuting", "unknown", "unknown"],
        );
        const block = result.statements[9].block;
        assert.equal(block.complete, true);
        const branches = block.statements[0].branches;
        assert.equal(branches[0].statements[0].wrapper.execution, "nonExecuting");
        assert.equal(
          branches[0].statements[1].branches[0].statements[0].wrapper.wrapperKind,
          "prepare",
        );
        assert.equal(branches[1].statements[0].wrapper.execution, "executesForAnalysis");
        assert.equal(result.statements[10].block.complete, false);
      }
      if (name === "wrapper-review.sql") {
        assert.equal(result.statements[0].function.wrapper.complete, true);
        const outer = result.statements[1].function.wrapper;
        assert.equal(outer.statements.length, 2);
        assert.equal(outer.statements[0].function.wrapper.statements.length, 2);
        const sourceBytes = Buffer.from(fixture(name));
        const visit = (statement) => {
          assert.equal(
            sourceBytes.subarray(statement.span.start.offset, statement.span.end.offset).toString(),
            statement.sql,
          );
          const owned = statement.wrapper || statement.function?.wrapper;
          if (owned) for (const child of owned.statements) visit(child);
        };
        result.statements.forEach(visit);
        assert.equal(result.statements[18].function.wrapper.statements[0].ordinal, 1);
        assert.equal(result.statements[19].sql, "SELECT 60;");
      }
    }
  },
);
