"use strict";

const transparentExpressionTypes = new Set([
  "ChainExpression",
  "TSNonNullExpression",
  "TSAsExpression",
  "TSTypeAssertion",
  "TSSatisfiesExpression",
]);

function isFunction(node) {
  return (
    node?.type === "ArrowFunctionExpression" ||
    node?.type === "FunctionDeclaration" ||
    node?.type === "FunctionExpression"
  );
}

function findContainingFunction(node) {
  let current = node.parent;
  while (current) {
    if (isFunction(current)) return current;
    current = current.parent;
  }
}

function isTransparentExpression(node) {
  return transparentExpressionTypes.has(node?.type);
}

function unwrapExpression(node) {
  let current = node;
  while (isTransparentExpression(current)) current = current.expression;
  return current;
}

function unwrapTransparentParent(node) {
  let current = node;
  while (isTransparentExpression(current.parent)) current = current.parent;
  return current;
}

function visitorKeys(context, node) {
  return context.sourceCode.visitorKeys[node.type] || [];
}

function traverse(context, node, visit, root = node) {
  if (!node) return;
  if (node !== root && isFunction(node)) return;
  visit(node);
  for (const key of visitorKeys(context, node)) {
    const value = node[key];
    if (Array.isArray(value)) {
      for (const child of value) {
        if (child?.type) traverse(context, child, visit, root);
      }
    } else if (value?.type) {
      traverse(context, value, visit, root);
    }
  }
}

function isUnconditionalBeforeReturn(node, block) {
  let current = node;
  while (current && current !== block) {
    const parent = current.parent;
    if (!parent || parent.type === "IfStatement" || parent.type.endsWith("Expression")) {
      return false;
    }
    if (
      parent.type.endsWith("Statement") &&
      parent.type !== "ExpressionStatement" &&
      parent.type !== "BlockStatement"
    ) {
      return false;
    }
    current = parent;
  }
  return current === block;
}

module.exports = {
  findContainingFunction,
  isFunction,
  isTransparentExpression,
  isUnconditionalBeforeReturn,
  traverse,
  unwrapExpression,
  unwrapTransparentParent,
};
