import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import {
  languageOptionsFindings,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const languageOptions = { ecmaVersion: 2024, sourceType: "module", globals: {} };
const base = {
  importSpecifier: "@example/db",
  executorTypeNames: ["QueryExecutor"],
  trustedSqlTags: [{ module: "@example/sql", name: "sql" }],
};

function testCase(name, mode, typed = false, config = base) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/executor-target",
    name,
  );
  return {
    code: fixture(`executor-target/${name}`),
    filename,
    options: [{ ...config, mode }],
    languageOptions: {
      parser: tsParser,
      parserOptions: typed ? { projectService: true, tsconfigRootDir: dirname(filename) } : {},
    },
  };
}

const suite = {
  valid: [
    testCase("imported-mode.ts", "all", true),
    testCase("opaque-eval.ts", "all"),
    testCase("opaque-wrapped-eval.ts", "all"),
    testCase("trusted-fragment-local.ts", "all"),
    testCase("trusted-fragment-imported.ts", "all", true),
    testCase("union-sources.ts", "all", true, { executorNames: ["read"] }),
    testCase("union-sources.ts", "forwarding-only", true, { executorNames: ["read"] }),
  ],
  invalid: [
    testCase("union.ts", "all"),
    testCase("union.ts", "all", true),
    testCase("union.ts", "forwarding-only"),
    testCase("union.ts", "forwarding-only", true),
    testCase("union-sources.ts", "all", true),
    testCase("union-sources.ts", "forwarding-only", true),
    testCase("forwarding-mode.ts", "all"),
    testCase("forwarding-mode.ts", "all", true),
    testCase("forwarding-mode.ts", "forwarding-only"),
    testCase("forwarding-mode.ts", "forwarding-only", true),
    testCase("opaque-eval.ts", "forwarding-only"),
    testCase("opaque-wrapped-eval.ts", "forwarding-only"),
    testCase("trusted-fragment-local.ts", "forwarding-only"),
    testCase("trusted-fragment-imported.ts", "forwarding-only", true),
    testCase("imported-mode.ts", "all"),
    testCase("imported-mode.ts", "forwarding-only"),
    testCase("imported-mode.ts", "forwarding-only", true),
  ],
};

it("snapshots union executor proof and both reporting modes", async () => {
  for (const mode of ["all", "forwarding-only"]) {
    const sample = testCase("union.ts", mode, true);
    const lines = languageOptionsFindings({
      ...sample,
      rule: "postgres-no-executor-adapter",
    }).map((finding) => finding.line);
    expect(lines).toEqual([19, 20, 23]);
  }
  for (const [name, typed, expected] of [
    ["trusted-fragment-local.ts", false, [46, 47, 49, 51, 52, 53, 54, 55, 57, 64, 88, 94, 96, 101]],
    [
      "trusted-fragment-imported.ts",
      true,
      [15, 16, 18, 20, 21, 22, 23, 24, 26, 33, 45, 51, 53, 54],
    ],
  ]) {
    const lines = languageOptionsFindings({
      ...testCase(name, "forwarding-only", typed),
      rule: "postgres-no-executor-adapter",
    }).map((finding) => finding.line);
    expect(lines).toEqual(expected);
  }
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-executor-adapter", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-executor-adapter-union-mode.snap");
}, 30_000);

it("keeps SQL checks on proven union and typeof parameters", async () => {
  const sqlSuite = {
    invalid: [
      {
        ...testCase("union-sql.ts", "all"),
        options: [base],
      },
    ],
  };
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-manual-transaction", languageOptions, sqlSuite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-executor-adapter-union-sql.snap");
}, 30_000);
