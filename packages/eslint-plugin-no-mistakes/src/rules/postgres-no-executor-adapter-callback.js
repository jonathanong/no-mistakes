"use strict";

const { localFunction } = require("./postgres-no-executor-adapter-target");

// A library closure may also be passed to a local helper whose parameter is a
// plain function type; see "Library executor terminals" in
// docs/eslint-rules/postgres-no-executor-adapter.md.
function callbackTarget(call, index, context, facts) {
  if (call.arguments.some((arg) => arg.type === "SpreadElement")) return false;
  const fn = localFunction(call.callee, context, facts);
  const param = fn?.params[index];
  const target = param?.type === "AssignmentPattern" ? param.left : param;
  return (
    target?.type === "Identifier" &&
    target.typeAnnotation?.typeAnnotation.type === "TSFunctionType" &&
    !fn.params.some((entry) => entry.type === "RestElement")
  );
}

module.exports = { callbackTarget };
