"use strict";

const { isModuleMockMemberCall, propertyName } = require("./module-mock-helpers");

function matchDirectMockCallApply(node, context, methods) {
  if (
    node.callee.type !== "MemberExpression" ||
    !["call", "apply"].includes(propertyName(node.callee.property)) ||
    node.callee.object.type !== "MemberExpression"
  ) {
    return null;
  }
  const direct = isModuleMockMemberCall({ callee: node.callee.object }, context);
  if (!direct || !methods.has(direct.method)) return null;
  if (propertyName(node.callee.property) === "call") {
    return { factory: node.arguments[2], specifierNode: node.arguments[1] };
  }
  const args = node.arguments[1];
  return {
    factory: args?.type === "ArrayExpression" ? args.elements[1] : undefined,
    specifierNode: args?.type === "ArrayExpression" ? args.elements[0] : undefined,
  };
}

module.exports = { matchDirectMockCallApply };
