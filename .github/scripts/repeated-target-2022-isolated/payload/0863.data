const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

function shape(occurrences) {
  return occurrences.map((occurrence) => {
    const nested = shape(occurrence.occurrences);
    return nested.length === 0 ? occurrence.kind : `${occurrence.kind}(${nested.join(",")})`;
  });
}

const CASES = [
  {
    name: "utility",
    sql: "DO $$ BEGIN CREATE TYPE x AS ENUM ('a'); END $$;",
    complete: true,
    shape: ["utility"],
  },
  {
    name: "raise between creates",
    sql: "DO $$ BEGIN CREATE TABLE a(id int); RAISE NOTICE 'x'; CREATE TABLE b(id int); END $$;",
    complete: true,
    shape: ["utility", "controlFlow", "utility"],
    statements: 2,
  },
  {
    name: "control flow",
    sql: "DO $$ BEGIN IF (SELECT COUNT(*) FROM t) > 0 THEN RAISE EXCEPTION 'bad'; END IF; END $$;",
    complete: true,
    shape: ["controlFlow(controlFlow)"],
    statements: 0,
  },
  {
    name: "create beside unknown loop",
    sql: "DO $$ BEGIN CREATE TABLE a(id int); LOOP SELECT 1; END LOOP; END $$;",
    complete: false,
    shape: ["utility", "controlFlow(unknown)"],
    statements: 1,
    diagnostic: /Unsupported procedural occurrence/,
  },
  {
    name: "create beside dynamic loop",
    sql: "DO $$ BEGIN CREATE TABLE a(id int); LOOP EXECUTE format('SELECT 1'); END LOOP; END $$;",
    complete: false,
    shape: ["utility", "controlFlow(dynamicExecute)"],
    statements: 1,
    diagnostic: /Dynamic EXECUTE is unknown/,
  },
  {
    name: "create beside loop dml",
    sql: "DO $$ BEGIN CREATE TABLE a(id int); FOR i IN 1..2 LOOP INSERT INTO a VALUES (i); END LOOP; END $$;",
    complete: false,
    shape: ["utility", "controlFlow(dml)"],
    statements: 1,
    diagnostic: /not an executed statement/,
  },
  {
    name: "loop dml",
    sql: "DO $$ BEGIN FOR i IN 1..2 LOOP INSERT INTO t(id) VALUES (i); END LOOP; END $$;",
    complete: false,
    shape: ["controlFlow(dml)"],
    statements: 0,
    diagnostic: /not an executed statement/,
  },
  {
    name: "dynamic execute",
    sql: "DO $$ BEGIN EXECUTE format('INSERT INTO t(id) VALUES (%s)', 1); END $$;",
    complete: false,
    shape: ["dynamicExecute"],
  },
  {
    name: "labeled loop",
    sql: "DO $$ BEGIN <<retry>> LOOP NULL; END LOOP retry; END $$;",
    complete: false,
    shape: ["controlFlow(unknown)"],
    statements: 0,
    diagnostic: /Unsupported procedural occurrence/,
  },
];

test(
  "compiled native CJS and ESM distinguish procedural occurrence kinds",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const entry of CASES) {
      const input = { sql: entry.sql, fileName: `${entry.name}.sql` };
      const pending = cjs.parsePostgresSql(input);
      assert.equal(typeof pending.then, "function");
      const facts = await pending;
      assert.deepEqual(await esm.parsePostgresSql(input), facts);
      assert.deepEqual(facts.diagnostics, []);
      const block = facts.statements[0].block;
      assert.equal(facts.statements[0].kind, "doBlock");
      assert.equal(block.complete, entry.complete, entry.name);
      assert.deepEqual(shape(block.occurrences), entry.shape, entry.name);
      if (entry.statements !== undefined) {
        assert.equal(block.statements.length, entry.statements, entry.name);
      }
      if (entry.diagnostic) {
        assert.match(block.diagnostics.map((item) => item.message).join("\n"), entry.diagnostic);
      }
      for (const occurrence of block.occurrences) {
        const text = entry.sql.slice(occurrence.span.start.offset, occurrence.span.end.offset);
        assert.ok(text.length > 0, entry.name);
        assert.ok(occurrence.span.start.offset < occurrence.span.end.offset);
      }
    }
  },
);
