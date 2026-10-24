"use strict";

const { literalString } = require("./module-mock-helpers");
const {
  callbackCall,
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
  if (callee?.type === "SequenceExpression" && callee.expressions.length > 0) {
    return calleeHasProperty(callee.expressions.at(-1), name);
  }
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
      const parent = callbackCall(node);
      return (
        parent &&
        (setupCallbackKind(parent, testCalleeNames, namespaces) ||
          isTestCall(parent, testCalleeNames, namespaces))
      );
    },
    resolveFunctionCallback(node, callback) {
      return resolveFunctionCallback(context, node, callback);
    },
    configureSerialMode(node) {
      if (!calleeHasProperty(node.callee, "configure")) return false;
      if (
        node.callee.type !== "MemberExpression" ||
        !isKnownTestCallee(node.callee.object, testCalleeNames, namespaces) ||
        !calleeHasProperty(node.callee.object, "describe")
      ) {
        return false;
      }
      const options = node.arguments[0];
      if (options?.type !== "ObjectExpression") return false;
      return options.properties.some(
        (property) =>
          property.type === "Property" &&
          (property.key?.name ?? property.key?.value) === "mode" &&
          property.value?.type === "Literal" &&
          property.value.value === "serial",
      );
    },
  };
}

module.exports = {
  createRuleHelpers,
};
