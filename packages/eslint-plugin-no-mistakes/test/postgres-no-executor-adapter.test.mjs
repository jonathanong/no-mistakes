import assert from "node:assert/strict";
import { describe, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { messages, require } from "./helpers.mjs";

const RULE = "postgres-no-executor-adapter";
const DB = { importSpecifier: "@example/db" };

const IMPORT = `import { read, write } from "@example/db";\n`;

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

function ids(code, option = DB, filename = "app.ts") {
  return messages(code, RULE, option, filename);
}

describe("postgres-no-executor-adapter", () => {
  it("reports an exact forwarder and names the judged call instead of rewriting it", () => {
    const arrow = check(`${IMPORT}run((statement) => read(statement), account);`);
    assert.deepEqual(arrow.ids, ["forward"]);
    assert.equal(arrow.output, `${IMPORT}run((statement) => read(statement), account);`);
    assert.match(arrow.messages[0], /read\(sql\)/);
    assert.match(arrow.messages[0], /const exec = read; exec\(sql\)/);
    assert.match(arrow.messages[0], /run\(read, account\)/);
    assert.match(arrow.messages[0], /does not connect/);

    const fn = check(`${IMPORT}run(function (statement) { return read(statement); }, account);`);
    assert.deepEqual(fn.ids, ["forward"]);
    assert.equal(
      fn.output,
      `${IMPORT}run(function (statement) { return read(statement); }, account);`,
    );
    assert.match(fn.messages[0], /read\(sql\)/);

    const mapped = check(`${IMPORT}items.map((statement) => read(statement));`);
    assert.deepEqual(mapped.ids, ["forward"]);
    assert.equal(mapped.output, `${IMPORT}items.map((statement) => read(statement));`);
    assert.match(mapped.messages[0], /items\.map\(read\)/);

    const zero = check(`${IMPORT}run(() => read());`);
    assert.deepEqual(zero.ids, ["forward"]);
    assert.equal(zero.output, `${IMPORT}run(() => read());`);

    const pair = check(`${IMPORT}run((statement, values) => read(statement, values), account);`);
    assert.deepEqual(pair.ids, ["forward"]);
    assert.equal(
      pair.output,
      `${IMPORT}run((statement, values) => read(statement, values), account);`,
    );
    assert.match(pair.messages[0], /read\(sql\)/);
  });

  it("reports conditional executor selection without rewriting it", () => {
    const code = `${IMPORT}run(flag ? read : write, account);`;
    const result = check(code);
    assert.deepEqual(result.ids, ["blocked"]);
    assert.equal(result.output, code);
    assert.match(result.messages[0], /conditional selection/);
    assert.match(
      result.messages[0],
      /read\(sql\) or write\(sql\) or \(flag \? read : write\)\(sql\)/,
    );
    assert.match(result.messages[0], /not a judged rewrite/);
    assert.match(result.messages[0], /cannot check SQL/);

    const nested = check(`${IMPORT}run(flag ? read : (other ? write : read), account);`);
    assert.deepEqual(nested.ids, ["blocked"]);
    assert.match(nested.messages[0], /\(flag \? read : \(other \? write : read\)\)\(sql\)/);
    assert.equal(nested.output, `${IMPORT}run(flag ? read : (other ? write : read), account);`);

    const inside = check(`${IMPORT}run((statement) => (flag ? read : write)(statement), account);`);
    assert.deepEqual(inside.ids, ["blocked"]);
    assert.match(inside.messages[0], /conditional selection/);
    assert.equal(
      inside.output,
      `${IMPORT}run((statement) => (flag ? read : write)(statement), account);`,
    );

    const returned = check(
      `${IMPORT}run((statement) => (flag ? read(statement) : write(statement)), account);`,
    );
    assert.deepEqual(returned.ids, ["blocked"]);
    assert.match(returned.messages[0], /conditional selection/);
    assert.equal(
      returned.output,
      `${IMPORT}run((statement) => (flag ? read(statement) : write(statement)), account);`,
    );

    const mixed = check(`${IMPORT}run(flag ? client.query : read, account);`);
    assert.deepEqual(mixed.ids, ["blocked"]);
    assert.match(mixed.messages[0], /client\.query\(sql\) or read\(sql\)/);
    assert.doesNotMatch(mixed.messages[0], /\(flag \? client\.query : read\)\(sql\)/);
    assert.equal(mixed.output, `${IMPORT}run(flag ? client.query : read, account);`);

    assert.deepEqual(ids(`${IMPORT}run(flag ? read : other, account);`), []);
    assert.deepEqual(
      ids(`${IMPORT}run((statement) => (flag ? read(statement) : normalize(statement)), account);`),
      [],
    );
  });

  it("does not autofix a different receiver, this, arity, rest, or side effect", () => {
    const member = check(`${IMPORT}run((statement) => client.query(statement), account);`);
    assert.deepEqual(member.ids, ["blocked"]);
    assert.match(member.messages[0], /receiver or this/);
    assert.match(member.messages[0], /client\.query\(sql\)/);
    assert.doesNotMatch(member.messages[0], /const exec = client\.query/);
    assert.match(member.messages[0], /run\(client\.query, account\)/);
    assert.equal(member.output, `${IMPORT}run((statement) => client.query(statement), account);`);

    const computed = check(`${IMPORT}run((statement) => client["query"](statement), account);`);
    assert.deepEqual(computed.ids, ["blocked"]);
    assert.match(computed.messages[0], /client\["query"\]/);

    const template = check(`${IMPORT}run((statement) => client[\`query\`](statement), account);`);
    assert.deepEqual(template.ids, ["blocked"]);

    const call = check(`${IMPORT}run((statement) => read.call(receiver, statement), account);`);
    assert.deepEqual(call.ids, ["blocked"]);
    assert.match(call.messages[0], /receiver or this/);
    assert.equal(
      call.output,
      `${IMPORT}run((statement) => read.call(receiver, statement), account);`,
    );

    const apply = check(`${IMPORT}run((statement) => read.apply(receiver, [statement]), account);`);
    assert.deepEqual(apply.ids, ["blocked"]);
    assert.match(apply.messages[0], /receiver or this/);
    assert.equal(
      apply.output,
      `${IMPORT}run((statement) => read.apply(receiver, [statement]), account);`,
    );

    const applySpread = check(
      `${IMPORT}run((...args) => read.apply(receiver, [...args]), account);`,
    );
    assert.deepEqual(applySpread.ids, ["blocked"]);

    const applyHole = check(
      `${IMPORT}run((statement) => read.apply(receiver, [statement, , extra]), account);`,
    );
    assert.deepEqual(applyHole.ids, ["blocked"]);

    const applyIdent = check(
      `${IMPORT}run((statement) => read.apply(receiver, statement), account);`,
    );
    assert.deepEqual(applyIdent.ids, ["blocked"]);

    assert.deepEqual(ids(`${IMPORT}run((statement) => read.call(statement), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run((statement) => read.apply(statement), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run((statement) => other.call(null, statement), account);`), []);
    const conditionalCall = check(
      `${IMPORT}run((statement) => (flag ? read : write).call(null, statement), account);`,
    );
    assert.deepEqual(conditionalCall.ids, ["blocked"]);
    assert.match(conditionalCall.messages[0], /receiver or this/);
    assert.match(conditionalCall.messages[0], /\(flag \? read : write\)\(sql\)/);
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
      assert.deepEqual(result.ids, ["blocked"], body);
      assert.match(result.messages[0], /arity or rest/, body);
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
      assert.deepEqual(result.ids, ["blocked"], body);
      assert.match(result.messages[0], /side effects/, body);
      assert.equal(result.output, `${IMPORT}run(${body}, account);`, body);
    }
  });

  it("does not autofix async, generators, await, or type assertions", () => {
    const asyncArrow = check(`${IMPORT}run(async (statement) => read(statement), account);`);
    assert.deepEqual(asyncArrow.ids, ["blocked"]);
    assert.match(asyncArrow.messages[0], /async/);
    assert.equal(asyncArrow.output, `${IMPORT}run(async (statement) => read(statement), account);`);

    const awaited = check(`${IMPORT}run(async (statement) => await read(statement), account);`);
    assert.deepEqual(awaited.ids, ["blocked"]);
    assert.match(awaited.messages[0], /async/);

    const generator = check(
      `${IMPORT}run(function* (statement) { return read(statement); }, account);`,
    );
    assert.deepEqual(generator.ids, ["blocked"]);
    assert.match(generator.messages[0], /async/);

    const asserted = check(`${IMPORT}run((statement) => read(statement) as Result, account);`);
    assert.deepEqual(asserted.ids, ["blocked"]);
    assert.match(asserted.messages[0], /type assertion/);
    assert.equal(
      asserted.output,
      `${IMPORT}run((statement) => read(statement) as Result, account);`,
    );

    const argAsserted = check(`${IMPORT}run((statement) => read(statement as string), account);`);
    assert.deepEqual(argAsserted.ids, ["blocked"]);
    assert.match(argAsserted.messages[0], /type assertion/);

    const wrapped = check(`${IMPORT}run(((statement) => read(statement)) as Executor, account);`);
    assert.deepEqual(wrapped.ids, ["blocked"]);
    assert.match(wrapped.messages[0], /type assertion/);
    assert.equal(
      wrapped.output,
      `${IMPORT}run(((statement) => read(statement)) as Executor, account);`,
    );

    const wrappedConditional = check(`${IMPORT}run((flag ? read : write) as Executor, account);`);
    assert.deepEqual(wrappedConditional.ids, ["blocked"]);
    assert.match(wrappedConditional.messages[0], /conditional selection/);

    const wrappedAsserted = check(
      `${IMPORT}run(((statement) => read(statement) as Result) as Executor, account);`,
    );
    assert.deepEqual(wrappedAsserted.ids, ["blocked"]);
    assert.match(wrappedAsserted.messages[0], /type assertion/);
    assert.equal(
      wrappedAsserted.output,
      `${IMPORT}run(((statement) => read(statement) as Result) as Executor, account);`,
    );
  });

  it("treats this on an otherwise exact forwarder as a different receiver", () => {
    const code = `${IMPORT}function load(account: string) {\n  return run(function (statement) {\n    return read(statement) as typeof this;\n  }, account);\n}\n`;
    const result = check(code);
    assert.deepEqual(result.ids, ["blocked"]);
    assert.match(result.messages[0], /receiver or this|type assertion/);
    assert.equal(result.output, code);

    const nested = check(
      `${IMPORT}run(function (statement) {\n  function inner() { return this; }\n  return read(statement);\n}, account);`,
    );
    assert.deepEqual(nested.ids, ["blocked"]);
    assert.match(nested.messages[0], /side effects/);
    assert.equal(
      nested.output,
      `${IMPORT}run(function (statement) {\n  function inner() { return this; }\n  return read(statement);\n}, account);`,
    );

    const classThis = check(
      `${IMPORT}run(function (statement) {\n  const Nested = class { method() { return this; } };\n  return read(statement);\n}, account);`,
    );
    assert.deepEqual(classThis.ids, ["blocked"]);
    assert.match(classThis.messages[0], /side effects/);

    const arrowThis = check(
      `${IMPORT}run(function (statement) {\n  const nested = () => this;\n  return read(statement);\n}, account);`,
    );
    assert.deepEqual(arrowThis.ids, ["blocked"]);
    assert.match(arrowThis.messages[0], /side effects/);
  });

  it("ignores shadowed selectors, unrelated objects, and transforming callbacks", () => {
    assert.deepEqual(
      ids(
        `${IMPORT}function load(read: (sql: string) => unknown, account: string) {\n  return run((statement) => read(statement), account);\n}\n`,
      ),
      [],
    );
    assert.deepEqual(
      ids(`import { read } from "@other/db";\nrun((statement) => read(statement), account);`),
      [],
    );
    assert.deepEqual(
      ids(
        `import { read } from "@example/db/read";\nrun((statement) => read(statement), account);`,
      ),
      [],
    );
    assert.deepEqual(
      ids(`import read from "@example/db";\nrun((statement) => read(statement), account);`),
      [],
    );
    assert.deepEqual(
      ids(`import * as db from "@example/db";\nrun((statement) => db.read(statement), account);`),
      [],
    );
    assert.deepEqual(
      ids(
        `import type { read } from "@example/db";\ndeclare function read(sql: string): unknown;\nrun((statement) => read(statement), account);`,
      ),
      [],
    );
    assert.deepEqual(ids(`${IMPORT}run((statement) => read(normalize(statement)), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run(({ statement }) => read(statement), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run((statement) => read(), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run((statement) => read("SELECT 1"), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run((statement) => () => read(statement), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run((statement) => read?.(statement), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run((statement) => read(statement?.trim()), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run(read?.(statement), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run(flag ? (client?.query as Executor) : read, account);`), []);
    assert.deepEqual(ids(`${IMPORT}run((statement) => flag && read(statement), account);`), []);
    assert.deepEqual(ids(`${IMPORT}run(read, account);`), []);
    assert.deepEqual(ids(`${IMPORT}run(client.query, account);`), []);
    assert.deepEqual(ids(`${IMPORT}(flag ? read : write)("SELECT 1");`), []);
    assert.deepEqual(ids(`${IMPORT}const exec = flag ? read : write;\nrun(exec, account);`), []);
    assert.deepEqual(
      ids(`${IMPORT}run((statement) => { if (flag) return read(statement); }, account);`),
      [],
    );
    assert.deepEqual(ids(`${IMPORT}run((statement) => { return; }, account);`), []);
    assert.deepEqual(ids(`${IMPORT}run(() => { }, account);`), []);
  });

  it("matches renamed imports and configured names without a module", () => {
    const renamed = check(
      `import { write as save } from "@example/db";\nrun((statement) => save(statement), account);`,
    );
    assert.deepEqual(renamed.ids, ["forward"]);
    assert.match(renamed.messages[0], /save\(sql\)/);
    assert.match(renamed.messages[0], /const exec = save; exec\(sql\)/);
    assert.equal(
      renamed.output,
      `import { write as save } from "@example/db";\nrun((statement) => save(statement), account);`,
    );

    const namesOnly = { executorNames: ["read"] };
    assert.deepEqual(
      ids(
        `import { read } from "@example/db";\nrun((statement) => read(statement), account);`,
        namesOnly,
      ),
      ["forward"],
    );
    assert.deepEqual(ids(`run((statement) => client.query(statement), account);`, namesOnly), []);

    const withModule = { importSpecifier: "@example/db", executorNames: ["read"] };
    assert.deepEqual(
      ids(`${IMPORT}run((statement) => client.query(statement), account);`, withModule),
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
    const typed = check(
      `import type { Tx } from "@example/db";\nfunction load(tx: Tx, account: string) {\n  return run((statement) => tx(statement), account);\n}\n`,
      option,
    );
    assert.deepEqual(typed.ids, ["forward"]);
    assert.match(typed.messages[0], /tx\(sql\)/);
    assert.doesNotMatch(typed.messages[0], /const exec = tx/);
    assert.equal(
      typed.output,
      `import type { Tx } from "@example/db";\nfunction load(tx: Tx, account: string) {\n  return run((statement) => tx(statement), account);\n}\n`,
    );

    const factory = check(
      `import { openTx } from "@example/db";\n{\n  let tx = openTx();\n  run((statement) => tx(statement), account);\n}\n`,
      option,
    );
    assert.deepEqual(factory.ids, ["forward"]);
    assert.match(factory.messages[0], /tx\(sql\)/);
    assert.doesNotMatch(factory.messages[0], /const exec = tx/);
    assert.match(factory.output, /run\(\(statement\) => tx\(statement\), account\)/);

    assert.deepEqual(
      ids(
        `import { openTx } from "@example/db";\n{\n  var tx = openTx();\n  run((statement) => tx(statement), account);\n}\n`,
        option,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `import { openTx } from "@example/db";\nconst tx = openTx();\nfunction inner(tx: (sql: string) => unknown, account: string) {\n  return run((statement) => tx(statement), account);\n}\n`,
        option,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `import type { Tx } from "@example/db/types";\nfunction load(tx: Tx, account: string) {\n  return run((statement) => tx(statement), account);\n}\n`,
        option,
      ),
      ["forward"],
    );
    const both = check(
      `import { openTx } from "@example/db";\nconst other = openTx();\nconst tx = openTx();\nrun((statement) => tx(statement), account);\nvoid other;\n`,
      option,
    );
    assert.deepEqual(both.ids, ["forward"]);
    assert.match(both.messages[0], /tx\(sql\)/);
    assert.match(both.output, /run\(\(statement\) => tx\(statement\), account\)/);

    const literal = check(
      `import { "read" as read } from "@example/db";\nrun((statement) => read(statement), account);`,
    );
    assert.deepEqual(literal.ids, ["forward"]);
    assert.match(literal.messages[0], /read\(sql\)/);
    assert.equal(
      literal.output,
      `import { "read" as read } from "@example/db";\nrun((statement) => read(statement), account);`,
    );
  });

  it("honors eslint and no-mistakes suppression without treating strings as directives", () => {
    assert.deepEqual(
      ids(
        `${IMPORT}// eslint-disable-next-line no-mistakes/postgres-no-executor-adapter\nrun((statement) => read(statement), account);`,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `${IMPORT}run(\n  // no-mistakes-disable-next-line postgres-no-executor-adapter\n  (statement) => read(statement),\n  account,\n);`,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `${IMPORT}run(\n  (statement) => read(statement), // no-mistakes-disable-line postgres-no-executor-adapter\n  account,\n);`,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `// no-mistakes-disable-file postgres-no-executor-adapter\n${IMPORT}run((statement) => read(statement), account);`,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `${IMPORT}// no-mistakes-disable-next-line postgres-no-manual-transaction\nrun((statement) => read(statement), account);`,
      ),
      ["forward"],
    );
    assert.deepEqual(
      ids(
        `${IMPORT}const note = "no-mistakes-disable-next-line postgres-no-executor-adapter";\nrun((statement) => read(statement), account);`,
      ),
      ["forward"],
    );
    assert.deepEqual(
      ids(
        `/* no-mistakes-disable-file postgres-no-executor-adapter */\n${IMPORT}run((statement) => read(statement), account);`,
      ),
      [],
    );
  });
});
