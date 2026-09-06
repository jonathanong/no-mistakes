"use strict";

const { rule } = require("../helpers");

const TRANSPARENT_EXPRESSION_TYPES = new Set([
  "ChainExpression",
  "TSAsExpression",
  "TSInstantiationExpression",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
]);

function unwrapExpression(expression) {
  let current = expression;
  while (TRANSPARENT_EXPRESSION_TYPES.has(current.type)) {
    current = current.expression;
  }
  return current;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow const aliases for differently named values",
      recommended: true,
    },
    schema: [],
    messages: {
      alias:
        "Do not create a differently named const alias. Use the original name directly so readers and tooling can trace symbols without indirection.",
    },
  },
  (context) => ({
    VariableDeclaration(node) {
      if (node.kind !== "const") return;
      for (const declarator of node.declarations) {
        if (declarator.id.type !== "Identifier" || !declarator.init) continue;
        const initializer = unwrapExpression(declarator.init);
        if (initializer.type === "Identifier" && declarator.id.name !== initializer.name) {
          context.report({ node: declarator, messageId: "alias" });
        }
      }
    },
  }),
);
