"use strict";

const { unwrapExpression } = require("./async-ast");

function isSafeValue(node, parameter) {
  const unwrapped = unwrapExpression(node);
  if (
    unwrapped.type === "Literal" ||
    (unwrapped.type === "Identifier" &&
      (unwrapped.name === "undefined" || unwrapped.name === parameter))
  ) {
    return true;
  }
  if (unwrapped.type !== "UnaryExpression" || unwrapped.operator !== "void") return false;
  const operand = unwrapExpression(unwrapped.argument);
  return (
    operand.type === "Literal" || (operand.type === "Identifier" && operand.name === "undefined")
  );
}

function isNonRejectingHandler(argument) {
  const handler = unwrapExpression(argument);
  if (handler.type !== "ArrowFunctionExpression" && handler.type !== "FunctionExpression") {
    return false;
  }
  if (handler.params.length > 1 || handler.params.some((item) => item.type !== "Identifier")) {
    return false;
  }
  const parameter = handler.params[0]?.name ?? null;
  if (handler.body.type !== "BlockStatement") return isSafeValue(handler.body, parameter);
  if (handler.body.body.length === 0) return true;
  if (handler.body.body.length !== 1 || handler.body.body[0].type !== "ReturnStatement") {
    return false;
  }
  const returned = handler.body.body[0].argument;
  return !returned || isSafeValue(returned, parameter);
}

module.exports = { isNonRejectingHandler };
