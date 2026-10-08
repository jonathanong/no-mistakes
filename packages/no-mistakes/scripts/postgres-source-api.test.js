const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const fixture = (name) =>
  readFileSync(join(__dirname, "../../../fixtures/postgres-facts/source", name), "utf8");

test(
  "compiled CJS and ESM source APIs retain safely attributed conditional occurrences",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    for (const name of [
      "nested-conditional-locks.sql",
      "conditional-adjacent-locks.sql",
      "nested-conditional-locks-mixed.sql",
    ]) {
      const sql = fixture(name);
      const facts = await api.parsePostgresSql({ sql });
      assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
      assert.deepEqual(facts.diagnostics, []);
      const statement = facts.statements.find((statement) => statement.kind === "doBlock");
      assert.ok(statement);
      assert.equal(
        Buffer.from(sql)
          .subarray(statement.span.start.offset, statement.span.end.offset)
          .toString(),
        statement.sql,
      );
      assert.equal(statement.block.complete, false);
      assert.equal(statement.block.statements.length, 1);
      assert.equal(statement.block.diagnostics.length, 2);
      assert.match(
        statement.block.diagnostics[0].message,
        /Unsupported nested procedural statement/,
      );
      assert.ok(statement.block.diagnostics[0].span);
      assert.deepEqual(
        facts.statements.map((statement) => statement.kind),
        name.endsWith("-mixed.sql") ? ["createIndex", "doBlock", "createIndex"] : ["doBlock"],
      );
      if (name.endsWith("-mixed.sql")) {
        for (const [index, expected] of [
          [0, "CREATE INDEX before_lock_idx ON sample_child(parent_id);"],
          [2, "CREATE INDEX after_lock_idx ON sample_parent(id);"],
        ]) {
          const neighbor = facts.statements[index];
          assert.equal(neighbor.sql, expected);
          assert.equal(
            Buffer.from(sql)
              .subarray(neighbor.span.start.offset, neighbor.span.end.offset)
              .toString(),
            expected,
          );
        }
      }
    }
  },
);

test(
  "compiled CJS and ESM expose CREATE INDEX ONLY metadata without false positives",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const basicSql = fixture("create-index-only-basic.sql");
    const basic = await cjs.parsePostgresSql({ sql: basicSql });
    assert.deepEqual(await esm.parsePostgresSql({ sql: basicSql }), basic);
    assert.deepEqual(basic.diagnostics, []);
    assert.equal(basic.statements.length, 1);
    assert.equal(basic.statements[0].kind, "createIndex");
    assert.equal(basic.statements[0].index.only, true);
    assert.equal(basic.statements[0].index.table.parts[0].identity, "example");
    assert.equal(basic.statements[0].index.keys[0].expression.sql, "id");
    const sql = fixture("create-index-only.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.equal(facts.diagnostics.length, 4);
    assert.deepEqual(
      facts.statements.map(({ ordinal }) => ordinal),
      [0, 1, 3, 4, 5, 7, 8, 9, 12],
    );
    assert.deepEqual(
      facts.statements.map(({ kind }) => kind),
      [
        "createIndex",
        "createIndex",
        "createIndex",
        "createIndex",
        "createIndex",
        "select",
        "createView",
        "wrapper",
        "createIndex",
      ],
    );
    assert.equal(facts.statements[0].index.only, true);
    assert.equal(facts.statements[0].index.table.parts[0].identity, "example");
    assert.equal(facts.statements[0].index.keys[0].expression.sql, "id");
    const qualified = facts.statements[1].index;
    assert.equal(qualified.only, true);
    assert.equal(qualified.unique, true);
    assert.deepEqual(
      qualified.table.parts.map(({ identity }) => identity),
      ["app", "Accounts"],
    );
    assert.equal(qualified.method, "gin");
    assert.equal(qualified.predicate.sql, "active");
    assert.equal(facts.statements[2].index.only, false);
    assert.equal(facts.statements[2].index.table.parts[0].identity, "ONLY");
    assert.equal(facts.statements[3].index.keys[0].expression.sql, "ONLY");
    assert.equal(facts.statements[4].index.predicate.sql, "ONLY IS TRUE");
    assert.match(facts.statements[6].view.query, /ON ONLY = b\.id/);
    assert.equal(facts.statements[7].wrapper.statements[0].index.only, true);
    assert.equal(facts.statements[8].index.only, false);
    assert.equal(
      facts.statements[8].index.structuralIdentity,
      '{"table":[["example",false]],"method":"btree","unique":false,"nullsDistinct":true,"keys":[["{\\"Identifier\\":{\\"value\\":\\"id\\",\\"quote_style\\":null}}",true,false,null]],"include":[],"predicate":null,"options":[]}',
    );
    const bytes = Buffer.from(sql);
    for (const statement of facts.statements) {
      assert.equal(
        bytes.subarray(statement.span.start.offset, statement.span.end.offset).toString(),
        statement.sql,
      );
    }
  },
);

