import { describe, expect, it } from "vitest";

import { require } from "./helpers.mjs";

const {
  alwaysExits,
  alwaysThrows,
  breakSkipsMatcher,
  caughtThrowCanContinue,
  contains,
} = require("../src/rules/test-no-delayed-rejects-abrupt.js");
const { canReachMatcher } = require("../src/rules/test-no-delayed-rejects-flow.js");
const {
  isNonRejectingHandler,
  isNonRejectingHandlerOrAbsent,
} = require("../src/rules/test-no-delayed-rejects-handlers.js");
const {
  mayThrow,
  possibleCaughtThrowCanContinue,
  thrownCompletionCanReachMatcher,
} = require("../src/rules/test-no-delayed-rejects-transfers.js");
const {
  compilePatterns,
  memberRootAndProperty,
  optionTypeAllowed,
  propsFromType,
  typeIncludesNull,
} = require("../src/rules/nullable-option-defaults-helpers.js");
const {
  recordAssignmentTag,
  setOrClearTag,
} = require("../src/rules/no-banned-import-outside-allowed-paths-aliases.js");

function node(type, extra = {}) {
  return { type, range: extra.range ?? [0, 10], ...extra };
}

function link(parent, child, key) {
  child.parent = parent;
  if (key) parent[key] = child;
  return child;
}

describe("delayed-rejects helpers", () => {
  it("covers throw, loop, switch, and try shapes", () => {
    expect(mayThrow(node("ThrowStatement"))).toBe(true);
    expect(mayThrow(node("ExpressionStatement"))).toBe(false);
    expect(
      mayThrow(
        node("BlockStatement", {
          body: [node("ThrowStatement"), node("ReturnStatement")],
        }),
      ),
    ).toBe(true);
    expect(
      mayThrow(
        node("IfStatement", {
          consequent: node("ThrowStatement"),
          alternate: node("ThrowStatement"),
        }),
      ),
    ).toBe(true);
    expect(mayThrow(node("SwitchStatement", { cases: [node("ThrowStatement")] }))).toBe(true);
    expect(mayThrow(node("LabeledStatement", { body: node("ThrowStatement") }))).toBe(true);
    expect(mayThrow(node("WhileStatement", { body: node("ThrowStatement") }))).toBe(true);
    expect(
      mayThrow(
        node("TryStatement", {
          block: node("BlockStatement", { body: [] }),
          handler: { body: node("BlockStatement", { body: [] }) },
          finalizer: node("ThrowStatement"),
        }),
      ),
    ).toBe(true);

    const matcher = node("Identifier", { range: [20, 21] });
    const tryNode = node("TryStatement", { range: [0, 30] });
    const block = link(tryNode, node("BlockStatement", { range: [1, 5] }), "block");
    const handler = node("BlockStatement", { range: [6, 29], body: [] });
    tryNode.handler = { body: handler, range: [6, 29] };
    handler.parent = tryNode;
    const throwStmt = link(block, node("ThrowStatement", { range: [2, 4] }));
    block.body = [throwStmt];
    expect(possibleCaughtThrowCanContinue(tryNode, matcher)).toBe(true);
    expect(thrownCompletionCanReachMatcher(throwStmt, matcher)).toBe(true);
    expect(alwaysExits(node("ReturnStatement"))).toBe(true);
    expect(alwaysThrows(node("ThrowStatement"))).toBe(true);
    expect(contains(tryNode, throwStmt)).toBe(true);
  });

  it("covers try/catch/finally completion and switch exclusivity", () => {
    const fn = node("FunctionDeclaration", { range: [0, 80] });
    const matcher = node("CallExpression", { range: [50, 55] });
    const tryNode = link(fn, node("TryStatement", { range: [1, 70] }));
    const block = link(tryNode, node("BlockStatement", { range: [2, 20], body: [] }), "block");
    const throwStmt = link(block, node("ThrowStatement", { range: [3, 8] }));
    block.body = [throwStmt];
    const handler = node("CatchClause", { range: [21, 40] });
    const handlerBody = link(
      handler,
      node("BlockStatement", { range: [22, 39], body: [] }),
      "body",
    );
    handler.parent = tryNode;
    tryNode.handler = handler;
    const continueStmt = link(handlerBody, node("ContinueStatement", { range: [23, 28] }));
    handlerBody.body = [continueStmt];
    const loop = node("ForStatement", { range: [0, 75] });
    tryNode.parent = loop;
    loop.body = tryNode;
    continueStmt.parent = handlerBody;
    expect(thrownCompletionCanReachMatcher(throwStmt, matcher)).toBe(false);
    expect(possibleCaughtThrowCanContinue(tryNode, matcher)).toBe(false);
    const exclusiveSwitch = node("SwitchStatement", { range: [0, 40], cases: [] });
    exclusiveSwitch.parent = fn;
    const first = link(exclusiveSwitch, node("SwitchCase", { range: [1, 10], consequent: [] }));
    const second = link(exclusiveSwitch, node("SwitchCase", { range: [11, 30], consequent: [] }));
    exclusiveSwitch.cases = [first, second];
    const suspension = link(second, node("AwaitExpression", { range: [12, 16] }));
    second.consequent = [suspension];
    matcher.parent = first;
    first.consequent = [matcher];
    canReachMatcher(suspension, matcher, fn);
    const finallyTry = node("TryStatement", { range: [0, 40] });
    const innerThrow = node("ThrowStatement", { range: [2, 4] });
    const innerBlock = link(
      finallyTry,
      node("BlockStatement", { range: [1, 5], body: [innerThrow] }),
      "block",
    );
    innerThrow.parent = innerBlock;
    const finalizer = link(finallyTry, node("ReturnStatement", { range: [6, 8] }), "finalizer");
    finallyTry.handler = {
      body: node("BlockStatement", { range: [9, 12], body: [node("ExpressionStatement")] }),
    };
    expect(alwaysExits(finallyTry)).toBe(true);
    expect(
      alwaysThrows(node("TryStatement", { block: throwStmt, handler: { body: throwStmt } })),
    ).toBe(true);
    expect(contains(finalizer, matcher)).toBe(false);
  });

  it("covers jump skippers and reachability", () => {
    const loop = node("ForStatement", { range: [0, 40] });
    const matcher = node("CallExpression", { range: [30, 35] });
    matcher.parent = loop;
    const brk = node("BreakStatement", { range: [2, 3] });
    brk.parent = loop;
    loop.body = brk;
    expect(breakSkipsMatcher(brk, matcher)).toBe(true);
    expect(caughtThrowCanContinue(node("ThrowStatement"), matcher)).toBe(false);
    const fn = node("FunctionDeclaration", { range: [0, 50] });
    const suspension = node("AwaitExpression", { range: [1, 2] });
    suspension.parent = fn;
    expect(canReachMatcher(suspension, matcher, fn)).toBe(true);
  });

  it("covers non-rejecting handler shapes", () => {
    expect(isNonRejectingHandler(null)).toBe(false);
    expect(isNonRejectingHandlerOrAbsent(null)).toBe(true);
    expect(isNonRejectingHandler({ type: "Literal", value: 1 })).toBe(false);
    expect(
      isNonRejectingHandler({
        type: "ArrowFunctionExpression",
        params: [{ type: "Identifier" }, { type: "Identifier" }],
        body: { type: "Literal", value: 1 },
      }),
    ).toBe(false);
    expect(
      isNonRejectingHandler({
        type: "ArrowFunctionExpression",
        params: [],
        body: { type: "Literal", value: 1 },
      }),
    ).toBe(true);
    expect(
      isNonRejectingHandler({
        type: "ArrowFunctionExpression",
        params: [],
        body: { type: "BlockStatement", body: [] },
      }),
    ).toBe(true);
    expect(
      isNonRejectingHandler({
        type: "ArrowFunctionExpression",
        params: [],
        body: {
          type: "BlockStatement",
          body: [{ type: "ReturnStatement", argument: { type: "Literal", value: 1 } }],
        },
      }),
    ).toBe(true);
  });
});

