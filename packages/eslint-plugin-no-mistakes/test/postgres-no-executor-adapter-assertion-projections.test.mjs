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
const fixtureDirectory = "executor-assertion-projections";
const options = {
  importSpecifier: "@example/db",
  executorNames: ["read"],
  executorTypeNames: ["SessionExecutor"],
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
};
const nonWritingCallees = [
  { module: "@example/assertions", export: "default" },
  { module: "@example/assertions", export: "namedCheck" },
  { module: "@example/assertions", export: "unchecked" },
  { module: "@example/assertions", export: "unknownCheck" },
  { module: "@example/mutable-assertions", export: "mutableCheck" },
];
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

function testCase(name, mode, { configured = false, typed = true } = {}) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture",
    fixtureDirectory,
    name,
  );
  return {
    code: fixture(`${fixtureDirectory}/${name}`),
    filename,
    options: [{ ...options, mode, ...(configured && { nonWritingCallees }) }],
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

it("preserves forwarding-only projections through stable assertions and configured imports", async () => {
  const rows = snapshotSuite(rule, languageOptions, {
    valid: [
      testCase("ambient.ts", "forwarding-only"),
      testCase("imported.ts", "forwarding-only"),
      testCase("configured.ts", "forwarding-only", { configured: true }),
      testCase("negative.ts", "all"),
      testCase("guard-proofs.ts", "forwarding-only"),
      testCase("configured-direct-named.ts", "forwarding-only", { configured: true }),
    ],
    invalid: [
      testCase("configured.ts", "forwarding-only"),
      testCase("ambient.ts", "forwarding-only", { typed: false }),
      testCase("imported.ts", "forwarding-only", { typed: false }),
      testCase("negative.ts", "forwarding-only"),
      testCase("namespace-write.ts", "forwarding-only"),
      testCase("computed-namespace-write.ts", "forwarding-only"),
      testCase("eval.ts", "forwarding-only"),
      testCase("binding-boundaries.ts", "forwarding-only"),
      testCase("configured-boundaries.ts", "forwarding-only", { configured: true }),
      testCase("configured-reassignment.ts", "forwarding-only", { configured: true }),
      testCase("guard-controls.ts", "forwarding-only"),
      testCase("configured-type-only.ts", "forwarding-only", { configured: true }),
    ],
  });
  expect(rows.map((row) => row.findings.length)).toEqual([
    0, 0, 0, 0, 0, 0, 2, 2, 2, 17, 2, 1, 1, 4, 8, 2, 12, 2,
  ]);
  const mutationFindings = languageOptionsFindings({
    ...testCase("negative.ts", "forwarding-only"),
    options: [],
    rule: "no-object-mutation",
  });
  expect(mutationFindings.map(({ line, ruleId }) => [line, ruleId])).toEqual([
    [29, "no-mistakes/no-object-mutation"],
  ]);
  const negativeRow = rows.find(
    (row) => row.kind === "invalid" && row.file.endsWith("/negative.ts"),
  );
  negativeRow.findings = [...negativeRow.findings, ...mutationFindings].sort(
    (left, right) => left.line - right.line || left.column - right.column,
  );
  const forwarding = testCase("acceptance-forwarding.ts", "forwarding-only");
  const configuredForwarding = testCase("acceptance-forwarding.ts", "forwarding-only", {
    configured: true,
  });
  const forwardingRows = snapshotSuite(rule, languageOptions, {
    invalid: [forwarding, configuredForwarding],
  });
  expect(
    forwardingRows.map(({ findings }) => findings.map(({ line, messageId }) => [line, messageId])),
  ).toEqual([
    [
      [21, "passed"],
      [27, "passed"],
      [33, "passed"],
    ],
    [
      [27, "passed"],
      [33, "passed"],
    ],
  ]);
  const forwardingMutation = languageOptionsFindings({
    ...forwarding,
    options: [],
    rule: "no-object-mutation",
  });
  expect(forwardingMutation.map(({ line, messageId }) => [line, messageId])).toEqual([
    [32, "mutation"],
  ]);
  for (const row of forwardingRows) {
    row.findings = [...row.findings, ...forwardingMutation].sort(
      (left, right) => left.line - right.line || left.column - right.column,
    );
  }
  rows.push(...forwardingRows);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-assertion-projections.snap",
  );
});
