import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const rule = "vitest-timeout-cap";
const finding = { unknownValues: "finding" };
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function capCase(name, options = {}) {
  return {
    code: fixture(`vitest-timeout-cap/${name}.ts`),
    filename: "fixture.test.ts",
    languageOptions: { parser: tsParser },
    options: [options],
  };
}

const suite = {
  valid: [
    // Scalar controls stay in effect when unknown values are ignored.
    capCase("copy-reverse-alias-overwrites"),
    capCase("copy-reverse-alias-source-mutation"),
    capCase("copy-alias-guard-composition"),
    // An absent first alias initializer does not invent a finding by itself.
    capCase("copy-alias-ambient"),
  ],
  invalid: [
    // 36 unknown. Original, forward, sibling and chained writes.
    capCase("copy-reverse-alias-overwrites", finding),
    // 6 unknown. Shared coercion stays unknown.
    capCase("copy-reverse-alias-source-mutation", finding),
    // 10 unknown. Consumer and ordering guards survive every alias direction.
    capCase("copy-alias-guard-composition", finding),
    // 2 unknown.
    capCase("copy-alias-ambient", finding),
  ],
};

describe("vitest-timeout-cap reverse copied aliases", () => {
  it("snapshots vitest-timeout-reverse-alias diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/vitest-timeout-reverse-alias.snap");
  }, 30_000);

  it("retains unknown shared coercion and verifies its actual mutation of both sources", async () => {
    const { observedTimeouts } =
      await import("../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap/copy-reverse-alias-source-mutation.ts");
    assert.deepEqual(observedTimeouts, [30_001, 60_000]);
  });
});
