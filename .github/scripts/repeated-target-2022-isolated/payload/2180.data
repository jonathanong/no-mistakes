const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");
function occurrences(statement) {
  const children =
    statement.kind === "doBlock"
      ? statement.block.statements
      : statement.kind === "conditional"
        ? statement.branches.flatMap((branch) => branch.statements)
        : [];
  return [statement, ...children.flatMap(occurrences)];
}

test(
  "compiled native CJS and ESM expose safely attributed nested constraint facts",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const name of [
      "nested-constraint-plain.sql",
      "nested-constraints.sql",
      "nested-constraints-single.sql",
    ]) {
      const sql = fixture(name);
      const input = { sql, fileName: name };
      const pending = cjs.parsePostgresSql(input);
      assert.equal(typeof pending.then, "function");
      const facts = await pending;
      assert.deepEqual(await esm.parsePostgresSql(input), facts);
      assert.deepEqual(facts.diagnostics, []);
      const plain = name === "nested-constraint-plain.sql";
      assert.deepEqual(
        facts.statements.map((statement) => statement.kind),
        plain ? ["doBlock"] : ["createIndex", "doBlock", "createIndex"],
      );
      const owner = facts.statements.find((statement) => statement.kind === "doBlock");
      assert.equal(owner.block.complete, true);
      assert.deepEqual(owner.block.diagnostics, []);
      const all = facts.statements.flatMap(occurrences);
      const constraints = all.filter((statement) => statement.kind === "alterTable");
      assert.equal(constraints.length, plain ? 1 : 3);
      const fk = constraints[0].operations[0];
      assert.equal(fk.kind, "addConstraint");
      assert.equal(fk.notValid, true);
      assert.equal(fk.constraint.kind, "foreignKey");
      assert.deepEqual(fk.constraint.name, {
        value: plain ? "sample_constraint" : "FK雪",
        identity: plain ? "sample_constraint" : "FK雪",
        quoted: !plain,
      });
      assert.deepEqual(
        constraints[0].table.parts.map((part) => part.value),
        plain ? ["sample_child"] : ["Mý", "Chïld"],
      );
      assert.deepEqual(
        fk.constraint.referencedTable.parts.map((part) => part.value),
        plain ? ["sample_parent"] : ["Mý", "Parent"],
      );
      assert.deepEqual(
        fk.constraint.columns.map((column) => column.value),
        [plain ? "parent_id" : "Parent Id"],
      );
      assert.deepEqual(
        fk.constraint.referencedColumns.map((column) => column.value),
        [plain ? "id" : "Id"],
      );
      for (const statement of all) {
        assert.equal(
          Buffer.from(sql)
            .subarray(statement.span.start.offset, statement.span.end.offset)
            .toString(),
          statement.sql,
        );
        if (statement.kind === "insert") assert.deepEqual(statement.insert.span, statement.span);
      }
      for (const statement of constraints) {
        assert.ok(statement.span.start.offset > owner.span.start.offset);
        assert.ok(statement.span.end.offset < owner.span.end.offset);
      }
      for (const diagnostic of owner.block.diagnostics) {
        assert.match(
          Buffer.from(sql)
            .subarray(diagnostic.span.start.offset, diagnostic.span.end.offset)
            .toString(),
          /^LOCK TABLE/,
        );
        assert.match(diagnostic.message, /Unsupported nested procedural statement/);
      }
    }
  },
);

test(
  "compiled nested unsupported SQL stays explicit beside supported constraints",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const input = { sql: fixture("nested-incomplete.sql") };
    const facts = await cjs.parsePostgresSql(input);
    assert.deepEqual(await esm.parsePostgresSql(input), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements[0].block.complete, false);
    assert.equal(facts.statements[0].block.diagnostics.length, 4);
    assert.ok(facts.statements[0].block.diagnostics.every((diagnostic) => diagnostic.span));
    assert.ok(
      occurrences(facts.statements[0]).some(
        (statement) =>
          statement.kind === "alterTable" &&
          statement.operations.some(
            (operation) =>
              operation.kind === "addConstraint" &&
              operation.constraint.name?.value === "supported",
          ),
      ),
    );
  },
);
