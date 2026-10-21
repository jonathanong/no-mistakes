"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { childNodes, resolveVariable, unwrapTs } = require("./postgres-query-text");

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

function sqlPiece(node, tags) {
  const value = unwrapTs(node);
  if (value.type === "Literal" && typeof value.value === "string") return true;
  if (value.type === "TemplateLiteral") return true;
  if (value.type !== "TaggedTemplateExpression") return false;
  const tag = unwrapTs(value.tag);
  return tag?.type === "Identifier" && tags.has(tag.name);
}

function appendBuilder(call, methods, tags) {
  const callee = unwrapTs(call.callee);
  if (!callee || callee.type !== "MemberExpression" || callee.computed) return null;
  if (!methods.has(callee.property?.name)) return null;
  const object = unwrapTs(callee.object);
  if (!object || object.type !== "Identifier") return null;
  if (call.arguments.length !== 1 || !sqlPiece(call.arguments[0], tags)) return null;
  return object;
}

function findAppend(node, methods, tags) {
  let builder = null;
  const visit = (current) => {
    if (!current || builder || isFunction(current)) return;
    if (current.type === "CallExpression") {
      const object = appendBuilder(current, methods, tags);
      if (object) builder = { object, call: current };
    }
    for (const child of childNodes(current)) visit(child);
  };
  visit(node);
  return builder;
}

function readGuard(statement, methods, tags) {
  const selector = selectorOf(unwrapTs(statement.test));
  if (!selector) return null;
  const builder = findAppend(statement.consequent, methods, tags);
  if (!builder) return null;
  const test = unwrapTs(statement.test);
  const value = stringText(test.left) ?? stringText(test.right);
  return { node: statement, selector, builder: builder.object, call: builder.call, value };
}

function guardsOf(func, methods, tags) {
  const found = [];
  const visit = (node) => {
    if (!node || typeof node !== "object" || isFunction(node)) return;
    if (node.type === "IfStatement") {
      const guard = readGuard(node, methods, tags);
      if (guard) found.push(guard);
    }
    for (const child of childNodes(node)) visit(child);
  };
  visit(func.body);
  return found;
}

function variableId(variable) {
  return `${variable.name}:${variable.defs[0].node.range[0]}`;
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
      const groups = new Map();
      for (const guard of guardsOf(func, methods, tags)) {
        const selector = resolveVariable(guard.selector, context);
        const builder = resolveVariable(guard.builder, context);
        if (!selector || !builder) continue;
        const id = `${variableId(selector)}|${variableId(builder)}`;
        const group = groups.get(id) ?? [];
        group.push(guard);
        groups.set(id, group);
      }
      for (const group of groups.values()) {
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
