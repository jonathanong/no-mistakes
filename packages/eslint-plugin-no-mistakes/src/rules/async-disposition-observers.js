"use strict";

const { isPromiseMethodCall } = require("./async-disposition-bindings");
const { unwrapTransparentParent, unwrapExpression } = require("./async-ast");
const { resolveImportedValue } = require("./configured-runtime-import");
const { resolveVariable, isReassigned, memberPropertyName } = require("./async-target-bindings");
const { localFunction } = require("./async-consumer-local-proof");
const { observerMutated, globalObserverMutated } = require("./async-observer-mutations");
const { propertiesWritten } = require("./async-disposition-mocks");

function isVitestExpect(resolved) {
  return (
    resolved &&
    !resolved.unresolved &&
    resolved.imported === "expect" &&
    ["vitest", "@jest/globals"].includes(resolved.source)
  );
}

function expectCallee(callee, context) {
  const node = unwrapExpression(callee);
  if (node?.type === "MemberExpression") {
    return memberPropertyName(node) === "expect" && isVitestExpect(resolveImportedValue(node, context));
  }
  if (node?.type !== "Identifier" || node.name !== "expect") return false;
  if (isReassigned(node, context) || propertiesWritten(node, context)) return false;
  const resolved = resolveImportedValue(node, context);
  if (resolved) return isVitestExpect(resolved);
  return !resolveVariable(node, context)?.defs.length && !globalObserverMutated("expect", context);
}

function assertionExpression(node, context) {
  const argument = unwrapTransparentParent(node);
  const call = argument.parent;
  const callee = unwrapExpression(call?.callee);
  if (
    call?.type !== "CallExpression" ||
    call.arguments.length !== 1 ||
    call.arguments[0] !== argument ||
    !expectCallee(callee, context)
  )
    return null;
  const modifier = unwrapTransparentParent(call).parent;
  if (
    modifier?.type !== "MemberExpression" ||
    modifier.object !== call ||
    modifier.optional ||
    !["rejects", "resolves"].includes(memberPropertyName(modifier))
  )
    return null;
  const matcher = modifier.parent;
  const assertion = matcher?.parent;
  return matcher?.type === "MemberExpression" &&
    matcher.object === modifier &&
    !matcher.optional &&
    assertion?.type === "CallExpression" &&
    assertion.callee === matcher &&
    !assertion.optional
    ? assertion
    : null;
}

function promiseChainFrom(node, origin, context) {
  let current = unwrapExpression(node);
  while (current !== origin) {
    if (isPromiseMethodCall(current, context, ["resolve"])) {
      current = unwrapExpression(current.arguments[0]);
      continue;
    }
    const callee = unwrapExpression(current?.callee);
    if (
      current?.type !== "CallExpression" ||
      current.optional ||
      callee?.type !== "MemberExpression" ||
      callee.optional ||
      !["then", "catch", "finally"].includes(memberPropertyName(callee))
    )
      return false;
    current = unwrapExpression(callee.object);
  }
  return node !== origin;
}

function handledCatchStatement(node, context, origin) {
  const call = unwrapExpression(node);
  const callee = unwrapExpression(call?.callee);
  const handler = unwrapExpression(call?.arguments?.[0]);
  return Boolean(
    call?.type === "CallExpression" &&
    unwrapTransparentParent(call).parent?.type === "ExpressionStatement" &&
    callee?.type === "MemberExpression" &&
    !callee.optional &&
    memberPropertyName(callee) === "catch" &&
    !call.optional &&
    promiseChainFrom(call, origin, context) &&
    !(origin.type === "Identifier" && propertiesWritten(origin, context)) &&
    !observerMutated(origin, context) &&
    !globalObserverMutated("Promise", context) &&
    (["ArrowFunctionExpression", "FunctionExpression"].includes(handler?.type) ||
      localFunction(handler, context)),
  );
}

module.exports = { assertionExpression, handledCatchStatement, promiseChainFrom };
