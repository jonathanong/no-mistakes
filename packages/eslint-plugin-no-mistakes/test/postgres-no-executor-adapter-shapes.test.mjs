import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { fixture, lint, __dirname, require } from "./helpers.mjs";

const RULE = "postgres-no-executor-adapter";
const id = RULE;
const DB = { importSpecifier: "@example/db" };
const SCOPED = {
  importSpecifier: "@example/db",
  executorFactoryNames: ["openTx"],
  executorTypeNames: ["Tx"],
};
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const rules = resolve(__dirname, "../src/rules");

function check(name, option = DB) {
  return lint(
    fixture(`${id}/${name}`),
    { [`no-mistakes/${RULE}`]: ["error", option] },
    resolve(root, name),
  );
}

function onLine(name, snippet, option = DB) {
  const line =
    fixture(`${id}/${name}`)
      .split("\n")
      .findIndex((row) => row.includes(snippet)) + 1;
  return check(name, option).filter((message) => message.line === line);
}

describe("postgres-no-executor-adapter shapes", () => {
  it("reports the shapes whose rewrite is the executor call", () => {
    assert.equal(check("shapes.ts").length, 8);

    const adapter = onLine("shapes.ts", "run(adapter, account)");
    assert.equal(adapter[0].messageId, "forward");
    assert.match(adapter[0].message, /read\(sql\)/);
    assert.match(adapter[0].message, /const exec = read; exec\(sql\)/);
    assert.match(adapter[0].message, /run\(read, account\)/);
    assert.doesNotMatch(adapter[0].message, /run\(adapter, account\)/);

    const value = onLine("shapes.ts", "new Runner(read)");
    assert.equal(value[0].messageId, "passed");
    assert.match(value[0].message, /read\(sql`…`\)/);

    const arrow = onLine("shapes.ts", "new Runner((statement");
    assert.equal(arrow[0].messageId, "forward");
    assert.match(arrow[0].message, /new Runner\(read\)/);
    assert.match(arrow[0].message, /read\(sql\)/);

    for (const snippet of ["run({ exec: read }", "run([read]", "run(read.bind(null), account)"]) {
      const reported = onLine("shapes.ts", snippet);
      assert.equal(reported.length, 1, snippet);
      assert.equal(reported[0].messageId, "passed");
      assert.match(reported[0].message, /read\(sql`…`\)/);
    }

    const nullish = onLine("shapes.ts", "run(read ?? write, account)");
    assert.equal(nullish[0].messageId, "passed");
    assert.match(nullish[0].message, /read\(sql`…`\)/);
    assert.doesNotMatch(nullish[0].message, /\bwrite\b/);
    assert.doesNotMatch(nullish[0].message, /\?\?/);

    const and = onLine("shapes.ts", "run(flag && read, account)");
    assert.equal(and[0].messageId, "passed");
    assert.match(and[0].message, /read\(sql`…`\)/);
    assert.doesNotMatch(and[0].message, /flag/);
    assert.doesNotMatch(and[0].message, /&&/);
  });

  it("names only the executor that the expression can pass", () => {
    const both = onLine("edges.ts", "run((flag ? read : write).bind(null), account)");
    assert.match(
      both[0].message,
      /read\(sql`…`\) or write\(sql`…`\) or \(flag \? read : write\)\(sql`…`\)/,
    );

    const partial = onLine("edges.ts", "run((flag ? read : other).bind(null), account)");
    assert.match(partial[0].message, /read\(sql`…`\)/);
    assert.doesNotMatch(partial[0].message, /other/);
    assert.doesNotMatch(partial[0].message, /flag \?/);

    const andWrite = onLine("edges.ts", "run(read && write, account)");
    assert.match(andWrite[0].message, /write\(sql`…`\)/);
    assert.doesNotMatch(andWrite[0].message, /\bread\b/);

    const orRead = onLine("edges.ts", "run(read || write, account)");
    assert.match(orRead[0].message, /read\(sql`…`\)/);
    assert.doesNotMatch(orRead[0].message, /\bwrite\b/);

    const query = onLine("edges.ts", "run(client.query ?? read, account)");
    assert.match(query[0].message, /read\(sql`…`\)/);
    assert.doesNotMatch(query[0].message, /client\.query/);

    const kept = onLine("edges.ts", "run(read ?? client.query, account)");
    assert.match(kept[0].message, /read\(sql`…`\)/);
    assert.doesNotMatch(kept[0].message, /client\.query/);

    const mixed = onLine("edges.ts", "run((flag ? read : other) ?? write, account)");
    assert.match(mixed[0].message, /read\(sql`…`\) or write\(sql`…`\)/);
    assert.doesNotMatch(mixed[0].message, /other/);
    assert.doesNotMatch(mixed[0].message, /\?\?/);

    const constructed = onLine("edges.ts", "new Runner(read, (statement");
    assert.deepEqual(
      constructed.map((message) => message.messageId),
      ["passed", "forward"],
    );
    assert.match(constructed[1].message, /new Runner\(read, write\)/);
  });

  it("leaves unjudged shapes silent, including an untyped client.query", () => {
    assert.deepEqual(check("silent.ts"), []);
    assert.deepEqual(
      check("names.ts", { importSpecifier: "@example/db", executorNames: ["read"] }),
      [],
    );
  });

  it("follows a proven query member through bind and new, and a broad query name", () => {
    const scoped = check("scoped.ts", SCOPED);
    assert.deepEqual(
      scoped.map((message) => message.messageId),
      ["passed", "passed"],
    );
    assert.match(scoped[0].message, /tx\.query\(sql`…`\)/);
    assert.doesNotMatch(scoped[0].message, /const exec = tx\.query/);
    assert.match(scoped[1].message, /alias\.query\(sql`…`\)/);
    assert.doesNotMatch(scoped.map((message) => message.message).join("\n"), /openTx\(\)/);

    const broad = check("broad.mjs", { executorNames: ["query"] });
    assert.equal(broad.length, 4);
    assert.ok(broad.every((message) => message.message.includes("client.query")));
  });

  it("keeps the existing suppression and does not autofix", () => {
    assert.equal(check("suppression.ts").length, 1);
    assert.match(check("suppression.ts")[0].message, /passes read/);

    const linter = new Linter({ configType: "flat", cwd: root });
    const code = fixture(`${id}/shapes.ts`);
    const fixed = linter.verifyAndFix(
      code,
      [
        {
          files: ["**/*.{js,jsx,ts,tsx,mjs,mts,cjs,cts}"],
          languageOptions: { ecmaVersion: 2024, sourceType: "module", parser: tsParser },
          plugins: { "no-mistakes": require("../src/index.js") },
          rules: { [`no-mistakes/${RULE}`]: ["error", DB] },
        },
      ],
      resolve(root, "shapes.ts"),
    );
    assert.equal(fixed.output, code);
  });

  it("imports binding helpers without a ref and member cycle", () => {
    const ref = readFileSync(resolve(rules, "postgres-no-executor-adapter-ref.js"), "utf8");
    const member = readFileSync(resolve(rules, "postgres-no-executor-adapter-member.js"), "utf8");
    const binding = readFileSync(resolve(rules, "postgres-no-executor-adapter-binding.js"), "utf8");
    const body = ref.slice(ref.indexOf("function executorRef"));
    assert.equal(body.includes("require("), false);
    assert.match(ref, /require\("\.\/postgres-no-executor-adapter-binding"\)/);
    assert.match(ref, /require\("\.\/postgres-no-executor-adapter-member"\)/);
    assert.doesNotMatch(member, /postgres-no-executor-adapter-ref/);
    assert.doesNotMatch(binding, /postgres-no-executor-adapter-ref/);
    assert.doesNotMatch(binding, /postgres-no-executor-adapter-member/);
  });
});
