"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { childNodes, resolveVariable, unwrapTs } = require("./postgres-query-text");

const { findAppends } = require("./postgres-no-correlated-sql-builder-append");
const { completeGuards } = require("./postgres-correlated-completion");

const RULE_ID = "postgres-no-correlated-sql-builder";

function isFunction(node) {
  return (
    node?.type === "FunctionDeclaration" ||
    node?.type === "FunctionExpression" ||
    node?.type === "ArrowFunctionExpression"
  );
}

function stringText(node) {
  const value = unwrapTs(node);
  if (value.type === "Literal" && typeof value.value === "string") return value.value;
  if (value.type === "TemplateLiteral" && value.expressions.length === 0) {
    return value.quasis[0].value.cooked;
  }
  return null;
}

function selectorOf(test) {
  if (test.type !== "BinaryExpression") return null;
  if (test.operator !== "===" && test.operator !== "==") return null;
  const left = unwrapTs(test.left);
  const right = unwrapTs(test.right);
  if (left?.type === "Identifier" && stringText(right) != null) return left;
  if (right?.type === "Identifier" && stringText(left) != null) return right;
  return null;
}

function readGuards(statement, methods, tags, context) {
  const selector = selectorOf(unwrapTs(statement.test));
  if (!selector) return [];
  const builders = findAppends(statement.consequent, methods, tags, context);
  const test = unwrapTs(statement.test);
  const value = stringText(test.left) ?? stringText(test.right);
  return builders.map((builder) => ({
    node: statement,
    selector,
    builder: builder.object,
    call: builder.call,
    value,
  }));
}

function guardsOf(func, methods, tags, context) {
  const found = [];
  const visit = (node) => {
    if (!node || typeof node !== "object" || isFunction(node)) return;
    if (node.type === "IfStatement") {
      found.push(...readGuards(node, methods, tags, context));
    }
    for (const child of childNodes(node)) visit(child);
  };
  visit(func.body);
  return found;
}

function groupGuards(guards, context) {
  const selectors = new Map();
  for (const guard of guards) {
    const selector = resolveVariable(guard.selector, context);
    const builder = resolveVariable(guard.builder, context);
    if (!selector || !builder) continue;

    let builders = selectors.get(selector);
    if (!builders) {
      builders = new Map();
      selectors.set(selector, builders);
    }
    const group = builders.get(builder) ?? [];
    group.push(guard);
    builders.set(builder, group);
  }
  return [...selectors.values()].flatMap((builders) => [...builders.values()]);
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow repeated guards that append SQL to one builder for one selector",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        additionalProperties: false,
        properties: {
          builderMethods: { type: "array", items: { type: "string", minLength: 1 } },
          tagNames: { type: "array", items: { type: "string", minLength: 1 } },
        },
      },
    ],
    messages: {
      correlated:
        "Independent appends on {{builder}} can leave an unknown or empty path for {{selector}}. Return a complete statement from one switch ({{cases}}) with a throwing default. A runtime own-key check, or a default that does not throw, is not judged.",
    },
  },
  (context) => {
    const option = context.options?.[0] ?? {};
    const methods = new Set(option.builderMethods ?? []);
    const tags = new Set(option.tagNames ?? []);
    if (methods.size === 0 || tags.size === 0) return {};
    const suppressed = ruleSuppression(context, RULE_ID);

    function check(func) {
      for (const group of groupGuards(guardsOf(func, methods, tags, context), context)) {
        if (group.length < 2 || completeGuards(group, func) || suppressed(group[0].node)) continue;
        context.report({
          node: group[0].node,
          messageId: "correlated",
          data: {
            selector: group[0].selector.name,
            builder: group[0].builder.name,
            cases: [...new Set(group.map((guard) => guard.value))]
              .map((value) => `case ${JSON.stringify(value)}:`)
              .join(" "),
          },
        });
      }
    }

    return {
      FunctionDeclaration: check,
      FunctionExpression: check,
      ArrowFunctionExpression: check,
    };
  },
);
