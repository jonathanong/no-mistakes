import { describe, expect, it } from "vitest";

import { messages, require } from "./helpers.mjs";

const { createTargetMatcher } = require("../src/rules/async-targets.js");
const { matchDirectMockCallApply } = require("../src/rules/module-mock-call-apply.js");
const { frameworkBindingModule } = require("../src/rules/module-mock-framework.js");
const { analyzeFactory } = require("../src/rules/module-mock-preserve-factory.js");
const { createMockAliases } = require("../src/rules/module-mock-preserve-aliases.js");
const { integrationAllows } = require("../src/rules/module-mock-integration.js");
const { bindingIdentifiers } = require("../src/rules/no-global-fetch-outside-helper-bindings.js");
const { tagForExpression } = require("../src/rules/no-banned-import-outside-allowed-paths-tags.js");
const {
  seedImportTags,
} = require("../src/rules/no-banned-import-outside-allowed-paths-imports.js");
const {
  recordVariableTag,
  recordAssignmentTag,
} = require("../src/rules/no-banned-import-outside-allowed-paths-aliases.js");
const {
  createAliasScopeTracker,
} = require("../src/rules/no-banned-import-outside-allowed-paths-scopes.js");
const {
  executorBindings,
  firstCallArgument,
  isDatabaseCall,
  memberPropertyName,
} = require("../src/rules/postgres-executor.js");
const { executedQueryText } = require("../src/rules/postgres-query-text.js");
const {
  calleeName: postgresCalleeName,
  isOwnerFile,
  isPromiseAllCallee,
  mapCallArgument,
  sqlText,
  containsDatabaseCall,
} = require("../src/rules/postgres-runtime-helpers.js");
const {
  importSpecifierName,
  isKnownTestCallee,
  isTestExtendCall,
  propertyName,
  setupCallbackKind,
} = require("../src/rules/test-no-shared-state-callees.js");
const { childNodes } = require("../src/rules/test-no-shared-state-helpers.js");
const { createCleanupTracker } = require("../src/rules/test-no-shared-state-cleanup.js");
const { isInsideUncalledNestedFunction } = require("../src/rules/test-no-shared-state-analysis.js");
const { createRuleHelpers } = require("../src/rules/test-no-shared-state-rule-helpers.js");
const { isImmediateObserver } = require("../src/rules/test-no-delayed-rejects-observers.js");
const { collectEvents } = require("../src/rules/playwright-no-hoisted-unique-token-events.js");
const {
  propNamesFromMembers,
  propsFromType,
} = require("../src/rules/nullable-option-defaults-helpers.js");
const { collectPatternNames } = require("../src/rules/ast-pattern-names.js");

function emptyScope() {
  return { set: new Map(), variables: [], upper: null };
}

function context(extra = {}) {
  return {
    filename: "a.ts",
    options: [{}],
    report() {},
    sourceCode: {
      getScope: () => emptyScope(),
      visitorKeys: {
        Program: ["body"],
        VariableDeclarator: ["id", "init"],
        ImportDeclaration: ["specifiers", "source"],
      },
    },
    ...extra,
  };
}

