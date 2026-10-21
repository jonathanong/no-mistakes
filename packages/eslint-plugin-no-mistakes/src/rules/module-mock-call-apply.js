"use strict";

const { unwrapExpression } = require("./async-ast");
const { isModuleMockMemberCall, memberPropertyName } = require("./module-mock-helpers");

function unwrap(node) {
  while (node?.type === "ChainExpression") node = node.expression;
  return node;
}

function matchDirectMockCallApply(node, context, methods) {
  const callee = unwrap(node.callee);
  const object = unwrap(callee?.object);
  const name = callee?.type === "MemberExpression" ? memberPropertyName(callee) : null;
  if (!["call", "apply"].includes(name) || object?.type !== "MemberExpression") return null;
  const direct = isModuleMockMemberCall({ callee: object }, context);
  if (!direct || !methods.has(direct.method)) return null;
  const args = node.arguments ?? [];
  if (args.some((arg) => arg?.type === "SpreadElement")) return null;
  if (name === "call") {
    return { factory: args[2], mock: direct, specifierNode: args[1] };
  }
  const list = unwrapExpression(args[1]);
  if (
    list?.type === "ArrayExpression" &&
    list.elements.some((element) => !element || element.type === "SpreadElement")
  ) {
    return null;
  }
  return {
    factory: list?.type === "ArrayExpression" ? list.elements[1] : undefined,
    mock: direct,
    specifierNode: list?.type === "ArrayExpression" ? list.elements[0] : undefined,
  };
}

module.exports = { matchDirectMockCallApply };
