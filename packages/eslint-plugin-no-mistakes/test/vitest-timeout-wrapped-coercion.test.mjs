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
    // Overwritten, shared, callable, getter, cyclic and unproved paths are silent
    // until the finding policy asks for them.
    capCase("wrapped-copy-coercion-unsafe"),
  ],
  invalid: [
    // 1 timeout, 1 invalid. Literal object and array wrappers keep numeric and null updates.
    capCase("wrapped-copy-coercion", finding),
    // 1 timeout, 1 invalid.
    capCase("wrapped-copy-coercion"),
    // 39 unknown.
    capCase("wrapped-copy-coercion-unsafe", finding),
  ],
};

describe("wrapped copied scalar coercion", () => {
  it("snapshots vitest-timeout-wrapped-coercion diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/vitest-timeout-wrapped-coercion.snap");
  }, 30_000);
});
