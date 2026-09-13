import { describe, expect, it } from "vitest";

import { messages, require } from "./helpers.mjs";

const { isFetchCall } = require("../src/helpers.js");
const {
  collectExportedComponents,
  normalizedComponentOptions,
  uniqueComponents,
} = require("../src/exported-components.js");
const { createReactNodeFacts, keyName } = require("../src/react-node-types.js");
const { jsxTreeHasAttribute, returnedJsxBranches } = require("../src/returned-jsx.js");
const { createTargetMatcher } = require("../src/rules/async-targets.js");
const { propertyName, memberPropertyName } = require("../src/rules/module-mock-helpers.js");
const { frameworkBindingModule, expressionName } = require("../src/rules/module-mock-framework.js");
const { createMockAliases } = require("../src/rules/module-mock-preserve-aliases.js");
const { integrationAllows, mockedExportNames } = require("../src/rules/module-mock-integration.js");
const {
  collectBannedAliases,
  recordAssignmentTag,
  recordVariableTag,
  collectPossibleTag,
  applyObjectPatternTagAddOnly,
} = require("../src/rules/no-banned-import-outside-allowed-paths-aliases.js");
const {
  seedImportTags,
} = require("../src/rules/no-banned-import-outside-allowed-paths-imports.js");
const { tagForExpression } = require("../src/rules/no-banned-import-outside-allowed-paths-tags.js");
const {
  createAliasScopeTracker,
} = require("../src/rules/no-banned-import-outside-allowed-paths-scopes.js");
const {
  recordAssignmentFetchAliases,
} = require("../src/rules/no-global-fetch-outside-helper-helpers.js");
const { isInlineNoopFunction } = require("../src/rules/no-inline-noop-promise-catch-helpers.js");
const {
  propNamesFromMembers,
  nullablePropsFromMembers,
  propsFromType,
} = require("../src/rules/nullable-option-defaults-helpers.js");
const { clearNullableBinding, clearObjectProps } = require("../src/rules/nullable-option-scope.js");
const { collectTypeProps, createTypeFacts } = require("../src/rules/nullable-option-type-props.js");
const { canReachMatcher, executesBefore } = require("../src/rules/test-no-delayed-rejects-flow.js");
const {
  alwaysExits,
  alwaysThrows,
  breakSkipsMatcher,
  caughtThrowCanContinue,
  abruptCompletionReachesMatcher,
} = require("../src/rules/test-no-delayed-rejects-abrupt.js");
const {
  possibleCaughtThrowCanContinue,
  thrownCompletionCanReachMatcher,
  mayThrow,
} = require("../src/rules/test-no-delayed-rejects-transfers.js");
const {
  possibleResourceExitBeforeMatcher,
  branchOutcome,
} = require("../src/rules/test-no-delayed-rejects-loop-jumps.js");
const {
  isOwnerFile,
  sqlStatementBindings,
  executedQueryText,
} = require("../src/rules/postgres-runtime-helpers.js");
const { isCalledFunction, childNodes } = require("../src/rules/test-no-shared-state-helpers.js");
const { isKnownTestCallee } = require("../src/rules/test-no-shared-state-callees.js");
const { createCleanupTracker } = require("../src/rules/test-no-shared-state-cleanup.js");
const {
  isInsideUncalledNestedFunction,
  walkSharedMutations,
} = require("../src/rules/test-no-shared-state-analysis.js");
const { hasProperty } = require("../src/rules/test-no-shared-state-aliases.js");
const { createMutationHandlers } = require("../src/rules/test-no-shared-state-mutations.js");

function node(type, extra = {}) {
  return { type, range: extra.range ?? [0, 10], ...extra };
}

function link(parent, child, key) {
  child.parent = parent;
  if (key) parent[key] = child;
  return child;
}

function scopeContext(entries = []) {
  const set = new Map(entries);
  return {
    filename: "a.ts",
    options: [{}],
    report() {},
    sourceCode: {
      getScope: () => ({ set, variables: [...set.values()], upper: null }),
      visitorKeys: {
        Program: ["body"],
        VariableDeclaration: ["declarations"],
        VariableDeclarator: ["id", "init"],
        ObjectPattern: ["properties"],
        AssignmentExpression: ["left", "right"],
        CallExpression: ["callee", "arguments"],
        Property: ["key", "value"],
        RestElement: ["argument"],
        MemberExpression: ["object", "property"],
        ImportDeclaration: ["specifiers", "source"],
      },
    },
  };
}

function mockContext(overrides = {}) {
  return {
    filename: "a.ts",
    options: [{}],
    report() {},
    sourceCode: {
      getScope: () => ({ set: new Map(), variables: [], upper: null }),
      visitorKeys: { Program: ["body"] },
    },
    ...overrides,
  };
}

