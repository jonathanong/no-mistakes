import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-multi-hop";
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

const proofs = ["proofs.ts", "date-proofs.ts"];

function suite(mode, valid, invalid) {
  return snapshotSuite(rule, languageOptions, {
    valid: valid.map((name) => testCase(name, mode)),
    invalid: invalid.map((name) => testCase(name, mode)),
  });
}

const CONTROLS = [
  57, 62, 66, 71, 75, 79, 88, 92, 96, 100, 104, 108, 112, 116, 120, 129, 133, 137, 141, 145,
];
const ALL_CONTROLS = [62, 66, 145];

function lines(rows) {
  return rows.map((row) => row.findings.map(({ line }) => line));
}

it("proves callees that forward the executor to proven helpers", async () => {
  const rows = suite("forwarding-only", proofs, ["controls.ts", "date-controls.ts"]);
  expect(lines(rows)).toEqual([[], [], CONTROLS, [11, 15, 19]]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-multi-hop-forwarding-only.snap",
  );
});

it("reports the same controls in mode all", async () => {
  const rows = suite("all", [...proofs, "date-controls.ts"], ["controls.ts"]);
  expect(lines(rows)).toEqual([[], [], [], ALL_CONTROLS]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-multi-hop-all.snap",
  );
});