describe("nullable option helpers", () => {
  it("ignores invalid regex and resolves nested types", () => {
    expect(compilePatterns(["("])).toEqual([]);
    expect(typeIncludesNull({ type: "TSNullKeyword" })).toBe(true);
    expect(typeIncludesNull({ type: "TSUnionType", types: [{ type: "TSNullKeyword" }] })).toBe(
      true,
    );
    expect(optionTypeAllowed(null, {}, [])).toBe(true);
    expect(optionTypeAllowed("opts", { optionObjectNames: ["options"] }, [])).toBe(false);
    expect(optionTypeAllowed("opts", { optionObjectNames: ["opts"] }, [])).toBe(true);
    const facts = {
      aliases: new Set(),
      typeProps: new Map([["Opts", new Set(["timeout"])]]),
      objectAliases: new Map([["Alias", "Opts"]]),
      includeAll: true,
      allTypeProps: new Map([["Extra", new Set(["retry"])]]),
    };
    expect([
      ...propsFromType(
        { type: "TSTypeReference", typeName: { type: "Identifier", name: "Alias" } },
        facts,
      ),
    ]).toContain("timeout");
    expect(
      propsFromType(
        {
          type: "TSTypeReference",
          typeName: { type: "Identifier", name: "Readonly" },
          typeArguments: { params: [{ type: "TSTypeLiteral", members: [] }] },
        },
        facts,
      ),
    ).toEqual(new Set());
    expect(
      memberRootAndProperty({
        type: "TSNonNullExpression",
        expression: {
          type: "MemberExpression",
          object: { type: "Identifier", name: "opts" },
          property: { name: "timeout" },
        },
      }),
    ).toEqual({ object: "opts", property: "timeout" });
  });
});

describe("banned-import alias recording", () => {
  it("clears non-equals assignments and skips optional chains", () => {
    const identifier = { type: "Identifier", name: "mod" };
    const variable = { name: "mod" };
    const context = {
      sourceCode: {
        getScope: () => ({
          set: new Map([["mod", variable]]),
          upper: null,
        }),
      },
    };
    const aliasMap = new Map([[variable, { kind: "direct" }]]);
    const cleared = new Set();
    setOrClearTag(identifier, null, context, aliasMap, cleared);
    expect(aliasMap.size).toBe(0);
    recordAssignmentTag(
      { operator: "||=", left: identifier, parent: { type: "ChainExpression" } },
      context,
      aliasMap,
      cleared,
      { banned: [] },
    );
    recordAssignmentTag({ operator: "+=", left: identifier }, context, aliasMap, cleared, {
      banned: [],
    });
    expect(cleared.has(variable)).toBe(true);
  });
});