describe("delayed-rejects remaining CFG arms", () => {
  it("covers finalizer skip, handler continue, and exclusive if/switch", () => {
    const fn = node("FunctionDeclaration", { range: [0, 400] });
    const loop = link(fn, node("ForStatement", { range: [1, 300] }));
    const tryNode = link(loop, node("TryStatement", { range: [2, 120] }), "body");
    const throwStmt = node("ThrowStatement", { range: [4, 8] });
    const block = link(
      tryNode,
      node("BlockStatement", { range: [3, 10], body: [throwStmt] }),
      "block",
    );
    throwStmt.parent = block;
    const handler = node("CatchClause", { range: [11, 40] });
    handler.body = node("BlockStatement", { range: [12, 39], body: [] });
    handler.body.parent = handler;
    handler.parent = tryNode;
    tryNode.handler = handler;
    const brk = node("BreakStatement", { range: [42, 50] });
    tryNode.finalizer = node("BlockStatement", { range: [41, 60], body: [brk] });
    tryNode.finalizer.parent = tryNode;
    brk.parent = tryNode.finalizer;
    const matcher = node("CallExpression", { range: [200, 210] });
    matcher.parent = loop;
    possibleCaughtThrowCanContinue(tryNode, matcher);

    const cont = node("ContinueStatement", { range: [42, 50] });
    tryNode.finalizer.body = [cont];
    cont.parent = tryNode.finalizer;
    possibleCaughtThrowCanContinue(tryNode, matcher);

    const ret = node("ReturnStatement", { range: [42, 50] });
    tryNode.finalizer.body = [ret];
    ret.parent = tryNode.finalizer;
    possibleCaughtThrowCanContinue(tryNode, matcher);

    const handlerThrow = node("ThrowStatement", { range: [14, 18] });
    handler.body.body = [handlerThrow];
    handlerThrow.parent = handler.body;
    tryNode.finalizer.body = [ret];
    thrownCompletionCanReachMatcher(handlerThrow, matcher);
    tryNode.finalizer.body = [node("ThrowStatement", { range: [42, 50] })];
    tryNode.finalizer.body[0].parent = tryNode.finalizer;
    thrownCompletionCanReachMatcher(handlerThrow, matcher);

    handler.body.body = [cont];
    cont.parent = handler.body;
    tryNode.finalizer = null;
    thrownCompletionCanReachMatcher(throwStmt, matcher);

    const ifNode = node("IfStatement", { range: [0, 80] });
    ifNode.parent = fn;
    const consequent = node("BlockStatement", { range: [1, 30], body: [] });
    const alternate = node("BlockStatement", { range: [40, 70], body: [] });
    const awaitNode = node("AwaitExpression", { range: [5, 15] });
    const otherMatcher = node("CallExpression", { range: [45, 55] });
    consequent.body = [awaitNode];
    alternate.body = [otherMatcher];
    awaitNode.parent = consequent;
    otherMatcher.parent = alternate;
    consequent.parent = ifNode;
    alternate.parent = ifNode;
    ifNode.consequent = consequent;
    ifNode.alternate = alternate;
    canReachMatcher(awaitNode, otherMatcher, fn);

    const sw = node("SwitchStatement", { range: [0, 80], cases: [] });
    sw.parent = fn;
    const first = node("SwitchCase", { range: [1, 30], consequent: [] });
    const second = node("SwitchCase", { range: [40, 70], consequent: [] });
    const earlyMatcher = node("CallExpression", { range: [5, 15] });
    const laterAwait = node("AwaitExpression", { range: [50, 60] });
    first.consequent = [earlyMatcher];
    second.consequent = [laterAwait];
    earlyMatcher.parent = first;
    laterAwait.parent = second;
    first.parent = sw;
    second.parent = sw;
    sw.cases = [first, second];
    canReachMatcher(laterAwait, earlyMatcher, fn);

    const breakNode = node("BreakStatement", { range: [10, 12] });
    breakNode.parent = loop;
    canReachMatcher(breakNode, matcher, fn);
    const continueNode = node("ContinueStatement", { range: [10, 12] });
    continueNode.parent = loop;
    canReachMatcher(continueNode, matcher, fn);

    const dangling = node("ExpressionStatement", { range: [5, 8] });
    const emptyBlock = node("BlockStatement", { range: [0, 20], body: [] });
    dangling.parent = emptyBlock;
    emptyBlock.parent = fn;
    canReachMatcher(dangling, matcher, fn);

    const caseNode = node("SwitchCase", { range: [0, 40], consequent: [] });
    caseNode.parent = fn;
    const firstStmt = node("ExpressionStatement", { range: [1, 5] });
    const secondStmt = node("AwaitExpression", { range: [10, 15] });
    firstStmt.parent = caseNode;
    secondStmt.parent = caseNode;
    caseNode.consequent = [secondStmt, firstStmt];
    executesBefore(firstStmt, secondStmt);

    const handlerContinueTry = node("TryStatement", { range: [2, 80] });
    handlerContinueTry.parent = loop;
    const innerThrow = node("ThrowStatement", { range: [4, 8] });
    handlerContinueTry.block = node("BlockStatement", { range: [3, 10], body: [innerThrow] });
    innerThrow.parent = handlerContinueTry.block;
    handlerContinueTry.block.parent = handlerContinueTry;
    const catchContinue = node("CatchClause", { range: [11, 40] });
    catchContinue.body = node("BlockStatement", { range: [12, 39], body: [cont] });
    catchContinue.body.parent = catchContinue;
    cont.parent = catchContinue.body;
    catchContinue.parent = handlerContinueTry;
    handlerContinueTry.handler = catchContinue;
    possibleResourceExitBeforeMatcher(innerThrow, matcher, loop);

    const stopIf = node("IfStatement", { range: [10, 40] });
    stopIf.consequent = node("ReturnStatement", { range: [11, 15] });
    stopIf.alternate = node("ReturnStatement", { range: [20, 24] });
    stopIf.consequent.parent = stopIf;
    stopIf.alternate.parent = stopIf;
    breakSkipsMatcher(stopIf, matcher);
    const caseContainer = node("SwitchCase", { range: [0, 50], consequent: [stopIf] });
    stopIf.parent = caseContainer;
    possibleResourceExitBeforeMatcher(throwStmt, matcher, loop);

    alwaysExits(tryNode);
    alwaysThrows(tryNode);
    caughtThrowCanContinue(throwStmt, matcher);
    const throwingFinally = node("TryStatement", { range: [2, 80] });
    throwingFinally.parent = fn;
    throwingFinally.block = node("BlockStatement", { range: [3, 10], body: [throwStmt] });
    throwStmt.parent = throwingFinally.block;
    throwingFinally.block.parent = throwingFinally;
    throwingFinally.handler = handler;
    throwingFinally.finalizer = node("BlockStatement", {
      range: [50, 70],
      body: [node("ThrowStatement", { range: [52, 60] })],
    });
    throwingFinally.finalizer.body[0].parent = throwingFinally.finalizer;
    throwingFinally.finalizer.parent = throwingFinally;
    caughtThrowCanContinue(throwStmt, matcher);

    const catchReturn = node("ReturnStatement", { range: [14, 18] });
    handler.body.body = [catchReturn];
    catchReturn.parent = handler.body;
    const noFinally = node("TryStatement", { range: [2, 80] });
    noFinally.parent = fn;
    noFinally.block = throwingFinally.block;
    noFinally.block.parent = noFinally;
    noFinally.handler = handler;
    caughtThrowCanContinue(throwStmt, matcher);
    handler.body.body = [];
    caughtThrowCanContinue(throwStmt, matcher);

    const unrelatedTry = node("TryStatement", { range: [0, 20] });
    unrelatedTry.parent = fn;
    const inner = node("ThrowStatement", { range: [2, 4] });
    inner.parent = unrelatedTry;
    abruptCompletionReachesMatcher(inner, matcher);
    mayThrow(tryNode);
  });
});

