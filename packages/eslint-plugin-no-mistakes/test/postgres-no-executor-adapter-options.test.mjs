import assert from "node:assert/strict";
import { resolve, dirname } from "node:path";
import { describe, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { fixture, __dirname, plugin } from "./helpers.mjs";

function check(typed) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/executor-target/options.ts",
  );
  return new Linter({ cwd: dirname(filename) })
    .verify(
      fixture("executor-target/options.ts"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed ? { project: resolve(dirname(filename), "tsconfig.json") } : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: {
          "no-mistakes/postgres-no-executor-adapter": [
            "error",
            { importSpecifier: "@example/db", executorTypeNames: ["QueryExecutor"] },
          ],
        },
      },
      { filename },
    )
    .map((message) => `${message.line}:${message.messageId}`);
}

const passed = (lines) => lines.map((line) => `${line}:passed`);

describe("executor options object targets", () => {
  it("accepts stable declarations whose named property has configured import identity", () => {
    assert.deepEqual(check(true), [
      ...passed([17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 30, 31]),
      "32:forward",
      ...passed([35, 36, 38, 40, 41, 42]),
    ]);
  });
  it("requires typed services for named object declarations", () => {
    assert.deepEqual(check(false), [
      ...passed([
        10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 30, 31,
      ]),
      "32:forward",
      ...passed([34, 35, 36, 37, 38, 40, 41, 42]),
    ]);
  });
});
