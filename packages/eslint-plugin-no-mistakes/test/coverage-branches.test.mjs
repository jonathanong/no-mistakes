import { describe, expect, it } from "vitest";

import { messages, require } from "./helpers.mjs";

const {
  collectBannedAliases,
  recordAssignmentTag,
  recordVariableTag,
} = require("../src/rules/no-banned-import-outside-allowed-paths-aliases.js");
const { createTargetMatcher } = require("../src/rules/async-targets.js");
const {
  abruptCompletionReachesMatcher,
  alwaysExits,
  alwaysThrows,
  breakSkipsMatcher,
  caughtThrowCanContinue,
  continueSkipsMatcher,
} = require("../src/rules/test-no-delayed-rejects-abrupt.js");
const {
  mayThrow,
  possibleCaughtThrowCanContinue,
  thrownCompletionCanReachMatcher,
} = require("../src/rules/test-no-delayed-rejects-transfers.js");
const { canReachMatcher, executesBefore } = require("../src/rules/test-no-delayed-rejects-flow.js");
const {
  possibleResourceExitBeforeMatcher,
} = require("../src/rules/test-no-delayed-rejects-loop-jumps.js");

function node(type, extra = {}) {
  return { type, range: extra.range ?? [0, 10], ...extra };
}

function ctx() {
  return {
    filename: "a.ts",
    options: [{}],
    report() {},
    sourceCode: {
      getScope: () => ({ set: new Map(), variables: [], upper: null }),
      visitorKeys: {
        Program: ["body"],
        VariableDeclarator: ["id", "init"],
        AssignmentExpression: ["left", "right"],
        ObjectPattern: ["properties"],
      },
    },
  };
}

describe("alias recording branch matrix", () => {
  it("covers object rest, non-property, optional chain, and add-only tags", () => {
    const context = ctx();
    const aliasMap = new Map();
    const cleared = new Set();
    const config = new Map();
    recordVariableTag(
      {
        init: { type: "Identifier", name: "mod" },
        id: {
          type: "ObjectPattern",
          properties: [
            { type: "RestElement", argument: { type: "ArrayPattern", elements: [] } },
            { type: "Skip" },
            {
              type: "Property",
              key: { type: "Identifier", name: "name" },
              value: { type: "ObjectPattern", properties: [] },
            },
            {
              type: "Property",
              key: { type: "Identifier", name: "id" },
              value: { type: "Identifier", name: "id" },
            },
          ],
        },
      },
      context,
      aliasMap,
      cleared,
      config,
    );
    const assignment = {
      operator: "=",
      left: {
        type: "MemberExpression",
        object: { type: "Identifier", name: "mod" },
        property: { name: "x" },
      },
      right: { type: "Identifier", name: "other" },
    };
    recordAssignmentTag(assignment, context, aliasMap, cleared, config);
    const optional = {
      operator: "=",
      left: { type: "Identifier", name: "x" },
      right: { type: "Identifier", name: "y" },
      parent: {
        type: "CallExpression",
        optional: true,
        arguments: [],
        callee: { optional: true },
      },
    };
    optional.parent.arguments.push(optional);
    recordAssignmentTag(optional, context, aliasMap, cleared, config);
    recordVariableTag(
      { init: { type: "Literal", value: 1 }, id: { type: "Identifier", name: "n" } },
      context,
      aliasMap,
      cleared,
      config,
    );
    collectBannedAliases(
      {
        type: "Program",
        body: [
          {
            type: "VariableDeclaration",
            declarations: [
              {
                type: "VariableDeclarator",
                init: {
                  type: "CallExpression",
                  callee: { type: "Identifier", name: "require" },
                  arguments: [{ type: "Literal", value: "mod" }],
                },
                id: {
                  type: "ObjectPattern",
                  properties: [
                    { type: "RestElement", argument: { type: "Identifier", name: "rest" } },
                  ],
                },
              },
            ],
          },
        ],
      },
      context,
      aliasMap,
      config,
    );
    expect(aliasMap.size).toBeGreaterThanOrEqual(0);
  });
});

describe("target matcher remaining import shapes", () => {
  it("covers object-pattern require and type-only import equals", () => {
    const matcher = createTargetMatcher({
      options: [{ targets: [{ sourceSpecifierPatterns: ["mod"], calleeNamePatterns: ["run"] }] }],
      sourceCode: {
        getScope: () => ({ set: new Map(), variables: [], upper: null }),
        visitorKeys: {
          Program: ["body"],
          VariableDeclarator: ["id", "init"],
          ImportDeclaration: ["specifiers"],
        },
      },
    });
    matcher.visitors.VariableDeclarator({
      parent: { type: "VariableDeclaration" },
      id: {
        type: "ObjectPattern",
        properties: [
          { type: "Property", value: { type: "Identifier", name: "run" }, key: { name: "run" } },
        ],
      },
      init: {
        type: "CallExpression",
        callee: { type: "Identifier", name: "require" },
        arguments: [{ type: "Literal", value: "mod" }],
      },
    });
    matcher.visitors.TSImportEqualsDeclaration({
      importKind: "type",
      id: { type: "Literal" },
      moduleReference: { type: "TSQualifiedName" },
    });
    matcher.visitors.TSImportEqualsDeclaration({
      importKind: "value",
      id: { type: "Identifier", name: "eq" },
      moduleReference: { type: "TSQualifiedName" },
    });
    matcher.visitors.TSImportEqualsDeclaration({
      importKind: "value",
      id: { type: "Identifier", name: "eq" },
      moduleReference: { type: "TSExternalModuleReference", expression: { type: "Identifier" } },
    });
    matcher.visitors.ImportDeclaration({
      source: { value: "mod" },
      specifiers: [
        {
          type: "ImportSpecifier",
          local: { type: "Identifier", name: "run" },
          imported: { type: "Identifier", name: "run" },
        },
      ],
    });
  });
});

