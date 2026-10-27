import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function correlatedCase(name, builderMethods = ["append"]) {
  return {
    code: fixture(`postgres-no-correlated-sql-builder/${name}.ts`),
    filename: "fixture.ts",
    options: [{ builderMethods, tagNames: ["sql"] }],
    languageOptions: { parser: tsParser },
  };
}

const suite = {
  valid: [
    // keeps unknown arguments, mutable bindings and split receivers silent.
    // Zero findings.
    correlatedCase("forwarded-method-controls"),
    // honors suppression on the guarded append.
    // Zero findings.
    correlatedCase("forwarded-method-suppression"),
    correlatedCase("forwarded-method-file-suppression"),
  ],
  invalid: [
    // retains apply, const aliases, comma methods and transparent wrappers.
    // Five correlated findings.
    correlatedCase("forwarded-methods"),
    // keeps every builder per guard without counting one guard twice.
    // Four correlated findings.
    correlatedCase("forwarded-method-multiple-builders"),
    // prioritizes configured methods named apply or call.
    // Two correlated findings.
    correlatedCase("forwarded-method-priority", ["apply", "call"]),
  ],
};

it("snapshots postgres-no-correlated-sql-builder diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-correlated-sql-builder", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-correlated-forwarded-methods.snap");
}, 30_000);
