"use strict";

const {
  unwrapExpression,
  staticMemberName,
  callableTarget,
} = require("./ts-no-function-aliases-ast");
const { resolveVariable, sameReference } = require("./ts-no-function-aliases-self");

function isUndefinedReceiver(expression, sourceCode) {
  const node = unwrapExpression(expression);
  if (node.type === "Identifier") {
    const variable = resolveVariable(sourceCode, node);
    return node.name === "undefined" && (!variable || variable.defs.length === 0);
  }
  return (
    node.type === "UnaryExpression" &&
    node.operator === "void" &&
    node.argument.type === "Literal" &&
    node.argument.value === 0
  );
}

function receiverMatchesTarget(target, thisArg, sourceCode) {
  if (target.type === "MemberExpression") {
    return sameReference(target.object, thisArg, sourceCode, false);
  }
  return isUndefinedReceiver(thisArg, sourceCode);
}

function forwardedCall(call, sourceCode) {
  const callee = unwrapExpression(call.callee);
  if (callee?.type !== "MemberExpression") return null;
  const method = staticMemberName(callee);
  if (method !== "call" && method !== "apply") return null;
  const target = callableTarget(callee.object);
  if (!target || call.arguments.length < 1) return null;
  if (!receiverMatchesTarget(target, call.arguments[0], sourceCode)) return null;
  if (method === "call") return { target, args: call.arguments.slice(1) };
  if (call.arguments.length !== 2 || call.arguments[1].type !== "ArrayExpression") return null;
  const args = call.arguments[1].elements;
  if (args.some((argument) => argument === null)) return null;
  return { target, args };
}

module.exports = { forwardedCall };
