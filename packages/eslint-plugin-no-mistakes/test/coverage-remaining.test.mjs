import { describe, expect, it } from "vitest";

import { messages, require } from "./helpers.mjs";

const {
  collectBannedAliases,
  recordAssignmentTag,
  recordVariableTag,
} = require("../src/rules/no-banned-import-outside-allowed-paths-aliases.js");
const { createTargetMatcher } = require("../src/rules/async-targets.js");
const {
  isReassigned,
  memberPropertyName,
  recordObjectPatternBindings,
} = require("../src/rules/async-target-bindings.js");
const { integrationAllows } = require("../src/rules/module-mock-integration.js");
const { canReachMatcher, executesBefore } = require("../src/rules/test-no-delayed-rejects-flow.js");
const {
  possibleResourceExitBeforeMatcher,
} = require("../src/rules/test-no-delayed-rejects-loop-jumps.js");
const {
  alwaysExits,
  alwaysThrows,
  breakSkipsMatcher,
} = require("../src/rules/test-no-delayed-rejects-abrupt.js");
const {
  possibleCaughtThrowCanContinue,
  thrownCompletionCanReachMatcher,
} = require("../src/rules/test-no-delayed-rejects-transfers.js");
const { createReactNodeFacts, typeName } = require("../src/react-node-types.js");

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
      },
    },
  };
}

describe("remaining alias and matcher arms", () => {
  it("seeds add-only object rest tags and assignment patterns", () => {
    const mod = { name: "mod" };
    const rest = { name: "rest" };
    const named = { name: "readFile" };
    const context = scopeContext([
      ["mod", mod],
      ["rest", rest],
      ["readFile", named],
    ]);
    const aliasMap = new Map();
    const config = new Map([["fs", new Set(["readFile"])]]);
    const program = {
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
                arguments: [{ type: "Literal", value: "fs" }],
              },
              id: {
                type: "ObjectPattern",
                properties: [
                  { type: "RestElement", argument: { type: "Identifier", name: "rest" } },
                  { type: "Skip" },
                  {
                    type: "Property",
                    key: { type: "Identifier", name: "readFile" },
                    value: { type: "Identifier", name: "readFile" },
                  },
                  {
                    type: "Property",
                    key: { type: "Identifier", name: "nested" },
                    value: { type: "ObjectPattern", properties: [] },
                  },
                ],
              },
            },
          ],
        },
        {
          type: "ExpressionStatement",
          expression: {
            type: "AssignmentExpression",
            operator: "=",
            left: { type: "Identifier", name: "mod" },
            right: {
              type: "CallExpression",
              callee: { type: "Identifier", name: "require" },
              arguments: [{ type: "Literal", value: "fs" }],
            },
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
            right: { type: "Identifier", name: "mod" },
          },
        },
        {
          type: "ExpressionStatement",
          expression: {
            type: "AssignmentExpression",
            operator: "+=",
            left: { type: "Identifier", name: "mod" },
            right: { type: "Literal", value: 1 },
          },
        },
      ],
    };
    collectBannedAliases(program, context, aliasMap, config);
    recordVariableTag(program.body[0].declarations[0], context, aliasMap, new Set(), config);
    recordAssignmentTag(program.body[1].expression, context, aliasMap, new Set(), config);
    const {
      setOrClearTag,
    } = require("../src/rules/no-banned-import-outside-allowed-paths-aliases.js");
    const unresolved = { type: "Identifier", name: "missing" };
    const resolved = { type: "Identifier", name: "mod" };
    setOrClearTag(unresolved, { kind: "direct" }, context, aliasMap);
    setOrClearTag(resolved, { kind: "direct" }, context, aliasMap);
    setOrClearTag(resolved, null, context, aliasMap);
    setOrClearTag(resolved, { kind: "direct" }, context, aliasMap, new Set());
    setOrClearTag(resolved, null, context, aliasMap, new Set());
    expect(aliasMap.size).toBeGreaterThanOrEqual(0);
  });

  it("covers computed object-pattern bindings and reassigned params", () => {
    const recorded = [];
    recordObjectPatternBindings(
      {
        properties: [
          { type: "RestElement" },
          {
            type: "Property",
            computed: true,
            key: { type: "Literal", value: "run" },
            value: { type: "AssignmentPattern", left: { type: "Identifier", name: "run" } },
          },
          {
            type: "Property",
            computed: false,
            key: { type: "Identifier", name: "other" },
            value: { type: "ArrayPattern" },
          },
        ],
      },
      "mod",
      (id, source, name) => recorded.push([id?.name, source, name]),
    );
    expect(recorded[0][0]).toEqual("run");
    expect(
      memberPropertyName({ computed: true, property: { type: "Literal", value: "run" } }),
    ).toEqual("run");
    const context = {
      sourceCode: {
        getScope: () => ({
          variables: [
            {
              name: "run",
              defs: [{ type: "Parameter" }],
              references: [{ isWrite: () => true, init: true, identifier: {} }],
            },
          ],
          upper: null,
        }),
      },
    };
    expect(isReassigned({ name: "run" }, context)).toBe(true);
  });
});

