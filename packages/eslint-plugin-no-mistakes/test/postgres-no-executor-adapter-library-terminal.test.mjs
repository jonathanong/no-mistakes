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

const SHARED_LINES = [30, 35, 59, 70, 74, 78, 78, 86, 87, 88, 89, 90];
const REPORTED_LINES = {
  "forwarding-only": [...SHARED_LINES, 90, 91, 92, 93, 94, 95, 96, 97, 97, 98],
  all: [...SHARED_LINES, 91, 92, 93, 94, 95, 96, 97, 98],
};

for (const mode of ["forwarding-only", "all"]) {
  it(`ends options proofs at configured library executors in ${mode} mode`, async () => {
    const rows = snapshotSuite(rule, languageOptions, {
      valid: [testCase("proofs.ts", mode), testCase("importedProofs.ts", mode)],
      invalid: [testCase("controls.ts", mode), testCase("importedControls.ts", mode)],
    });
    expect(rows[0].findings).toEqual([]);
    expect(rows[1].findings).toEqual([]);
    expect(rows[2].findings.map(({ line }) => line)).toEqual(REPORTED_LINES[mode]);
    expect(rows[3].findings.map(({ line }) => line)).toEqual([5, 6, 7, 8, 9, 10, 11, 12]);
    await expect(snapshotDocument(rows)).toMatchFileSnapshot(
      `snapshots/postgres-no-executor-adapter-library-terminal-${mode}.snap`,
    );
  });
}
