"use strict";

const { isTransparentExpression, unwrapExpression } = require("./async-ast");
const { memberName } = require("./playwright-static-call");
const { resolveAlias, variableFor } = require("./playwright-static-call-alias");

const LOCATOR_REFINEMENTS = new Set(["first", "last", "nth", "filter"]);

function sentinelSelector(argument) {
  const target = unwrapExpression(argument);
  if (target?.type === "Literal" && typeof target.value === "string") return target.value;
  if (target?.type === "TemplateLiteral" && target.expressions.length === 0) {
    return target.quasis[0]?.value.cooked ?? null;
  }
  return null;
}

function isSentinelScrollCall(node, context, locatorNames, testIds) {
  const callee = unwrapExpression(node.callee);
  if (memberName(callee) !== "scrollIntoViewIfNeeded") return false;
  let receiver = callee?.object;
  let named = false;
  const seen = new Set();
  for (;;) {
    if (!receiver) return false;
    if (isTransparentExpression(receiver)) {
      receiver = receiver.expression;
      continue;
    }
    if (receiver.type === "Identifier") {
      const isConfigured = locatorNames.has(receiver.name);
      const alias = resolveAlias(receiver, context, seen);
      if (alias?.kind === "init") {
        named ||= isConfigured;
        receiver = alias.init;
        continue;
      }
      if (named || isConfigured) {
        const variable = variableFor(receiver, context);
        const def = variable?.defs?.[0];
        if (!def || def.type !== "Parameter") return true;
      }
      return false;
    }
    if (receiver.type !== "CallExpression") return false;
    if (named) return true;
    const method = memberName(receiver.callee);
    if (LOCATOR_REFINEMENTS.has(method)) {
      receiver = receiver.callee.object;
      continue;
    }
    const selector = sentinelSelector(receiver.arguments[0]);
    if (method === "getByTestId") return testIds.has(selector);
    if (method === "locator" && typeof selector === "string") {
      const attribute = /^\[data-testid=(?:"([^"]+)"|'([^']+)'|([\w-]+))\]$/.exec(selector);
      return !!attribute && testIds.has(attribute[1] ?? attribute[2] ?? attribute[3]);
    }
    return false;
  }
}

module.exports = { isSentinelScrollCall };