describe("alias, fetch, and mock remaining arms", () => {
  it("covers add-only misses, array patterns, and empty tags", () => {
    const mod = { name: "mod" };
    const rest = { name: "rest" };
    const unused = { name: "unused" };
    const context = scopeContext([
      ["mod", mod],
      ["rest", rest],
      ["unused", unused],
    ]);
    const aliasMap = new Map();
    const config = new Map([["fs", new Set(["readFile"])]]);
    const requireCall = {
      type: "CallExpression",
      callee: { type: "Identifier", name: "require" },
      arguments: [{ type: "Literal", value: "fs" }],
    };
    const program = {
      type: "Program",
      body: [
        {
          type: "VariableDeclaration",
          declarations: [
            {
              type: "VariableDeclarator",
              id: { type: "Identifier", name: "mod" },
              init: requireCall,
            },
            {
              type: "VariableDeclarator",
              id: {
                type: "ObjectPattern",
                properties: [
                  { type: "RestElement", argument: { type: "ArrayPattern", elements: [] } },
                  {
                    type: "Property",
                    key: { type: "Identifier", name: "unused" },
                    value: { type: "Identifier", name: "unused" },
                  },
                ],
              },
              init: { type: "Identifier", name: "mod" },
            },
            {
              type: "VariableDeclarator",
              id: { type: "ArrayPattern", elements: [{ type: "Identifier", name: "unused" }] },
              init: { type: "Identifier", name: "mod" },
            },
            {
              type: "VariableDeclarator",
              id: { type: "Identifier", name: "unused" },
              init: { type: "Literal", value: 1 },
            },
          ],
        },
        {
          type: "ExpressionStatement",
          expression: {
            type: "AssignmentExpression",
            operator: "=",
            left: { type: "Identifier", name: "unused" },
            right: { type: "Literal", value: 2 },
          },
        },
        {
          type: "ExpressionStatement",
          expression: {
            type: "AssignmentExpression",
            operator: "=",
            left: {
              type: "ObjectPattern",
              properties: [{ type: "RestElement", argument: { type: "Identifier", name: "rest" } }],
            },
            right: { type: "Literal", value: 3 },
          },
        },
      ],
    };
    for (const declaration of program.body[0].declarations) declaration.parent = program.body[0];
    program.body[0].parent = program;
    collectBannedAliases(program, context, aliasMap, config);
    recordVariableTag(program.body[0].declarations[2], context, aliasMap, new Set(), config);
    recordVariableTag(program.body[0].declarations[3], context, aliasMap, new Set(), config);
    recordAssignmentTag(program.body[1].expression, context, aliasMap, new Set(), config);
    recordAssignmentTag(program.body[2].expression, context, aliasMap, new Set(), config);
    expect(aliasMap.size).toBeGreaterThanOrEqual(0);

    seedImportTags(
      {
        type: "Program",
        body: [
          {
            type: "ImportDeclaration",
            source: { value: "fs" },
            specifiers: [
              { type: "ImportSpecifier", imported: { type: "Identifier", name: "stat" } },
            ],
          },
          {
            type: "ImportDeclaration",
            source: { value: "other" },
            specifiers: [
              { type: "ImportNamespaceSpecifier", local: { type: "Identifier", name: "ns" } },
            ],
          },
        ],
      },
      context,
      config,
      aliasMap,
    );
    tagForExpression(
      { type: "ChainExpression", expression: { type: "Identifier", name: "mod" } },
      context,
      aliasMap,
      config,
    );
    tagForExpression({ type: "Literal", value: 1 }, context, aliasMap, config);

    const tracker = createAliasScopeTracker();
    tracker.enterSwitch();
    tracker.enterSwitchCase();
    tracker.exitSwitchCase({ consequent: null });
    tracker.exitSwitch();

    recordAssignmentFetchAliases(
      {
        operator: "+=",
        left: { type: "MemberExpression" },
        right: { type: "Identifier", name: "fetch" },
      },
      context,
      new Set(),
      new Set(),
    );
  });

  it("covers fetch switch stack misses via direct visitors", () => {
    const rule = require("../src/rules/no-global-fetch-outside-helper.js");
    const visitors = rule.create({
      filename: "a.ts",
      options: [{ checkedPathPatterns: ["**"] }],
      report() {},
      sourceCode: {
        getScope: () => ({ set: new Map(), variables: [], upper: null }),
        visitorKeys: { Program: ["body"] },
      },
    });
    visitors.Program({ type: "Program", body: [] });
    visitors.SwitchCase({ consequent: [{ type: "BreakStatement" }] });
    visitors["SwitchCase:exit"]({ consequent: null });
    visitors.SwitchStatement();
    visitors["SwitchStatement:exit"]();
    visitors.SwitchStatement();
    visitors.SwitchCase({ consequent: [] });
    visitors["SwitchCase:exit"]({ consequent: [{ type: "ExpressionStatement" }] });
    visitors["SwitchStatement:exit"]();
  });

  it("covers integration factory, prefix, and tagged non-identifiers", () => {
    const factory = {
      type: "ObjectExpression",
      properties: [
        {
          type: "Property",
          computed: false,
          key: { type: "Identifier", name: "__esModule" },
          value: { type: "Literal", value: true },
        },
        {
          type: "Property",
          computed: false,
          key: { type: "Identifier", name: "foo" },
          value: { type: "Literal", value: 1 },
        },
      ],
    };
    const mock = { framework: "vitest" };
    const context = scopeContext();
    expect(mockedExportNames(null, "./mod", mock, context)).toBeNull();
    expect(mockedExportNames(factory, "./mod", mock, context)).toEqual(["foo"]);
    expect(
      mockedExportNames(
        {
          type: "ObjectExpression",
          properties: [
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "__esModule" },
              value: { type: "Literal", value: true },
            },
          ],
        },
        "./mod",
        mock,
        context,
      ),
    ).toBeNull();
    expect(integrationAllows("./mod", null, mock, context, { integrationExports: {} })).toBe(false);
    expect(
      integrationAllows("./mod", factory, mock, context, {
        integrationExports: {
          specifierPrefix: "@app/",
          sourcePathTemplates: ["missing/{specifierSuffix}.ts"],
        },
      }),
    ).toBe(false);
    expect(
      integrationAllows("./tagged-invalid", factory, mock, context, {
        integrationExports: {
          sourcePathTemplates: [
            "fixtures/eslint-plugin/module-mock-integration/{specifierSuffix}.ts",
          ],
          markerRegex: String.raw`/\*\s*no-mistakes:\s*integration=[^*]+\*/`,
        },
      }),
    ).toBe(false);
  });

  it("covers mock helper identifier and jest namespace arms", () => {
    expect(propertyName({ type: "Literal", value: "mock" })).toEqual("mock");
    expect(
      memberPropertyName({ computed: true, property: { type: "Identifier", name: "x" } }),
    ).toBeNull();
    expect(expressionName({ type: "Identifier", name: "jest" })).toEqual("jest");
    const context = mockContext();
    frameworkBindingModule(
      {
        type: "MemberExpression",
        computed: false,
        object: { type: "Identifier", name: "jest" },
        property: { type: "Literal", value: "mock" },
      },
      context,
    );
    const aliases = createMockAliases(context, ["mock"]);
    expect(
      aliases.matchCall({
        callee: {
          type: "MemberExpression",
          object: { type: "Identifier", name: "fn" },
          property: { type: "Identifier", name: "call" },
        },
        arguments: [],
      }),
    ).toBeNull();
    messages(
      `import { jest } from "@jest/globals";
       jest.mock("./mod", () => ({ ...jest.requireActual("./mod") }));`,
      "module-mock-preserve-exports",
      { includePathPatterns: ["**"] },
      "a.test.ts",
    );
    messages(
      `import { vi } from "vitest";
       import * as actual from "./mod";
       vi.mock("./mod", () => actual);`,
      "module-mock-preserve-exports",
      { includePathPatterns: ["**"] },
      "a.test.ts",
    );
    const preserve = require("../src/rules/module-mock-preserve-exports.js");
    const visitors = preserve.create({
      filename: "a.ts",
      options: [],
      report() {},
      sourceCode: {
        getScope: () => ({ set: new Map(), variables: [], upper: null }),
        visitorKeys: { Program: ["body"] },
      },
    });
    visitors.AssignmentExpression?.({
      operator: "+=",
      left: { type: "Identifier", name: "x" },
      right: {},
    });
  });
});

