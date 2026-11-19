import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-object-projection";
const options = {
  importSpecifier: "@example/db",
  executorTypeNames: ["SessionExecutor"],
  executorFactoryNames: ["beginTransaction"],
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

it("accepts proven row projections inside object-literal arguments", async () => {
  const rows = snapshotSuite(rule, languageOptions, {
    valid: [testCase("proofs.ts"), testCase("silent.ts"), testCase("unrelated.ts")],
    invalid: [testCase("controls.ts"), testCase("unrelated-controls.ts")],
  });
  expect(rows.map((row) => row.findings.map(({ line }) => line))).toEqual([
    [],
    [],
    [],
    [19, 26, 34, 41, 48, 57, 66, 76],
    [25, 32, 40, 47, 55, 63, 72, 79, 87, 95, 102, 109, 116, 123],
  ]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-object-projection.snap",
  );
});
