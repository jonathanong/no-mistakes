import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, require } from "./helpers.mjs";

const rule = "ts-no-function-aliases";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function recallCase(name) {
  const filename = `ts-no-function-aliases/${name}`;
  return {
    code: fixture(filename),
    filename,
    languageOptions: { parser: tsParser },
  };
}

const suite = {
  valid: [
    // Line and file suppression remove the finding.
    recallCase("suppress-line.ts"),
    recallCase("suppress-file.ts"),
  ],
  invalid: [
    // 2 alias. Released identifier findings on the private receiver fixture.
    recallCase("private-recall.ts"),
    // 1 alias. Next-line suppression leaves the other finding.
    recallCase("suppress-next-line.ts"),
  ],
};

describe("ts-no-function-aliases recall", () => {
  it("snapshots ts-no-function-aliases-recall diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/ts-no-function-aliases-recall.snap");
  }, 30_000);

  it("ignores non-binding AST nodes when checking shadowed parameter names", () => {
    const { patternHasName } = require("../src/rules/ts-no-function-aliases.js").__test;
    assert.equal(patternHasName({ type: "TSParameterProperty" }, "wrapped"), false);
  });

  it("compares nested member receivers by binding and rejects opaque receiver expressions", () => {
    const { sameReference } = require("../src/rules/ts-no-function-aliases-self.js");
    const dbBinding = { defs: [] };
    const sourceCode = {
      getScope: () => ({ set: new Map([["db", dbBinding]]), upper: null }),
    };
    const receiver = () => ({
      type: "MemberExpression",
      computed: false,
      object: { type: "Identifier", name: "db" },
      property: { type: "Identifier", name: "connection" },
    });
    assert.equal(sameReference(receiver(), receiver(), sourceCode, false), true);
    assert.equal(
      sameReference({ type: "CallExpression" }, { type: "CallExpression" }, sourceCode, false),
      false,
    );
  });
});