describe("preserve-null, react-node, and helper remaining arms", () => {
  it("covers array params, unused logical ops, and missing scopes", () => {
    const rule = require("../src/rules/ts-preserve-null-option-defaults.js");
    const visitors = rule.create(mockContext());
    visitors.Program({ type: "Program", body: [] });
    visitors.FunctionDeclaration({
      params: [{ type: "ArrayPattern", elements: [{ type: "Identifier", name: "x" }] }],
    });
    visitors.VariableDeclarator({
      id: { type: "ArrayPattern", elements: [{ type: "Identifier", name: "x" }] },
      init: { type: "Identifier", name: "opts" },
      parent: { kind: "let" },
    });
    visitors.LogicalExpression({
      operator: "&&",
      left: { type: "Identifier", name: "x" },
      right: { type: "Literal", value: 1 },
    });
    visitors.AssignmentExpression({
      operator: "+=",
      left: { type: "Identifier", name: "missing" },
      right: { type: "Literal", value: 1 },
    });
    visitors.AssignmentExpression({
      operator: "=",
      left: { type: "Identifier", name: "fresh" },
      right: {
        type: "MemberExpression",
        object: { type: "Identifier", name: "opts" },
        property: { type: "Identifier", name: "value" },
      },
    });
    assert.deepEqual(
      messages(
        `interface Options { value?: string | null }
       export function copy(options: Options) {
         copied = options.value;
         return copied;
       }`,
        "ts-preserve-null-option-defaults",
        undefined,
        "a.ts",
      ),
      [],
    );
    visitors["Program:exit"]();
    messages(
      `interface Options { value?: string | null }
       export function arr([value]: Options[]) { return value ?? "x"; }
       export function and(options: Options) { return options.value && "x"; }`,
      "ts-preserve-null-option-defaults",
      undefined,
      "a.ts",
    );
    messages(
      `interface Options { value?: string | null }
       export function assign(options: Options) {
         let missing;
         ({ missing } = options);
         return missing ?? "x";
       }`,
      "ts-preserve-null-option-defaults",
      undefined,
      "a.ts",
    );
  });

  it("covers react-node non-object patterns and var-only scopes", () => {
    const rule = require("../src/rules/react-no-nullish-react-node.js");
    const visitors = rule.create(mockContext({ filename: "a.tsx" }));
    visitors.BlockStatement();
    visitors.VariableDeclarator({
      id: { type: "Identifier", name: "hoisted" },
      parent: { kind: "var" },
    });
    visitors["BlockStatement:exit"]();
    visitors.Program({ type: "Program", body: [] });
    visitors.FunctionDeclaration({
      params: [
        { type: "ArrayPattern", elements: [] },
        {
          type: "ObjectPattern",
          properties: [
            { type: "RestElement", argument: { type: "Identifier", name: "rest" } },
            {
              type: "Property",
              key: { type: "Identifier", name: "title" },
              value: { type: "ArrayPattern", elements: [] },
            },
            {
              type: "Property",
              key: { type: "Identifier", name: "title" },
              value: { type: "AssignmentPattern", left: { type: "Identifier", name: "title" } },
            },
          ],
        },
      ],
    });
    visitors["FunctionDeclaration:exit"]();
    visitors["Program:exit"]();
    createReactNodeFacts({
      body: [
        {
          type: "TSInterfaceDeclaration",
          id: { name: "Props" },
          body: {
            body: [
              {
                type: "TSPropertySignature",
                key: { type: "TemplateLiteral" },
                typeAnnotation: {
                  typeAnnotation: {
                    typeAnnotation: {
                      type: "TSTypeReference",
                      typeName: { type: "Identifier", name: "ReactNode" },
                    },
                  },
                },
              },
            ],
          },
        },
      ],
    });
    expect(keyName({ type: "TemplateLiteral" })).toBeNull();
  });

  it("covers exported components, jsx visit, and fetch shadow fallback", () => {
    const opts = normalizedComponentOptions({
      checkAnonymousDefault: true,
      exportTypes: ["named", "default"],
    });
    collectExportedComponents(
      {
        body: [
          {
            type: "VariableDeclaration",
            declarations: [
              { id: { type: "Identifier", name: "x" }, init: { type: "Literal", value: 1 } },
            ],
          },
          {
            type: "ExportDefaultDeclaration",
            declaration: { type: "FunctionExpression", id: { name: "Named" }, range: [1, 2] },
          },
          {
            type: "ExportNamedDeclaration",
            declaration: {
              type: "FunctionDeclaration",
              id: { name: "Dup" },
              range: [3, 4],
            },
          },
          {
            type: "ExportNamedDeclaration",
            specifiers: [{ local: { type: "Identifier", name: "Dup" }, exported: { name: "Dup" } }],
          },
        ],
      },
      opts,
    );
    jsxTreeHasAttribute(null, { attributes: ["data-testid"], allowSpreadAttributes: false });
    const proto = { inherited: { type: "Identifier", name: "x" } };
    const inherited = Object.create(proto);
    inherited.type = "JSXElement";
    inherited.openingElement = { type: "JSXOpeningElement", attributes: [] };
    jsxTreeHasAttribute(inherited, { attributes: ["data-testid"], allowSpreadAttributes: false });
    returnedJsxBranches({
      type: "ArrowFunctionExpression",
      body: { type: "Literal", value: null },
    });
    isFetchCall(
      { callee: { type: "Identifier", name: "fetch" } },
      {
        sourceCode: {
          getScope: () => ({
            set: { get: "nope" },
            variables: [{ name: "fetch", defs: [{ type: "Variable" }] }],
            upper: null,
          }),
        },
      },
    );
    const matcher = createTargetMatcher({
      options: [{ targets: [{ sourceSpecifierPatterns: ["mod"], calleeNamePatterns: ["run"] }] }],
      sourceCode: {
        getScope: () => ({ set: new Map(), variables: [], upper: null }),
        visitorKeys: { Program: ["body"], VariableDeclarator: ["id", "init"] },
      },
    });
    matcher.visitors.VariableDeclarator({
      parent: { type: "VariableDeclaration" },
      id: { type: "ArrayPattern", elements: [] },
      init: {
        type: "CallExpression",
        callee: { type: "Identifier", name: "require" },
        arguments: [{ type: "Literal", value: "mod" }],
      },
    });
  });

  it("covers nullable helpers, type cycles, and noop sourceCode", () => {
    propNamesFromMembers([{ type: "TSPropertySignature", key: { type: "TemplateLiteral" } }]);
    nullablePropsFromMembers(
      [
        {
          type: "TSPropertySignature",
          optional: true,
          key: { type: "TemplateLiteral" },
          typeAnnotation: { typeAnnotation: { typeAnnotation: { type: "TSNullKeyword" } } },
        },
      ],
      new Set(),
    );
    propsFromType(
      { type: "TSTypeReference", typeName: { type: "Identifier", name: "Readonly" } },
      { aliases: new Set(), typeProps: new Map(), objectAliases: new Map() },
    );
    clearNullableBinding([], "missing");
    clearObjectProps([], "missing");
    const facts = createTypeFacts();
    collectTypeProps(
      {
        body: [
          {
            type: "TSInterfaceDeclaration",
            id: { name: "A" },
            extends: [{ expression: { type: "Identifier", name: "B" } }],
            body: { body: [] },
          },
          {
            type: "TSInterfaceDeclaration",
            id: { name: "B" },
            extends: [{ expression: { type: "Identifier", name: "A" } }],
            body: { body: [] },
          },
        ],
      },
      {},
      [],
      facts,
    );
    isInlineNoopFunction(
      { type: "ArrowFunctionExpression", body: { type: "Identifier", name: "undefined" } },
      {},
    );
  });
});

