"use strict";

const { unwrapExpression } = require("./async-ast");
const {
  logicalCall,
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
  const invoked = logicalCall(call);
  if (invoked?.callee?.type !== "MemberExpression") return null;
  return memberPropertyName(invoked.callee);
}

function rejectionHandler(call) {
  const method = promiseMethodName(call);
  const args = logicalCall(call)?.arguments ?? [];
  if (method === "catch") return args[0] ?? null;
  if (method === "then") return args[1] ?? null;
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
  if (current?.type === "Literal") return true;
  return current?.type === "TemplateLiteral" && current.expressions.length === 0;
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

function chainOrigin(call) {
  let current = call;
  while (current?.type === "CallExpression") {
    const invoked = logicalCall(current);
    const method = invoked ? memberPropertyName(invoked.callee) : null;
    if (!CHAIN_METHODS.has(method)) break;
    const object = unwrapExpression(invoked.callee?.object);
    if (object?.type !== "CallExpression") return null;
    current = object;
  }
  return current?.type === "CallExpression" ? current : null;
}

function originatingCalleeName(call) {
  const current = chainOrigin(call);
  if (!current) return null;
  const callee = unwrapExpression(logicalCall(current)?.callee ?? current.callee);
  if (callee?.type === "Identifier") return callee.name;
  if (callee?.type === "MemberExpression") return memberPropertyName(callee);
  return null;
}

function spelledOriginName(call) {
  const callee = unwrapExpression(chainOrigin(call)?.callee);
  return callee?.type === "MemberExpression" ? memberPropertyName(callee) : null;
}

function isAllowedCallee(call, options) {
  const patterns = options.allowedCalleeNamePatterns ?? [];
  const spelled = spelledOriginName(call);
  if ((spelled === "call" || spelled === "apply") && stringMatches(spelled, patterns)) return true;
  const name = originatingCalleeName(call);
  return Boolean(name && stringMatches(name, patterns));
}

module.exports = {
  isAllowedCallee,
  isInlineNoopFunction,
  originatingCalleeName,
  rejectionHandler,
  shouldCheckFile,
};
