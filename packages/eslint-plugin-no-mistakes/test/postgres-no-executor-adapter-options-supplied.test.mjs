import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-options-supplied";
const options = {
  importSpecifier: "@example/db",
  executorTypeNames: ["SessionExecutor"],
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
  mode: "forwarding-only",
};
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

function testCase(name, mode = options.mode) {
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

it("proves supplied optional executors passed on unchanged", async () => {
  const rows = snapshotSuite(rule, languageOptions, {
    valid: [testCase("proofs.ts"), testCase("importedProofs.ts"), testCase("appendProofs.ts")],
    invalid: [testCase("controls.ts"), testCase("dynamicCallee.ts"), testCase("regressions.ts")],
  });
  expect(rows[0].findings).toEqual([]);
  expect(rows[1].findings).toEqual([]);
  expect(rows[2].findings).toEqual([]);
  expect(rows[3].findings.map(({ line }) => line)).toEqual([
    61, 68, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160,
  ]);
  expect(rows[4].findings.map(({ line }) => line)).toEqual([
    34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44,
  ]);
  expect(rows[5].findings.map(({ line }) => line)).toEqual([34, 41, 46, 51]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-options-supplied.snap",
  );
});

it("judges supplied executors the same way in mode all", async () => {
  const rows = snapshotSuite(rule, languageOptions, {
    valid: [testCase("appendProofs.ts", "all")],
    invalid: [testCase("regressions.ts", "all")],
  });
  expect(rows[0].findings).toEqual([]);
  expect(rows[1].findings.map(({ line }) => line)).toEqual([34, 41, 46, 51]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-options-supplied-all.snap",
  );
});
