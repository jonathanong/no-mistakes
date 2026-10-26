import assert from "node:assert/strict";
import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { describe, it } from "vitest";
import { __dirname, fixture, plugin } from "./helpers.mjs";

const filename = resolve(
  __dirname,
  "../../../test-cases/eslint-snippets/fixture/executor-target/scoped-options-sql.ts",
);
const code = fixture("executor-target/scoped-options-sql.ts");
const options = { importSpecifier: "@example/db", executorTypeNames: ["QueryExecutor"] };

function findings(rule, typed = true) {
  return new Linter({ cwd: dirname(filename) })
    .verify(
      code,
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          // Each test re-lints the same file; this is not an ESLint autofix cycle.
          parserOptions: typed
            ? {
                project: resolve(dirname(filename), "tsconfig.json"),
                disallowAutomaticSingleRunInference: true,
              }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: { [`no-mistakes/${rule}`]: ["error", options] },
      },
      { filename },
    )
    .map(({ line, messageId }) => `${line}:${messageId}`);
}

describe("PostgreSQL rules for declared options executors", () => {
  it("finds manual transactions through named, aliased, and identifier options", () => {
    assert.deepEqual(findings("postgres-no-manual-transaction"), [
      "12:manualTransaction",
      "15:manualTransaction",
      "18:manualTransaction",
      "21:manualTransaction",
    ]);
  });

  it("finds unbounded fanout through declared executor properties", () => {
    assert.deepEqual(findings("postgres-no-unbounded-query-fanout"), [
      "25:unboundedFanout",
      "28:unboundedFanout",
      "31:unboundedFanout",
    ]);
  });

  it("still reports opaque helper forwarding inside accepted options helpers", () => {
    assert.deepEqual(findings("postgres-no-executor-adapter"), [
      "35:passed",
      "38:passed",
      "41:passed",
      "43:passed",
      "44:passed",
      "45:passed",
    ]);
  });

  it("requires typed services for custom properties but preserves query recall", () => {
    assert.deepEqual(findings("postgres-no-manual-transaction", false), ["15:manualTransaction"]);
    assert.deepEqual(findings("postgres-no-unbounded-query-fanout", false), []);
  });
});
