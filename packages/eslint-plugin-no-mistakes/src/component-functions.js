"use strict";

function nonEmptyStrings(value, fallback) {
  if (!Array.isArray(value)) {
    return fallback;
  }
  const strings = value.filter((item) => typeof item === "string" && item.length > 0);
  return strings.length > 0 ? strings : fallback;
}

function functionFromExpression(node, opts) {
  const unwrapped = unwrapExpression(node, opts);
  return isFunctionNode(unwrapped) ? unwrapped : null;
}

function unwrapExpression(node, opts) {
  if (!node) {
    return null;
  }
  if (isExpressionWrapper(node)) {
    return unwrapExpression(node.expression, opts);
  }
  if (node.type !== "CallExpression" || !isConfiguredWrapper(node.callee, opts)) {
    return node;
  }
  for (const arg of node.arguments) {
    const unwrapped = unwrapExpression(arg, opts);
    if (isFunctionNode(unwrapped)) {
      return unwrapped;
    }
  }
  return node;
}

function isConfiguredWrapper(callee, opts) {
  const name = calleeName(callee);
  return Boolean(name && opts.wrappers.has(name));
}

function calleeName(node) {
  if (node.type === "Identifier") {
    return node.name;
  }
  if (node.type === "MemberExpression" && !node.computed) {
    return node.property.name;
  }
  return null;
}

function isFunctionNode(node) {
  return Boolean(
    node &&
    (node.type === "FunctionDeclaration" ||
      node.type === "FunctionExpression" ||
      node.type === "ArrowFunctionExpression"),
  );
}

function isExpressionWrapper(node) {
  return [
    "ChainExpression",
    "ParenthesizedExpression",
    "TSAsExpression",
    "TSSatisfiesExpression",
    "TSNonNullExpression",
    "TSTypeAssertion",
    "TypeCastExpression",
  ].includes(node.type);
}

module.exports = {
  functionFromExpression,
  isExpressionWrapper,
  isFunctionNode,
  nonEmptyStrings,
};
