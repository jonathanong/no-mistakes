"use strict";

const { literalString } = require("./helpers");

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
  for (const arg of wrapperArguments(node)) {
    const unwrapped = unwrapExpression(arg, opts);
    if (isFunctionNode(unwrapped)) {
      return unwrapped;
    }
  }
  return node;
}

function wrapperArguments(node) {
  const args = node.arguments || [];
  const name = staticCalleeName(node.callee);
  if (name !== "call" && name !== "apply") return args;
  if (args.some((arg) => arg?.type === "SpreadElement")) return [];
  if (name === "call") return args.slice(1);
  const list = args[1];
  if (
    list?.type !== "ArrayExpression" ||
    list.elements.some((element) => !element || element.type === "SpreadElement")
  ) {
    return [];
  }
  return list.elements;
}

function isConfiguredWrapper(callee, opts) {
  const name = calleeName(callee);
  return Boolean(name && opts.wrappers.has(name));
}

function staticCalleeName(node) {
  if (!node || node.type !== "MemberExpression") return null;
  if (node.computed && node.property?.type === "Identifier") return null;
  if (!node.computed && typeof node.property?.name === "string") return node.property.name;
  return literalString(node.property);
}

function calleeName(node) {
  let current = node;
  for (let depth = 0; depth < 4; depth += 1) {
    while (current?.type === "ChainExpression") current = current.expression;
    if (current?.type === "MemberExpression") {
      const name = staticCalleeName(current);
      if (name === "call" || name === "apply") {
        current = current.object;
        continue;
      }
      return name;
    }
    break;
  }
  return current?.type === "Identifier" ? current.name : null;
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
