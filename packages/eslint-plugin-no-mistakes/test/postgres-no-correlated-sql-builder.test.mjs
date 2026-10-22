import assert from "node:assert/strict";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { fixture, lint, messages, plugin, __dirname, require } from "./helpers.mjs";

const RULE = "postgres-no-correlated-sql-builder";
const id = RULE;
const OPTIONS = { builderMethods: ["append"], tagNames: ["sql"] };
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);

function check(name, option = OPTIONS) {
  return lint(
    fixture(`${id}/${name}`),
    { [`no-mistakes/${RULE}`]: ["error", option] },
    resolve(root, name),
  );
}

describe("postgres-no-correlated-sql-builder", () => {
  it("stays out of the presets and does not autofix", () => {
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.rules[RULE].meta.fixable, undefined);
    assert.equal(plugin.rules[RULE].meta.hasSuggestions, undefined);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
  });

  it("reports repeated appends of one selector and leaves a single guard alone", () => {
    const reported = check("repeated.ts");
    assert.equal(reported.length, 4);
    assert.ok(reported.every((message) => message.messageId === "correlated"));
    assert.match(reported[0].message, /appends on query/);
    assert.match(reported[0].message, /empty path for access/);
    assert.match(reported[0].message, /throwing default/);
    assert.match(reported[0].message, /case "owned": case "member":/);
    assert.match(reported[0].message, /own-key check/);
    assert.equal(check("silent.ts").length, 0);
    assert.equal(check("suppression.ts").length, 1);
  });

  it("abstains on complete throwing chains and returning guards", () => {
    assert.equal(check("complete-guards.ts").length, 0);
    assert.equal(check("incomplete-guards.ts").length, 14);
  });

  it("reports static keys, member tags, call forwarding, and a one-step forwarder", () => {
    assert.equal(check("static-calls.ts").length, 8);
    assert.equal(
      check("call-method.ts", { builderMethods: ["call"], tagNames: ["sql"] }).length,
      1,
    );
    assert.equal(check("static-call-limits.ts").length, 0);
  });

  it("is inert until both builder methods and tag names are configured", () => {
    assert.deepEqual(messages("export function f() {}", RULE, {}), []);
    assert.deepEqual(messages("export function f() {}", RULE, { builderMethods: ["append"] }), []);
    assert.deepEqual(messages("export function f() {}", RULE, { tagNames: ["sql"] }), []);
  });

  it("does not rewrite the file", () => {
    const linter = new Linter({ configType: "flat", cwd: root });
    const code = fixture(`${id}/repeated.ts`);
    const fixed = linter.verifyAndFix(
      code,
      [
        {
          files: ["**/*.{js,jsx,ts,tsx,mjs,mts,cjs,cts}"],
          languageOptions: { ecmaVersion: 2024, sourceType: "module", parser: tsParser },
          plugins: { "no-mistakes": require("../src/index.js") },
          rules: { [`no-mistakes/${RULE}`]: ["error", OPTIONS] },
        },
      ],
      resolve(root, "repeated.ts"),
    );
    assert.equal(fixed.output, code);
  });
});
