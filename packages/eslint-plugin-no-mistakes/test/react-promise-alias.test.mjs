import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const rule = "react-no-use-promise-resolve";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const cases = [
  ["invalid", [4, 6, 9, 13, 15, 16, 20, 23, 30]],
  ["window-read", [4]],
  ["controls", []],
  ["sibling-write", []],
  ["export-sibling", []],
  ["root-assign", []],
  ["root-define", []],
  ["root-reassign", []],
  ["outer-call", []],
  ["outer-apply", []],
  ["window-write", []],
  ["window-alias", []],
  ["self-write", []],
  ["global-write", []],
  ["global-object-assign", []],
  ["global-object-update", []],
  ["global-object-delete", []],
  ["global-object-method", []],
  ["root-patch", []],
  ["other-write", []],
  ["update", []],
  ["delete", []],
  ["dynamic-global", []],
  ["tdz", []],
  ["unbound", []],
  ["globalthis-write", []],
  ["globalthis-define", []],
  ["globalthis-alias", []],
  ["globalthis-chain", []],
  ["with", []],
  ["suppressed", []],
  ["suppressed-file", []],
  // Keep the older direct-identifier decisions, including a local Promise.
  ["legacy", [7, 8]],
];
const filename = (name) => `${rule}/promise-alias-${name}.tsx`;

function aliasCase(name) {
  const file = filename(name);
  return {
    code: fixture(file),
    filename: file,
    languageOptions: { parser: tsParser },
  };
}

const suite = {
  valid: cases.filter(([, lines]) => lines.length === 0).map(([name]) => aliasCase(name)),
  invalid: cases.filter(([, lines]) => lines.length > 0).map(([name]) => aliasCase(name)),
};

describe("React.use global Promise constructor aliases", () => {
  it("snapshots react-promise-alias diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/react-promise-alias.snap");
  }, 30_000);

  it("matches Oxlint for aliases, mutations, scope and suppressions", () => {
    const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture");
    for (const [name, lines] of cases) {
      // Oxlint rejects `with` in module syntax before its JS plugin runs.
      if (name === "with") continue;
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, rule, ".oxlintrc-promise-alias.json"),
          "--format",
          "json",
          resolve(root, filename(name)),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, lines.length ? 1 : 0, `${name}: ${result.stderr}`);
      const actual = JSON.parse(result.stdout).diagnostics.filter(
        (diagnostic) => diagnostic.code === `no-mistakes(${rule})`,
      );
      assert.deepEqual(
        actual.map((diagnostic) => diagnostic.labels[0].span.line),
        lines,
        name,
      );
    }
  }, 20_000);
});
