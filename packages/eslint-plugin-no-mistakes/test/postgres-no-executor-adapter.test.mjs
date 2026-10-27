import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { require } from "./helpers.mjs";

const RULE = "postgres-no-executor-adapter";
const DB = { importSpecifier: "@example/db" };

const IMPORT = `import { read, write } from "@example/db";\n`;
const SCOPED = {
  importSpecifier: "@example/db",
  executorFactoryNames: ["openTx"],
  executorTypeNames: ["Tx"],
};
const TX = `import { read, type Tx } from "@example/db";\n`;

function ruleConfig(option) {
  return {
    files: ["**/*.{js,jsx,ts,tsx,mjs,mts,cjs,cts}"],
    languageOptions: {
      ecmaVersion: 2024,
      sourceType: "module",
      parser: tsParser,
      parserOptions: { ecmaFeatures: { jsx: true } },
    },
    plugins: {
      "no-mistakes": require("../src/index.js"),
    },
    rules: {
      [`no-mistakes/${RULE}`]: ["error", option],
    },
  };
}

function check(code, option = DB, filename = "app.ts") {
  const linter = new Linter({ configType: "flat" });
  const config = ruleConfig(option);
  const reported = linter.verify(code, config, filename);
  const result = linter.verifyAndFix(code, config, filename);
  return {
    ids: reported.map((message) => message.messageId),
    output: result.output,
    messages: reported.map((message) => message.message),
  };
}