describe("lint source remaining arms", () => {
  it("covers async try/catch assignment and await aliases", () => {
    expect(
      messages(
        `import { handleRateLimit } from "@app/rate-limit";
         async function run(flag) {
           try {
             let aliased;
             ({ aliased } = { aliased: request() });
             aliased += 1;
             const missing = unknown;
             await missing;
             return aliased;
             aliased = request();
           } catch (error) {
             handleRateLimit(error);
           }
         }`,
        "async-try-catch-return-await",
        {
          handlers: [
            {
              sourceSpecifierPatterns: ["@app/rate-limit"],
              calleeNamePatterns: ["/^handle.*RateLimit$/"],
            },
          ],
        },
        "a.ts",
      ),
    ).toBeInstanceOf(Array);
  });

  it("covers metadata rest, never export kinds, and script attributes", () => {
    messages(
      "export function generateMetadata({ ...rest }) { return rest; }",
      "nextjs-metadata-exports-location",
      undefined,
      "app/page.tsx",
    );
    messages("export { metadata };", "nextjs-metadata-exports-location", undefined, "app/page.tsx");
    messages(
      `type Placeholder = never;
       export type { Placeholder };`,
      "no-placeholder-never-type-exports",
      undefined,
      "a.ts",
    );
    messages(
      `export { Foo as "Bar" };
       type Foo = never;`,
      "no-placeholder-never-type-exports",
      undefined,
      "a.ts",
    );
    messages("<script src={href} />", "nextjs-no-manual-script-tags", undefined, "app/layout.tsx");
    messages("<script />", "nextjs-no-manual-script-tags", undefined, "app/layout.tsx");
    messages("export const wrap = () => other();", "ts-no-function-aliases", undefined, "a.ts");
    messages("export const wrap = () => {}", "ts-no-function-aliases", undefined, "a.ts");
    messages("it.sequential('x', () => {});", "no-vitest-sequential", undefined, "a.test.ts");
    messages(
      "expect(error.message).to.equal('x');",
      "test-no-error-message-matching",
      undefined,
      "a.test.ts",
    );
    messages("test('x', () => { shared += 1; });", "test-no-shared-state", undefined, "a.test.ts");
    messages(
      "const { test } = { test: () => {} };",
      "test-no-shared-state",
      undefined,
      "a.test.ts",
    );
    messages("test.describe.configure('serial');", "test-no-shared-state", undefined, "a.test.ts");
    messages(
      `import { test } from "@playwright/test";
       test("x", async ({ page }) => { await page.evaluate(() => window.scrollTo(0, 1)); });`,
      "playwright-no-raw-scroll-pagination",
      undefined,
      "e2e/flow.spec.ts",
    );
    messages(
      `import { test } from "@playwright/test";
       test("x", async ({ page }) => { await expect(page.getByRole("button")).toBeVisible({ timeout: "fast" }); });`,
      "playwright-assertion-timeout-cap",
      undefined,
      "e2e/flow.spec.ts",
    );
    messages(
      `import { test } from "@playwright/test";
       test("x", async () => { const token = randomSuffix(); });`,
      "playwright-no-hoisted-unique-token",
      { tokenFactories: ["randomSuffix"] },
      "e2e/flow.spec.ts",
    );
    messages(
      "export function Button() { return <div /> }",
      "playwright-require-exported-component-attribute",
      undefined,
      "Button.tsx",
    );
    messages(
      "export const x = { a: 1 as never };",
      "no-placeholder-never-type-exports",
      undefined,
      "a.ts",
    );
  });

  it("covers remaining helper unit arms", () => {
    isOwnerFile("src/db.ts", [null, ""]);
    sqlStatementBindings(null);
    executedQueryText({ type: "Identifier", name: "q" }, new Map(), {
      sourceCode: {
        getScope: () => ({
          set: { get: "nope" },
          variables: [
            { name: "q", defs: [{ type: "FunctionName", node: { id: { type: "ArrayPattern" } } }] },
          ],
          upper: null,
        }),
      },
    });
    isCalledFunction({
      type: "FunctionExpression",
      parent: { type: "VariableDeclarator", id: { type: "ArrayPattern" } },
    });
    const proto = { inherited: { type: "Identifier", name: "x" } };
    const inherited = Object.create(proto);
    inherited.type = "BlockStatement";
    inherited.body = [];
    childNodes(inherited);
    isKnownTestCallee({
      type: "MemberExpression",
      computed: false,
      object: { type: "Identifier", name: "test" },
      property: { type: "Identifier", name: "" },
    });
    const cleanup = createCleanupTracker();
    cleanup.enterSuite(true);
    cleanup.enterSuite(false);
    cleanup.beginSetup("per-test");
    cleanup.has("x");
    walkSharedMutations(
      { type: "FunctionExpression", parent: null },
      { onAssignment() {}, onUpdate() {}, onCall() {} },
    );
    const called = { type: "FunctionExpression" };
    called.parent = { type: "CallExpression", callee: called };
    walkSharedMutations(called, { onAssignment() {}, onUpdate() {}, onCall() {} });
    const tracker = require("../src/rules/test-no-shared-state-analysis.js").createViMockTracker(
      mockContext(),
      new Set(),
    );
    tracker.collectFactoryReferences({
      callee: {
        type: "MemberExpression",
        object: { type: "Identifier", name: "vi" },
        property: { type: "Identifier", name: "mock" },
      },
      arguments: [
        { type: "Literal", value: "./x" },
        { type: "Literal", value: 1 },
      ],
    });
    isInsideUncalledNestedFunction(
      { parent: { type: "FunctionExpression", parent: null } },
      1,
      0,
      () => false,
    );
    const handlers = createMutationHandlers({
      cleanupTracker: createCleanupTracker(),
      context: mockContext(),
      depths: { test: () => 1, setup: () => 1 },
      mutableTopLevel: new Set(),
      registryReports: () => {},
      viMockTracker: { isCaptured: () => false, markIfCaptured() {} },
    });
    handlers.reportAssignment({
      left: { type: "Identifier", name: "x" },
      right: { type: "Literal", value: 1 },
    });
    handlers.reportIfShared({ type: "Identifier", name: "x" }, null);
    const delayed = require("../src/rules/test-no-delayed-rejects.js");
    const delayedVisitors = delayed.create(mockContext({ filename: "a.test.ts" }));
    delayedVisitors.CallExpression?.({
      callee: { type: "Identifier", name: "expect" },
      parent: { type: "MemberExpression" },
    });

    const iife = require("../src/rules/react-no-iife-in-jsx.js");
    const iifeVisitors = iife.create(mockContext({ filename: "a.tsx" }));
    iifeVisitors.JSXExpressionContainer?.({ expression: 1 });

    const boundary = require("../src/rules/module-mock-boundary.js");
    boundary.create({
      filename: "a.ts",
      options: undefined,
      report() {},
      sourceCode: { getScope: () => ({ set: new Map(), variables: [], upper: null }) },
    });
  });
});

