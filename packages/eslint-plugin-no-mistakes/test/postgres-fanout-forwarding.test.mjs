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

function fanoutCase(name, executorNames) {
  return {
    code: fixture(`postgres-executor-recall/${name}.ts`),
    filename: "fixture.ts",
    options: [{ importSpecifier: "@example/db", ...(executorNames && { executorNames }) }],
    languageOptions: { parser: tsParser },
  };
}

const suite = {
  valid: [
    // checks call/apply on Promise.all and map with the actual collection receiver.
    // Zero findings.
    fanoutCase("fanout-forwarding", []),
    // preserves line, next-line, and file suppression.
    // Zero findings.
    fanoutCase("fanout-forwarding-suppressed"),
    fanoutCase("fanout-forwarding-file"),
    // leaves bounded receivers and unknown forwarding arguments alone.
    // Zero findings.
    fanoutCase("fanout-forwarding-silent"),
  ],
  invalid: [
    // checks call/apply on Promise.all and map with the actual collection receiver.
    // 15 unboundedFanout findings.
    fanoutCase("fanout-forwarding"),
  ],
};

it("snapshots postgres-no-unbounded-query-fanout diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-unbounded-query-fanout", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-fanout-forwarding.snap");
}, 30_000);
