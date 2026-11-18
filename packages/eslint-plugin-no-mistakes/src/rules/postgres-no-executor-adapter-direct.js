"use strict";

const { sqlText, unwrapTs } = require("./postgres-query-text");
const { resolvePassed } = require("./postgres-no-executor-adapter-passed");

function directOption(expression, call, index, property, context, bindings, options) {
  if (
    call.type !== "CallExpression" ||
    call.optional ||
    unwrapTs(call.callee).optional ||
    property?.type !== "Property" ||
    property.computed
  )
    return false;
  const object = unwrapTs(call.arguments[index]);
  if (object?.type !== "ObjectExpression" || property.parent !== object) return false;
  const direct = bindings.resolveCall?.(call, false);
  if (
    !direct ||
    sqlText(direct.arguments[0]) === null ||
    !direct.arguments.slice(1).some((argument) => unwrapTs(argument) === object)
  )
    return false;
  const passed = resolvePassed(expression, context, bindings, options, new Set());
  return Boolean(passed?.definite && !passed.adapter);
}

module.exports = { directOption };
