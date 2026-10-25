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
const expected = [4, 19, 22, 26, 30, 34, 38, 42, 81, 84, 90];
it.each([false, true])(
  "follows bounded helper chains and fails closed for holders (typed=%s)",
  (typed) => {
    const filename = resolve(root, "cross-file-receiver-helper-chain.ts");
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
it("keeps helper captures reported under real type-aware Oxlint", () => {
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
      resolve(root, "cross-file-receiver-helper-chain.ts"),
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 1, result.stderr);
  assert.deepEqual(
    JSON.parse(result.stdout).diagnostics.map((item) => [item.labels[0].span.line, item.code]),
    expected.map((line) => [line, "no-mistakes(async-call-disposition)"]),
  );
});
