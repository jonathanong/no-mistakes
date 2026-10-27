import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const filename = resolve(
  __dirname,
  "../../../test-cases/eslint-snippets/fixture/executor-target/scoped-options-sql.ts",
);
const code = fixture("executor-target/scoped-options-sql.ts");
const option = { importSpecifier: "@example/db", executorTypeNames: ["QueryExecutor"] };

function scopedCase(typed) {
  return {
    code,
    filename,
    options: [option],
    languageOptions: {
      parser: tsParser,
      parserOptions: typed
        ? {
            project: resolve(dirname(filename), "tsconfig.json"),
            disallowAutomaticSingleRunInference: true,
          }
        : {},
    },
  };
}

const manual = {
  invalid: [
    // finds manual transactions through named, aliased, and identifier options.
    // Four manualTransaction findings.
    scopedCase(true),
    // requires typed services for custom properties but preserves query recall.
    // One manualTransaction finding.
    scopedCase(false),
  ],
};
const fanout = {
  valid: [
    // requires typed services for custom properties but preserves query recall.
    // Zero findings.
    scopedCase(false),
  ],
  invalid: [
    // finds unbounded fanout through declared executor properties.
    // Three unboundedFanout findings.
    scopedCase(true),
  ],
};
const adapter = {
  invalid: [
    // still reports opaque helper forwarding inside accepted options helpers.
    // Six passed findings.
    scopedCase(true),
  ],
};

it("snapshots postgres scoped options diagnostics", async () => {
  await expect(
    snapshotDocument({
      "postgres-no-manual-transaction": snapshotSuite(
        "postgres-no-manual-transaction",
        languageOptions,
        manual,
      ),
      "postgres-no-unbounded-query-fanout": snapshotSuite(
        "postgres-no-unbounded-query-fanout",
        languageOptions,
        fanout,
      ),
      "postgres-no-executor-adapter": snapshotSuite(
        "postgres-no-executor-adapter",
        languageOptions,
        adapter,
      ),
    }),
  ).toMatchFileSnapshot("snapshots/postgres-scoped-options-sql.snap");
}, 30_000);
