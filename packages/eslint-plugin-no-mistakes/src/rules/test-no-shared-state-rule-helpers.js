"use strict";

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
  if (!callee) return false;
  if (callee.type === "Identifier") return callee.name === name;
  if (callee.type === "CallExpression") return calleeHasProperty(callee.callee, name);
  if (callee.type !== "MemberExpression" || callee.computed) return false;
  return callee.property.name === name || calleeHasProperty(callee.object, name);
}

function createRuleHelpers(context, testCalleeNames) {
  return {
    calleeHasProperty,
    isDescribeCall(node) {
      return (
        (node.callee.type === "Identifier" && node.callee.name === "describe") ||
        (isKnownTestCallee(node.callee, testCalleeNames) &&
          calleeHasProperty(node.callee, "describe"))
      );
    },
    isInlineCallback(node) {
      const parent = node.parent;
      return (
        parent?.type === "CallExpression" &&
        (setupCallbackKind(parent, testCalleeNames) || isTestCall(parent, testCalleeNames))
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
