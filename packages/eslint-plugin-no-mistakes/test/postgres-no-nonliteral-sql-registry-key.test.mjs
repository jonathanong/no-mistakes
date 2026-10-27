import assert from "node:assert/strict";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, lint, plugin, __dirname, require } from "./helpers.mjs";

const RULE = "postgres-no-nonliteral-sql-registry-key";
const id = RULE;
const DB = { importSpecifier: "@example/db" };
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function check(name, option = DB) {
  return lint(
    fixture(`${id}/${name}`),
    { [`no-mistakes/${RULE}`]: ["error", option] },
    resolve(root, name),
  );
}

function registryCase(name, option = DB) {
  return {
    code: fixture(`${id}/${name}`),
    filename: resolve(root, name),
    options: [option],
    languageOptions: { parser: tsParser },
  };
}

const suite = {
  valid: [
    // recognizes a finite-key cast only for the same registry.
    // Zero findings.
    registryCase("finite-key.ts"),
    // follows one const SQL argument without duplicate reports.
    // Zero findings.
    registryCase("bound-silent.ts"),
    // ignores literal keys, mutated registries, and lookups outside the executor.
    // Zero findings.
    registryCase("silent.ts"),
    // Zero findings.
    registryCase("dynamic.ts", { executorNames: [] }),
  ],
  invalid: [
    // names each registry entry in a throwing switch.
    // Three dynamicKey findings.
    registryCase("dynamic.ts"),
    // Four findings.
    registryCase("other-key.ts"),
    // Two findings.
    registryCase("bound-sql.ts"),
    // suppresses a call or lookup without hiding another use of a shared initializer.
    // One dynamicKey finding.
    registryCase("bound-suppression.ts"),
    // emits a complete switch expression with deduplicated final values.
    registryCase("duplicate-keys.ts"),
    // keeps await and yield selectors in their original evaluation context.
    // Three findings.
    registryCase("selector-contexts.ts"),
    // keeps a disable comment.
    // One finding.
    registryCase("suppression.ts"),
  ],
};

it("snapshots postgres-no-nonliteral-sql-registry-key diagnostics", async () => {
  await expect(snapshotDocument(snapshotSuite(RULE, languageOptions, suite))).toMatchFileSnapshot(
    "snapshots/postgres-no-nonliteral-sql-registry-key.snap",
  );
}, 30_000);

describe("postgres-no-nonliteral-sql-registry-key", () => {
  it("stays out of the presets and does not autofix", () => {
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.rules[RULE].meta.fixable, undefined);
    assert.equal(plugin.rules[RULE].meta.hasSuggestions, undefined);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
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

  it("does not rewrite the file", () => {
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