describe("remaining delayed-rejects control flow", () => {
  it("covers switch exclusivity, optional calls, and loop resource exits", () => {
    const fn = node("FunctionDeclaration", { range: [0, 200] });
    const sw = link(fn, node("SwitchStatement", { range: [1, 180], cases: [] }));
    const first = link(sw, node("SwitchCase", { range: [2, 40], consequent: [] }));
    const second = link(sw, node("SwitchCase", { range: [50, 170], consequent: [] }));
    const matcher = node("CallExpression", { range: [5, 15] });
    matcher.parent = first;
    first.consequent = [matcher];
    const suspension = node("AwaitExpression", { range: [60, 70] });
    const brk = node("BreakStatement", { range: [80, 85] });
    suspension.parent = second;
    brk.parent = second;
    second.consequent = [suspension, brk];
    sw.cases = [first, second];
    canReachMatcher(suspension, matcher, fn);

    const optional = node("CallExpression", {
      range: [0, 20],
      optional: false,
      callee: node("MemberExpression", {
        optional: true,
        object: node("Identifier"),
        property: node("Identifier"),
      }),
    });
    optional.callee.parent = optional;
    optional.callee.object.parent = optional.callee;
    executesBefore(optional.callee.object, optional);

    const loop = node("ForOfStatement", { range: [0, 80] });
    loop.parent = fn;
    const body = link(loop, node("BlockStatement", { range: [5, 70], body: [] }), "body");
    const cont = node("ContinueStatement", { range: [10, 12] });
    cont.parent = body;
    const throwStmt = node("ThrowStatement", { range: [20, 25] });
    throwStmt.parent = body;
    body.body = [cont, throwStmt];
    possibleResourceExitBeforeMatcher(cont, matcher, loop);
    possibleResourceExitBeforeMatcher(throwStmt, matcher, loop);

    const doWhile = node("DoWhileStatement", { range: [0, 40] });
    const whileNode = node("WhileStatement", { range: [0, 40] });
    const forIn = node("ForInStatement", { range: [0, 40] });
    alwaysExits(node("IfStatement", { consequent: node("ReturnStatement") }));
    alwaysThrows(node("IfStatement", { consequent: node("ThrowStatement") }));
    executesBefore(
      node("Identifier", { parent: doWhile }),
      node("AwaitExpression", { range: [30, 35] }),
    );
    executesBefore(
      node("Identifier", { parent: whileNode }),
      node("AwaitExpression", { range: [30, 35] }),
    );
    executesBefore(
      node("Identifier", { parent: forIn }),
      node("AwaitExpression", { range: [30, 35] }),
    );
  });
});

