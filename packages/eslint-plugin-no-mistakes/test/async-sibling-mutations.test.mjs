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
const filename = resolve(root, "sibling-mutations.ts");
const expected = [4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 26, 28];
it.each([false, true])("rejects sibling mutation but preserves pure reads (typed=%s)", (typed) => {
  const findings = new Linter({ cwd: dirname(filename) }).verify(
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
    findings.map(({ line, messageId }) => [line, messageId]),
    expected.map((line) => [line, "disposition"]),
  );
});
it("rejects sibling mutations in type-aware Oxlint", () => {
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
      filename,
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 1, result.stderr);
  assert.deepEqual(
    JSON.parse(result.stdout).diagnostics.map((item) => [item.labels[0].span.line, item.code]),
    expected.map((line) => [line, "no-mistakes(async-call-disposition)"]),
  );
});
