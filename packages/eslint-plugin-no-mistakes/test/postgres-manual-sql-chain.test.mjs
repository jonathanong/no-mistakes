import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, lint, __dirname, plugin } from "./helpers.mjs";

const id = "postgres-no-manual-transaction";
const directory = "postgres-manual-sql-chain";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", directory);
const options = { importSpecifier: "@example/db" };
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
function check(name, extra = {}) {
  return lint(
    fixture(`${directory}/${name}`),
    { [`no-mistakes/${id}`]: ["error", { ...options, ...extra }] },
    resolve(root, name),
    {},
    name.endsWith(".cjs") ? "script" : "module",
  );
}

function chainCase(name, extra = {}) {
  return {
    code: fixture(`${directory}/${name}`),
    filename: resolve(root, name),
    options: [{ ...options, ...extra }],
    languageOptions: {
      ...(name.endsWith(".ts") ? { parser: tsParser } : {}),
      ...(name.endsWith(".cjs") ? { sourceType: "script" } : {}),
    },
  };
}

const suite = {
  valid: [
    // keeps uncertain values and cross-scope/dynamic reads unknown.
    // Zero findings.
    chainCase("controls.ts"),
    chainCase("dynamic.cjs"),
    chainCase("rewrites.ts"),
    // honors suppression, explicit empty scope and owner files.
    // Zero findings.
    chainCase("file.ts"),
    chainCase("recall.ts", { importSpecifier: "", executorNames: [] }),
    chainCase("recall.ts", { owners: ["recall.ts"] }),
  ],
  invalid: [
    // finds literal/template chains through existing static executor forms.
    // Six manualTransaction findings.
    chainCase("recall.ts"),
    // preserves released one-hop behavior.
    // Three manualTransaction findings.
    chainCase("legacy.ts"),
    // honors suppression, explicit empty scope and owner files.
    // One manualTransaction finding.
    chainCase("suppressed.ts"),
  ],
};

it("snapshots postgres-no-manual-transaction diagnostics", async () => {
  await expect(snapshotDocument(snapshotSuite(id, languageOptions, suite))).toMatchFileSnapshot(
    "snapshots/postgres-manual-sql-chain.snap",
  );
}, 30_000);

describe("manual transaction primitive const SQL chains", () => {
  it("does not autofix", () => {
    assert.equal(plugin.rules[id].meta.fixable, undefined);
  });
  it("matches exact ESLint and real Oxlint diagnostics", () => {
    for (const name of [
      "recall.ts",
      "controls.ts",
      "legacy.ts",
      "suppressed.ts",
      "file.ts",
      "dynamic.cjs",
      "rewrites.ts",
    ]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc.json"),
          "--format",
          "json",
          resolve(root, name),
        ],
        { encoding: "utf8" },
      );
      const expected = check(name);
      assert.equal(result.status, expected.length ? 1 : 0, result.stderr);
      const actual = JSON.parse(result.stdout).diagnostics.filter((d) => d.code.includes(id));
      assert.deepEqual(
        actual
          .map((d) => [d.labels[0].span.line, d.labels[0].span.column, d.message])
          .sort((a, b) => a[0] - b[0]),
        expected.map((d) => [d.line, d.column, d.message]),
        name,
      );
    }
  });
});
