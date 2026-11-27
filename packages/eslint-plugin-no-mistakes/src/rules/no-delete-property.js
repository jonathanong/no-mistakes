"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");

function unwrapExpression(node) {
  let current = node;
  while (
    current &&
    ["ChainExpression", "TSAsExpression", "TSNonNullExpression", "TSTypeAssertion"].includes(
      current.type,
    )
  ) {
    current = current.expression;
  }
  return current;
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow deleting object properties", recommended: true },
    schema: [],
    messages: {
      delete:
        "Avoid deleting object properties. Use an omitted copy or an explicit nullable value so object shape changes stay traceable.",
    },
  },
  (context) => {
    const suppressed = ruleSuppression(context, "no-delete-property");
    return {
      UnaryExpression(node) {
        if (node.operator !== "delete") return;
        const argument = unwrapExpression(node.argument);
        if (argument?.type !== "MemberExpression" || suppressed(node)) return;
        context.report({ node, messageId: "delete" });
      },
    };
  },
);
