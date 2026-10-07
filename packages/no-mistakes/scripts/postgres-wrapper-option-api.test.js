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

test(
  "compiled atomic bodies distinguish BEGIN aliases from nested declarations",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = readFileSync(
      join(__dirname, "../../../fixtures/postgres-facts/source/wrapper-atomic-identifiers.sql"),
      "utf8",
    );
    const result = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 8);
    assert.deepEqual(
      [1, 3, 5].map((index) => result.statements[index].sql),
      ["SELECT 93;", "SELECT 94;", "SELECT 95;"],
    );
    assert.equal(result.statements[0].function.wrapper.complete, false);
    assert.equal(result.statements[0].function.wrapper.statements.length, 14);
    assert.deepEqual(
      result.statements[0].function.wrapper.statements.map((child) => child.query.complete),
      Array.from({ length: 14 }, (_, index) => index !== 11),
    );
    assert.equal(result.statements[2].function.wrapper.complete, true);
    assert.equal(result.statements[4].function.wrapper.complete, false);
    assert.equal(result.statements[6].function.wrapper.complete, false);
    assert.equal(result.statements[7].sql, "SELECT 96;");
  },
);

test(
  "compiled wrappers reuse decoded literals and bound transaction BEGIN",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const [fixture, expected] of [
      [
        "wrapper-string-options.sql",
        [
          ...Array(3).fill("nonExecuting"),
          ...Array(6).fill("executesForAnalysis"),
          ...Array(9).fill("unknown"),
          "nonExecuting",
        ],
      ],
      [
        "wrapper-signed-options.sql",
        ["executesForAnalysis", "nonExecuting", "nonExecuting", ...Array(6).fill("unknown")],
      ],
    ]) {
      const sql = readFileSync(
        join(__dirname, "../../../fixtures/postgres-facts/source", fixture),
        "utf8",
      );
      const result = await cjs.parsePostgresSql({ sql });
      assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
      assert.deepEqual(result.diagnostics, []);
      assert.deepEqual(
        result.statements.slice(0, expected.length).map((item) => item.wrapper.execution),
        expected,
      );
      for (const parent of result.statements) {
        assert.equal(sql.slice(parent.span.start.offset, parent.span.end.offset), parent.sql);
        for (const child of (parent.wrapper || parent.function?.wrapper)?.statements || []) {
          assert.equal(sql.slice(child.span.start.offset, child.span.end.offset), child.sql);
        }
      }
      if (fixture === "wrapper-signed-options.sql") {
        assert.equal(result.statements.length, 22);
        assert.equal(result.statements[9].function.wrapper.complete, false);
        assert.equal(result.statements[10].sql, "SELECT 81;");
        assert.equal(result.statements[11].function.wrapper.complete, true);
        assert.equal(result.statements[12].sql, "SELECT 82;");
        assert.deepEqual(
          result.statements.slice(13).map((item) => item.wrapper.execution),
          [
            "executesForAnalysis",
            "nonExecuting",
            "executesForAnalysis",
            "nonExecuting",
            "unknown",
            "unknown",
            "unknown",
            "executesForAnalysis",
            "nonExecuting",
          ],
        );
      }
    }
  },
);
