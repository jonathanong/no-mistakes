"use strict";

const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const {
  isPromiseChainMember,
  literalPropertyName,
  promiseChainBase,
} = require("./test-no-delayed-rejects-chains");

const PROMISE_AGGREGATES = new Set(["all", "allSettled", "any", "race"]);

function findVariable(scope, name) {
  let current = scope;
  while (current) {
    const variable =
      current.set?.get?.(name) ?? current.variables?.find((item) => item.name === name);
    if (variable) return variable;
    current = current.upper;
  }
  return null;
}

function isImmediateObserver(node, declarator, context, isSameConst) {
  if (node.type === "AwaitExpression") {
    const argument = unwrapExpression(node.argument);
    return (
      isSameConst(argument, declarator, context) ||
      (argument.type === "CallExpression" &&
        isPromiseChainMember(argument.callee) &&
        isSameConst(promiseChainBase(argument), declarator, context))
    );
  }
  if (
    node.type !== "CallExpression" ||
    node.callee.type !== "MemberExpression" ||
    unwrapExpression(node.callee.object).type !== "Identifier" ||
    unwrapExpression(node.callee.object).name !== "Promise" ||
    !PROMISE_AGGREGATES.has(literalPropertyName(node.callee)) ||
    node.arguments.length !== 1
  ) {
    return false;
  }
  const promise = unwrapExpression(node.callee.object);
  const variable = findVariable(context.sourceCode.getScope(promise), promise.name);
  if (variable?.defs.length) return false;
  const iterable = unwrapExpression(node.arguments[0]);
  if (
    iterable.type !== "ArrayExpression" ||
    !iterable.elements.some(
      (element) => element && isSameConst(unwrapExpression(element), declarator, context),
    )
  ) {
    return false;
  }
  if (literalPropertyName(node.callee) === "allSettled") return true;
  const aggregate = unwrapTransparentParent(node);
  return aggregate.parent?.type === "AwaitExpression" && aggregate.parent.argument === aggregate;
}

module.exports = { isImmediateObserver };
