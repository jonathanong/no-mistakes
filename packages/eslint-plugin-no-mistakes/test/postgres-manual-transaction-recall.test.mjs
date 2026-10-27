import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/postgres-manual-transaction-recall/fixture",
);
const recallCases = [
  ["pg", "pg", [4, 5, 8, 10, 12, 15, 18]],
  ["configured", "@data-stores/psql", [3, 4, 7, 9, 11, 14, 17]],
];
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function recallCase(name, typed, option) {
  const filename = resolve(root, `${name}.ts`);
  return {
    code: readFileSync(filename, "utf8"),
    filename,
    options: [option],
    languageOptions: {
      parser: tsParser,
      parserOptions: typed
        ? { project: resolve(root, "tsconfig.json"), disallowAutomaticSingleRunInference: true }
        : {},
    },
  };
}

const valid = [];
const invalid = [];
for (const typed of [false, true]) {
  for (const [name, importSpecifier] of recallCases) {
    // retains manual transaction recall through connect wrappers.
    // Seven manualTransaction findings.
    invalid.push(recallCase(name, typed, { importSpecifier }));
    // Zero findings.
    valid.push(recallCase(name, typed, { importSpecifier, executorNames: [] }));
    valid.push(recallCase(name, typed, { importSpecifier, executorNames: ["execute"] }));
  }
  // Zero findings.
  valid.push(recallCase("controls", typed, { importSpecifier: "@data-stores/psql" }));
}

it("snapshots postgres-no-manual-transaction diagnostics", async () => {
  await expect(
    snapshotDocument(
      snapshotSuite("postgres-no-manual-transaction", languageOptions, { valid, invalid }),
    ),
  ).toMatchFileSnapshot("snapshots/postgres-manual-transaction-recall.snap");
}, 30_000);
it("matches recall and suppression in real type-aware Oxlint", () => {
  for (const [name, , lines] of recallCases) {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--type-aware",
        "--deny-warnings",
        "--format",
        "json",
        "--config",
        resolve(root, `.oxlintrc-${name}.json`),
        resolve(root, `${name}.ts`),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    assert.deepEqual(
      JSON.parse(result.stdout).diagnostics.map(({ labels, code }) => [labels[0].span.line, code]),
      lines.map((line) => [line, "no-mistakes(postgres-no-manual-transaction)"]),
    );
  }
});
