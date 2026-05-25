import { describe, it, expect } from "vitest";

import helpers from "../src/helpers.js";

const { callMethodName, cssSelectorValues, isFetchCall, isSelectorCall, rule } = helpers;

describe("helpers", () => {
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

    it("returns true when scope.set exists but does not shadow fetch", () => {
      const node = { callee: { type: "Identifier", name: "fetch" } };
      const context = {
        sourceCode: {
          getScope: () => ({
            set: new Map(),
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

    it("returns false if scope.set.get finds a local variable binding", () => {
      const node = { callee: { type: "Identifier", name: "fetch" } };
      const context = {
        sourceCode: {
          getScope: () => ({
            set: new Map([["fetch", { defs: [{ type: "Variable" }] }]]),
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
});
