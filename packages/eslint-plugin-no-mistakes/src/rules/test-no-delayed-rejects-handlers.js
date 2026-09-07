"use strict";

const { unwrapExpression } = require("./async-ast");

function isSafeValue(node) {
  const unwrapped = unwrapExpression(node);
  if (unwrapped.type === "Literal") return true;
  if (unwrapped.type !== "UnaryExpression" || unwrapped.operator !== "void") return false;
  const operand = unwrapExpression(unwrapped.argument);
  return operand.type === "Literal";
}

function isNonRejectingHandler(argument) {
  if (!argument) return false;
  const handler = unwrapExpression(argument);
  if (handler.type !== "ArrowFunctionExpression" && handler.type !== "FunctionExpression") {
    return false;
  }
  if (handler.params.length > 1 || handler.params.some((item) => item.type !== "Identifier")) {
    return false;
  }
  if (handler.body.type !== "BlockStatement") return isSafeValue(handler.body);
  if (handler.body.body.length === 0) return true;
  if (handler.body.body.length !== 1 || handler.body.body[0].type !== "ReturnStatement") {
    return false;
  }
  const returned = handler.body.body[0].argument;
  return !returned || isSafeValue(returned);
}

function isAbsentHandler(argument) {
  if (!argument) return true;
  const unwrapped = unwrapExpression(argument);
  if (unwrapped.type === "Literal") return true;
  if (unwrapped.type !== "UnaryExpression" || unwrapped.operator !== "void") return false;
  const operand = unwrapExpression(unwrapped.argument);
  return operand.type === "Literal";
}

function isNonRejectingHandlerOrAbsent(argument) {
  return isAbsentHandler(argument) || isNonRejectingHandler(argument);
}

module.exports = { isNonRejectingHandler, isNonRejectingHandlerOrAbsent };
