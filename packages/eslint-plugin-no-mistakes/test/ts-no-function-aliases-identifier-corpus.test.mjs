import assert from "node:assert/strict";
import { runInNewContext } from "node:vm";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, lint, require } from "./helpers.mjs";

const id = "ts-no-function-aliases";
const module = { exports: {} };
runInNewContext(fixture(`${id}/legacy-v095-rule.cjs`), {
  module,
  require: () => require("../src/helpers"),
});
const legacyConfig = {
  files: ["**/*.ts"],
  languageOptions: { parser: tsParser },
  plugins: { legacy: { rules: { [id]: module.exports } } },
  rules: { [`legacy/${id}`]: "error" },
};
const shape = (items) =>
  items.map(({ line, column, endLine, endColumn, messageId }) => ({
    line,
    column,
    endLine,
    endColumn,
    messageId,
  }));

// Saved independently of the implementation: identifier binding origin must
// never affect the released syntactic contract, including its historical quirks.
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const corpus = JSON.parse(fixture(`${id}/identifier-release-corpus.json`));

function corpusCase(code) {
  return {
    code,
    filename: "fixture.ts",
    languageOptions: { parser: tsParser },
  };
}

const suite = {
  valid: corpus.filter((item) => item.expected === 0).map((item) => corpusCase(item.code)),
  invalid: [
    // 34 alias.
    corpusCase(fixture(`${id}/release-controls.ts`)),
    // 2 alias.
    corpusCase(fixture(`${id}/private-recall.ts`)),
    // 366 alias, one per remaining corpus source.
    ...corpus.filter((item) => item.expected !== 0).map((item) => corpusCase(item.code)),
  ],
};

describe("identifier compatibility corpus from v0.95.0", () => {
  it("snapshots ts-no-function-aliases-identifier-corpus diagnostics", async () => {
    await expect(
      snapshotDocument({
        [id]: snapshotSuite(id, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/ts-no-function-aliases-identifier-corpus.snap");
  }, 30_000);

  for (const [name, expected] of [
    ["release-controls.ts", 34],
    ["private-recall.ts", 2],
  ]) {
    it(`retains historical identifier diagnostics in ${name}`, () => {
      const code = fixture(`${id}/${name}`);
      const legacy = new Linter().verify(code, legacyConfig, { filename: "fixture.ts" });
      assert.equal(legacy.length, expected);
      assert.deepEqual(
        shape(lint(code, { [`no-mistakes/${id}`]: "error" }, "fixture.ts")),
        shape(legacy),
      );
    });
  }
  for (const { name, code, expected } of JSON.parse(
    fixture(`${id}/identifier-release-corpus.json`),
  )) {
    it(name, () => {
      const legacy = new Linter().verify(code, legacyConfig, { filename: "fixture.ts" });
      assert.equal(
        legacy.some((item) => item.fatal),
        false,
      );
      assert.equal(legacy.length, expected);
      const current = lint(code, { [`no-mistakes/${id}`]: "error" }, "fixture.ts");
      assert.deepEqual(shape(current), shape(legacy));
    });
  }
});
