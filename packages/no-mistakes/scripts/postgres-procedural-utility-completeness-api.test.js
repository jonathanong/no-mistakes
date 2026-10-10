const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = readFileSync(
  join(__dirname, "../../../fixtures/postgres-facts/source/procedural-utility-completeness.sql"),
  "utf8",
);

test(
  "compiled native CJS and ESM keep classified procedural utilities complete",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const input = { sql: fixture, fileName: "procedural-utility-completeness.sql" };
    const pending = cjs.parsePostgresSql(input);
    assert.equal(typeof pending.then, "function");
    const facts = await pending;
    assert.deepEqual(await esm.parsePostgresSql(input), facts);
    assert.deepEqual(facts.diagnostics, []);

    const blocks = facts.statements.map((statement) => statement.block);
    assert.deepEqual(
      blocks.map((block) => block.complete),
      [
        true,
        true,
        true,
        true,
        true,
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
      ],
    );
    assert.deepEqual(
      blocks.slice(0, 6).map((block) => block.diagnostics),
      [[], [], [], [], [], []],
    );
    assert.equal(blocks[0].occurrences[0].kind, "controlFlow");
    assert.equal(blocks[0].occurrences[0].occurrences[0].kind, "utility");
    assert.equal(blocks[1].occurrences[0].kind, "utility");
    assert.equal(blocks[2].occurrences[0].kind, "utility");
    assert.equal(blocks[3].occurrences[0].kind, "utility");
    assert.equal(blocks[4].occurrences[0].occurrences[0].kind, "utility");
    const bytes = Buffer.from(fixture);
    const originalSlice = (span) => bytes.subarray(span.start.offset, span.end.offset).toString();
    assert.equal(
      originalSlice(blocks[0].occurrences[0].occurrences[0].span),
      "CREATE TYPE x AS ENUM ('a');",
    );
    assert.equal(
      originalSlice(blocks[1].occurrences[0].span),
      "LOCK TABLE t IN SHARE ROW EXCLUSIVE MODE;",
    );
    assert.equal(
      originalSlice(blocks[2].occurrences[0].span),
      "LOCK TABLE ONLY db.public.t, q IN SHARE UPDATE EXCLUSIVE MODE NOWAIT;",
    );
    assert.equal(blocks[5].occurrences[0].occurrences[0].kind, "dml");
    assert.equal(blocks[6].occurrences[0].kind, "dynamicExecute");
    assert.ok(blocks.slice(7).every((block) => block.occurrences[0].kind === "unknown"));
  },
);
