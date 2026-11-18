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
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
};
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

function fixtureRows(rows) {
  const prefix = "test-cases/eslint-snippets/fixture/";
  return rows.map((row) => ({
    ...row,
    file: row.file.replace(prefix, ""),
    findings: row.findings.map((finding) => ({
      ...finding,
      file: finding.file.replace(prefix, ""),
    })),
  }));
}

function testCase(name, mode, typed = true, loose = false) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/executor-bind-projections",
    name,
  );
  return {
    code: fixture(`executor-bind-projections/${name}`),
    filename,
    options: [{ ...options, mode }],
    languageOptions: {
      ...languageOptions,
      parserOptions: typed
        ? {
            project: resolve(dirname(filename), loose ? "tsconfig.loose.json" : "tsconfig.json"),
            tsconfigRootDir: dirname(filename),
            disallowAutomaticSingleRunInference: true,
          }
        : {},
    },
  };
}

it.each(["local.ts", "imported.ts"])(
  "accepts required primitive bind projections in %s",
  (name) => {
    const sample = testCase(name, "forwarding-only");
    expect(languageOptionsFindings({ ...sample, rule })).toEqual([]);
  },
);

it("snapshots typed projection boundaries and unchanged default-mode diagnostics", async () => {
  const suite = {
    valid: [
      testCase("local.ts", "forwarding-only"),
      testCase("imported.ts", "forwarding-only"),
      testCase("local.ts", "all"),
      testCase("imported.ts", "all"),
    ],
    invalid: [
      testCase("negative.ts", "forwarding-only"),
      testCase("negative-imported.ts", "forwarding-only"),
      testCase("negative.ts", "forwarding-only", true, true),
      testCase("negative.ts", "all"),
      testCase("negative-imported.ts", "all"),
      testCase("local.ts", "forwarding-only", false),
    ],
  };
  const rows = snapshotSuite(rule, languageOptions, suite);
  expect(rows.map((row) => row.findings.length)).toEqual([0, 0, 0, 0, 24, 14, 24, 1, 1, 7]);
  await expect(snapshotDocument(fixtureRows(rows))).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-bind-projections.snap",
  );
});

it("reports loop writes, scalar property requirements, and private members", async () => {
  const rows = snapshotSuite(rule, languageOptions, {
    invalid: [testCase("review.ts", "forwarding-only")],
  });
  expect(rows[0].findings.map(({ line }) => line)).toEqual([7, 11, 17, 23, 29]);
  await expect(snapshotDocument(fixtureRows(rows))).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-bind-review.snap",
  );
});
