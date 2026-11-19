import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-inline-date";
const options = {
  importSpecifier: "./db.mjs",
  executorTypeNames: ["SessionExecutor"],
  executorFactoryNames: ["beginTransaction", "beginBoundedTransaction"],
  trustedSqlTags: [{ module: "./sql.mjs", name: "default" }],
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

it("proves an inline new Date() option value is fresh and non-row", async () => {
  const rows = snapshotSuite(rule, languageOptions, {
    valid: [testCase("datefields.mts")],
    invalid: [testCase("controls.mts")],
  });
  expect(rows.map((row) => row.findings.map(({ line }) => line))).toEqual([
    [],
    [15, 21, 28, 36, 45, 51, 58],
  ]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-inline-date.snap",
  );
});
