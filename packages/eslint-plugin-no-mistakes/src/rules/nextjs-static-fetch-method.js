"use strict";

const { isFetchCall, isStaticString, literalString, rule } = require("../helpers");

function isMethodKey(property) {
  if (property.computed) return literalString(property.key) === "method";
  return (
    (property.key.type === "Identifier" && property.key.name === "method") ||
    (property.key.type === "Literal" && property.key.value === "method")
  );
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require static fetch() method option",
      recommended: true,
    },
    schema: [],
    messages: {
      dynamic:
        "fetch() method option must be a string literal or an expression-free template literal so it can be statically analyzed.",
    },
  },
  (context) => ({
    CallExpression(node) {
      if (!isFetchCall(node, context)) return;
      const opts = node.arguments[1];
      if (!opts || opts.type !== "ObjectExpression") return;
      const methodProp = opts.properties.findLast((p) => p.type === "Property" && isMethodKey(p));
      if (!methodProp) return;
      if (!isStaticString(methodProp.value)) {
        context.report({ node: methodProp.value, messageId: "dynamic" });
      }
    },
  }),
);
