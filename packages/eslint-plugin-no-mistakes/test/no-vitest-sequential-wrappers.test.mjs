import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, lint } from "./helpers.mjs";

const rule = "no-vitest-sequential";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function wrapperCase(name, filename) {
  return {
    code: fixture(`no-vitest-sequential/${name}`),
    filename,
    languageOptions: { parser: tsParser },
  };
}

const suite = {
  valid: [
    // Dynamic, non-sequential, shadowed and suppressed wrappers.
    wrapperCase("typed-wrappers-valid.test.ts", "typed.test.ts"),
    wrapperCase("typed-wrappers-valid.test.ts", "typed-wrappers-valid.test.ts"),
  ],
  invalid: [
    // 14 sequential, one per wrapped serial call.
    wrapperCase("typed-wrappers.test.ts", "typed.test.ts"),
    // Same fixture under the filename Oxlint compares.
    wrapperCase("typed-wrappers.test.ts", "typed-wrappers.test.ts"),
  ],
};

describe("no-vitest-sequential typed wrappers", () => {
  it("snapshots no-vitest-sequential-wrappers diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/no-vitest-sequential-wrappers.snap");
  }, 30_000);

  it("matches real Oxlint findings for the saved fixtures", () => {
    const root = resolve(
      __dirname,
      "../../../test-cases/eslint-snippets/fixture/no-vitest-sequential",
    );
    for (const name of ["typed-wrappers.test.ts", "typed-wrappers-valid.test.ts"]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc-wrappers.json"),
          "--format",
          "json",
          resolve(root, name),
        ],
        { encoding: "utf8" },
      );
      const expected = lint(
        fixture(`no-vitest-sequential/${name}`),
        { "no-mistakes/no-vitest-sequential": "error" },
        name,
      );
      assert.equal(result.status, expected.length ? 1 : 0, result.stderr);
      const actual = JSON.parse(result.stdout).diagnostics.filter((diagnostic) =>
        diagnostic.code.includes("no-vitest-sequential"),
      );
      assert.deepEqual(
        actual.map((diagnostic) => diagnostic.labels[0].span.line).sort((a, b) => a - b),
        expected.map((finding) => finding.line),
      );
    }
  });
});