test(
  "compiled CJS and ESM expose exact source spans for PostgreSQL constraints",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("constraint-source-spans.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    const bytes = Buffer.from(sql);
    const assertRange = (constraint, expected) => {
      assert.ok(constraint.span);
      assert.equal(
        bytes.subarray(constraint.span.start.offset, constraint.span.end.offset).toString(),
        expected,
      );
    };
    const create = facts.statements[0];
    assertRange(create.columns[0].constraints[0], "PRIMARY KEY");
    assertRange(create.columns[1].constraints[0], "REFERENCES public.parent(id)");
    assertRange(
      create.columns[2].constraints[0],
      "REFERENCES public.parent(id) ON DELETE SET DEFAULT",
    );
    assertRange(create.columns[3].constraints[0], "UNIQUE");
    assertRange(create.columns[3].constraints[1], "UNIQUE");
    assertRange(create.columns[4].constraints[0], "PRIMARY /* split keyword */ KEY");
    assertRange(create.columns[5].constraints[0], "UNIQUE");
    assertRange(create.columns[6].constraints[0], "UNIQUE");
    assertRange(create.constraints[0], "CONSTRAINT \"ck_name\" CHECK (length('λ') > 0)");
    assertRange(create.constraints[1], "UNIQUE (parent_id)");
    assertRange(
      facts.statements[2].operations[1].constraint,
      "CONSTRAINT after_other_operation CHECK (parent_id > 0)",
    );
    assertRange(
      facts.statements[3].operations[1].constraint,
      "CONSTRAINT after_array_default CHECK (parent_id > 0)",
    );
    assertRange(facts.statements[4].operations[0].column.constraints[0], "CHECK (added > 0)");
    const alter = facts.statements[1];
    assertRange(
      alter.operations[0].constraint,
      "CONSTRAINT fk_parent FOREIGN KEY (parent_id) REFERENCES public.parent(id) NOT VALID",
    );
    assertRange(alter.operations[1].constraint, "CHECK (id > 0)");
  },
);

test("constraint span issue example reports exact byte offsets", { skip: !compiled }, async () => {
  const api = require("../index.js");
  const esm = await import("../index.mjs");
  const sql = fixture("constraint-source-spans-basic.sql");
  const facts = await api.parsePostgresSql({ sql });
  assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
  const create = facts.statements[0];
  const alter = facts.statements[1];
  const values = [
    [create.columns[0].constraints[0], "REFERENCES public.parents(id)"],
    [create.constraints[0], "CONSTRAINT children_check CHECK (parent_id IS NOT NULL)"],
    [
      alter.operations[0].constraint,
      "CONSTRAINT children_parent_fk\n  FOREIGN KEY (parent_id) REFERENCES public.parents(id) NOT VALID",
    ],
  ];
  for (const [constraint, expected] of values) {
    assert.equal(
      Buffer.from(sql)
        .subarray(constraint.span.start.offset, constraint.span.end.offset)
        .toString(),
      expected,
    );
    assert.equal(constraint.span.start.offset, sql.indexOf(expected));
  }
});

test(
  "constraint spans survive wrappers and use decoded literal EXECUTE coordinates",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("constraint-source-spans-nested.sql");
    const facts = await api.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    const wrapped = facts.statements[0].wrapper.statements[0].constraints[0];
    assert.equal(
      Buffer.from(sql).subarray(wrapped.span.start.offset, wrapped.span.end.offset).toString(),
      "CONSTRAINT wrapper_check CHECK (id > 0)",
    );
    const execute = facts.statements[1].block.statements[0].execute;
    const decoded = execute.statements[0].constraints[0];
    assert.equal(
      Buffer.from(execute.decodedSql)
        .subarray(decoded.span.start.offset, decoded.span.end.offset)
        .toString(),
      "CONSTRAINT decoded_check CHECK (id > 0)",
    );
  },
);

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

