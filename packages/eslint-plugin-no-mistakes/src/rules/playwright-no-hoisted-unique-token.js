"use strict";

const { rule } = require("../helpers");
const { childNodes, isFunctionNode } = require("./test-no-shared-state-helpers");
const { resolveVariable } = require("./test-no-shared-state-aliases");
const { calleeName, propertyName, setupCallbackKind } = require("./test-no-shared-state-callees");

const PLAYWRIGHT_PATH_PATTERN =
  /(?:^|[/\\])(?:e2e|playwright)(?:[/\\]|$)|(?:^|[/\\])e2e\.(?:spec|test)\.[cm]?[jt]sx?$|\.pw\.(?:spec|test)\.[cm]?[jt]sx?$/;

const TEST_BODY_NAMES = new Set(["test", "it"]);

function isPlaywrightPath(filename) {
  return PLAYWRIGHT_PATH_PATTERN.test(filename.replace(/\\/g, "/"));
}

function nearestOwnFunction(node) {
  let current = node.parent;
  while (current) {
    if (isFunctionNode(current)) return current;
    if (current.type === "Program") return null;
    current = current.parent;
  }
  return null;
}

// A declaration is already safely scoped to its own re-entry unit when the function it lives in
// is passed straight into a setup hook (`beforeAll`/`beforeEach`/`afterEach`/`afterAll`, bare or
// `test.`-qualified) or a test body (`test`/`it`, including `.only`/`.skip`-style modifiers) — but
// NOT `describe`, which registers once and is exactly as re-entry-hazardous as module scope.
function isSharedScopeShield(fn) {
  const parent = fn.parent;
  if (!parent || parent.type !== "CallExpression") return false;
  if (setupCallbackKind(parent)) return true;
  const callee = parent.callee;
  if (callee.type === "Identifier") return TEST_BODY_NAMES.has(callee.name);
  if (callee.type === "MemberExpression" && !callee.computed) {
    return TEST_BODY_NAMES.has(calleeName(callee)) && propertyName(callee.property) !== "describe";
  }
  return false;
}

// Skip Identifier positions that are names, not value references: the non-computed `.property`
// of a member access, and a non-computed, non-shorthand object-literal `.key`.
function isReferenceIdentifier(node) {
  const parent = node.parent;
  if (!parent) return true;
  if (parent.type === "MemberExpression" && !parent.computed && parent.property === node) {
    return false;
  }
  if (parent.type === "Property" && !parent.computed && !parent.shorthand && parent.key === node) {
    return false;
  }
  return true;
}

function collectReferenceIdentifiers(node, results) {
  if (!node) return;
  if (node.type === "Identifier" && isReferenceIdentifier(node)) results.push(node);
  for (const child of childNodes(node)) collectReferenceIdentifiers(child, results);
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
    if (!isPlaywrightPath(context.filename)) return {};
    const tokenFactories = context.options?.[0]?.tokenFactories;
    if (!tokenFactories || tokenFactories.length === 0) return {};
    const factoryNames = new Set(tokenFactories);

    // declarator node -> factory name, for declarations not already shielded inside their own hook/test.
    const candidates = new Map();
    const beforeAllCallbacks = [];

    return {
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
        if (!fn || !isSharedScopeShield(fn)) candidates.set(node, node.init.callee.name);
      },
      CallExpression(node) {
        if (setupCallbackKind(node) !== "before-once") return;
        const callback = node.arguments.find((argument) => isFunctionNode(argument));
        if (callback) beforeAllCallbacks.push(callback);
      },
      "Program:exit"() {
        if (candidates.size === 0) return;
        for (const callback of beforeAllCallbacks) {
          const references = [];
          collectReferenceIdentifiers(callback.body, references);
          for (const identifier of references) {
            const variable = resolveVariable(identifier, context);
            const declarator = variable?.defs?.[0]?.node;
            const factory = declarator && candidates.get(declarator);
            if (factory) {
              context.report({
                node: identifier,
                messageId: "hoisted",
                data: { name: identifier.name, factory },
              });
            }
          }
        }
      },
    };
  },
);
