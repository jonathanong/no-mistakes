import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint, require } from "./helpers.mjs";

const rule = "no-mistakes/test-no-error-message-matching";
function diagnostics(name, sourceType = "module") {
  return lint(
    fixture(`test-no-error-message-matching/${name}`),
    { [rule]: "error" },
    name,
    {},
    sourceType,
  ).map(({ messageId, ruleId, line }) => ({ messageId, ruleId, line }));
}

describe("error-message assertion binding boundaries", () => {
  it("reports literal-named ESM imports at the exact assertion", () => {
    assert.deepEqual(diagnostics("literal-import.test.ts"), [
      { messageId: "message", ruleId: rule, line: 3 },
    ]);
  });

  it("reports computed CommonJS destructuring through its default wrapper", () => {
    assert.deepEqual(diagnostics("computed-require.test.cjs", "commonjs"), [
      { messageId: "message", ruleId: rule, line: 3 },
    ]);
  });

  it("keeps unrelated objects, loaders and factories outside runner bindings", () => {
    for (const name of [
      "ordinary-object.test.js",
      "custom-loader.test.js",
      "factory-loader.test.js",
      "redeclared-custom.test.js",
      "callee-factory.test.js",
    ]) {
      assert.deepEqual(diagnostics(name), [], name);
    }
  });

  it("accepts a name-only compatibility property without inventing a key", () => {
    // Compatibility AST adapters can omit type; real parser nodes are covered above.
    const { propertyName } = require("../src/rules/test-message-expect.js");
    assert.equal(propertyName({ name: "message" }), "message");
    assert.equal(propertyName({ name: 42 }), null);
  });
});
