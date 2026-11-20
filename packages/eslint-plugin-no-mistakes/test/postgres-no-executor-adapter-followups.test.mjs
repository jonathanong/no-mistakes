import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-adapter-followups";
const options = {
  importSpecifier: "@example/db",
  executorTypeNames: ["SessionExecutor"],
  executorFactoryNames: ["beginTransaction"],
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
  mode: "forwarding-only",
};
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

function testCase(name, caseOptions = options, typed = name.endsWith(".ts")) {
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
      parserOptions: typed
        ? {
            project: resolve(dirname(filename), "tsconfig.json"),
            tsconfigRootDir: dirname(filename),
            disallowAutomaticSingleRunInference: true,
          }
        : { ecmaFeatures: { jsx: true } },
    },
  };
}

const allMode = { ...options, mode: "all" };

it("pins executor-row identity, read-only element consts, and the follow-up controls", async () => {
  const rows = snapshotSuite(rule, languageOptions, {
    valid: [
      testCase("proofs-elements.ts"),
      testCase("proofs-loop.ts"),
      testCase("proofs-scalar.ts"),
      testCase("proofs-options.ts"),
      testCase("proofs-imported.ts"),
      testCase("proofs-hops.ts"),
      testCase("proofs-omit.ts"),
      testCase("proofs-generic.ts"),
      testCase("proofs-generic.ts", allMode),
      testCase("proofs-quoted.ts"),
    ],
    invalid: [
      testCase("controls-executor.ts"),
      testCase("controls-order.ts"),
      testCase("controls-scalar.ts"),
      testCase("controls-options.ts"),
      testCase("controls-library.ts"),
      testCase("controls-hops.ts"),
      testCase("controls-generic.ts"),
      testCase("controls-branches.ts"),
      testCase("proofs-options.ts", allMode),
      testCase("proofs-omit.ts", allMode),
    ],
  });
  expect(rows.filter((row) => row.kind === "valid").every((row) => row.findings.length === 0)).toBe(
    true,
  );
  expect(rows.filter((row) => row.kind === "invalid").every((row) => row.findings.length > 0)).toBe(
    true,
  );
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-followups.snap",
  );
});
