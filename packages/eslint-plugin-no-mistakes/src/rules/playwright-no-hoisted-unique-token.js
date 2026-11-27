"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { isTransparentExpression, unwrapExpression } = require("./async-ast");
const { reportUnanalyzable } = require("./module-binding");
const { createHoistedModuleState } = require("./playwright-hoisted-module");
const { isFunctionNode } = require("./test-no-shared-state-helpers");
const { hasProperty, resolveVariable } = require("./test-no-shared-state-aliases");
const { calleeName, setupCallbackKind } = require("./test-no-shared-state-callees");
const { configuredFactory, invokedCall, isBeforeAllCall } = require("./playwright-static-call");
const {
  collectEvents,
  writeReusesTrackedValue,
} = require("./playwright-no-hoisted-unique-token-events");

const TEST_BODY_NAMES = new Set(["test", "it"]);

function nearestOwnFunction(node) {
  for (let current = node.parent; current && current.type !== "Program"; current = current.parent) {
    if (isFunctionNode(current)) return current;
  }
  return null;
}

// A declaration is already safely scoped to its own re-entry unit when the function it lives in
// is passed straight into a setup hook (`beforeAll`/`beforeEach`/`afterEach`/`afterAll`, bare or
// `test.`-qualified) or a test body (`test`/`it`, including `.only`/`.skip`-style modifiers) — but
// NOT `describe` anywhere in the callee chain (e.g. `test.describe.only`), which registers once
// and is exactly as re-entry-hazardous as module scope. `testCalleeNames` carries every local alias
// of the imported `test`/`it`/`describe` bindings, so an aliased `import { test as pw } from
// "@playwright/test"` is recognized the same as the bare names.
function outerParent(node) {
  let current = node?.parent;
  while (isTransparentExpression(current)) current = current.parent;
  return current;
}

function isSharedScopeShield(fn, testCalleeNames) {
  const parent = outerParent(fn);
  if (!parent || parent.type !== "CallExpression") return false;
  if (setupCallbackKind(parent, testCalleeNames)) return true;
  const callee = parent.callee;
  if (callee.type === "Identifier") return TEST_BODY_NAMES.has(callee.name);
  if (callee.type === "MemberExpression" && !callee.computed) {
    return TEST_BODY_NAMES.has(calleeName(callee)) && !hasProperty(callee, "describe");
  }
  return false;
}

// A declaration nested arbitrarily deep inside a `beforeAll` callback — even inside a local helper
// function the hook itself defines and invokes — is minted fresh on every hook re-entry, so it is
// never a hoisting hazard regardless of how many function boundaries separate it from the hook.
function isWithinBeforeAllCallback(node, beforeAll) {
  for (let fn = nearestOwnFunction(node); fn; fn = nearestOwnFunction(fn)) {
    let parent = outerParent(fn);
    if (parent?.type === "ArrayExpression") parent = outerParent(parent);
    if (parent?.type === "CallExpression" && beforeAll(parent)) return true;
  }
  return false;
}

const BRANCHING_ANCESTOR_TYPES = new Set([
  "IfStatement",
  "ConditionalExpression",
  "SwitchStatement",
  "SwitchCase",
  "TryStatement",
  "CatchClause",
  "LogicalExpression",
  "ForStatement",
  "ForInStatement",
  "ForOfStatement",
  "WhileStatement",
  "DoWhileStatement",
]);

