import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };
const common = {
  executorFactoryNames: ["beginTransaction", "beginBoundedTransaction"],
  executorTypeNames: ["TransactionQuery", "QueryExecutor", "PoolClient"],
};
const packageOptions = {
  ...common,
  importSpecifier: "@data-stores/psql",
  trustedSqlTags: [
    { module: "sql-template-strings", name: "default" },
    { module: "sql-template-strings", name: "SQL" },
  ],
};
const relativeOptions = {
  ...common,
  importSpecifier: "./db.mjs",
  trustedSqlTags: [{ module: "./sql.mjs", name: "default" }],
};

function testCase(directory, name, options, mode) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture",
    directory,
    name,
  );
  return {
    code: fixture(`${directory}/${name}`),
    filename,
    options: [{ ...options, mode }],
    languageOptions: {
      ...languageOptions,
      parserOptions: {
        project: resolve(dirname(filename), "tsconfig.json"),
        tsconfigRootDir: dirname(filename),
        disallowAutomaticSingleRunInference: true,
      },
    },
  };
}

const CONTROL_LINES = [37, 41, 45, 49, 53, 57, 61, 65, 69, 70, 71, 72, 73, 74, 75, 76, 77];
// The adapter closure reports, and so does the caller that hands it the options.
const T4_LINES = [18, 25];

for (const mode of ["forwarding-only", "all"]) {
  it(`clears library terminals under the validator options in ${mode} mode`, async () => {
    const package_ = "executor-library-package";
    const rows = snapshotSuite(rule, languageOptions, {
      valid: [
        testCase(package_, "terminals.ts", packageOptions, mode),
        testCase("executor-library-validator", "terminals.mts", relativeOptions, mode),
      ],
      invalid: [
        testCase(package_, "controls.ts", packageOptions, mode),
        testCase("executor-library-validator", "controls.mts", relativeOptions, mode),
      ],
    });
    expect(rows[3].findings.map(({ line }) => line)).toEqual(T4_LINES);
    expect(rows[0].findings).toEqual([]);
    expect(rows[1].findings).toEqual([]);
    expect(rows[2].findings.map(({ line }) => line)).toEqual(CONTROL_LINES);
    await expect(snapshotDocument(rows)).toMatchFileSnapshot(
      `snapshots/postgres-no-executor-adapter-library-validator-${mode}.snap`,
    );
  });
}
