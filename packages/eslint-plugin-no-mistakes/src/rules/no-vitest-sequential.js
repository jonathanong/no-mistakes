"use strict";

const { rule } = require("../helpers");

const TEST_NAMES = new Set(["test", "it", "describe"]);

function propertyName(node) {
  if (node.type === "Literal") return String(node.value);
  return node.name;
}

function rootTestName(node) {
  if (node?.type === "Identifier") return node.name;
  if (node?.type === "MemberExpression") return rootTestName(node.object);
  if (node?.type === "CallExpression") return rootTestName(node.callee);
  return null;
}

function hasSequentialOption(node) {
  return node.arguments
    .slice(1)
    .some(
      (argument) =>
        argument.type === "ObjectExpression" &&
        argument.properties.some(
          (property) =>
            property.type === "Property" &&
            propertyName(property.key) === "sequential" &&
            property.value.type === "Literal" &&
            property.value.value === true,
        ),
    );
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow Vitest sequential test modifiers", recommended: false },
    schema: [],
    messages: { sequential: "Use parallel tests instead of .sequential." },
  },
  (context) => ({
    CallExpression(node) {
      if (node.callee.type === "MemberExpression") {
        if (propertyName(node.callee.property) !== "sequential") return;
        if (!TEST_NAMES.has(rootTestName(node.callee.object))) return;
        context.report({ node: node.callee, messageId: "sequential" });
        return;
      }
      if (!TEST_NAMES.has(rootTestName(node.callee))) return;
      if (!hasSequentialOption(node)) return;
      context.report({ node: node.callee, messageId: "sequential" });
    },
  }),
);
