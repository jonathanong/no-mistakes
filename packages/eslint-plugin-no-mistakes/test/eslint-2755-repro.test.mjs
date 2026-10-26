import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { it } from "vitest";
import { __dirname, plugin } from "./helpers.mjs";

const root = resolve(__dirname, "../../../test-cases/eslint-plugin/issue-2755/fixture");
const filename = resolve(root, "repro.ts");
const rules = JSON.parse(readFileSync(resolve(root, ".oxlintrc.json"), "utf8")).rules;
const expected = [
  // Manual transaction recall takes precedence over proving a receiver's library.
  [45, "manualTransaction"],
  [46, "manualTransaction"],
  [52, "manualTransaction"],
  [53, "manualTransaction"],
  [61, "manualTransaction"],
  [62, "manualTransaction"],
];
it.each([false, true])(
  "retains transaction findings on unproven receivers from issue #2755 (typed=%s)",
  (typed) => {
    const findings = new Linter({ cwd: root }).verify(
      readFileSync(filename, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed
            ? { project: resolve(root, "tsconfig.json"), disallowAutomaticSingleRunInference: true }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules,
      },
      { filename },
    );
    assert.deepEqual(
      findings.map(({ line, messageId }) => [line, messageId]),
      expected,
    );
  },
);
it("matches the full issue repro under real type-aware Oxlint", () => {
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--type-aware",
      "--deny-warnings",
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      filename,
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 1, result.stderr);
  assert.deepEqual(
    JSON.parse(result.stdout).diagnostics.map(({ labels, code }) => [labels[0].span.line, code]),
    expected.map(([line]) => [line, "no-mistakes(postgres-no-manual-transaction)"]),
  );
});