test(
  "compiled source API preserves conditional occurrence provenance",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const sql = fixture("conditional.sql");
    const facts = await api.parsePostgresSql({ sql });
    assert.deepEqual(facts.diagnostics, []);
    const block = facts.statements[0].block;
    assert.equal(block.complete, true);
    assert.equal(block.bodyEncoding, "dollarQuoted");
    const encoded = fixture("conditional-single-quoted.sql");
    const quoted = await api.parsePostgresSql({ sql: encoded });
    assert.deepEqual(quoted.diagnostics, []);
    assert.equal(quoted.statements[0].block.bodyEncoding, "singleQuoted");
    assert.equal(quoted.statements[0].block.complete, true);
    const quoteStatement = quoted.statements[0].block.statements[1];
    assert.equal(
      Buffer.from(encoded)
        .subarray(quoteStatement.span.start.offset, quoteStatement.span.end.offset)
        .toString(),
      quoteStatement.sql,
    );
    assert.equal(quoteStatement.operations[0].column.name.value, "雪");
    assert.equal(quoteStatement.operations[0].column.default.sql, "'snow''s'");
    const conditional = block.statements[0];
    assert.equal(conditional.kind, "conditional");
    assert.equal(conditional.branches.length, 3);
    assert.equal(conditional.branches[2].condition, null);
    for (const branch of conditional.branches) {
      const statement = branch.statements[0];
      assert.equal(statement.kind, "alterTable");
      assert.equal(
        Buffer.from(sql)
          .subarray(statement.span.start.offset, statement.span.end.offset)
          .toString(),
        statement.sql,
      );
    }
    const table = await api.parsePostgresSql({ sql: fixture("table-source-boundary.sql") });
    assert.deepEqual(table.diagnostics, []);
    assert.equal(table.statements.length, 1);
    const neighbors = await api.parsePostgresSql({ sql: fixture("table-source-neighbors.sql") });
    assert.deepEqual(neighbors.diagnostics, []);
    assert.deepEqual(
      neighbors.statements.map((statement) => statement.kind),
      ["createView", "createTable", "select", "alterTable"],
    );
    assert.equal(neighbors.statements[0].view.dependenciesComplete, true);
    assert.equal(
      neighbors.statements[0].view.dependencies.find((name) => name.parts[0].identity === "Topics")
        .parts[0].quoted,
      true,
    );
  },
);

test(
  "compiled CJS and ESM source roots retain direct calls and ordered arguments",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    const source = { sql: fixture("expression-roots.sql") };
    const facts = await api.parsePostgresSql(source);
    assert.deepEqual(await esm.parsePostgresSql(source), facts);
    assert.deepEqual(facts.diagnostics, []);
    const columns = facts.statements[0].columns;
    assert.equal(columns[0].default.root.kind, "functionCall");
    assert.equal(columns[1].default.root.expression.kind, "case");
    assert.equal(columns[3].generated.expression.root.kind, "binary");
    const args = columns[6].default.root.arguments;
    assert.deepEqual(
      args.slice(0, 4).map((arg) => arg.root.kind),
      Array(4).fill("columnReference"),
    );
    assert.deepEqual(
      args.slice(4, 7).map((arg) => arg.root.kind),
      ["binary", "functionCall", "literal"],
    );
    assert.equal(columns[7].default.root.syntax, "value");
    assert.equal(columns[8].default.root.syntax, "call");
  },
);

