"use strict";

const { rule } = require("../helpers");
const { childNodes, isFunctionNode } = require("./test-no-shared-state-helpers");
const { hasProperty, resolveVariable } = require("./test-no-shared-state-aliases");
const {
  calleeName,
  importSpecifierName,
  setupCallbackKind,
} = require("./test-no-shared-state-callees");

const PLAYWRIGHT_PATH_PATTERN =
  /(?:^|[/\\])(?:e2e|playwright)(?:[/\\]|$)|(?:^|[/\\])e2e\.(?:spec|test)\.[cm]?[jt]sx?$|\.pw\.(?:spec|test)\.[cm]?[jt]sx?$/;

const TEST_BODY_NAMES = new Set(["test", "it"]);

// The TypeScript node types that wrap a runtime value expression rather than being purely
// type-positioned; every other "TS"-prefixed node type is skipped entirely by `collectEvents`.
const TS_VALUE_WRAPPER_TYPES = new Set([
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSNonNullExpression",
  "TSInstantiationExpression",
  "TSTypeAssertion",
]);

function isPlaywrightPath(filename) {
  return PLAYWRIGHT_PATH_PATTERN.test(filename.replace(/\\/g, "/"));
}

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
function isSharedScopeShield(fn, testCalleeNames) {
  const parent = fn.parent;
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
function isWithinBeforeAllCallback(node, testCalleeNames) {
  for (let fn = nearestOwnFunction(node); fn; fn = nearestOwnFunction(fn)) {
    const parent = fn.parent;
    if (
      parent?.type === "CallExpression" &&
      setupCallbackKind(parent, testCalleeNames) === "before-once"
    ) {
      return true;
    }
  }
  return false;
}

// An Identifier assigned to (not read) by a plain `=` assignment is a write, not a hoisted-value
// reference: a hook that reassigns the tracked variable before reading it is no longer reusing the
// hoisted value.
function isWriteTarget(node) {
  const parent = node.parent;
  return parent?.type === "AssignmentExpression" && parent.operator === "=" && parent.left === node;
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

const KEYED_MEMBER_TYPES = new Set(["MethodDefinition", "PropertyDefinition"]);

// Skip Identifier positions that are names, not value references: the non-computed `.property`
// of a member access, a non-computed, non-shorthand object-literal `.key`, a non-computed class
// method/field `.key` (a class declared inside the hook can legally name a member after the
// tracked variable without reading it), and a plain-assignment write target.
function isReferenceIdentifier(node) {
  const parent = node.parent;
  if (!parent) return true;
  if (parent.type === "MemberExpression" && !parent.computed && parent.property === node) {
    return false;
  }
  if (parent.type === "Property" && !parent.computed && !parent.shorthand && parent.key === node) {
    return false;
  }
  if (KEYED_MEMBER_TYPES.has(parent.type) && !parent.computed && parent.key === node) {
    return false;
  }
  return !isWriteTarget(node);
}

// Collects every read and write of an Identifier reachable from `node`, in source order.
// TypeScript type-only positions (`type X = typeof suffix`, `: typeof suffix` annotations, a
// `TSInterfaceDeclaration` body, ...) are skipped entirely — the value-wrapper expressions above
// recurse into their runtime `.expression` only, never a type operand.
function collectEvents(node, results) {
  if (!node || typeof node.type !== "string") return;
  if (node.type.startsWith("TS")) {
    if (TS_VALUE_WRAPPER_TYPES.has(node.type)) collectEvents(node.expression, results);
    return;
  }
  if (node.type === "Identifier") {
    if (isWriteTarget(node)) results.push({ node, isWrite: true });
    else if (isReferenceIdentifier(node)) results.push({ node, isWrite: false });
  }
  for (const child of childNodes(node)) collectEvents(child, results);
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
        properties: { tokenFactories: { type: "array", items: { type: "string" } } },
      },
    ],
    messages: {
      hoisted:
        "`{{name}}` is generated once by `{{factory}}()` at module/describe scope, but `beforeAll` can re-run in the same worker process with module state preserved — the hoisted value is reused unchanged on re-entry and collides with itself. Move the `{{factory}}()` call to the first statement inside this `beforeAll` instead.",
    },
  },
  (context) => {
    let isPlaywrightFile = isPlaywrightPath(context.filename);
    const tokenFactories = context.options?.[0]?.tokenFactories;
    if (!tokenFactories || tokenFactories.length === 0) return {};
    const factoryNames = new Set(tokenFactories);

    // declarator node -> factory name, for declarations not already shielded inside their own hook/test.
    const candidates = new Map();
    const beforeAllCallbacks = [];
    // Local aliases of the imported `test`/`it`/`describe` bindings (e.g. `import { test as pw }`),
    // so `pw.beforeAll(...)` is recognized the same as `test.beforeAll(...)`.
    const testCalleeNames = new Set(["test", "it", "describe"]);

    return {
      ImportDeclaration(node) {
        if (node.source.value !== "@playwright/test") return;
        isPlaywrightFile = true;
        for (const specifier of node.specifiers) {
          if (specifier.type !== "ImportSpecifier") continue;
          const imported = importSpecifierName(specifier);
          if (["describe", "it", "test"].includes(imported) && specifier.local?.name) {
            testCalleeNames.add(specifier.local.name);
          }
        }
      },
      VariableDeclarator(node) {
        if (
          node.id.type !== "Identifier" ||
          node.init?.type !== "CallExpression" ||
          node.init.callee.type !== "Identifier" ||
          !factoryNames.has(node.init.callee.name)
        ) {
          return;
        }
        const fn = nearestOwnFunction(node);
        if (
          (!fn || !isSharedScopeShield(fn, testCalleeNames)) &&
          !isWithinBeforeAllCallback(node, testCalleeNames)
        ) {
          candidates.set(node, node.init.callee.name);
        }
      },
      CallExpression(node) {
        if (setupCallbackKind(node, testCalleeNames) !== "before-once") return;
        const callback = node.arguments.find((argument) => isFunctionNode(argument));
        if (callback) beforeAllCallbacks.push(callback);
      },
      "Program:exit"() {
        if (!isPlaywrightFile || candidates.size === 0) return;
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
              if (isUnconditionalWrite(event.node, callback)) refreshed.add(declarator);
              continue;
            }
            if (refreshed.has(declarator)) continue;
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