const recorded = [];
function recordDiagnostic(code, option, filename, expected) {
  recorded.push({ code, option, filename, empty: expected.length === 0 });
}

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
describe("postgres-no-executor-adapter", () => {
  it("reports an exact forwarder and names the judged call instead of rewriting it", () => {
    const arrow = check(`${IMPORT}run((statement) => read(statement), account);`);
    recordDiagnostic(`${IMPORT}run((statement) => read(statement), account);`, DB, "app.ts", [
      "forward",
    ]);
    assert.equal(arrow.output, `${IMPORT}run((statement) => read(statement), account);`);

    const fn = check(`${IMPORT}run(function (statement) { return read(statement); }, account);`);
    recordDiagnostic(
      `${IMPORT}run(function (statement) { return read(statement); }, account);`,
      DB,
      "app.ts",
      ["forward"],
    );
    assert.equal(
      fn.output,
      `${IMPORT}run(function (statement) { return read(statement); }, account);`,
    );

    const mapped = check(`${IMPORT}items.map((statement) => read(statement));`);
    recordDiagnostic(`${IMPORT}items.map((statement) => read(statement));`, DB, "app.ts", [
      "forward",
    ]);
    assert.equal(mapped.output, `${IMPORT}items.map((statement) => read(statement));`);

    const zero = check(`${IMPORT}run(() => read());`);
    recordDiagnostic(`${IMPORT}run(() => read());`, DB, "app.ts", []);
    assert.equal(zero.output, `${IMPORT}run(() => read());`);

    const pair = check(`${IMPORT}run((statement, values) => read(statement, values), account);`);
    recordDiagnostic(
      `${IMPORT}run((statement, values) => read(statement, values), account);`,
      DB,
      "app.ts",
      ["forward"],
    );
    assert.equal(
      pair.output,
      `${IMPORT}run((statement, values) => read(statement, values), account);`,
    );
  });

  it("reports conditional executor selection without rewriting it", () => {
    const code = `${IMPORT}run(flag ? read : write, account);`;
    const result = check(code);
    recordDiagnostic(code, DB, "app.ts", ["blocked"]);
    assert.equal(result.output, code);

    const nested = check(`${IMPORT}run(flag ? read : (other ? write : read), account);`);
    recordDiagnostic(`${IMPORT}run(flag ? read : (other ? write : read), account);`, DB, "app.ts", [
      "blocked",
    ]);

    assert.equal(nested.output, `${IMPORT}run(flag ? read : (other ? write : read), account);`);

    const inside = check(`${IMPORT}run((statement) => (flag ? read : write)(statement), account);`);
    recordDiagnostic(
      `${IMPORT}run((statement) => (flag ? read : write)(statement), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    assert.equal(
      inside.output,
      `${IMPORT}run((statement) => (flag ? read : write)(statement), account);`,
    );

    const returned = check(
      `${IMPORT}run((statement) => (flag ? read(statement) : write(statement)), account);`,
    );
    recordDiagnostic(
      `${IMPORT}run((statement) => (flag ? read(statement) : write(statement)), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    assert.equal(
      returned.output,
      `${IMPORT}run((statement) => (flag ? read(statement) : write(statement)), account);`,
    );

    // An untyped member is not an executor. Name only `read`, not the member or the conditional.
    // One passed finding.
    const untypedMember = check(`${IMPORT}run(flag ? client.query : read, account);`);
    recordDiagnostic(`${IMPORT}run(flag ? client.query : read, account);`, DB, "app.ts", [
      "passed",
    ]);
    assert.equal(untypedMember.output, `${IMPORT}run(flag ? client.query : read, account);`);

    const mixed = check(
      `${TX}function load(tx: Tx, flag: boolean, account: string) {\n  return run(flag ? tx.query : read, account);\n}\n`,
      SCOPED,
    );
    recordDiagnostic(
      `${TX}function load(tx: Tx, flag: boolean, account: string) {\n  return run(flag ? tx.query : read, account);\n}\n`,
      SCOPED,
      "app.ts",
      ["blocked"],
    );

    const oneBranch = check(`${IMPORT}run(flag ? read : other, account);`);
    recordDiagnostic(`${IMPORT}run(flag ? read : other, account);`, DB, "app.ts", ["passed"]);
    assert.equal(oneBranch.output, `${IMPORT}run(flag ? read : other, account);`);

    const nestedPartial = check(`${IMPORT}run(flag ? read : (other ? write : unknown), account);`);
    recordDiagnostic(
      `${IMPORT}run(flag ? read : (other ? write : unknown), account);`,
      DB,
      "app.ts",
      ["passed"],
    );

    const lambda = check(
      `${IMPORT}run((statement) => (flag ? read(statement) : normalize(statement)), account);`,
    );
    recordDiagnostic(
      `${IMPORT}run((statement) => (flag ? read(statement) : normalize(statement)), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );
    assert.equal(
      lambda.output,
      `${IMPORT}run((statement) => (flag ? read(statement) : normalize(statement)), account);`,
    );

    const nestedLambda = check(
      `${IMPORT}run((statement) => (flag ? read(statement) : (other ? write(statement) : normalize(statement))), account);`,
    );
    recordDiagnostic(
      `${IMPORT}run((statement) => (flag ? read(statement) : (other ? write(statement) : normalize(statement))), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    const valueAndAdapter = check(
      `${IMPORT}run(flag ? read : ((statement) => write(statement)), account);`,
    );
    recordDiagnostic(
      `${IMPORT}run(flag ? read : ((statement) => write(statement)), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    const adapterAndUnknown = check(
      `${IMPORT}run(flag ? ((statement) => read(statement)) : other, account);`,
    );
    recordDiagnostic(
      `${IMPORT}run(flag ? ((statement) => read(statement)) : other, account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    const bothAdapters = check(
      `${IMPORT}run(flag ? ((statement) => read(statement)) : ((statement) => write(statement)), account);`,
    );
    recordDiagnostic(
      `${IMPORT}run(flag ? ((statement) => read(statement)) : ((statement) => write(statement)), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    recordDiagnostic(`${IMPORT}run(flag ? other : another, account);`, DB, "app.ts", []);
    recordDiagnostic(
      `${IMPORT}run((statement) => (flag ? normalize(statement) : other(statement)), account);`,
      DB,
      "app.ts",
      [],
    );
  });

  it("does not autofix a different receiver, this, arity, rest, or side effect", () => {
    const memberCode = `${TX}function load(tx: Tx, account: string) {\n  return run((statement) => tx.query(statement), account);\n}\n`;
    const member = check(memberCode, SCOPED);
    recordDiagnostic(memberCode, SCOPED, "app.ts", ["blocked"]);

    assert.equal(member.output, memberCode);
    recordDiagnostic(
      `${IMPORT}run((statement) => client.query(statement), account);`,
      DB,
      "app.ts",
      [],
    );

    const computed = check(
      `${TX}function load(tx: Tx, account: string) {\n  return run((statement) => tx["query"](statement), account);\n}\n`,
      SCOPED,
    );
    recordDiagnostic(
      `${TX}function load(tx: Tx, account: string) {\n  return run((statement) => tx["query"](statement), account);\n}\n`,
      SCOPED,
      "app.ts",
      ["blocked"],
    );

    const template = check(
      `${TX}function load(tx: Tx, account: string) {\n  return run((statement) => tx[\`query\`](statement), account);\n}\n`,
      SCOPED,
    );
    recordDiagnostic(
      `${TX}function load(tx: Tx, account: string) {\n  return run((statement) => tx[\`query\`](statement), account);\n}\n`,
      SCOPED,
      "app.ts",
      ["blocked"],
    );

    const call = check(`${IMPORT}run((statement) => read.call(receiver, statement), account);`);
    recordDiagnostic(
      `${IMPORT}run((statement) => read.call(receiver, statement), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    assert.equal(
      call.output,
      `${IMPORT}run((statement) => read.call(receiver, statement), account);`,
    );

    const apply = check(`${IMPORT}run((statement) => read.apply(receiver, [statement]), account);`);
    recordDiagnostic(
      `${IMPORT}run((statement) => read.apply(receiver, [statement]), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    assert.equal(
      apply.output,
      `${IMPORT}run((statement) => read.apply(receiver, [statement]), account);`,
    );

    const applySpread = check(
      `${IMPORT}run((...args) => read.apply(receiver, [...args]), account);`,
    );
    recordDiagnostic(
      `${IMPORT}run((...args) => read.apply(receiver, [...args]), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    const applyHole = check(
      `${IMPORT}run((statement) => read.apply(receiver, [statement, , extra]), account);`,
    );
    recordDiagnostic(
      `${IMPORT}run((statement) => read.apply(receiver, [statement, , extra]), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    const applyIdent = check(
      `${IMPORT}run((statement) => read.apply(receiver, statement), account);`,
    );
    recordDiagnostic(
      `${IMPORT}run((statement) => read.apply(receiver, statement), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    recordDiagnostic(
      `${IMPORT}run((statement) => read.call(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}run((statement) => read.apply(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}run((statement) => other.call(null, statement), account);`,
      DB,
      "app.ts",
      [],
    );
    const conditionalCall = check(
      `${IMPORT}run((statement) => (flag ? read : write).call(null, statement), account);`,
    );
    recordDiagnostic(
      `${IMPORT}run((statement) => (flag ? read : write).call(null, statement), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    assert.equal(
      conditionalCall.output,
      `${IMPORT}run((statement) => (flag ? read : write).call(null, statement), account);`,
    );

    for (const body of [
      "(statement) => read(statement, extra)",
      "(statement, values) => read(statement)",
      "(statement, values) => read(values, statement)",
      "(...args) => read(...args)",
      "(statement = null) => read(statement)",
      "(statement) => read(...statement)",
    ]) {
      const result = check(`${IMPORT}run(${body}, account);`);
      recordDiagnostic(`${IMPORT}run(${body}, account);`, DB, "app.ts", ["blocked"]);

      assert.equal(result.output, `${IMPORT}run(${body}, account);`);
    }

    const effects = [
      "(statement) => { log(); return read(statement); }",
      "(statement) => { read(statement); }",
      "(statement) => (log(), read(statement))",
      "(statement) => void read(statement)",
    ];
    for (const body of effects) {
      const result = check(`${IMPORT}run(${body}, account);`);
      recordDiagnostic(`${IMPORT}run(${body}, account);`, DB, "app.ts", ["blocked"]);

      assert.equal(result.output, `${IMPORT}run(${body}, account);`, body);
    }
  });

  it("does not autofix async, generators, await, or type assertions", () => {
    const asyncArrow = check(`${IMPORT}run(async (statement) => read(statement), account);`);
    recordDiagnostic(`${IMPORT}run(async (statement) => read(statement), account);`, DB, "app.ts", [
      "blocked",
    ]);

    assert.equal(asyncArrow.output, `${IMPORT}run(async (statement) => read(statement), account);`);

    const awaited = check(`${IMPORT}run(async (statement) => await read(statement), account);`);
    recordDiagnostic(
      `${IMPORT}run(async (statement) => await read(statement), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    const generator = check(
      `${IMPORT}run(function* (statement) { return read(statement); }, account);`,
    );
    recordDiagnostic(
      `${IMPORT}run(function* (statement) { return read(statement); }, account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    const asserted = check(`${IMPORT}run((statement) => read(statement) as Result, account);`);
    recordDiagnostic(
      `${IMPORT}run((statement) => read(statement) as Result, account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    assert.equal(
      asserted.output,
      `${IMPORT}run((statement) => read(statement) as Result, account);`,
    );

    const argAsserted = check(`${IMPORT}run((statement) => read(statement as string), account);`);
    recordDiagnostic(
      `${IMPORT}run((statement) => read(statement as string), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    const wrapped = check(`${IMPORT}run(((statement) => read(statement)) as Executor, account);`);
    recordDiagnostic(
      `${IMPORT}run(((statement) => read(statement)) as Executor, account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    assert.equal(
      wrapped.output,
      `${IMPORT}run(((statement) => read(statement)) as Executor, account);`,
    );

    const wrappedConditional = check(`${IMPORT}run((flag ? read : write) as Executor, account);`);
    recordDiagnostic(`${IMPORT}run((flag ? read : write) as Executor, account);`, DB, "app.ts", [
      "blocked",
    ]);

    const wrappedAsserted = check(
      `${IMPORT}run(((statement) => read(statement) as Result) as Executor, account);`,
    );
    recordDiagnostic(
      `${IMPORT}run(((statement) => read(statement) as Result) as Executor, account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    assert.equal(
      wrappedAsserted.output,
      `${IMPORT}run(((statement) => read(statement) as Result) as Executor, account);`,
    );
  });

  it("treats this on an otherwise exact forwarder as a different receiver", () => {
    const code = `${IMPORT}function load(account: string) {\n  return run(function (statement) {\n    return read(statement) as typeof this;\n  }, account);\n}\n`;
    const result = check(code);
    recordDiagnostic(code, DB, "app.ts", ["blocked"]);

    assert.equal(result.output, code);

    const nested = check(
      `${IMPORT}run(function (statement) {\n  function inner() { return this; }\n  return read(statement);\n}, account);`,
    );
    recordDiagnostic(
      `${IMPORT}run(function (statement) {\n  function inner() { return this; }\n  return read(statement);\n}, account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    assert.equal(
      nested.output,
      `${IMPORT}run(function (statement) {\n  function inner() { return this; }\n  return read(statement);\n}, account);`,
    );

    const classThis = check(
      `${IMPORT}run(function (statement) {\n  const Nested = class { method() { return this; } };\n  return read(statement);\n}, account);`,
    );
    recordDiagnostic(
      `${IMPORT}run(function (statement) {\n  const Nested = class { method() { return this; } };\n  return read(statement);\n}, account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    const arrowThis = check(
      `${IMPORT}run(function (statement) {\n  const nested = () => this;\n  return read(statement);\n}, account);`,
    );
    recordDiagnostic(
      `${IMPORT}run(function (statement) {\n  const nested = () => this;\n  return read(statement);\n}, account);`,
      DB,
      "app.ts",
      ["blocked"],
    );
  });

  it("ignores shadowed selectors, unrelated objects, and transforming callbacks", () => {
    recordDiagnostic(
      `${IMPORT}function load(read: (sql: string) => unknown, account: string) {\n  return run((statement) => read(statement), account);\n}\n`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `import { read } from "@other/db";\nrun((statement) => read(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `import { read } from "@example/db/read";\nrun((statement) => read(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `import read from "@example/db";\nrun((statement) => read(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `import * as db from "@example/db";\nrun((statement) => db.read(statement), account);`,
      DB,
      "app.ts",
      ["blocked"],
    );
    recordDiagnostic(
      `import type { read } from "@example/db";\ndeclare function read(sql: string): unknown;\nrun((statement) => read(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}run((statement) => read(normalize(statement)), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}run(({ statement }) => read(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(`${IMPORT}run((statement) => read(), account);`, DB, "app.ts", []);
    recordDiagnostic(`${IMPORT}run((statement) => read("SELECT 1"), account);`, DB, "app.ts", []);
    recordDiagnostic(
      `${IMPORT}run((statement) => () => read(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(`${IMPORT}run((statement) => read?.(statement), account);`, DB, "app.ts", [
      "forward",
    ]);
    recordDiagnostic(
      `${IMPORT}run((statement) => read(statement?.trim()), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(`${IMPORT}run(read?.(statement), account);`, DB, "app.ts", []);
    const assertedMember = check(
      `${IMPORT}run(flag ? (client?.query as Executor) : read, account);`,
    );
    recordDiagnostic(
      `${IMPORT}run(flag ? (client?.query as Executor) : read, account);`,
      DB,
      "app.ts",
      ["passed"],
    );

    recordDiagnostic(
      `${IMPORT}run((statement) => flag && read(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(`${IMPORT}(flag ? read : write)("SELECT 1");`, DB, "app.ts", []);
    recordDiagnostic(
      `${IMPORT}run((statement) => { if (flag) return read(statement); }, account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(`${IMPORT}run((statement) => { return; }, account);`, DB, "app.ts", []);
    recordDiagnostic(`${IMPORT}run(() => { }, account);`, DB, "app.ts", []);
  });

  it("reports an executor passed as a value and a const adapter that closes over one", () => {
    const passed = check(`${IMPORT}run(read, account);`);
    recordDiagnostic(`${IMPORT}run(read, account);`, DB, "app.ts", ["passed"]);
    assert.equal(passed.output, `${IMPORT}run(read, account);`);

    const queryCode = `${TX}function load(tx: Tx, account: string) {\n  return run(tx.query, account);\n}\n`;
    const query = check(queryCode, SCOPED);
    recordDiagnostic(queryCode, SCOPED, "app.ts", ["passed"]);

    assert.equal(query.output, queryCode);
    const eitherQuery = check(
      `${TX}function f(tx: Tx, u: Tx, flag: boolean, a: string) {\n  return run((flag ? tx : u).query, a);\n}\n`,
      SCOPED,
    );
    recordDiagnostic(
      `${TX}function f(tx: Tx, u: Tx, flag: boolean, a: string) {\n  return run((flag ? tx : u).query, a);\n}\n`,
      SCOPED,
      "app.ts",
      ["passed"],
    );

    // One unproven receiver keeps the member unproven. Naming it would claim that receiver.
    // Zero findings.
    recordDiagnostic(
      `${TX}function f(tx: Tx, client: { query(sql: string): unknown }, flag: boolean, a: string) {\n  run((flag ? tx : client).query, a);\n  run((flag ? client : tx).query, a);\n}\n`,
      SCOPED,
      "app.ts",
      [],
    );
    recordDiagnostic(`${IMPORT}run(client.query, account);`, DB, "app.ts", []);
    recordDiagnostic(`${IMPORT}run(client["query"], account);`, DB, "app.ts", []);

    const computed = check(
      `${TX}function load(tx: Tx, account: string) {\n  return run(tx["query"], account);\n}\n`,
      SCOPED,
    );
    recordDiagnostic(
      `${TX}function load(tx: Tx, account: string) {\n  return run(tx["query"], account);\n}\n`,
      SCOPED,
      "app.ts",
      ["passed"],
    );

    const untypedAlias = check(
      `${IMPORT}const exec = flag ? client.query : read;\nrun(exec, account);`,
    );
    recordDiagnostic(
      `${IMPORT}const exec = flag ? client.query : read;\nrun(exec, account);`,
      DB,
      "app.ts",
      ["passed"],
    );

    const mixed = check(
      `${TX}function load(tx: Tx, flag: boolean, account: string) {\n  const exec = flag ? tx.query : read;\n  return run(exec, account);\n}\n`,
      SCOPED,
    );
    recordDiagnostic(
      `${TX}function load(tx: Tx, flag: boolean, account: string) {\n  const exec = flag ? tx.query : read;\n  return run(exec, account);\n}\n`,
      SCOPED,
      "app.ts",
      ["passed"],
    );

    const selected = check(`${IMPORT}const exec = flag ? read : write;\nrun(exec, account);`);
    recordDiagnostic(
      `${IMPORT}const exec = flag ? read : write;\nrun(exec, account);`,
      DB,
      "app.ts",
      ["passed"],
    );

    assert.match(selected.output, /run\(exec, account\)/);

    const alias = check(`${IMPORT}const exec = read;\nrun(exec, account);`);
    recordDiagnostic(`${IMPORT}const exec = read;\nrun(exec, account);`, DB, "app.ts", ["passed"]);

    const nestedAlias = check(
      `${IMPORT}const reader = read;\nconst exec = flag ? reader : write;\nrun(exec, account);`,
    );
    recordDiagnostic(
      `${IMPORT}const reader = read;\nconst exec = flag ? reader : write;\nrun(exec, account);`,
      DB,
      "app.ts",
      ["passed"],
    );

    const asserted = check(`${IMPORT}run(read as Executor, account);`);
    recordDiagnostic(`${IMPORT}run(read as Executor, account);`, DB, "app.ts", ["passed"]);

    assert.equal(asserted.output, `${IMPORT}run(read as Executor, account);`);

    const adapter = check(
      `${IMPORT}const adapter = (statement) => read(statement);\nrun(adapter, account);`,
    );
    recordDiagnostic(
      `${IMPORT}const adapter = (statement) => read(statement);\nrun(adapter, account);`,
      DB,
      "app.ts",
      ["forward"],
    );

    assert.match(adapter.output, /run\(adapter, account\)/);

    const functionAdapter = check(
      `${IMPORT}const adapter = function (statement) { return read(statement); };\nrun(adapter, account);`,
    );
    recordDiagnostic(
      `${IMPORT}const adapter = function (statement) { return read(statement); };\nrun(adapter, account);`,
      DB,
      "app.ts",
      ["forward"],
    );

    const memberAdapter = check(
      `${TX}function load(tx: Tx, account: string) {\n  const adapter = (statement) => tx.query(statement);\n  return run(adapter, account);\n}\n`,
      SCOPED,
    );
    recordDiagnostic(
      `${TX}function load(tx: Tx, account: string) {\n  const adapter = (statement) => tx.query(statement);\n  return run(adapter, account);\n}\n`,
      SCOPED,
      "app.ts",
      ["blocked"],
    );

    const asyncAdapter = check(
      `${IMPORT}const adapter = async (statement) => read(statement);\nrun(adapter as Executor, account);`,
    );
    recordDiagnostic(
      `${IMPORT}const adapter = async (statement) => read(statement);\nrun(adapter as Executor, account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    const exactWrapped = check(
      `${IMPORT}const adapter = (statement) => read(statement);\nrun(adapter as Executor, account);`,
    );
    recordDiagnostic(
      `${IMPORT}const adapter = (statement) => read(statement);\nrun(adapter as Executor, account);`,
      DB,
      "app.ts",
      ["blocked"],
    );

    recordDiagnostic(`${IMPORT}let exec = read;\nrun(exec, account);`, DB, "app.ts", []);
    recordDiagnostic(
      `${IMPORT}var exec = read;\nvar exec = write;\nrun(exec, account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}const { read: exec } = { read };\nrun(exec, account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}const left = right;\nconst right = left;\nrun(left, account);`,
      DB,
      "app.ts",
      [],
    );
    const partialAlias = check(`${IMPORT}const exec = flag ? read : other;\nrun(exec, account);`);
    recordDiagnostic(
      `${IMPORT}const exec = flag ? read : other;\nrun(exec, account);`,
      DB,
      "app.ts",
      ["passed"],
    );

    assert.match(partialAlias.output, /run\(exec, account\)/);
    recordDiagnostic(`${IMPORT}const exec = other();\nrun(exec, account);`, DB, "app.ts", []);
    recordDiagnostic(`${IMPORT}const exec = client?.query;\nrun(exec, account);`, DB, "app.ts", []);
    recordDiagnostic(
      `${IMPORT}for (const exec of [read]) {\n  run(exec, account);\n}\n`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}const adapter = (statement) => read(normalize(statement));\nrun(adapter, account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(`${IMPORT}const adapter = () => read();\nrun(adapter);`, DB, "app.ts", []);
    // A stored property read is not followed: the object can be mutated.
    // Constructor arguments and fresh object or array values are fixture-backed.
    // Zero findings.
    recordDiagnostic(
      `${IMPORT}const options = { exec: (statement) => read(statement) };\nrun(options.exec, account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}function load(read: (sql: string) => unknown, account: string) {\n  return run(read, account);\n}\n`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(`import { read } from "@other/db";\nrun(read, account);`, DB, "app.ts", []);

    const renamed = check(`import { write as save } from "@example/db";\nrun(save, account);`);
    recordDiagnostic(
      `import { write as save } from "@example/db";\nrun(save, account);`,
      DB,
      "app.ts",
      ["passed"],
    );

    const namesOnly = { executorNames: ["read"] };
    recordDiagnostic(
      `import { read } from "@example/db";\nrun(read, account);`,
      namesOnly,
      "app.ts",
      ["passed"],
    );
    recordDiagnostic(`run(client.query, account);`, namesOnly, "app.ts", []);
    recordDiagnostic(`run(client.query, account);`, { executorNames: ["query"] }, "app.ts", [
      "passed",
    ]);
    recordDiagnostic(
      `${IMPORT}run(client.query, account);`,
      {
        importSpecifier: "@example/db",
        executorNames: ["read"],
      },
      "app.ts",
      [],
    );
  });

  it("matches renamed imports and configured names without a module", () => {
    const renamed = check(
      `import { write as save } from "@example/db";\nrun((statement) => save(statement), account);`,
    );
    recordDiagnostic(
      `import { write as save } from "@example/db";\nrun((statement) => save(statement), account);`,
      DB,
      "app.ts",
      ["forward"],
    );

    assert.equal(
      renamed.output,
      `import { write as save } from "@example/db";\nrun((statement) => save(statement), account);`,
    );

    const namesOnly = { executorNames: ["read"] };
    recordDiagnostic(
      `import { read } from "@example/db";\nrun((statement) => read(statement), account);`,
      namesOnly,
      "app.ts",
      ["forward"],
    );
    recordDiagnostic(
      `run((statement) => client.query(statement), account);`,
      namesOnly,
      "app.ts",
      [],
    );

    const withModule = { importSpecifier: "@example/db", executorNames: ["read"] };
    recordDiagnostic(
      `${IMPORT}run((statement) => client.query(statement), account);`,
      withModule,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${TX}function load(tx: Tx, account: string) {\n  return run((statement) => tx.query(statement), account);\n}\n`,
      { ...withModule, executorTypeNames: ["Tx"] },
      "app.ts",
      ["blocked"],
    );
  });

  it("follows scoped factory and typed executors and ignores shadows", () => {
    const option = {
      importSpecifier: "@example/db",
      executorNames: [],
      executorFactoryNames: ["openTx"],
      executorTypeNames: ["Tx"],
    };
    const typedPassed = check(
      `import type { Tx } from "@example/db";\nfunction load(tx: Tx, account: string) {\n  return run(tx, account);\n}\n`,
      option,
    );
    recordDiagnostic(
      `import type { Tx } from "@example/db";\nfunction load(tx: Tx, account: string) {\n  return run(tx, account);\n}\n`,
      option,
      "app.ts",
      ["passed"],
    );

    assert.match(typedPassed.output, /run\(tx, account\)/);

    const typed = check(
      `import type { Tx } from "@example/db";\nfunction load(tx: Tx, account: string) {\n  return run((statement) => tx(statement), account);\n}\n`,
      option,
    );
    recordDiagnostic(
      `import type { Tx } from "@example/db";\nfunction load(tx: Tx, account: string) {\n  return run((statement) => tx(statement), account);\n}\n`,
      option,
      "app.ts",
      ["forward"],
    );

    assert.equal(
      typed.output,
      `import type { Tx } from "@example/db";\nfunction load(tx: Tx, account: string) {\n  return run((statement) => tx(statement), account);\n}\n`,
    );

    const factoryPassed = check(
      `import { openTx } from "@example/db";\n{\n  let tx = openTx();\n  run(tx, account);\n}\n`,
      option,
    );
    recordDiagnostic(
      `import { openTx } from "@example/db";\n{\n  let tx = openTx();\n  run(tx, account);\n}\n`,
      option,
      "app.ts",
      ["passed"],
    );

    const memberQuery = check(
      `import { openTx } from "@example/db";\n{\n  const tx = openTx();\n  const alias = tx;\n  run(alias.query, account);\n  run(openTx().query, account);\n}\n`,
      option,
    );
    recordDiagnostic(
      `import { openTx } from "@example/db";\n{\n  const tx = openTx();\n  const alias = tx;\n  run(alias.query, account);\n  run(openTx().query, account);\n}\n`,
      option,
      "app.ts",
      ["passed"],
    );

    const aliasedFactory = check(
      `import { openTx } from "@example/db";\n{\n  const tx = openTx();\n  const alias = tx;\n  run(alias, account);\n}\n`,
      option,
    );
    recordDiagnostic(
      `import { openTx } from "@example/db";\n{\n  const tx = openTx();\n  const alias = tx;\n  run(alias, account);\n}\n`,
      option,
      "app.ts",
      ["passed"],
    );

    const factory = check(
      `import { openTx } from "@example/db";\n{\n  let tx = openTx();\n  run((statement) => tx(statement), account);\n}\n`,
      option,
    );
    recordDiagnostic(
      `import { openTx } from "@example/db";\n{\n  let tx = openTx();\n  run((statement) => tx(statement), account);\n}\n`,
      option,
      "app.ts",
      ["forward"],
    );

    assert.match(factory.output, /run\(\(statement\) => tx\(statement\), account\)/);

    recordDiagnostic(
      `import { openTx } from "@example/db";\n{\n  var tx = openTx();\n  run(tx, account);\n}\n`,
      option,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `import { openTx } from "@example/db";\n{\n  var tx = openTx();\n  run((statement) => tx(statement), account);\n}\n`,
      option,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `import { openTx } from "@example/db";\nconst tx = openTx();\nfunction inner(tx: (sql: string) => unknown, account: string) {\n  return run((statement) => tx(statement), account);\n}\n`,
      option,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `import type { Tx } from "@example/db/types";\nfunction load(tx: Tx, account: string) {\n  return run((statement) => tx(statement), account);\n}\n`,
      option,
      "app.ts",
      ["forward"],
    );
    const both = check(
      `import { openTx } from "@example/db";\nconst other = openTx();\nconst tx = openTx();\nrun((statement) => tx(statement), account);\nvoid other;\n`,
      option,
    );
    recordDiagnostic(
      `import { openTx } from "@example/db";\nconst other = openTx();\nconst tx = openTx();\nrun((statement) => tx(statement), account);\nvoid other;\n`,
      option,
      "app.ts",
      ["forward"],
    );

    assert.match(both.output, /run\(\(statement\) => tx\(statement\), account\)/);

    const literal = check(
      `import { "read" as read } from "@example/db";\nrun((statement) => read(statement), account);`,
    );
    recordDiagnostic(
      `import { "read" as read } from "@example/db";\nrun((statement) => read(statement), account);`,
      DB,
      "app.ts",
      ["forward"],
    );

    assert.equal(
      literal.output,
      `import { "read" as read } from "@example/db";\nrun((statement) => read(statement), account);`,
    );
  });

  it("honors eslint and no-mistakes suppression without treating strings as directives", () => {
    recordDiagnostic(
      `${IMPORT}// eslint-disable-next-line no-mistakes/postgres-no-executor-adapter\nrun((statement) => read(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}// no-mistakes-disable-next-line postgres-no-executor-adapter\nrun(read, account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}run(\n  // no-mistakes-disable-next-line postgres-no-executor-adapter\n  (statement) => read(statement),\n  account,\n);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}run(\n  (statement) => read(statement), // no-mistakes-disable-line postgres-no-executor-adapter\n  account,\n);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `// no-mistakes-disable-file postgres-no-executor-adapter\n${IMPORT}run((statement) => read(statement), account);`,
      DB,
      "app.ts",
      [],
    );
    recordDiagnostic(
      `${IMPORT}// no-mistakes-disable-next-line postgres-no-manual-transaction\nrun((statement) => read(statement), account);`,
      DB,
      "app.ts",
      ["forward"],
    );
    recordDiagnostic(
      `${IMPORT}const note = "no-mistakes-disable-next-line postgres-no-executor-adapter";\nrun((statement) => read(statement), account);`,
      DB,
      "app.ts",
      ["forward"],
    );
    recordDiagnostic(
      `/* no-mistakes-disable-file postgres-no-executor-adapter */\n${IMPORT}run((statement) => read(statement), account);`,
      DB,
      "app.ts",
      [],
    );
  });
});

it("snapshots postgres-no-executor-adapter diagnostics", async () => {
  if (recorded.length < 80) throw new Error(`recorded ${recorded.length} adapter diagnostics`);
  const valid = [];
  const invalid = [];
  for (const item of recorded) {
    const entry = {
      code: item.code,
      filename: item.filename,
      options: [item.option],
      languageOptions: { parser: tsParser },
    };
    (item.empty ? valid : invalid).push(entry);
  }
  await expect(
    snapshotDocument(
      snapshotSuite("postgres-no-executor-adapter", languageOptions, { valid, invalid }),
    ),
  ).toMatchFileSnapshot("snapshots/postgres-no-executor-adapter.snap");
}, 30_000);
