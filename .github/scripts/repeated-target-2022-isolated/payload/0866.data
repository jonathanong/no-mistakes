const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) => ({
  sql: readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8"),
  fileName: name,
});
const parts = (name) => name.parts.map(({ value, quoted }) => [value, quoted]);

test(
  "compiled CJS and ESM retain structured replay-safety syntax without interpreting values",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const inputs = [
      "replay-structured-triggers.sql",
      "replay-structured-execute.sql",
      "replay-structured-casts.sql",
      "replay-structured-execute-unsupported.sql",
    ].map(fixture);
    const pending = cjs.parsePostgresSql(inputs);
    assert.ok(pending instanceof Promise);
    const reports = await pending;
    assert.deepEqual(await esm.parsePostgresSql(inputs), reports);
    for (const [index, input] of inputs.entries()) {
      assert.deepEqual(await cjs.parsePostgresSql(input), reports[index]);
      assert.deepEqual(await esm.parsePostgresSql(input), reports[index]);
    }

    const [triggers, commands, casts, unsupported] = reports;
    assert.deepEqual(triggers.diagnostics, []);
    const [restricted, unrestricted, multiple] = triggers.statements.map(({ trigger }) => trigger);
    assert.deepEqual(restricted.events, ['UPDATE OF "Mixed", a']);
    assert.deepEqual(
      restricted.eventFacts.map(({ kind, updateOf, updateColumns }) => ({
        kind,
        updateOf,
        columns: updateColumns.map(({ value, quoted }) => [value, quoted]),
      })),
      [
        {
          kind: "update",
          updateOf: true,
          columns: [
            ["Mixed", true],
            ["a", false],
          ],
        },
      ],
    );
    assert.deepEqual(unrestricted.eventFacts, [
      { kind: "update", updateOf: false, updateColumns: [] },
    ]);
    assert.deepEqual(
      multiple.eventFacts,
      ["insert", "delete", "truncate"].map((kind) => ({
        kind,
        updateOf: false,
        updateColumns: [],
      })),
    );

    assert.deepEqual(commands.diagnostics, []);
    const block = commands.statements[0].block;
    assert.equal(block.complete, true);
    assert.deepEqual(block.diagnostics, []);
    const [concatenated, parameterized] = block.statements;
    for (const statement of [concatenated, parameterized]) {
      assert.equal(statement.kind, "literalExecute");
      assert.equal(statement.execute.complete, true);
      assert.deepEqual(statement.execute.diagnostics, []);
      const insert = statement.execute.statements[0];
      assert.equal(insert.kind, "insert");
      assert.equal(insert.insert.onConflict.action.kind, "doNothing");
      const span = insert.span;
      assert.equal(
        Buffer.from(statement.execute.decodedSql)
          .subarray(span.start.offset, span.end.offset)
          .toString(),
        insert.sql,
      );
    }
    assert.equal(concatenated.execute.bodyEncoding, "concatenated");
    const commandSlice = ({ literalSpan }) =>
      Buffer.from(inputs[1].sql)
        .subarray(literalSpan.start.offset, literalSpan.end.offset)
        .toString();
    assert.equal(
      commandSlice(concatenated.execute),
      "'INSERT INTO t ' || 'VALUES(1) ON CONFLICT DO NOTHING'",
    );
    assert.equal(
      commandSlice(parameterized.execute),
      "'INSERT INTO t VALUES($1) ON CONFLICT DO NOTHING'",
    );
    assert.equal(concatenated.execute.decodedSql, "INSERT INTO t VALUES(1) ON CONFLICT DO NOTHING");
    assert.deepEqual(concatenated.execute.using, []);
    assert.equal(parameterized.execute.bodyEncoding, "singleQuoted");
    assert.equal(parameterized.execute.using.length, 1);
    assert.equal(parameterized.execute.using[0].sql, "1");
    assert.deepEqual(parameterized.execute.using[0].root.value, { kind: "number", value: "1" });
    const parameter = parameterized.execute.statements[0].insert.source.rows[0][0];
    assert.deepEqual(parameter.root, { kind: "parameter", placeholder: "$1" });
    const usingSpan = parameterized.execute.using[0].span;
    assert.equal(
      Buffer.from(inputs[1].sql).subarray(usingSpan.start.offset, usingSpan.end.offset).toString(),
      "1",
    );

    assert.deepEqual(casts.diagnostics, []);
    const action = casts.statements[0].insert.onConflict.action;
    const assignments = Object.fromEntries(
      action.assignments.map(({ columns, expression }) => [
        columns[0].parts[0].identity,
        expression,
      ]),
    );
    for (const name of ["quoted_date", "qualified_date", "nested_date"]) {
      assert.equal(assignments[name].root.kind, "cast");
      assert.equal(assignments[name].childrenComplete, true);
    }
    assert.deepEqual(parts(assignments.quoted_date.root.dataTypeFacts.name), [["date", true]]);
    assert.deepEqual(parts(assignments.qualified_date.root.dataTypeFacts.name), [
      ["schema.with.dot", true],
      ["date", false],
    ]);
    const nestedCast = assignments.nested_date.children[0].children[0];
    assert.equal(nestedCast.root.kind, "cast");
    assert.deepEqual(parts(nestedCast.root.dataTypeFacts.name), [
      ["schema.with.dot", true],
      ["date", false],
    ]);
    for (const [name, operator] of [
      ["and_predicate", "AND"],
      ["or_predicate", "OR"],
    ]) {
      const expression = assignments[name];
      assert.deepEqual(expression.root, { kind: "binary", operator });
      assert.equal(expression.childrenComplete, true);
      assert.deepEqual(
        expression.children.map(({ role, root, childrenComplete }) => [
          role,
          root.kind,
          root.negated,
          childrenComplete,
        ]),
        [
          ["binaryLeft", "nullTest", false, true],
          ["binaryRight", "nullTest", true, true],
        ],
      );
      // Missing token boundaries must not erase fully represented syntax.
      assert.deepEqual(
        expression.children.map(({ span }) => span),
        [null, null],
      );
    }
    assert.equal(assignments.cast_predicate.childrenComplete, true);
    assert.equal(action.predicate.childrenComplete, true);
    assert.equal(assignments.opaque_expression.childrenComplete, false);

    const opaque = unsupported.statements[0].block;
    assert.equal(opaque.complete, false);
    assert.ok(opaque.diagnostics.length > 0);
    assert.equal(opaque.statements.filter(({ kind }) => kind === "literalExecute").length, 0);
  },
);
