"use strict";

const { fetchInvocation, isStaticString, rule } = require("../helpers");

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require static fetch() URL arguments",
      recommended: true,
    },
    schema: [],
    messages: {
      dynamic:
        "fetch() URL must be a string literal or an expression-free template literal so it can be statically analyzed.",
    },
  },
  (context) => ({
    CallExpression(node) {
      const invocation = fetchInvocation(node, context);
      if (!invocation) return;
      const url = invocation.arguments?.[0];
      if (!isStaticString(url)) {
        context.report({ node: url ?? node, messageId: "dynamic" });
      }
    },
  }),
);
