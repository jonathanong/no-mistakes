import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { it } from "vitest";
import { __dirname, plugin } from "./helpers.mjs";

const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/proven-consumers/dependency-copy-spellings",
);
const caller = resolve(root, "caller.ts");
// Dropped consumers, a callback on the right of ||, rest, a key after the spread,
// and an optional member that is not the left of ?? stay reported.
const expected = [38, 50, 56, 68, 80, 86, 98, 104, 116, 122, 134, 140, 146, 152];
const targets = [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] }];

function reports(typed) {
  return new Linter({ cwd: dirname(caller) })
    .verify(
      readFileSync(caller, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed
            ? { project: resolve(root, "tsconfig.json"), disallowAutomaticSingleRunInference: true }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: { "no-mistakes/async-call-disposition": ["error", { targets }] },
      },
      { filename: caller },
    )
    .map(({ line, messageId }) => [line, messageId]);
}

it.each([false, true])("follows the remaining dependency copy spellings (typed=%s)", (typed) => {
  assert.deepEqual(
    reports(typed),
    expected.map((line) => [line, "disposition"]),
  );
});

it("follows the remaining copy spellings under real type-aware Oxlint", () => {
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--type-aware",
      "--deny-warnings",
      "--config",
      resolve(root, "../.oxlintrc-proven.json"),
      "--format",
      "json",
      caller,
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 1, result.stderr);
  assert.deepEqual(
    JSON.parse(result.stdout).diagnostics.map((item) => [item.labels[0].span.line, item.code]),
    expected.map((line) => [line, "no-mistakes(async-call-disposition)"]),
  );
});
