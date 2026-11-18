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
const options = {
  importSpecifier: "@example/db",
  executorNames: ["read", "write"],
  executorTypeNames: ["SessionExecutor"],
};
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

function testCase(name, mode, typed = true) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/executor-direct-options",
    name,
  );
  return {
    code: fixture(`executor-direct-options/${name}`),
    filename,
    options: [{ ...options, mode }],
    languageOptions: {
      ...languageOptions,
      parserOptions: typed ? { projectService: true, tsconfigRootDir: dirname(filename) } : {},
    },
  };
}

it.each(["all", "forwarding-only"])(
  "accepts direct SQL with session options in %s mode",
  (mode) => {
    expect(languageOptionsFindings({ ...testCase("positive.ts", mode), rule })).toEqual([]);
  },
);

it.each(["all", "forwarding-only"])(
  "snapshots direct options boundaries in %s mode",
  async (mode) => {
    const suite = {
      valid: [
        testCase("positive.ts", mode),
        testCase("positive.ts", mode, false),
        testCase("receiver.ts", mode),
      ],
      invalid: [
        testCase("negative.ts", mode),
        testCase("negative.ts", mode, false),
        testCase("receiver.ts", mode, false),
      ],
    };
    await expect(snapshotDocument(snapshotSuite(rule, languageOptions, suite))).toMatchFileSnapshot(
      `snapshots/postgres-no-executor-adapter-direct-options-${mode}.snap`,
    );
  },
);
