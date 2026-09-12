import { describe, expect, it } from "vitest";

import { require } from "./helpers.mjs";

const {
  collectExportedComponents,
  normalizedComponentOptions,
  shouldCheckComponent,
} = require("../src/exported-components.js");
const { functionFromExpression, isFunctionNode, nonEmptyStrings } = require("../src/component-functions.js");
const { createReactNodeFacts, keyName, typeAnnotation, typeName } = require("../src/react-node-types.js");
const { jsxTreeHasAttribute, returnedJsxBranches } = require("../src/returned-jsx.js");

describe("component-functions", () => {
  it("falls back when wrappers are not a non-empty string list", () => {
    expect(nonEmptyStrings(undefined, ["memo"])).toEqual(["memo"]);
    expect(nonEmptyStrings(["", ""], ["memo"])).toEqual(["memo"]);
  });

  it("unwraps configured wrappers and typed expressions", () => {
    const opts = { wrappers: new Set(["memo"]) };
    expect(functionFromExpression(null, opts)).toBeNull();
    expect(
      functionFromExpression(
        {
          type: "CallExpression",
          callee: { type: "Identifier", name: "memo" },
          arguments: [{ type: "ArrowFunctionExpression" }],
        },
        opts,
      ).type,
    ).toEqual("ArrowFunctionExpression");
    expect(
      functionFromExpression(
        {
          type: "CallExpression",
          callee: { type: "MemberExpression", computed: true, property: { name: "memo" } },
          arguments: [{ type: "Literal", value: 1 }],
        },
        opts,
      ),
    ).toBeNull();
  });

  it("rejects unknown node types as functions", () => {
    expect(isFunctionNode({ type: "Identifier" })).toBe(false);
  });
});

describe("react-node-types", () => {
  it("covers key and type-name fallbacks", () => {
    expect(keyName(null)).toBeNull();
    expect(keyName({ type: "Identifier", name: "children" })).toEqual("children");
    expect(keyName({ type: "Literal", value: 1 })).toEqual("1");
    expect(keyName({ type: "MemberExpression" })).toBeNull();
    expect(typeAnnotation(null)).toBeNull();
    expect(
      typeName({
        type: "TSParenthesizedType",
        typeAnnotation: {
          type: "TSTypeReference",
          typeName: {
            type: "TSQualifiedName",
            left: { type: "Identifier", name: "React" },
            right: { type: "Identifier", name: "ReactNode" },
          },
        },
      }),
    ).toEqual("React.ReactNode");
    expect(
      typeName({
        type: "TSTypeReference",
        typeName: { type: "TSQualifiedName", left: { type: "ThisType" }, right: { type: "Identifier" } },
      }),
    ).toBeNull();
    expect(typeName({ type: "TSUnionType" })).toBeNull();
  });

  it("tracks ReactNode aliases, heritage, and object props", () => {
    const facts = createReactNodeFacts({
      body: [
        {
          type: "ImportDeclaration",
          source: { value: "react" },
          specifiers: [
            {
              type: "ImportSpecifier",
              imported: { type: "Identifier", name: "ReactNode" },
              local: { name: "Node" },
            },
            { type: "ImportDefaultSpecifier", local: { name: "React" } },
          ],
        },
        {
          type: "ExportNamedDeclaration",
          declaration: {
            type: "TSTypeAliasDeclaration",
            id: { name: "Alias" },
            typeAnnotation: { type: "TSTypeReference", typeName: { type: "Identifier", name: "Node" } },
          },
        },
        {
          type: "TSInterfaceDeclaration",
          id: { name: "Base" },
          body: {
            body: [
              {
                type: "TSPropertySignature",
                key: { type: "Identifier", name: "title" },
                typeAnnotation: {
                  typeAnnotation: {
                    typeAnnotation: { type: "TSTypeReference", typeName: { type: "Identifier", name: "Node" } },
                  },
                },
              },
              { type: "TSMethodSignature" },
            ],
          },
        },
        {
          type: "TSInterfaceDeclaration",
          id: { name: "Props" },
          extends: [{ expression: { type: "Identifier", name: "Base" } }, { expression: { type: "Literal" } }],
          body: { body: [] },
        },
        {
          type: "TSTypeAliasDeclaration",
          id: { name: "LiteralProps" },
          typeAnnotation: {
            type: "TSTypeLiteral",
            members: [
              {
                type: "TSPropertySignature",
                key: { type: "Literal", value: "slot" },
                typeAnnotation: {
                  typeAnnotation: {
                    typeAnnotation: { type: "TSTypeReference", typeName: { type: "Identifier", name: "Alias" } },
                  },
                },
              },
            ],
          },
        },
      ],
    });
    expect(facts.reactNodeNames.has("Node")).toBe(true);
    expect(facts.aliases.get("Alias")).toBe(true);
    expect(facts.objectProps.size).toBeGreaterThan(0);
  });
});

