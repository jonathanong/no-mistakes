import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  languageOptionsFindings,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-nonwriting-projections";
const options = {
  importSpecifier: "@example/db",
  executorNames: ["read", "write"],
  executorTypeNames: ["SessionExecutor"],
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
};
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

function testCase(name, mode, typed = true) {
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
      parserOptions: typed
        ? {
            project: resolve(dirname(filename), "tsconfig.json"),
            tsconfigRootDir: dirname(filename),
            disallowAutomaticSingleRunInference: true,
          }
        : {},
    },
  };
}

const positiveFiles = ["local.ts", "imported.ts", "helpers.ts", "reader-pure-bodies.ts"];

it.each(positiveFiles)("accepts projections around visible nonwriting calls in %s", (name) => {
  expect(languageOptionsFindings({ ...testCase(name, "forwarding-only"), rule })).toEqual([]);
});

it("snapshots nonwriting projection boundaries and mode controls", async () => {
  const suite = {
    valid: [
      ...positiveFiles.map((name) => testCase(name, "forwarding-only")),
      ...positiveFiles.map((name) => testCase(name, "all")),
      testCase("negative.ts", "all"),
      testCase("reader-boundaries.ts", "all"),
      testCase("reader-hidden-effects.ts", "all"),
      testCase("reader-unsupported-bodies.ts", "all"),
    ],
    invalid: [
      testCase("negative.ts", "forwarding-only"),
      testCase("local.ts", "forwarding-only", false),
      testCase("imported.ts", "forwarding-only", false),
      testCase("reader-boundaries.ts", "forwarding-only"),
      testCase("reader-hidden-effects.ts", "forwarding-only"),
      testCase("reader-unsupported-bodies.ts", "forwarding-only"),
    ],
  };
  const rows = snapshotSuite(rule, languageOptions, suite);
  expect(rows.map((row) => row.findings.length)).toEqual([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 2, 3, 22, 3, 9,
  ]);
  const hiddenEffects = rows.find(
    (row) => row.kind === "invalid" && row.file.endsWith("/reader-hidden-effects.ts"),
  );
  expect(hiddenEffects.findings.map(({ line, messageId }) => [line, messageId])).toEqual([
    [35, "passed"],
    [40, "passed"],
    [45, "passed"],
  ]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-nonwriting-projections.snap",
  );
});
