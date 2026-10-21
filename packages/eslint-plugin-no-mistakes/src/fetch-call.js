"use strict";

const { logicalCall, memberPropertyName, propertyName } = require("./rules/module-mock-helpers");

const LOCAL_BINDING_TYPES = new Set([
  "Variable",
  "Parameter",
  "CatchClause",
  "FunctionName",
  "ImportBinding",
  "ClassName",
]);
const FETCH_ROOTS = new Set(["globalThis", "window", "self", "global"]);

function isFetchShadowed(scope) {
  while (scope) {
    const get = scope.set?.get;
    const variable = typeof get === "function" ? get.call(scope.set, "fetch") : null;
    if (variable) return variable.defs.some((def) => LOCAL_BINDING_TYPES.has(def.type));
    if (!scope.set || typeof get !== "function") {
      const fallback = scope.variables?.find((item) => item.name === "fetch");
      if (fallback) return fallback.defs.some((def) => LOCAL_BINDING_TYPES.has(def.type));
    }
    scope = scope.upper;
  }
  return false;
}

function unboundRoot(node, context) {
  if (node?.type !== "Identifier" || !FETCH_ROOTS.has(node.name)) return false;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables?.find((item) => item.name === node.name);
    if (variable) return !variable.defs?.some((def) => LOCAL_BINDING_TYPES.has(def.type));
    scope = scope.upper;
  }
  return true;
}

function bindingVariable(node, context) {
  if (!context?.sourceCode?.getScope) return null;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables?.find((item) => item.name === node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function fetchCallee(node, context, seen) {
  if (!node) return false;
  const visiting = seen || new Set();
  if (node.type === "MemberExpression") {
    if (memberPropertyName(node) !== "fetch") return false;
    const object = node.object.type === "ChainExpression" ? node.object.expression : node.object;
    return unboundRoot(object, context);
  }
  if (node.type === "Identifier") {
    if (node.name === "fetch" && !isFetchShadowed(context?.sourceCode?.getScope?.(node)))
      return true;
    const variable = bindingVariable(node, context);
    if (!variable || visiting.has(variable) || variable.defs.length !== 1) return false;
    visiting.add(variable);
    const def = variable.defs[0];
    if (def.type !== "Variable" || def.parent?.kind !== "const") return false;
    if (def.node.id.type === "Identifier") {
      if (!def.node.init) return false;
      return fetchCallee(def.node.init, context, visiting);
    }
    if (def.node.id.type !== "ObjectPattern") return false;
    for (const item of def.node.id.properties) {
      if (item.type === "Property") {
        const value = item.value.type === "AssignmentPattern" ? item.value.left : item.value;
        if (
          value.type === "Identifier" &&
          value.name === node.name &&
          propertyName(item.key) === "fetch"
        ) {
          return unboundRoot(def.node.init, context);
        }
      }
    }
    return false;
  }
  return false;
}

function fetchInvocation(node, context) {
  const invoked = logicalCall(node);
  if (!invoked?.callee || !fetchCallee(invoked.callee, context)) return null;
  return invoked.arguments;
}

function isFetchCall(node, context) {
  return fetchInvocation(node, context) !== null;
}

module.exports = { fetchInvocation, isFetchCall };
