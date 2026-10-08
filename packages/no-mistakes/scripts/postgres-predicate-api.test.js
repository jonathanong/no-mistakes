const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");

test(
  "compiled CJS and ESM expose recursive PostgreSQL predicates, parameters, and typed temporal literals",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("expression-predicates-and-temporal-values.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);

    const assignments = facts.statements[0].insert.onConflict.action.assignments;
    const byTarget = Object.fromEntries(
      assignments.map((assignment) => [assignment.columns[0].parts.at(-1).identity, assignment]),
    );
    const root = (name) => byTarget[name].expression.root;
    const children = (name) => byTarget[name].expression.children;
    const slice = (span) =>
      span && Buffer.from(sql).subarray(span.start.offset, span.end.offset).toString();

    assert.deepEqual(root("null_check"), { kind: "nullTest", negated: false });
    assert.deepEqual(root("not_null_check"), { kind: "nullTest", negated: true });
    assert.deepEqual(root("distinct_check"), { kind: "distinctness", negated: false });
    assert.deepEqual(root("not_distinct_check"), { kind: "distinctness", negated: true });
    for (const name of ["null_check", "not_null_check"]) {
      assert.equal(children(name).length, 1);
      assert.equal(children(name)[0].role, "nullOperand");
      assert.equal(children(name)[0].root.kind, "columnReference");
      assert.deepEqual(
        children(name)[0].root.name.parts.map((part) => [part.identity, part.quoted]),
        [
          [name === "null_check" ? "target" : "excluded", false],
          ["nullable", true],
        ],
      );
      assert.ok(slice(children(name)[0].span));
      assert.equal(children(name)[0].childrenComplete, true);
    }
    assert.deepEqual(
      children("distinct_check").map((child) => [child.role, child.root.kind]),
      [
        ["distinctLeft", "columnReference"],
        ["distinctRight", "columnReference"],
      ],
    );
    assert.deepEqual(
      children("distinct_check").map((child) => slice(child.span)),
      ['target."nullable"', 'EXCLUDED."nullable"'],
    );
    assert.deepEqual(
      children("not_distinct_check").map((child) => child.role),
      ["distinctLeft", "distinctRight"],
    );

    const and = byTarget.and_check.expression;
    assert.equal(and.root.kind, "binary");
    assert.deepEqual(
      and.children.map((child) => child.root.kind),
      ["nullTest", "nullTest"],
    );
    assert.deepEqual(
      and.children.map((child) => child.root.negated),
      [false, true],
    );
    assert.deepEqual(
      and.children.map((child) => child.span),
      [null, null],
    );
    assert.equal(and.childrenComplete, false);
    assert.ok(slice(and.children[0].children[0].span));
    const or = byTarget.or_check.expression;
    assert.equal(or.root.kind, "parenthesized");
    assert.equal(or.children[0].root.kind, "binary");
    assert.deepEqual(
      or.children[0].children.map((child) => child.root.kind),
      ["nullTest", "nullTest"],
    );
    const not = byTarget.not_check.expression;
    assert.equal(not.root.kind, "unary");
    assert.equal(not.children[0].root.kind, "parenthesized");
    assert.equal(not.children[0].children[0].root.kind, "nullTest");

    const conditional = byTarget.case_check.expression;
    assert.equal(conditional.root.kind, "case");
    assert.equal(conditional.children[0].root.kind, "nullTest");
    const callArgument = conditional.children[1];
    assert.equal(callArgument.root.kind, "functionCall");
    assert.deepEqual(callArgument.children[0].root, { kind: "parameter", placeholder: "$2" });
    const fallback = conditional.children[2];
    assert.equal(fallback.root.kind, "cast");
    assert.deepEqual(fallback.children[0].root, { kind: "parameter", placeholder: "$1" });
    assert.equal(slice(fallback.children[0].span), "$1");
    const nestedParameters = children("parameter_value")[0].children;
    assert.deepEqual(
      nestedParameters.map((child) => child.root.kind),
      ["cast", "parameter"],
    );
    assert.deepEqual(
      [nestedParameters[0].children[0].root.placeholder, nestedParameters[1].root.placeholder],
      ["$3", "$4"],
    );

    const literal = (name) => root(name);
    assert.deepEqual(literal("null_literal"), {
      kind: "literal",
      sql: "NULL",
      value: { kind: "null" },
    });
    assert.deepEqual(literal("string_null_literal"), {
      kind: "literal",
      sql: "'NULL'",
      value: { kind: "string", value: "NULL" },
    });
    assert.deepEqual(literal("boolean_literal"), {
      kind: "literal",
      sql: "true",
      value: { kind: "boolean", value: true },
    });
    assert.deepEqual(literal("number_literal"), {
      kind: "literal",
      sql: "900719925474099312345",
      value: { kind: "number", value: "900719925474099312345" },
    });
    assert.deepEqual(literal("escaped_string_literal"), {
      kind: "literal",
      sql: "'it''s'",
      value: { kind: "string", value: "it's" },
    });
    const nestedLiterals = children("nested_literals");
    assert.deepEqual(
      nestedLiterals.map((child) => child.root.value),
      [
        { kind: "null" },
        { kind: "string", value: "NULL" },
        { kind: "boolean", value: true },
        { kind: "number", value: "900719925474099312345" },
        { kind: "string", value: "it's" },
      ],
    );
    assert.equal(slice(nestedLiterals[4].span), "'it''s'");
    assert.deepEqual(
      children("case_literals").map((child) => child.root.value),
      [{ kind: "boolean", value: true }, { kind: "null" }, { kind: "string", value: "NULL" }],
    );
    const literalVariants = children("literal_variants");
    assert.deepEqual(
      literalVariants.map((child) => child.root.value),
      [
        { kind: "string", value: "it's" },
        { kind: "string", value: "dollar 'quoted'" },
        { kind: "other", sql: "X'AB'" },
        { kind: "string", value: "snowman" },
        { kind: "string", value: "café" },
      ],
    );
    assert.deepEqual(
      literalVariants.map((child) => slice(child.span)),
      ["E'it\\'s'", "$$dollar 'quoted'$$", "X'AB'", "U&'snowman'", "N'café'"],
    );

    for (const [name, dataType, value] of [
      ["timestamp_value", "TIMESTAMP", "now"],
      ["timestamp_epoch_value", "TIMESTAMP", "epoch"],
      ["date_value", "DATE", "today"],
      ["timestamp_local_value", "TIMESTAMP WITHOUT TIME ZONE", "2025-01-02 03:04:05"],
      ["timestamp_tz_value", "TIMESTAMP WITH TIME ZONE", "2025-01-02 03:04:05+00"],
      ["time_local_value", "TIME WITHOUT TIME ZONE", "03:04:05"],
      ["time_tz_value", "TIME WITH TIME ZONE", "03:04:05+00"],
    ]) {
      assert.deepEqual(root(name), {
        kind: "typedLiteral",
        dataType,
        value,
        sql: `${dataType} '${value}'`,
      });
      assert.equal(byTarget[name].expression.childrenComplete, true);
    }

    // The additive expression facts preserve the established source summary/provenance contract.
    const legacySql = fixture("insert-recursive-expressions.sql");
    const legacy = await cjs.parsePostgresSql({ sql: legacySql });
    assert.deepEqual(await esm.parsePostgresSql({ sql: legacySql }), legacy);
    const oldAssignments = legacy.statements[0].insert.onConflict.action.assignments;
    assert.deepEqual(
      oldAssignments.slice(0, 3).map((item) => item.provenance),
      ["derived", "derived", "excludedColumn"],
    );
    assert.deepEqual(
      oldAssignments[3].expression.columns.map((column) =>
        column.parts.map((part) => part.identity).join("."),
      ),
      ["excluded.value", "target.value"],
    );
    assert.equal(oldAssignments[3].complete, false);
  },
);
