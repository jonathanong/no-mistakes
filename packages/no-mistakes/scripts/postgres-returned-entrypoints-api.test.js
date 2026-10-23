const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const source = (name) => ({
  fileName: name,
  source: readFileSync(
    join(__dirname, "../../../fixtures/postgres-facts/executed-source", name),
    "utf8",
  ),
});

test(
  "compiled CJS and ESM recover private entrypoints returned from branch arms",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const [name, expected] of [
      ["control-flow-branch-returned-functions.ts", 2],
      ["control-flow-returned-entrypoint-wrappers.ts", 3],
    ]) {
      const options = {
        sources: [source(name)],
        importSpecifier: "@example/db",
        trustedSqlTags: [{ module: "@example/db", name: "sql" }],
        resolveImports: true,
      };
      const pending = cjs.extractExecutedPostgresSql(options);
      assert.ok(pending instanceof Promise);
      const rows = await pending;
      assert.deepEqual(await esm.extractExecutedPostgresSql(options), rows);
      assert.equal(rows.length, expected);
      for (const row of rows) {
        assert.equal(row.complete, true, JSON.stringify(row));
        assert.equal(row.onlyValuePlaceholders, false);
        assert.equal(row.incompleteReason, null);
        assert.ok(row.sqlText.includes("; DELETE FROM records"), JSON.stringify(row));
      }
    }
    const selectedOptions = {
      sources: [source("control-flow-returned-entrypoint-static-branches.ts")],
      importSpecifier: "@example/db",
      trustedSqlTags: [{ module: "@example/db", name: "sql" }],
      resolveImports: true,
    };
    const selectedRows = await cjs.extractExecutedPostgresSql(selectedOptions);
    assert.deepEqual(await esm.extractExecutedPostgresSql(selectedOptions), selectedRows);
    assert.equal(selectedRows.length, 2);
    assert.ok(selectedRows.every((row) => row.complete));
    assert.deepEqual(
      selectedRows.find((row) => row.line === 2),
      {
        fileName: "control-flow-returned-entrypoint-static-branches.ts",
        line: 2,
        annotation: "returns/contextual",
        sqlText: "/* returns/contextual */ SELECT 8",
        complete: true,
        onlyValuePlaceholders: false,
        kind: "composed",
        incompleteReason: null,
      },
    );
    assert.ok(selectedRows.find((row) => row.line === 9).sqlText.includes("DELETE FROM records"));
    const options = {
      sources: [source("control-flow-returned-entrypoint-unsupported.ts")],
      importSpecifier: "@example/db",
      trustedSqlTags: [{ module: "@example/db", name: "sql" }],
      resolveImports: true,
    };
    const rows = await cjs.extractExecutedPostgresSql(options);
    assert.deepEqual(await esm.extractExecutedPostgresSql(options), rows);
    // The loop retains zero/one previews, both with an unexplored residual.
    assert.deepEqual(
      rows.map((row) => [row.line, row.sqlText, row.incompleteReason]),
      [
        [5, null, "untrusted-array-intrinsics"],
        [10, "/* returns/unsupported */ SELECT 7", "sql-variants-truncated"],
        [10, "/* returns/unsupported */ SELECT 7; DELETE FROM records", "sql-variants-truncated"],
      ],
    );
    for (const row of rows) {
      assert.equal(row.complete, false, JSON.stringify(row));
      assert.equal(row.onlyValuePlaceholders, false);
      assert.ok(row.incompleteReason);
    }
  },
);
