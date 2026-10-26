import assert from "node:assert/strict";
import { runInNewContext } from "node:vm";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { describe, it } from "vitest";
import { fixture, lint, require } from "./helpers.mjs";

const id = "ts-no-function-aliases";
function legacyDiagnostics(code) {
  const module = { exports: {} };
  runInNewContext(fixture(`${id}/legacy-v095-rule.cjs`), {
    module,
    require: () => require("../src/helpers"),
  });
  return new Linter().verify(
    code,
    {
      files: ["**/*.ts"],
      languageOptions: { parser: tsParser },
      plugins: { legacy: { rules: { [id]: module.exports } } },
      rules: { [`legacy/${id}`]: "error" },
    },
    { filename: "fixture.ts" },
  );
}

describe("function alias last-release compatibility", () => {
  it("retains released identifier contracts using the frozen v0.95.0 rule", () => {
    const code = fixture(`${id}/legacy-compatibility.ts`);
    const legacy = legacyDiagnostics(code);
    const current = lint(code, { [`no-mistakes/${id}`]: "error" }, "fixture.ts");
    const shape = (diagnostics) => diagnostics.map(({ line, messageId }) => ({ line, messageId }));
    assert.equal(legacy.length, 14);
    assert.deepEqual(shape(current), shape(legacy));
  });
  it("preserves the released distinction between generic call arguments and an instantiated callee", () => {
    const code = fixture(`${id}/legacy-instantiation.ts`);
    const legacy = legacyDiagnostics(code);
    const current = lint(code, { [`no-mistakes/${id}`]: "error" }, "fixture.ts");
    assert.deepEqual(
      legacy.map((item) => item.line),
      [3],
    );
    assert.deepEqual(
      current.map((item) => item.line),
      [3],
    );
  });
  it("preserves released JSDoc-decorated identifier wrappers", () => {
    const code = fixture(`${id}/release-jsdoc.mjs`);
    const legacy = legacyDiagnostics(code);
    const current = lint(code, { [`no-mistakes/${id}`]: "error" }, "fixture.mjs");
    assert.equal(legacy.length, 7);
    assert.deepEqual(
      current.map((item) => item.line),
      legacy.map((item) => item.line),
    );
  });
});
