import assert from "node:assert/strict";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { fixture, lint, plugin, __dirname, require } from "./helpers.mjs";

const RULE = "postgres-no-nonliteral-sql-registry-key";
const id = RULE;
const DB = { importSpecifier: "@example/db" };
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);

function check(name, option = DB) {
  return lint(
    fixture(`${id}/${name}`),
    { [`no-mistakes/${RULE}`]: ["error", option] },
    resolve(root, name),
  );
}

describe("postgres-no-nonliteral-sql-registry-key", () => {
  it("stays out of the presets and does not autofix", () => {
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.rules[RULE].meta.fixable, undefined);
    assert.equal(plugin.rules[RULE].meta.hasSuggestions, undefined);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
  });

  it("names each registry entry in a throwing switch", () => {
    const messages = check("dynamic.ts");
    assert.equal(messages.length, 3);
    assert.ok(messages.every((message) => message.messageId === "dynamicKey"));
    assert.match(messages[0].message, /TABLES\[key\]/);
    assert.match(messages[0].message, /case "orders": return "orders";/);
    assert.match(messages[0].message, /case "accounts": return "accounts";/);
    assert.match(messages[0].message, /default: throw new Error\("unknown"\)/);
    assert.match(messages[0].message, /own-key check is not judged/);
    assert.match(messages[2].message, /case "order items": return "order items";/);
  });

  it("ignores literal keys, mutated registries, and lookups outside the executor", () => {
    assert.equal(check("silent.ts").length, 0);
    assert.equal(check("dynamic.ts", { executorNames: [] }).length, 0);
  });

  it("keeps a disable comment and does not rewrite the file", () => {
    assert.equal(check("suppression.ts").length, 1);
    const linter = new Linter({ configType: "flat", cwd: root });
    const code = fixture(`${id}/dynamic.ts`);
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
      resolve(root, "dynamic.ts"),
    );
    assert.equal(fixed.output, code);
  });
});
