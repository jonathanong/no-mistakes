"use strict";

const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const {
  isNonRejectingHandler,
  isNonRejectingHandlerOrAbsent,
} = require("./test-no-delayed-rejects-handlers");

const PROMISE_CHAIN_METHODS = new Set(["catch", "finally", "then"]);

function literalPropertyName(member) {
  if (!member.computed && member.property.type === "Identifier") return member.property.name;
  if (member.computed && member.property.type === "Literal") return member.property.value;
  return null;
}

function promiseChainBase(node) {
  let current = unwrapExpression(node);
  while (
    current.type === "CallExpression" &&
    current.callee.type === "MemberExpression" &&
    PROMISE_CHAIN_METHODS.has(literalPropertyName(current.callee))
  ) {
    current = unwrapExpression(current.callee.object);
  }
  return current;
}

function continuationCall(node) {
  const chainNode = unwrapTransparentParent(node);
  const member = chainNode.parent;
  if (
    member?.type !== "MemberExpression" ||
    member.object !== chainNode ||
    !PROMISE_CHAIN_METHODS.has(literalPropertyName(member)) ||
    member.parent?.type !== "CallExpression" ||
    member.parent.callee !== member
  ) {
    return null;
  }
  return member.parent;
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

module.exports = { chainIsSafelyObserved, literalPropertyName, promiseChainBase };
