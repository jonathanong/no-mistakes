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

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow Vitest sequential test modifiers", recommended: false },
    schema: [],
    messages: { sequential: "Use parallel tests instead of .sequential." },
  },
  (context) => ({
    CallExpression(node) {
      if (node.callee.type !== "MemberExpression") return;
      if (propertyName(node.callee.property) !== "sequential") return;
      if (!TEST_NAMES.has(rootTestName(node.callee.object))) return;
      context.report({ node: node.callee, messageId: "sequential" });
    },
  }),
);
