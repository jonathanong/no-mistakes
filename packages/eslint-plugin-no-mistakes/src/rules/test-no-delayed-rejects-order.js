"use strict";

function contains(ancestor, node) {
  return ancestor.range[0] <= node.range[0] && ancestor.range[1] >= node.range[1];
}

function orderedChildren(node) {
  if (node.type === "CallExpression" || node.type === "NewExpression") {
    return [node.callee, ...node.arguments];
  }
  if (node.type === "ArrayExpression") return node.elements.filter(Boolean);
  if (node.type === "SequenceExpression") return node.expressions;
  return null;
}

function childExecutesBefore(parent, child, later) {
  const children = orderedChildren(parent);
  if (!children) return false;
  const childIndex = children.indexOf(child);
  const laterIndex = children.findIndex((candidate) => contains(candidate, later));
  return childIndex !== -1 && laterIndex !== -1 && childIndex < laterIndex;
}

module.exports = { childExecutesBefore };
