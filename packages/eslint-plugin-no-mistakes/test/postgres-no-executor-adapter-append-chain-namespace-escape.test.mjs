import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-append-chain";
const baseOptions = {
  importSpecifier: "@example/db",
  executorTypeNames: ["SessionExecutor"],
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
};
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

function testCase(name) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture",
    fixtureDirectory,
    name,
  );
  return {
    code: fixture(`${fixtureDirectory}/${name}`),
    filename,
    options: [{ ...baseOptions, mode: "forwarding-only" }],
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

function suite(valid, invalid) {
  return snapshotSuite(rule, languageOptions, {
    valid: valid.map((name) => testCase(name)),
    invalid: invalid.map((name) => testCase(name)),
  });
}

const lines = (rows) => rows.map((row) => row.findings.map(({ line }) => line));

const VALID = [
  "starNamedProof.ts",
  "starNamespaceProof.ts",
  "elementKeep.ts",
  "typeofTypeFragments.ts",
];
const INVALID = [
  "starNsMutControls.ts",
  "starNamedMutControls.ts",
  "requireMemberControls.ts",
  "requireValueControls.ts",
  "dynamicMemberControls.ts",
  "dynamicValueControls.ts",
  "equalsControls.ts",
  "valueControls.ts",
  "assignControls.ts",
  "reflectControls.ts",
  "elementControls.ts",
  "computedKeyControls.ts",
  "assertedControls.ts",
  "destructureSibling.ts",
  "typeofValueFragments.ts",
];

it("escapes namespace, require, and dynamic-import sql fragments", async () => {
  const rows = suite(VALID, INVALID);
  expect(lines(rows)).toEqual([
    [],
    [],
    [],
    [],
    [10],
    [10],
    [12],
    [12, 20],
    [13, 21],
    [12, 20],
    [12],
    [12, 20],
    [12, 20],
    [12, 20],
    [12],
    [12, 20],
    [12, 22],
    [12, 20],
    [12, 20],
  ]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-append-chain-namespace-escape.snap",
  );
});