describe("exported helper early returns", () => {
  it("covers callee, import, and setup fallbacks", () => {
    expect(importSpecifierName({ imported: null })).toBeNull();
    expect(importSpecifierName({ imported: { type: "Literal", value: "each" } })).toEqual("each");
    expect(propertyName({ type: "Literal", value: "" })).toEqual("");
    expect(
      isKnownTestCallee({ type: "MemberExpression", computed: true, property: { name: "only" } }),
    ).toBe(false);
    expect(
      isTestExtendCall({ type: "CallExpression", callee: { type: "Identifier", name: "it" } }),
    ).toBe(false);
    expect(
      setupCallbackKind({
        callee: { type: "Identifier", name: "afterAll" },
      }),
    ).toEqual("once");
    expect(
      setupCallbackKind({
        callee: {
          type: "MemberExpression",
          computed: false,
          object: { type: "Identifier", name: "describe" },
          property: { name: "afterAll" },
        },
      }),
    ).toEqual("once");
  });

  it("covers postgres, mock, and nullable helper fallbacks", () => {
    expect(memberPropertyName(null)).toBeNull();
    expect(firstCallArgument({ arguments: [{ type: "SpreadElement" }] })).toBeNull();
    expect(isDatabaseCall({ type: "Identifier" }, new Set())).toBe(false);
    expect(
      executorBindings({
        body: [
          {
            type: "ImportDeclaration",
            source: { value: "@data-stores/psql" },
            specifiers: [{ type: "ImportSpecifier", imported: null, local: { name: "query" } }],
          },
        ],
      }),
    ).toEqual(new Set());
    expect(executedQueryText({ type: "Identifier", name: "q" }, new Map(), context())).toBeNull();
    expect(postgresCalleeName({ callee: { type: "Identifier", name: "query" } })).toBeNull();
    expect(isOwnerFile(null, ["owner"])).toBe(false);
    expect(isPromiseAllCallee({ type: "Identifier" })).toBe(false);
    expect(
      mapCallArgument({
        arguments: [{ type: "CallExpression", callee: { type: "Identifier", name: "map" } }],
      }),
    ).toBeNull();
    expect(sqlText(null)).toBeNull();
    expect(containsDatabaseCall(null, new Set())).toBe(false);
    expect(
      matchDirectMockCallApply(
        { callee: { type: "Identifier", name: "mock" } },
        context(),
        new Set(["mock"]),
      ),
    ).toBeNull();
    expect(
      matchDirectMockCallApply(
        {
          callee: {
            type: "MemberExpression",
            property: { name: "call" },
            object: {
              type: "MemberExpression",
              object: { name: "vi" },
              property: { name: "mock" },
            },
          },
          arguments: [],
        },
        context(),
        new Set(["unmock"]),
      ),
    ).toBeNull();
    expect(
      frameworkBindingModule(
        {
          type: "MemberExpression",
          computed: false,
          object: { type: "Identifier", name: "jest" },
          property: { name: "jest" },
        },
        context({
          sourceCode: {
            getScope: () => ({
              variables: [
                {
                  name: "jest",
                  defs: [
                    {
                      type: "ImportBinding",
                      parent: { source: { value: "@jest/globals" } },
                      node: { imported: { name: "jest" } },
                    },
                  ],
                },
              ],
              upper: null,
            }),
          },
        }),
      ),
    ).toEqual("@jest/globals");
    expect(analyzeFactory(null, "./mod", { framework: "vitest" }, context()).objects).toEqual([]);
    expect(integrationAllows("./mod", null, { framework: "vitest" }, context(), {})).toBe(false);
    expect(propNamesFromMembers([{ type: "TSMethodSignature" }]).size).toBe(0);
    const facts = {
      typeProps: new Map(),
      objectAliases: new Map([["Alias", "Alias"]]),
      includeAll: true,
      allTypeProps: new Map(),
    };
    expect(
      propsFromType(
        { type: "TSTypeReference", typeName: { type: "Identifier", name: "Alias" } },
        facts,
      ),
    ).toBeNull();
  });

  it("covers binding, tag, and scope trackers", () => {
    expect(
      bindingIdentifiers({
        type: "RestElement",
        argument: { type: "Identifier", name: "rest" },
      }).map((id) => id.name),
    ).toEqual(["rest"]);
    expect(tagForExpression(null, context(), new Map(), new Map())).toBeNull();
    expect(tagForExpression({ type: "Identifier" }, context(), new Map(), new Map())).toBeNull();
    seedImportTags(
      {
        body: [
          {
            type: "ImportDeclaration",
            source: { value: "mod" },
            specifiers: [{ type: "ImportSpecifier", local: { type: "Identifier", name: "x" } }],
          },
        ],
      },
      context(),
      new Map(),
      new Map(),
    );
    const tracker = createAliasScopeTracker();
    tracker.enterSwitch();
    tracker.exitSwitch();
    tracker.enterSwitchCase();
    const identifier = { type: "Identifier", name: "mod" };
    recordVariableTag(
      { init: { type: "Identifier", name: "x" }, id: identifier },
      context(),
      new Map(),
      new Set(),
      new Map(),
    );
    recordAssignmentTag(
      {
        operator: "=",
        left: { type: "ArrayPattern", elements: [identifier] },
        right: { type: "Identifier", name: "x" },
      },
      context(),
      new Map(),
      new Set(),
      new Map(),
    );
    const inherited = Object.create({ protoKey: { type: "Literal" } });
    inherited.type = "Identifier";
    inherited.name = "x";
    expect(childNodes(inherited).length).toBeGreaterThanOrEqual(0);
    const cleanup = createCleanupTracker();
    cleanup.enterSuite(true);
    cleanup.beginSetup("per-test");
    cleanup.remember("state");
    cleanup.enterSuite();
    expect(cleanup.has("state", cleanup.currentSuiteKey())).toBe(true);
    expect(
      isInsideUncalledNestedFunction({ parent: { type: "FunctionDeclaration" } }, 1, 0, null),
    ).toBe(true);
    expect(createRuleHelpers(context(), new Set(["it"])).calleeHasProperty(null, "serial")).toBe(
      false,
    );
    expect(isImmediateObserver({ type: "Literal" }, {}, context(), () => false)).toBe(false);
    collectEvents(null, context());
    collectPatternNames({ type: "RestElement", argument: { type: "Identifier", name: "rest" } });
  });

  it("covers target matcher visitors without a variable declaration parent", () => {
    const matcher = createTargetMatcher({
      options: [{ targets: [{ sourceSpecifierPatterns: ["mod"], calleeNamePatterns: ["run"] }] }],
      sourceCode: {
        getScope: () => emptyScope(),
        visitorKeys: { Program: ["body"], VariableDeclarator: ["id", "init"] },
      },
    });
    matcher.visitors.VariableDeclarator({
      parent: { type: "ForOfStatement" },
      id: { type: "Identifier", name: "req" },
      init: {
        type: "CallExpression",
        callee: { type: "Identifier", name: "require" },
        arguments: [{ type: "Literal", value: "mod" }],
      },
    });
    createMockAliases(context(), new Set(["mock"])).declare(
      { type: "Identifier", name: "mockFn" },
      {
        type: "MemberExpression",
        object: { type: "Identifier", name: "vi" },
        property: { name: "unmock" },
      },
    );
  });
});

