import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint } from "./helpers.mjs";

const rule = "ts-preserve-null-option-defaults";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
const cases = [
  ["invalid", [7, 7, 13, 20]],
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

describe("nullable option scalar const aliases", () => {
  for (const [name, lines] of cases) {
    it(`preserves the saved ${name} decisions`, () => {
      const actual = diagnostics(name);
      assert.deepEqual(
        actual.map(({ line, messageId }) => [line, messageId]),
        lines.map((line) => [line, "default"]),
      );
      for (const diagnostic of actual) {
        assert.match(diagnostic.message, /Preserve explicit null and check undefined explicitly/);
      }
    });
  }

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
