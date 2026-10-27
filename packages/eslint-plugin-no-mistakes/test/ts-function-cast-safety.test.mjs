import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, lint } from "./helpers.mjs";

const rule = "ts-no-function-aliases";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const tsLanguage = { parser: tsParser };
// Each source is saved so type identity, shadowing, and receiver counterexamples
// stay reviewable independently of the implementation's normalization helpers.
const corpus = JSON.parse(fixture("ts-object-recall/cast-safety-corpus.json"));

function castCase(code) {
  return {
    code,
    filename: "fixture.ts",
    languageOptions: tsLanguage,
  };
}

const suite = {
  valid: [
    // 28 corpus sources with no alias, then the two suppressed fixtures.
    ...corpus.filter((item) => item.expected === 0).map((item) => castCase(item.code)),
    castCase(fixture("ts-object-recall/asserted-suppressed.ts")),
    castCase(fixture("ts-object-recall/asserted-file-suppressed.ts")),
  ],
  invalid: [
    // 9 corpus sources with 1 alias each, then asserted-functions with 4 alias.
    ...corpus.filter((item) => item.expected !== 0).map((item) => castCase(item.code)),
    castCase(fixture("ts-object-recall/asserted-functions.ts")),
  ],
};

describe("cast identity and reachability safety corpus", () => {
  it("snapshots ts-function-cast-safety diagnostics", async () => {
    for (const { name, code } of corpus) {
      const diagnostics = lint(code, { [`no-mistakes/${rule}`]: "error" }, "fixture.ts");
      assert.equal(
        diagnostics.some((item) => item.fatal),
        false,
        name,
      );
    }
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/ts-function-cast-safety.snap");
  }, 30_000);
});

describe("cast contract Oxlint parity", () => {
  for (const [name, expected] of [
    ["asserted-functions", 4],
    ["asserted-suppressed", 0],
    ["asserted-file-suppressed", 0],
  ]) {
    it(name, () => {
      const cwd = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          ".oxlintrc.json",
          "--format",
          "json",
          resolve(cwd, `../ts-object-recall/${name}.ts`),
        ],
        { cwd, encoding: "utf8" },
      );
      assert.equal(result.status, expected ? 1 : 0, result.stderr);
      const actual = JSON.parse(result.stdout).diagnostics.filter(
        (item) => item.code === `no-mistakes(${rule})`,
      );
      assert.equal(actual.length, expected);
      assert.deepEqual(
        actual.map((item) => item.labels[0].span.line),
        lint(
          fixture(`ts-object-recall/${name}.ts`),
          { [`no-mistakes/${rule}`]: "error" },
          "fixture.ts",
        ).map((item) => item.line),
      );
    });
  }
});
