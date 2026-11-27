"use strict";

const { fetchInvocation, isStaticString, rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");

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
  (context) => {
    const suppressed = ruleSuppression(context, "nextjs-static-fetch-url");
    return {
      CallExpression(node) {
        const invocation = fetchInvocation(node, context);
        if (!invocation) return;
        const target = invocation.arguments?.[0] ?? node;
        if (!isStaticString(invocation.arguments?.[0]) && !suppressed(target)) {
          context.report({ node: target, messageId: "dynamic" });
        }
      },
    };
  },
);
