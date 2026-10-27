import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, lint } from "./helpers.mjs";

const rule = "ts-preserve-null-option-defaults";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const cases = [
  ["invalid", [7, 7, 13, 20, 27, 33]],
  ["controls", []],
  ["rewrite", []],
  ["suppressed", [8, 20]],
  ["file-suppressed", []],
];
const filename = (name) => `scalar-alias-${name}.ts`;
const diagnostics = (name) =>
  lint(
    fixture(`${rule}/${filename(name)}`),
    { [`no-mistakes/${rule}`]: "error" },
    resolve(root, filename(name)),
  );

function aliasCase(name) {
  // Same diagnostics as the absolute path passed to lint(). A relative filename
  // keeps the snapshot portable; pathAllowed sees either form.
  return {
    code: fixture(`${rule}/${filename(name)}`),
    filename: `test-cases/eslint-snippets/fixture/${rule}/${filename(name)}`,
    languageOptions: { parser: tsParser },
  };
}

const suite = {
  valid: cases.filter(([, lines]) => lines.length === 0).map(([name]) => aliasCase(name)),
  invalid: cases.filter(([, lines]) => lines.length > 0).map(([name]) => aliasCase(name)),
};

describe("nullable option scalar const aliases", () => {
  it("snapshots nullable-scalar-alias diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/nullable-scalar-alias.snap");
  }, 30_000);

  it("matches exact Oxlint report locations and messages", () => {
    for (const [name, lines] of cases) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc-scalar-alias.json"),
          "--format",
          "json",
          resolve(root, filename(name)),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, lines.length ? 1 : 0, `${name}: ${result.stderr}`);
      const actual = JSON.parse(result.stdout).diagnostics.filter(
        ({ code }) => code === `no-mistakes(${rule})`,
      );
      const code = fixture(`${rule}/${filename(name)}`);
      const expected = diagnostics(name).map(({ line, column, endLine, endColumn, message }) => {
        const text = code.split("\n");
        const offset =
          text.slice(0, line - 1).reduce((sum, part) => sum + part.length + 1, 0) + column - 1;
        const end =
          text.slice(0, endLine - 1).reduce((sum, part) => sum + part.length + 1, 0) +
          endColumn -
          1;
        return { message, line, column, offset, length: end - offset };
      });
      assert.deepEqual(
        actual.map(({ message, labels }) => ({ message, ...labels[0].span })),
        expected,
        name,
      );
    }
  }, 20_000);
});