test(
  "compiled source API preserves typed SELECT scopes through both module facades",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    const input = { sql: fixture("query-scopes.sql") };
    const facts = await api.parsePostgresSql(input);
    assert.deepEqual(await esm.parsePostgresSql(input), facts);
    assert.deepEqual(facts.diagnostics, []);
    const query = facts.statements[0].query;
    assert.equal(facts.statements[0].kind, "select");
    assert.equal(query.complete, true);
    assert.deepEqual(
      query.ctes.map((c) => [c.name.identity, c.used]),
      [
        ["unused", false],
        ["base", true],
        ["used", true],
      ],
    );
    assert.ok(query.equalities.some((e) => e.context.underOr && !e.context.mandatory));
    assert.ok(query.exists.some((e) => e.correlated && e.correlations.length > 0));
    assert.ok(facts.statements[2].query.ctes.every((c) => c.cyclic));
    const unsupported = await api.parsePostgresSql({ sql: fixture("query-unsupported.sql") });
    assert.ok(
      unsupported.statements.every((s) => !s.query.complete && s.query.unsupported.length > 0),
    );
  },
);

test(
  "compiled CJS and ESM source APIs emit recursive view facts and preserve neighbors",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    const source = { sql: fixture("recursive-views.sql") };
    const facts = await api.parsePostgresSql(source);
    assert.deepEqual(await esm.parsePostgresSql(source), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 8);
    for (const [index, names, quoted, replace, materialized] of [
      [0, ["recursive_view"], false, false, false],
      [1, ["App", "RecursiveView"], true, true, false],
      [2, ["app", "commented"], false, false, false],
      [3, ["ordinary_view"], false, false, false],
      [4, ["App", "OrdinaryView"], true, true, false],
      [5, ["materialized_view"], false, false, true],
      [6, ["recursive"], true, false, false],
    ]) {
      const statement = facts.statements[index];
      assert.equal(statement.kind, "createView");
      assert.deepEqual(
        statement.view.name.parts.map((part) => part.identity),
        names,
      );
      assert.ok(statement.view.name.parts.every((part) => part.quoted === quoted));
      assert.equal(statement.view.materialized, materialized);
      assert.equal(statement.view.orReplace, replace);
      assert.equal(statement.view.columns[0].identity, index === 1 || index === 4 ? "X" : "x");
      assert.equal(statement.view.dependenciesComplete, true);
      assert.equal(
        Buffer.from(source.sql)
          .subarray(statement.span.start.offset, statement.span.end.offset)
          .toString(),
        statement.sql,
      );
    }
    assert.equal(facts.statements[0].view.query, facts.statements[3].view.query);
    assert.equal(facts.statements[1].view.query, facts.statements[4].view.query);
    assert.match(facts.statements[6].view.query, /^WITH RECURSIVE/);
    const invalid = await api.parsePostgresSql({ sql: fixture("recursive-views-invalid.sql") });
    assert.equal(invalid.diagnostics.length, 4);
    assert.equal(invalid.statements.length, 1);
    assert.equal(invalid.statements[0].ordinal, 4);
  },
);

test(
  "compiled recursive views preserve implicit CTE scope and require columns",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    const sources = [
      { sql: fixture("recursive-view-scope.sql") },
      { sql: fixture("recursive-view-missing-columns.sql") },
    ];
    const [facts, invalid] = await api.parsePostgresSql(sources);
    assert.deepEqual(await esm.parsePostgresSql(sources), [facts, invalid]);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 7);
    for (const [index, expected] of [
      [0, ["app.seed"]],
      [1, []],
      [2, []],
      [3, []],
      [4, ['app."Nums"']],
      [5, ["ordinary"]],
    ]) {
      const statement = facts.statements[index];
      assert.equal(statement.kind, "createView");
      assert.equal(statement.view.temporary, index < 4);
      assert.equal(statement.view.orReplace, index === 2 || index === 3);
      assert.equal(statement.view.materialized, false);
      assert.equal(statement.view.dependenciesComplete, true);
      assert.deepEqual(
        statement.view.dependencies.map((name) => name.sql),
        expected,
      );
    }
    const nested = facts.statements[6].block.statements[0].branches[0].statements[0];
    assert.deepEqual(
      nested.view.dependencies.map((name) => name.sql),
      ["app.seed"],
    );
    assert.equal(invalid.diagnostics.length, 3);
    assert.ok(
      invalid.diagnostics
        .slice(0, 2)
        .every((diagnostic) => /explicit column-name list/.test(diagnostic.message)),
    );
    assert.equal(invalid.statements[0].block.complete, false);
    assert.match(invalid.statements[0].block.diagnostics[0].message, /explicit column-name list/);
    assert.equal(invalid.statements[1].sql, "SELECT 2;");
  },
);

