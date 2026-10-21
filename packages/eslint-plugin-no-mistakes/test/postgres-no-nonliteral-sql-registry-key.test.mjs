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

  it("recognizes a finite-key cast only for the same registry", () => {
    assert.equal(check("finite-key.ts").length, 0);
    assert.equal(check("other-key.ts").length, 4);
  });

  it("follows one const SQL argument without duplicate reports", () => {
    assert.equal(check("bound-sql.ts").length, 2);
    assert.equal(check("bound-silent.ts").length, 0);
  });

  it("emits a complete switch expression with deduplicated final values", () => {
    const [message] = check("duplicate-keys.ts");
    assert.match(message.message, /switch \(key\) \{/);
    assert.equal(message.message.match(/case "orders"/g).length, 1);
    assert.ok(message.message.includes(JSON.stringify('order "items"')));
    assert.ok(message.message.includes('return "{{accounts}}";'));
    const expression = message.message.slice(
      message.message.indexOf("((key) => {"),
      message.message.indexOf("). A runtime"),
    );
    const select = new Function("kind", `return ${expression};`);
    assert.equal(select("orders"), 'order "items"');
    assert.equal(select("accounts"), "{{accounts}}");
    assert.throws(() => select("other"), /unknown/);
  });

  it("keeps await and yield selectors in their original evaluation context", async () => {
    const messages = check("selector-contexts.ts");
    assert.equal(messages.length, 3);
    const expressions = messages.map(({ message }) =>
      message.slice(message.indexOf("((key) => {"), message.indexOf("). A runtime")),
    );
    const awaited = new Function("choose", `return (async () => ${expressions[0]})();`);
    assert.equal(await awaited(async () => "orders"), "orders");
    const iterator = new Function(`return (function* () { return ${expressions[1]}; })();`)();
    assert.deepEqual(iterator.next(), { value: "key", done: false });
    assert.deepEqual(iterator.next("accounts"), { value: "accounts", done: true });
    let calls = 0;
    const sequence = new Function("chooseSync", "kind", `return ${expressions[2]};`);
    assert.equal(
      sequence(() => {
        calls++;
      }, "orders"),
      "orders",
    );
    assert.equal(calls, 1);
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
