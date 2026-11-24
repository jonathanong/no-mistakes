"use strict";

const { memberName } = require("./playwright-static-call");
const { resolveAlias } = require("./playwright-static-call-alias");

const LOCATOR_BUILDERS = new Set([
  "locator",
  "getByTestId",
  "getByRole",
  "getByText",
  "getByLabel",
  "getByAltText",
  "getByPlaceholder",
  "getByTitle",
]);
const LOCATOR_REFINEMENTS = new Set(["first", "last", "nth", "filter"]);

function sentinelSelector(argument) {
  if (argument?.type === "Literal") return argument.value;
  if (argument?.type === "TemplateLiteral" && argument.expressions.length === 0) {
    return argument.quasis[0].value.cooked;
  }
  return null;
}

function isSentinelScrollCall(node, context, locatorNames, testIds) {
  if (memberName(node.callee) !== "scrollIntoViewIfNeeded") return false;
  let receiver = node.callee.object;
  let named = false;
  const seen = new Set();
  for (;;) {
    if (receiver.type === "ChainExpression") {
      receiver = receiver.expression;
      continue;
    }
    if (receiver.type === "Identifier") {
      const alias = resolveAlias(receiver, context, seen);
      if (alias?.kind !== "init") return false;
      named ||= locatorNames.has(receiver.name);
      receiver = alias.init;
      continue;
    }
    if (receiver.type !== "CallExpression") return false;
    const method = memberName(receiver.callee);
    if (LOCATOR_REFINEMENTS.has(method)) {
      receiver = receiver.callee.object;
      continue;
    }
    if (named && LOCATOR_BUILDERS.has(method)) return true;
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
