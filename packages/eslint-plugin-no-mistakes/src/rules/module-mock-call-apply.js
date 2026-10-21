"use strict";

const { isModuleMockMemberCall, propertyName } = require("./module-mock-helpers");

function unwrap(node) {
  while (node?.type === "ChainExpression") node = node.expression;
  return node;
}

function matchDirectMockCallApply(node, context, methods) {
  const callee = unwrap(node.callee);
  const object = unwrap(callee?.object);
  const name = callee?.type === "MemberExpression" ? propertyName(callee.property) : null;
  if (!["call", "apply"].includes(name) || object?.type !== "MemberExpression") return null;
  const direct = isModuleMockMemberCall({ callee: object }, context);
  if (!direct || !methods.has(direct.method)) return null;
  if (name === "call") {
    return { factory: node.arguments[2], mock: direct, specifierNode: node.arguments[1] };
  }
  const args = node.arguments[1];
  return {
    factory: args?.type === "ArrayExpression" ? args.elements[1] : undefined,
    mock: direct,
    specifierNode: args?.type === "ArrayExpression" ? args.elements[0] : undefined,
  };
}

module.exports = { matchDirectMockCallApply };
