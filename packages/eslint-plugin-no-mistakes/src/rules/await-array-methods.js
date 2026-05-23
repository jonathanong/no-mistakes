"use strict";

const { callMethodName, rule } = require("../helpers");

const BANNED_METHODS = new Set(["sort", "toSorted", "every", "findIndex", "slice", "toSpliced"]);

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow awaiting synchronous array methods",
      recommended: false,
    },
    schema: [],
    messages: {
      awaited:
        "Do not await {{method}}(). This array method returns a synchronous value; remove await or await the async work explicitly.",
    },
  },
  (context) => ({
    AwaitExpression(node) {
      if (node.argument.type !== "CallExpression") return;
      const method = callMethodName(node.argument);
      if (!BANNED_METHODS.has(method)) return;
      context.report({ node, messageId: "awaited", data: { method } });
    },
  }),
);
