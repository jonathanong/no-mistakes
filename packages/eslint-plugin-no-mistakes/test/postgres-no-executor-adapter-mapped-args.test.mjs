import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-mapped-args";
const baseOptions = {
  importSpecifier: "@example/db",
  executorTypeNames: ["SessionExecutor"],
  trustedSqlTags: [{ module: "@example/sql", name: "default" }],
};
const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module", globals: {} };

function testCase(name, mode) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture",
    fixtureDirectory,
    name,
  );
  return {
    code: fixture(`${fixtureDirectory}/${name}`),
    filename,
    options: [{ ...baseOptions, mode }],
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

function suite(mode, valid, invalid) {
  return snapshotSuite(rule, languageOptions, {
    valid: valid.map((name) => testCase(name, mode)),
    invalid: invalid.map((name) => testCase(name, mode)),
  });
}

const CALLEES = [
  37, 45, 53, 61, 69, 77, 89, 97, 105, 112, 120, 128, 136, 144, 146, 150, 150, 155, 160, 160, 166,
  173, 180, 187, 195, 203, 211, 219, 227, 232, 232, 241, 249, 257, 265, 273, 280, 285,
];
const CALLERS = Array.from({ length: 34 }, (_, index) => 289 + index * 4);
const CONTROLS = [...CALLEES, ...CALLERS];
const ALL_CONTROLS = [232];

function lines(rows) {
  return rows.map((row) => row.findings.map(({ line }) => line));
}

it("accepts mapped projections and awaited helper results as hop arguments", async () => {
  const rows = suite("forwarding-only", ["proofs.ts"], ["controls.ts"]);
  expect(lines(rows)).toEqual([[], CONTROLS]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-mapped-args-forwarding-only.snap",
  );
});

it("reports the same controls in mode all", async () => {
  const rows = suite("all", ["proofs.ts"], ["controls.ts"]);
  expect(lines(rows)).toEqual([[], ALL_CONTROLS]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-mapped-args-all.snap",
  );
});
