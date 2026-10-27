import assert from "node:assert/strict";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, plugin, __dirname, require } from "./helpers.mjs";

const RULE = "postgres-no-correlated-sql-builder";
const id = RULE;
const OPTIONS = { builderMethods: ["append"], tagNames: ["sql"] };
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function correlatedFile(name, option = OPTIONS) {
  return {
    code: fixture(`${id}/${name}`),
    filename: resolve(root, name),
    options: [option],
    languageOptions: { parser: tsParser },
  };
}

const suite = {
  valid: [
    // reports repeated appends of one selector and leaves a single guard alone.
    // Zero findings.
    correlatedFile("silent.ts"),
    // abstains on complete throwing chains and returning guards.
    // Zero findings.
    correlatedFile("complete-guards.ts"),
    // reports static keys, member tags, call forwarding, and a one-step forwarder.
    // Zero findings.
    correlatedFile("static-call-limits.ts"),
    // is inert until both builder methods and tag names are configured.
    // Zero findings.
    {
      code: "export function f() {}",
      filename: "fixture.jsx",
      options: [{}],
    },
    {
      code: "export function f() {}",
      filename: "fixture.jsx",
      options: [{ builderMethods: ["append"] }],
    },
    {
      code: "export function f() {}",
      filename: "fixture.jsx",
      options: [{ tagNames: ["sql"] }],
    },
  ],
  invalid: [
    // reports repeated appends of one selector and leaves a single guard alone.
    // Four correlated findings.
    correlatedFile("repeated.ts"),
    // One correlated finding.
    correlatedFile("suppression.ts"),
    // abstains on complete throwing chains and returning guards.
    // 14 correlated findings.
    correlatedFile("incomplete-guards.ts"),
    // reports static keys, member tags, call forwarding, and a one-step forwarder.
    // Nine correlated findings.
    correlatedFile("static-calls.ts"),
    // One correlated finding.
    correlatedFile("call-method.ts", { builderMethods: ["call"], tagNames: ["sql"] }),
  ],
};

it("snapshots postgres-no-correlated-sql-builder diagnostics", async () => {
  await expect(snapshotDocument(snapshotSuite(RULE, languageOptions, suite))).toMatchFileSnapshot(
    "snapshots/postgres-no-correlated-sql-builder.snap",
  );
}, 30_000);

describe("postgres-no-correlated-sql-builder", () => {
  it("stays out of the presets and does not autofix", () => {
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.rules[RULE].meta.fixable, undefined);
    assert.equal(plugin.rules[RULE].meta.hasSuggestions, undefined);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
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
