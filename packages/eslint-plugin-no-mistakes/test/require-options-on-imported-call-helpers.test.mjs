import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { require } from "./helpers.mjs";

describe("require-options-on-imported-call helpers", () => {
  const { __test } = require("../src/rules/require-options-on-imported-call");

  it("inspects statically visible object keys", () => {
    assert.equal(__test.staticPropertyName({ type: "SpreadElement" }), null);
    assert.equal(
      __test.staticPropertyName({
        type: "Property",
        computed: true,
        key: { type: "Identifier", name: "timeoutMs" },
      }),
      null,
    );
    assert.equal(
      __test.staticPropertyName({
        type: "Property",
        computed: true,
        key: { type: "Literal", value: "timeoutMs" },
      }),
      "timeoutMs",
    );
    assert.equal(
      __test.staticPropertyName({
        type: "Property",
        computed: true,
        key: {
          type: "TemplateLiteral",
          expressions: [],
          quasis: [{ type: "TemplateElement", value: { cooked: "timeoutMs" } }],
        },
      }),
      "timeoutMs",
    );
    assert.deepEqual([...__test.visiblePropertyNames({ type: "Identifier" })], []);
    assert.deepEqual(
      [...__test.visiblePropertyNames({ type: "ObjectExpression", properties: [] })],
      [],
    );
    assert.equal(
      __test.hasRequiredOptions(undefined, {
        propertyMatch: "any",
        requiredProperties: ["timeoutMs"],
      }),
      false,
    );
    assert.equal(
      __test.hasRequiredOptions(
        { type: "ObjectExpression", properties: [{ type: "SpreadElement" }] },
        { propertyMatch: "all", requiredProperties: ["timeoutMs"] },
      ),
      false,
    );
    assert.equal(
      __test.hasRequiredOptions(
        {
          type: "TSAsExpression",
          expression: {
            type: "ObjectExpression",
            properties: [
              {
                type: "Property",
                computed: false,
                key: { type: "Identifier", name: "timeoutMs" },
              },
              {
                type: "Property",
                computed: false,
                key: { type: "Identifier", name: "signal" },
              },
            ],
          },
        },
        { propertyMatch: "all", requiredProperties: ["timeoutMs", "signal"] },
      ),
      true,
    );
    assert.equal(
      __test.hasRequiredOptions(
        {
          type: "ObjectExpression",
          properties: [
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "timeoutMs" },
              value: { type: "Identifier", name: "undefined" },
            },
          ],
        },
        { propertyMatch: "any", requiredProperties: ["timeoutMs"] },
      ),
      false,
    );
    assert.equal(
      __test.hasRequiredOptions(
        {
          type: "ObjectExpression",
          properties: [
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "timeoutMs" },
              value: {
                type: "UnaryExpression",
                operator: "void",
                argument: { type: "Literal", value: 0 },
              },
            },
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "signal" },
              value: { type: "Identifier", name: "signal" },
            },
          ],
        },
        { propertyMatch: "any", requiredProperties: ["timeoutMs", "signal"] },
      ),
      true,
    );
    assert.equal(
      __test.hasRequiredOptions(
        {
          type: "ObjectExpression",
          properties: [
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "timeoutMs" },
              value: {
                type: "TSAsExpression",
                expression: { type: "Identifier", name: "undefined" },
              },
            },
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "signal" },
              value: { type: "Identifier", name: "undefined" },
            },
          ],
        },
        { propertyMatch: "all", requiredProperties: ["timeoutMs", "signal"] },
      ),
      false,
    );
    assert.equal(
      __test.isDefinitelyUndefinedValue({ type: "Identifier", name: "timeoutMs" }),
      false,
    );
    assert.equal(__test.isDefinitelyUndefinedValue({ type: "Literal", value: 0 }), false);
  });

  it("treats parameter, catch, and function-name defs as unstable", () => {
    const { isReassigned, namespaceSourceFromInit } = require("../src/rules/async-target-bindings");
    const id = { type: "Identifier", name: "validateUrl" };
    function mockContext(variable) {
      return {
        sourceCode: {
          getScope: () => ({ variables: variable ? [variable] : [], upper: null }),
        },
      };
    }
    assert.equal(isReassigned(id, mockContext(null)), false);
    assert.equal(
      isReassigned(
        id,
        mockContext({ name: "validateUrl", references: [], defs: [{ type: "FunctionName" }] }),
      ),
      true,
    );
    assert.equal(
      namespaceSourceFromInit({ type: "Literal", value: "ssrf" }, mockContext(null), new Map()),
      null,
    );
    assert.equal(
      namespaceSourceFromInit({ type: "Identifier", name: "ssrf" }, mockContext(null), new Map()),
      null,
    );
  });
});
