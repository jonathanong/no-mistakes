import { describe, expect, it } from "vitest";

import { require } from "./helpers.mjs";

const helpers = require("../src/helpers.js");
const asyncAst = require("../src/rules/async-ast.js");
const { matchDirectMockCallApply } = require("../src/rules/module-mock-call-apply.js");
const moduleMockFramework = require("../src/rules/module-mock-framework.js");
const {
  isInlineTestCallback,
  mutatingCallPropertyName,
} = require("../src/rules/test-no-shared-state-helpers.js");

const { isFetchCall, attributeName, callMethodName } = helpers;
const { findContainingFunction, traverse, unwrapTransparentParent, isUnconditionalBeforeReturn } =
  asyncAst;

describe("helpers coverage", () => {
  it("returns null for namespaced JSX attributes", () => {
    expect(
      attributeName({
        type: "JSXAttribute",
        name: { type: "JSXNamespacedName", name: "foo" },
      }),
    ).toBeNull();
  });

  it("uses the fetch-shadow fallback when scope.set.get is missing", () => {
    const node = { callee: { type: "Identifier", name: "fetch" } };
    const shadowed = {
      sourceCode: {
        getScope: () => ({
          set: {},
          variables: [{ name: "fetch", defs: [{ type: "Variable" }] }],
          upper: null,
        }),
      },
    };
    expect(isFetchCall(node, shadowed)).toBe(false);
    const notShadowed = {
      sourceCode: {
        getScope: () => ({
          set: {},
          variables: [{ name: "other", defs: [{ type: "Variable" }] }],
          upper: { set: null, variables: [], upper: null },
        }),
      },
    };
    expect(isFetchCall(node, notShadowed)).toBe(true);
    expect(callMethodName({ callee: { type: "Identifier", name: "click" } })).toEqual("click");
    expect(callMethodName({ callee: { type: "MemberExpression", computed: true } })).toBeNull();
    expect(helpers.cssSelectorValues("[data-pw=unquoted]", ["data-pw"])).toEqual([
      { attribute: "data-pw", operator: "=", value: "unquoted" },
    ]);
    expect(helpers.cssSelectorValues('[data-pw="x" i]', ["data-pw"])[0].value).toEqual("x");
    expect(helpers.isSelectorCall({ callee: { type: "Identifier", name: "click" } })).toBe(true);
    expect(helpers.options({ options: [] })).toEqual({});
    expect(helpers.canonicalAttribute({})).toEqual("data-pw");
    expect(helpers.selectorAttributes({})).toEqual(["data-testid", "data-pw"]);
    expect(
      helpers.staticTemplate({
        type: "TemplateLiteral",
        expressions: [{ type: "Identifier" }],
        quasis: [{ value: { raw: "a" } }, { value: { raw: "" } }],
      }),
    ).toBe(true);
    expect(helpers.selectorLiteral({ value: { type: "Literal", value: "id" } })).toEqual("id");
    expect(helpers.cssSelectorValues("[data-pw='quoted']", ["data-pw"])[0].value).toEqual("quoted");
    expect(helpers.cssSelectorValues('[data-pw="x" s]', ["data-pw"])[0].value).toEqual("x");
    expect(helpers.cssSelectorValues('[data-pw*="part"]', ["data-pw"])[0].operator).toEqual("*=");
    expect(helpers.isSelectorCall({ callee: { type: "Identifier", name: "notAMethod" } })).toBe(
      false,
    );
    expect(callMethodName({ callee: { type: "CallExpression" } })).toBeNull();
    expect(helpers.isStaticString(null)).toBe(false);
    expect(helpers.isStaticString({ type: "Literal", value: 1 })).toBe(false);
  });
});

describe("async-ast", () => {
  it("walks transparent parents and nested functions", () => {
    const fn = { type: "FunctionExpression" };
    const inner = { type: "Identifier", parent: { type: "TSAsExpression", parent: fn } };
    expect(findContainingFunction(inner)).toEqual(fn);
    expect(unwrapTransparentParent(inner).type).toEqual("TSAsExpression");
    const context = { sourceCode: { visitorKeys: { BlockStatement: ["body"] } } };
    const seen = [];
    traverse(context, null, (node) => seen.push(node));
    traverse(
      context,
      {
        type: "BlockStatement",
        body: [null, { type: "FunctionExpression" }, { type: "Literal", value: 1 }],
      },
      (node) => seen.push(node.type),
    );
    expect(seen).toContain("BlockStatement");
    expect(seen).toContain("Literal");
    const ifParent = { type: "IfStatement" };
    const node = { parent: ifParent };
    ifParent.parent = { type: "BlockStatement" };
    expect(isUnconditionalBeforeReturn(node, ifParent.parent)).toBe(false);
    const forParent = { type: "ForStatement" };
    const loopNode = { parent: forParent };
    expect(isUnconditionalBeforeReturn(loopNode, { type: "BlockStatement" })).toBe(false);
  });
});

describe("module-mock helpers", () => {
  it("matches call/apply mock shapes and ignores unrelated callees", () => {
    expect(
      matchDirectMockCallApply({ callee: { type: "Identifier" } }, {}, new Set(["mock"])),
    ).toBeNull();
    const context = {
      sourceCode: {
        getScope: () => ({ variables: [], upper: null }),
      },
    };
    expect(
      matchDirectMockCallApply(
        {
          callee: {
            type: "MemberExpression",
            property: { name: "call" },
            object: { type: "Identifier" },
          },
        },
        context,
        new Set(["mock"]),
      ),
    ).toBeNull();
  });

  it("covers framework binding fallbacks", () => {
    const { expressionName, frameworkBindingModule } = moduleMockFramework;
    expect(expressionName({ type: "Identifier", name: "vi" })).toEqual("vi");
    expect(expressionName({ type: "MemberExpression", computed: true })).toBeNull();
    expect(
      expressionName({
        type: "MemberExpression",
        computed: false,
        object: { type: "Identifier", name: "vi" },
        property: { name: "fn" },
      }),
    ).toEqual("vi.fn");
    expect(
      frameworkBindingModule({ type: "Literal" }, { sourceCode: { getScope: () => null } }),
    ).toBeNull();
    expect(
      frameworkBindingModule(
        { type: "Identifier", name: "vi" },
        { sourceCode: { getScope: () => ({ variables: [], upper: null }) } },
      ),
    ).toEqual("vitest");
    expect(
      frameworkBindingModule(
        { type: "Identifier", name: "jest" },
        { sourceCode: { getScope: () => ({ variables: [], upper: null }) } },
      ),
    ).toEqual("@jest/globals");
  });
});

describe("test-no-shared-state helpers", () => {
  it("detects inline test callbacks and non-member mutating calls", () => {
    const testCall = {
      type: "CallExpression",
      callee: { type: "Identifier", name: "it" },
    };
    expect(isInlineTestCallback({ parent: testCall })).toBe(true);
    expect(isInlineTestCallback({ parent: { type: "FunctionDeclaration" } })).toBe(false);
    expect(mutatingCallPropertyName({ callee: { type: "Identifier", name: "push" } })).toBeNull();
  });
});