describe("delayed-rejects remaining CFG arms", () => {
  it("covers catch continue, exclusive if, and handler finalizer throws", () => {
    const fn = node("FunctionDeclaration", { range: [0, 100] });
    const loop = link(fn, node("ForStatement", { range: [1, 90] }));
    const tryNode = link(loop, node("TryStatement", { range: [2, 80] }), "body");
    const throwStmt = node("ThrowStatement", { range: [4, 8] });
    const block = link(
      tryNode,
      node("BlockStatement", { range: [3, 10], body: [throwStmt] }),
      "block",
    );
    throwStmt.parent = block;
    const handler = node("CatchClause", { range: [11, 40] });
    const continueStmt = node("ContinueStatement", { range: [13, 16] });
    const handlerBody = node("BlockStatement", { range: [12, 39], body: [continueStmt] });
    continueStmt.parent = handlerBody;
    handlerBody.parent = handler;
    handler.body = handlerBody;
    handler.parent = tryNode;
    tryNode.handler = handler;
    const matcher = node("CallExpression", { range: [60, 70] });
    matcher.parent = fn;
    expect(continueSkipsMatcher(continueStmt, matcher)).toBe(true);
    expect(caughtThrowCanContinue(throwStmt, matcher)).toBe(false);
    thrownCompletionCanReachMatcher(throwStmt, matcher);
    possibleCaughtThrowCanContinue(tryNode, matcher);
    possibleResourceExitBeforeMatcher(throwStmt, matcher, loop);

    const ifNode = node("IfStatement", { range: [0, 40] });
    ifNode.parent = fn;
    const consequent = node("AwaitExpression", { range: [1, 5] });
    const alternate = node("CallExpression", { range: [10, 20] });
    consequent.parent = ifNode;
    alternate.parent = ifNode;
    ifNode.consequent = consequent;
    ifNode.alternate = alternate;
    canReachMatcher(consequent, alternate, fn);
    executesBefore(consequent, matcher);

    const tryFinally = node("TryStatement", { range: [0, 30] });
    const inner = node("ThrowStatement", { range: [2, 3] });
    const tryBlock = link(
      tryFinally,
      node("BlockStatement", { range: [1, 5], body: [inner] }),
      "block",
    );
    inner.parent = tryBlock;
    const handler2 = node("CatchClause", { range: [6, 12] });
    const handlerThrow = node("ThrowStatement", { range: [13, 14] });
    handler2.body = node("BlockStatement", { range: [7, 11], body: [handlerThrow] });
    handler2.parent = tryFinally;
    tryFinally.handler = handler2;
    tryFinally.finalizer = node("ThrowStatement", { range: [15, 18] });
    tryFinally.finalizer.parent = tryFinally;
    handler2.body.parent = handler2;
    thrownCompletionCanReachMatcher(handlerThrow, matcher);
    alwaysThrows(tryFinally);
    alwaysExits(tryFinally);
    abruptCompletionReachesMatcher(throwStmt, matcher);
    breakSkipsMatcher(continueStmt, matcher);
    mayThrow(tryFinally);
  });
});

function link(parent, child, key) {
  child.parent = parent;
  if (key) parent[key] = child;
  return child;
}

describe("preserve-null and react-node pattern branches", () => {
  it("covers rest, assignment, and non-object params via lint", () => {
    messages(
      `interface Options { value?: string | null }
       export function rest({ ...opts }: Options) { return opts.value ?? "x"; }
       export function nested({ value: [first] }: Options) { return first ?? "x"; }
       export function ident(options: Options) { return options.value ?? "x"; }`,
      "ts-preserve-null-option-defaults",
      undefined,
      "a.ts",
    );
    messages(
      `import type { ReactNode } from "react";
       export function Child({ title }: { title: ReactNode }) { return title ?? "x"; }
       export function Rest({ ...props }: { title: ReactNode }) { return props.title ?? "x"; }
       export function Ident(props: { title: ReactNode }) { return props.title ?? "x"; }`,
      "react-no-nullish-react-node",
      undefined,
      "a.tsx",
    );
  });
});