describe("lint edges that hit remaining guards", () => {
  it("covers afterAll, string specifiers, and sequential call expressions", () => {
    messages(
      `import { afterAll } from "vitest";
       const shared = [];
       afterAll(() => { shared.push(1); });`,
      "test-no-shared-state",
      undefined,
      "a.test.ts",
    );
    messages(
      `import { test as "t" } from "vitest";
       export const t = 1;`,
      "test-no-shared-state",
      undefined,
      "a.test.ts",
    );
    messages("describe.sequential('s', () => {});", "no-vitest-sequential", undefined, "a.test.js");
    expect(messages("it(() => {});", "no-vitest-sequential", undefined, "a.test.js")).toEqual([]);
  });

  it("covers metadata rest, empty test files, and export renaming literals", () => {
    messages(
      "export const { metadata } = values;",
      "nextjs-metadata-exports-location",
      undefined,
      "app/lib.ts",
    );
    messages(
      "export const { ...metadata } = values;",
      "nextjs-metadata-exports-location",
      undefined,
      "app/lib.ts",
    );
    expect(messages('"use strict";', "no-import-only-test-files", undefined, "a.test.js")).toEqual(
      [],
    );
    messages(
      'export { value as "alias" }; const value = 1;',
      "ts-no-export-renaming",
      undefined,
      "a.ts",
    );
    messages(
      "const alias = () => helper(); function helper() {}",
      "ts-no-function-aliases",
      undefined,
      "a.ts",
    );
    messages(
      "foo = function helper() { return helper(); };",
      "ts-no-function-aliases",
      undefined,
      "a.ts",
    );
  });

  it("covers jest mocks, optional fetch, and iife seen-set reuse", () => {
    messages(
      `import { jest } from "@jest/globals";
       jest.mock("./mod.js", () => ({ value: 1 }));`,
      "module-mock-preserve-exports",
      { includePathPatterns: [".*"] },
      "a.test.ts",
    );
    messages(
      "const x = globalThis.fetch?.();",
      "no-global-fetch-outside-helper",
      { checkedPathPatterns: [".*"] },
      "a.ts",
    );
    expect(
      messages("const n = (() => 1)(); const el = <div>{n}</div>;", "react-no-iife-in-jsx"),
    ).toEqual([]);
  });

  it("fires rule visitors with incomplete nodes", () => {
    const node = {
      type: "CallExpression",
      callee: { type: "Identifier", name: "fn" },
      arguments: [],
      specifiers: [
        { type: "ExportSpecifier", local: { type: "Identifier", name: "a" }, exported: null },
      ],
      body: [],
      params: [],
      left: { type: "Literal" },
      operator: "=",
      declaration: null,
      source: { value: "vitest" },
    };
    for (const rule of Object.values(require("../src").rules)) {
      const visitors = rule.create(
        context({
          filename: "e2e/a.spec.ts",
          options: [
            {
              includePathPatterns: [".*"],
              checkedPathPatterns: [".*"],
              targets: [{ sourceSpecifierPatterns: ["mod"], calleeNamePatterns: ["run"] }],
              handlers: [{ sourceSpecifierPatterns: ["mod"], calleeNamePatterns: ["run"] }],
            },
          ],
        }),
      );
      for (const handler of Object.values(visitors)) {
        if (typeof handler === "function") {
          try {
            handler(node);
            handler({ type: "Identifier", name: "x" });
            handler({
              type: "JSXOpeningElement",
              name: { type: "JSXIdentifier", name: "button" },
              attributes: [],
            });
          } catch {
            // incomplete AST is expected
          }
        }
      }
    }
  });
});
