"use strict";

const { unwrapTransparentParent, unwrapExpression } = require("./async-ast");
const { resolveVariable, isReassigned, memberPropertyName } = require("./async-target-bindings");
const { localFunction } = require("./async-consumer-local-proof");
const { propertiesWritten } = require("./async-disposition-mocks");

function assertionExpression(node, context) {
  const argument = unwrapTransparentParent(node);
  const call = argument.parent;
  const callee = unwrapExpression(call?.callee);
  if (
    call?.type !== "CallExpression" ||
    call.arguments.length !== 1 ||
    call.arguments[0] !== argument ||
    callee?.type !== "Identifier" ||
    callee.name !== "expect" ||
    isReassigned(callee, context) ||
    propertiesWritten(callee, context)
  )
    return null;
  const variable = resolveVariable(callee, context);
  const definition = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (
    variable?.defs.length &&
    (definition?.type !== "ImportBinding" ||
      definition.node.type !== "ImportSpecifier" ||
      definition.node.imported.name !== "expect" ||
      !["vitest", "@jest/globals"].includes(definition.parent.source.value))
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
    assertion?.type === "CallExpression" &&
    assertion.callee === matcher
    ? assertion
    : null;
}

function promiseChainFrom(node, origin) {
  let current = unwrapExpression(node);
  while (current !== origin) {
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
  const callee = unwrapExpression(node?.callee);
  const handler = unwrapExpression(node?.arguments?.[0]);
  return Boolean(
    node?.type === "CallExpression" &&
    node.parent?.type === "ExpressionStatement" &&
    callee?.type === "MemberExpression" &&
    !callee.optional &&
    memberPropertyName(callee) === "catch" &&
    !node.optional &&
    promiseChainFrom(node, origin) &&
    (["ArrowFunctionExpression", "FunctionExpression"].includes(handler?.type) ||
      localFunction(handler, context)),
  );
}

module.exports = { assertionExpression, handledCatchStatement };
