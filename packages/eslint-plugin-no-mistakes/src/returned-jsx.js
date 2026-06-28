"use strict";

const { attributeName } = require("./helpers");
const { isExpressionWrapper, isFunctionNode } = require("./component-functions");

function returnedJsxBranches(fn) {
  if (fn.type === "ArrowFunctionExpression" && fn.body.type !== "BlockStatement") {
    return jsxBranches(fn.body);
  }
  const branches = [];
  collectReturnBranches(fn.body, branches);
  return branches;
}

function collectReturnBranches(node, branches) {
  if (!node || (node.type !== "BlockStatement" && isFunctionNode(node))) {
    return;
  }
  if (node.type === "ReturnStatement") {
    for (const branch of jsxBranches(node.argument)) {
      branches.push(branch);
    }
    return;
  }
  if (node.type === "ClassDeclaration" || node.type === "ClassExpression") {
    return;
  }
  for (const child of childNodes(node)) {
    collectReturnBranches(child, branches);
  }
}

function jsxBranches(node) {
  const unwrapped = unwrapSyntaxExpression(node);
  if (!unwrapped || isNullLiteral(unwrapped)) {
    return [];
  }
  if (isJsxNode(unwrapped)) {
    return [unwrapped];
  }
  if (unwrapped.type === "ConditionalExpression") {
    return [...jsxBranches(unwrapped.consequent), ...jsxBranches(unwrapped.alternate)];
  }
  if (unwrapped.type === "LogicalExpression") {
    return [...jsxBranches(unwrapped.left), ...jsxBranches(unwrapped.right)];
  }
  if (unwrapped.type === "SequenceExpression") {
    return jsxBranches(unwrapped.expressions.at(-1));
  }
  if (unwrapped.type === "ArrayExpression") {
    return unwrapped.elements.flatMap((element) => jsxBranches(element));
  }
  return [];
}

function unwrapSyntaxExpression(node) {
  let current = node;
  while (current && isExpressionWrapper(current)) {
    current = current.expression;
  }
  return current;
}

function isNullLiteral(node) {
  return node.type === "Literal" && node.value === null;
}

function isJsxNode(node) {
  return node.type === "JSXElement" || node.type === "JSXFragment";
}

function jsxTreeHasAttribute(node, opts) {
  let found = false;
  visitNode(node, (current) => {
    if (found || current.type !== "JSXOpeningElement") {
      return;
    }
    found = current.attributes.some((attribute) => {
      if (attribute.type === "JSXSpreadAttribute") {
        return opts.allowSpreadAttributes;
      }
      const name = attributeName(attribute);
      return Boolean(name && opts.attributes.includes(name));
    });
  });
  return found;
}

function visitNode(node, callback) {
  if (!node || typeof node.type !== "string") {
    return;
  }
  callback(node);
  for (const child of childNodes(node)) {
    visitNode(child, callback);
  }
}

function childNodes(node) {
  const children = [];
  for (const key in node) {
    if (key === "parent" || key === "tokens" || key === "comments") {
      continue;
    }
    if (Object.prototype.hasOwnProperty.call(node, key)) {
      const value = node[key];
      if (Array.isArray(value)) {
        for (let i = 0; i < value.length; i++) {
          const child = value[i];
          if (isAstNode(child)) {
            children.push(child);
          }
        }
      } else if (isAstNode(value)) {
        children.push(value);
      }
    }
  }
  return children;
}

function isAstNode(value) {
  return Boolean(value && typeof value.type === "string");
}

module.exports = {
  jsxTreeHasAttribute,
  returnedJsxBranches,
};
