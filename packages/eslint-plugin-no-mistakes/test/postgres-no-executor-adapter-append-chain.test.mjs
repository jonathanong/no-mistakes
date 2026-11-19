import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());

const rule = "postgres-no-executor-adapter";
const fixtureDirectory = "executor-append-chain";
const baseOptions = {
  importSpecifier: "@example/db",
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
  const parserOptions = typed
    ? {
        project: resolve(dirname(filename), "tsconfig.json"),
        tsconfigRootDir: dirname(filename),
        disallowAutomaticSingleRunInference: true,
      }
    : {};
  return {
    code: fixture(`${fixtureDirectory}/${name}`),
    filename,
    options: [{ ...baseOptions, mode }],
    languageOptions: { ...languageOptions, parserOptions },
  };
}

function suite(mode, valid, invalid, typed = true) {
  return snapshotSuite(rule, languageOptions, {
    valid: valid.map((name) => testCase(name, mode, typed)),
    invalid: invalid.map((name) => testCase(name, mode, typed)),
  });
}

const lines = (rows) => rows.map((row) => row.findings.map(({ line }) => line));

const PROOFS = ["proofs.ts", "importedConstProofs.ts", "importedProofs.ts"];
const CONTROLS = ["controls.ts", "importedControls.ts"];
const MUTATED = ["aliasControls.ts", "exportedConstControls.ts", "listedConstControls.ts"];
const ESCAPED = [
  "initializerAliasControls.ts",
  "shorthandPropertyControls.ts",
  "propertyAssignmentControls.ts",
  "bindingElementControls.ts",
  "parameterDefaultControls.ts",
  "exportAssignmentControls.ts",
];
const ESCAPED_LINES = [[15], [15], [15], [15], [14], [12]];
const CONTROL_LINES = [67, 70, 73, 76, 79, 82, 85, 88, 91, 94, 97, 100, 109];

it("proves constant append chains and reports the controls in forwarding-only mode", async () => {
  const rows = suite("forwarding-only", PROOFS, CONTROLS);
  expect(lines(rows)).toEqual([[], [], [], CONTROL_LINES, [6, 7, 8]]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-append-chain-forwarding-only.snap",
  );
});

it("accepts every append-chain fixture in mode all", async () => {
  const rows = suite("all", [...PROOFS, ...CONTROLS], []);
  expect(lines(rows)).toEqual([[], [], [], [], []]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-append-chain-all.snap",
  );
});

it("proves only file-local constants without type information", async () => {
  const rows = suite(
    "forwarding-only",
    ["proofs.ts"],
    ["importedConstProofs.ts", "controls.ts"],
    false,
  );
  expect(lines(rows)).toEqual([[], [23, 26], CONTROL_LINES]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-append-chain-untyped.snap",
  );
});

it("reports constants mutated through aliases or other modules", async () => {
  const rows = suite("forwarding-only", [], MUTATED);
  expect(lines(rows)).toEqual([[14, 17], [11], [11]]);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-append-chain-mutated.snap",
  );
});

it("reports the same mutated constants without type information", async () => {
  const rows = suite("forwarding-only", [], MUTATED, false);
  expect(lines(rows)).toEqual([[14, 17], [11], [11]]);
});

it("reports constants that escape through an initializer or other declaration position", async () => {
  const rows = suite("forwarding-only", [], ESCAPED);
  expect(lines(rows)).toEqual(ESCAPED_LINES);
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-append-chain-escaped.snap",
  );
});

it("reports the same escaped constants without type information", () => {
  const rows = suite("forwarding-only", [], ESCAPED, false);
  expect(lines(rows)).toEqual(ESCAPED_LINES);
});
