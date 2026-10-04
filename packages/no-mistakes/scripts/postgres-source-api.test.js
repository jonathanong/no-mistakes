const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");

test(
  "compiled pure-source SQL API exposes async CJS and ESM facts without a project root",
  {
    skip: !compiled,
  },
  async () => {
    const api = require("../index.js");
    const source = { sql: fixture("schema.sql"), fileName: "/nonexistent/migration.sql" };
    const pending = api.parsePostgresSql(source);
    assert.equal(typeof pending.then, "function");
    const single = await pending;
    assert.equal(single.fileName, source.fileName);
    assert.equal(single.schemaVersion, 1);
    assert.equal(single.statements[0].kind, "createTable");
    assert.equal(single.statements[0].table.parts[1].identity, "Accounts");
    assert.deepEqual(single.diagnostics, []);
    const esm = await import("../index.mjs");
    const batch = await esm.parsePostgresSql([source, { sql: fixture("partial.sql") }]);
    assert.deepEqual(batch[0], single);
    assert.deepEqual(
      batch[1].statements.map((statement) => statement.ordinal),
      [0, 2],
    );
    assert.equal(batch[1].diagnostics.length, 1);
    await assert.rejects(api.parsePostgresSql({ fileName: "missing.sql" }));
  },
);
