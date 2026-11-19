import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-library-terminal";
const options = {
  importSpecifier: "@example/db",
  executorTypeNames: ["SessionExecutor"],
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
};
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

function testCase(name, mode) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture",
    fixtureDirectory,
    name,
  );
  return {
    code: fixture(`${fixtureDirectory}/${name}`),
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

// Closure lines in dynamicControls.ts report in both modes; the callers that
// hand the options to an adapter whose helper builds dynamic SQL report too.
const CLOSURES = [42, 46, 50, 54];
const CALLERS = [63, 67, 68, 69];
const RESOLVER_CLOSURES = [41, 45, 49, 53];

for (const mode of ["forwarding-only", "all"]) {
  it(`requires the helper's SQL proof behind a closure terminal in ${mode} mode`, async () => {
    const rows = snapshotSuite(rule, languageOptions, {
      valid: [testCase("dynamicProofs.ts", mode), testCase("dynamicImportedProofs.ts", mode)],
      invalid: [
        testCase("dynamicControls.ts", mode),
        testCase("dynamicImported.ts", mode),
        testCase("dynamicResolvers.ts", mode),
      ],
    });
    expect(rows[0].findings).toEqual([]);
    expect(rows[1].findings).toEqual([]);
    expect(rows[2].findings.map(({ line }) => line)).toEqual([...CLOSURES, ...CALLERS]);
    expect(rows[3].findings.map(({ line }) => line)).toEqual([5, 6, 7, 8]);
    expect(rows[4].findings.map(({ line }) => line)).toEqual(RESOLVER_CLOSURES);
    await expect(snapshotDocument(rows)).toMatchFileSnapshot(
      `snapshots/postgres-no-executor-adapter-library-dynamic-${mode}.snap`,
    );
  });
}
