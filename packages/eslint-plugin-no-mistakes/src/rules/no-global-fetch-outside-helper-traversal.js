"use strict";

const MAYBE_EXECUTED_ANCESTORS = new Set([
  "ClassBody",
  "ConditionalExpression",
  "DoWhileStatement",
  "FieldDefinition",
  "ForInStatement",
  "ForOfStatement",
  "ForStatement",
  "IfStatement",
  "LogicalExpression",
  "PropertyDefinition",
  "SwitchCase",
  "SwitchStatement",
  "TryStatement",
  "WhileStatement",
]);

function childNodes(node) {
  const children = [];
  for (const [key, value] of Object.entries(node)) {
    if (key === "parent") continue;
    if (Array.isArray(value)) {
      for (const item of value) {
        if (item?.type) children.push(item);
      }
    } else if (value?.type) {
      children.push(value);
    }
  }
  return children;
}

function collectVariableDeclarators(node, declarators = []) {
  if (node.type === "VariableDeclarator") declarators.push(node);
  for (const child of childNodes(node)) collectVariableDeclarators(child, declarators);
  return declarators;
}

function collectAssignmentExpressions(node, assignments = []) {
  if (node.type === "AssignmentExpression") assignments.push(node);
  for (const child of childNodes(node)) collectAssignmentExpressions(child, assignments);
  return assignments;
}

function isMaybeExecuted(node) {
  let child = node;
  let current = node.parent;
  while (current) {
    if (current.type === "ForStatement" && current.init === child) {
      child = current;
      current = current.parent;
      continue;
    }
    if (MAYBE_EXECUTED_ANCESTORS.has(current.type)) return true;
    if (
      current.type === "FunctionDeclaration" ||
      current.type === "FunctionExpression" ||
      current.type === "ArrowFunctionExpression"
    ) {
      return false;
    }
    child = current;
    current = current.parent;
  }
  return false;
}

module.exports = {
  childNodes,
  collectAssignmentExpressions,
  collectVariableDeclarators,
  isMaybeExecuted,
};
