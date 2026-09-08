"use strict";

const { unwrapExpression } = require("./async-ast");
const {
  memberPropertyName,
  repoRelativeFilename,
  stringMatches,
} = require("./module-mock-helpers");

const CHAIN_METHODS = new Set(["catch", "finally", "then"]);

function shouldCheckFile(filename, options) {
  const file = repoRelativeFilename(filename);
  const checked = options.checkedPathPatterns ?? [];
  const allowed = options.allowedPathPatterns ?? [];
  if (checked.length > 0 && !stringMatches(file, checked)) return false;
  return !stringMatches(file, allowed);
}

function promiseMethodName(call) {
  const callee = unwrapExpression(call.callee);
  if (callee?.type !== "MemberExpression" || callee.computed) return null;
  return callee.property.name;
}

function rejectionHandler(call) {
  const method = promiseMethodName(call);
  if (method === "catch") return call.arguments[0] ?? null;
  if (method === "then") return call.arguments[1] ?? null;
  return null;
}

function isVoidZero(node) {
  const current = unwrapExpression(node);
  if (current?.type !== "UnaryExpression" || current.operator !== "void") return false;
  const argument = unwrapExpression(current.argument);
  return argument?.type === "Literal" && argument.value === 0;
}

function isShadowedUndefined(node, sourceCode) {
  let scope = sourceCode?.getScope?.(node) ?? null;
  while (scope) {
    const variable = scope.set?.get("undefined");
    if (variable?.defs?.length > 0) return true;
    scope = scope.upper;
  }
  return false;
}

function isNoopExpression(node, sourceCode) {
  const current = unwrapExpression(node);
  if (current.type === "Identifier" && current.name === "undefined") {
    return Boolean(sourceCode) && !isShadowedUndefined(current, sourceCode);
  }
  return isVoidZero(current);
}

function isNoopLiteralExpression(node) {
  const current = unwrapExpression(node);
  return current?.type === "Literal" && current.regex == null;
}

function isNoopStatement(statement, sourceCode) {
  if (statement.type === "EmptyStatement") return true;
  if (statement.type === "ReturnStatement") {
    return statement.argument == null || isNoopExpression(statement.argument, sourceCode);
  }
  return (
    statement.type === "ExpressionStatement" &&
    (isNoopExpression(statement.expression, sourceCode) ||
      isNoopLiteralExpression(statement.expression))
  );
}

function isInlineFunction(node) {
  if (node?.type === "ArrowFunctionExpression") return true;
  return node?.type === "FunctionExpression" && node.id == null;
}

function isInlineNoopFunction(node, sourceCode) {
  const current = unwrapExpression(node);
  if (!isInlineFunction(current) || current.generator) return false;
  if (current.body.type !== "BlockStatement") return isNoopExpression(current.body, sourceCode);
  return current.body.body.every((statement) => isNoopStatement(statement, sourceCode));
}

function originatingCalleeName(call) {
  let current = unwrapExpression(call);
  while (current?.type === "CallExpression") {
    const method = promiseMethodName(current);
    if (!CHAIN_METHODS.has(method)) break;
    const callee = unwrapExpression(current.callee);
    current = unwrapExpression(callee?.object);
  }
  if (current?.type !== "CallExpression") return null;
  const callee = unwrapExpression(current.callee);
  if (callee?.type === "Identifier") return callee.name;
  if (callee?.type === "MemberExpression") return memberPropertyName(callee);
  return null;
}

function isAllowedCallee(call, options) {
  const name = originatingCalleeName(call);
  return Boolean(name && stringMatches(name, options.allowedCalleeNamePatterns ?? []));
}

module.exports = {
  isAllowedCallee,
  isInlineNoopFunction,
  originatingCalleeName,
  rejectionHandler,
  shouldCheckFile,
};
