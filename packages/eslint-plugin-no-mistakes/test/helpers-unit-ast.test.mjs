import { describe, it, expect } from "vitest";

import helpers from "../src/helpers.js";

const {
  attributeName,
  isStaticString,
  isStringLiteralNode,
  literalString,
  selectorLiteral,
  selectorValueNode,
  staticTemplate,
} = helpers;

describe("helpers", () => {
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
