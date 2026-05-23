"use strict";

const { rule } = require("../helpers");

const TEST_NAMES = new Set(["test", "it", "describe"]);

function propertyName(node) {
  if (node.type === "Literal") return String(node.value);
  return node.name;
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow Vitest sequential test modifiers", recommended: false },
    schema: [],
    messages: { sequential: "Use parallel tests instead of .sequential." },
  },
  (context) => ({
    MemberExpression(node) {
      if (propertyName(node.property) !== "sequential") return;
      if (node.object.type !== "Identifier" || !TEST_NAMES.has(node.object.name)) return;
      context.report({ node, messageId: "sequential" });
    },
  }),
);
