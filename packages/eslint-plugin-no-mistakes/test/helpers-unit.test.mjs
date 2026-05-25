import { describe, it, expect } from "vitest";
import {
  attributeName,
  callMethodName,
  canonicalAttribute,
  cssSelectorValues,
  isFetchCall,
  isSelectorAttribute,
  isSelectorCall,
  isStringLiteralNode,
  isStaticString,
  literalString,
  options,
  rule,
  selectorAttributes,
  selectorLiteral,
  selectorValueNode,
  staticTemplate,
} from "../src/helpers.js";

describe("helpers", () => {
  describe("options", () => {
    it("returns context.options[0] if it exists", () => {
      const context = { options: [{ foo: "bar" }] };
      expect(options(context)).toEqual({ foo: "bar" });
    });

    it("returns an empty object if options[0] is undefined", () => {
      const context = { options: [] };
      expect(options(context)).toEqual({});
    });
  });

  describe("selectorAttributes", () => {
    it("returns selectorAttributes from option if it exists", () => {
      const option = { selectorAttributes: ["data-test"] };
      expect(selectorAttributes(option)).toEqual(["data-test"]);
    });

    it("returns DEFAULT_SELECTOR_ATTRIBUTES if missing from option", () => {
      const option = {};
      expect(selectorAttributes(option)).toEqual(["data-testid", "data-pw"]);
    });
  });

  describe("canonicalAttribute", () => {
    it("returns canonicalAttribute from option if it exists", () => {
      const option = { canonicalAttribute: "data-test" };
      expect(canonicalAttribute(option)).toEqual("data-test");
    });

    it("returns 'data-pw' if missing from option", () => {
      const option = {};
      expect(canonicalAttribute(option)).toEqual("data-pw");
    });
  });

  describe("isSelectorAttribute", () => {
    it("returns true if attribute is in array", () => {
      expect(isSelectorAttribute("data-testid", ["data-testid", "data-pw"])).toBe(true);
    });

    it("returns false if attribute is not in array", () => {
      expect(isSelectorAttribute("id", ["data-testid", "data-pw"])).toBe(false);
    });
  });
  describe("attributeName", () => {
    it("returns null if attribute is null", () => {
      expect(attributeName(null)).toBeNull();
    });

    it("returns null if type is not JSXAttribute", () => {
      expect(attributeName({ type: "Identifier" })).toBeNull();
    });

    it("returns null if name.type is not JSXIdentifier", () => {
      expect(attributeName({ type: "JSXAttribute", name: { type: "Identifier" } })).toBeNull();
    });

    it("returns the attribute name", () => {
      const attr = { type: "JSXAttribute", name: { type: "JSXIdentifier", name: "data-pw" } };
      expect(attributeName(attr)).toEqual("data-pw");
    });
  });

  describe("literalString", () => {
    it("returns the string value of a Literal", () => {
      const node = { type: "Literal", value: "foo" };
      expect(literalString(node)).toEqual("foo");
    });

    it("returns null for a Literal that is not a string", () => {
      const node = { type: "Literal", value: 123 };
      expect(literalString(node)).toBeNull();
    });

    it("returns joined quasis for a TemplateLiteral without expressions", () => {
      const node = {
        type: "TemplateLiteral",
        expressions: [],
        quasis: [{ value: { raw: "foo" } }, { value: { raw: "bar" } }],
      };
      expect(literalString(node)).toEqual("foobar");
    });

    it("returns null for a TemplateLiteral with expressions", () => {
      const node = {
        type: "TemplateLiteral",
        expressions: [{ type: "Identifier" }],
        quasis: [],
      };
      expect(literalString(node)).toBeNull();
    });

    it("returns null for other node types", () => {
      expect(literalString({ type: "Identifier" })).toBeNull();
    });
  });

  describe("staticTemplate", () => {
    it("returns true if it has expressions and at least one non-empty quasi", () => {
      const node = {
        type: "TemplateLiteral",
        expressions: [{ type: "Identifier" }],
        quasis: [{ value: { raw: "foo" } }],
      };
      expect(staticTemplate(node)).toBe(true);
    });

    it("returns false if all quasis are empty", () => {
      const node = {
        type: "TemplateLiteral",
        expressions: [{ type: "Identifier" }],
        quasis: [{ value: { raw: "" } }],
      };
      expect(staticTemplate(node)).toBe(false);
    });

    it("returns false if there are no expressions", () => {
      const node = {
        type: "TemplateLiteral",
        expressions: [],
        quasis: [{ value: { raw: "foo" } }],
      };
      expect(staticTemplate(node)).toBe(false);
    });

    it("returns false for non-TemplateLiteral nodes", () => {
      expect(staticTemplate({ type: "Literal" })).toBe(false);
    });

    it("returns false for null node", () => {
      expect(staticTemplate(null)).toBe(false);
    });
  });
  describe("selectorLiteral", () => {
    it("returns null if attribute has no value", () => {
      expect(selectorLiteral({ type: "JSXAttribute", value: null })).toBeNull();
    });

    it("returns null if attribute value expression resolves to null (e.g. non string literal)", () => {
      const attr = {
        type: "JSXAttribute",
        value: { type: "Literal", value: 123 },
      };
      expect(selectorLiteral(attr)).toBeNull();
    });

    it("returns literal string directly from attribute value", () => {
      const attr = {
        type: "JSXAttribute",
        value: { type: "Literal", value: "foo" },
      };
      expect(selectorLiteral(attr)).toEqual("foo");
    });

    it("returns literal string from JSXExpressionContainer", () => {
      const attr = {
        type: "JSXAttribute",
        value: {
          type: "JSXExpressionContainer",
          expression: { type: "Literal", value: "foo" },
        },
      };
      expect(selectorLiteral(attr)).toEqual("foo");
    });
  });

  describe("callMethodName", () => {
    it("returns property name for non-computed MemberExpression", () => {
      const node = {
        callee: {
          type: "MemberExpression",
          computed: false,
          property: { name: "getByTestId" },
        },
      };
      expect(callMethodName(node)).toEqual("getByTestId");
    });

    it("returns null for computed MemberExpression", () => {
      const node = {
        callee: {
          type: "MemberExpression",
          computed: true,
          property: { name: "getByTestId" },
        },
      };
      expect(callMethodName(node)).toBeNull();
    });

    it("returns callee name for Identifier", () => {
      const node = {
        callee: { type: "Identifier", name: "getByTestId" },
      };
      expect(callMethodName(node)).toEqual("getByTestId");
    });

    it("returns null for other callee types", () => {
      const node = {
        callee: { type: "CallExpression" },
      };
      expect(callMethodName(node)).toBeNull();
    });
  });

  describe("isSelectorCall", () => {
    it("returns true for known playwright methods", () => {
      const node = { callee: { type: "Identifier", name: "getByTestId" } };
      expect(isSelectorCall(node)).toBe(true);
    });

    it("returns false for unknown methods", () => {
      const node = { callee: { type: "Identifier", name: "unknownMethod" } };
      expect(isSelectorCall(node)).toBe(false);
    });

    it("returns false when method name cannot be resolved", () => {
      const node = { callee: { type: "CallExpression" } };
      expect(isSelectorCall(node)).toBe(false);
    });
  });
  describe("cssSelectorValues", () => {
    it("extracts values from CSS attribute selectors", () => {
      const source = "[data-testid=\"foo\"] [data-pw='bar'] [data-testid=baz]";
      const attrs = ["data-testid", "data-pw"];
      expect(cssSelectorValues(source, attrs)).toEqual([
        { attribute: "data-testid", operator: "=", value: "foo" },
        { attribute: "data-testid", operator: "=", value: "baz" },
        { attribute: "data-pw", operator: "=", value: "bar" },
      ]);
    });

    it("handles different operators and modifiers", () => {
      const source = '[data-testid^="foo" i] [data-testid$="bar" s] [data-testid*="baz"]';
      const attrs = ["data-testid"];
      expect(cssSelectorValues(source, attrs)).toEqual([
        { attribute: "data-testid", operator: "^=", value: "foo" },
        { attribute: "data-testid", operator: "$=", value: "bar" },
        { attribute: "data-testid", operator: "*=", value: "baz" },
      ]);
    });

    it("returns empty array if no matches", () => {
      expect(cssSelectorValues(".class-name", ["data-testid"])).toEqual([]);
    });
  });

  describe("isFetchCall", () => {
    it("returns false if callee is not Identifier 'fetch'", () => {
      const node = { callee: { type: "Identifier", name: "axios" } };
      expect(isFetchCall(node, {})).toBe(false);
    });

    it("returns true if it is a global fetch call", () => {
      const node = { callee: { type: "Identifier", name: "fetch" } };
      const context = {
        sourceCode: {
          getScope: () => ({
            variables: [],
            upper: null,
          }),
        },
      };
      expect(isFetchCall(node, context)).toBe(true);
    });

    it("returns false if fetch is shadowed by a local variable", () => {
      const node = { callee: { type: "Identifier", name: "fetch" } };
      const context = {
        sourceCode: {
          getScope: () => ({
            variables: [
              {
                name: "fetch",
                defs: [{ type: "Variable" }],
              },
            ],
            upper: null,
          }),
        },
      };
      expect(isFetchCall(node, context)).toBe(false);
    });

    it("checks upper scopes for shadowing", () => {
      const node = { callee: { type: "Identifier", name: "fetch" } };
      const context = {
        sourceCode: {
          getScope: () => ({
            variables: [],
            upper: {
              variables: [
                {
                  name: "fetch",
                  defs: [{ type: "Parameter" }],
                },
              ],
              upper: null,
            },
          }),
        },
      };
      expect(isFetchCall(node, context)).toBe(false);
    });

    it("returns true if fetch is in scope but not a local binding (e.g. built-in)", () => {
      const node = { callee: { type: "Identifier", name: "fetch" } };
      const context = {
        sourceCode: {
          getScope: () => ({
            variables: [
              {
                name: "fetch",
                defs: [{ type: "ImplicitGlobalVariable" }],
              },
            ],
            upper: null,
          }),
        },
      };
      expect(isFetchCall(node, context)).toBe(true);
    });
  });
  describe("rule", () => {
    it("wraps meta and create into an object", () => {
      const meta = { type: "problem" };
      const create = () => ({});
      expect(rule(meta, create)).toEqual({ meta, create });
    });
  });

  describe("isStringLiteralNode", () => {
    it("returns true for string literal node", () => {
      const node = { type: "Literal", value: "foo" };
      expect(isStringLiteralNode(node)).toBe(true);
    });

    it("returns false for non-string literal node", () => {
      const node = { type: "Literal", value: 123 };
      expect(isStringLiteralNode(node)).toBe(false);
    });
  });

  describe("isStaticString", () => {
    it("returns true for static string node", () => {
      const node = { type: "Literal", value: "foo" };
      expect(isStaticString(node)).toBe(true);
    });

    it("returns false for null", () => {
      expect(isStaticString(null)).toBe(false);
    });
  });

  describe("selectorValueNode", () => {
    it("returns null if attribute value is missing", () => {
      expect(selectorValueNode({ type: "JSXAttribute", value: null })).toBeNull();
    });

    it("returns null if attribute value is JSXEmptyExpression", () => {
      const attr = {
        type: "JSXAttribute",
        value: {
          type: "JSXExpressionContainer",
          expression: { type: "JSXEmptyExpression" },
        },
      };
      expect(selectorValueNode(attr)).toBeNull();
    });

    it("returns the expression if it is a valid expression", () => {
      const expr = { type: "Literal", value: "foo" };
      const attr = {
        type: "JSXAttribute",
        value: {
          type: "JSXExpressionContainer",
          expression: expr,
        },
      };
      expect(selectorValueNode(attr)).toEqual(expr);
    });

    it("returns the value directly if not a container", () => {
      const val = { type: "Literal", value: "foo" };
      const attr = {
        type: "JSXAttribute",
        value: val,
      };
      expect(selectorValueNode(attr)).toEqual(val);
    });
  });
});
