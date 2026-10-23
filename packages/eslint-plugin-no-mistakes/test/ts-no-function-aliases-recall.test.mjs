import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages, require } from "./helpers.mjs";

describe("ts-no-function-aliases recall", () => {
  it("reports statically forwarded members and call/apply forms while preserving controls", () => {
    assert.deepEqual(
      messages(
        fixture("ts-no-function-aliases/recall.ts"),
        "ts-no-function-aliases",
        undefined,
        "ts-no-function-aliases/recall.ts",
      ),
      Array.from({ length: 26 }, () => "alias"),
    );
  });

  it("keeps all three rule-specific suppression forms", () => {
    assert.deepEqual(
      messages(
        fixture("ts-no-function-aliases/suppress-next-line.ts"),
        "ts-no-function-aliases",
        undefined,
        "ts-no-function-aliases/suppress-next-line.ts",
      ),
      ["alias"],
    );
    for (const name of ["suppress-line.ts", "suppress-file.ts"]) {
      assert.deepEqual(
        messages(
          fixture(`ts-no-function-aliases/${name}`),
          "ts-no-function-aliases",
          undefined,
          `ts-no-function-aliases/${name}`,
        ),
        [],
      );
    }
  });

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
