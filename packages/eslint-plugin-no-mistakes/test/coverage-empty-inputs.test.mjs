import { describe, expect, it } from "vitest";

import { require } from "./helpers.mjs";

const { createReactNodeFacts } = require("../src/react-node-types.js");
const {
  compileTargets,
  matchesAny,
  patternToRegExp,
} = require("../src/rules/async-patterns.js");
const { createTargetMatcher } = require("../src/rules/async-targets.js");
const {
  nullablePropsFromMembers,
  propNamesFromMembers,
  propsFromType,
  typeIncludesNull,
} = require("../src/rules/nullable-option-defaults-helpers.js");
const { collectTypeProps, createTypeFacts } = require("../src/rules/nullable-option-type-props.js");

describe("empty and invalid helper inputs", () => {
  it("covers empty target and pattern fallbacks", () => {
    expect(compileTargets({}, "targets")).toEqual([]);
    expect(
      compileTargets(
        { targets: [{ sourceSpecifierPatterns: ["("], calleeNamePatterns: ["foo"] }] },
        "targets",
      ).length,
    ).toBeGreaterThan(0);
    expect(patternToRegExp("/(/")).toBeNull();
    expect(matchesAny(1, [])).toBe(false);
  });

  it("covers empty type-member walks", () => {
    expect(nullablePropsFromMembers().size).toBe(0);
    expect(propNamesFromMembers().size).toBe(0);
    expect(typeIncludesNull(null)).toBe(false);
    expect(propsFromType(null, createTypeFacts())).toBeNull();
    createReactNodeFacts({});
    collectTypeProps({}, {}, [], createTypeFacts());
  });

  it("covers target matcher visitors without resolved variables", () => {
    const matcher = createTargetMatcher({
      options: [{ targets: [{ sourceSpecifierPatterns: ["mod"], calleeNamePatterns: ["run"] }] }],
      sourceCode: {
        getScope: () => ({ set: new Map(), variables: [], upper: null }),
        visitorKeys: {
          Program: ["body"],
          ImportDeclaration: ["specifiers", "source"],
          VariableDeclarator: ["id", "init"],
        },
      },
    });
    matcher.visitors.Program({
      type: "Program",
      body: [
        {
          type: "ImportDeclaration",
          source: { value: "mod" },
          specifiers: [
            {
              type: "ImportSpecifier",
              local: { type: "Identifier", name: "run" },
              imported: { type: "Literal", value: "run" },
            },
            { type: "ImportNamespaceSpecifier", local: { type: "Identifier", name: "ns" } },
            { type: "ImportDefaultSpecifier", local: { type: "Identifier", name: "def" } },
          ],
        },
        {
          type: "TSImportEqualsDeclaration",
          importKind: "value",
          id: { type: "Identifier", name: "eq" },
          moduleReference: {
            type: "TSExternalModuleReference",
            expression: { type: "Literal", value: "mod" },
          },
        },
        {
          type: "VariableDeclarator",
          parent: { type: "VariableDeclaration" },
          id: { type: "Identifier", name: "req" },
          init: {
            type: "CallExpression",
            callee: { type: "Identifier", name: "require" },
            arguments: [{ type: "Literal", value: "mod" }],
          },
        },
      ],
    });
    expect(matcher.isTargetCall({ callee: { type: "Identifier", name: "run" } })).toBe(false);
  });
});
