import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-options-object";
const options = {
  importSpecifier: "@example/db",
  executorTypeNames: ["SessionExecutor"],
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
  mode: "forwarding-only",
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
    options: [options],
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

it("accepts options-object executors only for declared executor-typed properties", async () => {
  const rows = snapshotSuite(rule, languageOptions, {
    valid: [testCase("proofs.ts"), testCase("importedProofs.ts")],
    invalid: [testCase("controls.ts"), testCase("importedControls.ts")],
  });
  expect(rows[0].findings).toEqual([]);
  expect(rows[1].findings).toEqual([]);
  expect(rows[2].findings.map(({ line }) => line)).toEqual([56, 80, 92, 99]);
  expect(rows[3].findings.map(({ line }) => line)).toEqual([15, 21, 27, 33, 39, 45, 51, 57]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-options-object.snap",
  );
});
