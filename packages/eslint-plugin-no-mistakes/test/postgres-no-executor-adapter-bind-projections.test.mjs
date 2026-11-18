import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import {
  languageOptionsFindings,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const rule = "postgres-no-executor-adapter";
const options = {
  importSpecifier: "@example/db",
  executorNames: ["read", "write"],
  executorTypeNames: ["SessionExecutor"],
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
};
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

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
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-bind-projections.snap",
  );
});
