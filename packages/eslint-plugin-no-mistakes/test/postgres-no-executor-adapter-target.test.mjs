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
    .map((message) => `${message.line}:${message.messageId}`);
}

const passed = (lines) => lines.map((line) => `${line}:passed`);

describe("typed executor argument targets", () => {
  it("accepts declared local executor slots, preserving callback reports", () => {
    assert.deepEqual(check("local.ts"), ["11:forward", "17:blocked"]);
  });
  it("keeps reporting unproven, overloaded, rest and spread targets", () => {
    assert.deepEqual(
      check("unsafe.ts"),
      passed([8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 20, 21, 25, 28, 30, 32, 35, 38, 40]),
    );
  });
  it("rejects imported helpers with writes or direct eval, preserving shadowed bindings", () => {
    assert.deepEqual(check("stability.ts", true), passed([5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]));
  });
  it("requires parser type information for imported targets", () => {
    assert.deepEqual(
      check("imported.ts"),
      passed([
        3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 13, 14, 15, 16, 17, 18, 21, 22, 23, 24, 26, 28, 30, 31,
        33, 35,
      ]),
    );
    assert.deepEqual(
      check("imported.ts", true),
      passed([5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 18, 21, 24, 26, 28, 30, 31, 33, 35]),
    );
  });
});