test(
  "compiled CJS and ESM INSERT facts preserve conflict provenance and spans",
  { skip: !compiled },
  async () => {
    const sql = fixture("insert.sql");
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const result = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 13);
    for (const statement of result.statements) {
      assert.equal(statement.kind, "insert");
      assert.equal(
        Buffer.from(sql)
          .subarray(statement.span.start.offset, statement.span.end.offset)
          .toString(),
        statement.sql,
      );
    }
    const inserts = result.statements.map((statement) => statement.insert);
    assert.deepEqual(
      inserts.map((insert) => insert.source.kind),
      [
        "values",
        "values",
        "defaultValues",
        "select",
        "values",
        "values",
        "values",
        "values",
        "values",
        "values",
        "values",
        "values",
        "values",
      ],
    );
    assert.equal(inserts[1].table.parts[1].identity, "Accounts");
    assert.equal(inserts[1].columns[0].parts[0].identity, "ID");
    assert.equal(inserts[1].onConflict.target.kind, "omitted");
    assert.equal(inserts[4].onConflict.target.kind, "columns");
    assert.equal(inserts[5].onConflict.target.kind, "constraint");
    assert.equal(inserts[6].onConflict.predicate.sql, "id > 0");
    assert.deepEqual(
      inserts[7].onConflict.action.assignments.map((assignment) => assignment.provenance),
      ["targetColumn", "literal", "placeholder", "unresolved", "derived", "excludedColumn"],
    );
    assert.equal(inserts[7].complete, true);
    assert.equal(inserts[7].diagnostics.length, 0);
    assert.equal(inserts[8].onConflict.action.kind, "doNothing");
    assert.equal(inserts[9].complete, false);
    assert.equal(inserts[12].complete, true);
    const malformed = await cjs.parsePostgresSql({ sql: fixture("insert-errors.sql") });
    assert.equal(malformed.diagnostics.length, 10);
    assert.equal(malformed.statements.length, 1);
    assert.equal(malformed.statements[0].ordinal, 10);
    assert.equal(malformed.statements[0].insert.complete, true);
    assert.deepEqual(await esm.parsePostgresSql({ sql: fixture("insert-errors.sql") }), malformed);
    const modifierSql = fixture("insert-values-modifiers.sql");
    const modifiers = await cjs.parsePostgresSql({ sql: modifierSql });
    assert.deepEqual(await esm.parsePostgresSql({ sql: modifierSql }), modifiers);
    assert.deepEqual(modifiers.diagnostics, []);
    assert.deepEqual(
      modifiers.statements.map((item) => item.insert.complete),
      [false, false, false, false, true],
    );
    assert.ok(modifiers.statements.every((item) => item.insert.source.kind === "values"));
    const recoverySql = fixture("insert-review-invalid.sql");
    const recovery = await cjs.parsePostgresSql({ sql: recoverySql });
    assert.deepEqual(await esm.parsePostgresSql({ sql: recoverySql }), recovery);
    assert.equal(recovery.diagnostics.length, 3);
    assert.deepEqual(
      recovery.statements.map((item) => item.ordinal),
      [1, 3, 5, 6],
    );
    assert.equal(recovery.statements[2].insert.complete, true);
    for (const item of recovery.statements) {
      assert.equal(
        Buffer.from(recoverySql).subarray(item.span.start.offset, item.span.end.offset).toString(),
        item.sql,
      );
    }
    const reviewSql = fixture("insert-review.sql");
    const review = await cjs.parsePostgresSql({ sql: reviewSql });
    assert.deepEqual(await esm.parsePostgresSql({ sql: reviewSql }), review);
    assert.deepEqual(review.diagnostics, []);
    assert.equal(review.statements.length, 11);
    assert.equal(review.statements[0].insert.onConflict, null);
    assert.equal(review.statements[2].insert.onConflict.predicate.sql, "id > 0");
    assert.equal(review.statements[2].insert.source.query.ctes.length, 1);
    assert.deepEqual(
      review.statements[7].insert.onConflict.action.assignments.map((item) => item.provenance),
      ["literal", "literal", "literal", "unresolved", "unresolved", "unresolved"],
    );
    assert.deepEqual(
      review.statements.slice(8).map((item) => item.block.complete),
      [false, false, true],
    );
  },
);

