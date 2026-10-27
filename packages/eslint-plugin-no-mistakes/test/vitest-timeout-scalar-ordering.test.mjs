import { expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const rule = "vitest-timeout-cap";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function capCase(options) {
  return {
    code: fixture("vitest-timeout-cap/copy-scalar-ordering.ts"),
    filename: "fixture.test.ts",
    languageOptions: { parser: tsParser },
    options: [options],
  };
}

// Earlier scalar coercion stays visible. Uncertain write ordering stays unknown.
const suite = {
  invalid: [
    // 1 timeout, 31 unknown.
    capCase({ unknownValues: "finding" }),
    // 1 timeout.
    capCase({}),
  ],
};

it("snapshots vitest-timeout-scalar-ordering diagnostics", async () => {
  await expect(
    snapshotDocument({
      [rule]: snapshotSuite(rule, languageOptions, suite),
    }),
  ).toMatchFileSnapshot("snapshots/vitest-timeout-scalar-ordering.snap");
}, 30_000);
