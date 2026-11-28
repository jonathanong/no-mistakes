"use strict";

const { isFunction, unwrapExpression } = require("./async-ast");
const { isTestRunner } = require("./test-message-runner");

const HOOK_NAMES = new Set(["beforeAll", "beforeEach", "afterAll", "afterEach"]);

function isLifecycleHookCall(callNode) {
  const callee = unwrapExpression(callNode.callee);
  return callee?.type === "Identifier" && HOOK_NAMES.has(callee.name);
}

function enclosingTestBlock(node, context) {
  let current = node.parent;
  while (current) {
    if (
      isFunction(current) &&
      current.parent.type === "CallExpression" &&
      isTestRunner(current.parent.callee, context)
    ) {
      return current;
    }
    current = current.parent;
  }
  return null;
}

function enclosingSuiteChain(node) {
  const scopes = [];
  let current = node;
  while (current) {
    if (isFunction(current) || current.type === "Program") scopes.push(current);
    current = current.parent;
  }
  return scopes;
}

function effectiveExemptionScope(node, context) {
  const testBlock = enclosingTestBlock(node, context);
  if (testBlock) return testBlock;
  let current = node.parent;
  while (current.type !== "Program") {
    if (isFunction(current)) {
      if (current.parent.type === "CallExpression" && isLifecycleHookCall(current.parent)) {
        return enclosingSuiteChain(current.parent.parent)[0];
      }
      return current;
    }
    current = current.parent;
  }
  return current;
}

function sourceDeclarationLineNode(node) {
  let current = node;
  while (
    current.parent.type !== "VariableDeclaration" &&
    current.parent.type !== "ExpressionStatement"
  ) {
    current = current.parent;
  }
  return current.parent;
}

function recordScopePath(map, scope, resolved, variable) {
  let entry = map.get(scope);
  if (!entry) {
    entry = { keys: new Set(), vars: new Set() };
    map.set(scope, entry);
  }
  if (resolved?.kind === "repo") entry.keys.add(resolved.key);
  if (variable) entry.vars.add(variable);
}

function isExemptInScope(map, scopeChain, source) {
  for (const scope of scopeChain) {
    const entry = map.get(scope);
    if (!entry) continue;
    if (entry.keys.has(source.pathKey) || (source.pathVar && entry.vars.has(source.pathVar))) {
      return true;
    }
  }
  return false;
}

module.exports = {
  effectiveExemptionScope,
  enclosingSuiteChain,
  enclosingTestBlock,
  isExemptInScope,
  recordScopePath,
  sourceDeclarationLineNode,
};
