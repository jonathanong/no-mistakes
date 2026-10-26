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
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/proven-consumers",
);
const expected = [12, 13, 14, 15, 17, 19, 20, 21, 28, 29, 30, 31, 32, 33, 34, 35, 38];
it.each([false, true])(
  "handles assertions, rejection handlers, and proven inline consumers (typed=%s)",
  (typed) => {
    const filename = resolve(root, "disposition-observers.ts");
    const messages = new Linter({ cwd: dirname(filename) }).verify(
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
        rules: {
          "no-mistakes/async-call-disposition": [
            "error",
            {
              targets: [
                { sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] },
              ],
            },
          ],
        },
      },
      { filename },
    );
    assert.deepEqual(
      messages.map(({ line, messageId }) => [line, messageId]),
      expected.map((line) => [line, "disposition"]),
    );
  },
);
it("checks disposition observers under real type-aware Oxlint", () => {
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--type-aware",
      "--deny-warnings",
      "--config",
      resolve(root, ".oxlintrc-proven.json"),
      "--format",
      "json",
      resolve(root, "disposition-observers.ts"),
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 1, result.stderr);
  assert.deepEqual(
    JSON.parse(result.stdout).diagnostics.map((item) => [item.labels[0].span.line, item.code]),
    expected.map((line) => [line, "no-mistakes(async-call-disposition)"]),
  );
});

it("accepts unshadowed global assertions but requires observation", () => {
  const filename = resolve(root, "disposition-global-expect.ts");
  const messages = new Linter().verify(
    readFileSync(filename, "utf8"),
    {
      files: ["**/*.ts"],
      languageOptions: { parser: tsParser },
      plugins: { "no-mistakes": plugin },
      rules: {
        "no-mistakes/async-call-disposition": [
          "error",
          {
            targets: [
              { sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] },
            ],
          },
        ],
      },
    },
    { filename },
  );
  assert.deepEqual(
    messages.map(({ line, messageId }) => [line, messageId]),
    [[6, "disposition"]],
  );
});
