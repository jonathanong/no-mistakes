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

test(
  "compiled source API exposes procedural origin and original generated storage",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const facts = await api.parsePostgresSql({ sql: fixture("unsupported.sql") });
    assert.deepEqual(facts.diagnostics, []);
    const block = facts.statements[0].block;
    assert.equal(facts.statements[0].kind, "doBlock");
    assert.equal(block.language, "plpgsql");
    assert.equal(block.complete, true);
    assert.deepEqual(
      block.statements.map((statement) => statement.kind),
      ["alterTable", "alterTable"],
    );
    assert.equal(block.statements[0].span.start.line, 3);
    const virtual = await api.parsePostgresSql({ sql: fixture("schema-virtual.sql") });
    assert.deepEqual(virtual.diagnostics, []);
    assert.equal(virtual.statements[0].columns[1].generated.storage, "VIRTUAL");
    const empty = await api.parsePostgresSql({ sql: fixture("generated-empty.sql") });
    assert.deepEqual(empty.diagnostics, []);
    assert.deepEqual(
      empty.statements[0].columns.map((column) => column.generated.storage),
      ["VIRTUAL", "STORED", "VIRTUAL"],
    );
  },
);

test(
  "compiled source API preserves FETCH expressions and procedural source ownership",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const sql = fixture("fetch-no-delimiter.sql");
    const facts = await api.parsePostgresSql({ sql });
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements[0].sql, sql.slice(sql.indexOf("SELECT")));
    assert.equal(facts.statements[0].span.end.offset, Buffer.byteLength(sql));
    const nested = await api.parsePostgresSql({ sql: fixture("fetch-procedural.sql") });
    assert.deepEqual(nested.diagnostics, []);
    assert.equal(nested.statements[0].kind, "doBlock");
    assert.equal(nested.statements[0].block.complete, true);
    assert.deepEqual(nested.statements[0].block.diagnostics, []);
    assert.match(
      nested.statements[0].block.statements[0].view.query,
      /FETCH FIRST \(COALESCE\(NULL, 100\)\) ROWS ONLY/,
    );
  },
);
