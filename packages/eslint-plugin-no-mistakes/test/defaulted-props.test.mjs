import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { require } from "./helpers.mjs";

describe("defaulted prop helpers", () => {
  it("returns an empty set outside functions", () => {
    const { defaultedPropsForNode } = require("../src/defaulted-props");
    assert.equal(defaultedPropsForNode({ parent: null }).size, 0);
  });

  it("handles function-like nodes without bodies", () => {
    const { defaultedPropsForNode } = require("../src/defaulted-props");
    const node = {
      type: "Identifier",
      name: "testId",
      parent: {
        type: "BlockStatement",
        parent: { type: "FunctionDeclaration", params: [], body: null },
      },
    };
    assert.equal(defaultedPropsForNode(node).size, 0);
    assert.equal(
      defaultedPropsForNode({
        type: "Identifier",
        name: "testId",
        parent: { type: "FunctionDeclaration", params: [], body: { type: "ExpressionStatement" } },
      }).size,
      0,
    );
  });

  it("does not record rest bindings as literal defaults", () => {
    const { __test } = require("../src/defaulted-props");
    assert.equal(
      __test.patternHasLiteralDefault(
        {
          type: "ObjectPattern",
          properties: [{ type: "RestElement" }],
        },
        "rest",
      ),
      false,
    );
  });

  it("walks scope chains when finding variables", () => {
    const { __test } = require("../src/defaulted-props");
    const variable = { name: "testId" };
    assert.equal(__test.findVariable({ variables: [variable], upper: null }, "testId"), variable);
    assert.equal(
      __test.findVariable({ variables: [{ name: "other" }, variable], upper: null }, "testId"),
      variable,
    );
    assert.equal(
      __test.findVariable(
        { variables: [{ name: "other" }], upper: { variables: [variable], upper: null } },
        "testId",
      ),
      variable,
    );
    assert.equal(__test.findVariable(null, "missing"), null);
    assert.equal(__test.findVariable({ variables: [], upper: null }, "missing"), null);
  });
});
