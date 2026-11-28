"use strict";

const {
  isTransparentExpression,
  unwrapExpression,
  unwrapTransparentParent,
} = require("./async-ast");
const { literalString } = require("./module-mock-helpers");
const {
  isNonRejectingHandler,
  isNonRejectingHandlerOrAbsent,
} = require("./test-no-delayed-rejects-handlers");

const PROMISE_CHAIN_METHODS = new Set(["catch", "finally", "then"]);

function literalPropertyName(member) {
  member = unwrapExpression(member);
  if (member?.type !== "MemberExpression") return null;
  const property = unwrapExpression(member.property);
  if (!property) return null;
  if (member.computed && property.type === "Identifier") return null;
  if (property.type === "Identifier") return property.name;
  return literalString(property);
}

function isPromiseChainMember(node) {
  const member = unwrapExpression(node);
  return (
    member?.type === "MemberExpression" && PROMISE_CHAIN_METHODS.has(literalPropertyName(member))
  );
}

function promiseChainBase(node) {
  let current = unwrapExpression(node);
  while (current?.type === "CallExpression" && isPromiseChainMember(current.callee)) {
    current = unwrapExpression(unwrapExpression(current.callee).object);
  }
  return current;
}

function continuationCall(node) {
  let chainNode = unwrapTransparentParent(node);
  while (isTransparentExpression(chainNode.parent)) chainNode = chainNode.parent;
  const member = chainNode.parent;
  if (
    member?.type !== "MemberExpression" ||
    unwrapExpression(member.object) !== unwrapExpression(chainNode) ||
    !PROMISE_CHAIN_METHODS.has(literalPropertyName(member))
  ) {
    return null;
  }
  let parent = member.parent;
  while (isTransparentExpression(parent)) parent = parent.parent;
  return parent?.type === "CallExpression" && unwrapExpression(parent.callee) === member
    ? parent
    : null;
}

function initialCallIsSafe(node) {
  const property = literalPropertyName(node.callee);
  if (property === "catch") return isNonRejectingHandler(node.arguments[0]);
  return (
    property === "then" &&
    isNonRejectingHandlerOrAbsent(node.arguments[0]) &&
    isNonRejectingHandler(node.arguments[1])
  );
}

function applyContinuationSafety(safe, node) {
  const property = literalPropertyName(node.callee);
  if (property === "finally") {
    return safe && isNonRejectingHandlerOrAbsent(node.arguments[0]);
  }
  if (property === "catch") {
    return safe || isNonRejectingHandler(node.arguments[0]);
  }
  if (safe) return isNonRejectingHandlerOrAbsent(node.arguments[0]);
  return isNonRejectingHandler(node.arguments[0]) && isNonRejectingHandler(node.arguments[1]);
}

function chainIsSafelyObserved(node) {
  let safe = initialCallIsSafe(node);
  if (!safe) return false;
  let current = node;
  while (true) {
    const continuation = continuationCall(current);
    if (!continuation) return safe;
    safe = applyContinuationSafety(safe, continuation);
    current = continuation;
  }
}

module.exports = {
  chainIsSafelyObserved,
  isPromiseChainMember,
  literalPropertyName,
  promiseChainBase,
};
