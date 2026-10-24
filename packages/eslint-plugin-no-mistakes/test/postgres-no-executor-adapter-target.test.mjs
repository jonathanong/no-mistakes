import assert from "node:assert/strict";
import { resolve, dirname } from "node:path";
import { describe, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { fixture, __dirname, plugin } from "./helpers.mjs";

function check(name, typed = false) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/executor-target",
    name,
  );
  return new Linter({ cwd: dirname(filename) })
    .verify(
      fixture(`executor-target/${name}`),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed
            ? {
                project: resolve(dirname(filename), "tsconfig.json"),
              }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: {
          "no-mistakes/postgres-no-executor-adapter": [
            "error",
            {
              importSpecifier: "@example/db",
              executorTypeNames: ["QueryExecutor"],
            },
          ],
        },
      },
      { filename },
    )
    .map((message) => message.messageId);
}

describe("typed executor argument targets", () => {
  it("accepts declared local executor slots, preserving callback reports", () => {
    assert.deepEqual(check("local.ts"), ["forward", "blocked"]);
  });
  it("keeps reporting unproven, overloaded, rest and spread targets", () => {
    assert.deepEqual(check("unsafe.ts"), Array(20).fill("passed"));
  });
  it("requires parser type information for imported targets", () => {
    assert.deepEqual(check("imported.ts"), Array(17).fill("passed"));
    assert.deepEqual(check("imported.ts", true), Array(13).fill("passed"));
  });
});