// `collectEvents` walks the callback body in source order, not control-flow order, so a write
// nested inside a branch, loop, or logical short-circuit may never execute on a given re-entry —
// it cannot "refresh" the tracked value just because it was visited earlier in the traversal. Only
// a write with no branching ancestor between it and the callback body is unconditional.
function isUnconditionalWrite(node, callback) {
  for (let current = node.parent; current && current !== callback; current = current.parent) {
    if (BRANCHING_ANCESTOR_TYPES.has(current.type)) return false;
    // A write inside a function nested in the callback (a locally-defined helper) only runs if
    // that function is itself called — never assume the enclosing callback executes it unconditionally.
    if (isFunctionNode(current) && current !== callback) return false;
  }
  return true;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description:
        "disallow reading a module/describe-scope unique-token call inside a re-entrant beforeAll",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          tokenFactories: { type: "array", items: { type: "string" } },
          unanalyzable: { enum: ["finding", "ignore"] },
        },
      },
    ],
    messages: {
      hoisted:
        "`{{name}}` is generated once by `{{factory}}()` at module/describe scope, but `beforeAll` can re-run in the same worker process with module state preserved — the hoisted value is reused unchanged on re-entry and collides with itself. Move the `{{factory}}()` call to the first statement inside this `beforeAll` instead.",
      unanalyzable:
        "Cannot tell whether this beforeAll is Playwright's hook. Import or require @playwright/test with a static name so hoisted tokens can be checked.",
    },
  },
  (context) => {
    const option = context.options?.[0] ?? {};
    const tokenFactories = option.tokenFactories;
    const surfaceOpaque = reportUnanalyzable(option);
    if (!tokenFactories || tokenFactories.length === 0) return {};
    const suppressed = ruleSuppression(context, "playwright-no-hoisted-unique-token");
    const factoryNames = new Set(tokenFactories);
    const moduleState = createHoistedModuleState(context);
    const { namespaces, testCalleeNames } = moduleState;

    // declarator node -> factory name, for declarations not already shielded inside their own hook/test.
    const candidates = new Map();
    const beforeAllCallbacks = [];
    const opaqueCallbacks = [];
    const beforeAll = (node) => {
      const kind = setupCallbackKind(node, testCalleeNames);
      const callee = unwrapExpression(node.callee);
      if (kind === "before-once" && callee?.type === "Identifier") return true;
      return isBeforeAllCall(node, testCalleeNames, namespaces, context);
    };

    return {
      ImportDeclaration(node) {
        moduleState.admitImport(node);
      },
      TSImportEqualsDeclaration(node) {
        moduleState.noteBinding(node.id);
      },
      VariableDeclarator(node) {
        moduleState.noteBinding(node.id);
        const init = unwrapExpression(node.init);
        const factory =
          node.id.type === "Identifier" && init?.type === "CallExpression"
            ? configuredFactory(invokedCall(init)?.callee, context, factoryNames)
            : null;
        if (!factory) return;
        const fn = nearestOwnFunction(node);
        if (
          (!fn || !isSharedScopeShield(fn, testCalleeNames)) &&
          !isWithinBeforeAllCallback(node, beforeAll)
        ) {
          candidates.set(node, factory);
        }
      },
      CallExpression(node) {
        const args = (invokedCall(node)?.arguments ?? node.arguments).map((argument) =>
          unwrapExpression(argument),
        );
        const callback = args.find((argument) => isFunctionNode(argument));
        if (!callback) return;
        if (beforeAll(node)) beforeAllCallbacks.push(callback);
        else if (surfaceOpaque && moduleState.opaqueBeforeAll(node)) {
          opaqueCallbacks.push({ node, callback });
        }
      },
      "Program:exit"() {
        if (!moduleState.isPlaywrightFile() || candidates.size === 0) return;
        for (const { node, callback } of opaqueCallbacks) {
          const events = [];
          collectEvents(callback.body, events);
          const readsCandidate = events.some(
            (event) =>
              !event.isWrite &&
              resolveVariable(event.node, context)?.defs?.some((def) => candidates.has(def.node)),
          );
          if (readsCandidate && !suppressed(node))
            context.report({ node, messageId: "unanalyzable" });
        }
        for (const callback of beforeAllCallbacks) {
          const events = [];
          collectEvents(callback.body, events);
          const refreshed = new Set();
          for (const event of events) {
            // A `var` redeclared in the same scope (`var suffix; var suffix = randomSuffix();`)
            // is one Variable with multiple defs — the factory-call declarator may be any of
            // them, not necessarily defs[0], so every def must be checked against `candidates`.
            const variable = resolveVariable(event.node, context);
            const declarator = variable?.defs?.find((def) => candidates.has(def.node))?.node;
            if (!declarator) continue;
            if (event.isWrite) {
              const refreshes =
                isUnconditionalWrite(event.node, callback) &&
                !writeReusesTrackedValue(event.node, declarator, context);
              if (refreshes) refreshed.add(declarator);
              continue;
            }
            if (refreshed.has(declarator) || suppressed(event.node)) continue;
            context.report({
              node: event.node,
              messageId: "hoisted",
              data: { name: event.node.name, factory: candidates.get(declarator) },
            });
          }
        }
      },
    };
  },
);
