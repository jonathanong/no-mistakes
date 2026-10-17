const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled CJS and ESM recover booleans without changing bind or opaque proofs",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const fileName = "boolean-stringification.ts";
    const source = readFileSync(
      join(__dirname, "../../../fixtures/postgres-facts/executed-source", fileName),
      "utf8",
    );
    const options = {
      sources: [{ fileName, source }],
      importSpecifier: "@example/db",
      trustedSqlTags: [{ module: "@example/db", name: "sql" }],
      resolveImports: true,
    };
    const pending = cjs.extractExecutedPostgresSql(options);
    assert.equal(typeof pending.then, "function");
    const rows = await pending;
    assert.deepEqual(await esm.extractExecutedPostgresSql(options), rows);
    const whole = rows.filter((row) => row.line <= 9);
    assert.equal(whole.length, 9);
    assert.ok(whole.every((row) => row.complete && !row.onlyValuePlaceholders));
    assert.equal(rows.find((row) => row.line === 2).sqlText, "/* booleans/true */ SELECT true");
    assert.equal(rows.find((row) => row.line === 3).sqlText, "/* booleans/false */ SELECT false");
    for (const line of [13, 16]) {
      const row = rows.find((row) => row.line === line);
      assert.ok(!row.complete && row.onlyValuePlaceholders);
      assert.ok(row.sqlText.includes("sql_placeholder_1"));
    }
    for (const line of [17, 19, 21]) {
      const row = rows.find((row) => row.line === line);
      assert.ok(!row.complete && !row.onlyValuePlaceholders);
    }
    const legacy = await cjs.extractExecutedPostgresSql({ ...options, resolveImports: false });
    assert.ok(legacy.filter((row) => row.line <= 9).every((row) => !row.complete));
  },
);