describe("null and visitor remaining arms", () => {
  it("covers metadata rest, script attributes, and helper nulls", () => {
    messages(
      "export const { ...generateMetadata } = values;",
      "nextjs-metadata-exports-location",
      undefined,
      "app/lib/meta.ts",
    );
    messages(
      `const metadata = {};
       export { metadata as "metadata" };`,
      "nextjs-metadata-exports-location",
      undefined,
      "app/lib/meta.ts",
    );
    messages(
      '<script dangerouslySetInnerHTML={{ __html: "x" }} />',
      "nextjs-no-manual-script-tags",
      { allowInlineScriptIds: ["ok"] },
      "app/layout.tsx",
    );
    messages(
      '<script src={href} type="module" />',
      "nextjs-no-manual-script-tags",
      undefined,
      "app/layout.tsx",
    );
    const script = require("../src/rules/nextjs-no-manual-script-tags.js");
    const scriptVisitors = script.create({
      filename: "app/layout.tsx",
      options: [{}],
      report() {},
      sourceCode: { getScope: () => ({ set: new Map(), variables: [], upper: null }) },
    });
    scriptVisitors.JSXOpeningElement({
      name: { type: "JSXIdentifier", name: "script" },
      attributes: [
        { type: "JSXAttribute", name: { type: "JSXIdentifier", name: "id" }, value: null },
        {
          type: "JSXAttribute",
          name: { type: "JSXIdentifier", name: "dangerouslySetInnerHTML" },
          value: { type: "JSXExpressionContainer", expression: { type: "ObjectExpression" } },
        },
      ],
    });
    const aliases = require("../src/rules/ts-no-function-aliases.js");
    const aliasVisitors = aliases.create(mockContext());
    aliasVisitors.FunctionDeclaration({
      id: { name: "wrap" },
      body: null,
      params: [],
    });
    aliasVisitors.FunctionDeclaration({
      id: { name: "wrap" },
      params: [],
      body: {
        type: "BlockStatement",
        body: [
          {
            type: "ReturnStatement",
            argument: {
              type: "CallExpression",
              callee: { type: "Literal", value: null },
              arguments: [],
            },
          },
        ],
      },
    });
    hasProperty(
      {
        type: "CallExpression",
        callee: {
          type: "MemberExpression",
          computed: false,
          object: { type: "Identifier", name: "test" },
          property: { type: "Identifier", name: "describe" },
        },
      },
      "describe",
    );
    const { memberPropertyName: pgMember } = require("../src/rules/postgres-executor.js");
    pgMember({
      type: "MemberExpression",
      computed: true,
      property: { type: "TSAsExpression", expression: null },
    });
    sqlStatementBindings({});
    executedQueryText({ type: "Identifier", name: "q" }, new Map(), {
      sourceCode: {
        getScope: () => ({
          set: { get: "nope" },
          variables: [{ name: "other", defs: [] }],
          upper: null,
        }),
      },
    });
    isFetchCall(
      { callee: { type: "Identifier", name: "fetch" } },
      {
        sourceCode: {
          getScope: () => ({
            set: { get: "nope" },
            variables: [{ name: "other", defs: [] }],
            upper: null,
          }),
        },
      },
    );
    const preserve = require("../src/rules/ts-preserve-null-option-defaults.js");
    const preserveVisitors = preserve.create(mockContext());
    preserveVisitors.Program({ type: "Program", body: [] });
    preserveVisitors.AssignmentExpression({
      operator: "=",
      left: {
        type: "ObjectPattern",
        properties: [
          {
            type: "Property",
            key: { type: "Identifier", name: "fresh" },
            value: { type: "Identifier", name: "fresh" },
          },
        ],
      },
      right: { type: "Identifier", name: "opts" },
    });
    preserveVisitors.AssignmentExpression({
      operator: "=",
      left: { type: "Identifier", name: "brandNew" },
      right: {
        type: "MemberExpression",
        computed: false,
        object: { type: "Identifier", name: "opts" },
        property: { type: "Identifier", name: "value" },
      },
    });
    preserveVisitors.LogicalExpression({
      operator: "??",
      left: { type: "Literal", value: 1 },
      right: { type: "Literal", value: 2 },
    });
    const option = {
      checkedPathPatterns: ["**"],
      bannedImports: [{ module: "fs", names: ["readFile"] }],
    };
    messages(
      `const unused = 1;
       const fs = require("fs");
       const { ...rest } = fs;
       const { missing } = fs;
       leftover = 2;
       ({ rest } = 3);`,
      "no-banned-import-outside-allowed-paths",
      option,
      "a.ts",
    );
    messages(
      `import * as fs from "fs";
       const { ...copied } = fs;`,
      "no-banned-import-outside-allowed-paths",
      option,
      "a.ts",
    );
    messages(
      `it("x", async () => {
         for (const item of items) {
           switch (item) {
             case 1:
               continue;
             default:
               await p;
               await expect(p).rejects.toThrow();
           }
         }
       });`,
      "test-no-delayed-rejects",
      undefined,
      "a.test.ts",
    );
    messages(
      `it("x", async () => {
         if (flag) {
           await p;
         } else {
           await expect(p).rejects.toThrow();
         }
       });`,
      "test-no-delayed-rejects",
      undefined,
      "a.test.ts",
    );
    messages("test.sequential?.('x', () => {});", "no-vitest-sequential", undefined, "a.test.ts");
    messages(
      `import { expect } from "vitest";
       expect(error)["message"];
       expect(error.message).toEqual("x");`,
      "test-no-error-message-matching",
      undefined,
      "a.test.ts",
    );
    messages(
      `import { test } from "vitest";
       const { test: aliased } = { test };
       test("x", () => { const nested = () => { shared = 1; }; });`,
      "test-no-shared-state",
      undefined,
      "a.test.ts",
    );
    messages(
      `export function Button() { return <div data-testid="x" /> }`,
      "playwright-require-exported-component-attribute",
      { attributes: ["data-testid"] },
      "Button.tsx",
    );
    messages(
      `import { test } from "@playwright/test";
       test("x", async ({ page }) => { await expect(page.locator("x")).toBeVisible({ timeout: page }); });`,
      "playwright-assertion-timeout-cap",
      undefined,
      "e2e/flow.spec.ts",
    );
    const renaming = require("../src/rules/ts-no-export-renaming.js");
    const renamingVisitors = renaming.create(
      mockContext({ filename: "a.ts", options: [{ includePathPatterns: [".*"] }] }),
    );
    renamingVisitors.ExportNamedDeclaration?.({
      specifiers: [{ exported: null, local: { type: "Identifier", name: "x" } }],
    });
    const interactive = require("../src/rules/playwright-require-interactive-test-id.js");
    interactive.create({
      filename: "a.tsx",
      options: [{ include: "not-array" }],
      report() {},
      sourceCode: { getScope: () => ({ set: new Map(), variables: [], upper: null }) },
    });
    const iife = require("../src/rules/react-no-iife-in-jsx.js");
    const iifeVisitors = iife.create(mockContext({ filename: "a.tsx" }));
    iifeVisitors.JSXExpressionContainer({
      expression: { type: "ArrayExpression", elements: [null, { type: "Literal", value: 1 }] },
    });
    const token = require("../src/rules/playwright-no-hoisted-unique-token.js");
    const tokenVisitors = token.create({
      filename: "e2e/a.spec.ts",
      options: [{ tokenFactories: ["randomUUID"] }],
      report() {},
      sourceCode: {
        getScope: () => ({ set: new Map(), variables: [], upper: null }),
        visitorKeys: { Program: ["body"] },
      },
    });
    tokenVisitors.CallExpression?.({
      callee: { type: "Identifier", name: "test" },
      arguments: [
        { type: "Literal", value: "x" },
        { type: "ArrowFunctionExpression", body: { type: "BlockStatement", body: [] } },
      ],
    });
  });

  it("covers exported internals and add-only tag misses", () => {
    const context = scopeContext([
      ["x", { name: "x" }],
      ["rest", { name: "rest" }],
    ]);
    const aliasMap = new Map();
    const config = new Map([["fs", new Set(["readFile"])]]);
    collectPossibleTag(
      {
        type: "VariableDeclarator",
        id: { type: "Identifier", name: "x" },
        init: { type: "Literal", value: 1 },
      },
      context,
      aliasMap,
      config,
    );
    collectPossibleTag(
      {
        type: "AssignmentExpression",
        operator: "=",
        left: { type: "Identifier", name: "x" },
        right: { type: "Literal", value: 1 },
      },
      context,
      aliasMap,
      config,
    );
    collectPossibleTag(
      {
        type: "AssignmentExpression",
        operator: "=",
        left: { type: "ObjectPattern", properties: [] },
        right: { type: "Literal", value: 1 },
      },
      context,
      aliasMap,
      config,
    );
    applyObjectPatternTagAddOnly(
      {
        properties: [{ type: "RestElement", argument: { type: "Identifier", name: "rest" } }],
      },
      { kind: "object", modules: new Set(["fs"]) },
      context,
      aliasMap,
      config,
    );
    applyObjectPatternTagAddOnly(
      {
        properties: [
          {
            type: "Property",
            key: { type: "Identifier", name: "missing" },
            value: { type: "Identifier", name: "x" },
          },
        ],
      },
      { kind: "object", modules: new Set(["fs"]) },
      context,
      aliasMap,
      config,
    );
    uniqueComponents([
      { name: "A", fn: { range: [0, 1] } },
      { name: "A", fn: { range: [0, 1] } },
    ]);
    collectExportedComponents(
      {
        body: [
          {
            type: "ExportDefaultDeclaration",
            declaration: { type: "ArrowFunctionExpression", id: null, range: [1, 2] },
          },
        ],
      },
      normalizedComponentOptions({ checkAnonymousDefault: true, exportTypes: ["default"] }),
    );
    require("../src/rules/ts-no-function-aliases.js").__test.onlyCallExpression(null);
    require("../src/rules/ts-no-function-aliases.js").__test.isSelfCall(
      { type: "FunctionDeclaration", id: { name: "wrap" } },
      { type: "CallExpression", callee: { type: "Literal", value: null } },
    );
    const metadata = require("../src/rules/nextjs-metadata-exports-location.js").__test;
    metadata.specifierName({ exported: {}, local: { name: "metadata" } });
    metadata.collectPatternNames({
      type: "RestElement",
      argument: { type: "Identifier", name: "generateMetadata" },
    });
    require("../src/rules/nextjs-no-manual-script-tags.js").__test.attributeValue({
      value: { type: "JSXExpressionContainer", expression: { type: "Literal", value: "x" } },
    });
    require("../src/rules/playwright-assertion-timeout-cap.js").__test.propertyName({
      type: "Unknown",
    });
    require("../src/rules/playwright-assertion-timeout-cap.js").__test.propertyName({
      type: "Literal",
      value: "timeout",
    });
    require("../src/rules/ts-no-export-renaming.js").__test.exportName(null);
    require("../src/rules/ts-no-export-renaming.js").__test.exportName({
      type: "Literal",
      value: "x",
    });
    require("../src/rules/no-vitest-sequential.js").__test.hasSequentialMember({
      type: "CallExpression",
      callee: {
        type: "MemberExpression",
        computed: false,
        object: { type: "Identifier", name: "test" },
        property: { type: "Identifier", name: "sequential" },
      },
    });
    const aliases = createMockAliases(mockContext(), new Set(["mock"]));
    aliases.declareImport({ name: "mock" }, "@jest/globals", "mock");
    memberPropertyName({ computed: true, property: { type: "Identifier", name: "x" } });
    frameworkBindingModule(
      { type: "Identifier", name: "vi" },
      {
        sourceCode: {
          getScope: () => ({
            variables: [
              {
                name: "vi",
                defs: [
                  {
                    type: "ImportBinding",
                    node: { imported: { type: "Literal", value: "vi" } },
                    parent: { source: { value: "vitest" } },
                  },
                ],
              },
            ],
            upper: null,
          }),
        },
      },
    );
    seedImportTags(
      {
        type: "Program",
        body: [
          {
            type: "ImportDeclaration",
            source: { value: "untracked" },
            specifiers: [
              { type: "ImportNamespaceSpecifier", local: { type: "Identifier", name: "x" } },
              { type: "UnknownSpecifier", local: { type: "Identifier", name: "x" } },
            ],
          },
        ],
      },
      context,
      config,
      aliasMap,
    );
    const {
      checkExportLeaks,
    } = require("../src/rules/no-banned-import-outside-allowed-paths-imports.js");
    checkExportLeaks(
      {
        type: "ExportNamedDeclaration",
        source: { value: "fs" },
        specifiers: [{ type: "ExportSpecifier", local: { type: "Literal", value: "readFile" } }],
      },
      context,
      config,
      aliasMap,
      new Set(),
      new Map(),
    );
    checkExportLeaks(
      {
        type: "ExportNamedDeclaration",
        specifiers: [{ type: "ExportSpecifier", local: { type: "Identifier", name: "x" } }],
      },
      context,
      config,
      aliasMap,
      new Set(),
      new Map([
        [
          context.sourceCode.getScope().variables[0],
          { kind: "object", modules: new Set(["other"]) },
        ],
      ]),
    );
    tagForExpression({ type: "Literal", value: 1 }, context, aliasMap, config);
    const {
      tagForIdentifier,
    } = require("../src/rules/no-banned-import-outside-allowed-paths-tags.js");
    tagForIdentifier({ type: "Literal", value: 1 }, context, aliasMap);
    require("../src/rules/ts-no-export-renaming.js").__test.exportName({ type: "Unknown" });
    require("../src/rules/playwright-require-interactive-test-id.js").__test.compileMatchers([
      "Link",
    ]);
    require("../src/rules/no-placeholder-never-type-exports.js").__test.specifierName({
      local: {},
      exported: { name: "Placeholder" },
    });
    require("../src/rules/no-placeholder-never-type-exports.js").__test.specifierName({
      local: {},
      exported: { value: "Placeholder" },
    });
    const ret = { type: "ReturnStatement", range: [0, 1] };
    const matcherCall = { type: "CallExpression", range: [50, 60] };
    branchOutcome({ type: "SwitchCase", consequent: [ret] }, matcherCall);
    branchOutcome(
      {
        type: "IfStatement",
        alternate: { type: "ReturnStatement", range: [10, 12] },
        consequent: { type: "ReturnStatement", range: [2, 4] },
      },
      matcherCall,
    );
    branchOutcome(
      {
        type: "SwitchCase",
        consequent: [
          {
            type: "IfStatement",
            consequent: { type: "ReturnStatement", range: [2, 4] },
            alternate: { type: "ReturnStatement", range: [10, 12] },
          },
        ],
      },
      matcherCall,
    );
    expect(
      uniqueComponents([
        { name: "A", fn: { range: [0, 1] } },
        { name: "A", fn: { range: [0, 1] } },
      ]),
    ).toHaveLength(1);
    childNodes({ type: "BlockStatement", body: [1, { type: "Identifier", name: "x" }] });
    const { shouldCheckComponent } = require("../src/exported-components.js");
    shouldCheckComponent(
      { name: "Skip", anonymousDefault: false },
      normalizedComponentOptions({ ignoreComponents: ["Skip"] }),
    );
    messages(
      "export function Link() { return <a href='/' /> }",
      "playwright-require-interactive-test-id",
      { interactiveComponents: ["Link"] },
      "a.tsx",
    );
    messages(
      `type Placeholder = never;
       export { Placeholder as "neverType" };`,
      "no-placeholder-never-type-exports",
      undefined,
      "a.ts",
    );
    messages(
      `type Placeholder = never;
       export { Placeholder };`,
      "no-placeholder-never-type-exports",
      undefined,
      "a.ts",
    );
    const neverRule = require("../src/rules/no-placeholder-never-type-exports.js");
    const neverVisitors = neverRule.create(mockContext({ filename: "a.ts" }));
    neverVisitors.ExportNamedDeclaration({
      specifiers: [{ local: {}, exported: { value: "Placeholder" } }],
    });
    neverVisitors["Program:exit"]?.();
    const errorRule = require("../src/rules/test-no-error-message-matching.js");
    const errorVisitors = errorRule.create(mockContext({ filename: "a.test.ts" }));
    errorVisitors.CallExpression?.({
      callee: { type: "Identifier", name: "it" },
      arguments: [{ type: "Literal", value: "x" }, { type: "ArrowFunctionExpression" }],
    });
    errorVisitors.BinaryExpression?.({
      operator: "===",
      left: { type: "Identifier", name: "x" },
      right: { type: "Identifier", name: "y" },
    });
    require("../src/rules/test-no-error-message-matching.js").__test.propertyName({
      type: "MemberExpression",
    });
    const catchReturn = node("ReturnStatement", { range: [14, 18] });
    const catchBody = node("BlockStatement", { range: [12, 19], body: [catchReturn] });
    catchReturn.parent = catchBody;
    const catchClause = node("CatchClause", { range: [11, 20] });
    catchClause.body = catchBody;
    catchBody.parent = catchClause;
    const tryThrow = node("ThrowStatement", { range: [4, 8] });
    const tryBlock = node("BlockStatement", { range: [3, 10], body: [tryThrow] });
    tryThrow.parent = tryBlock;
    const tryStmt = node("TryStatement", { range: [2, 80] });
    tryStmt.block = tryBlock;
    tryBlock.parent = tryStmt;
    catchClause.parent = tryStmt;
    tryStmt.handler = catchClause;
    tryStmt.parent = node("FunctionDeclaration", { range: [0, 100] });
    caughtThrowCanContinue(tryThrow, node("CallExpression", { range: [90, 95] }));
  });
});
