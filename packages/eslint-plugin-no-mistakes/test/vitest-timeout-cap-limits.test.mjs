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
    // Helper parameters stay unknown without substituting caller arguments.
    capCase("limits-parameter"),
    // A default import or another file's runner re-export is not this runner.
    capCase("limits-runner-identity"),
    capCase("limits-runner-identity", finding),
    // The immutable budget rewrite is within the default cap.
    capCase("limits-budget-rewrite"),
    capCase("limits-budget-rewrite", finding),
  ],
  invalid: [
    // 1 unknown. Helper parameters are not caller arguments.
    capCase("limits-parameter", finding),
    // 1 timeout. The rewrite is judged against overrideMax.
    capCase("limits-budget-rewrite", { overrideMax: 3_000 }),
  ],
};

describe("vitest-timeout-cap documented limits", () => {
  it("snapshots vitest-timeout-cap-limits diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/vitest-timeout-cap-limits.snap");
  }, 30_000);
});
