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
  trustedSqlTags: [{ module: "sql-template-strings", name: "default" }],
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

const CONTROL_LINES = [37, 38, 39, 40, 41];

for (const mode of ["forwarding-only", "all"]) {
  it(`clears option types re-exported from another package in ${mode} mode`, async () => {
    const dir = "executor-reexport-package";
    const rows = snapshotSuite(rule, languageOptions, {
      valid: [testCase(dir, "terminals.ts", packageOptions, mode)],
      invalid: [testCase(dir, "controls.ts", packageOptions, mode)],
    });
    expect(rows[0].findings).toEqual([]);
    expect(rows[1].findings.map(({ line }) => line)).toEqual(CONTROL_LINES);
    await expect(snapshotDocument(rows)).toMatchFileSnapshot(
      `snapshots/postgres-no-executor-adapter-reexport-${mode}.snap`,
    );
  });
}

for (const mode of ["forwarding-only", "all"]) {
  it(`reports instead of crashing on a symbol-less property and stops at the nearest manifest in ${mode} mode`, async () => {
    const rows = snapshotSuite(rule, languageOptions, {
      valid: [],
      invalid: [
        testCase("executor-reexport-package", "crash.ts", packageOptions, mode),
        testCase("executor-package-boundary", "controls.ts", packageOptions, mode),
      ],
    });
    expect(rows[0].findings.length).toBeGreaterThan(0);
    expect(rows[1].findings.map(({ line }) => line)).toEqual([15, 16]);
    await expect(snapshotDocument(rows)).toMatchFileSnapshot(
      `snapshots/postgres-no-executor-adapter-reexport-crash-${mode}.snap`,
    );
  });
}