describe("exported-components", () => {
  const opts = normalizedComponentOptions({
    componentNamePattern: "",
    components: "not-an-array",
    ignoreComponents: ["/Ignore.*/"],
    wrappers: ["memo"],
    exportTypes: ["named", "default"],
    checkAnonymousDefault: true,
  });

  it("matches ignore lists, regex matchers, and anonymous defaults", () => {
    expect(shouldCheckComponent({ name: "Ignored", anonymousDefault: false }, opts)).toBe(false);
    expect(shouldCheckComponent({ name: "Button", anonymousDefault: false }, opts)).toBe(true);
    expect(shouldCheckComponent({ anonymousDefault: true }, opts)).toBe(true);
    const namedOnly = normalizedComponentOptions({
      components: ["Exact", "/^Icon/"],
      exportTypes: ["named"],
    });
    expect(shouldCheckComponent({ name: "Exact" }, namedOnly)).toBe(true);
    expect(shouldCheckComponent({ name: "IconHome" }, namedOnly)).toBe(true);
    expect(shouldCheckComponent({ name: "other" }, namedOnly)).toBe(false);
  });

  it("collects named, default, and duplicate exported components", () => {
    const fn = { type: "FunctionDeclaration", id: { name: "Button" }, range: [1, 2], loc: { start: { line: 1 } } };
    const duplicate = { type: "FunctionDeclaration", id: { name: "Button" }, loc: { start: { line: 1 } } };
    const program = {
      body: [
        { type: "FunctionDeclaration", id: { name: "Local" } },
        {
          type: "VariableDeclaration",
          declarations: [
            { id: { type: "ObjectPattern" }, init: null },
            {
              id: { type: "Identifier", name: "Wrapped" },
              init: {
                type: "CallExpression",
                callee: { type: "Identifier", name: "memo" },
                arguments: [{ type: "ArrowFunctionExpression" }],
              },
            },
          ],
        },
        {
          type: "ExportNamedDeclaration",
          source: { value: "./other" },
          specifiers: [],
        },
        {
          type: "ExportNamedDeclaration",
          declaration: { type: "FunctionDeclaration", id: { name: "Button" }, range: [1, 2] },
        },
        {
          type: "ExportNamedDeclaration",
          declaration: {
            type: "VariableDeclaration",
            declarations: [
              {
                id: { type: "Identifier", name: "MemoButton" },
                init: {
                  type: "CallExpression",
                  callee: { type: "Identifier", name: "memo" },
                  arguments: [{ type: "ArrowFunctionExpression" }],
                },
              },
            ],
          },
        },
        {
          type: "ExportNamedDeclaration",
          specifiers: [
            { local: { type: "Identifier", name: "Local" }, exported: { name: "default" } },
            { local: { type: "Identifier", name: "Wrapped" }, exported: { name: "Wrapped" } },
            { local: { type: "Literal" } },
          ],
        },
        {
          type: "ExportDefaultDeclaration",
          declaration: { type: "Identifier", name: "Local" },
        },
        {
          type: "ExportDefaultDeclaration",
          declaration: { type: "FunctionDeclaration", id: null, range: [9, 10] },
        },
        {
          type: "ExportDefaultDeclaration",
          declaration: { type: "FunctionDeclaration", id: { name: "NamedDefault" }, range: [11, 12] },
        },
        {
          type: "ExportDefaultDeclaration",
          declaration: { type: "ArrowFunctionExpression", id: { name: "Anon" }, range: [13, 14] },
        },
      ],
    };
    const components = collectExportedComponents(program, opts);
    expect(components.some((component) => component.name === "Button")).toBe(true);
    const again = collectExportedComponents(
      { body: [{ type: "ExportNamedDeclaration", declaration: { type: "FunctionDeclaration", id: { name: "Button" }, fn } }] },
      opts,
    );
    expect(
      collectExportedComponents(
        {
          body: [
            { type: "ExportNamedDeclaration", declaration: { type: "FunctionDeclaration", id: { name: "Button" }, ...fn } },
            { type: "ExportNamedDeclaration", specifiers: [{ local: { type: "Identifier", name: "Missing" } }] },
          ],
        },
        opts,
      ).some((component) => component.name === "Button"),
    ).toBe(true);
    expect(duplicate.loc.start.line).toEqual(1);
    expect(again.length).toBeGreaterThan(0);
  });

  it("covers default-only exports, non-function initializers, and loc uniqueness", () => {
    const defaultOnly = normalizedComponentOptions({
      exportTypes: ["default"],
      checkAnonymousDefault: false,
      wrappers: ["memo"],
    });
    collectExportedComponents(
      {
        body: [
          {
            type: "VariableDeclaration",
            declarations: [{ id: { type: "Identifier", name: "NotFn" }, init: { type: "Literal", value: 1 } }],
          },
          {
            type: "ExportNamedDeclaration",
            declaration: { type: "FunctionDeclaration", id: { name: "Button" }, loc: { start: { line: 2 } } },
          },
          {
            type: "ExportDefaultDeclaration",
            declaration: { type: "FunctionDeclaration", id: null, loc: { start: { line: 3 } } },
          },
          {
            type: "ExportDefaultDeclaration",
            declaration: { type: "ArrowFunctionExpression", loc: { start: { line: 5 } } },
          },
        ],
      },
      defaultOnly,
    );
    const named = normalizedComponentOptions({ exportTypes: ["named"] });
    const locFn = { type: "FunctionDeclaration", id: { name: "Dup" }, loc: { start: { line: 8 } } };
    const dupes = collectExportedComponents(
      {
        body: [
          locFn,
          { type: "ExportNamedDeclaration", declaration: locFn },
          {
            type: "ExportNamedDeclaration",
            specifiers: [{ local: { type: "Identifier", name: "Dup" }, exported: { name: "Dup" } }],
          },
        ],
      },
      named,
    );
    expect(dupes.filter((component) => component.name === "Dup").length).toBeGreaterThan(0);
  });
});

describe("returned-jsx", () => {
  it("skips class bodies and reports missing attributes", () => {
    expect(returnedJsxBranches({ type: "FunctionDeclaration", body: { type: "ClassDeclaration" } })).toEqual(
      [],
    );
    expect(
      returnedJsxBranches({
        type: "ArrowFunctionExpression",
        body: { type: "JSXElement" },
      }),
    ).toEqual([{ type: "JSXElement" }]);
    expect(
      jsxTreeHasAttribute(
        {
          type: "JSXElement",
          openingElement: {
            type: "JSXOpeningElement",
            attributes: [{ type: "JSXSpreadAttribute" }],
          },
        },
        { allowSpreadAttributes: false, attributes: ["data-pw"] },
      ),
    ).toBe(false);
  });
});