test(
  "compiled ESM and CJS literal EXECUTE retain decoded facts and wrapper provenance",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("literal-execute.sql");
    const result = await api.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.deepEqual(
      result.statements.map((value) => value.kind),
      ["createTable", "doBlock", "createTable"],
    );
    const block = result.statements[1].block;
    assert.equal(block.complete, false);
    for (const [index, count] of [
      [0, 2],
      [1, 1],
    ]) {
      const wrapper = block.statements[index];
      assert.equal(wrapper.kind, "literalExecute");
      const execute = wrapper.execute;
      assert.equal(execute.complete, true);
      assert.equal(execute.statements.length, count);
      for (const child of execute.statements) {
        assert.equal(child.kind, "insert");
        assert.equal(
          Buffer.from(execute.decodedSql)
            .subarray(child.span.start.offset, child.span.end.offset)
            .toString(),
          child.sql,
        );
      }
      assert.equal(
        Buffer.from(sql).subarray(wrapper.span.start.offset, wrapper.span.end.offset).toString(),
        wrapper.sql,
      );
    }
    assert.equal(block.statements[2].execute.complete, false);
    assert.ok(block.statements[2].execute.diagnostics.length);
    assert.ok(block.statements.slice(3).every((value) => value.kind === "other"));
  },
);

test(
  "compiled branch EXECUTEs share literal facts across IF ELSIF and nested ELSE",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("literal-execute-conditional.sql");
    const result = await api.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    const branches = result.statements[0].block.statements[0].branches;
    const nested = branches[2].statements[0].branches[0];
    for (const statement of [
      branches[0].statements[0],
      branches[1].statements[0],
      nested.statements[1],
    ]) {
      assert.equal(statement.kind, "literalExecute");
      assert.equal(statement.execute.statements[0].kind, "insert");
      assert.equal(statement.execute.complete, true);
    }
    assert.equal(nested.statements[0].kind, "other");
  },
);

test(
  "compiled CJS and ESM distinguish composite syntax from assignment lineage",
  { skip: !compiled },
  async () => {
    const sql = readFileSync(
      join(__dirname, "../../../fixtures/postgres-facts/source/insert-composite.sql"),
      "utf8",
    );
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.equal(facts.statements.length, 10);
    assert.equal(facts.diagnostics.length, 1);
    const bytes = Buffer.from(sql);
    facts.statements.forEach(({ insert }, index) => {
      const assignment = insert.onConflict.action.assignments[0];
      assert.equal(insert.complete, index < 7);
      assert.equal(assignment.complete, index < 7);
      assert.equal(insert.diagnostics.length, index < 7 ? 0 : 1);
      assert.equal(
        assignment.provenance,
        index === 2 ? "excludedColumn" : [4, 5, 9].includes(index) ? "unresolved" : "derived",
      );
      if (index < 2) {
        const root = assignment.expression.root;
        assert.equal(root.kind, "functionCall");
        assert.equal(root.name.parts[0].identity, index === 0 ? "coalesce" : "greatest");
        assert.equal(root.argumentsComplete, true);
        assert.deepEqual(
          root.arguments.map((arg) => arg.sql),
          ["t.v", "EXCLUDED.v"],
        );
        assert.deepEqual(
          root.arguments.map((arg) => arg.root.kind),
          ["columnReference", "columnReference"],
        );
        assert.deepEqual(
          assignment.expression.columns.map((column) => column.sql),
          ["EXCLUDED.v", "t.v"],
        );
        for (const argument of root.arguments) {
          assert.equal(
            bytes.subarray(argument.span.start.offset, argument.span.end.offset).toString(),
            argument.sql,
          );
        }
      }
      if (index === 3) {
        const call = assignment.expression.root.expression.expression;
        assert.equal(call.kind, "functionCall");
        assert.deepEqual(
          call.arguments.map((arg) => arg.root.kind),
          ["functionCall", "literal", "literal"],
        );
        assert.equal(call.arguments[0].root.argumentsComplete, true);
      }
      if (index >= 7)
        assert.match(insert.diagnostics[0].message, /incompletely represented syntax/);
    });
  },
);

