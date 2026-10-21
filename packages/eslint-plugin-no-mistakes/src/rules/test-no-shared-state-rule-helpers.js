"use strict";

const { literalString } = require("./module-mock-helpers");
const {
  isKnownTestCallee,
  isTestCall,
  setupCallbackKind,
  isFunctionNode,
} = require("./test-no-shared-state-helpers");

function resolveFunctionCallback(context, node, callback) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const get = scope.set?.get;
    const resolvedVariable =
      (typeof get === "function" ? get.call(scope.set, callback.name) : null) ||
      (typeof get !== "function"
        ? scope.variables?.find((item) => item.name === callback.name)
        : null);

    if (!resolvedVariable) {
      scope = scope.upper;
      continue;
    }

    const declaration = resolvedVariable.defs[0]?.node;
    if (declaration?.type === "FunctionDeclaration") return declaration;
    if (declaration?.type === "VariableDeclarator" && isFunctionNode(declaration.init)) {
      return declaration.init;
    }
    return null;
  }
  return null;
}

function calleeHasProperty(callee, name) {
  while (callee?.type === "ChainExpression") callee = callee.expression;
  if (!callee) return false;
  if (callee.type === "Identifier") return callee.name === name;
  if (callee.type === "CallExpression") return calleeHasProperty(callee.callee, name);
  if (callee.type !== "MemberExpression") return false;
  if (callee.computed && callee.property?.type === "Identifier") {
    return calleeHasProperty(callee.object, name);
  }
  const prop = callee.computed ? literalString(callee.property) : callee.property?.name;
  return prop === name || calleeHasProperty(callee.object, name);
}

function createRuleHelpers(context, testCalleeNames, namespaces) {
  return {
    calleeHasProperty,
    isDescribeCall(node) {
      return (
        (node.callee.type === "Identifier" && node.callee.name === "describe") ||
        (isKnownTestCallee(node.callee, testCalleeNames, namespaces) &&
          calleeHasProperty(node.callee, "describe"))
      );
    },
    isInlineCallback(node) {
      const parent = node.parent;
      return (
        parent?.type === "CallExpression" &&
        (setupCallbackKind(parent, testCalleeNames, namespaces) ||
          isTestCall(parent, testCalleeNames, namespaces))
      );
    },
    resolveFunctionCallback(node, callback) {
      return resolveFunctionCallback(context, node, callback);
    },
  };
}

module.exports = {
  createRuleHelpers,
};
