import assert from "node:assert/strict";
import { resolve, dirname } from "node:path";
import { describe, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { fixture, __dirname, plugin } from "./helpers.mjs";

function reports(name) {
  const filename = resolve(
    __dirname,
    `../../../test-cases/eslint-snippets/fixture/executor-collection-key/${name}`,
  );
  return new Linter({ cwd: dirname(filename) })
    .verify(
      fixture(`executor-collection-key/${name}`),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: { project: resolve(dirname(filename), "tsconfig.json") },
        },
        plugins: { "no-mistakes": plugin },
        rules: {
          "no-mistakes/postgres-no-executor-adapter": ["error", { importSpecifier: "@example/db" }],
        },
      },
      { filename },
    )
    .map((message) => [message.line, message.messageId]);
}

describe("postgres-no-executor-adapter cast writes", () => {
  it("invalidates typed collection methods written through assertions", () => {
    assert.deepEqual(reports("cast-write-direct.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("cast-write-delete.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("cast-write-computed.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("cast-write-dynamic-member.ts"), [[6, "passed"]]);
    assert.deepEqual(reports("cast-write-proto.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("cast-write-destructure.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("cast-write-constructor-prototype.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("cast-write-iteration.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("cast-write-globalthis.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("cast-write-dynamic-globalthis.ts"), [[6, "passed"]]);
    assert.deepEqual(reports("cast-write-shadowed-globalthis.ts"), []);
    assert.deepEqual(reports("cast-write-source-facts.ts"), [[11, "passed"]]);
  });
});