test(
  "compiled modifier SQL cannot imply complete typed assignment operands",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("insert-modifiers.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 5);
    facts.statements.forEach(({ insert }, index) => {
      const assignment = insert.onConflict.action.assignments[0];
      assert.equal(assignment.expression.root.argumentsComplete, true);
      assert.equal(assignment.expression.root.modifiers.length === 0, index === 4);
      assert.equal(assignment.provenance, "derived");
      assert.equal(assignment.complete, index === 4);
      assert.equal(insert.complete, index === 4);
      assert.equal(insert.diagnostics.length, index === 4 ? 0 : 1);
    });
  },
);

test(
  "compiled unary assignment roots retain typed operands and independent lineage",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("insert-unary.sql");
    const facts = await cjs.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), facts);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 4);
    facts.statements.forEach(({ insert }, index) => {
      const assignment = insert.onConflict.action.assignments[0];
      assert.equal(insert.complete, index < 3);
      assert.equal(assignment.complete, index < 3);
      assert.equal(insert.diagnostics.length, index < 3 ? 0 : 1);
      assert.equal(
        assignment.provenance,
        index === 0 ? "literal" : index === 1 ? "derived" : "unresolved",
      );
      const root = assignment.expression.root;
      if (index === 1) {
        assert.deepEqual(
          root.arguments.slice(1).map((arg) => arg.root.expression.kind),
          ["literal", "literal"],
        );
      } else {
        assert.equal(root.kind, "unary");
        assert.equal(
          root.expression.kind,
          index === 0 ? "literal" : index === 2 ? "columnReference" : "parenthesized",
        );
      }
    });
  },
);

test(
  "compiled CJS/ESM accepts expression arbiters and nested assignment targets",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("insert-conflict-expressions.sql");
    const pending = api.parsePostgresSql({ sql });
    assert.equal(typeof pending.then, "function");
    const result = await pending;
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 5);
    const inserts = result.statements.slice(0, 4).map((statement) => statement.insert);
    for (const [index, expected] of [
      [0, "values[1]"],
      [2, "1"],
      [3, 'coalesce("Values"[1], 2)'],
    ]) {
      const target = inserts[index].onConflict.action.assignments[0].target;
      assert.equal(target.subscripts[0].sql, expected);
      assert.equal(
        sql.slice(target.subscripts[0].span.start.offset, target.subscripts[0].span.end.offset),
        expected,
      );
      assert.ok(sql.slice(target.span.start.offset, target.span.end.offset).endsWith("]"));
      assert.equal(inserts[index].onConflict.action.assignments[0].provenance, "literal");
    }
    assert.equal(inserts[1].onConflict.target.kind, "expressions");
    assert.equal(
      inserts[1].onConflict.target.expressions[0].functions[0].name.parts[0].identity,
      "lower",
    );
    assert.equal(inserts[3].onConflict.target.expressions.length, 2);
    assert.equal(inserts[3].onConflict.predicate.sql, '"ID" > 0');
    assert.equal(inserts[3].onConflict.action.assignments.length, 3);
    assert.equal(inserts[3].onConflict.action.assignments[0].target.subscripts.length, 2);
    const rhs = inserts[3].onConflict.action.assignments[1].expression.span;
    assert.equal(
      sql.slice(rhs.start.offset, rhs.end.offset),
      "coalesce(EXCLUDED.slug, lower('x'))",
    );
    const assignment = inserts[3].onConflict.action.assignments[1].span;
    assert.equal(
      sql.slice(assignment.start.offset, assignment.end.offset),
      "slug = coalesce(EXCLUDED.slug, lower('x'))",
    );
    const invalid = await api.parsePostgresSql({
      sql: fixture("insert-conflict-expressions-invalid.sql"),
    });
    assert.deepEqual(
      await esm.parsePostgresSql({ sql: fixture("insert-conflict-expressions-invalid.sql") }),
      invalid,
    );
    assert.equal(invalid.diagnostics.length, 15);
    assert.deepEqual(
      invalid.statements.map((statement) => statement.kind),
      Array(15).fill("select"),
    );
  },
);