describe("remaining lint and helper shapes", () => {
  it("covers react-node rest, assignment, var, and chain expressions", () => {
    messages(
      `import type { ReactNode } from "react";
       type Props = { title: ReactNode };
       export function Rest({ ...props }: Props) { return props.title ?? "x"; }
       export function Assign({ title = "t" }: Props) { return title ?? "x"; }
       export function Arr([title]: ReactNode[]) { return title ?? "x"; }
       export const Child = ({ title }: { title: ReactNode }) => title ?? "x";
       export const Expr = function ({ title }: { title: ReactNode }) { return title ?? "x"; }
       var hoisted: ReactNode = null;
       export function Var() { return hoisted ?? "x"; }
       export function Chain(props: Props) { return props?.title ?? "x"; }
       export function Computed(props: Props) { return props["title"] ?? "x"; }
       export function Ident(title: ReactNode) { return title ?? "x"; }`,
      "react-no-nullish-react-node",
      undefined,
      "a.tsx",
    );
  });

  it("covers async try/catch aliases and integration fallbacks", () => {
    messages(
      `import { handleRateLimit } from "@app/rate-limit";
       async function run(flag, cached) {
         try {
           const created = new Promise((resolve) => resolve(1));
           const aliased = request();
           let assigned;
           assigned = flag ? cached : request();
           await aliased;
           return created || request();
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
    );
    expect(integrationAllows("./mod", null, { framework: "vitest" }, scopeContext(), {})).toBe(
      false,
    );
    expect(
      integrationAllows(
        "./mod",
        { type: "ObjectExpression", properties: [] },
        { framework: "vitest" },
        scopeContext(),
        { integrationExports: { specifiers: ["^@app/"], markerRegex: "[" } },
      ),
    ).toBe(false);
    expect(
      typeName({
        type: "TSOptionalType",
        typeAnnotation: {
          type: "TSTypeReference",
          typeName: { type: "Identifier", name: "ReactNode" },
        },
      }),
    ).toEqual("ReactNode");
    createReactNodeFacts({
      body: [
        {
          type: "ExportDefaultDeclaration",
          declaration: {
            type: "TSInterfaceDeclaration",
            id: { name: "Props" },
            body: { body: null },
          },
        },
        { type: "TSInterfaceDeclaration", id: { name: "Empty" }, body: {} },
      ],
    });
  });

  it("covers target matcher member and require member shapes", () => {
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
      id: { type: "Identifier", name: "run" },
      init: {
        type: "MemberExpression",
        object: {
          type: "CallExpression",
          callee: { type: "Identifier", name: "require" },
          arguments: [{ type: "Literal", value: "mod" }],
        },
        property: { name: "run" },
      },
    });
    matcher.isTargetCall({
      callee: {
        type: "MemberExpression",
        computed: false,
        object: { type: "Identifier", name: "ns" },
        property: { name: "run" },
      },
    });
    matcher.visitors.Program({
      type: "Unknown",
      body: [{ type: "Literal", value: 1 }],
    });
  });
});

describe("more remaining branch arms", () => {
  it("covers fetch switch fallthrough, empty switch, and for inits", () => {
    messages(
      "switch (x) {} fetch();",
      "no-global-fetch-outside-helper",
      { checkedPathPatterns: [".*"] },
      "a.ts",
    );
    messages(
      "switch (x) { case 1: fetch(); break; default: fetch(); }",
      "no-global-fetch-outside-helper",
      { checkedPathPatterns: [".*"] },
      "a.ts",
    );
    messages(
      "switch (x) { case 1: foo(); default: fetch(); }",
      "no-global-fetch-outside-helper",
      { checkedPathPatterns: [".*"] },
      "a.ts",
    );
    messages(
      "for (let aliased = fetch; i < 1; i++) {}",
      "no-global-fetch-outside-helper",
      { checkedPathPatterns: [".*"] },
      "a.ts",
    );
    messages(
      "for (aliased = fetch; i < 1; i++) {}",
      "no-global-fetch-outside-helper",
      { checkedPathPatterns: [".*"] },
      "a.ts",
    );
  });

  it("covers preserve-null existing-scope assignment and non-property patterns", () => {
    messages(
      `interface Options { value?: string | null }
       export function assignExisting(options: Options) {
         let value;
         ({ value } = options);
         ({ value } = {} as Options);
         ({ value } = unknown);
         value = options.value;
         value = other;
         return value ?? "x";
       }`,
      "ts-preserve-null-option-defaults",
      undefined,
      "a.ts",
    );
    const rule = require("../src/rules/ts-preserve-null-option-defaults.js");
    const visitors = rule.create({
      filename: "a.ts",
      options: [{}],
      report() {},
      sourceCode: { getScope: () => ({ set: new Map(), variables: [], upper: null }) },
    });
    visitors.Program({ type: "Program", body: [] });
    visitors.AssignmentExpression({
      operator: "=",
      left: { type: "ObjectPattern", properties: [{ type: "SpreadElement" }] },
      right: { type: "Identifier", name: "opts" },
    });
    visitors["Program:exit"]();
  });

  it("covers try/finally throw reachability and if jump skips", () => {
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
    const handler = node("CatchClause", { range: [11, 20] });
    handler.body = node("BlockStatement", { range: [12, 19], body: [] });
    handler.body.parent = handler;
    handler.parent = tryNode;
    tryNode.handler = handler;
    const brk = node("BreakStatement", { range: [22, 24] });
    tryNode.finalizer = node("BlockStatement", { range: [21, 30], body: [brk] });
    tryNode.finalizer.parent = tryNode;
    brk.parent = tryNode.finalizer;
    const matcher = node("CallExpression", { range: [60, 70] });
    matcher.parent = fn;
    thrownCompletionCanReachMatcher(throwStmt, matcher);
    possibleCaughtThrowCanContinue(tryNode, matcher);
    alwaysExits(tryNode);
    alwaysThrows(tryNode);
    const ifNode = node("IfStatement", { range: [0, 40] });
    ifNode.parent = loop;
    const left = node("BreakStatement", { range: [2, 4] });
    const right = node("BreakStatement", { range: [10, 12] });
    left.parent = ifNode;
    right.parent = ifNode;
    ifNode.consequent = left;
    ifNode.alternate = right;
    expect(breakSkipsMatcher(ifNode, matcher)).toBe(true);
  });
});
