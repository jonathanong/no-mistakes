import assert from "node:assert/strict";
import { runInNewContext } from "node:vm";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { describe, it } from "vitest";
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
describe("identifier compatibility corpus from v0.95.0", () => {
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