test(
  "compiled CJS/ESM retains mixed indirection, opclasses and nested call spans",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    const sql = fixture("insert-conflict-indirection.sql");
    const result = await api.parsePostgresSql({ sql });
    assert.deepEqual(await esm.parsePostgresSql({ sql }), result);
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.statements.length, 5);
    const inserts = result.statements.slice(0, 4).map((statement) => statement.insert);
    const slice = (span) => sql.slice(span.start.offset, span.end.offset);
    const arbiter = inserts[0].onConflict.target.expressions[0];
    assert.deepEqual(
      arbiter.functions.map((ref) => slice(ref.span)),
      ["lower(upper(slug))", "upper(slug)"],
    );
    const value = inserts[0].onConflict.action.assignments[0].expression;
    assert.equal(slice(value.root.arguments[1].span), "lower(upper(slug))");
    assert.equal(slice(value.root.arguments[1].root.arguments[0].span), "upper(slug)");
    assert.equal(slice(value.functions[0].span), "coalesce(EXCLUDED.slug, lower(upper(slug)))");
    const unary = result.statements[0].insert.onConflict.action.assignments[3];
    assert.equal(unary.complete, true);
    assert.equal(slice(unary.expression.span), "-abs(length(lower(slug)))");
    assert.equal(unary.expression.root.kind, "unary");
    const unaryCall = unary.expression.root.expression;
    assert.equal(slice(unaryCall.arguments[0].span), "length(lower(slug))");
    assert.equal(slice(unaryCall.arguments[0].root.arguments[0].span), "lower(slug)");
    const target = inserts[1].onConflict.action.assignments[1].target;
    assert.deepEqual(
      target.indirection.map((step) => step.kind),
      ["subscript", "field", "subscript", "field"],
    );
    assert.equal(slice(target.indirection[1].span), '"Items"');
    const fieldOnly = inserts[1].onConflict.action.assignments[3];
    assert.equal(fieldOnly.complete, true);
    assert.equal(fieldOnly.columns[0].sql, "records");
    assert.equal(fieldOnly.target.base.sql, "records");
    assert.deepEqual(fieldOnly.target.subscripts, []);
    assert.equal(slice(fieldOnly.target.span), 'records."Name"');
    assert.equal(fieldOnly.target.indirection[0].name.quoted, true);
    assert.equal(slice(fieldOnly.target.indirection[0].span), '"Name"');
    const leading = inserts[1].onConflict.action.assignments[2];
    assert.equal(leading.columns[0].sql, "records");
    assert.equal(leading.target.base.sql, "records");
    assert.deepEqual(
      leading.target.indirection.map((step) => step.kind),
      ["field", "field", "subscript", "subscript", "field"],
    );
    const parameter = inserts[2].onConflict.target.operatorClasses[0].parameters[0];
    assert.equal(parameter.name.identity, "siglen");
    assert.equal(slice(parameter.value.span), "32");
    for (const [predicate, expected, nested] of [
      [
        inserts[2].onConflict.predicate,
        "coalesce(is_ready(lower(slug)), false)",
        "is_ready(lower(slug))",
      ],
      [
        inserts[2].onConflict.action.predicate,
        "coalesce(is_ready(upper(slug)), true)",
        "is_ready(upper(slug))",
      ],
    ]) {
      assert.equal(slice(predicate.span), expected);
      assert.equal(slice(predicate.functions[0].span), expected);
      assert.equal(slice(predicate.functions[1].span), nested);
      assert.equal(slice(predicate.root.arguments[0].span), nested);
      assert.equal(
        slice(predicate.root.arguments[0].root.arguments[0].span),
        expected.endsWith("false)") ? "lower(slug)" : "upper(slug)",
      );
    }
    assert.equal(slice(target.indirection[2].span), "[coalesce(records[1].idx, 2)]");
    assert.equal(slice(target.subscripts[1].functions[0].span), "coalesce(records[1].idx, 2)");
    assert.deepEqual(
      inserts[2].onConflict.target.operatorClasses.map((value) => value && slice(value.span)),
      ["text_pattern_ops (siglen = 32)", '"Ops"."IntOps"', null],
    );
    assert.equal(slice(inserts[3].onConflict.target.operatorClasses[0].span), '"Ops"."TextOps"');
  },
);
