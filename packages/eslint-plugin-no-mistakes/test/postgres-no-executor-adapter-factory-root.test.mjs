import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-factory-root";
const options = {
  importSpecifier: "@example/db",
  executorTypeNames: ["SessionExecutor"],
  executorFactoryNames: ["beginTransaction"],
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
  mode: "forwarding-only",
};
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

function testCase(name, caseOptions = options) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture",
    fixtureDirectory,
    name,
  );
  return {
    code: fixture(`${fixtureDirectory}/${name}`),
    filename,
    options: [caseOptions],
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

it("accepts executor-factory roots proved like executor parameters", async () => {
  const rows = snapshotSuite(rule, languageOptions, {
    valid: [testCase("proofs.ts")],
    invalid: [testCase("controls.ts"), testCase("shapes.ts")],
  });
  expect(rows[0].findings.map(({ line }) => line)).toEqual([]);
  expect(rows[1].findings.map(({ line }) => line)).toEqual([18, 26, 31, 35, 44]);
  expect(rows[2].findings.map(({ line }) => line)).toEqual([
    21, 29, 37, 45, 53, 60, 67, 82, 95, 104,
  ]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-factory-root.snap",
  );
});

it("still reports forwarding when executor type names are not configured", () => {
  const { executorTypeNames, ...factoryOnly } = options;
  expect(executorTypeNames).toEqual(["SessionExecutor"]);
  const rows = snapshotSuite(rule, languageOptions, {
    valid: [],
    invalid: [testCase("proofs.ts", factoryOnly)],
  });
  expect(rows[0].findings.map(({ line }) => line)).toEqual([15, 23]);
});
