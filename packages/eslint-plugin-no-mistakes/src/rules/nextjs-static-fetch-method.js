"use strict";

const { fetchInvocation, isStaticString, literalString, rule } = require("../helpers");

const { unwrapExpression } = require("./async-ast");
const { argumentAt } = require("./logical-call");

function effectiveMethod(object) {
  for (let index = object.properties.length - 1; index >= 0; index -= 1) {
    const property = object.properties[index];
    if (property.type === "SpreadElement") {
      const value = unwrapExpression(property.argument);
      const method = value.type === "ObjectExpression" ? effectiveMethod(value) : property;
      if (method) return method;
      continue;
    }
    const key = unwrapExpression(property.key);
    const name = property.computed ? literalString(key) : (key.name ?? key.value);
    if (name === "method") return property.value;
    if (property.computed && name === null && key.type !== "Literal") return key;
  }
  return null;
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
      const invocation = fetchInvocation(node, context);
      if (!invocation) return;
      const optionsArg = argumentAt(invocation.arguments, 1);
      const opts = unwrapExpression(optionsArg);
      const method = opts?.type === "ObjectExpression" ? effectiveMethod(opts) : null;
      if (optionsArg === null || (method && !isStaticString(method))) {
        context.report({ node: method ?? node, messageId: "dynamic" });
      }
    },
  }),
);
